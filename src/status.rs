//! status：三态对照（locked/installed/path）与 daily 日常更新编排。
//! 语义基准 ohmyenv.ps1 status（134-162 行）与 daily（164-208 行）：
//! - status：msi 与 official（exe 含 %）走环境展开，其余 Join EnvRoot；installed 实跑 exe 探测；
//!   path 查用户 PATH 原始值（不展开比较，大小写不敏感）；按 核心基础/扩展 两层分组。
//! - daily：逐工具 --latest 解析；同版本跳过；同主版本（首段比较）自动安装+注册+锁定；
//!   跨主版本保留待人工确认（--include-breaking 强制）；报告追加写 <EnvRoot>\logs\update-daily.log。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::catalog::Catalog;
use crate::envpath;
use crate::install::{install_tool, InstallOptions};
use crate::resolve::{resolve_tool, ResolveOptions};
use crate::toolver;

/// status 三态行：locked=pin version，installed=实跑探测，path=用户 PATH 是否含 bin。
#[derive(Clone)]
pub struct StatusRow {
    pub name: String,
    pub category: String,
    pub locked: Option<String>,
    pub installed: Option<String>,
    pub path: bool,
    /// 本平台无 effective exe（平台不适用，如 shellcheck 在 Windows）时为 None，渲染为 -。
    pub exe: Option<PathBuf>,
}

/// 收集全部工具三态（按 catalog 书写顺序）。
pub fn collect_status(cat: &Catalog, env_root: &Path) -> Result<Vec<StatusRow>, String> {
    collect_status_with(cat, env_root, |_| Ok(()))
}

/// 流式收集：每探完一个工具立即回调 on_row（status 命令逐行输出的关键——
/// 探测要逐工具拉起 `--version` 子进程，整批探完才打印会被感知为卡顿）；
/// verify 等纯数据消费方传空回调走 collect_status。
pub fn collect_status_with<F: FnMut(&StatusRow) -> Result<(), String>>(
    cat: &Catalog,
    env_root: &Path,
    mut on_row: F,
) -> Result<Vec<StatusRow>, String> {
    let mut rows = Vec::new();
    for name in &cat.order {
        let def = cat.tool(name)?;
        // 平台不适用（无本平台 exe，如 shellcheck 在 Windows 只有 linux 字段）：如实空态行，不探测
        if !toolver::platform_managed(def) {
            let row = StatusRow {
                name: name.clone(),
                category: def.category.clone().unwrap_or_default(),
                locked: def.pin_version().map(str::to_string),
                installed: None,
                path: false,
                exe: None,
            };
            on_row(&row)?;
            rows.push(row);
            continue;
        }
        let exe = toolver::exe_path(def, env_root)?;
        let installed = toolver::installed_version(&exe, name);
        // vsbuild 特判：PATH 在机器级（HKLM）而非用户 PATH，按 MSBuild 与 cl 目录全在判定
        if crate::vsbuild::is_vsbuild(def) {
            let dirs = crate::vsbuild::machine_path_dirs(env_root);
            let in_path = !dirs.is_empty()
                && dirs
                    .iter()
                    .all(|d| crate::platform::machine_path_contains(d).unwrap_or(false));
            let row = StatusRow {
                name: name.clone(),
                category: def.category.clone().unwrap_or_default(),
                locked: def.pin_version().map(str::to_string),
                installed,
                path: in_path,
                exe: Some(exe),
            };
            on_row(&row)?;
            rows.push(row);
            continue;
        }
        let is_official = toolver::is_official(def);
        // bin 目录：official 取 exe 上一级（展开后），其余 EnvRoot\bin 字段；无 bin 则 path=false
        let in_path = match (def.bin(), is_official) {
            (Some(_), true) => {
                let bin = exe
                    .parent()
                    .ok_or_else(|| format!("{name} official exe 无法取上级目录"))?;
                envpath::user_path_contains(bin)?
            }
            (Some(b), false) => {
                let raw = crate::platform::expand_install_path(b);
                // 相对路径拼到 EnvRoot 下（Windows 名录是相对 bin；Linux 多为 ~/ 绝对）
                let bin = if raw.is_absolute() {
                    raw
                } else {
                    env_root.join(raw)
                };
                envpath::user_path_contains(&bin)?
            }
            (None, _) => false,
        };
        let row = StatusRow {
            name: name.clone(),
            category: def.category.clone().unwrap_or_default(),
            locked: def.pin_version().map(str::to_string),
            installed,
            path: in_path,
            exe: Some(exe),
        };
        on_row(&row)?;
        rows.push(row);
    }
    Ok(rows)
}

/// 分层：extras → 扩展工具，其余 → 核心基础工具（对齐 pwsh status 分组）。
pub fn tier_of(category: &str) -> &'static str {
    if category == "extras" {
        "扩展工具"
    } else {
        "核心基础工具"
    }
}

/// 分类中文名（对齐 helpers.ps1 ToolCategories）。
pub fn category_label(category: &str) -> &str {
    match category {
        "key" => "密钥",
        "agent" => "智能体环境",
        "project" => "项目管理",
        "base" => "基础工具",
        "extras" => "扩展工具",
        other => other,
    }
}

/// 同主版本判定：首段（. 分隔）相等（对齐 pwsh daily 的 ($v -split '\.')[0] 比较）。
pub fn is_same_major(a: &str, b: &str) -> bool {
    a.split('.').next() == b.split('.').next()
}

/// daily 行结果。
pub struct DailyRow {
    pub tool: String,
    pub action: &'static str,
    pub from: String,
    pub to: String,
}

/// daily 汇总计数。
#[derive(Default)]
pub struct DailyOutcome {
    pub updated: u32,
    pub held: u32,
    pub fresh: u32,
}

/// 日常更新检查：返回逐工具行与汇总；报告追加写日志（含 dry-run 预览）。
pub fn run_daily(
    cat: &Catalog,
    env_root: &Path,
    dry_run: bool,
    include_breaking: bool,
) -> Result<(Vec<DailyRow>, DailyOutcome), String> {
    run_daily_with(cat, env_root, dry_run, include_breaking, |_| {})
}

/// run_daily 的流式形态：每工具的判定行算完即经回调输出（解析逐工具走网络，
/// 整批返回再打印会被感知为卡顿），返回全量行与汇总供退出码判定。
pub fn run_daily_with<F>(
    cat: &Catalog,
    env_root: &Path,
    dry_run: bool,
    include_breaking: bool,
    mut on_row: F,
) -> Result<(Vec<DailyRow>, DailyOutcome), String>
where
    F: FnMut(&DailyRow),
{
    let log_dir = env_root.join("logs");
    fs::create_dir_all(&log_dir)
        .map_err(|e| format!("创建日志目录失败: {}: {e}", log_dir.display()))?;
    let log_file = log_dir.join("update-daily.log");

    let mut report = vec![format!("===== 日常更新检查 {} =====", now_stamp())];
    let mut rows = Vec::new();
    let mut outcome = DailyOutcome::default();
    let ropts = ResolveOptions {
        latest: true,
        ..ResolveOptions::default()
    };
    let iopts = InstallOptions {
        register_path: true,
        update_lock: true,
        force: false,
    };
    let mut put = |row: DailyRow| {
        on_row(&row);
        rows.push(row);
    };

    for name in &cat.order {
        let def = cat.tool(name)?;
        // 平台不适用（无本平台 exe）：跳过，不解析不安装
        if !toolver::platform_managed(def) {
            eprintln!("[跳过] {name}: 当前平台不适用");
            report.push(format!("[跳过] {name}: 当前平台不适用"));
            continue;
        }
        // 版本锁定（hold）：日常更新不碰
        if def.is_held() {
            eprintln!("[跳过] {name}: hold 锁定不自动更新");
            report.push(format!("[跳过] {name}: hold 锁定不自动更新"));
            continue;
        }
        // evergreen 引导器条目不走日常更新（无远端版本可解析，install 幂等即更新语义）
        if crate::vsbuild::is_vsbuild(def) || crate::rustup::is_rustup(def) {
            eprintln!("[跳过] {name}: evergreen 引导器不走日常更新");
            report.push(format!("[跳过] {name}: evergreen 引导器不走日常更新"));
            continue;
        }
        let r = resolve_tool(name, def, &ropts)?;
        let cur = def.pin_version().map(str::to_string).unwrap_or_default();
        if r.version == cur {
            eprintln!("[跳过] {name}: {cur} 已是最新");
            report.push(format!("[跳过] {name}: {cur} 已是最新"));
            put(DailyRow {
                tool: name.clone(),
                action: "skipped",
                from: cur,
                to: r.version,
            });
            outcome.fresh += 1;
            continue;
        }
        if include_breaking || is_same_major(&cur, &r.version) {
            if dry_run {
                eprintln!("[预览] {name}: {cur} -> {}（同主版本，无影响）", r.version);
                report.push(format!("[预览] {name}: {cur} -> {}（同主版本）", r.version));
                put(DailyRow {
                    tool: name.clone(),
                    action: "preview",
                    from: cur,
                    to: r.version.clone(),
                });
            } else {
                eprintln!("[更新] {name}: {cur} -> {}（同主版本，无影响）", r.version);
                report.push(format!("[更新] {name}: {cur} -> {}（同主版本）", r.version));
                install_tool(cat, env_root, name, &r, &iopts)?;
                put(DailyRow {
                    tool: name.clone(),
                    action: "updated",
                    from: cur,
                    to: r.version.clone(),
                });
            }
            outcome.updated += 1;
        } else {
            eprintln!(
                "[保留] {name}: {cur} -> {}（跨主版本，需人工确认；--include-breaking 强制更新）",
                r.version
            );
            report.push(format!(
                "[保留] {name}: {cur} -> {}（跨主版本，需人工确认）",
                r.version
            ));
            put(DailyRow {
                tool: name.clone(),
                action: "held",
                from: cur,
                to: r.version.clone(),
            });
            outcome.held += 1;
        }
    }

    let summary = format!(
        "===== 汇总: {} {} | 保留 {} | 已最新 {} =====",
        if dry_run { "预览" } else { "更新" },
        outcome.updated,
        outcome.held,
        outcome.fresh
    );
    eprintln!("{summary}");
    report.push(summary);
    append_log(&log_file, &report)?;
    eprintln!("[LOG] {}", log_file.display());
    Ok((rows, outcome))
}

/// 报告追加写日志（UTF-8 无 BOM，对齐 pwsh Add-Content -Encoding utf8）。
pub fn append_log(log_file: &Path, lines: &[String]) -> Result<(), String> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|e| format!("打开日志失败: {}: {e}", log_file.display()))?;
    for line in lines {
        writeln!(f, "{line}").map_err(|e| format!("写日志失败: {}: {e}", log_file.display()))?;
    }
    Ok(())
}

/// 时间戳：UTC（无本地时区依赖；pwsh 用本地时间 Get-Date，此处为无依赖取舍，仅用于日志抬头）。
pub fn now_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// days-from-epoch 转年月日（Howard Hinnant civil_from_days 算法）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 分层与分类标签_对齐pwsh映射表() {
        // 期望值来源：helpers.ps1 ToolNames 顺序与 ToolCategories 映射
        assert_eq!(tier_of("key"), "核心基础工具");
        assert_eq!(tier_of("agent"), "核心基础工具");
        assert_eq!(tier_of("project"), "核心基础工具");
        assert_eq!(tier_of("base"), "核心基础工具");
        assert_eq!(tier_of("extras"), "扩展工具");
        assert_eq!(category_label("key"), "密钥");
        assert_eq!(category_label("agent"), "智能体环境");
        assert_eq!(category_label("project"), "项目管理");
        assert_eq!(category_label("base"), "基础工具");
        assert_eq!(category_label("extras"), "扩展工具");
    }

    #[test]
    fn 同主版本_首段比较() {
        assert!(is_same_major("1.3.1", "1.4.0"));
        assert!(is_same_major("2.55.0.windows.4", "2.56.0"));
        assert!(!is_same_major("1.3.1", "2.0.0"));
        assert!(!is_same_major("10.0.400", "9.0.0"));
    }

    #[test]
    fn civil日期换算_已知锚点() {
        // 锚点：day 0 = 1970-01-01；day 19723 = 2024-01-01（独立来源：Unix epoch 定义）
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19723), (2024, 1, 1));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
    }

    #[test]
    fn 日志追加_多段累计() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let log = dir.path().join("update-daily.log");
        append_log(&log, &["第一段".to_string()])?;
        append_log(&log, &["第二段".to_string()])?;
        let text = fs::read_to_string(&log).map_err(|e| e.to_string())?;
        assert_eq!(text, "第一段\n第二段\n", "追加写应累计而非覆盖");
        assert!(!text.starts_with('\u{FEFF}'), "应无 BOM");
        Ok(())
    }

    /// 平台不适用工具（effective exe 缺失）出空态行不报错。
    /// 现实对应 shellcheck（仅 linux 字段）在 Windows 触发；用全平台无 exe 的 ghost 做可移植验证。
    #[test]
    fn status_平台不适用工具出空态行() -> Result<(), String> {
        let toml = "[tools.ghost]\ncategory = \"extras\"\nlinux_dir = \"~/.local/bin\"\n";
        let cat = Catalog::parse(toml, PathBuf::from("synthetic.toml"))?;
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let rows = collect_status(&cat, dir.path())?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "ghost");
        assert!(rows[0].installed.is_none(), "空态不探测");
        assert!(!rows[0].path);
        assert!(rows[0].exe.is_none(), "本平台无 exe 渲染为 -");
        assert!(!toolver::platform_managed(cat.tool("ghost")?));
        Ok(())
    }
}

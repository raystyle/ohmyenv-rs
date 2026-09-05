//! doctor：部署异常诊断（2026-09-02 用户需求：检测部署的错误异常）。
//! 与 verify 的分工：verify 答「部署域维度是否 PASS」（pin 对齐断言），doctor 答
//! 「环境里有哪些部署错误与异味」——版本漂移、探测失败、PATH 死链与重复、
//! pin/sha 缺失、缓存孤儿、EnvRoot 不可写等，输出 check=OK/WARN/FAIL 逐项行，
//! FAIL 即 exit 1（WARN 不拦退出）。

use std::path::Path;

use crate::catalog::Catalog;
use crate::status::{self, StatusRow};
use crate::toolver;

/// 单项诊断结果。detail 为该项的明细（stderr 人称提示用）。
pub struct DoctorRow {
    pub name: &'static str,
    pub status: &'static str, // "OK" | "WARN" | "FAIL"
    pub detail: Vec<String>,
}

// ===================== D07 三层诊断（2026-09-05）：系统 / agent / 依赖 =====================
// doctor 升核心命令：先答「本机是什么系统、装了 agent 没有、agent 依赖装了没有」，
// 再出环境错误 check 节（十项）。前三层是事实陈述与缺口计数（缺口走 WARN，不拦退出，
// 检测驱动安装）；FAIL 语义仍专属 check 节的环境错误。

/// 系统层事实（非诊断，不占 OK/WARN/FAIL 三态）。
pub struct SysFacts {
    pub os: &'static str,
    pub arch: &'static str,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
}

pub fn system_facts() -> SysFacts {
    SysFacts {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        avx: x86_feature("avx"),
        avx2: x86_feature("avx2"),
        avx512f: x86_feature("avx512f"),
    }
}

#[cfg(target_arch = "x86_64")]
fn x86_feature(f: &str) -> bool {
    match f {
        "avx" => std::arch::is_x86_feature_detected!("avx"),
        "avx2" => std::arch::is_x86_feature_detected!("avx2"),
        _ => std::arch::is_x86_feature_detected!("avx512f"),
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn x86_feature(_f: &str) -> bool {
    false // 非 x86_64（如 darwin-arm64）指令集检测不适用，如实 false（oma caps 同口径）
}

/// agent 层健康：二进制在位、版本对 pin、token 配置可用（D07 裁定「只负责二进制是否
/// 安装好、版本正确、多余检测下是否 token 配置可用，不取设置」；登录态细节归 oma agents 域）。
pub struct AgentHealth {
    pub name: String,
    pub binary: &'static str, // "ok" | "missing"
    pub version: Option<String>,
    pub locked: Option<String>,
    pub drift: bool,
    pub token: &'static str, // "ok" | "missing" | "na"
}

pub fn agent_health(srows: &[StatusRow]) -> Vec<AgentHealth> {
    srows
        .iter()
        .filter(|r| r.category == "agent")
        .map(|r| AgentHealth {
            name: r.name.clone(),
            binary: if r.installed.is_some() {
                "ok"
            } else {
                "missing"
            },
            version: r.installed.clone(),
            locked: r.locked.clone(),
            drift: matches!(
                (&r.installed, &r.locked),
                (Some(installed), Some(locked)) if installed != locked
            ),
            token: token_state(&r.name),
        })
        .collect()
}

/// token 配置可用性：只探凭据文件在位与最简形态，不读设置、不验 scope、不取内容细节、
/// **不读环境变量**（用户裁定 2026-09-05：环境变量可能承载隐私，doctor 探测面不碰）。
/// 判据实证（2026-09-05 本机）：
/// - grok：`~/.grok/auth.json` 在位非空（oma S026 同源判据）
/// - kimi：`~/.kimi-code/credentials/kimi-code.json` 的 access_token 非空（本机实证无
///   hasToken 字段，oma S026 的 hasToken 判据是另一凭据形态）
/// - claude / codex：本机实际在用但凭据形态未实证（无稳定文件判据），如实 na，待实证后补
fn token_state(name: &str) -> &'static str {
    let Some(home) = dirs::home_dir() else {
        return "na";
    };
    match name {
        "grok" => {
            let p = home.join(".grok").join("auth.json");
            match std::fs::metadata(&p).map(|m| m.len() > 0) {
                Ok(true) => "ok",
                Ok(false) => "missing",
                Err(_) => "missing",
            }
        }
        "kimi" => {
            let p = home
                .join(".kimi-code")
                .join("credentials")
                .join("kimi-code.json");
            let Ok(text) = std::fs::read_to_string(&p) else {
                return "missing";
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
                return "missing";
            };
            match v.get("access_token") {
                Some(serde_json::Value::String(s)) if !s.is_empty() => "ok",
                _ => "missing",
            }
        }
        _ => "na",
    }
}

/// 依赖层分组统计（九类 taxonomy 逐组：工具数、缺失数、漂移数）。
pub struct DepGroupStat {
    pub category: String,
    pub label: &'static str,
    pub tools: usize,
    pub missing: usize,
    pub drift: usize,
}

pub fn dep_group_stats(srows: &[StatusRow]) -> Vec<DepGroupStat> {
    status::GROUPS
        .iter()
        .filter_map(|(cat, label)| {
            let group: Vec<&StatusRow> = srows.iter().filter(|r| &r.category == cat).collect();
            if group.is_empty() {
                return None; // 空组不输出（如 mac 侧无 service 组）
            }
            Some(DepGroupStat {
                category: cat.to_string(),
                label,
                tools: group.len(),
                missing: group.iter().filter(|r| r.installed.is_none()).count(),
                drift: group
                    .iter()
                    .filter(|r| {
                        matches!(
                            (&r.installed, &r.locked),
                            (Some(installed), Some(locked)) if installed != locked
                        )
                    })
                    .count(),
            })
        })
        .collect()
}

/// 跑全部诊断项，流式形态：每项算完即经回调输出（三态采集期间先出 envroot 项，
/// 采集完成即连出五个派生项；返回全量行供汇总）。
pub fn run_doctor_with<F>(
    cat: &Catalog,
    env_root: &Path,
    on_row: F,
) -> Result<Vec<DoctorRow>, String>
where
    F: FnMut(&DoctorRow),
{
    let srows = status::collect_status(cat, env_root)?;
    run_doctor_with_status(cat, env_root, &srows, on_row)
}

/// 同 run_doctor_with，但复用调用方已采集的三态行（cmd_doctor 三层诊断共用一次采集，
/// 避免 41 工具版本探针跑两遍）。
pub fn run_doctor_with_status<F>(
    cat: &Catalog,
    env_root: &Path,
    srows: &[StatusRow],
    mut on_row: F,
) -> Result<Vec<DoctorRow>, String>
where
    F: FnMut(&DoctorRow),
{
    let mut rows: Vec<DoctorRow> = Vec::new();
    let mut put = |row: DoctorRow| {
        on_row(&row);
        rows.push(row);
    };

    // 1. EnvRoot 可写（装不进东西是一切部署错误之源）
    put(check_envroot_writable(env_root));

    // 2-7. 三态派生项（复用调用方采集的三态行）
    put(check_version_drift(srows));
    put(check_probe_fail(srows));
    put(check_not_on_path(cat, srows, env_root));
    put(check_pin_missing(cat, srows));
    put(check_sha_missing(cat, srows));

    // 8-9. 用户 PATH 卫生（死链仅报 EnvRoot 域内，系统条目不掺和）
    let entries = crate::platform::user_path_entries().unwrap_or_default();
    put(check_dead_path_entries(&entries, env_root));
    put(check_dup_path_entries(&entries));

    // 10. 缓存孤儿（下载缓存里已无任何 pin 指向的资产）
    put(check_cache_orphans(cat, env_root));
    Ok(rows)
}

/// 收集全量形态（run_doctor_with 的空回调兼容口）。
pub fn run_doctor(cat: &Catalog, env_root: &Path) -> Result<Vec<DoctorRow>, String> {
    run_doctor_with(cat, env_root, |_| {})
}

/// 汇总：FAIL 与 WARN 计数。
pub fn summarize(rows: &[DoctorRow]) -> (usize, usize, Vec<String>, Vec<String>) {
    let fails = rows
        .iter()
        .filter(|r| r.status == "FAIL")
        .map(|r| r.name.to_string())
        .collect::<Vec<_>>();
    let warns = rows
        .iter()
        .filter(|r| r.status == "WARN")
        .map(|r| r.name.to_string())
        .collect::<Vec<_>>();
    (fails.len(), warns.len(), fails, warns)
}

fn check_envroot_writable(env_root: &Path) -> DoctorRow {
    let probe = env_root.join(".ome-doctor-probe");
    let ok = std::fs::create_dir_all(env_root)
        .ok()
        .and_then(|_| std::fs::write(&probe, b"1").ok())
        .is_some();
    let _ = std::fs::remove_file(&probe);
    DoctorRow {
        name: "envroot-writable",
        status: if ok { "OK" } else { "FAIL" },
        detail: if ok {
            vec![]
        } else {
            vec![format!("EnvRoot 不可写: {}", env_root.display())]
        },
    }
}

/// 版本漂移：locked 已设但 installed 缺失或不等——部署错误的核心形态。
fn check_version_drift(srows: &[StatusRow]) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        let Some(locked) = &r.locked else { continue };
        match &r.installed {
            None => detail.push(format!("{}: pin {locked} 但未装/探测不到", r.name)),
            Some(inst) if inst != locked => {
                detail.push(format!("{}: pin {locked} 实装 {inst}", r.name))
            }
            _ => {}
        }
    }
    DoctorRow {
        name: "version-drift",
        status: if detail.is_empty() { "OK" } else { "FAIL" },
        detail,
    }
}

/// 探测失败：exe 在位但版本读不出（正则缺项或二进制损坏）。
fn check_probe_fail(srows: &[StatusRow]) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        if r.exe.is_some() && r.installed.is_none() {
            detail.push(format!(
                "{}: exe 在位但版本探测失败（检查 toolver 正则或二进制完整性）",
                r.name
            ));
        }
    }
    DoctorRow {
        name: "probe-fail",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
}

/// 装而未上 PATH：已装且定义了 bin 但用户 PATH 不含（Windows）。
fn check_not_on_path(cat: &Catalog, srows: &[StatusRow], env_root: &Path) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        if r.installed.is_none() || r.path || r.exe.is_none() {
            continue;
        }
        // 只查有 bin 字段的绿色/官方布局工具；msi 型（pwsh）与机器级（vsbuild）PATH 由安装器管，跳过
        let Ok(def) = cat.tool(&r.name) else { continue };
        if def.bin().is_none() || crate::vsbuild::is_vsbuild(def) {
            continue;
        }
        let _ = env_root;
        detail.push(format!(
            "{}: 已装但不在用户 PATH（ome deploy {} 可补）",
            r.name, r.name
        ));
    }
    DoctorRow {
        name: "not-on-path",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
}

/// 本平台在管但未 pin：install 不带选项会失败的前置异味（evergreen 条目如 vsbuild/rust 无 pin 属设计，排除）。
fn check_pin_missing(cat: &Catalog, srows: &[StatusRow]) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        if r.exe.is_none() {
            continue; // 平台不适用不算
        }
        let Ok(def) = cat.tool(&r.name) else { continue };
        if !toolver::platform_managed(def)
            || r.locked.is_some()
            || crate::vsbuild::is_vsbuild(def)
            || crate::rustup::is_rustup(def)
            || crate::selfupdate::is_ome_self(def)
        {
            continue;
        }
        detail.push(format!(
            "{}: 本平台在管但未 pin（ome pin {} --version <ver>）",
            r.name, r.name
        ));
    }
    DoctorRow {
        name: "pin-missing",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
}

/// 已 pin 但 sha 缺失（校验降级到官方源；官方源也缺则裸下载）。
/// uv-git 型（uv tool install git 源）无下载资产、sha 不适用，排除。
fn check_sha_missing(cat: &Catalog, srows: &[StatusRow]) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        if r.locked.is_none() {
            continue;
        }
        let Ok(def) = cat.tool(&r.name) else { continue };
        if sha_missing_for_tool(def) {
            detail.push(format!("{}: pin 无 sha256（重装一次可回填）", r.name));
        }
    }
    DoctorRow {
        name: "sha-missing",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
}

/// 单工具 sha 缺失判定（纯函数可测）：已 pin 资产型但无 sha；uv-git 无资产语义除外。
fn sha_missing_for_tool(def: &crate::catalog::Tool) -> bool {
    def.pin_sha256().is_none() && def.extract() != Some("uv-git")
}

/// PATH 死链：EnvRoot 域内的用户 PATH 条目指向不存在的目录。
fn check_dead_path_entries(entries: &[String], env_root: &Path) -> DoctorRow {
    let root = env_root.display().to_string().to_lowercase();
    let mut detail = Vec::new();
    for e in entries {
        let t = e.trim();
        if t.is_empty() || !t.to_lowercase().starts_with(&root) {
            continue;
        }
        let expanded = crate::platform::expand_env_vars(t);
        if !Path::new(&expanded).is_dir() {
            detail.push(format!("死链: {t}"));
        }
    }
    DoctorRow {
        name: "dead-path-entries",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
}

/// PATH 重复条目（展开后大小写不敏感比较）。
fn check_dup_path_entries(entries: &[String]) -> DoctorRow {
    let mut seen: Vec<String> = Vec::new();
    let mut detail = Vec::new();
    for e in entries {
        let key = crate::platform::expand_env_vars(e.trim())
            .trim_end_matches(['\\', '/'])
            .to_lowercase();
        if key.is_empty() {
            continue;
        }
        let cur = e.trim().to_string();
        if seen.contains(&key) && !detail.iter().any(|d: &String| d.contains(&cur)) {
            detail.push(format!("重复: {cur}"));
        }
        seen.push(key);
    }
    DoctorRow {
        name: "dup-path-entries",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
}

/// 缓存孤儿：cache 下已无任何 pin 指向的资产文件。
/// 派生资产（专用模块的引导器/插件与自升级通道资产——被消费但不属任何 pin）放行不算孤儿。
fn check_cache_orphans(cat: &Catalog, env_root: &Path) -> DoctorRow {
    let cache = env_root.join("cache");
    let Ok(rd) = std::fs::read_dir(&cache) else {
        return DoctorRow {
            name: "cache-orphans",
            status: "OK",
            detail: vec![],
        };
    };
    let pinned: Vec<String> = cat
        .order
        .iter()
        .filter_map(|n| {
            cat.tools
                .get(n)
                .map(|t| t.pin_asset().unwrap_or("").to_string())
        })
        .filter(|a| !a.is_empty())
        .collect();
    let bootstraps: Vec<String> = cat
        .order
        .iter()
        .filter_map(|n| cat.tools.get(n).and_then(|t| t.bootstrap_asset()))
        .map(str::to_string)
        .collect();
    let mut detail = Vec::new();
    let mut bytes = 0u64;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_asset_like = name.ends_with(".zip")
            || name.ends_with(".tar.gz")
            || name.ends_with(".tar.xz")
            || name.ends_with(".exe")
            || name.ends_with(".msi");
        if is_asset_like && !pinned.contains(&name) && !is_derived_asset(&name, &bootstraps) {
            if let Ok(md) = entry.metadata() {
                bytes += md.len();
            }
            detail.push(name);
        }
    }
    let status = if detail.is_empty() { "OK" } else { "WARN" };
    if !detail.is_empty() {
        let mb = bytes as f64 / 1024.0 / 1024.0;
        detail.insert(
            0,
            format!(
                "{} 个孤儿资产共 {mb:.1} MB（无 pin 指向，可清）",
                detail.len()
            ),
        );
    }
    DoctorRow {
        name: "cache-orphans",
        status,
        detail,
    }
}

/// 派生资产判定（纯函数可测）：catalog 声明的 bootstrap 资产（如 7z 的 7zr.exe）、
/// 专用安装模块的引导器（vs_buildtools / rustup-init）、docker compose 插件、
/// ome 自升级通道资产——均被安装链消费但不属任何 pin。
fn is_derived_asset(name: &str, bootstraps: &[String]) -> bool {
    bootstraps.iter().any(|b| b == name)
        || name == crate::vsbuild::BOOTSTRAPPER
        || name == crate::rustup::INIT_EXE
        || name.starts_with("docker-compose-")
        || name.starts_with("ome-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, category: &str, locked: Option<&str>, installed: Option<&str>) -> StatusRow {
        StatusRow {
            name: name.to_string(),
            category: category.to_string(),
            locked: locked.map(str::to_string),
            installed: installed.map(str::to_string),
            path: false,
            exe: None,
        }
    }

    /// agent 层健康（D07）：drift 只在双值不等时真；missing 由 installed 缺失决定。
    #[test]
    fn agent健康_drift与missing判定() {
        let srows = vec![
            row("claude", "agent", Some("2.1.251"), Some("2.1.246")),
            row("grok", "agent", Some("1.0.13"), Some("1.0.13")),
            row("kimi", "agent", Some("0.39.1"), None),
            row("jq", "cli", Some("1.8.2"), Some("1.8.2")),
        ];
        let hs = agent_health(&srows);
        assert_eq!(hs.len(), 3, "只取 agent 类");
        assert!(hs[0].drift, "2.1.246 != 2.1.251");
        assert!(!hs[1].drift, "同版不漂移");
        assert_eq!(hs[2].binary, "missing");
        assert!(hs[2].version.is_none());
    }

    /// 依赖分组统计（D07）：按九类归类计数；空组不出；missing/drift 计数与组内一致。
    #[test]
    fn 依赖分组_计数与空组过滤() {
        let srows = vec![
            row("claude", "agent", Some("2.1.251"), Some("2.1.246")),
            row("kimi", "agent", Some("0.39.1"), None),
            row("jq", "cli", Some("1.8.2"), Some("1.8.2")),
            row("gh", "cli", None, None),
        ];
        let gs = dep_group_stats(&srows);
        let agent = gs.iter().find(|g| g.category == "agent").unwrap();
        assert_eq!((agent.tools, agent.missing, agent.drift), (2, 1, 1));
        let cli = gs.iter().find(|g| g.category == "cli").unwrap();
        assert_eq!((cli.tools, cli.missing, cli.drift), (2, 1, 0));
        assert!(!gs.iter().any(|g| g.category == "runtime"), "空组不出");
    }

    /// sha 缺失判定：普通资产型缺 sha 命中；uv-git（无下载资产）豁免；已有 sha 不命中。
    #[test]
    fn sha缺失判定_uvgit豁免() {
        let plain = crate::catalog::Tool {
            extract: Some("zip".to_string()),
            ..Default::default()
        };
        assert!(sha_missing_for_tool(&plain));
        let uv_git = crate::catalog::Tool {
            extract: Some("uv-git".to_string()),
            ..Default::default()
        };
        assert!(!sha_missing_for_tool(&uv_git), "uv-git 无 sha 语义不告警");
        #[cfg(windows)]
        {
            let with_sha = crate::catalog::Tool {
                extract: Some("zip".to_string()),
                sha256: Some("ABCD".to_string()),
                ..Default::default()
            };
            assert!(!sha_missing_for_tool(&with_sha));
        }
    }

    /// 派生资产判定：bootstrap 清单、模块引导器、compose/ome 前缀族放行；真孤儿命中。
    #[test]
    fn 派生资产判定_白名单与前缀族() {
        let bootstraps = vec!["7zr.exe".to_string()];
        assert!(is_derived_asset("7zr.exe", &bootstraps));
        assert!(is_derived_asset(crate::vsbuild::BOOTSTRAPPER, &bootstraps));
        assert!(is_derived_asset(crate::rustup::INIT_EXE, &bootstraps));
        assert!(is_derived_asset("docker-compose-v5.5.0.exe", &bootstraps));
        assert!(is_derived_asset(
            "ome-x86_64-pc-windows-msvc.exe",
            &bootstraps
        ));
        assert!(
            !is_derived_asset("claude-win32-x64.zip", &bootstraps),
            "真孤儿不放行"
        );
        assert!(!is_derived_asset(
            "reader-v0.1.0-x86_64-pc-windows-msvc.zip",
            &bootstraps
        ));
    }

    /// 死链判定：EnvRoot 域内不存在目录命中；域外与存在目录不命中。
    #[test]
    fn 死链判定_envroot域内() {
        let dir = tempfile::tempdir().expect("临时目录");
        let root = dir.path().join("env");
        std::fs::create_dir_all(root.join("jq")).expect("建目录");
        let entries = vec![
            root.join("jq").display().to_string(),
            root.join("ghost").display().to_string(),
            "C:\\Windows\\System32".to_string(),
        ];
        let row = check_dead_path_entries(&entries, &root);
        assert_eq!(row.status, "WARN");
        assert_eq!(row.detail.len(), 1, "只报域内死链: {:?}", row.detail);
        assert!(row.detail[0].contains("ghost"));
    }

    /// 重复条目判定：尾斜杠与大小写不敏感；唯一条目 OK。
    #[test]
    fn 重复条目判定_大小写与尾斜杠() {
        let entries = vec![
            r"D:\ohmyenv\jq".to_string(),
            r"d:\OHMYENV\jq\".to_string(),
            r"D:\ohmyenv\gh\bin".to_string(),
        ];
        let row = check_dup_path_entries(&entries);
        assert_eq!(row.status, "WARN");
        assert_eq!(row.detail.len(), 1);

        let ok = check_dup_path_entries(&[r"D:\a".to_string(), r"D:\b".to_string()]);
        assert_eq!(ok.status, "OK");
    }
}

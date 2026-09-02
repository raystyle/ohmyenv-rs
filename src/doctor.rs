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

/// 跑全部诊断项，流式形态：每项算完即经回调输出（三态采集期间先出 envroot 项，
/// 采集完成即连出五个派生项；返回全量行供汇总）。
pub fn run_doctor_with<F>(
    cat: &Catalog,
    env_root: &Path,
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

    // 2-7. 三态派生项（一次采集）
    let srows = status::collect_status(cat, env_root)?;
    put(check_version_drift(&srows));
    put(check_probe_fail(&srows));
    put(check_not_on_path(cat, &srows, env_root));
    put(check_pin_missing(cat, &srows));
    put(check_sha_missing(cat, &srows));

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

/// 本平台在管但未 pin：install 不带选项会失败的前置异味（evergreen 条目如 vsbuild 无 pin 属设计，排除）。
fn check_pin_missing(cat: &Catalog, srows: &[StatusRow]) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        if r.exe.is_none() {
            continue; // 平台不适用不算
        }
        let Ok(def) = cat.tool(&r.name) else { continue };
        if !toolver::platform_managed(def) || r.locked.is_some() || crate::vsbuild::is_vsbuild(def)
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
fn check_sha_missing(cat: &Catalog, srows: &[StatusRow]) -> DoctorRow {
    let mut detail = Vec::new();
    for r in srows {
        if r.locked.is_none() {
            continue;
        }
        let Ok(def) = cat.tool(&r.name) else { continue };
        if def.pin_sha256().is_none() {
            detail.push(format!("{}: pin 无 sha256（重装一次可回填）", r.name));
        }
    }
    DoctorRow {
        name: "sha-missing",
        status: if detail.is_empty() { "OK" } else { "WARN" },
        detail,
    }
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
    let mut detail = Vec::new();
    let mut bytes = 0u64;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_asset_like = name.ends_with(".zip")
            || name.ends_with(".tar.gz")
            || name.ends_with(".tar.xz")
            || name.ends_with(".exe")
            || name.ends_with(".msi");
        if is_asset_like && !pinned.contains(&name) {
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

#[cfg(test)]
mod tests {
    use super::*;

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

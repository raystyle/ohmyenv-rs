//! catalog 结构机检（D28 入册清单化）：对真仓与夹具同规则，入册漏带在 cargo test
//! 常规面红灯，不再靠装后实测暴露。规则即契约（R001 字段表与入册 checklist 的机检化）：
//! 1. 凡平台在管（exe / linux_exe / mac_exe 任一在位）的工具节必须有 `probe_pattern`
//!    ——toolver 正则漏带两犯（rclone D22、gitleaks D26）的根治面；
//! 2. `probe_pattern` 必须可编译且至少含一个捕获组（parse_version 取第 1 组）；
//! 3. sha 族字段（sha256 / linux_sha256 / mac_sha256）在位必为 64 位 hex
//!    ——digest 转写错配（M014）的格式面拦截（值对错由 seed.py --plan 域面 diff 对账）。

use std::path::Path;

use ome::catalog::Catalog;
use regex::Regex;

fn lint_catalog(path: &Path) -> Vec<String> {
    let cat = Catalog::load(path).expect("catalog 应能解析");
    let mut errs = Vec::new();
    for name in &cat.order {
        let t = cat.tool(name).expect("order 与 tools 应一致");
        let managed = t.exe.is_some() || t.linux_exe.is_some() || t.mac_exe.is_some();
        if managed {
            match &t.probe_pattern {
                None => errs.push(format!(
                    "{name}: 在管工具缺 probe_pattern（入册清单漏带，R001 入册 checklist）"
                )),
                Some(p) => match Regex::new(p) {
                    Err(e) => errs.push(format!("{name}: probe_pattern 不可编译: {e}")),
                    Ok(re) => {
                        if re.captures_len() < 2 {
                            errs.push(format!(
                                "{name}: probe_pattern 无捕获组（parse_version 取第 1 组）: {p}"
                            ));
                        }
                    }
                },
            }
        } else if let Some(p) = &t.probe_pattern {
            // 平台不在管的节带正则不拦（字段冗余无害），但坏正则仍拦
            if let Err(e) = Regex::new(p) {
                errs.push(format!("{name}: probe_pattern 不可编译: {e}"));
            }
        }
        for (k, v) in [
            ("sha256", &t.sha256),
            ("linux_sha256", &t.linux_sha256),
            ("mac_sha256", &t.mac_sha256),
        ] {
            if let Some(v) = v {
                if v.len() != 64 || !v.chars().all(|c| c.is_ascii_hexdigit()) {
                    errs.push(format!("{name}: {k} 非 64 位 hex: {v}"));
                }
            }
        }
    }
    errs
}

#[test]
fn 真仓catalog_结构机检() {
    let errs = lint_catalog(Path::new("catalog/tools.toml"));
    assert!(
        errs.is_empty(),
        "catalog 结构机检未过:\n{}",
        errs.join("\n")
    );
}

#[test]
fn 夹具catalog_结构机检() {
    let errs = lint_catalog(Path::new("tests/fixtures/tools.toml"));
    assert!(
        errs.is_empty(),
        "fixtures 结构机检未过:\n{}",
        errs.join("\n")
    );
}

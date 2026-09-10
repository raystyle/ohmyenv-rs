//! catalog 结构机检（D28 入册清单化；D37 完全解耦后只保夹具形态）：数据面四规则（在管
//! 必有 probe_pattern、正则可编译含捕获组、sha 64 hex、pin 资产名必被 asset_pattern 命中）。
//! 权威数据与数据门已迁 ohmycloud（vitest 形态），真仓测随权威件退役，本测作消费面格式
//! 的在库回归。
//! 常规面红灯，不再靠装后实测暴露。规则即契约（R001 字段表与入册 checklist 的机检化）：
//! 1. 凡平台在管（exe / linux_exe / mac_exe 任一在位）的工具节必须有 `probe_pattern`
//!    ——toolver 正则漏带两犯（rclone D22、gitleaks D26）的根治面；
//! 2. `probe_pattern` 必须可编译且至少含一个捕获组（parse_version 取第 1 组）；
//! 3. sha 族字段（sha256 / linux_sha256 / mac_sha256）在位必为 64 位 hex
//!    ——digest 转写错配（M014）的格式面拦截（值对错由 seed.py --plan 域面 diff 对账）；
//! 4. pin 资产名必被同平台 asset_pattern 命中——resolve 恒按 pattern 对 release 清单重筛
//!    （pin 只锁 tag 不锁资产名），pattern 按命名惯例拼错（claude linux 写 x86_64 实为
//!    x64，ohmyenv-rs#10 / M014 同型）在入册面红灯，零网络即可判（D29 增补）。

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
        for (plat, pat, pinned) in [
            ("win", &t.asset_pattern, &t.asset),
            ("linux", &t.linux_asset_pattern, &t.linux_asset),
            ("mac", &t.mac_asset_pattern, &t.mac_asset),
        ] {
            if let (Some(p), Some(a)) = (pat, pinned) {
                match Regex::new(p) {
                    Err(e) => errs.push(format!("{name}: {plat} asset_pattern 不可编译: {e}")),
                    Ok(re) => {
                        if !re.is_match(a) {
                            errs.push(format!(
                                "{name}: {plat} asset_pattern 不命中 pin 资产名: {p} vs {a}"
                            ));
                        }
                    }
                }
            }
        }
    }
    errs
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

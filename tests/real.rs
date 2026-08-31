//! 真机对齐闸门测试（R004）：OME_TEST_REAL=1 才执行，否则整体 skip。
//! 对照基准是 pwsh ohmyenv.ps1（D:\ohmypwsh，只读调用不修改）；pwsh 侧输出为人称格式，
//! 解析只提取稳定字段（工具行 / ===== 头的 tag），放过颜色码、时间与路径噪音。

use std::collections::HashMap;
use std::process::Command;

fn gated() -> bool {
    std::env::var("OME_TEST_REAL")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false)
}

fn skip(reason: &str) {
    eprintln!("[SKIP] {reason}（设 OME_TEST_REAL=1 启用真机对齐）");
}

/// 跑 pwsh ohmyenv.ps1 status，解析每工具的 locked/installed。
/// 行格式（helpers.ps1 status）：`  jq : locked=1.8.2  installed=1.8.2  path=True`
fn pwsh_status() -> HashMap<String, (String, String)> {
    let out = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-File",
            r"D:\ohmypwsh\scripts\ohmyenv.ps1",
            "status",
        ])
        .output()
        .expect("pwsh ohmyenv.ps1 status 应可运行");
    assert!(out.status.success(), "pwsh status 应成功");
    let text = String::from_utf8_lossy(&out.stdout);
    let re = regex::Regex::new(r"^(\S+) : locked=(\S*)\s+installed=(\S+)\s+path=\w+")
        .expect("静态正则应合法");
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(caps) = re.captures(line) {
            map.insert(
                caps[1].to_string(),
                (caps[2].to_string(), caps[3].to_string()),
            );
        }
    }
    map
}

/// 跑 ome status（真实 catalog + 默认 EnvRoot），解析 key=value 三态。
fn ome_status() -> HashMap<String, (String, String)> {
    let out = Command::new(assert_cmd::cargo::cargo_bin("ome"))
        .arg("status")
        .output()
        .expect("ome status 应可运行");
    assert!(out.status.success(), "ome status 应成功");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut map: HashMap<String, (String, String)> = HashMap::new();
    let mut cur: Option<String> = None;
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("tool=") {
            cur = Some(name.to_string());
        } else if let Some(locked) = line.strip_prefix("locked=") {
            if let Some(name) = &cur {
                map.entry(name.clone()).or_default().0 = locked.to_string();
            }
        } else if let Some(inst) = line.strip_prefix("installed=") {
            if let Some(name) = &cur {
                map.entry(name.clone()).or_default().1 = inst.to_string();
            }
        }
    }
    map
}

#[test]
fn real_status_三态与pwsh逐项一致() {
    if !gated() {
        skip("真机闸门未开");
        return;
    }
    let pwsh = pwsh_status();
    let ome = ome_status();
    assert!(!pwsh.is_empty(), "pwsh status 应解析出工具行");
    assert_eq!(
        pwsh.len(),
        ome.len(),
        "工具数应一致（pwsh {} vs ome {ome:?}）",
        pwsh.len()
    );
    let mut diffs = Vec::new();
    let mut known = Vec::new();
    for (tool, (locked, installed)) in &pwsh {
        match ome.get(tool) {
            Some((ol, oi)) => {
                if locked != ol || installed != oi {
                    let d = format!(
                        "{tool}: pwsh locked={locked} installed={installed} vs ome locked={ol} installed={oi}"
                    );
                    // 已知上游分歧（M001）：catalog 转换器对 rmux 取 psd1 Win 侧（相对 exe），
                    // 而 pwsh 运行时被 New-ToolDef 覆盖为 %LOCALAPPDATA% 官方布局；
                    // 数据修复前仅容忍这一处（locked 一致、仅 installed 探测路径不同）
                    if tool == "rmux" && locked == ol && oi == "-" {
                        known.push(d);
                    } else {
                        diffs.push(d);
                    }
                }
            }
            None => diffs.push(format!("{tool}: ome status 缺该工具")),
        }
    }
    for d in &known {
        eprintln!("[KNOWN] 已知上游分歧（M001，待数据层修复）: {d}");
    }
    assert!(diffs.is_empty(), "三态不一致:\n{}", diffs.join("\n"));
}

#[test]
fn real_query_jq_latest_与pwsh同tag() {
    if !gated() {
        skip("真机闸门未开");
        return;
    }
    // pwsh 侧头行：`===== jq (jq-1.8.2) =====`，只提取括号内 tag（稳定字段）
    let pwsh_out = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-File",
            r"D:\ohmypwsh\scripts\ohmyenv.ps1",
            "query",
            "jq",
            "-Latest",
        ])
        .output()
        .expect("pwsh query 应可运行");
    assert!(pwsh_out.status.success(), "pwsh query jq 应成功");
    let text = String::from_utf8_lossy(&pwsh_out.stdout);
    let re = regex::Regex::new(r"===== jq \(([^)]+)\) =====").expect("静态正则应合法");
    let pwsh_tag = re
        .captures(&text)
        .map(|c| c[1].to_string())
        .expect("pwsh query 输出应含 ===== jq (<tag>) ===== 头");

    let ome_out = Command::new(assert_cmd::cargo::cargo_bin("ome"))
        .args(["query", "jq", "--latest"])
        .output()
        .expect("ome query 应可运行");
    assert!(ome_out.status.success(), "ome query jq 应成功");
    let ome_text = String::from_utf8_lossy(&ome_out.stdout);
    let ome_tag = ome_text
        .lines()
        .find_map(|l| l.strip_prefix("tag="))
        .expect("ome query 输出应含 tag= 行");

    assert_eq!(ome_tag, pwsh_tag, "ome 与 pwsh 解析的最新 tag 应一致");
}

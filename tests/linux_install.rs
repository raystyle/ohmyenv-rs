//! Linux / macOS 部署集成测试：验证 install / status 在非 Windows 下可闭环。
//! 使用真实 GitHub 资产（jq），全程在临时 HOME 沙盒内（含 catalog 副本，pin 回写不落仓库），不污染用户真实 profile。
//!
//! 门控纪律（2026-09-10 M016 教训）：**文件级 cfg 会把整文件在 Windows 上摘掉，本机绿不算数**。
//! 故此处只给两个 POSIX 行为用例挂 `#[cfg(not(windows))]`，取件源与断言助手全平台参与编译，
//! 让本机 `cargo test`/`clippy` 也能对它们把关（Windows 上助手未用，允许 dead_code）。

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn sandbox() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("创建沙盒失败");
    let home = dir.path().join("home");
    fs::create_dir_all(&home).expect("创建 HOME 失败");
    let env_root = home.join(".local").join("share").join("ohmyenv");
    fs::create_dir_all(&env_root).expect("创建 env_root 失败");
    (dir, home, env_root)
}

fn ome(home: &Path, env_root: &Path) -> Command {
    ome_reg(home, env_root, false)
}

/// `allow_path_reg`：断言 profile 写入的用例传 true（沙盒 HOME 已隔离真实 profile，
/// 注册面本身是被测行为）；其余用例默认 O5 隔离（不写任何用户面）。
fn ome_reg(home: &Path, env_root: &Path, allow_path_reg: bool) -> Command {
    // catalog 落沙盒副本：install 的 pin 与 sha 回写不得触达真实 catalog
    let catalog = env_root.join("tools.sandbox.toml");
    fs::copy(catalog_source(env_root), &catalog).expect("复制 catalog 到沙盒失败");
    let mut cmd = Command::cargo_bin("ark").expect("ome 二进制应已构建");
    cmd.env("HOME", home);
    cmd.env("SHELL", "/bin/bash");
    cmd.env("ARK_CATALOG", &catalog);
    if !allow_path_reg {
        // O5（S017）：沙盒 install 不得写真实用户 PATH（profile）
        cmd.env("ARK_TEST_NO_PATH_REG", "1");
    }
    cmd.args(["--env-root", &env_root.to_string_lossy()]);
    cmd
}

/// 清单取件源（D37 终态：本仓不再持权威件，端上清单一律云端拉取）。
/// 序：`ARK_TEST_CATALOG` 显式指定、仓库件（开发态若在）、用户数据副本、云端三重门。
fn catalog_source(env_root: &Path) -> PathBuf {
    if let Some(p) = ark::platform::env_var_or("ARK_TEST_CATALOG", "OME_TEST_CATALOG") {
        let p = PathBuf::from(p);
        if p.exists() {
            return p;
        }
    }
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("catalog")
        .join("tools.toml");
    if repo.exists() {
        return repo;
    }
    if let Some(dir) = dirs::data_local_dir() {
        let p = dir.join("ohmyenv").join("catalog").join("tools.toml");
        if p.exists() {
            return p;
        }
    }
    ark::catalog::fetch_cloud(env_root)
        .expect("云端清单拉取失败（需网络与镜像可达；也可用 ARK_TEST_CATALOG 指定）")
        .path
}

#[cfg(not(windows))]
#[test]
fn linux_jq_安装部署状态闭环() {
    let (_guard, home, env_root) = sandbox();
    let profile = home.join(".bashrc");

    // 1) install：下载 jq 到 ~/.local/bin 并注册 PATH
    ome_reg(&home, &env_root, true)
        .args(["install", "jq", "--latest"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tool=jq"))
        .stdout(predicates::str::contains("action=installed"));

    let bin = home.join(".local").join("bin").join("jq");
    assert!(bin.exists(), "jq 二进制应已安装到 ~/.local/bin");
    let out = std::process::Command::new(&bin)
        .arg("--version")
        .output()
        .expect("jq 应可执行");
    assert!(out.status.success(), "jq --version 应成功");
    // 期望值来自被测二进制自身输出（独立于 catalog pin 与上游 --latest 漂移）
    let ver_text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let installed = ver_text.trim_start_matches("jq-").to_string();
    assert!(!installed.is_empty(), "应能解析 jq 版本: {ver_text}");

    let profile_text = fs::read_to_string(&profile).expect("profile 应已写入");
    assert!(
        profile_text.contains(&format!(
            "export PATH=\"{}:$PATH\"",
            home.join(".local/bin").display()
        )),
        "profile 应包含 ~/.local/bin 的 PATH 导出"
    );

    // 2) status：jq 应显示已安装且在 PATH 中
    ome_reg(&home, &env_root, true)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tool=jq"))
        .stdout(predicates::str::contains(format!("installed={installed}")))
        .stdout(predicates::str::contains("path=true"));
}

#[cfg(not(windows))]
#[test]
fn linux_profile_path_幂等() {
    let (_guard, home, env_root) = sandbox();
    let profile = home.join(".bashrc");

    // 首次 install 写入 PATH
    ome_reg(&home, &env_root, true)
        .args(["install", "jq", "--latest"])
        .assert()
        .success();

    let first = fs::read_to_string(&profile).expect("profile 应存在");
    let count1 = first
        .lines()
        .filter(|l| l.starts_with("export PATH="))
        .count();

    // 再次 install 不应重复写入
    ome_reg(&home, &env_root, true)
        .args(["install", "jq", "--latest"])
        .assert()
        .success();

    let second = fs::read_to_string(&profile).expect("profile 应存在");
    let count2 = second
        .lines()
        .filter(|l| l.starts_with("export PATH="))
        .count();
    assert_eq!(count1, count2, "PATH 导出应幂等，不重复添加");
}

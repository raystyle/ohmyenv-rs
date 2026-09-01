//! Linux / macOS 部署集成测试：验证 install / deploy / status 在非 Windows 下可闭环。
//! 使用真实 GitHub 资产（jq），全程在临时 HOME 沙盒内（含 catalog 副本，pin 回写不落仓库），不污染用户真实 profile。

#![cfg(not(windows))]

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
    // catalog 落沙盒副本：install/deploy 的 pin 与 sha 回写不得触达仓库 catalog
    let catalog = env_root.join("tools.sandbox.toml");
    let repo_catalog = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("catalog")
        .join("tools.toml");
    fs::copy(&repo_catalog, &catalog).expect("复制 catalog 到沙盒失败");
    let mut cmd = Command::cargo_bin("ome").expect("ome 二进制应已构建");
    cmd.env("HOME", home);
    cmd.env("SHELL", "/bin/bash");
    cmd.env("OME_CATALOG", &catalog);
    cmd.args(["--env-root", &env_root.to_string_lossy()]);
    cmd
}

#[test]
fn linux_jq_安装部署状态闭环() {
    let (_guard, home, env_root) = sandbox();
    let profile = home.join(".bashrc");

    // 1) install：下载并安装 jq 到 ~/.local/bin
    ome(&home, &env_root)
        .args(["install", "jq", "--latest"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tool=jq"))
        .stdout(predicates::str::contains("action=installed"));

    let bin = home.join(".local").join("bin").join("jq");
    assert!(bin.exists(), "jq 二进制应已安装到 ~/.local/bin");
    assert!(
        std::process::Command::new(&bin)
            .arg("--version")
            .output()
            .is_ok(),
        "jq 应可执行"
    );

    // 2) deploy：幂等跳过安装，但注册 PATH
    ome(&home, &env_root)
        .args(["deploy", "jq", "--latest"])
        .assert()
        .success()
        .stdout(predicates::str::contains("action=skipped"));

    let profile_text = fs::read_to_string(&profile).expect("profile 应已写入");
    assert!(
        profile_text.contains(&format!(
            "export PATH=\"{}:$PATH\"",
            home.join(".local/bin").display()
        )),
        "profile 应包含 ~/.local/bin 的 PATH 导出"
    );

    // 3) status：jq 应显示已安装且在 PATH 中
    ome(&home, &env_root)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tool=jq"))
        .stdout(predicates::str::contains("installed=1.8.2"))
        .stdout(predicates::str::contains("path=true"));
}

#[test]
fn linux_profile_path_幂等() {
    let (_guard, home, env_root) = sandbox();
    let profile = home.join(".bashrc");

    // 首次 deploy 写入
    ome(&home, &env_root)
        .args(["deploy", "jq", "--latest"])
        .assert()
        .success();

    let first = fs::read_to_string(&profile).expect("profile 应存在");
    let count1 = first
        .lines()
        .filter(|l| l.starts_with("export PATH="))
        .count();

    // 再次 deploy 不应重复写入
    ome(&home, &env_root)
        .args(["deploy", "jq", "--latest"])
        .assert()
        .success();

    let second = fs::read_to_string(&profile).expect("profile 应存在");
    let count2 = second
        .lines()
        .filter(|l| l.starts_with("export PATH="))
        .count();
    assert_eq!(count1, count2, "PATH 导出应幂等，不重复添加");
}

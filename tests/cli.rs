//! CLI 集成冒烟测试（离线路径）：以 OME_CATALOG 指向夹具，断退出码与 key=value 标记行。
//! 网络路径（query --latest 等）不在此处测，真机对齐走 OME_TEST_REAL 闸门。

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tools.toml")
}

fn ome() -> Command {
    let mut cmd = Command::cargo_bin("ome").expect("ome 二进制应已构建");
    cmd.env("OME_CATALOG", fixture());
    cmd
}

#[test]
fn pin_无选项_打印当前pin_sha截16位() {
    ome()
        .args(["pin", "age"])
        .assert()
        .success()
        .stdout(contains("tool=age"))
        .stdout(contains("tag=v1.3.1"))
        .stdout(contains("version=1.3.1"))
        .stdout(contains("asset=age-v1.3.1-windows-amd64.zip"))
        .stdout(contains("sha256=C56E8CE22F7E80CB..."));
}

#[test]
fn lock_别名等价pin() {
    ome()
        .args(["lock", "vault"])
        .assert()
        .success()
        .stdout(contains("tool=vault").and(contains("version=1.20.2")));
}

#[test]
fn dies_query_未知工具() {
    ome()
        .args(["query", "nonexistent"])
        .assert()
        .failure()
        .stderr(contains("未知工具"));
}

#[test]
fn dies_latest与tag互斥() {
    ome()
        .args(["query", "age", "--latest", "--tag", "v1.3.1"])
        .assert()
        .failure();
}

#[test]
fn status_沙盒_三态输出exe缺失为横线() {
    // 沙盒 EnvRoot：exe 不存在 → installed=-；path 读真实用户 PATH（只读）判定 bin 未注册
    let dir = tempfile::tempdir().expect("创建沙盒失败");
    ome()
        .args(["status", "--env-root", &dir.path().to_string_lossy()])
        .assert()
        .success()
        .stdout(contains("tool=age"))
        .stdout(contains("locked=1.3.1"))
        .stdout(contains("installed=-"))
        .stdout(contains("path=false"))
        .stdout(contains("# [核心基础工具]"))
        .stdout(contains("#   [密钥]"));
}

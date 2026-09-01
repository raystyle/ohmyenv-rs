//! install 链路集成测试：临时 EnvRoot 沙盒 + 动态生成的沙盒 catalog（OME_CATALOG 指向）。
//! 全程离线：cdn_url 分支解析不触网；幂等跳过在下载前短路；防穿越在校验前拦截。
//! 不碰真实 D:\ohmyenv（除只读借用 jq.exe 作假 exe）与真实注册表（不跑 deploy/update）。

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::str::contains;

/// 沙盒：临时 EnvRoot + 写入 catalog，返回（临时目录守卫， catalog 路径， env_root 路径）。
fn sandbox(catalog_text: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("创建沙盒失败");
    let env_root = dir.path().join("envroot");
    fs::create_dir_all(&env_root).expect("创建 envroot 失败");
    let catalog = dir.path().join("tools.toml");
    fs::write(&catalog, catalog_text).expect("写沙盒 catalog 失败");
    (dir, catalog, env_root)
}

fn ome(catalog: &Path, env_root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ome").expect("ome 二进制应已构建");
    cmd.env("OME_CATALOG", catalog);
    cmd.args(["--env-root", &env_root.to_string_lossy()]);
    cmd
}

#[test]
#[cfg(windows)]
fn dies_install_防穿越_目录越出envroot() {
    // dir 越出 EnvRoot：必须在下载前以「危险路径」拒绝（对齐 Test-SafeUnderRoot 语义）
    let catalog_text = r#"
[tools.evil]
dir = '..\evil'
exe = 'evil\evil.exe'
extract = "copy"
cdn_url = "https://example.invalid/evil.exe"
tag = "v1.0.0"
version = "1.0.0"
asset = "evil.exe"
"#;
    let (_guard, catalog, env_root) = sandbox(catalog_text);
    ome(&catalog, &env_root)
        .args(["install", "evil"])
        .assert()
        .failure()
        .stderr(contains("危险路径"));
    // 拒绝后不应在沙盒外留任何目录
    assert!(!_guard.path().join("evil").exists(), "不应创建越界目录");
}

#[test]
fn install_幂等跳过_已装版本一致不触网() {
    // 借真实 jq.exe 当假 exe（只读复制进沙盒）；缺失时闸门跳过
    let real_jq = Path::new(r"D:\ohmyenv\jq\jq.exe");
    if !real_jq.exists() {
        eprintln!("[SKIP] 无 D:\\ohmyenv\\jq\\jq.exe 可借用，跳过幂等测试");
        return;
    }
    // 先探测真实版本，动态生成与之匹配的 pin（期望值来自 exe 自身输出，非被测逻辑）
    let out = std::process::Command::new(real_jq)
        .arg("--version")
        .output()
        .expect("jq --version 应可运行");
    let line = String::from_utf8_lossy(&out.stdout);
    let ver = line
        .trim()
        .strip_prefix("jq-")
        .expect("jq 版本行应为 jq-<ver> 格式")
        .to_string();

    let catalog_text = format!(
        r#"
[tools.jq]
dir = "jq"
bin = "jq"
exe = 'jq\jq.exe'
extract = "copy"
cdn_url = "https://example.invalid/jq-windows-amd64.exe"
tag = "v{ver}"
version = "{ver}"
asset = "jq-windows-amd64.exe"
"#
    );
    let (_guard, catalog, env_root) = sandbox(&catalog_text);
    // 预置「已装」假 exe
    let jq_dir = env_root.join("jq");
    fs::create_dir_all(&jq_dir).expect("创建 jq 目录失败");
    fs::copy(real_jq, jq_dir.join("jq.exe")).expect("复制假 exe 失败");

    // cdn_url 指向不可达地址：若未走幂等短路，下载必失败；成功即证明未触网
    ome(&catalog, &env_root)
        .args(["install", "jq"])
        .assert()
        .success()
        .stdout(contains("tool=jq"))
        .stdout(contains("action=skipped"))
        .stdout(contains(format!("version={ver}")));
}

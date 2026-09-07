//! 镜像兜底链真网测试（D08，OME_TEST_MIRROR=1 才跑否则整体 skip）。
//! 断官方源场景：官方段 URL 故意不可达，断言回落 env.ohmygh.com 镜像段成功
//! 且 sha256 与 catalog pin 锚一致（信任锚即 pin 的端到端实证）。
//! 资产选 zoxide（545KB 小资产，镜像种子 69/69 在位，ohmycloud#2）。

use std::path::PathBuf;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn gated() -> bool {
    std::env::var("OME_TEST_MIRROR")
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[test]
fn 断官方源_镜像回落下载且sha与pin一致() -> TestResult<()> {
    if !gated() {
        eprintln!("skip: OME_TEST_MIRROR != 1");
        return Ok(());
    }
    // 期望值来源：catalog\tools.toml [tools.zoxide] pin（独立来源，非被测逻辑回显）
    let asset = "zoxide-0.10.0-x86_64-pc-windows-msvc.zip";
    let pin_sha = "F465AE548F8754C8E7EDBC60B45FBF58C92BFE123DB83D790252D6810FA5DAF1";
    let sandbox = std::env::temp_dir().join(format!("ome-mirror-test-{}", std::process::id()));
    std::fs::create_dir_all(&sandbox)?;
    let got: PathBuf = ome::download::download_asset_with_mirror(
        &sandbox,
        asset,
        // 官方段故意给不可达地址（.invalid TLD 保测不会真通），逼回落镜像段
        "https://official-invalid.ome-test.invalid/x.zip",
        Some(pin_sha),
        true,
        "zoxide",
        "0.10.0",
    )?;
    let actual = ome::download::sha256_file(&got)?;
    assert_eq!(
        actual, pin_sha,
        "镜像段下载产物 sha 必须与 catalog pin 一致（信任锚即 pin）"
    );
    std::fs::remove_dir_all(&sandbox)?;
    Ok(())
}

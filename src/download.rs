//! download：资产下载与缓存复用，语义对齐 helpers.ps1 的 Save-ReleaseAsset。
//! 缓存目录 <EnvRoot>\cache\<asset>：
//! - 命中且 sha256 一致则复用；不符删除重下；无 sha 基准直接复用。
//! - 下载走 ureq（3 次指数退避），失败回退系统 curl.exe（--retry 5）。
//! - sha256 计算用 sha2，比较统一大写。

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use sha2::{Digest, Sha256};

const MAX_ATTEMPTS: u32 = 3;

/// 缓存路径：<EnvRoot>\cache\<asset>。
pub fn cache_path(env_root: &Path, asset_name: &str) -> PathBuf {
    env_root.join("cache").join(asset_name)
}

/// 计算文件 sha256，返回大写 hex（比较基准统一大写）。
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let f = File::open(path).map_err(|e| format!("打开文件失败: {}: {e}", path.display()))?;
    let mut reader = BufReader::new(f);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("读取文件失败: {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:X}", hasher.finalize()))
}

/// 下载资产到缓存并复用：对齐 Save-ReleaseAsset 的缓存三分支。
/// expected_sha256 为 None 时无校验基准，已有缓存直接复用；force 跳过复用直接重下。
pub fn download_asset(
    env_root: &Path,
    asset_name: &str,
    url: &str,
    expected_sha256: Option<&str>,
    force: bool,
) -> Result<PathBuf, String> {
    let dest = cache_path(env_root, asset_name);
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("创建缓存目录失败: {}: {e}", dir.display()))?;
    }

    if dest.exists() && !force {
        if let Some(exp) = expected_sha256 {
            let actual = sha256_file(&dest)?;
            if actual.eq_ignore_ascii_case(exp) {
                eprintln!("[OK] 命中缓存（sha256 一致）: {}", dest.display());
                return Ok(dest);
            }
            eprintln!("[WARN] 缓存 sha256 不匹配，删除后重新下载: {}", dest.display());
            fs::remove_file(&dest)
                .map_err(|e| format!("删除旧缓存失败: {}: {e}", dest.display()))?;
        } else {
            eprintln!("[INFO] 已有缓存但无 sha256 基准，复用: {}", dest.display());
            return Ok(dest);
        }
    }

    download_url(url, &dest)?;
    if let Some(exp) = expected_sha256 {
        let actual = sha256_file(&dest)?;
        if !actual.eq_ignore_ascii_case(exp) {
            return Err(format!(
                "sha256 校验失败: {}\n期望 {}\n实际 {}",
                dest.display(),
                exp.to_uppercase(),
                actual
            ));
        }
    }
    eprintln!("[OK] 已下载: {}", dest.display());
    Ok(dest)
}

/// 强制重下（删旧再下）：校验清单类资产每次取新，不复用缓存。
pub fn download_fresh(env_root: &Path, asset_name: &str, url: &str) -> Result<PathBuf, String> {
    let dest = cache_path(env_root, asset_name);
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("创建缓存目录失败: {}: {e}", dir.display()))?;
    }
    if dest.exists() {
        fs::remove_file(&dest).map_err(|e| format!("删除旧文件失败: {}: {e}", dest.display()))?;
    }
    download_url(url, &dest)?;
    Ok(dest)
}

/// ureq 下载（3 次指数退避），失败回退系统 curl.exe（-L --fail --retry 5）。
fn download_url(url: &str, dest: &Path) -> Result<(), String> {
    let mut last_err = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        match download_once(url, dest) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e;
                if attempt < MAX_ATTEMPTS {
                    let wait = 2u64.pow(attempt);
                    eprintln!("[WARN] 下载失败，{wait}s 后重试（{attempt}/{MAX_ATTEMPTS}）: {last_err}");
                    std::thread::sleep(Duration::from_secs(wait));
                }
            }
        }
    }

    // 回退系统 curl.exe（对齐 pwsh：ureq/IWR 之外的独立网络栈兜底）
    let curl = which::which("curl")
        .map_err(|_| format!("下载失败且 curl.exe 不可用: {url}\n{last_err}"))?;
    eprintln!("[WARN] ureq 下载失败，改用 curl.exe: {last_err}");
    let status = Command::new(curl)
        .args([
            "-L",
            "--fail",
            "--retry",
            "5",
            "--retry-delay",
            "3",
            "--connect-timeout",
            "20",
            "-sS",
            "-o",
        ])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| format!("curl.exe 执行失败: {e}"))?;
    if !status.success() || !dest.exists() {
        return Err(format!("curl.exe 下载失败（{:?}）: {url}", status.code()));
    }
    Ok(())
}

/// 单次 ureq 下载：30s 连接超时，流式写盘。
fn download_once(url: &str, dest: &Path) -> Result<(), String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .build();
    let resp = agent
        .get(url)
        .set("User-Agent", "ome-bootstrap")
        .call()
        .map_err(|e| format!("HTTP 请求失败: {url}: {e}"))?;
    let mut reader = resp.into_reader();
    let f = File::create(dest).map_err(|e| format!("创建文件失败: {}: {e}", dest.display()))?;
    let mut writer = BufWriter::new(f);
    io::copy(&mut reader, &mut writer).map_err(|e| format!("写入文件失败: {}: {e}", dest.display()))?;
    writer
        .flush()
        .map_err(|e| format!("写入文件失败: {}: {e}", dest.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "abc" 的 sha256 是公开常数（FIPS 180-4 示例值），作独立期望值来源。
    #[test]
    fn sha256_计算_大写hex() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("abc.txt");
        fs::write(&path, b"abc").map_err(|e| e.to_string())?;
        assert_eq!(
            sha256_file(&path)?,
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
        Ok(())
    }

    #[test]
    fn 缓存_无sha基准_直接复用不触网() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let dest = cache_path(dir.path(), "demo.zip");
        fs::create_dir_all(dest.parent().ok_or("无父目录")?).map_err(|e| e.to_string())?;
        fs::write(&dest, b"cached").map_err(|e| e.to_string())?;

        // 无 sha 基准：即使 URL 不可达也应直接复用（不发起网络请求）
        let got = download_asset(dir.path(), "demo.zip", "https://example.invalid/x", None, false)?;
        assert_eq!(got, dest);
        assert_eq!(fs::read(&got).map_err(|e| e.to_string())?, b"cached");
        Ok(())
    }

    #[test]
    fn 缓存_sha一致_复用不触网() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let dest = cache_path(dir.path(), "demo.zip");
        fs::create_dir_all(dest.parent().ok_or("无父目录")?).map_err(|e| e.to_string())?;
        fs::write(&dest, b"abc").map_err(|e| e.to_string())?;

        let sha = "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD";
        let got = download_asset(dir.path(), "demo.zip", "https://example.invalid/x", Some(sha), false)?;
        assert_eq!(got, dest, "sha 一致应复用缓存");
        Ok(())
    }

    #[test]
    fn dies_缓存_sha不符_删除后重下失败() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let dest = cache_path(dir.path(), "demo.zip");
        fs::create_dir_all(dest.parent().ok_or("无父目录")?).map_err(|e| e.to_string())?;
        fs::write(&dest, b"stale").map_err(|e| e.to_string())?;

        // sha 不符：应删除旧缓存并尝试重下；本机不可达端口（连接即刻拒绝）最终报错
        let bogus = "0000000000000000000000000000000000000000000000000000000000000000";
        let err = download_asset(dir.path(), "demo.zip", "http://127.0.0.1:1/x", Some(bogus), false)
            .expect_err("不可达 URL 应报错");
        assert!(!dest.exists(), "旧缓存应已被删除");
        assert!(!err.is_empty());
        Ok(())
    }
}

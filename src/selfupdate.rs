//! selfupdate：ome 自身升级（`ome self update`）。
//! 产物源：GitHub Actions push main 构建的三平台资产，滚动挂在 pre-release tag `dev`
//! （本地测试期不封版：无稳定版本号，每次 push 覆盖资产，资产 sha256 即版本基准）。
//! 升级判定：release 资产的 API digest（sha256）与运行中 exe 的 sha256 对比，一致即已最新；
//! 不同则经 download_asset 下载到缓存（digest 校验）后替换部署位，并顺手刷新数据目录 catalog
//! （raw.githubusercontent main，best-effort，失败不拦升级）。
//! Windows 运行中 exe 可改名不可删：旧 exe 改名 .old 保留、新 exe 就位，下次升级开头清理。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

use crate::download::{download_asset, sha256_file};
use crate::platform;

const REPO: &str = "raystyle/ohmyenv-rs";
const ROLLING_TAG: &str = "dev";
const UA: &str = "ome-selfupdate";

/// 升级结果。
pub struct SelfUpdateOutcome {
    /// updated：已替换；current：已是最新
    pub action: &'static str,
    pub asset: String,
    pub sha256: String,
    pub exe: PathBuf,
    pub catalog_synced: bool,
}

/// 编译目标对应的 CI 资产名（build.yml 的资产命名约定）。
pub fn asset_for_this_platform() -> Result<&'static str, String> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        return Ok("ome-x86_64-pc-windows-msvc.exe");
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Ok("ome-x86_64-unknown-linux-gnu");
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return Ok("ome-aarch64-apple-darwin");
    }
    #[allow(unreachable_code)]
    Err("当前平台无 CI 构建资产（dev release 未覆盖此目标）".to_string())
}

/// 自升级主流程：dev release 元数据 → digest 对比 → 下载校验 → 替换 → 刷 catalog。
pub fn self_update(env_root: &Path) -> Result<SelfUpdateOutcome, String> {
    let asset_name = asset_for_this_platform()?;
    let release = fetch_release(ROLLING_TAG)?;
    let assets = release
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| "dev release 无资产列表".to_string())?;
    let asset = assets
        .iter()
        .find(|a| a.get("name").and_then(Value::as_str) == Some(asset_name))
        .ok_or_else(|| format!("dev release 缺资产 {asset_name}（CI 是否已跑完？）"))?;
    let digest = asset
        .get("digest")
        .and_then(Value::as_str)
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or_else(|| format!("资产 {asset_name} 无 sha256 digest，拒绝无校验升级"))?
        .to_uppercase();
    let dl_url = asset
        .get("browser_download_url")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("资产 {asset_name} 无下载地址"))?
        .to_string();

    let exe = std::env::current_exe().map_err(|e| format!("定位自身 exe 失败: {e}"))?;
    let mine = sha256_file(&exe)?;
    if mine == digest {
        eprintln!("[OK] 已是最新构建（sha256 一致）");
        return Ok(SelfUpdateOutcome {
            action: "current",
            asset: asset_name.to_string(),
            sha256: sha8(&digest),
            exe,
            catalog_synced: sync_catalog(),
        });
    }

    eprintln!("[INFO] 本地 {mine} 与远端 {digest} 不同，下载更新");
    let cached = download_asset(env_root, asset_name, &dl_url, Some(&digest), false)?;
    replace_exe(&exe, &cached)?;
    let catalog_synced = sync_catalog();
    Ok(SelfUpdateOutcome {
        action: "updated",
        asset: asset_name.to_string(),
        sha256: sha8(&digest),
        exe,
        catalog_synced,
    })
}

/// 替换部署位 exe：Windows 改名旧的为 .old 再 copy 新的（运行中 exe 不可删）；
/// Unix copy 到同目录临时文件后 chmod 755 再 rename 原子覆盖。
fn replace_exe(exe: &Path, new_file: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        let old = exe.with_extension("exe.old");
        let _ = std::fs::remove_file(&old); // 上次升级残留（进程已退出才删得掉）
        std::fs::rename(exe, &old).map_err(|e| format!("改名旧 exe 失败: {e}"))?;
        if let Err(e) = std::fs::copy(new_file, exe) {
            // 回滚：把旧名改回来，不留半损状态
            let _ = std::fs::rename(&old, exe);
            return Err(format!("写入新 exe 失败（已回滚）: {e}"));
        }
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let tmp = exe.with_extension("ome-new");
        std::fs::copy(new_file, &tmp).map_err(|e| format!("写临时文件失败: {e}"))?;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod 755 失败: {e}"))?;
        std::fs::rename(&tmp, exe).map_err(|e| format!("替换 exe 失败: {e}"))?;
    }
    Ok(())
}

/// 刷新数据目录 catalog：raw.githubusercontent main 源（CDN 无限流）。
/// best-effort：失败只提示，不拦升级主流程。
fn sync_catalog() -> bool {
    let dest = platform::metadata_dir().join("catalog").join("tools.toml");
    let Some(dir) = dest.parent().map(|p| p.to_path_buf()) else {
        return false;
    };
    let url = format!("https://raw.githubusercontent.com/{REPO}/main/catalog/tools.toml");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout(Duration::from_secs(30))
        .build();
    let Ok(resp) = agent.get(&url).set("User-Agent", UA).call() else {
        eprintln!("[WARN] catalog 同步失败（不影响升级）: {url}");
        return false;
    };
    let Ok(text) = resp.into_string() else {
        return false;
    };
    match std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&dest, &text)) {
        Ok(()) => {
            eprintln!("[OK] catalog 已同步: {}", dest.display());
            true
        }
        Err(e) => {
            eprintln!("[WARN] catalog 写入失败（不影响升级）: {e}");
            false
        }
    }
}

/// 取 release 元数据：直连 api.github.com（带 GH_TOKEN 注入），403/限流回退 gh api。
fn fetch_release(tag: &str) -> Result<Value, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/tags/{tag}");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout(Duration::from_secs(30))
        .build();
    let mut req = agent
        .get(&url)
        .set("User-Agent", UA)
        .set("Accept", "application/vnd.github+json");
    if let Ok(tok) = std::env::var("GH_TOKEN") {
        req = req.set("Authorization", &format!("Bearer {tok}"));
    }
    match req.call() {
        Ok(resp) => {
            let body = resp
                .into_string()
                .map_err(|e| format!("读取响应体失败: {e}"))?;
            serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败: {e}"))
        }
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("403") || msg.to_lowercase().contains("rate limit") {
                eprintln!("[INFO] api.github.com 直连受限，改用 gh api（认证通道）");
                gh_api(&url)
            } else {
                Err(format!("查询 dev release 失败: {msg}"))
            }
        }
    }
}

/// gh api 兜底（认证通道）：与 resolve.rs 同思路，selfupdate 独立持有。
fn gh_api(url: &str) -> Result<Value, String> {
    let path = url
        .strip_prefix("https://api.github.com")
        .ok_or_else(|| format!("非 api.github.com 地址: {url}"))?;
    let gh = which::which("gh").map_err(|_| "gh 不可用，无法回退 gh api".to_string())?;
    let out = Command::new(gh)
        .arg("api")
        .arg(path)
        .output()
        .map_err(|e| format!("gh api 执行失败: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gh api 失败: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("gh api 输出解析失败: {e}"))
}

/// 摘要展示用短 sha（前 8 位）。
fn sha8(s: &str) -> String {
    s.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 资产名_当前平台必有映射() {
        // 本 CI 覆盖的三目标之一，或明确报不支持
        match asset_for_this_platform() {
            Ok(name) => assert!(name.starts_with("ome-"), "资产名应带 ome- 前缀: {name}"),
            Err(e) => assert!(e.contains("无 CI 构建资产")),
        }
    }

    #[test]
    fn 短sha_取前8位() {
        assert_eq!(sha8("ABCDEF1234"), "ABCDEF12");
    }
}

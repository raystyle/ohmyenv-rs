//! selfupdate：ome 自身升级（`ome self update`），三通道：
//! - **dev**（默认）：pre-release tag `dev` 的滚动资产——CI push main 构建上传，本地测试期升级源；
//! - **stable**：`releases/latest` 正式版——CI 推 v* tag（封版）触发；
//! - **git**：源码安装——浅克隆仓库 cargo build 后替换（封版前无任何 release 时的通道，需 git 与 cargo）。
//!
//! 升级判定：release 资产的 API digest（sha256）与运行中 exe 的 sha256 对比，一致即已最新；
//! 不同则经 download_asset 下载到缓存（digest 校验）后替换部署位，并同步数据目录 catalog。
//! Windows 运行中 exe 可改名不可删：旧 exe 改名 .old 保留、新 exe 就位，下次升级开头清理。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

use crate::download::{download_asset, sha256_file};
use crate::platform;

const REPO: &str = "raystyle/ohmyenv-rs";
const UA: &str = "ome-selfupdate";

/// 升级通道。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Channel {
    /// dev 滚动源（main push CI，pre-release tag `dev`）
    Dev,
    /// 正式版（v* tag CI，releases/latest）
    Stable,
    /// 源码安装（浅克隆 + cargo build）
    Git,
}

/// 升级结果。
pub struct SelfUpdateOutcome {
    /// updated：已替换；current：已是最新
    pub action: &'static str,
    pub channel: &'static str,
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
    Err("当前平台无 CI 构建资产（release 未覆盖此目标）".to_string())
}

/// 自升级主流程。
pub fn self_update(env_root: &Path, channel: Channel) -> Result<SelfUpdateOutcome, String> {
    match channel {
        Channel::Git => self_update_git(),
        Channel::Dev => self_update_release(env_root, "tags/dev"),
        Channel::Stable => self_update_release(env_root, "latest"),
    }
}

/// release 通道（dev 滚动 / latest 正式）：元数据 → digest 对比 → 下载校验 → 替换 → 刷 catalog。
fn self_update_release(env_root: &Path, endpoint: &str) -> Result<SelfUpdateOutcome, String> {
    let channel = if endpoint == "latest" {
        "stable"
    } else {
        "dev"
    };
    let asset_name = asset_for_this_platform()?;
    let release = fetch_release(endpoint)?;
    let assets = release
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| "release 无资产列表".to_string())?;
    let asset = assets
        .iter()
        .find(|a| a.get("name").and_then(Value::as_str) == Some(asset_name))
        .ok_or_else(|| format!("release 缺资产 {asset_name}（CI 是否已跑完？）"))?;
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
            channel,
            asset: asset_name.to_string(),
            sha256: sha8(&digest),
            exe,
            catalog_synced: sync_catalog_from_raw(),
        });
    }

    eprintln!("[INFO] 本地 {mine} 与远端 {digest} 不同，下载更新");
    let cached = download_asset(env_root, asset_name, &dl_url, Some(&digest), false)?;
    replace_exe(&exe, &cached)?;
    let catalog_synced = sync_catalog_from_raw();
    Ok(SelfUpdateOutcome {
        action: "updated",
        channel,
        asset: asset_name.to_string(),
        sha256: sha8(&digest),
        exe,
        catalog_synced,
    })
}

/// git 通道：浅克隆仓库构建后替换（封版前无 release 的源码安装；需 git 与 cargo）。
fn self_update_git() -> Result<SelfUpdateOutcome, String> {
    let git = which::which("git").map_err(|_| "git 通道需要 git 在 PATH".to_string())?;
    let cargo = which::which("cargo").map_err(|_| {
        "git 通道需要 cargo 在 PATH（无 Rust 工具链时用 dev/stable 通道）".to_string()
    })?;

    let work = std::env::temp_dir().join(format!("ome-selfupdate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    let url = format!("https://github.com/{REPO}");
    eprintln!("[INFO] 浅克隆 {url}");
    let out = Command::new(&git)
        .args(["clone", "--depth", "1"])
        .arg(&url)
        .arg(&work)
        .output()
        .map_err(|e| format!("git clone 启动失败: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git clone 失败: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }

    eprintln!("[INFO] cargo build --release（源码构建耗时较长）");
    let out = Command::new(&cargo)
        .args(["build", "--release", "--locked"])
        .current_dir(&work)
        .output()
        .map_err(|e| format!("cargo build 启动失败: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo build 失败: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }

    #[cfg(windows)]
    let bin = work.join("target").join("release").join("ome.exe");
    #[cfg(not(windows))]
    let bin = work.join("target").join("release").join("ome");

    let exe = std::env::current_exe().map_err(|e| format!("定位自身 exe 失败: {e}"))?;
    let (mine, built) = (sha256_file(&exe)?, sha256_file(&bin)?);
    let catalog_src = work.join("catalog").join("tools.toml");
    if mine == built {
        eprintln!("[OK] 已是最新构建（sha256 一致）");
        let catalog_synced = sync_catalog_from_file(&catalog_src);
        let _ = std::fs::remove_dir_all(&work);
        return Ok(SelfUpdateOutcome {
            action: "current",
            channel: "git",
            asset: "source".to_string(),
            sha256: sha8(&built),
            exe,
            catalog_synced,
        });
    }
    replace_exe(&exe, &bin)?;
    let catalog_synced = sync_catalog_from_file(&catalog_src);
    let _ = std::fs::remove_dir_all(&work);
    Ok(SelfUpdateOutcome {
        action: "updated",
        channel: "git",
        asset: "source".to_string(),
        sha256: sha8(&built),
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

/// 刷新数据目录 catalog（来源为本地文件，git 通道用）。best-effort：失败只提示。
fn sync_catalog_from_file(src: &Path) -> bool {
    let dest = platform::metadata_dir().join("catalog").join("tools.toml");
    let Ok(text) = std::fs::read(src) else {
        eprintln!("[WARN] catalog 源读取失败（不影响升级）");
        return false;
    };
    let write = dest
        .parent()
        .map(std::fs::create_dir_all)
        .and_then(|_| std::fs::write(&dest, &text).ok());
    match write {
        Some(()) => {
            eprintln!("[OK] catalog 已同步: {}", dest.display());
            true
        }
        None => {
            eprintln!("[WARN] catalog 写入失败（不影响升级）");
            false
        }
    }
}

/// 刷新数据目录 catalog：raw.githubusercontent main 源（CDN 无限流）。best-effort。
fn sync_catalog_from_raw() -> bool {
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
    let dest = platform::metadata_dir().join("catalog").join("tools.toml");
    let write = dest
        .parent()
        .map(std::fs::create_dir_all)
        .and_then(|_| std::fs::write(&dest, &text).ok());
    match write {
        Some(()) => {
            eprintln!("[OK] catalog 已同步: {}", dest.display());
            true
        }
        None => {
            eprintln!("[WARN] catalog 写入失败（不影响升级）");
            false
        }
    }
}

/// 取 release 元数据：直连 api.github.com（带 GH_TOKEN 注入），403/限流回退 gh api。
fn fetch_release(endpoint: &str) -> Result<Value, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/{endpoint}");
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
            } else if msg.contains("404") {
                Err(format!(
                    "尚无对应 release（未封版无正式版；dev 通道需先有 main push 的 CI）: {endpoint}"
                ))
            } else {
                Err(format!("查询 release 失败: {msg}"))
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

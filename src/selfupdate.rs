//! selfupdate：ome 自身升级（`ark self update`），三通道：
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

use crate::download::sha256_file;
use crate::platform;

/// 自升级源仓库（D41 更名；旧 ohmyenv-rs 名 GitHub 301 兜底，官方路径不断）。
/// doctor 网络探针同源引用（自测 7 机检）。
pub const REPO: &str = "raystyle/ark-rs";
const UA: &str = "ark-selfupdate";

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

/// 是否自管条目（extract = "ome-self"；D41 起双接受 "ark-self"，数据面改名可单方回退）：无 pin 无资产，升级走 self update 三通道。
pub fn is_ome_self(def: &crate::catalog::Tool) -> bool {
    matches!(def.extract(), Some("ome-self") | Some("ark-self"))
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

/// 编译目标三元组（build.yml 资产命名约定的公共段）。
fn platform_triple() -> Result<&'static str, String> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        return Ok("x86_64-pc-windows-msvc.exe");
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return Ok("x86_64-unknown-linux-gnu");
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return Ok("aarch64-apple-darwin");
    }
    #[allow(unreachable_code)]
    Err("当前平台无 CI 构建资产（release 未覆盖此目标）".to_string())
}

/// 编译目标对应的 CI 资产主名（D41 B：`ark-<triple>`，release 双附主名）。
pub fn asset_for_this_platform() -> Result<String, String> {
    Ok(format!("ark-{}", platform_triple()?))
}

/// 兼容资产名（`ome-<triple>`，旧二进制认的名；存量机水位清零后随 B 收口撤除）。
pub fn asset_compat_for_this_platform() -> Result<String, String> {
    Ok(format!("ome-{}", platform_triple()?))
}

/// 自升级主流程。
pub fn self_update(env_root: &Path, channel: Channel) -> Result<SelfUpdateOutcome, String> {
    match channel {
        Channel::Git => self_update_git(env_root),
        Channel::Dev => self_update_release(env_root, "tags/dev"),
        Channel::Stable => self_update_release(env_root, "latest"),
    }
}

/// 镜像优先开关（`ARK_MIRROR=1`，读回 `OME_MIRROR`）：self update 跳过官方 API 直取镜像边车锚。
/// 供断源验收（远端不可构造官方断网）与未来默认切自建过渡；锚语义不变（边车取不到即拒绝）。
fn mirror_first() -> bool {
    crate::platform::env_var_or("ARK_MIRROR", "OME_MIRROR").as_deref() == Some("1")
}

/// release 通道（dev 滚动 / latest 正式）：元数据 → digest 对比 → 下载校验 → 替换 → 刷 catalog。
fn self_update_release(env_root: &Path, endpoint: &str) -> Result<SelfUpdateOutcome, String> {
    let channel = if endpoint == "latest" {
        "stable"
    } else {
        "dev"
    };
    let asset_name = asset_for_this_platform()?;
    let asset_compat = asset_compat_for_this_platform().ok();
    // 镜像段按通道分（oma 同型，段名与通道同名）。D41 B 读序：ark/ 段配 ark-* 主名先，
    // 404 回落 ome/ 段配 ome-* 兼容名（切换期 CI 双写双段，存量机水位清零后撤兼容）。
    // dev 通道禁止回落 stable，避免把正式版装进滚动源；latest 段已退役（D30 封版拆分）。
    let mirror_ver = if channel == "stable" { "stable" } else { "dev" };
    let official = if mirror_first() {
        Err("ARK_MIRROR=1 镜像优先，跳过官方 API".to_string())
    } else {
        official_asset_meta(endpoint, &asset_name, asset_compat.as_deref())
    };
    let (digest, dl_url, seg_used, asset_used) = match official {
        Ok((d, u, name)) => (d, u, String::new(), name),
        Err(api_err) => mirror_fallback_meta(env_root, mirror_ver, &asset_name, asset_compat.as_deref(), &api_err)?,
    };

    let exe = std::env::current_exe().map_err(|e| format!("定位自身 exe 失败: {e}"))?;
    let mine = sha256_file(&exe)?;
    if mine == digest {
        eprintln!("[OK] 已是最新构建（sha256 一致）");
        return Ok(SelfUpdateOutcome {
            action: "current",
            channel,
            asset: asset_used,
            sha256: sha8(&digest),
            exe: platform::self_deploy_target().unwrap_or(exe),
            catalog_synced: sync_catalog_from_cloud(env_root),
        });
    }

    eprintln!("[INFO] 本地 {mine} 与远端 {digest} 不同，下载更新");
    // 镜像段内回落（官方 URL 失败时 download 层再兜一次）：镜像路径已命中则沿用其段；
    // 官方路径按命中资产名前缀取段（纯函数 fallback_seg，三态单测覆盖）
    let seg_for_fallback = fallback_seg(&seg_used, &asset_used);
    let cached = crate::download::download_asset_with_mirror(
        env_root,
        &asset_used,
        &dl_url,
        Some(&digest),
        false,
        seg_for_fallback,
        mirror_ver,
    )?;
    let exe = replace_deployed_and_current(&cached)?;
    // D41 C：升级后顺手搬旧元数据（幂等；失败只告警不拦升级收尾）
    if let Err(e) = platform::migrate_legacy_metadata() {
        eprintln!("[WARN] 元数据搬迁失败（旧位读回继续）: {e}");
    }
    let catalog_synced = sync_catalog_from_cloud(env_root);
    Ok(SelfUpdateOutcome {
        action: "updated",
        channel,
        asset: asset_used,
        sha256: sha8(&digest),
        exe,
        catalog_synced,
    })
}

/// 镜像段内回落的段名裁定（纯函数）：镜像路径已命中则沿用其段；官方路径按命中资产名
/// 前缀取段（ome-* 配 ome/ 段、ark-* 配 ark/ 段）——过渡窗 release 仅 ome-* 时官方命中的
/// 是兼容名，回落段必须跟着兼容族走，否则 ark/ome-* 拼出 404 断保供。
fn fallback_seg<'a>(seg_used: &'a str, asset_used: &str) -> &'a str {
    if !seg_used.is_empty() {
        return seg_used;
    }
    if asset_used.starts_with("ome-") {
        "ome"
    } else {
        "ark"
    }
}

/// 镜像段读序尝试表（纯函数，自测 3 三态矩阵的构造面）：ark/ 段配 ark-* 主名先，
/// ome/ 段配 ome-* 兼容名回落；无兼容名时单尝试。每项含段、资产、边车与下载 URL。
fn mirror_attempts(
    base: &str,
    channel: &str,
    primary: &str,
    compat: Option<&str>,
) -> Vec<(String, String, String, String)> {
    let mut out = vec![];
    for (seg, asset) in [("ark", primary)].into_iter().chain(compat.map(|c| ("ome", c))) {
        out.push((
            seg.to_string(),
            asset.to_string(),
            format!("{base}/{seg}/{channel}/{asset}.sha256"),
            format!("{base}/{seg}/{channel}/{asset}"),
        ));
    }
    out
}

/// 镜像段读序落锚（D41 B）：按尝试表取首个在位边车为锚，命中返回（锚、下载 URL、段、资产名）；
/// 全败报双链错误（含官方 API 原因）。
fn mirror_fallback_meta(
    env_root: &Path,
    channel: &str,
    primary: &str,
    compat: Option<&str>,
    api_err: &str,
) -> Result<(String, String, String, String), String> {
    let attempts = mirror_attempts(crate::download::MIRROR_BASE, channel, primary, compat);
    eprintln!(
        "[WARN] 官方 API 失败（{api_err}），镜像段读序试边车：{}",
        attempts
            .iter()
            .map(|(seg, a, _, _)| format!("{seg}/{channel}/{a}"))
            .collect::<Vec<_>>()
            .join(" 先、")
    );
    let mut last = String::new();
    for (seg, asset, sidecar_url, dl) in attempts {
        match crate::download::mirror_sidecar_sha(env_root, &sidecar_url) {
            Ok(digest) => return Ok((digest, dl, seg, asset)),
            Err(e) => last = format!("{seg}/{asset}: {e}"),
        }
    }
    Err(format!("镜像段读序全败（官方: {api_err}; {last}）"))
}

/// 官方 release 资产元数据（digest 大写 + 下载直链 + 命中资产名）；主名缺失试兼容名
/// （双附过渡期），双双缺失报主名错；API 段失败由调用方走镜像边车读序。
fn official_asset_meta(
    endpoint: &str,
    asset_name: &str,
    compat: Option<&str>,
) -> Result<(String, String, String), String> {
    asset_meta(endpoint, asset_name).or_else(|primary_err| match compat {
        Some(c) => asset_meta(endpoint, c).map_err(|_| primary_err),
        None => Err(primary_err),
    })
}

fn asset_meta(endpoint: &str, asset_name: &str) -> Result<(String, String, String), String> {
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
    Ok((digest, dl_url, asset_name.to_string()))
}

/// git 通道：浅克隆仓库构建后替换（封版前无 release 的源码安装；需 git 与 cargo）。
fn self_update_git(env_root: &Path) -> Result<SelfUpdateOutcome, String> {
    let git = which::which("git").map_err(|_| "git 通道需要 git 在 PATH".to_string())?;
    let cargo = which::which("cargo").map_err(|_| {
        "git 通道需要 cargo 在 PATH（无 Rust 工具链时用 dev/stable 通道）".to_string()
    })?;

    let work = std::env::temp_dir().join(format!("ark-selfupdate-{}", std::process::id()));
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

    // 产物名随 crate 名派生（D41 更名后随 CARGO_PKG_NAME 走，不再硬编码）
    #[cfg(windows)]
    let bin = work
        .join("target")
        .join("release")
        .join(format!("{}.exe", env!("CARGO_PKG_NAME")));
    #[cfg(not(windows))]
    let bin = work.join("target").join("release").join(env!("CARGO_PKG_NAME"));

    let exe = std::env::current_exe().map_err(|e| format!("定位自身 exe 失败: {e}"))?;
    let (mine, built) = (sha256_file(&exe)?, sha256_file(&bin)?);
    if mine == built {
        eprintln!("[OK] 已是最新构建（sha256 一致）");
        let catalog_synced = sync_catalog_from_cloud(env_root);
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
    let exe = replace_deployed_and_current(&bin)?;
    // D41 C：同 release 通道（幂等搬迁，失败只告警）
    if let Err(e) = platform::migrate_legacy_metadata() {
        eprintln!("[WARN] 元数据搬迁失败（旧位读回继续）: {e}");
    }
    let catalog_synced = sync_catalog_from_cloud(env_root);
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

/// 先替换自部署目标（用户 PATH 上的 ark），若当前进程 exe 不同再替换运行中副本（cargo run）。
/// D41 C：随替换重建 `ome` 别名（部署位同目录同内容副本，best-effort 不拦升级）。
fn replace_deployed_and_current(new_file: &Path) -> Result<PathBuf, String> {
    let current = std::env::current_exe().map_err(|e| format!("定位自身 exe 失败: {e}"))?;
    let deploy = platform::self_deploy_target()?;
    if let Some(parent) = deploy.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建部署目录失败: {}: {e}", parent.display()))?;
    }
    replace_exe(&deploy, new_file)?;
    if !path_same(&current, &deploy) && current.exists() {
        let _ = replace_exe(&current, new_file);
    }
    if let Ok(alias) = platform::ome_alias_target() {
        if !path_same(&alias, &deploy) {
            if let Err(e) = replace_exe(&alias, new_file) {
                eprintln!("[WARN] ome 别名重建失败（不拦升级）: {e}");
            }
        }
    }
    Ok(deploy)
}

fn path_same(a: &Path, b: &Path) -> bool {
    let na = a
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('/', "\\");
    let nb = b
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('/', "\\");
    na.eq_ignore_ascii_case(&nb)
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

/// 刷新数据目录 catalog（D37 终态：一律云端权威，走边车锚加解析加验签三重门）。
/// best-effort：失败只提示，用户可稍后 `ark catalog sync`；不再从已退役的仓库件取源。
fn sync_catalog_from_cloud(env_root: &Path) -> bool {
    let target = crate::catalog::user_data_catalog_path();
    match crate::catalog::sync_to(
        env_root,
        &target,
        true,
        crate::catalog::auto_ttl(),
    ) {
        Ok(outcome) => {
            eprintln!("[OK] catalog 已同步（云端验签）: {} [{}]", target.display(), outcome.action());
            true
        }
        Err(e) => {
            eprintln!("[WARN] catalog 云端刷新失败（不影响升级，可稍后 `ark catalog sync`）: {e}");
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
        // 本 CI 覆盖的三目标之一，或明确报不支持；D41 B 起主名 ark-、兼容名 ome-
        match (asset_for_this_platform(), asset_compat_for_this_platform()) {
            (Ok(primary), Ok(compat)) => {
                assert!(primary.starts_with("ark-"), "主名应带 ark- 前缀: {primary}");
                assert!(compat.starts_with("ome-"), "兼容名应带 ome- 前缀: {compat}");
                assert_eq!(
                    primary.trim_start_matches("ark-"),
                    compat.trim_start_matches("ome-"),
                    "主名与兼容名共用三元组"
                );
            }
            (Err(e), _) => assert!(e.contains("无 CI 构建资产")),
            _ => panic!("主名可解析则兼容名必可解析"),
        }
    }

    #[test]
    fn 段内回落裁定_三态() {
        // 镜像命中沿用其段；官方命中按资产族；默认主段
        assert_eq!(fallback_seg("ome", "ome-x.exe"), "ome", "镜像命中的段直接沿用");
        assert_eq!(fallback_seg("ark", "ark-x.exe"), "ark", "镜像命中的段直接沿用");
        assert_eq!(fallback_seg("", "ome-x.exe"), "ome", "官方命中兼容名回落兼容段");
        assert_eq!(fallback_seg("", "ark-x.exe"), "ark", "官方命中主名回落主段");
    }

    #[test]
    fn 镜像段读序_尝试表三态构造() {
        // D41 自测 3（构造面）：ark 主先、ome 兼容回落、URL 形态、无兼容名单尝试
        let base = "https://mirror.example";
        let two = mirror_attempts(base, "dev", "ark-x.exe", Some("ome-x.exe"));
        assert_eq!(two.len(), 2, "双名双段两尝试");
        assert_eq!(two[0].0, "ark", "ark 段必须先试");
        assert_eq!(two[0].2, format!("{base}/ark/dev/ark-x.exe.sha256"), "边车 URL 形态");
        assert_eq!(two[1].0, "ome", "ome 段回落");
        assert_eq!(two[1].3, format!("{base}/ome/dev/ome-x.exe"), "下载 URL 形态");
        let one = mirror_attempts(base, "stable", "ark-x.exe", None);
        assert_eq!(one.len(), 1, "无兼容名单尝试");
        assert!(one[0].2.contains("/ark/stable/"), "stable 通道段名随通道");
    }

    #[test]
    fn 自管条目_新旧extract双接受() {
        // D41（R8）：数据面改名可单方回退——引擎双接受，六消费分支同判定
        let mk = |e: &str| crate::catalog::Tool {
            extract: Some(e.to_string()),
            ..Default::default()
        };
        assert!(is_ome_self(&mk("ome-self")), "旧标记仍受认（数据面未改名期）");
        assert!(is_ome_self(&mk("ark-self")), "新标记受认");
        assert!(!is_ome_self(&mk("zip")), "非自管不误判");
        assert!(!is_ome_self(&mk("npm-tgz")), "npm 型不误判");
    }

    #[test]
    fn 短sha_取前8位() {
        assert_eq!(sha8("ABCDEF1234"), "ABCDEF12");
    }
}

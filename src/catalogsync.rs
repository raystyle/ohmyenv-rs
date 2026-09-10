//! catalogsync：运行态软件清单（catalog）云端刷新（D33，2026-09-10 用户方向）。
//!
//! 口径（用户三项定稿）：
//! - **权威落位**：仓库副本是开发与离线兜底源；云端 `env.ohmygh.com/ome/catalog/tools.toml`
//!   是运行态权威，部署机按 TTL 刷新用户数据副本。
//! - **刷新时机**：TTL 默认 24 小时（`OME_CATALOG_TTL` 秒级可调，0 关；`OME_OFFLINE=1` 关），
//!   另加显式 `ome catalog status` / `ome catalog sync`。
//! - **信任锚**：镜像 `.sha256` 边车自算即锚（先边车后资产，锚不符或解析不过一律拒收），
//!   与自举（`catalog::bootstrap_catalog`）同款语义；签名留后。
//!
//! 边界：
//! - **只刷用户数据副本**：仓库 cwd、二进制同级、`OME_CATALOG` 指定面一律不读不改（开发态零干扰）；
//! - 自动路径网络异常快速退化（单次 5s 探活、不重试、不拖慢用户命令），失败记退避标记；
//! - 部署位 pin 回写仍是临时态（R001 四.7）：云端刷新会以云端权威覆盖本地副本。

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::catalog::Catalog;

/// 云端清单在镜像里的键（seed-mirror 路线 B 推 `ome/catalog/tools.toml` 加 `.sha256` 边车）。
pub const CLOUD_CATALOG_KEY: &str = "ome/catalog/tools.toml";
/// 自动刷新默认 TTL（秒）：一天一次锚比对。
pub const DEFAULT_TTL_SECS: u64 = 24 * 60 * 60;
/// 自动路径探活超时（秒）：网络异常时快速退化，不拖慢用户命令。
const PROBE_TIMEOUT_SECS: u64 = 5;

/// 刷新结果。
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// 未联网：`off` 显式关闭（TTL 0 或离线）、`fresh` 未过期、`unreachable` 探活失败（已退避）。
    Skipped(&'static str),
    /// 本地与云端同锚（只更新检查标记）。
    InSync { sha: String },
    /// 已从云端拉取并落位。
    Updated { sha: String },
}

impl Outcome {
    /// 命令面动作词（数据块 `action` 字段值）。
    pub fn action(&self) -> &'static str {
        match self {
            Outcome::Skipped(_) => "skipped",
            Outcome::InSync { .. } => "current",
            Outcome::Updated { .. } => "updated",
        }
    }

    /// 跳过原因（非跳过为空串）；`unreachable` 供自动路径退避后如实标注。
    pub fn reason(&self) -> &'static str {
        match self {
            Outcome::Skipped(r) => r,
            _ => "",
        }
    }

    /// 关联的清单 sha（跳过时为空）。
    pub fn sha(&self) -> Option<&str> {
        match self {
            Outcome::Skipped(_) => None,
            Outcome::InSync { sha } | Outcome::Updated { sha } => Some(sha),
        }
    }
}

/// 用户数据副本路径：`<metadata>\catalog\tools.toml`（self-deploy 同步位，运行态权威的落点）。
pub fn user_data_catalog_path() -> PathBuf {
    crate::platform::metadata_dir().join("catalog").join("tools.toml")
}

/// 检查标记路径：与目标同目录的 `.last-sync`（记上次检查时刻与锚，不碰 catalog 文件 mtime）。
fn marker_path(target: &Path) -> Option<PathBuf> {
    target.parent().map(|d| d.join(".last-sync"))
}

/// 云端清单 URL（带锚击穿 query：锚变缓存键变，锚同则缓存对象必与锚一致）。
fn cloud_catalog_url(sha: &str) -> String {
    crate::download::with_query(
        &format!("{}/{CLOUD_CATALOG_KEY}", crate::download::MIRROR_BASE),
        &format!("v={sha}"),
    )
}

/// 云端边车锚 URL（带时间戳击穿每次回源由调用方补 query）。
fn cloud_sidecar_url() -> String {
    format!("{}/{CLOUD_CATALOG_KEY}.sha256", crate::download::MIRROR_BASE)
}

/// TTL 解析（纯函数）：离线优先，其次显式秒数（0 关），非法值回落默认。
pub fn resolve_ttl(ttl_env: Option<&str>, offline_env: Option<&str>) -> u64 {
    if offline_env.map(|v| v.trim() == "1").unwrap_or(false) {
        return 0;
    }
    match ttl_env.map(str::trim).filter(|v| !v.is_empty()) {
        None => DEFAULT_TTL_SECS,
        Some(v) => v.parse::<u64>().unwrap_or(DEFAULT_TTL_SECS),
    }
}

/// 当前 TTL（读 `OME_CATALOG_TTL` / `OME_OFFLINE`）。
pub fn auto_ttl() -> u64 {
    let ttl = std::env::var("OME_CATALOG_TTL").ok();
    let offline = std::env::var("OME_OFFLINE").ok();
    resolve_ttl(ttl.as_deref(), offline.as_deref())
}

/// 标记文件解析（纯函数）：`<unix 秒>\n<sha256>\n`。
pub fn parse_marker(text: &str) -> Option<(u64, String)> {
    let mut lines = text.lines();
    let at = lines.next()?.trim().parse::<u64>().ok()?;
    let sha = lines.next()?.trim();
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some((at, sha.to_uppercase()))
}

/// 标记文件文本（纯函数，与 parse_marker 对称；小写写入，读出一律大写）。
pub fn marker_text(at: u64, sha: &str) -> String {
    format!("{at}\n{}\n", sha.to_lowercase())
}

/// 是否需要联网比对（纯函数）：目标缺失、无标记、标记过期、或本地已被改写（标记锚与本地不符）。
pub fn needs_check(
    target_exists: bool,
    local_sha: Option<&str>,
    marker: Option<&(u64, String)>,
    now: u64,
    ttl: u64,
) -> bool {
    if !target_exists {
        return true;
    }
    let Some((at, sha)) = marker else {
        return true;
    };
    if now.saturating_sub(*at) >= ttl {
        return true;
    }
    match local_sha {
        Some(local) => !local.eq_ignore_ascii_case(sha),
        None => true,
    }
}

/// 解析面来源分类（纯函数，供 status 报告）：userdata / repo / env / other。
pub fn classify_origin(
    path: &Path,
    user_data: &Path,
    cwd_catalog: Option<&Path>,
    env_catalog: Option<&Path>,
) -> &'static str {
    let same = |a: &Path, b: &Path| normalize_path(a) == normalize_path(b);
    if env_catalog.is_some_and(|e| same(path, e)) {
        return "env";
    }
    if same(path, user_data) {
        return "userdata";
    }
    if cwd_catalog.is_some_and(|c| same(path, c)) {
        return "repo";
    }
    "other"
}

/// 路径归一化比较（大小写不敏感、去尾分隔符）。
fn normalize_path(p: &Path) -> String {
    p.to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 文件 sha（大写）；不存在或读失败为 None。
fn file_sha(path: &Path) -> Option<String> {
    crate::download::sha256_file(path).ok().map(|s| s.to_uppercase())
}

fn read_marker(target: &Path) -> Option<(u64, String)> {
    let path = marker_path(target)?;
    std::fs::read_to_string(path).ok().and_then(|t| parse_marker(&t))
}

fn write_marker(target: &Path, at: u64, sha: &str) {
    if let Some(path) = marker_path(target) {
        let _ = std::fs::write(path, marker_text(at, sha));
    }
}

/// 云端清单锚（边车首 token，大写；走下载链重试与 curl 兜底，命令面用）。
pub fn cloud_sha(env_root: &Path) -> Result<String, String> {
    crate::download::mirror_sidecar_sha(env_root, &cloud_sidecar_url())
}

/// 短超时探活取锚（自动路径用）：单次请求、无重试、不拖慢用户命令。
fn probe_cloud_sha() -> Result<String, String> {
    let url = crate::download::with_query(&cloud_sidecar_url(), &format!("t={}", now_secs()));
    let text =
        crate::download::fetch_text_short(&url, Duration::from_secs(PROBE_TIMEOUT_SECS))?;
    crate::download::parse_sidecar_sha(&text, &url)
}

/// 按给定锚拉取云端清单到缓存，过 sha 与解析双验证。
pub fn fetch_with_anchor(env_root: &Path, sha: &str) -> Result<(PathBuf, String), String> {
    let path = crate::download::download_fresh(
        env_root,
        "cloud-tools.toml",
        &cloud_catalog_url(sha),
    )?;
    let got = crate::download::sha256_file(&path)?;
    if !got.eq_ignore_ascii_case(sha) {
        return Err(format!("云端清单锚不符: 边车 {sha} 实拉 {got}"));
    }
    Catalog::load(&path)?;
    Ok((path, sha.to_uppercase()))
}

/// 取锚后拉取（命令面 sync 用）。
pub fn fetch_cloud(env_root: &Path) -> Result<(PathBuf, String), String> {
    let sha = cloud_sha(env_root)?;
    fetch_with_anchor(env_root, &sha)
}

/// 落位（先写同目录临时文件再替换，避免半截文件成为运行态）。
fn place(src: &Path, target: &Path) -> Result<(), String> {
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("建目录失败: {}: {e}", dir.display()))?;
    }
    let tmp = target.with_file_name("tools.toml.tmp");
    std::fs::copy(src, &tmp).map_err(|e| format!("写临时文件失败: {}: {e}", tmp.display()))?;
    if std::fs::rename(&tmp, target).is_err() {
        // 占用或跨卷时退回复制覆盖（极端路径，不阻断刷新）
        std::fs::copy(&tmp, target).map_err(|e| format!("落位失败: {}: {e}", target.display()))?;
        let _ = std::fs::remove_file(&tmp);
    }
    Ok(())
}

/// 刷新到指定目标（命令面与自动路径共用；target 独立解析，便于沙盒测试）。
pub fn sync_to(env_root: &Path, target: &Path, force: bool, ttl: u64) -> Result<Outcome, String> {
    if ttl == 0 && !force {
        return Ok(Outcome::Skipped("off"));
    }
    let now = now_secs();
    let local_sha = file_sha(target);
    if !force
        && !needs_check(
            target.exists(),
            local_sha.as_deref(),
            read_marker(target).as_ref(),
            now,
            ttl,
        )
    {
        return Ok(Outcome::Skipped("fresh"));
    }
    let (fetched, sha) = fetch_cloud(env_root)?;
    if local_sha.as_deref().is_some_and(|l| l.eq_ignore_ascii_case(&sha)) {
        write_marker(target, now, &sha);
        return Ok(Outcome::InSync { sha });
    }
    place(&fetched, target)?;
    write_marker(target, now, &sha);
    Ok(Outcome::Updated { sha })
}

/// 自动刷新（仅用户数据副本路径）：TTL 判定在联网之前；网络异常单次探活即退化，
/// 失败记退避标记（锚取本地值），TTL 内不再重试，保证不拖慢用户命令。
pub fn auto_refresh(env_root: &Path) -> Result<Outcome, String> {
    let ttl = auto_ttl();
    if ttl == 0 {
        return Ok(Outcome::Skipped("off"));
    }
    let target = user_data_catalog_path();
    let now = now_secs();
    let local_sha = file_sha(&target);
    if !needs_check(
        target.exists(),
        local_sha.as_deref(),
        read_marker(&target).as_ref(),
        now,
        ttl,
    ) {
        return Ok(Outcome::Skipped("fresh"));
    }
    let cloud = match probe_cloud_sha() {
        Ok(sha) => sha,
        Err(_) => {
            if let Some(local) = &local_sha {
                write_marker(&target, now, local);
            }
            return Ok(Outcome::Skipped("unreachable"));
        }
    };
    if local_sha.as_deref().is_some_and(|l| l.eq_ignore_ascii_case(&cloud)) {
        write_marker(&target, now, &cloud);
        return Ok(Outcome::InSync { sha: cloud });
    }
    let (fetched, sha) = fetch_with_anchor(env_root, &cloud)?;
    place(&fetched, &target)?;
    write_marker(&target, now, &sha);
    Ok(Outcome::Updated { sha })
}

/// 命令入口接线：解析面**就是**用户数据副本时按 TTL 刷新；跳过与失败都不拦命令。
pub fn auto_refresh_if_user_data(env_root: &Path, resolved: &Path) {
    if normalize_path(resolved) != normalize_path(&user_data_catalog_path()) {
        return;
    }
    if let Ok(Outcome::Updated { sha }) = auto_refresh(env_root) {
        let short = &sha[..sha.len().min(8)];
        eprintln!("[OK] catalog 已刷新: {}（云端 {short}）", resolved.display());
    }
}

/// 命令面状态：解析面路径与来源、本地与云端锚、检查年龄、TTL 与离线态。
pub struct State {
    pub path: PathBuf,
    pub origin: &'static str,
    pub local_sha: Option<String>,
    pub cloud_sha: Option<String>,
    pub cloud_error: Option<String>,
    pub age_secs: Option<u64>,
    pub ttl_secs: u64,
    pub offline: bool,
    pub synced: bool,
}

/// 采集状态（云端不可达时如实标注 error 字段，不报错退出）。
pub fn status(env_root: &Path, resolved: &Path) -> State {
    let user_data = user_data_catalog_path();
    let cwd_catalog = std::env::current_dir()
        .ok()
        .map(|c| c.join("catalog").join("tools.toml"));
    let env_catalog = std::env::var("OME_CATALOG")
        .ok()
        .map(|v| PathBuf::from(v.trim()))
        .filter(|p| !p.as_os_str().is_empty());
    let local_sha = file_sha(resolved);
    let ttl_secs = auto_ttl();
    let now = now_secs();
    let age_secs = read_marker(&user_data)
        .map(|(at, _)| now.saturating_sub(at))
        .or_else(|| {
            std::fs::metadata(&user_data)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| now.saturating_sub(d.as_secs()))
        });
    let (cloud_sha, cloud_error) = match cloud_sha(env_root) {
        Ok(s) => (Some(s), None),
        Err(e) => (None, Some(e)),
    };
    let synced = match (&local_sha, &cloud_sha) {
        (Some(l), Some(c)) => l.eq_ignore_ascii_case(c),
        _ => false,
    };
    State {
        path: resolved.to_path_buf(),
        origin: classify_origin(
            resolved,
            &user_data,
            cwd_catalog.as_deref(),
            env_catalog.as_deref(),
        ),
        local_sha,
        cloud_sha,
        cloud_error,
        age_secs,
        ttl_secs,
        offline: ttl_secs == 0,
        synced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA_A: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    const SHA_B: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";

    #[test]
    fn ttl解析_默认与关闭与离线() {
        assert_eq!(resolve_ttl(None, None), DEFAULT_TTL_SECS);
        assert_eq!(resolve_ttl(Some("0"), None), 0, "0 表示关闭自动刷新");
        assert_eq!(resolve_ttl(Some("600"), None), 600);
        assert_eq!(resolve_ttl(Some("abc"), None), DEFAULT_TTL_SECS, "非法值回落默认");
        assert_eq!(resolve_ttl(Some("600"), Some("1")), 0, "OME_OFFLINE=1 优先关闭");
        assert_eq!(resolve_ttl(None, Some("0")), DEFAULT_TTL_SECS, "离线只认 1");
    }

    #[test]
    fn 标记读写_对称且坏内容不采纳() {
        assert_eq!(
            parse_marker(&marker_text(1_700_000_000, SHA_A)),
            Some((1_700_000_000, SHA_A.to_string())),
            "小写写入大写读出"
        );
        assert_eq!(parse_marker(""), None);
        assert_eq!(parse_marker("123\n"), None, "缺 sha 行");
        assert_eq!(parse_marker("x\nabcd\n"), None, "时刻非法");
        assert_eq!(parse_marker("1\nzz\n"), None, "sha 非法");
    }

    #[test]
    fn 需要联网_四种判据() {
        let marker = (1000u64, SHA_A.to_string());
        assert!(needs_check(false, None, None, 1000, 60), "目标缺失即需检查");
        assert!(needs_check(true, None, None, 1000, 60), "无标记即需检查");
        assert!(
            !needs_check(true, Some(SHA_A), Some(&marker), 1030, 60),
            "标记新鲜且本地未改写则跳过"
        );
        assert!(
            needs_check(true, Some(SHA_A), Some(&marker), 1060, 60),
            "超过 TTL 需检查"
        );
        assert!(
            needs_check(true, Some(SHA_B), Some(&marker), 1030, 60),
            "本地被改写（self-deploy 或人为）即需检查"
        );
    }

    #[test]
    fn 来源分类_四态且大小写与尾分隔符不敏感() {
        let user_data = PathBuf::from(r"C:\data\ohmyenv\catalog\tools.toml");
        let cwd = PathBuf::from(r"D:\ohmyenv-rs\catalog\tools.toml");
        let env = PathBuf::from(r"C:\tmp\custom.toml");
        assert_eq!(
            classify_origin(
                Path::new(r"C:\DATA\ohmyenv\catalog\tools.toml\"),
                &user_data,
                Some(&cwd),
                None
            ),
            "userdata"
        );
        assert_eq!(classify_origin(&cwd, &user_data, Some(&cwd), None), "repo");
        assert_eq!(classify_origin(&env, &user_data, Some(&cwd), Some(&env)), "env");
        assert_eq!(
            classify_origin(Path::new("/x/tools.toml"), &user_data, None, None),
            "other"
        );
    }

    #[test]
    fn 动作词与原因_命令面稳定() {
        assert_eq!(Outcome::Skipped("fresh").action(), "skipped");
        assert_eq!(Outcome::Skipped("fresh").reason(), "fresh");
        assert_eq!(Outcome::Skipped("unreachable").reason(), "unreachable");
        assert_eq!(Outcome::Updated { sha: SHA_A.into() }.action(), "updated");
        assert_eq!(Outcome::InSync { sha: SHA_A.into() }.action(), "current");
        assert_eq!(Outcome::InSync { sha: SHA_A.into() }.sha(), Some(SHA_A));
        assert_eq!(Outcome::Skipped("off").sha(), None);
    }
}

//! install：安装主编排，对齐 helpers.ps1 的 Install-ToolVersion（1019-1267 行）。
//! 流程：防穿越校验 → 幂等跳过（顺带补 PATH/sha 回填/滞后锁定）→ sha 基准（pin > 官方源）
//! → 下载（含 bootstrap 资产与 MZ 头校验）→ 删旧目录全量重建 → extract 分派
//! → 装后验版本（5 次递增重试）→ 回写 lock（write_pin / write_sha256）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::catalog::{self, Catalog, Tool};
use crate::checksum;
use crate::download;
use crate::extract;
use crate::resolve::Resolution;
use crate::toolver;

/// 安装选项（对齐 Install-ToolVersion 的 -RegisterPath / -UpdateLock / -Force）。
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub register_path: bool,
    pub update_lock: bool,
    pub force: bool,
}

/// 安装结果动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallAction {
    Installed,
    Skipped,
}

impl InstallAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallAction::Installed => "installed",
            InstallAction::Skipped => "skipped",
        }
    }
}

/// 安装结果：动作、版本、安装目录（msi 无绿色目录，为 None）。
pub struct InstallOutcome {
    pub action: InstallAction,
    pub version: String,
    pub dir: Option<PathBuf>,
}

/// 防穿越：path 必须在允许的安全根之下。
/// Windows：在 EnvRoot 之下（大小写不敏感）；
/// Linux / macOS：在 $HOME 之下，或显式在 EnvRoot 之下。
pub fn is_safe_under_root(root: &Path, path: &Path) -> bool {
    let (Ok(full_root), Ok(full_path)) = (std::path::absolute(root), std::path::absolute(path))
    else {
        return false;
    };
    #[cfg(windows)]
    {
        let root_str = full_root.to_string_lossy().replace('/', "\\");
        let root_str = root_str.trim_end_matches('\\').to_lowercase() + "\\";
        let path_str = full_path
            .to_string_lossy()
            .replace('/', "\\")
            .to_lowercase();
        path_str.starts_with(&root_str)
    }
    #[cfg(not(windows))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| full_root.clone());
        let allowed_roots = [home, full_root];
        allowed_roots.iter().any(|r| {
            let r_str = r.to_string_lossy().trim_end_matches('/').to_string() + "/";
            full_path.to_string_lossy().starts_with(&r_str)
        })
    }
}

/// 安装目录解析：msi 无绿色目录；official 走 exe 上两级（官方目录）；其余 EnvRoot\dir。
fn install_dir(
    def: &Tool,
    env_root: &Path,
    is_msi: bool,
    is_official: bool,
) -> Result<Option<PathBuf>, String> {
    if is_msi {
        return Ok(None);
    }
    if is_official {
        let exe = toolver::exe_path(def, env_root)?;
        let dir = exe
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "official exe 路径无法上溯两级".to_string())?;
        return Ok(Some(dir.to_path_buf()));
    }
    let dir = def.dir().ok_or_else(|| "工具缺少 dir 字段".to_string())?;
    Ok(Some(crate::platform::join_if_relative(
        env_root,
        crate::platform::expand_install_path(dir),
    )))
}

/// 注册 PATH 的目录：official 取 exe 上一级（展开后），其余按平台字段解析（支持 `~` 与 `$VAR`）。
fn bin_dir(def: &Tool, env_root: &Path, is_official: bool) -> Result<Option<PathBuf>, String> {
    if is_official {
        let exe = toolver::exe_path(def, Path::new("."))?;
        return Ok(exe.parent().map(Path::to_path_buf));
    }
    Ok(def.bin().map(|b| {
        crate::platform::join_if_relative(env_root, crate::platform::expand_install_path(b))
    }))
}

/// 安装单工具（下载 → 校验 → 解压 → 验版本 → 回写）。
pub fn install_tool(
    cat: &Catalog,
    env_root: &Path,
    name: &str,
    res: &Resolution,
    opts: &InstallOptions,
) -> Result<InstallOutcome, String> {
    let def = cat.tool(name)?;
    let is_msi = def.extract() == Some("msi");
    let is_official = toolver::is_official(def);
    let install_dir = install_dir(def, env_root, is_msi, is_official)?;
    let exe_path = toolver::exe_path(def, env_root)?;

    // 防穿越：绿色目录必须在 EnvRoot 下（official/msi 除外）
    if !is_msi && !is_official {
        let dir = install_dir
            .as_ref()
            .ok_or_else(|| format!("{name} 无法确定安装目录"))?;
        if !is_safe_under_root(env_root, dir) {
            return Err(format!("危险路径，拒绝操作: {}", dir.display()));
        }
    }

    // ── 幂等跳过：已装版本 == 解析版本且非 force ──
    let cur = toolver::installed_version(&exe_path, name);
    if !opts.force && cur.as_deref() == Some(res.version.as_str()) {
        eprintln!(
            "[INFO] {name} {} 已安装，跳过（--force 强制重装）",
            res.version
        );
        let cache = download::cache_path(env_root, &res.asset_name);
        // 顺带回填 sha256（命中缓存时）
        if def.sha256.is_none() && cache.exists() {
            let sha = download::sha256_file(&cache)?;
            catalog::write_sha256(&cat.path, name, &sha)?;
            eprintln!("[OK] 已回填 sha256（命中缓存）");
        }
        if opts.register_path {
            register_bin(def, env_root, is_official)?;
        }
        // 老环境补 bunx shim（bun 已存在但同目录缺 bunx.exe）
        if name == "bun" {
            if let Some(dir) = &install_dir {
                extract::ensure_bunx_shim(dir)?;
            }
        }
        if opts.update_lock && def.tag.as_deref() != Some(res.tag.as_str()) {
            // 已安装版本与解析一致但锁定滞后（如上次安装中断）：补齐锁定
            catalog::write_pin(&cat.path, name, res)?;
            if cache.exists() {
                let sha = download::sha256_file(&cache)?;
                catalog::write_sha256(&cat.path, name, &sha)?;
            }
            eprintln!("[OK] {name} 已锁定: {}（补齐滞后锁定）", res.version);
        }
        return Ok(InstallOutcome {
            action: InstallAction::Skipped,
            version: res.version.clone(),
            dir: install_dir,
        });
    }

    // ── sha 校验优先级：pin 的 sha256 > 官方校验源三型 ──
    let expected_sha = checksum::expected_sha256(def, res, env_root)?;

    // 下载（tag 与锁定不一致时强制重下，对齐 -Force:$forceDownload）
    let force_download = def.tag.as_deref() != Some(res.tag.as_str());
    let cache = download::download_asset(
        env_root,
        &res.asset_name,
        &res.asset_url,
        expected_sha.as_deref(),
        force_download,
    )?;

    // 额外 bootstrap 资产（如 7z 的 7zr.exe）：仅 Windows 下 7z-extra 使用；先下载最小解压器，MZ 头校验
    #[cfg(windows)]
    if let Some(bootstrap) = def.bootstrap_asset() {
        let repo = def
            .repo()
            .ok_or_else(|| format!("{name} 使用 bootstrap_asset 但缺少 repo 字段"))?;
        let boot_url = format!(
            "https://github.com/{repo}/releases/download/{}/{bootstrap}",
            res.tag
        );
        let boot_path = download::download_asset(env_root, bootstrap, &boot_url, None, false)?;
        let head = fs::read(&boot_path)
            .map_err(|e| format!("读取 BootstrapAsset 失败: {}: {e}", boot_path.display()))?;
        if head.len() < 2 {
            return Err(format!("{name} BootstrapAsset 缺失或为空: {bootstrap}"));
        }
        if head[0] != 0x4D || head[1] != 0x5A {
            return Err(format!(
                "{name} BootstrapAsset 不是有效 Windows 可执行文件: {bootstrap}"
            ));
        }
    }

    // 下载后 sha 处理：同 tag 且同资产（同平台）才用 pin 的 sha256 校验，跨 tag/跨平台接受新值并回填。
    let sha = download::sha256_file(&cache)?;
    let pinned_asset = def.asset.as_deref().unwrap_or("");
    let same_asset = pinned_asset.is_empty() || pinned_asset == res.asset_name;
    let sha_backfilled = if def.tag.as_deref() == Some(res.tag.as_str()) && same_asset {
        if let Some(pinned) = &def.sha256 {
            if !sha.eq_ignore_ascii_case(pinned) {
                return Err(format!("{name} 缓存 sha256 与锁定不符"));
            }
            false
        } else {
            true
        }
    } else {
        true
    };

    // 删旧目录全量重建（Windows 绿色目录类）。Linux 多个工具可能共享 ~/.local/bin，不整删。
    #[cfg(windows)]
    if !is_msi && !is_official {
        if let Some(dir) = &install_dir {
            if dir.exists() {
                fs::remove_dir_all(dir)
                    .map_err(|e| format!("删除旧目录失败: {}: {e}", dir.display()))?;
            }
            fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}: {e}", dir.display()))?;
        }
    }
    #[cfg(not(windows))]
    if !is_msi && !is_official {
        if let Some(dir) = &install_dir {
            fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}: {e}", dir.display()))?;
            // 仅删除目标 exe 文件（不清理共享 bin 目录下的其他工具）
            let _ = fs::remove_file(&exe_path);
        }
    }

    // extract 分派（msi/rmux 分支不使用 install_dir，传 env_root 占位）
    let target_dir = install_dir.as_deref().unwrap_or(env_root);
    extract::extract_asset(name, def, &cache, target_dir, env_root)?;

    // 装后验版本（5 次递增重试：7zsfx 等解包后文件/杀软可能瞬态未就绪）
    let installed = toolver::installed_version_retried(&exe_path, name).ok_or_else(|| {
        format!(
            "{name} 安装后未找到可执行文件或无法读取版本: {}",
            exe_path.display()
        )
    })?;
    if installed != res.version {
        return Err(format!(
            "{name} 版本不符: 期望 {}，实际 {installed}",
            res.version
        ));
    }
    eprintln!("[OK] {name} 安装完成: {installed} @ {}", exe_path.display());

    // 成功才回写 lock
    if sha_backfilled {
        catalog::write_sha256(&cat.path, name, &sha)?;
    }
    if opts.register_path {
        register_bin(def, env_root, is_official)?;
    }
    if opts.update_lock {
        catalog::write_pin(&cat.path, name, res)?;
        catalog::write_sha256(&cat.path, name, &sha)?;
        eprintln!("[OK] {name} 已锁定: {}", res.version);
    }

    Ok(InstallOutcome {
        action: InstallAction::Installed,
        version: installed,
        dir: install_dir,
    })
}

/// 注册 bin 目录进用户 PATH（Windows 注册表 / Linux profile）。
fn register_bin(def: &Tool, env_root: &Path, is_official: bool) -> Result<(), String> {
    if let Some(dir) = bin_dir(def, env_root, is_official)? {
        crate::platform::add_user_path(&dir)?;
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn 防穿越_root内放行_同级与上级拒绝() {
        let root = Path::new(r"D:\sandbox\env");
        assert!(is_safe_under_root(root, Path::new(r"D:\sandbox\env\jq")));
        assert!(
            is_safe_under_root(root, Path::new(r"d:\SANDBOX\env\jq")),
            "大小写不敏感"
        );
        assert!(!is_safe_under_root(root, Path::new(r"D:\sandbox\evil")));
        assert!(!is_safe_under_root(root, Path::new(r"D:\sandbox")));
        // root 自身不算「之下」（root 补尾斜杠后自身不匹配）
        assert!(!is_safe_under_root(root, root));
        // 兄弟前缀不能误判（env2 不是 env 之下）
        assert!(!is_safe_under_root(root, Path::new(r"D:\sandbox\env2\jq")));
    }

    #[test]
    fn 防穿越_相对路径按当前目录展开() {
        // std::path::absolute 不做符号链接解析，与 GetFullPath 语义一致
        let root = Path::new(".");
        let abs_root = std::path::absolute(root).expect("应可取绝对路径");
        let inside = abs_root.join("sub");
        assert!(is_safe_under_root(root, &inside));
    }
}

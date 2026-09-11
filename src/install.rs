//! install：安装主编排，对齐 helpers.ps1 的 Install-ToolVersion（1019-1267 行）。
//! 流程：防穿越校验 → 幂等跳过（顺带补 PATH/sha 回填/滞后锁定）→ sha 基准（pin > 官方源）
//! → 下载（含 bootstrap 资产与 MZ 头校验）→ 删旧目录全量重建 → extract 分派
//! → 装后验版本（5 次递增重试）→ 回写 lock（write_pin / write_sha256）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::catalog::{self, Catalog, Tool};
use crate::checksum;
use crate::download;
use crate::extract;
use crate::resolve::Resolution;
use crate::toolver;

/// 安装选项（对齐 Install-ToolVersion 的 -RegisterPath / -UpdateLock / -Force）。
/// `configure` 为 deploy 侧：PATH、用户环境变量、注册表与配置；download 为 false。
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub configure: bool,
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
    // D39 R016：安装配置部署逻辑数据面——与 tools.toml 同目录的 manifest.toml，
    // 工具节有原语用数据、无节走内建回退（双轨过渡，omc manifest 上线后撤内建）
    let mf = crate::manifest::load(cat.path.parent().unwrap_or(Path::new(".")))
        .unwrap_or_else(|e| {
            eprintln!("[WARN] {e}（忽略，走内建回退）");
            crate::manifest::ManifestFile::default()
        });
    // manifest 节键：catalog 的 manifest 字段声明的节键优先，缺省同名节（R016 二）
    let ms = mf.manifest.get(def.manifest.as_deref().unwrap_or(name));
    let is_msi = def.extract() == Some("msi");
    let is_official = toolver::is_official(def);
    let install_dir = install_dir(def, env_root, is_msi, is_official)?;
    let exe_path = toolver::exe_path(def, env_root)?;
    // shim 落点：目标二进制所在目录（R016「同 bin 目录」；POSIX 嵌套布局与 official 型下
    // install_dir 可能不是二进制所在目录，取 exe 父目录才与 PATH 注册面一致）。msi 无 EnvRoot 落点，跳过。
    let shim_dir = install_dir.as_deref().and_then(|_| exe_path.parent());

    // 防穿越：绿色目录必须在 EnvRoot 下（official/msi 除外）
    if !is_msi && !is_official {
        let dir = install_dir
            .as_ref()
            .ok_or_else(|| format!("{name} 无法确定安装目录"))?;
        if !is_safe_under_root(env_root, dir) {
            return Err(format!("危险路径，拒绝操作: {}", dir.display()));
        }
    }

    // ── agent 类存量纳管（D07，2026-09-05 用户三裁延续）：PATH 任意位在位即跳过，
    //    不迁移不重装（对齐 oma agents install 的「已装任何来源即跳过」判定）；--force 才装 EnvRoot ──
    if def.category.as_deref() == Some("agent") && !opts.force {
        if let Some(found) = toolver::find_on_path(name) {
            eprintln!(
                "[INFO] {name} 已在 PATH 安装（{}），存量原地纳管跳过（--force 装进 EnvRoot）",
                found.display()
            );
            return Ok(InstallOutcome {
                action: InstallAction::Skipped,
                version: res.version.clone(),
                dir: install_dir,
            });
        }
    }

    // ── 幂等跳过：已装版本 == 解析版本且非 force ──
    let cur = toolver::installed_version(&exe_path, def);
    if !opts.force && cur.as_deref() == Some(res.version.as_str()) {
        eprintln!(
            "[INFO] {name} {} 已安装，跳过（--force 强制重装）",
            res.version
        );
        let cache = download::cache_path(env_root, &res.asset_name);
        // 顺带回填 sha256（命中缓存时）
        if def.pin_sha256().is_none() && cache.exists() {
            let sha = download::sha256_file(&cache)?;
            catalog::write_sha256(&cat.path, name, &sha)?;
            eprintln!("[OK] 已回填 sha256（命中缓存）");
        }
        if opts.configure {
            register_bin(def, env_root, is_official)?;
            ensure_user_env_overrides(name, ms)?;
        }
        // 老环境补 bunx shim（bun 已存在但同目录缺 bunx.exe）；
        // D39：manifest shims 原语优先（R016 L1 通用别名），未声明才回退 bunx 内建（双轨过渡）
        if ms.is_some_and(|m| m.shims.is_some()) {
            if let (Some(m), Some(dir)) = (ms, shim_dir) {
                crate::manifest::apply_shims(m, dir)?;
            }
        } else if name == "bun" {
            if let Some(dir) = shim_dir {
                extract::ensure_bunx_shim(dir)?;
            }
        }
        if opts.update_lock && def.pin_tag() != Some(res.tag.as_str()) {
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

    // uv-git 型：uv tool install git 安装（无下载资产无 sha，幂等逻辑上面已覆盖）
    if def.extract() == Some("uv-git") {
        return install_uv_git(cat, name, def, res, opts, &exe_path, install_dir);
    }

    // npm-tgz 型：release tgz 过锚下载后 npm install -g（Node CLI；幂等逻辑上面已覆盖）
    if def.extract() == Some("npm-tgz") {
        return install_npm_tgz(cat, name, def, res, opts, env_root, install_dir);
    }

    // ── sha 校验优先级：pin 的 sha256 > 官方校验源三型 ──
    let expected_sha = checksum::expected_sha256(def, res, env_root)?;

    // 下载（tag 与锁定不一致时强制重下，对齐 -Force:$forceDownload）；
    // 官方失败回落 env.ohmygh.com 镜像（D08，仅当有 sha 锚：pin 或官方 sums 均可作锚）
    let force_download = def.pin_tag() != Some(res.tag.as_str());
    let cache = download::download_asset_with_mirror(
        env_root,
        &res.asset_name,
        &res.asset_url,
        expected_sha.as_deref(),
        force_download,
        name,
        &res.version,
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

    // 下载后 sha：同 tag 且同资产才用 pin sha 核对；仅同发行缺 sha 时回填。
    // D37 起 update 与 install 都不回写 pin（锁定单源归数据面），跨 tag 新 sha 自然不落 pin 字段。
    let sha = download::sha256_file(&cache)?;
    let pinned_asset = def.pin_asset().unwrap_or("");
    let same_asset = pinned_asset.is_empty() || pinned_asset == res.asset_name;
    let same_release = def.pin_tag() == Some(res.tag.as_str()) && same_asset;
    let sha_backfilled = if same_release {
        if let Some(pinned) = def.pin_sha256() {
            if !sha.eq_ignore_ascii_case(pinned) {
                return Err(format!("{name} 缓存 sha256 与锁定不符"));
            }
            false
        } else {
            true
        }
    } else {
        false
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
    let installed = toolver::installed_version_retried(&exe_path, def).ok_or_else(|| {
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
    if opts.configure {
        register_bin(def, env_root, is_official)?;
        ensure_user_env_overrides(name, ms)?;
        // D39 R016 L1/L2：manifest 节的 shims 与受控命令在装成后执行（三平台矩阵验收）
        if let (Some(m), Some(dir)) = (ms, shim_dir) {
            crate::manifest::apply_shims(m, dir)?;
        }
        if let Some(m) = ms {
            crate::manifest::run_post_install(m, name)?;
        }
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

/// deploy 侧：按工具补运行时遥测关闭等用户级环境变量（幂等；download 不写注册表）。
/// - pwsh：msi 属性 DISABLE_TELEMETRY（extract 已传）之外的双保险运行时变量，顺带关更新检查
///   （对齐 ohmypwsh set-pwsh.ps1 L124-126）
/// - dotnet：绿色安装无安装期开关，遥测只能走运行时变量
fn ensure_user_env_overrides(
    name: &str,
    ms: Option<&crate::manifest::ToolManifest>,
) -> Result<(), String> {
    // D39：manifest 节 env_set 原语优先（R016 L1），该原语缺省才回退内建表（双轨过渡；
    // 判定粒度是原语而非整工具：节只声明 shims 时 env_set 仍走内建）
    if let Some(m) = ms {
        if m.env_set.is_some() {
            return crate::manifest::apply_env_set(m);
        }
    }
    let vars: &[(&str, &str)] = match name {
        "pwsh" => &[
            ("POWERSHELL_TELEMETRY_OPTOUT", "1"),
            ("POWERSHELL_UPDATECHECK", "Off"),
        ],
        "dotnet" => &[("DOTNET_CLI_TELEMETRY_OPTOUT", "1")],
        _ => return Ok(()),
    };
    for (k, v) in vars {
        crate::platform::set_user_env_var(k, v)?;
        eprintln!("[OK] 用户环境变量已设: {k}={v}（新终端生效）");
    }
    Ok(())
}

/// 注册 bin 目录进用户 PATH（Windows 注册表 / Linux profile）。
fn register_bin(def: &Tool, env_root: &Path, is_official: bool) -> Result<(), String> {
    // npm-tgz：bin 落 npm 全局 bin（npm 自管 PATH 面），不注册 EnvRoot 目录（防死链）
    if def.extract() == Some("npm-tgz") {
        return Ok(());
    }
    if let Some(dir) = bin_dir(def, env_root, is_official)? {
        if crate::platform::add_user_path(&dir)? {
            eprintln!("[OK] PATH 已注册: {}（新终端生效）", dir.display());
        } else {
            eprintln!("[INFO] PATH 已含: {}", dir.display());
        }
    }
    Ok(())
}

/// uv-git 安装：uv tool install --force git+repo@tag（依赖 uv 运行时；shim 落 uv 管的
/// UV_TOOL_BIN_DIR，register 走 official 分支注册该目录）。
/// 升级占用解锁（browser-harness M102 方案吸收）：Windows 运行中进程锁 venv Scripts\，
/// uv 无法替换——升级前用工具自身 CLI 停守护（--reload 停 daemon、rmux kill-server 停
/// worker 会话），装后提示 x-monitor 恢复值守栈（ome 不擅自拉起）。
fn install_uv_git(
    cat: &Catalog,
    name: &str,
    def: &Tool,
    res: &Resolution,
    opts: &InstallOptions,
    exe_path: &Path,
    install_dir: Option<PathBuf>,
) -> Result<InstallOutcome, String> {
    let uv = which::which("uv").map_err(|_| {
        format!("{name} 为 uv tool 安装型，需要 uv 运行时在位（先 ome install uv）")
    })?;
    let repo = def
        .repo()
        .ok_or_else(|| format!("{name} 缺少 repo 字段（uv-git 型必需）"))?;
    let url = format!("git+https://github.com/{repo}@{}", res.tag);

    // 升级（旧 exe 在位）：先停工具自己的守护栈解锁 venv（best-effort，CLI 子命令以工具为准）
    let upgrading = exe_path.exists();
    if upgrading {
        for args in [["--reload"].as_slice(), ["rmux", "kill-server"].as_slice()] {
            if let Ok(status) = Command::new(exe_path).args(args).status() {
                if !status.success() {
                    eprintln!("[WARN] {} {:?} 停止未成功（继续尝试解锁）", name, args);
                }
            }
        }
    }

    eprintln!("[INFO] uv tool install {url}（依赖 uv 运行时，首次较慢）");
    let status = Command::new(&uv)
        .args(["tool", "install", "--force"])
        .arg(&url)
        .status()
        .map_err(|e| format!("uv 启动失败: {e}"))?;
    if !status.success() {
        return Err(format!(
            "{name} uv tool install 失败 exit={}（venv 可能被运行中进程锁定，停掉后重试）",
            status.code().unwrap_or(-1)
        ));
    }
    let version = toolver::installed_version_retried(exe_path, def)
        .ok_or_else(|| format!("{name} 装后版本探测失败（检查 catalog probe_pattern 字段）"))?;
    if opts.update_lock && def.pin_tag() != Some(res.tag.as_str()) {
        catalog::write_pin(&cat.path, name, res)?;
        eprintln!("[OK] {name} 已锁定: {}", res.version);
    }
    if opts.configure {
        register_bin(def, Path::new("."), true)?;
    }
    if upgrading {
        eprintln!("[HINT] 升级已停守护栈；恢复值守: {name} x-monitor");
    }
    Ok(InstallOutcome {
        action: InstallAction::Installed,
        version,
        dir: install_dir,
    })
}

/// npm-tgz 安装：release tgz 下载过锚后 npm install -g（Node CLI，如 browser-harness 的 bh）。
/// bin 落 npm 全局 bin（随各端 node 生态走，无静态路径）：exe 定位与幂等探测走 PATH 现查
/// （toolver::exe_path 的 npm-tgz 分支）；需 node 与 npm 在 PATH（fnm 供给）。
fn install_npm_tgz(
    cat: &Catalog,
    name: &str,
    def: &Tool,
    res: &Resolution,
    opts: &InstallOptions,
    env_root: &Path,
    install_dir: Option<PathBuf>,
) -> Result<InstallOutcome, String> {
    let npm = which::which("npm").map_err(|_| {
        format!("{name} 为 npm 全局装型，需要 node 与 npm 在 PATH（先 ome install fnm 装 node）")
    })?;
    let exe_path = toolver::exe_path(def, env_root)?;

    let expected = checksum::expected_sha256(def, res, env_root)?;
    let cache = download::download_asset_with_mirror(
        env_root,
        &res.asset_name,
        &res.asset_url,
        expected.as_deref(),
        true,
        name,
        &res.version,
    )?;

    eprintln!(
        "[INFO] npm install -g {}（依赖拉取走 npm registry，首次较慢）",
        cache.display()
    );
    let status = Command::new(&npm)
        .args(["install", "-g", "--no-fund", "--no-audit"])
        .arg(&cache)
        .status()
        .map_err(|e| format!("npm 启动失败: {e}"))?;
    if !status.success() {
        return Err(format!(
            "{name} npm install -g 失败 exit={}",
            status.code().unwrap_or(-1)
        ));
    }

    let version = toolver::installed_version_retried(&exe_path, def).ok_or_else(|| {
        format!("{name} 装后版本探测失败（检查 PATH 与 catalog probe_pattern 字段）")
    })?;
    if opts.update_lock && def.pin_tag() != Some(res.tag.as_str()) {
        catalog::write_pin(&cat.path, name, res)?;
        if cache.exists() {
            let sha = download::sha256_file(&cache)?;
            catalog::write_sha256(&cat.path, name, &sha)?;
        }
        eprintln!("[OK] {name} 已锁定: {}", res.version);
    }
    eprintln!("[OK] {name} 安装完成: {version} @ {}", exe_path.display());
    Ok(InstallOutcome {
        action: InstallAction::Installed,
        version,
        dir: install_dir,
    })
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

//! package：把 catalog 工具打包成可 scp 的分发目录。
//!
//! 对齐 ohmypwsh `download-mac-tools.ps1` 的输出约定：
//! `<out>/<tool>/bin/<tool>`（单二进制）或 `<out>/<tool>/`（多文件运行时）。
//! 本模块只准备目录，不调用 scp，也不改写 catalog pin。

use std::fs;
use std::path::{Path, PathBuf};

use crate::catalog::{Catalog, Tool};
use crate::checksum;
use crate::download;
use crate::extract;
use crate::resolve::{resolve_tool, Resolution, ResolveOptions};

/// 打包结果。
pub struct PackageOutcome {
    pub tool: String,
    pub version: String,
    pub package_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub main_bin: PathBuf,
}

/// 把工具打包到指定目录。
pub fn package_tool(
    cat: &Catalog,
    env_root: &Path,
    name: &str,
    res: &Resolution,
    out_dir: &Path,
) -> Result<PackageOutcome, String> {
    let def = cat.tool(name)?;
    let package_dir = out_dir.join(name);
    let bin_dir = package_dir.join("bin");

    // sha 基准 + 下载
    let expected = checksum::expected_sha256(def, res, env_root)?;
    let cache = download::download_asset(env_root, &res.asset_name, &res.asset_url, expected.as_deref(), false)?;

    // 清空目标目录
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)
            .map_err(|e| format!("删除旧打包目录失败: {}: {e}", package_dir.display()))?;
    }
    fs::create_dir_all(&package_dir)
        .map_err(|e| format!("创建打包目录失败: {}: {e}", package_dir.display()))?;

    // 提取到 <out>/<tool>/
    extract::extract_asset(name, def, &cache, &package_dir, env_root)
        .map_err(|e| format!("{name} 解包失败: {e}"))?;

    // 定位主二进制（此时还不能创建 bin/，避免被 flatten 误处理）
    let main_leaf = main_binary_leaf(def, name)?;
    let main_bin = find_or_place_main_binary(name, &package_dir, &main_leaf)?;

    // 单二进制工具额外复制到 bin/，保持 mac-tools 约定
    fs::create_dir_all(&bin_dir)
        .map_err(|e| format!("创建 bin 目录失败: {}: {e}", bin_dir.display()))?;
    if main_bin.parent() != Some(&bin_dir) {
        let bin_target = bin_dir.join(&main_leaf);
        fs::copy(&main_bin, &bin_target)
            .map_err(|e| format!("复制主二进制到 bin 失败: {}: {e}", bin_target.display()))?;
        #[cfg(not(windows))]
        set_executable(&bin_target)?;
    }

    Ok(PackageOutcome {
        tool: name.to_string(),
        version: res.version.clone(),
        package_dir,
        bin_dir,
        main_bin,
    })
}

/// 解析并打包工具。
pub fn package_tool_resolved(
    cat: &Catalog,
    env_root: &Path,
    name: &str,
    opts: &ResolveOptions,
    out_dir: &Path,
) -> Result<PackageOutcome, String> {
    let def = cat.tool(name)?;
    let res = resolve_tool(name, def, opts)?;
    package_tool(cat, env_root, name, &res, out_dir)
}

/// 取主二进制叶子名：优先 linux_exe / exe 的叶子，兜底工具名。
fn main_binary_leaf(def: &Tool, name: &str) -> Result<String, String> {
    let exe = def.exe().unwrap_or(name);
    let leaf = exe.rsplit(['\\', '/']).next().unwrap_or(exe);
    Ok(leaf.to_string())
}

/// 在 package_dir 下查找主二进制；找不到则尝试在 bin/ 下创建占位提示。
fn find_or_place_main_binary(
    tool: &str,
    package_dir: &Path,
    leaf: &str,
) -> Result<PathBuf, String> {
    // 1) package_dir 下递归查找
    if let Some(found) = find_file(package_dir, leaf) {
        return Ok(found);
    }
    // 2) 兜底：把工具名当二进制名在 package_dir 下找（copy 提取类型已落地）
    let fallback = package_dir.join(leaf);
    if fallback.exists() {
        return Ok(fallback);
    }
    Err(format!("{tool} 打包后未找到主二进制: {leaf}"))
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = fs::metadata(path)
        .map_err(|e| format!("读取权限失败: {}: {e}", path.display()))?
        .permissions();
    perm.set_mode(perm.mode() | 0o755);
    fs::set_permissions(path, perm)
        .map_err(|e| format!("设置可执行权限失败: {}: {e}", path.display()))
}

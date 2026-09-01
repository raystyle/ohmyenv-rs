//! selfdeploy：自部署——复制当前 exe 到 <EnvRoot>\ome\bin\ome.exe 并注册用户 PATH。
//! 幂等：目标与当前 exe 同路径则跳过复制；sha256 一致则跳过复制；PATH 注册由 envpath 幂等处理。

use std::path::{Path, PathBuf};

use crate::download::sha256_file;
#[cfg(windows)]
use crate::envpath;

/// 自部署结果。
pub struct SelfDeployOutcome {
    pub copied: bool,
    pub path_registered: bool,
    pub bin_dir: PathBuf,
    pub exe: PathBuf,
}

/// 复制 exe 到目标（纯文件逻辑，可测）：同路径跳过；sha256 一致跳过；否则覆盖复制。
/// 返回是否实际复制。
pub fn deploy_copy(src: &Path, dst: &Path) -> Result<bool, String> {
    let abs = |p: &Path| {
        std::path::absolute(p)
            .map(|x| x.to_string_lossy().to_lowercase())
            .map_err(|e| format!("取绝对路径失败: {}: {e}", p.display()))
    };
    if abs(src)? == abs(dst)? {
        return Ok(false); // 当前 exe 即目标（已在 bin 里运行）
    }
    if dst.exists() && sha256_file(src)? == sha256_file(dst)? {
        return Ok(false); // 内容一致，幂等跳过
    }
    if let Some(dir) = dst.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("创建目录失败: {}: {e}", dir.display()))?;
    }
    std::fs::copy(src, dst)
        .map_err(|e| format!("复制失败: {} -> {}: {e}", src.display(), dst.display()))?;
    Ok(true)
}

/// 自部署：复制当前 exe 到 <EnvRoot>\ome\bin\ome.exe，注册该 bin 目录进用户 PATH。
#[cfg(windows)]
pub fn self_deploy(env_root: &Path) -> Result<SelfDeployOutcome, String> {
    let src = std::env::current_exe().map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
    let bin_dir = env_root.join("ome").join("bin");
    let dst = bin_dir.join("ome.exe");
    let copied = deploy_copy(&src, &dst)?;
    if copied {
        eprintln!("[OK] 已复制: {} -> {}", src.display(), dst.display());
    } else {
        eprintln!("[INFO] 目标已是最新，跳过复制: {}", dst.display());
    }
    let path_registered = envpath::add_user_path(&bin_dir)?;
    Ok(SelfDeployOutcome {
        copied,
        path_registered,
        bin_dir,
        exe: dst,
    })
}

/// Linux / macOS：复制当前二进制到 `~/.local/bin/ome`，并确保 `~/.local/bin` 在用户 PATH 中。
#[cfg(not(windows))]
pub fn self_deploy(_env_root: &Path) -> Result<SelfDeployOutcome, String> {
    use crate::platform;

    let src = std::env::current_exe().map_err(|e| format!("获取当前二进制路径失败: {e}"))?;
    let dst = platform::self_deploy_target()?;
    let bin_dir = dst
        .parent()
        .ok_or("self-deploy 目标路径缺少父目录")?
        .to_path_buf();
    let copied = deploy_copy(&src, &dst)?;
    if copied {
        eprintln!("[OK] 已复制: {} -> {}", src.display(), dst.display());
    } else {
        eprintln!("[INFO] 目标已是最新，跳过复制: {}", dst.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&dst)
            .map_err(|e| format!("读取权限失败: {}: {e}", dst.display()))?
            .permissions();
        perm.set_mode(perm.mode() | 0o755);
        std::fs::set_permissions(&dst, perm)
            .map_err(|e| format!("设置可执行权限失败: {}: {e}", dst.display()))?;
    }
    let path_registered = platform::add_user_path(&bin_dir)?;
    Ok(SelfDeployOutcome {
        copied,
        path_registered,
        bin_dir,
        exe: dst,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_copy_复制与幂等() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let src = dir.path().join("src.exe");
        let dst = dir.path().join("bin").join("ome.exe");
        std::fs::write(&src, b"v1-binary").map_err(|e| e.to_string())?;

        assert!(deploy_copy(&src, &dst)?, "首次应复制");
        assert_eq!(
            std::fs::read(&dst).map_err(|e| e.to_string())?,
            b"v1-binary"
        );

        assert!(!deploy_copy(&src, &dst)?, "sha 一致应跳过");

        std::fs::write(&src, b"v2-binary").map_err(|e| e.to_string())?;
        assert!(deploy_copy(&src, &dst)?, "内容变化应覆盖复制");
        assert_eq!(
            std::fs::read(&dst).map_err(|e| e.to_string())?,
            b"v2-binary"
        );
        Ok(())
    }

    #[test]
    fn deploy_copy_同路径跳过() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let exe = dir.path().join("ome.exe");
        std::fs::write(&exe, b"self").map_err(|e| e.to_string())?;
        assert!(!deploy_copy(&exe, &exe)?, "同路径应跳过（自复制）");
        Ok(())
    }
}

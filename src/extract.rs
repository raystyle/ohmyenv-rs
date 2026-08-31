//! extract：解压/安装分派，对齐 helpers.ps1 Install-ToolVersion 的 switch（1148-1242 行）。
//! zip/targz 展平顶层单包裹目录；copy/single 单 exe 落目录；gsudo 只取 x64；
//! 7z-extra 用 bootstrap 7zr.exe 解压并 shim 7za.exe 成 7z.exe 后清空目录其余文件；
//! 7zsfx 同步等待退出码；msi 走 msiexec /qn；rmux 跑资产内官方 install.ps1；
//! bun 装后补 bunx shim（硬链接优先，失败回退纯 ASCII 无 BOM 的 bunx.cmd）。

use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::Command;

use crate::catalog::Tool;

/// 解压/安装主分派（对齐 pwsh switch ($d.Extract)）。
/// cache_path 为已下载资产，install_dir 为目标目录（msi/official 类由调用方另行处理）。
pub fn extract_asset(
    tool: &str,
    def: &Tool,
    cache_path: &Path,
    install_dir: &Path,
    env_root: &Path,
) -> Result<(), String> {
    let kind = def
        .extract
        .as_deref()
        .ok_or_else(|| format!("{tool} 缺少 extract 字段"))?;
    match kind {
        "zip" => {
            extract_zip(cache_path, install_dir)?;
            flatten_single_wrapper(install_dir)?;
            // bun 单二进制：同目录补 bunx.exe shim（bunx = bun x，bun 按 argv[0] 切换）
            if tool == "bun" {
                ensure_bunx_shim(install_dir)?;
            }
            Ok(())
        }
        "targz" => {
            extract_targz(cache_path, install_dir)?;
            flatten_single_wrapper(install_dir)?;
            Ok(())
        }
        // copy/single：单 exe 落目录（文件名取 exe 字段的叶子名，对齐 pwsh copy 分支）
        "copy" | "single" => {
            let leaf = def
                .exe
                .as_deref()
                .and_then(|e| e.rsplit(['\\', '/']).next())
                .ok_or_else(|| format!("{tool} 缺少 exe 字段，无法确定落地文件名"))?;
            fs::copy(cache_path, install_dir.join(leaf))
                .map_err(|e| format!("{tool} 复制资产失败: {e}"))?;
            Ok(())
        }
        "gsudo" => {
            // gsudo.portable.zip 多架构（x64/x86/arm64/net46-AnyCpu）；只取 x64 展平，其余架构删除
            extract_zip(cache_path, install_dir)?;
            let x64 = install_dir.join("x64");
            if !x64.exists() {
                return Err(format!("{tool} 资产缺少 x64 目录"));
            }
            move_children(&x64, install_dir)?;
            for arch in ["x64", "x86", "arm64", "net46-AnyCpu"] {
                let p = install_dir.join(arch);
                if p.exists() {
                    let _ = fs::remove_dir_all(&p);
                }
            }
            Ok(())
        }
        "7zsfx" => {
            // SFX 自解压 exe 是 GUI 子系统：必须同步等待拿真实退出码（对齐 pwsh Start-Process -Wait）
            let status = Command::new(cache_path)
                .arg("-y")
                .arg(format!("-o{}", install_dir.display()))
                .status()
                .map_err(|e| format!("{tool} 自解压启动失败: {e}"))?;
            if !status.success() {
                return Err(format!("{tool} 自解压失败（exit={:?}）", status.code()));
            }
            Ok(())
        }
        "7z-archive" => {
            // 7zXXX-x64.exe 是 7z 归档；Windows 自带 tar(bsdtar) 可直接解包
            run_tar(&["-xf", &cache_path.to_string_lossy(), "-C", &install_dir.to_string_lossy()])
                .map_err(|e| format!("{tool} 7z 归档解包失败: {e}"))
        }
        "7z-extra" => extract_7z_extra(tool, def, cache_path, install_dir, env_root),
        "msi" => {
            // per-machine 静默安装；MSI 自行注册 PATH；0/3010（需重启）均视为成功
            let status = Command::new("msiexec.exe")
                .arg("/i")
                .arg(cache_path)
                .args(["/qn", "/norestart", "DISABLE_TELEMETRY=1"])
                .status()
                .map_err(|e| format!("{tool} msiexec 启动失败: {e}"))?;
            match status.code() {
                Some(0) | Some(3010) => Ok(()),
                code => Err(format!("{tool} MSI 安装失败（exit={code:?}）")),
            }
        }
        "rmux" => extract_rmux(tool, cache_path),
        other => Err(format!("未知解压类型: {other}")),
    }
}

/// zip 解压（zip crate，防路径穿越：拒绝越出目标的条目）。
pub fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let f = File::open(archive).map_err(|e| format!("打开 zip 失败: {}: {e}", archive.display()))?;
    let mut zip =
        zip::ZipArchive::new(f).map_err(|e| format!("读取 zip 失败: {}: {e}", archive.display()))?;
    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {e}"))?;
        let Some(name) = entry.enclosed_name() else {
            continue; // 非法/越界名直接跳过（zip crate 已过滤 .. 穿越）
        };
        let out = dest.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| format!("创建目录失败: {}: {e}", out.display()))?;
        } else {
            if let Some(p) = out.parent() {
                fs::create_dir_all(p)
                    .map_err(|e| format!("创建目录失败: {}: {e}", p.display()))?;
            }
            let mut w = File::create(&out)
                .map_err(|e| format!("创建文件失败: {}: {e}", out.display()))?;
            io::copy(&mut entry, &mut w)
                .map_err(|e| format!("写出文件失败: {}: {e}", out.display()))?;
        }
    }
    Ok(())
}

/// tar.gz 解压（flate2 + tar crate）。
pub fn extract_targz(archive: &Path, dest: &Path) -> Result<(), String> {
    let f =
        File::open(archive).map_err(|e| format!("打开 tar.gz 失败: {}: {e}", archive.display()))?;
    let gz = flate2::read::GzDecoder::new(f);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(dest)
        .map_err(|e| format!("tar.gz 解压失败: {}: {e}", archive.display()))
}

/// 展平顶层单包裹目录（如 age zip 的 age/、python 的 python/ 包裹层）：
/// 目录内无文件且只有一个子目录时，把子目录内容上提一层。返回是否发生了展平。
pub fn flatten_single_wrapper(dir: &Path) -> Result<bool, String> {
    let entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("读取目录失败: {}: {e}", dir.display()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("读取目录失败: {}: {e}", dir.display()))?;
    let mut dirs = entries.iter().filter(|e| e.path().is_dir());
    let file_count = entries.iter().filter(|e| e.path().is_file()).count();
    let (Some(inner), None) = (dirs.next(), dirs.next()) else {
        return Ok(false);
    };
    if file_count > 0 {
        return Ok(false);
    }
    let inner = inner.path();
    move_children(&inner, dir)?;
    fs::remove_dir(&inner).map_err(|e| format!("清理包裹目录失败: {}: {e}", inner.display()))?;
    Ok(true)
}

/// 把 src 下全部条目移动到 dst（对齐 pwsh 的 Move-Item -Force）。
fn move_children(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|e| format!("读取目录失败: {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| format!("读取目录失败: {e}"))?;
        let target = dst.join(entry.file_name());
        if target.exists() {
            if target.is_dir() {
                fs::remove_dir_all(&target)
                    .map_err(|e| format!("覆盖清理失败: {}: {e}", target.display()))?;
            } else {
                fs::remove_file(&target)
                    .map_err(|e| format!("覆盖清理失败: {}: {e}", target.display()))?;
            }
        }
        fs::rename(entry.path(), &target).map_err(|e| {
            format!(
                "移动失败: {} -> {}: {e}",
                entry.path().display(),
                target.display()
            )
        })?;
    }
    Ok(())
}

/// 7z-extra：用 bootstrap 7zr.exe 解压 extra.7z，取 x64/7za.exe（或顶层 7za.exe）shim 成 7z.exe，
/// 只保留 7z.exe、清理解压出的其余文件保持目录干净（对齐 pwsh 7z-extra 分支）。
fn extract_7z_extra(
    tool: &str,
    def: &Tool,
    cache_path: &Path,
    install_dir: &Path,
    env_root: &Path,
) -> Result<(), String> {
    let bootstrap = def
        .bootstrap_asset
        .as_deref()
        .ok_or_else(|| format!("{tool} 缺少 bootstrap_asset 字段"))?;
    let boot = env_root.join("cache").join(bootstrap);
    if !boot.exists() {
        return Err(format!("{tool} 缺少 BootstrapAsset: {bootstrap}"));
    }
    let status = Command::new(&boot)
        .arg("x")
        .arg(cache_path)
        .arg(format!("-o{}", install_dir.display()))
        .arg("-y")
        .status()
        .map_err(|e| format!("{tool} 7zr 启动失败: {e}"))?;
    if !status.success() {
        return Err(format!("{tool} extra.7z 解压失败（exit={:?}）", status.code()));
    }
    let mut src7za = install_dir.join("x64").join("7za.exe");
    if !src7za.exists() {
        src7za = install_dir.join("7za.exe");
    }
    if !src7za.exists() {
        return Err(format!("{tool} extra.7z 内未找到 7za.exe"));
    }
    fs::copy(&src7za, install_dir.join("7z.exe")).map_err(|e| format!("{tool} shim 7z.exe 失败: {e}"))?;
    // 只保留 7z.exe（7za 单文件 standalone），删除其余文件与空目录
    remove_all_except(install_dir, "7z.exe")?;
    Ok(())
}

/// 递归删除 dir 下除指定文件名外的全部文件与目录（7z-extra 清场用）。
fn remove_all_except(dir: &Path, keep_name: &str) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| format!("读取目录失败: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            remove_all_except(&path, keep_name)?;
            // 清完内容后目录若已空则删除（7z.exe 只留在顶层，子目录必空）
            let _ = fs::remove_dir(&path);
        } else if entry.file_name() != keep_name {
            fs::remove_file(&path).map_err(|e| format!("清理文件失败: {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

/// rmux 官方布局：zip 内附官方 install.ps1，跑它装到 LOCALAPPDATA 官方目录并自校验。
fn extract_rmux(tool: &str, cache_path: &Path) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!("rmux-install-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| format!("创建临时目录失败: {}: {e}", tmp.display()))?;
    let result = (|| {
        extract_zip(cache_path, &tmp)?;
        // 包根：第一个含 install.ps1 的子目录
        let pkg_root = fs::read_dir(&tmp)
            .map_err(|e| format!("读取临时目录失败: {e}"))?
            .filter_map(|e| e.ok().map(|x| x.path()))
            .find(|p| p.is_dir() && p.join("install.ps1").exists())
            .ok_or_else(|| "rmux 资产内未找到 install.ps1".to_string())?;
        let status = Command::new("pwsh")
            .args(["-NoProfile", "-File"])
            .arg(pkg_root.join("install.ps1"))
            .status()
            .map_err(|e| format!("rmux install.ps1 启动失败（pwsh 不可用？）: {e}"))?;
        if !status.success() {
            return Err(format!("{tool} 官方 install.ps1 失败（exit={:?}）", status.code()));
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&tmp);
    result
}

/// 调系统 tar.exe（bsdtar）。
fn run_tar(args: &[&str]) -> Result<(), String> {
    let status = Command::new("tar")
        .args(args)
        .status()
        .map_err(|e| format!("tar.exe 不可用: {e}"))?;
    if !status.success() {
        return Err(format!("tar 失败（exit={:?}）", status.code()));
    }
    Ok(())
}

/// bunx.cmd 兜底内容（纯函数便于测试）：纯 ASCII 无 BOM（cmd.exe 不识别 BOM），
/// %~dp0bun.exe 必须加引号（可重定位目录可能含空格）。对齐 Ensure-BunxShim。
pub fn bunx_cmd_content() -> &'static str {
    "@echo off\r\n\"%~dp0bun.exe\" x %*\r\n"
}

/// 为 bun 在部署目录创建 bunx.exe shim：优先硬链接（需 NTFS），失败回退 bunx.cmd；幂等。
pub fn ensure_bunx_shim(bun_dir: &Path) -> Result<(), String> {
    let exe = bun_dir.join("bun.exe");
    let shim = bun_dir.join("bunx.exe");
    if !exe.exists() {
        eprintln!("[WARN] 未找到 bun.exe，跳过 bunx shim: {}", exe.display());
        return Ok(());
    }
    if shim.exists() {
        eprintln!("[INFO] bunx 已存在，跳过: {}", shim.display());
        return Ok(());
    }
    if fs::hard_link(&exe, &shim).is_ok() {
        eprintln!("[OK] bunx shim 已创建（硬链接）: {}", shim.display());
        return Ok(());
    }
    let cmd = bun_dir.join("bunx.cmd");
    fs::write(&cmd, bunx_cmd_content()).map_err(|e| format!("写 bunx.cmd 失败: {e}"))?;
    eprintln!("[OK] bunx shim 已创建（bunx.cmd 兜底）: {}", cmd.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// 造一个目录树：dir/inner/{a.txt, sub/b.txt}，用于展平测试。
    fn make_wrapper(dir: &Path, inner: &str) -> Result<PathBuf, String> {
        let inner_dir = dir.join(inner);
        fs::create_dir_all(inner_dir.join("sub")).map_err(|e| e.to_string())?;
        fs::write(inner_dir.join("a.txt"), b"a").map_err(|e| e.to_string())?;
        fs::write(inner_dir.join("sub").join("b.txt"), b"b").map_err(|e| e.to_string())?;
        Ok(inner_dir)
    }

    #[test]
    fn 展平_顶层单包裹目录_内容上提() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let root = tmp.path().join("install");
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        make_wrapper(&root, "age")?;

        assert!(flatten_single_wrapper(&root)?, "单包裹目录应展平");
        assert!(root.join("a.txt").exists(), "文件应上提到顶层");
        assert!(root.join("sub").join("b.txt").exists(), "子目录应整体上提");
        assert!(!root.join("age").exists(), "包裹层应被删除");
        Ok(())
    }

    #[test]
    fn 展平_有顶层文件或多目录_不动() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;

        // 场景一：顶层有文件 + 一个目录（非包裹结构）
        let root1 = tmp.path().join("case1");
        fs::create_dir_all(root1.join("sub")).map_err(|e| e.to_string())?;
        fs::write(root1.join("top.txt"), b"t").map_err(|e| e.to_string())?;
        assert!(!flatten_single_wrapper(&root1)?, "有顶层文件不应展平");
        assert!(root1.join("sub").exists());

        // 场景二：两个顶层目录
        let root2 = tmp.path().join("case2");
        fs::create_dir_all(root2.join("a")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root2.join("b")).map_err(|e| e.to_string())?;
        assert!(!flatten_single_wrapper(&root2)?, "多目录不应展平");
        Ok(())
    }

    #[test]
    fn bunx_cmd内容_纯ascii无bom且含引号() {
        let content = bunx_cmd_content();
        assert!(content.is_ascii(), "cmd 内容必须纯 ASCII（BOM 会让 cmd 读坏首行）");
        assert!(content.starts_with("@echo off"), "首行必须是 @echo off（无 BOM 前缀）");
        assert!(content.contains("\"%~dp0bun.exe\" x %*"), "bun.exe 路径必须加引号");
        assert!(content.ends_with("\r\n"), "cmd 需要 CRLF 行尾");
    }

    #[test]
    fn bunx_shim_硬链接优先_幂等() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let dir = tmp.path();
        fs::write(dir.join("bun.exe"), b"fake-bun").map_err(|e| e.to_string())?;

        ensure_bunx_shim(dir)?;
        let shim = dir.join("bunx.exe");
        if shim.exists() {
            // NTFS：硬链接成功
            assert_eq!(fs::read(&shim).map_err(|e| e.to_string())?, b"fake-bun");
        } else {
            // 非 NTFS：回退 cmd
            let cmd = fs::read(dir.join("bunx.cmd")).map_err(|e| e.to_string())?;
            assert_eq!(cmd, bunx_cmd_content().as_bytes(), "cmd 内容应与模板一致");
        }
        // 幂等：再跑一遍不报错不覆盖
        ensure_bunx_shim(dir)?;
        Ok(())
    }

    #[test]
    fn bunx_shim_无bun时静默跳过() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        ensure_bunx_shim(tmp.path())?;
        assert!(!tmp.path().join("bunx.exe").exists());
        assert!(!tmp.path().join("bunx.cmd").exists());
        Ok(())
    }

    #[test]
    fn zip解压_含目录与嵌套文件() -> Result<(), String> {
        let tmp = tempfile::tempdir().map_err(|e| e.to_string())?;
        let zip_path = tmp.path().join("demo.zip");
        {
            let f = File::create(&zip_path).map_err(|e| e.to_string())?;
            let mut zw = zip::ZipWriter::new(f);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("demo/a.txt", opts).map_err(|e| e.to_string())?;
            io::Write::write_all(&mut zw, b"hello").map_err(|e| e.to_string())?;
            zw.add_directory("demo/sub/", opts).map_err(|e| e.to_string())?;
            zw.start_file("demo/sub/b.txt", opts).map_err(|e| e.to_string())?;
            io::Write::write_all(&mut zw, b"world").map_err(|e| e.to_string())?;
            zw.finish().map_err(|e| e.to_string())?;
        }
        let dest = tmp.path().join("out");
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        extract_zip(&zip_path, &dest)?;
        assert_eq!(
            fs::read(dest.join("demo").join("a.txt")).map_err(|e| e.to_string())?,
            b"hello"
        );
        assert_eq!(
            fs::read(dest.join("demo").join("sub").join("b.txt")).map_err(|e| e.to_string())?,
            b"world"
        );
        // 展平后应只剩 a.txt 与 sub/
        assert!(flatten_single_wrapper(&dest)?);
        assert!(dest.join("a.txt").exists());
        Ok(())
    }
}

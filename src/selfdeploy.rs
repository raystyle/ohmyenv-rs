//! selfdeploy：自部署——复制当前 exe 到用户程序目录（Windows `%LOCALAPPDATA%\Programs\ome`，
//! Linux / macOS `~/.local/bin`），同步 catalog 到用户数据目录，并注册用户 PATH（幂等）。
//! Windows 顺带清理旧自部署位 `<EnvRoot>\ome\bin` 的 PATH 残留。
//! 幂等：目标与当前 exe 同路径则跳过复制；sha256 一致则跳过复制；PATH 注册由 envpath 幂等处理。

use std::path::{Path, PathBuf};

use crate::download::sha256_file;
use crate::platform;

/// 自部署结果。
pub struct SelfDeployOutcome {
    pub copied: bool,
    pub path_registered: bool,
    pub bin_dir: PathBuf,
    pub exe: PathBuf,
    /// 同步到用户数据目录的 catalog 路径；无源可同步时为 None。
    pub catalog: Option<PathBuf>,
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

/// 同步 catalog 到用户数据目录 `<metadata>\catalog\tools.toml`（自部署即同步数据源，幂等）。
/// 当前活动 catalog 不存在（如任意目录运行无源二进制）时跳过，返回 None。
fn deploy_catalog() -> Result<Option<PathBuf>, String> {
    let src = match crate::catalog::resolve_catalog_path() {
        Ok(p) if p.exists() => p,
        _ => return Ok(None),
    };
    let dst = platform::metadata_dir().join("catalog").join("tools.toml");
    if deploy_copy(&src, &dst)? {
        eprintln!(
            "[OK] 已同步 catalog: {} -> {}",
            src.display(),
            dst.display()
        );
    } else {
        eprintln!("[INFO] catalog 已是最新: {}", dst.display());
    }
    Ok(Some(dst))
}

/// 同步 SKILL.md 到用户数据目录（D09：agent 发现入口，自适应生成——本机实装清单与
/// 使用引导，非静态文件；生成快照随环境变化，`ome skill` 随时刷新，init 时顺带生成）。
pub fn deploy_skill() -> Result<PathBuf, String> {
    let dst = platform::metadata_dir().join("SKILL.md");
    std::fs::create_dir_all(dst.parent().unwrap_or(Path::new(".")))
        .map_err(|e| format!("创建数据目录失败: {e}"))?;
    // init 路径无现成渲染文本：写静态骨架（命令图与工作流），`ome skill` 再充实环境清单
    let want = include_str!("../SKILL.md");
    let cur = std::fs::read_to_string(&dst).unwrap_or_default();
    if cur != want {
        std::fs::write(&dst, want).map_err(|e| format!("写 SKILL.md 失败: {e}"))?;
        eprintln!("[OK] 已同步 SKILL.md: {}", dst.display());
    }
    Ok(dst)
}

/// 自适应渲染环境 SKILL（D09）：本机实装依赖（十类分组、名称与版本）、类级使用引导、
/// 命令图与检测驱动工作流。agent 直读 stdout 或数据目录落盘件。
pub fn render_skill(cat: &crate::catalog::Catalog, env_root: &Path) -> Result<String, String> {
    let srows = crate::status::collect_status(cat, env_root)?;
    let mut out = String::new();
    out.push_str("# SKILL.md：ome 环境自适应清单\n\n> 由 `ome skill` 生成（快照随环境变化，缺什么 `ome install` 补）。\n\n");
    out.push_str("## 本机可用依赖\n\n");
    // 类级使用引导（静态知识，简短）
    let guides: &[(&str, &str)] = &[
        (
            "agent",
            "终端智能体，直接调命令行即可（claude -p / codex exec 等）；升级走各家自更新",
        ),
        ("base", "编排类 CLI"),
        (
            "runtime",
            "语言与子系统运行时（python/node 家族等），写代码直接用解释器命令",
        ),
        (
            "runtime-manager",
            "运行时版本管理器（uv 管 python、fnm 管 node），建虚拟环境与切版本先用它们",
        ),
        (
            "compiler",
            "编译器与构建工具链（rust/go/zig/vsbuild），编译走 cargo、go build、zig、MSBuild",
        ),
        ("derived", "经运行时包管理器安装的衍生工具"),
        ("mux", "多路终端多路复用器，多 agent 并行会话的宿主"),
        ("service", "远程服务客户端"),
        ("security", "密钥与加密工具（age 加解密、sops 密文配置）"),
        (
            "cli",
            "命令行工具，按名直用（rg 搜索、jq/mq JSON/结构化查询、gh GitHub 操作等）",
        ),
    ];
    for (cat_key, label) in crate::status::GROUPS {
        let group: Vec<&crate::status::StatusRow> =
            srows.iter().filter(|r| &r.category == cat_key).collect();
        if group.is_empty() {
            continue;
        }
        let installed: Vec<&crate::status::StatusRow> = group
            .iter()
            .copied()
            .filter(|r| r.installed.is_some())
            .collect();
        if installed.is_empty() {
            out.push_str(&format!("### {label}\n\n（未装；`ome install` 补）\n\n"));
            continue;
        }
        let guide = guides
            .iter()
            .find(|(k, _)| k == cat_key)
            .map(|(_, g)| *g)
            .unwrap_or("");
        out.push_str(&format!("### {label}\n\n> {guide}\n\n"));
        for r in &installed {
            let v = r.installed.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "- {} {}（{}）\n",
                r.name,
                v,
                r.exe
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            ));
        }
        out.push('\n');
    }
    out.push_str("## ome 命令与工作流\n\n- 诊断环境：`ome doctor`（三层 + verdict 一锤定音：ready/degraded/broken）\n- 缺什么装什么：`ome install`（省略则全量，幂等，官方失败回落 env.ohmygh.com 镜像）\n- 看三态：`ome status`；升级：`ome update`（省略则全量；agent 类走自更新）\n- 命令全图：`ome --llms`\n\n> 环境变化后重跑 `ome skill` 刷新本清单。\n");
    Ok(out)
}

/// 自部署：复制当前 exe 到用户程序目录，同步 catalog 到用户数据目录，注册 bin 目录进用户 PATH。
#[cfg(windows)]
pub fn self_deploy(env_root: &Path) -> Result<SelfDeployOutcome, String> {
    let src = std::env::current_exe().map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
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
    let path_registered = platform::add_user_path(&bin_dir)?;
    // 清理旧自部署位 <EnvRoot>\ome\bin 的 PATH 残留（一次性迁移，幂等）
    let legacy_bin = env_root.join("ome").join("bin");
    if platform::remove_user_path(&legacy_bin)? {
        eprintln!("[OK] 已移除旧 PATH 残留: {}", legacy_bin.display());
    }
    let catalog = deploy_catalog()?;
    let _skill = deploy_skill();
    Ok(SelfDeployOutcome {
        copied,
        path_registered,
        bin_dir,
        exe: dst,
        catalog,
    })
}

/// Linux / macOS：复制当前二进制到 `~/.local/bin/ome`，同步 catalog，并确保 `~/.local/bin` 在用户 PATH 中。
#[cfg(not(windows))]
pub fn self_deploy(_env_root: &Path) -> Result<SelfDeployOutcome, String> {
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
    let catalog = deploy_catalog()?;
    let _skill = deploy_skill();
    Ok(SelfDeployOutcome {
        copied,
        path_registered,
        bin_dir,
        exe: dst,
        catalog,
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

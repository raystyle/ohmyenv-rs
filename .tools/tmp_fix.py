import io

def rep(p, old, new, count=1):
    t = io.open(p, encoding='utf-8').read()
    assert old in t, (p, old[:40])
    io.open(p, 'w', encoding='utf-8', newline='\n').write(t.replace(old, new, count))
    print(p, 'ok')

# 1) 新 helper（POSIX 用户 bin 直链兜底）
rep('src/install.rs',
 '''/// manifest 原语应用（shims 加 post_install；D39 共识②上提到早退通道：uv-git 与''',
 '''/// POSIX 嵌套布局可发现性兜底：exe 不在 ~/.local/bin 时保证 ~/.local/bin/<name> 直链
/// （ohmycloud lan-linux 实测 2026-09-11：skip 分支注册的 PATH 目录在非交互 shell
/// （omc hostExec）不加载 profile 形同虚设；~/.local/bin 是 XDG 用户 bin 基建，
/// 登录与非交互 PATH 均含）。幂等：完好链接跳过，悬空链接（target 已卸）先删再建。
#[cfg(not(windows))]
fn ensure_user_bin_link(name: &str, exe: &Path) {
    let Some(home) = dirs::home_dir() else { return };
    let dst = home.join(".local").join("bin").join(name);
    if exe == dst || dst.exists() {
        return;
    }
    let _ = std::fs::create_dir_all(home.join(".local").join("bin"));
    // 悬空链接（上次装的 target 已卸）：exists 为 false 但 symlink_metadata 在，删后重建
    if dst.symlink_metadata().is_ok() {
        let _ = std::fs::remove_file(&dst);
    }
    if let Err(e) = std::os::unix::fs::symlink(exe, &dst) {
        eprintln!("[WARN] 用户 bin 直链失败 {} -> {}: {e}", dst.display(), exe.display());
    } else {
        eprintln!("[OK] 已建用户 bin 直链: {} -> {}", dst.display(), exe.display());
    }
}

#[cfg(windows)]
fn ensure_user_bin_link(_name: &str, _exe: &Path) {}

/// manifest 原语应用（shims 加 post_install；D39 共识②上提到早退通道：uv-git 与''')

# 2) 幂等 skip 分支：configure 块补直链（ohmycloud 复现路径的修复点）
rep('src/install.rs',
 '''        if opts.configure {
            register_bin(def, env_root, is_official)?;
            ensure_user_env_overrides(ms)?;
        }
        // 老环境补别名（bun 已存在但同目录缺 bunx.exe）：manifest shims 节唯一来源''',
 '''        if opts.configure {
            register_bin(def, env_root, is_official)?;
            ensure_user_env_overrides(ms)?;
            // 注册面幂等兜底：skip 也要保证可发现性（POSIX 嵌套布局的用户 bin 直链）
            ensure_user_bin_link(name, &exe_path);
        }
        // 老环境补别名（bun 已存在但同目录缺 bunx.exe）：manifest shims 节唯一来源''')

# 3) 主链装成后 configure 块同样补
rep('src/install.rs',
 '''    if opts.configure {
        register_bin(def, env_root, is_official)?;
        ensure_user_env_overrides(ms)?;
        // D39 R016 L1/L2：manifest 节的 shims 与受控命令在装成后执行（三平台矩阵验收）。''',
 '''    if opts.configure {
        register_bin(def, env_root, is_official)?;
        ensure_user_env_overrides(ms)?;
        ensure_user_bin_link(name, &exe_path);
        // D39 R016 L1/L2：manifest 节的 shims 与受控命令在装成后执行（三平台矩阵验收）。''')

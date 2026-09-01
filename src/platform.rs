//! platform：跨平台抽象层。
//!
//! Windows：EnvRoot 为 `D:\ohmyenv` 或 `C:\ohmyenv`，工具集中安装；PATH 通过注册表管理。
//! Linux / macOS：各软件按系统标准目录安装；PATH 通过当前 shell 的 profile 文件（如 `~/.bashrc`）管理。
//! ome 自身与 EnvRoot 解耦：二进制进用户程序目录（Windows `%LOCALAPPDATA%\Programs\ome`）、
//! 元数据进用户数据目录（Windows `%LOCALAPPDATA%\ohmyenv`；Linux `~/.local/share/ohmyenv`；
//! macOS `~/Library/Application Support/ohmyenv`）。

use std::path::{Path, PathBuf};

/// 默认 EnvRoot：显式参数与环境变量已在 `catalog::resolve_env_root` 处理，此处仅返回平台默认值。
pub fn default_env_root() -> PathBuf {
    #[cfg(windows)]
    {
        if Path::new(r"D:\").exists() {
            PathBuf::from(r"D:\ohmyenv")
        } else {
            PathBuf::from(r"C:\ohmyenv")
        }
    }
    #[cfg(not(windows))]
    {
        data_dir().join("ohmyenv")
    }
}

/// ome 自身元数据目录（独立 app 数据目录，与 EnvRoot 解耦）：
/// Windows `%LOCALAPPDATA%\ohmyenv`；Linux `~/.local/share/ohmyenv`；macOS `~/Library/Application Support/ohmyenv`。
pub fn metadata_dir() -> PathBuf {
    data_dir().join("ohmyenv")
}

/// 可执行文件后缀。
pub fn exe_suffix() -> &'static str {
    #[cfg(windows)]
    {
        ".exe"
    }
    #[cfg(not(windows))]
    {
        ""
    }
}

/// 展开安装路径中的 `~` 与环境变量引用。
pub fn expand_install_path(raw: &str) -> PathBuf {
    if raw.starts_with("~/") || raw == "~" {
        if let Some(home) = dirs::home_dir() {
            return if raw == "~" {
                home
            } else {
                home.join(&raw[2..])
            };
        }
    }
    PathBuf::from(expand_env_vars(raw))
}

/// 展开后的路径若为相对路径则拼到 EnvRoot 下（Windows 名录是相对 dir/bin；Linux/macOS 多为 ~/ 绝对）。
pub fn join_if_relative(env_root: &Path, p: PathBuf) -> PathBuf {
    if p.is_absolute() {
        p
    } else {
        env_root.join(p)
    }
}

/// PATH 环境变量条目分隔符。
pub fn path_separator() -> char {
    #[cfg(windows)]
    {
        ';'
    }
    #[cfg(not(windows))]
    {
        ':'
    }
}

/// 用户级本地数据目录（Windows `%LOCALAPPDATA%`；Linux XDG_DATA_HOME 或 `~/.local/share`；macOS `~/Library/Application Support`）。
fn data_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .expect("无法确定用户主目录")
            .join(".local")
            .join("share")
    })
}

/// 自部署目标路径。
/// Windows：`%LOCALAPPDATA%\Programs\ome\ome.exe`
/// Linux / macOS：`~/.local/bin/ome`
pub fn self_deploy_target() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        Ok(data_dir().join("Programs").join("ome").join("ome.exe"))
    }
    #[cfg(not(windows))]
    {
        let bin_dir = dirs::home_dir()
            .ok_or("无法确定用户主目录")?
            .join(".local")
            .join("bin");
        Ok(bin_dir.join("ome"))
    }
}

/// 将 dir 注册进用户 PATH；返回是否实际新增。
pub fn add_user_path(dir: &Path) -> Result<bool, String> {
    #[cfg(windows)]
    {
        windows::add_user_path(&dir.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        unix::add_user_path(dir)
    }
}

/// 从用户 PATH 移除 dir；返回是否实际移除。
pub fn remove_user_path(dir: &Path) -> Result<bool, String> {
    #[cfg(windows)]
    {
        windows::remove_user_path(&dir.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        unix::remove_user_path(dir)
    }
}

/// 用户 PATH 是否已含 dir。
pub fn user_path_contains(dir: &Path) -> Result<bool, String> {
    #[cfg(windows)]
    {
        windows::user_path_contains(&dir.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        unix::user_path_contains(dir)
    }
}

/// PATH 条目合并（纯函数）：把缺失目录追加到 raw 尾部（大小写不敏感、忽略尾反斜杠比较）。
/// 返回新值与是否变化；供用户级与机器级 PATH 写入共用。
pub fn merge_path_entries(raw: &str, dirs: &[String]) -> (String, bool) {
    let mut parts: Vec<String> = raw
        .split(';')
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.trim().to_string())
        .collect();
    let norm = |s: &str| s.trim_end_matches('\\').to_lowercase();
    let mut changed = false;
    for d in dirs {
        let ds = d.trim().to_string();
        if ds.is_empty() {
            continue;
        }
        if !parts.iter().any(|p| norm(p) == norm(&ds)) {
            parts.push(ds);
            changed = true;
        }
    }
    (parts.join(";"), changed)
}

/// profile 环境变量块合并（纯函数）：在 `# >>> ome env` 标记块内幂等 upsert
/// `export KEY="value"` 行（同 KEY 替换、无块则追加到文末）。Linux/macOS 写用户环境变量用。
pub fn merge_env_exports(text: &str, key: &str, value: &str) -> String {
    const MARKER: &str = "# >>> ome env";
    const END: &str = "# <<< ome env";
    let line = format!("export {key}=\"{value}\"");
    let prefix_tag = format!("export {key}=");
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let start = lines.iter().position(|l| l.trim_start() == MARKER);
    let end = lines.iter().position(|l| l.trim_start() == END);
    match (start, end) {
        (Some(s), Some(e)) if e > s => {
            let mut block: Vec<String> = lines[s + 1..e].to_vec();
            block.retain(|l| !l.trim_start().starts_with(&prefix_tag));
            block.push(line);
            lines.splice(s + 1..e, block);
        }
        _ => {
            if !lines.is_empty() && !text.ends_with('\n') {
                if let Some(last) = lines.last_mut() {
                    last.push('\n');
                }
            }
            lines.push(MARKER.to_string());
            lines.push(line);
            lines.push(END.to_string());
        }
    }
    lines.join("\n") + "\n"
}

/// 设置用户级环境变量（幂等）。Windows 写 HKCU\Environment 并同步当前进程；
/// Linux/macOS 写 profile 的 ome 标记块。用于装后遥测关闭等运行时开关。
pub fn set_user_env_var(key: &str, value: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows::set_user_env_var(key, value)
    }
    #[cfg(not(windows))]
    {
        unix::set_user_env_var(key, value)
    }
}

/// 当前进程是否管理员（以写权限打开 HKLM Environment 判定；非 Windows 恒 false）。
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        windows::is_elevated()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// 机器 PATH 是否已含 dir（非 Windows 恒 false）。
pub fn machine_path_contains(dir: &Path) -> Result<bool, String> {
    #[cfg(windows)]
    {
        windows::machine_path_contains(&dir.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        let _ = dir;
        Ok(false)
    }
}

/// 把缺失目录追加进机器 PATH（REG_EXPAND_SZ，需管理员）；返回是否写入。非 Windows 恒不写入。
pub fn machine_path_add(dirs: &[PathBuf]) -> Result<bool, String> {
    #[cfg(windows)]
    {
        windows::machine_path_add(dirs)
    }
    #[cfg(not(windows))]
    {
        let _ = dirs;
        Ok(false)
    }
}

/// 展开环境变量引用。
/// Windows：`%VAR%`；Linux / macOS：`$VAR` 与 `${VAR}`。
pub fn expand_env_vars(s: &str) -> String {
    #[cfg(windows)]
    {
        use regex::Regex;
        let re = Regex::new("%([^%]+)%").expect("静态正则应合法");
        re.replace_all(s, |caps: &regex::Captures| {
            std::env::var(&caps[1]).unwrap_or_else(|_| caps[0].to_string())
        })
        .into_owned()
    }
    #[cfg(not(windows))]
    {
        use regex::Regex;
        let re = Regex::new(r"\$\{([^}]+)\}|\$([A-Za-z_][A-Za-z0-9_]*)").expect("静态正则应合法");
        re.replace_all(s, |caps: &regex::Captures| {
            let name = caps
                .get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");
            std::env::var(name).unwrap_or_else(|_| caps[0].to_string())
        })
        .into_owned()
    }
}

/// 判定 exe 字段是否表示 official 布局（使用环境变量展开 / 绝对路径，不纳入 EnvRoot 管理）。
/// Windows：含 `%`；Linux / macOS：含 `$` 或为绝对路径。
pub fn is_official_exe(exe: &str) -> bool {
    #[cfg(windows)]
    {
        exe.contains('%')
    }
    #[cfg(not(windows))]
    {
        exe.contains('$') || Path::new(exe).is_absolute()
    }
}

/// 标准化 PATH 条目比较：展开环境变量、大小写不敏感（Windows）/ 敏感（Unix）。
pub fn path_entries_eq(a: &str, b: &str) -> bool {
    let a_exp = expand_env_vars(a);
    let b_exp = expand_env_vars(b);
    #[cfg(windows)]
    {
        a_exp.eq_ignore_ascii_case(&b_exp)
    }
    #[cfg(not(windows))]
    {
        a_exp == b_exp
    }
}

// ── Windows 实现 ──

#[cfg(windows)]
mod windows {
    use super::*;
    use winreg::enums::{RegType, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

    /// 机器级 Environment 键（Session Manager）。
    const MACHINE_ENV: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

    fn read_user_path_raw() -> Result<String, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
        env.get_value::<String, _>("Path").or(Ok(String::new()))
    }

    fn write_user_path_raw(raw: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags("Environment", KEY_WRITE)
            .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
        let bytes: Vec<u8> = raw
            .encode_utf16()
            .chain(std::iter::once(0u16))
            .flat_map(u16::to_le_bytes)
            .collect();
        env.set_raw_value(
            "Path",
            &RegValue {
                vtype: RegType::REG_EXPAND_SZ,
                bytes,
            },
        )
        .map_err(|e| format!("写用户 PATH 失败: {e}"))
    }

    pub fn add_user_path(dir: &str) -> Result<bool, String> {
        let raw = read_user_path_raw()?;
        let Some(new_raw) = crate::envpath::add_path_entry(&raw, dir) else {
            return Ok(false);
        };
        write_user_path_raw(&new_raw)?;
        let cur = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{dir};{cur}"));
        Ok(true)
    }

    pub fn remove_user_path(dir: &str) -> Result<bool, String> {
        let raw = read_user_path_raw()?;
        let new_raw = crate::envpath::remove_path_entry(&raw, dir);
        if new_raw == raw {
            return Ok(false);
        }
        write_user_path_raw(&new_raw)?;
        let expanded_dir = expand_env_vars(dir);
        let cur = std::env::var("PATH").unwrap_or_default();
        let kept = cur
            .split(';')
            .filter(|p| !p.is_empty() && !expand_env_vars(p).eq_ignore_ascii_case(&expanded_dir))
            .collect::<Vec<_>>()
            .join(";");
        std::env::set_var("PATH", kept);
        Ok(true)
    }

    pub fn user_path_contains(dir: &str) -> Result<bool, String> {
        let raw = read_user_path_raw()?;
        Ok(raw.split(';').any(|p| p.eq_ignore_ascii_case(dir)))
    }

    /// 管理员判定：以写权限打开 HKLM Environment（无管理员时 OpenKey 报权限错）。
    pub fn is_elevated() -> bool {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        hklm.open_subkey_with_flags(MACHINE_ENV, KEY_READ | KEY_WRITE)
            .is_ok()
    }

    fn read_machine_path_raw() -> Result<String, String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env = hklm
            .open_subkey_with_flags(MACHINE_ENV, KEY_READ)
            .map_err(|e| format!("读取 HKLM Environment 失败: {e}"))?;
        env.get_value::<String, _>("Path").or(Ok(String::new()))
    }

    pub fn machine_path_contains(dir: &str) -> Result<bool, String> {
        let raw = read_machine_path_raw()?;
        let norm = |s: &str| s.trim_end_matches('\\').to_lowercase();
        Ok(raw
            .split(';')
            .any(|p| !p.trim().is_empty() && norm(p) == norm(dir)))
    }

    pub fn machine_path_add(dirs: &[PathBuf]) -> Result<bool, String> {
        let raw = read_machine_path_raw()?;
        let dirs: Vec<String> = dirs
            .iter()
            .map(|d| d.to_string_lossy().to_string())
            .collect();
        let (new_raw, changed) = merge_path_entries(&raw, &dirs);
        if !changed {
            return Ok(false);
        }
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let env = hklm
            .open_subkey_with_flags(MACHINE_ENV, KEY_WRITE)
            .map_err(|e| format!("打开 HKLM Environment 写失败（需管理员）: {e}"))?;
        let bytes: Vec<u8> = new_raw
            .encode_utf16()
            .chain(std::iter::once(0u16))
            .flat_map(u16::to_le_bytes)
            .collect();
        env.set_raw_value(
            "Path",
            &RegValue {
                vtype: RegType::REG_EXPAND_SZ,
                bytes,
            },
        )
        .map_err(|e| format!("写机器 PATH 失败: {e}"))?;
        Ok(true)
    }

    /// 用户级环境变量（REG_SZ，幂等覆盖），并同步当前进程。
    pub fn set_user_env_var(key: &str, value: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags("Environment", KEY_WRITE)
            .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
        env.set_value(key, &value)
            .map_err(|e| format!("写用户环境变量失败: {key}: {e}"))?;
        std::env::set_var(key, value);
        Ok(())
    }
}

// ── Linux / macOS 实现 ──

#[cfg(not(windows))]
mod unix {
    use super::*;

    const OME_PATH_MARKER: &str = "# >>> ome PATH";
    const OME_PATH_END: &str = "# <<< ome PATH";

    fn profile_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("无法确定用户主目录")?;
        // 优先按当前 shell 选择；bash 为默认。
        if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                return Ok(home.join(".zshrc"));
            }
            if shell.contains("fish") {
                return Ok(home.join(".config").join("fish").join("config.fish"));
            }
        }
        Ok(home.join(".bashrc"))
    }

    fn read_profile() -> Result<String, String> {
        let path = profile_path()?;
        if !path.exists() {
            return Ok(String::new());
        }
        std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 profile 失败: {}: {e}", path.display()))
    }

    fn write_profile(text: &str) -> Result<(), String> {
        let path = profile_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建 profile 目录失败: {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, text).map_err(|e| format!("写 profile 失败: {}: {e}", path.display()))
    }

    fn remove_ome_path_block(text: &str) -> String {
        let mut out = Vec::new();
        let mut skip = false;
        for line in text.lines() {
            if line.trim().starts_with(OME_PATH_MARKER) {
                skip = true;
                continue;
            }
            if skip && line.trim().starts_with(OME_PATH_END) {
                skip = false;
                continue;
            }
            if !skip {
                out.push(line);
            }
        }
        out.join("\n")
    }

    fn format_export(dir: &str) -> String {
        format!(r#"export PATH="{}:$PATH""#, dir)
    }

    pub fn add_user_path(dir: &Path) -> Result<bool, String> {
        let dir_str = dir.to_string_lossy().to_string();
        let mut text = read_profile()?;
        // 幂等：已存在同目录块则跳过
        if text.contains(&format_export(&dir_str)) {
            return Ok(false);
        }
        text = remove_ome_path_block(&text);
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!("{OME_PATH_MARKER}\n"));
        text.push_str(&format_export(&dir_str));
        text.push('\n');
        text.push_str(&format!("{OME_PATH_END}\n"));
        write_profile(&text)?;
        // 同步当前进程 PATH
        let cur = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{dir_str}:{cur}"));
        Ok(true)
    }

    pub fn remove_user_path(dir: &Path) -> Result<bool, String> {
        let dir_str = dir.to_string_lossy().to_string();
        let text = read_profile()?;
        if !text.contains(&format_export(&dir_str)) {
            return Ok(false);
        }
        let new_text = remove_ome_path_block(&text);
        write_profile(&new_text)?;
        let cur = std::env::var("PATH").unwrap_or_default();
        let kept = cur
            .split(':')
            .filter(|p| !p.is_empty() && !path_entries_eq(p, &dir_str))
            .collect::<Vec<_>>()
            .join(":");
        std::env::set_var("PATH", kept);
        Ok(true)
    }

    pub fn user_path_contains(dir: &Path) -> Result<bool, String> {
        let dir_str = dir.to_string_lossy().to_string();
        let text = read_profile()?;
        Ok(text.contains(&format_export(&dir_str)))
    }

    /// 用户级环境变量：profile 的 ome env 标记块内幂等 upsert，并同步当前进程。
    pub fn set_user_env_var(key: &str, value: &str) -> Result<(), String> {
        let text = read_profile()?;
        let new_text = merge_env_exports(&text, key, value);
        if new_text != text {
            write_profile(&new_text)?;
        }
        std::env::set_var(key, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn self_deploy_target_windows_用户程序目录与envroot解耦() {
        let t = self_deploy_target().expect("self-deploy 目标应可解析");
        assert!(
            t.ends_with("Programs\\ome\\ome.exe"),
            "目标应在用户程序目录: {}",
            t.display()
        );
        assert!(
            !t.starts_with(default_env_root()),
            "目标不应在 EnvRoot 下: {}",
            t.display()
        );
    }

    #[cfg(windows)]
    #[test]
    fn metadata_dir_windows_独立于envroot() {
        let m = metadata_dir();
        assert!(
            m.ends_with("ohmyenv"),
            "元数据目录名应为 ohmyenv: {}",
            m.display()
        );
        assert!(
            !m.starts_with(default_env_root()),
            "元数据目录应独立于 EnvRoot: {}",
            m.display()
        );
    }

    #[test]
    fn path条目合并_缺失追加存在跳过() {
        let raw = r"C:\win;D:\ohmyenv\jq";
        // 已存在（尾反斜杠与大小写不敏感）不重复追加
        let (out, changed) = merge_path_entries(raw, &[r"d:\OHMYENV\jq\".to_string()]);
        assert!(!changed);
        assert_eq!(out, raw);
        // 缺失追加到尾部，保序
        let (out, changed) = merge_path_entries(
            raw,
            &[r"D:\ohmyenv\vsbuild\MSBuild\Current\Bin".to_string()],
        );
        assert!(changed);
        assert_eq!(
            out,
            r"C:\win;D:\ohmyenv\jq;D:\ohmyenv\vsbuild\MSBuild\Current\Bin"
        );
        // 空条目被清理后合并
        let (out, _) = merge_path_entries("C:\\win;;", &["E:\\x".to_string()]);
        assert_eq!(out, r"C:\win;E:\x");
    }

    #[test]
    fn profile环境变量块_幂等upsert() {
        // 无块：追加标记块到文末
        let t1 = merge_env_exports("export A=1\n", "DOTNET_CLI_TELEMETRY_OPTOUT", "1");
        assert!(t1.contains("# >>> ome env"));
        assert!(t1.contains("export DOTNET_CLI_TELEMETRY_OPTOUT=\"1\""));
        // 同 KEY 同值：幂等不变
        assert_eq!(
            merge_env_exports(&t1, "DOTNET_CLI_TELEMETRY_OPTOUT", "1"),
            t1
        );
        // 块内追加第二个变量：首个保留、不重复
        let t3 = merge_env_exports(&t1, "POWERSHELL_UPDATECHECK", "Off");
        assert!(t3.contains("export POWERSHELL_UPDATECHECK=\"Off\""));
        assert_eq!(t3.matches("export DOTNET_CLI_TELEMETRY_OPTOUT").count(), 1);
        // 同 KEY 改值：替换旧值
        let t4 = merge_env_exports(&t3, "POWERSHELL_UPDATECHECK", "On");
        assert!(t4.contains("export POWERSHELL_UPDATECHECK=\"On\""));
        assert!(!t4.contains("export POWERSHELL_UPDATECHECK=\"Off\""));
    }

    #[cfg(not(windows))]
    #[test]
    fn expand_env_vars_linux_语法() {
        std::env::set_var("OME_TEST_PLAT_X", "/home/demo");
        assert_eq!(expand_env_vars("$OME_TEST_PLAT_X/bin"), "/home/demo/bin");
        assert_eq!(expand_env_vars("${OME_TEST_PLAT_X}/bin"), "/home/demo/bin");
        assert_eq!(expand_env_vars("$OME_NO_SUCH_VAR/x"), "$OME_NO_SUCH_VAR/x");
    }

    #[cfg(not(windows))]
    #[test]
    fn is_official_exe_linux_判定() {
        assert!(is_official_exe("$HOME/.local/bin/jq"));
        assert!(is_official_exe("/usr/local/bin/jq"));
        assert!(!is_official_exe("jq/bin/jq"));
    }

    #[cfg(not(windows))]
    #[test]
    fn path_entries_eq_大小写敏感_仅linux() {
        assert!(path_entries_eq("/home/a/bin", "/home/a/bin"));
        assert!(!path_entries_eq("/home/a/bin", "/home/A/bin"));
    }
}

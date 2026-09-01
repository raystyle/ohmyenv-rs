//! platform：跨平台抽象层。
//!
//! Windows：EnvRoot 为 `D:\ohmyenv` 或 `C:\ohmyenv`，工具集中安装；PATH 通过注册表管理。
//! Linux / macOS：ome 元数据存用户目录（`~/.local/share/ohmyenv`），各软件按系统标准目录安装；
//! PATH 通过当前 shell 的 profile 文件（如 `~/.bashrc`）管理。

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

/// ome 自身元数据目录。Windows 下与 EnvRoot 一致（保持兼容）；Linux 下为 XDG_DATA_HOME。
pub fn metadata_dir() -> PathBuf {
    #[cfg(windows)]
    {
        default_env_root()
    }
    #[cfg(not(windows))]
    {
        data_dir().join("ohmyenv")
    }
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

/// 用户级数据目录（XDG_DATA_HOME 或等价目录）。
#[cfg(not(windows))]
fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .expect("无法确定用户主目录")
            .join(".local")
            .join("share")
    })
}

/// 自部署目标路径。
/// Windows：`<EnvRoot>\ome\bin\ome.exe`
/// Linux：`~/.local/bin/ome`
pub fn self_deploy_target() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        Ok(default_env_root().join("ome").join("bin").join("ome.exe"))
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
    use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

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
}

#[cfg(test)]
mod tests {

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

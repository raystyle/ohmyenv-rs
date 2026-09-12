//! platform：跨平台抽象层。
//!
//! Windows：EnvRoot 为 `D:\ohmyenv` 或 `C:\ohmyenv`，工具集中安装；PATH 通过注册表管理。
//! Linux / macOS：各软件按系统标准目录安装；PATH 通过当前 shell 的 profile 文件（如 `~/.bashrc`）管理。
//! ark 自身与 EnvRoot 解耦：二进制进用户程序目录（Windows `%LOCALAPPDATA%\Programs\ark`）、
//! 元数据进用户数据目录（Windows `%LOCALAPPDATA%\ark`；Linux `~/.local/share/ark`；
//! macOS `~/Library/Application Support/ark`）；迁移过渡期旧 `ohmyenv` 目录在而新目录未建时读回旧位。

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

/// ark 自身元数据目录（独立 app 数据目录，与 EnvRoot 解耦）：
/// Windows `%LOCALAPPDATA%\ark`；Linux `~/.local/share/ark`；macOS `~/Library/Application Support/ark`。
/// D41 迁移过渡：新 `ark` 目录未建而旧 `ohmyenv` 目录在（旧二进制所写）时读回旧位，
/// 令新旧二进制共享同一份 catalog 与 seq 记录；C 阶段搬迁七件套后自然归位主名。
pub fn metadata_dir() -> PathBuf {
    resolve_metadata_dir(&data_dir())
}

/// 元数据目录解析（传 data 便于测）：主名 `ark`；未建且旧 `ohmyenv` 在则读回旧位。
fn resolve_metadata_dir(data: &Path) -> PathBuf {
    let ark = data.join("ark");
    if !ark.exists() {
        let old = data.join("ohmyenv");
        if old.exists() {
            return old;
        }
    }
    ark
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

/// 自部署目标路径（D41：ark 接管部署位，旧 ome 位处置与 `ome` 别名过渡归 C 阶段）。
/// Windows：`%LOCALAPPDATA%\Programs\ark\ark.exe`
/// Linux / macOS：`~/.local/bin/ark`
pub fn self_deploy_target() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        Ok(data_dir().join("Programs").join("ark").join("ark.exe"))
    }
    #[cfg(not(windows))]
    {
        let bin_dir = dirs::home_dir()
            .ok_or("无法确定用户主目录")?
            .join(".local")
            .join("bin");
        Ok(bin_dir.join("ark"))
    }
}

/// 读环境变量（D41 更名 Ark）：主名优先，主名未设或纯空白时读回旧名；都未设返回 None。
/// 同设时主名胜（旧名只作过渡期读回，不参与合并）。
pub fn env_var_or(primary: &str, fallback: &str) -> Option<String> {
    let read = |k: &str| {
        std::env::var(k)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    read(primary).or_else(|| read(fallback))
}

/// 用户面写入总闸门（测试隔离）：`ARK_TEST_NO_PATH_REG=1`（读回 `OME_TEST_NO_PATH_REG`）时，**所有**用户环境写入面
/// 一律跳过（PATH 注册、用户级变量、profile 钩子、用户 bin 直链）。变量名沿历史（PATH 注册
/// 是最早的那面），覆盖面已扩到四面——只守一面会让沙盒测试从别的门漏进真实环境。
pub fn user_env_write_blocked() -> bool {
    env_var_or("ARK_TEST_NO_PATH_REG", "OME_TEST_NO_PATH_REG").as_deref() == Some("1")
}

/// 将 dir 注册进用户 PATH；返回是否实际新增。
pub fn add_user_path(dir: &Path) -> Result<bool, String> {
    // O5（S017）：测试隔离开关——沙盒 EnvRoot 的集成测试不得污染真实注册面/Profile
    // （M002「沙盒漏写面」同型回归的根治；与 ARK_TEST_REAL/MIRROR 同为闸门口径）
    if user_env_write_blocked() {
        eprintln!("[INFO] ARK_TEST_NO_PATH_REG=1：跳过用户 PATH 注册（测试隔离）");
        return Ok(false);
    }
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

/// 用户 PATH 原始条目（保序不去空；Windows 读 HKCU\Environment，非 Windows 读 profile 标记块）。
/// doctor 诊断死链与重复条目用。
pub fn user_path_entries() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    {
        windows::read_user_path_raw().map(|raw| raw.split(';').map(str::to_string).collect())
    }
    #[cfg(not(windows))]
    {
        unix::profile_path_entries()
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
/// O3（S017）：写 profile 自定义钩子行（POSIX；Windows no-op）。幂等。
pub fn ensure_profile_hook(marker: &str, line: &str) {
    if user_env_write_blocked() {
        return;
    }
    #[cfg(not(windows))]
    if let Err(e) = unix::ensure_profile_hook(marker, line) {
        eprintln!("[WARN] profile 钩子写入失败: {e}");
    }
    #[cfg(windows)]
    let _ = (marker, line);
}

/// Linux/macOS 写 profile 的 ome 标记块。用于装后遥测关闭等运行时开关。
pub fn set_user_env_var(key: &str, value: &str) -> Result<(), String> {
    if user_env_write_blocked() {
        eprintln!("[INFO] ARK_TEST_NO_PATH_REG=1：跳过用户级变量写入（测试隔离）: {key}");
        return Ok(());
    }
    #[cfg(windows)]
    {
        windows::set_user_env_var(key, value)
    }
    #[cfg(not(windows))]
    {
        unix::set_user_env_var(key, value)
    }
}

/// 读用户级环境变量（未设置返回 None）。Windows 读 HKCU\Environment；
/// Linux/macOS 读 profile 的 ome 标记块。heal 判 OK/HEALED 用。
pub fn get_user_env_var(key: &str) -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        windows::get_user_env_var(key)
    }
    #[cfg(not(windows))]
    {
        unix::get_user_env_var(key)
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

    /// 原始用户 PATH（未展开值；doctor 诊断死链与重复条目用，PATH 写入共用）。
    pub fn read_user_path_raw() -> Result<String, String> {
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
        let added = if let Some(new_raw) = crate::envpath::add_path_entry(&raw, dir) {
            write_user_path_raw(&new_raw)?;
            notify_env_change();
            true
        } else {
            false
        };
        ensure_process_path(dir);
        Ok(added)
    }

    /// 当前进程 PATH 缺 dir 时前置插入（注册表已有但本会话未刷新时仍可在子进程里找到）。
    fn ensure_process_path(dir: &str) {
        let cur = std::env::var("PATH").unwrap_or_default();
        if let Some(new) = crate::envpath::add_path_entry(&cur, dir) {
            std::env::set_var("PATH", new);
        }
    }

    /// 通知 Explorer / 新进程重读环境（已打开的终端不会自动刷新）。
    fn notify_env_change() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };
        let mut name: Vec<u16> = "Environment".encode_utf16().collect();
        name.push(0);
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                name.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                1000,
                std::ptr::null_mut(),
            );
        }
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
        let norm = |s: &str| {
            expand_env_vars(s)
                .trim_end_matches(['\\', '/'])
                .to_lowercase()
        };
        let want = norm(dir);
        Ok(raw
            .split(';')
            .any(|p| !p.trim().is_empty() && norm(p) == want))
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
        notify_env_change();
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
        notify_env_change();
        Ok(())
    }

    /// 读用户级环境变量（未设置返回 None；heal 判 OK/HEALED 用）。
    pub fn get_user_env_var(key: &str) -> Result<Option<String>, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
        Ok(env.get_value(key).ok())
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

    /// O3（S017）：写自定义钩子行进独立标记块（幂等：块内已含该行不重写）。
    /// 供 fnm 的 `eval "$(fnm env)"` 等非 export 形态钩子用。
    pub(super) fn ensure_profile_hook(marker: &str, line: &str) -> Result<(), String> {
        let text = read_profile()?;
        if text.contains(line) {
            return Ok(());
        }
        let end = marker.replace(">>>", "<<<");
        let new_text = format!("{text}\n{marker}\n{line}\n{end}\n");
        write_profile(&new_text)
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

    fn profile_is_fish() -> bool {
        std::env::var("SHELL")
            .map(|s| s.contains("fish"))
            .unwrap_or(false)
    }

    fn format_export(dir: &str) -> String {
        if profile_is_fish() {
            format!("fish_add_path -P {dir}")
        } else {
            format!(r#"export PATH="{}:$PATH""#, dir)
        }
    }

    fn parse_export_dir(line: &str) -> Option<String> {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("export PATH=") {
            let v = rest.trim().trim_matches('"');
            return Some(v.trim_end_matches(":$PATH").to_string());
        }
        t.strip_prefix("fish_add_path -P ")
            .or_else(|| t.strip_prefix("fish_add_path "))
            .map(|s| s.trim().to_string())
    }

    fn ome_path_dirs(text: &str) -> Vec<String> {
        let mut dirs = Vec::new();
        let mut in_block = false;
        for line in text.lines() {
            if line.trim().starts_with(OME_PATH_MARKER) {
                in_block = true;
                continue;
            }
            if in_block && line.trim().starts_with(OME_PATH_END) {
                break;
            }
            if in_block {
                if let Some(d) = parse_export_dir(line) {
                    dirs.push(d);
                }
            }
        }
        dirs
    }

    fn upsert_ome_path_block(text: &str, dirs: &[String]) -> String {
        let body: String = dirs
            .iter()
            .map(|d| format!("{}\n", format_export(d)))
            .collect();
        let block = format!("{OME_PATH_MARKER}\n{body}{OME_PATH_END}\n");
        if text.contains(OME_PATH_MARKER) {
            let stripped = remove_ome_path_block(text);
            let mut out = stripped;
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&block);
            out
        } else {
            let mut out = text.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&block);
            out
        }
    }

    pub fn add_user_path(dir: &Path) -> Result<bool, String> {
        let dir_str = dir.to_string_lossy().to_string();
        let text = read_profile()?;
        let mut dirs = ome_path_dirs(&text);
        let added = if dirs.iter().any(|d| path_entries_eq(d, &dir_str)) {
            false
        } else {
            dirs.push(dir_str.clone());
            write_profile(&upsert_ome_path_block(&text, &dirs))?;
            true
        };
        let cur = std::env::var("PATH").unwrap_or_default();
        if !cur.split(':').any(|p| path_entries_eq(p, &dir_str)) {
            std::env::set_var("PATH", format!("{dir_str}:{cur}"));
        }
        Ok(added)
    }

    pub fn remove_user_path(dir: &Path) -> Result<bool, String> {
        let dir_str = dir.to_string_lossy().to_string();
        let text = read_profile()?;
        let dirs = ome_path_dirs(&text);
        if !dirs.iter().any(|d| path_entries_eq(d, &dir_str)) {
            return Ok(false);
        }
        let kept: Vec<String> = dirs
            .into_iter()
            .filter(|d| !path_entries_eq(d, &dir_str))
            .collect();
        let new_text = if kept.is_empty() {
            remove_ome_path_block(&text)
        } else {
            upsert_ome_path_block(&text, &kept)
        };
        write_profile(&new_text)?;
        let cur = std::env::var("PATH").unwrap_or_default();
        let kept_path = cur
            .split(':')
            .filter(|p| !p.is_empty() && !path_entries_eq(p, &dir_str))
            .collect::<Vec<_>>()
            .join(":");
        std::env::set_var("PATH", kept_path);
        Ok(true)
    }

    pub fn user_path_contains(dir: &Path) -> Result<bool, String> {
        let dir_str = dir.to_string_lossy().to_string();
        let text = read_profile()?;
        Ok(ome_path_dirs(&text)
            .iter()
            .any(|d| path_entries_eq(d, &dir_str)))
    }

    /// PATH 原始条目（profile 的 ome PATH 标记块内各目录；doctor 诊断用）。
    pub fn profile_path_entries() -> Result<Vec<String>, String> {
        let text = read_profile()?;
        Ok(ome_path_dirs(&text))
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

    /// 读用户级环境变量：profile 的 ome env 标记块内找 `export KEY="value"`（未设置返回 None）。
    pub fn get_user_env_var(key: &str) -> Result<Option<String>, String> {
        let text = read_profile()?;
        let prefix = format!("export {key}=\"");
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix(&prefix) {
                if let Some(v) = rest.strip_suffix('"') {
                    return Ok(Some(v.to_string()));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_var_or_主名优先与旧名读回() {
        // D41 自测 1：同设主名优先、主名空白视同未设、仅旧名读回、都未设为 None
        std::env::set_var("ARK_TEST_ENVVAR_X", "primary");
        std::env::set_var("OME_TEST_ENVVAR_X", "fallback");
        assert_eq!(
            env_var_or("ARK_TEST_ENVVAR_X", "OME_TEST_ENVVAR_X").as_deref(),
            Some("primary"),
            "同设时主名优先"
        );
        std::env::remove_var("ARK_TEST_ENVVAR_X");
        assert_eq!(
            env_var_or("ARK_TEST_ENVVAR_X", "OME_TEST_ENVVAR_X").as_deref(),
            Some("fallback"),
            "主名未设读回旧名"
        );
        std::env::set_var("ARK_TEST_ENVVAR_X", "  ");
        assert_eq!(
            env_var_or("ARK_TEST_ENVVAR_X", "OME_TEST_ENVVAR_X").as_deref(),
            Some("fallback"),
            "主名纯空白视同未设"
        );
        std::env::remove_var("ARK_TEST_ENVVAR_X");
        std::env::remove_var("OME_TEST_ENVVAR_X");
        assert_eq!(
            env_var_or("ARK_TEST_ENVVAR_X", "OME_TEST_ENVVAR_X"),
            None,
            "都未设为 None"
        );
    }

    #[cfg(windows)]
    #[test]
    fn self_deploy_target_windows_用户程序目录与envroot解耦() {
        let t = self_deploy_target().expect("self-deploy 目标应可解析");
        assert!(
            t.ends_with("Programs\\ark\\ark.exe"),
            "目标应在用户程序目录: {}",
            t.display()
        );
        assert!(
            !t.starts_with(default_env_root()),
            "目标不应在 EnvRoot 下: {}",
            t.display()
        );
    }

    #[test]
    fn 元数据目录_主名ark_旧ohmyenv在位读回() {
        // D41 迁移过渡三态：皆无取主名、仅旧在取旧位、新在归位主名
        let dir = tempfile::tempdir().expect("临时目录");
        assert_eq!(
            resolve_metadata_dir(dir.path()),
            dir.path().join("ark"),
            "两者皆无应取主名 ark"
        );
        std::fs::create_dir_all(dir.path().join("ohmyenv")).expect("建旧目录");
        assert_eq!(
            resolve_metadata_dir(dir.path()),
            dir.path().join("ohmyenv"),
            "旧 ohmyenv 在而新未建应读回旧位（共享 catalog 与 seq）"
        );
        std::fs::create_dir_all(dir.path().join("ark")).expect("建新目录");
        assert_eq!(
            resolve_metadata_dir(dir.path()),
            dir.path().join("ark"),
            "新 ark 建成后应归位主名"
        );
    }

    #[cfg(windows)]
    #[test]
    fn metadata_dir_windows_独立于envroot() {
        let m = metadata_dir();
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

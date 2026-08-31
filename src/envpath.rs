//! envpath：注册表用户 PATH 管理，对齐 helpers.ps1 的 Add-EnvPath/Remove-EnvPath（1321-1370 行）。
//! 读 HKCU\Environment\Path 原始值（不展开 %VAR%），比较时展开后去重（大小写不敏感，
//! 对齐 PowerShell -contains/-ne 语义），前置插入，写回保留 REG_EXPAND_SZ 类型，
//! 并同步当前进程 PATH。
//! 危险守卫：本模块只动 HKCU 用户 PATH，绝不碰 HKLM 系统 PATH。

use crate::toolver::expand_env_vars;

/// 纯函数：向原始 PATH 串前置插入 dir；已存在（展开后相等）返回 None。
/// 对齐 Add-EnvPath：$parts 存未展开，比较用展开后形式。
pub fn add_path_entry(raw: &str, dir: &str) -> Option<String> {
    let expanded_dir = expand_env_vars(dir);
    let parts: Vec<&str> = raw.split(';').filter(|p| !p.is_empty()).collect();
    let exists = parts
        .iter()
        .any(|p| expand_env_vars(p).eq_ignore_ascii_case(&expanded_dir));
    if exists {
        return None;
    }
    let mut new_parts = vec![dir];
    new_parts.extend(parts);
    Some(new_parts.join(";"))
}

/// 纯函数：从原始 PATH 串移除 dir（展开后相等者全部移除）。对齐 Remove-EnvPath。
pub fn remove_path_entry(raw: &str, dir: &str) -> String {
    let expanded_dir = expand_env_vars(dir);
    raw.split(';')
        .filter(|p| !p.is_empty())
        .filter(|p| !expand_env_vars(p).eq_ignore_ascii_case(&expanded_dir))
        .collect::<Vec<_>>()
        .join(";")
}

/// 注册表读用户 PATH 原始值（winreg 不展开 REG_EXPAND_SZ，拿到的即未展开串）。
#[cfg(windows)]
fn read_user_path_raw() -> Result<String, String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
    // Path 值可能不存在（新用户）：按空串处理
    env.get_value::<String, _>("Path").or(Ok(String::new()))
}

/// 注册表写用户 PATH，强制 REG_EXPAND_SZ 类型（避免 %VAR% 引用被降级为字面路径）。
#[cfg(windows)]
fn write_user_path_raw(raw: &str) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE, RegType};
    use winreg::{RegKey, RegValue};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_WRITE)
        .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
    // REG_EXPAND_SZ 原始字节：UTF-16LE + 结尾 NUL
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

/// 用户 PATH 幂等前置注册并同步当前进程；返回是否实际新增。只动 HKCU，不碰系统 PATH。
#[cfg(windows)]
pub fn add_user_path(dir: &str) -> Result<bool, String> {
    let raw = read_user_path_raw()?;
    let Some(new_raw) = add_path_entry(&raw, dir) else {
        eprintln!("[INFO] PATH 已存在，跳过: {dir}");
        return Ok(false);
    };
    write_user_path_raw(&new_raw)?;
    // 同步当前进程 PATH（前置，对齐 pwsh $env:Path = "$Dir;$env:Path"）
    let cur = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{dir};{cur}"));
    eprintln!("[OK] PATH 已注册（前置）: {dir}");
    Ok(true)
}

/// 从用户 PATH 移除 dir 并同步当前进程；返回是否有移除。只动 HKCU，不碰系统 PATH。
#[cfg(windows)]
pub fn remove_user_path(dir: &str) -> Result<bool, String> {
    let raw = read_user_path_raw()?;
    let new_raw = remove_path_entry(&raw, dir);
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
    eprintln!("[OK] PATH 已移除: {dir}");
    Ok(true)
}

/// 用户 PATH 是否已含 dir：读注册表原始值、分号切分、大小写不敏感比较（对齐 ohmyenv.ps1 status 的
/// `-contains` 判定；.NET GetEnvironmentVariable(User) 读的也是不展开的原始值）。
#[cfg(windows)]
pub fn user_path_contains(dir: &str) -> Result<bool, String> {
    let raw = read_user_path_raw()?;
    Ok(raw.split(';').any(|p| p.eq_ignore_ascii_case(dir)))
}

#[cfg(not(windows))]
pub fn user_path_contains(_dir: &str) -> Result<bool, String> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_前置插入_保未展开条目() {
        // %USERPROFILE% 条目必须以未展开形式保留（review 实测降级为字面路径会静默失效）
        std::env::set_var("OME_TEST_HOME", r"C:\Users\demo");
        let raw = r"%OME_TEST_HOME%\bin;C:\tools";
        let new = add_path_entry(raw, r"D:\ohmyenv\jq").expect("应新增");
        assert_eq!(new, r"D:\ohmyenv\jq;%OME_TEST_HOME%\bin;C:\tools");
    }

    #[test]
    fn add_展开后已存在_跳过且大小写不敏感() {
        std::env::set_var("OME_TEST_HOME", r"C:\Users\demo");
        // 已存在条目的展开形式与 dir 相同（变量形式不同也算重复）
        let raw = r"%OME_TEST_HOME%\bin;C:\tools";
        assert_eq!(add_path_entry(raw, r"C:\Users\demo\bin"), None);
        // 大小写不敏感（对齐 PowerShell -contains）
        assert_eq!(add_path_entry(raw, r"c:\users\demo\BIN"), None);
        assert_eq!(add_path_entry(raw, r"C:\TOOLS"), None);
    }

    #[test]
    fn add_空串与尾分号边界() {
        assert_eq!(
            add_path_entry("", r"D:\x").as_deref(),
            Some(r"D:\x"),
            "空 PATH 应只含新条目（不产出前导/尾随分号）"
        );
        assert_eq!(
            add_path_entry(r"C:\a;", r"D:\x").as_deref(),
            Some(r"D:\x;C:\a"),
            "尾分号产生的空段应被过滤（对齐 Where-Object {{ $_ }}）"
        );
        // 孤 \ 段是非空串，PowerShell Where-Object { $_ } 判定为真会保留——实现须对齐而非自作聪明清掉
        assert_eq!(
            add_path_entry(r"C:\a;\", r"D:\x").as_deref(),
            Some(r"D:\x;C:\a;\")
        );
    }

    #[test]
    fn remove_展开后匹配_大小写不敏感() {
        std::env::set_var("OME_TEST_HOME", r"C:\Users\demo");
        let raw = r"D:\ohmyenv\jq;%OME_TEST_HOME%\bin;C:\tools";
        assert_eq!(remove_path_entry(raw, r"d:\ohmyenv\JQ"), r"%OME_TEST_HOME%\bin;C:\tools");
        assert_eq!(remove_path_entry(raw, r"C:\Users\demo\bin"), r"D:\ohmyenv\jq;C:\tools");
    }

    #[test]
    fn remove_不存在_原样返回() {
        let raw = r"C:\a;C:\b";
        assert_eq!(remove_path_entry(raw, r"C:\nope"), raw);
    }
}

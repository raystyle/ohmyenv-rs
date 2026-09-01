//! toolver：已装版本探测，移植 helpers.ps1 的 Get-InstalledVersion（888-935 行）。
//! 每工具的版本参数表（默认 --version；7z 用 --help、rmux 用 -V、oscdimg 无参读横幅）
//! 与输出版本正则表；exe 路径解析：official 工具 exe 字段含 %VAR% 环境变量（展开为绝对路径），
//! 其余相对 EnvRoot 拼接。装后读版本带 5 次递增重试（500ms * i，对齐 Install-ToolVersion 末尾）。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use regex::Regex;

use crate::catalog::Tool;

/// exe 路径解析：official（exe 使用环境变量或绝对路径）展开为绝对路径；
/// 其余 Windows 下相对 EnvRoot；Linux / macOS 下有平台专属 exe（`linux_exe`/`mac_exe`，
/// 相对 install_dir）走 dir 展开，回退通用 exe（Windows 风格，自带 dir 段）时相对 EnvRoot。
pub fn exe_path(tool: &Tool, env_root: &Path) -> Result<PathBuf, String> {
    let exe = tool.exe().ok_or_else(|| "工具缺少 exe 字段".to_string())?;
    if crate::platform::is_official_exe(exe) {
        return Ok(PathBuf::from(expand_env_vars(exe)));
    }
    #[cfg(windows)]
    {
        Ok(env_root.join(exe))
    }
    #[cfg(not(windows))]
    if let Some(pexe) = tool.platform_exe() {
        let base = tool
            .dir()
            .map(|d| {
                crate::platform::join_if_relative(env_root, crate::platform::expand_install_path(d))
            })
            .unwrap_or_else(|| env_root.to_path_buf());
        // 专属 exe 允许带子目录（如 python 的 bin/python），将反斜杠统一为正斜杠
        Ok(base.join(pexe.replace('\\', "/")))
    } else {
        // 通用 exe 是 Windows 名录风格：路径自带 dir 段，相对 EnvRoot 直接拼
        Ok(env_root.join(exe.replace('\\', "/")))
    }
}

/// 工具是否 official 布局（exe 使用环境变量或绝对路径，installDir/bin 走官方目录，不进 EnvRoot）。
pub fn is_official(tool: &Tool) -> bool {
    tool.exe()
        .map(crate::platform::is_official_exe)
        .unwrap_or(false)
}

/// 当前平台是否管理该工具：effective exe 存在。
/// 只有 `linux_*`/`mac_*` 字段而无通用 `exe` 的工具（如 shellcheck）在 Windows 属平台不适用，
/// status 出空态行、install/update/daily/pin/query 跳过（对齐 mac 侧「如实空态」语义）。
pub fn platform_managed(tool: &Tool) -> bool {
    tool.exe().is_some()
}

/// 环境变量展开（Windows `%VAR%`；Linux / macOS `$VAR` / `${VAR}`）。
/// 未定义的变量原样保留。
pub fn expand_env_vars(s: &str) -> String {
    crate::platform::expand_env_vars(s)
}

/// 版本探测参数表（对齐 Get-InstalledVersion 的 switch；oscdimg 无参数读横幅）。
pub fn version_args(tool: &str) -> Vec<&'static str> {
    match tool {
        "7z" => vec!["--help"],
        "rmux" => vec!["-V"],
        // oscdimg 无 --version；无参运行首行横幅含版本（OSCDIMG 2.56 ...），等价 pwsh 读 FileVersion
        "oscdimg" => vec![],
        // vsbuild 探测 MSBuild：-version 的 stdout 首行即裸版本号（17.14.51.32402）
        "vsbuild" => vec!["-version"],
        // zig / go 用子命令 version，不认 --version
        "zig" => vec!["version"],
        "go" => vec!["version"],
        _ => vec!["--version"],
    }
}

/// 输出版本正则表（与 helpers.ps1 的 switch 逐条对应）；无表项的工具返回 None。
pub fn version_pattern(tool: &str) -> Option<&'static str> {
    Some(match tool {
        "pwsh" => r"PowerShell\s+(\d+\.\d+\.\d+)",
        "gh" => r"gh version (\d+\.\d+\.\d+)",
        "git" => r"git version (\S+)",
        "age" => r"^v?(\d+\.\d+\.\d+)",
        "sops" => r"sops[ -]v?(\d+\.\d+\.\d+)",
        "vault" => r"Vault\s+v?(\d+\.\d+\.\d+)",
        "aria2" => r"aria2 version (\d+\.\d+\.\d+)",
        "7z" => r"7-Zip[^\r\n]*?(\d+\.\d+)",
        "dotnet" => r"^(\d+\.\d+\.\d+)",
        "fnm" => r"fnm\s+v?(\d+\.\d+\.\d+)",
        "bun" => r"^v?(\d+\.\d+\.\d+)",
        "gsudo" => r"gsudo\s+v?(\d+\.\d+\.\d+)",
        "uv" => r"uv (\d+\.\d+\.\d+)",
        "python" => r"Python (\d+\.\d+\.\d+)",
        "rg" => r"ripgrep (\d+\.\d+\.\d+)",
        "jq" => r"jq-(\d+\.\d+\.\d+)",
        "mq" => r"mq\s+v?(\d+\.\d+\.\d+)",
        "yq" => r"version v?(\d+\.\d+\.\d+)",
        "starship" => r"starship (\d+\.\d+\.\d+)",
        "just" => r"just\s+v?(\d+\.\d+\.\d+)",
        "ast-grep" => r"(\d+\.\d+\.\d+)",
        "nushell" => r"^(\d+\.\d+\.\d+)",
        "herdr" => r"^herdr\s+v?(\d+\.\d+\.\d+)",
        "rumdl" => r"rumdl\s+(\d+\.\d+\.\d+)",
        "rmux" => r"rmux\s+(\d+\.\d+\.\d+)",
        "oscdimg" => r"OSCDIMG\s+(\d+\.\d+)",
        "reader" => r"reader\s+(\d+\.\d+\.\d+)",
        // go version 输出：go version go1.27.0 darwin/arm64
        "go" => r"go version go(\d+\.\d+\.\d+)",
        "zig" => r"(\d+\.\d+\.\d+)",
        // shellcheck --version 输出含 version: 0.11.0 行
        "shellcheck" => r"version:\s*(\d+\.\d+\.\d+)",
        // MSBuild -version：中文横幅「…版本 17.14.51+…」或英文首行裸版本，均取首段三段号
        "vsbuild" => r"(\d+\.\d+\.\d+)",
        _ => return None,
    })
}

/// 从首个非空行解析版本（纯函数，对齐 pwsh 的 Where-Object 非空 + Select-First 1 + switch）。
pub fn parse_version(tool: &str, output: &str) -> Option<String> {
    let line = output.lines().find(|l| !l.trim().is_empty())?;
    let pattern = version_pattern(tool)?;
    let re = Regex::new(pattern).ok()?;
    let caps = re.captures(line)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// 探测已装版本：exe 不存在直接 None；运行 exe 取首行非空输出按正则表解析。
pub fn installed_version(exe: &Path, tool: &str) -> Option<String> {
    if !exe.exists() {
        return None;
    }
    let out = Command::new(exe).args(version_args(tool)).output().ok()?;
    // stdout 与 stderr 合并取首个非空行（对齐 pwsh 2>&1）
    let merged = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_version(tool, &merged)
}

/// 装后版本读取：5 次递增重试（500ms * i），对齐 Install-ToolVersion 末尾的重试循环
/// （7zsfx 等解包后文件/杀软可能瞬态未就绪）。
pub fn installed_version_retried(exe: &Path, tool: &str) -> Option<String> {
    for i in 1..=5u32 {
        if let Some(v) = installed_version(exe, tool) {
            return Some(v);
        }
        std::thread::sleep(Duration::from_millis(500 * u64::from(i)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 正则表命中：期望值取自各工具真实 --version 输出样例（对照 helpers.ps1 正则逐条）。
    #[test]
    fn 版本正则表_命中真实输出样例() {
        let cases: &[(&str, &str, &str)] = &[
            ("pwsh", "PowerShell 7.6.5", "7.6.5"),
            ("gh", "gh version 2.98.0 (2026-01-01)", "2.98.0"),
            ("git", "git version 2.55.0.windows.4", "2.55.0.windows.4"),
            ("age", "v1.3.1", "1.3.1"),
            ("sops", "sops 3.13.3 (latest)", "3.13.3"),
            ("vault", "Vault v2.0.4 (abc), built 2026-01-01", "2.0.4"),
            ("aria2", "aria2 version 1.37.0", "1.37.0"),
            ("7z", "\n7-Zip 26.02 (x64) : Copyright", "26.02"),
            ("dotnet", "10.0.400", "10.0.400"),
            ("fnm", "fnm 1.39.0", "1.39.0"),
            ("bun", "1.3.14", "1.3.14"),
            ("gsudo", "gsudo v2.6.1", "2.6.1"),
            ("uv", "uv 0.12.6 (abc123 2026-01-01)", "0.12.6"),
            ("python", "Python 3.12.11", "3.12.11"),
            ("rg", "ripgrep 15.2.0", "15.2.0"),
            ("jq", "jq-1.8.2", "1.8.2"),
            ("mq", "mq 0.8.4", "0.8.4"),
            (
                "yq",
                "yq (https://github.com/mikefarah/yq/) version v4.53.6",
                "4.53.6",
            ),
            ("starship", "starship 1.26.0", "1.26.0"),
            ("just", "just 1.58.0", "1.58.0"),
            ("ast-grep", "ast-grep 0.45.1", "0.45.1"),
            ("nushell", "0.115.1", "0.115.1"),
            ("herdr", "herdr 0.8.2", "0.8.2"),
            ("rumdl", "rumdl 0.2.62", "0.2.62"),
            ("rmux", "rmux 0.10.0", "0.10.0"),
            ("go", "go version go1.27.0 darwin/arm64", "1.27.0"),
            ("zig", "0.16.0", "0.16.0"),
            ("shellcheck", "version: 0.11.0", "0.11.0"),
            (
                "oscdimg",
                "\nOSCDIMG 2.56 CD-ROM and DVD-ROM Premastering Utility",
                "2.56",
            ),
            ("reader", "reader 0.1.0", "0.1.0"),
            ("vsbuild", "17.14.51.32402", "17.14.51"),
            (
                "vsbuild",
                "适用于 .NET Framework MSBuild 版本 17.14.51+25f168cee",
                "17.14.51",
            ),
        ];
        for (tool, line, expect) in cases {
            assert_eq!(
                parse_version(tool, line).as_deref(),
                Some(*expect),
                "{tool} 应解析出版本"
            );
        }
    }

    #[test]
    fn 版本解析_取首个非空行() {
        // 7z --help 首行为空行（pwsh 注释实测）
        assert_eq!(
            parse_version("7z", "\n\n7-Zip 26.02 (x64)").as_deref(),
            Some("26.02")
        );
    }

    #[test]
    fn dies_未知工具与垃圾输出解析为none() {
        assert_eq!(parse_version("unknown-tool", "1.2.3"), None);
        assert_eq!(parse_version("jq", "not a version"), None);
        assert_eq!(parse_version("jq", ""), None);
    }

    #[test]
    #[cfg(windows)]
    fn env变量展开_未定义原样保留() {
        std::env::set_var("OME_TEST_VAR_X", r"C:\somewhere");
        assert_eq!(
            expand_env_vars(r"%OME_TEST_VAR_X%\bin\tool.exe"),
            r"C:\somewhere\bin\tool.exe"
        );
        assert_eq!(
            expand_env_vars(r"%OME_NO_SUCH_VAR%\x.exe"),
            r"%OME_NO_SUCH_VAR%\x.exe"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn exe路径_专属exe相对安装目录_通用exe相对envroot() {
        let root = Path::new("/tmp/ome-root");
        // 专属 exe（linux_exe，mac 上 mac_exe 同理）相对 install_dir：dir 展开后绝对路径直用
        let platform = Tool {
            dir: Some("~/.local/bin".to_string()),
            exe: Some(r"jq\jq.exe".to_string()),
            linux_exe: Some("jq".to_string()),
            ..Tool::default()
        };
        let home = dirs::home_dir().expect("应能取 home");
        assert_eq!(
            exe_path(&platform, root).expect("应解析"),
            home.join(".local").join("bin").join("jq")
        );
        // 相对 dir（EnvRoot 下子目录语义）拼 EnvRoot 再接专属 exe
        let rel_dir = Tool {
            dir: Some("python".to_string()),
            linux_exe: Some("bin/python".to_string()),
            ..Tool::default()
        };
        assert_eq!(
            exe_path(&rel_dir, root).expect("应解析"),
            root.join("python").join("bin").join("python")
        );
        // 回退通用 exe（Windows 名录风格，路径自带 dir 段）直接相对 EnvRoot——
        // 回归：曾误拼 install_dir 产出 age/age/age.exe（黄金 status.txt 契约：<root>/age/age.exe）
        let generic = Tool {
            dir: Some("age".to_string()),
            exe: Some(r"age\age.exe".to_string()),
            ..Tool::default()
        };
        assert_eq!(
            exe_path(&generic, root).expect("应解析"),
            root.join("age").join("age.exe")
        );
    }

    #[test]
    #[cfg(windows)]
    fn exe路径_official展开_envroot拼接() {
        let root = Path::new(r"D:\sandbox");
        let green = Tool {
            exe: Some(r"jq\jq.exe".to_string()),
            ..Tool::default()
        };
        assert_eq!(
            exe_path(&green, root).expect("应解析"),
            PathBuf::from(r"D:\sandbox\jq\jq.exe")
        );
        assert!(!is_official(&green));

        std::env::set_var("OME_TEST_VAR_Y", r"C:\official");
        let official = Tool {
            exe: Some(r"%OME_TEST_VAR_Y%\rmux\bin\rmux.exe".to_string()),
            ..Tool::default()
        };
        assert_eq!(
            exe_path(&official, root).expect("应解析"),
            PathBuf::from(r"C:\official\rmux\bin\rmux.exe")
        );
        assert!(is_official(&official));
    }
}

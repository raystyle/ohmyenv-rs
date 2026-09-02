//! rustup：Rust 接管（自 ohmypwsh `scripts\set-rust.ps1` 平移，2026-09-02）。
//! rsproxy 引导器直链（无版本无 sha，evergreen 语义）：rustup-init 引导 stable 工具链，
//! 无 pin（`rustup update stable` 即更新语义，官方 6 周滚动，stable 只收安全补丁）。
//! 安装根重定位 EnvRoot：RUSTUP_HOME=`<EnvRoot>\rustup`、CARGO_HOME=`<EnvRoot>\cargo`（用户环境变量）；
//! rsproxy 双镜像：rustup 分发（RUSTUP_DIST_SERVER/RUSTUP_UPDATE_ROOT）加 cargo sparse（config.toml）；
//! cargo bin 进用户 PATH（ome 惯例尾部追加；set-rust 为前置，已装机器由 ps1 前置位保持不变）。
//! 幂等：rustc 在位不重跑 init（update stable 照跑保最新）；config.toml 内容一致不重写。

use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

use crate::catalog::Tool;
#[cfg(windows)]
use crate::download;
use crate::install::InstallAction;
use crate::install::InstallOutcome;
#[cfg(windows)]
use crate::platform;
#[cfg(windows)]
use crate::toolver;

/// 引导器缓存文件名。
pub const INIT_EXE: &str = "rustup-init.exe";

/// rsproxy rustup 分发镜像（用户环境变量两键）。
const DIST_SERVER: &str = "https://rsproxy.cn";
const UPDATE_ROOT: &str = "https://rsproxy.cn/rustup";

/// cargo 镜像配置（与 set-rust.ps1 逐字节一致：crates-io 换 rsproxy sparse、git 走 cli、http 两开关）。
const CARGO_CONFIG: &str = r#"[source.crates-io]
replace-with = "rsproxy"

[source.rsproxy]
registry = "sparse+https://rsproxy.cn/index/"

[net]
git-fetch-with-cli = true

[http]
check-revoke = false
multiplexing = true
"#;

/// 是否 rustup 引导器型条目（extract = "rustup"）。
pub fn is_rustup(def: &Tool) -> bool {
    def.extract() == Some("rustup")
}

/// RUSTUP_HOME：`<EnvRoot>\rustup`。
pub fn rustup_home(env_root: &Path) -> PathBuf {
    env_root.join("rustup")
}

/// CARGO_HOME：`<EnvRoot>\cargo`。
pub fn cargo_home(env_root: &Path) -> PathBuf {
    env_root.join("cargo")
}

/// rustc.exe（`<EnvRoot>\cargo\bin\rustc.exe`，verify dev-rust 维度断言位）。
pub fn rustc_exe(env_root: &Path) -> PathBuf {
    cargo_home(env_root).join("bin").join("rustc.exe")
}

/// rustup-init 引导参数（纯函数可测；对齐 set-rust.ps1：stable + msvc host + 不动 PATH）。
pub fn init_args() -> Vec<String> {
    [
        "-y",
        "--default-toolchain",
        "stable",
        "--default-host",
        "x86_64-pc-windows-msvc",
        "--no-modify-path",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// 安装（幂等）：环境变量重定位 → rustup-init 引导（缺 rustc 时）→ update stable 保最新
/// → cargo 镜像 config.toml → 用户 PATH → rustc 版本校验。
pub fn install(def: &Tool, env_root: &Path) -> Result<InstallOutcome, String> {
    #[cfg(not(windows))]
    {
        let _ = (def, env_root);
        Err("rust 条目仅支持 Windows（POSIX 无 rust 部署维度）".to_string())
    }
    #[cfg(windows)]
    {
        let rustup_home = rustup_home(env_root);
        let cargo_home = cargo_home(env_root);
        let rustc = rustc_exe(env_root);
        let url = def
            .cdn_url()
            .ok_or_else(|| "rust 条目缺少 cdn_url 字段".to_string())?;

        // ── 1. 目录与环境变量（重定位 + rsproxy 镜像；比较后写，进程内同步供子进程消费）──
        for dir in [&rustup_home, &cargo_home] {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("创建目录失败: {}: {e}", dir.display()))?;
        }
        for (k, v) in [
            ("RUSTUP_DIST_SERVER", DIST_SERVER),
            ("RUSTUP_UPDATE_ROOT", UPDATE_ROOT),
            ("RUSTUP_HOME", &rustup_home.display().to_string()),
            ("CARGO_HOME", &cargo_home.display().to_string()),
        ] {
            let cur = platform::get_user_env_var(k)?;
            if cur.as_deref() != Some(v) {
                platform::set_user_env_var(k, v)?;
                eprintln!("[OK] 用户环境变量已设: {k}={v}（新终端生效）");
            } else {
                eprintln!("[INFO] {k} 已是 {v}");
            }
            std::env::set_var(k, v);
        }

        // ── 2. rustup-init 引导（rustc 在位则跳过；引导器读取进程内重定位变量）──
        let had_rustc = toolver::installed_version(&rustc, "rust").is_some();
        if !had_rustc {
            // 永续引导器：rsproxy 直链无版本无官方 sha，不做 pin 校验（evergreen 语义）
            let init = download::download_asset(env_root, INIT_EXE, url, None, false)?;
            eprintln!("[INFO] 运行 rustup-init（stable / x86_64-pc-windows-msvc）...");
            let status = Command::new(&init)
                .args(init_args())
                .status()
                .map_err(|e| format!("rustup-init 启动失败: {}: {e}", init.display()))?;
            if !status.success() {
                return Err(format!(
                    "rustup-init 失败 exit={}",
                    status.code().unwrap_or(-1)
                ));
            }
        } else {
            eprintln!("[INFO] rustc 已安装，跳过 rustup-init");
        }

        // ── 3. 保持最新 stable（幂等；失败不拦——网络抖动不应让在位安装报错，末尾版本校验兜底）──
        let rustup = cargo_home.join("bin").join("rustup.exe");
        if rustup.exists() {
            for args in [vec!["update", "stable"], vec!["default", "stable"]] {
                if let Ok(st) = Command::new(&rustup).args(&args).status() {
                    if !st.success() {
                        eprintln!("[WARN] rustup {:?} 未成功（继续）", args);
                    }
                }
            }
        }

        // ── 4. cargo 镜像（内容一致不重写；UTF-8 无 BOM）──
        let cargo_cfg = cargo_home.join("config.toml");
        let existing = std::fs::read_to_string(&cargo_cfg).unwrap_or_default();
        if existing != CARGO_CONFIG {
            std::fs::write(&cargo_cfg, CARGO_CONFIG)
                .map_err(|e| format!("写 cargo config 失败: {}: {e}", cargo_cfg.display()))?;
            eprintln!("[OK] cargo 镜像已写入 config.toml");
        } else {
            eprintln!("[INFO] cargo 镜像已是最新");
        }

        // ── 5. 用户 PATH（cargo bin）──
        let bin_dir = cargo_home.join("bin");
        if !platform::user_path_contains(&bin_dir)? {
            platform::add_user_path(&bin_dir)?;
            eprintln!("[OK] 用户 PATH 已加 cargo bin（新终端生效）");
        } else {
            eprintln!("[INFO] cargo bin 已在 PATH");
        }

        // ── 6. 校验 ──
        let version = toolver::installed_version_retried(&rustc, "rust")
            .ok_or_else(|| "rustc 版本校验失败（安装后未探测到）".to_string())?;
        eprintln!("[OK] rust 安装完成: {version}");
        Ok(InstallOutcome {
            action: if had_rustc {
                InstallAction::Skipped
            } else {
                InstallAction::Installed
            },
            version,
            dir: Some(cargo_home),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 引导器参数形态：静默 + stable 工具链 + msvc host + 不动 PATH（对齐 set-rust.ps1）。
    #[test]
    fn 引导器参数_形态对齐脚本() {
        assert_eq!(
            init_args(),
            vec![
                "-y".to_string(),
                "--default-toolchain".to_string(),
                "stable".to_string(),
                "--default-host".to_string(),
                "x86_64-pc-windows-msvc".to_string(),
                "--no-modify-path".to_string(),
            ]
        );
    }

    /// cargo 镜像配置：crates-io 替换源 + rsproxy sparse + git cli + http 两开关（脚本语义关键标志）。
    #[test]
    fn cargo镜像_关键标志齐全() {
        assert!(CARGO_CONFIG.contains(r#"replace-with = "rsproxy""#));
        assert!(CARGO_CONFIG.contains(r#"registry = "sparse+https://rsproxy.cn/index/""#));
        assert!(CARGO_CONFIG.contains("git-fetch-with-cli = true"));
        assert!(CARGO_CONFIG.contains("check-revoke = false"));
        assert!(CARGO_CONFIG.contains("multiplexing = true"));
    }
}

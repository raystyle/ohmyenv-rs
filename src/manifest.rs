//! manifest.toml：安装配置部署逻辑的数据面（R016 B 层，D39 第一波引擎）。
//!
//! 结构：`schema_version` 加每工具一节 `[manifest.<tool>]`，含 L1 声明原语
//! （`env_set` 用户级键值表、`shims` 别名表）与 L2 受控命令（`post_install`
//! 分平台 argv 数组——每条是参数数组非 shell 字符串，无元字符解释）。
//!
//! 生命周期：与 tools.toml 同目录（catalog sync 顺带拉取三件套，同锚同签）；
//! 文件或工具节缺失时零原语、内建行为回退（双轨过渡，omc manifest 数据上线后撤内建）。
//! 高 `schema_version` 拒载并提示升级 ome（R016 前进兼容红线）。

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// 引擎支持的 manifest schema 大版本（R016 v0.2 定稿）。
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// manifest.toml 根结构。
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ManifestFile {
    pub schema_version: Option<u32>,
    pub manifest: HashMap<String, ToolManifest>,
}

/// 单工具节：L1 声明原语加 L2 受控命令（未识别字段由 serde default 容忍，lint 白名单把关）。
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ToolManifest {
    /// L1：用户级环境变量键值表（win=注册表 HKCU、POSIX=profile 标记块，platform 通道）。
    pub env_set: Option<HashMap<String, String>>,
    /// L1：别名表（键=别名名、值=同 bin 目录的源名；win=硬链接加 .cmd 兜底、POSIX=符号链接）。
    pub shims: Option<HashMap<String, String>>,
    /// L2：受控命令（分平台 argv 数组）。
    pub post_install: Option<PostInstall>,
}

/// L2 受控命令：每平台一组 argv 数组；`skip` 显式声明「该平台无命令」（三键齐备 lint 依据）。
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PostInstall {
    pub win: Option<Vec<Vec<String>>>,
    pub linux: Option<Vec<Vec<String>>>,
    pub mac: Option<Vec<Vec<String>>>,
    pub skip: Option<Vec<String>>,
}

/// L2 单条命令执行超时（R016 三节：300s 默认）。
const POST_INSTALL_TIMEOUT_SECS: u64 = 300;

/// 载入 manifest.toml：与 tools.toml 同目录；缺文件返回空（零原语）；
/// 高 schema 版本拒载（报错由调用方传导）。
pub fn load(tools_dir: &Path) -> Result<ManifestFile, String> {
    let path = tools_dir.join("manifest.toml");
    if !path.exists() {
        return Ok(ManifestFile::default());
    }
    parse(&std::fs::read_to_string(&path).map_err(|e| format!("读 manifest 失败: {}: {e}", path.display()))?)
}

/// 解析（纯函数可测；反序列化走 toml_edit serde feature，零新增依赖）。
pub fn parse(text: &str) -> Result<ManifestFile, String> {
    let f: ManifestFile =
        toml_edit::de::from_str(text).map_err(|e| format!("manifest.toml 解析失败: {e}"))?;
    match f.schema_version.unwrap_or(1) {
        v if v <= SUPPORTED_SCHEMA_VERSION => Ok(f),
        v => Err(format!(
            "manifest schema_version {v} 高于引擎支持 {SUPPORTED_SCHEMA_VERSION}：请 ome self update 后重试"
        )),
    }
}

/// 当前平台键选 L2 命令（纯函数可测）。
pub fn platform_commands<'a>(pi: &'a PostInstall) -> impl Iterator<Item = &'a Vec<String>> {
    let list = if cfg!(windows) {
        pi.win.as_ref()
    } else if cfg!(target_os = "macos") {
        pi.mac.as_ref()
    } else {
        pi.linux.as_ref()
    };
    list.into_iter().flatten()
}

/// 平台是否被显式跳过（三键齐备语义：有命令或进 skip 即齐备）。
pub fn platform_covered(pi: &PostInstall) -> bool {
    let (has, name) = if cfg!(windows) {
        (pi.win.is_some(), "win")
    } else if cfg!(target_os = "macos") {
        (pi.mac.is_some(), "mac")
    } else {
        (pi.linux.is_some(), "linux")
    };
    has || pi.skip.as_ref().is_some_and(|s| s.iter().any(|p| p == name))
}

/// L1：应用用户级环境变量（逐键幂等）。
pub fn apply_env_set(m: &ToolManifest) -> Result<(), String> {
    let Some(kv) = &m.env_set else { return Ok(()) };
    for (k, v) in kv {
        crate::platform::set_user_env_var(k, v)?;
        eprintln!("[OK] manifest env_set 已设: {k}={v}（新终端生效）");
    }
    Ok(())
}

/// L1：生成别名（win=硬链接加 .cmd 兜底、POSIX=符号链接；目标已存在即跳过，幂等）。
pub fn apply_shims(m: &ToolManifest, bin_dir: &Path) -> Result<(), String> {
    let Some(shims) = &m.shims else { return Ok(()) };
    for (alias, source) in shims {
        let (src, dst) = if cfg!(windows) {
            (bin_dir.join(format!("{source}.exe")), bin_dir.join(format!("{alias}.exe")))
        } else {
            (bin_dir.join(source), bin_dir.join(alias))
        };
        if !src.exists() {
            eprintln!("[WARN] manifest shims 源不存在，跳过: {}", src.display());
            continue;
        }
        if dst.exists() {
            eprintln!("[INFO] manifest shims 已存在，跳过: {}", dst.display());
            continue;
        }
        #[cfg(windows)]
        {
            if std::fs::hard_link(&src, &dst).is_ok() {
                eprintln!("[OK] manifest shim 已创建（硬链接）: {}", dst.display());
                continue;
            }
            let cmd = bin_dir.join(format!("{alias}.cmd"));
            let exe = src.display().to_string().replace('\\', "\\\\");
            std::fs::write(&cmd, format!("@\"{exe}\" %*\r\n"))
                .map_err(|e| format!("写 {alias}.cmd 失败: {e}"))?;
            eprintln!("[OK] manifest shim 已创建（cmd 兜底）: {}", cmd.display());
        }
        #[cfg(not(windows))]
        {
            std::os::unix::fs::symlink(&src, &dst)
                .map_err(|e| format!("符号链接失败 {} -> {}: {e}", dst.display(), src.display()))?;
            eprintln!("[OK] manifest shim 已创建（符号链接）: {}", dst.display());
        }
    }
    Ok(())
}

/// L2：逐条执行受控命令；超时 300s 杀进程；失败只报不回滚，报告含退出码与输出尾行（R016 三节）。
pub fn run_post_install(m: &ToolManifest, tool: &str) -> Result<(), String> {
    let Some(pi) = &m.post_install else { return Ok(()) };
    if !platform_covered(pi) {
        return Err(format!(
            "{tool} post_install 未覆盖当前平台（无命令且未声明 skip，R016 三键齐备）"
        ));
    }
    for argv in platform_commands(pi) {
        if argv.is_empty() {
            return Err(format!("{tool} post_install 含空命令"));
        }
        eprintln!("[INFO] manifest post_install: {}", argv.join(" "));
        let mut child = std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("{tool} post_install 启动失败（{}）: {e}", argv.join(" ")))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(POST_INSTALL_TIMEOUT_SECS);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        let _ = child.kill();
                        return Err(format!(
                            "{tool} post_install 超时（{}s）已终止: {}",
                            POST_INSTALL_TIMEOUT_SECS,
                            argv.join(" ")
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => return Err(format!("{tool} post_install 等待失败: {e}")),
            }
        };
        // 失败报告：退出码加输出尾行（R016 三节，omc 评审建议）
        if !status.success() {
            let tail = |pipe: &mut dyn std::io::Read| {
                let mut s = String::new();
                let _ = pipe.read_to_string(&mut s);
                s.lines()
                    .rev()
                    .take(3)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join(" | ")
            };
            let mut tail_s = String::new();
            if let Some(mut p) = child.stdout.take() {
                tail_s.push_str(&tail(&mut p));
            }
            if let Some(mut p) = child.stderr.take() {
                if !tail_s.is_empty() {
                    tail_s.push_str(" ; ");
                }
                tail_s.push_str(&tail(&mut p));
            }
            return Err(format!(
                "{tool} post_install 失败（{}）: 退出码 {:?}{}（R016：只报不回滚）",
                argv.join(" "),
                status.code(),
                if tail_s.is_empty() { String::new() } else { format!("，尾行: {tail_s}") }
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 解析_工具节与三键() {
        let f = parse(
            "schema_version = 1\n[manifest.pwsh.env_set]\nPOWERSHELL_TELEMETRY_OPTOUT = \"1\"\n[manifest.bun]\n[manifest.bun.shims]\nbunx = \"bun\"\n[manifest.demo.post_install]\nwin = [[\"cmd\", \"/c\", \"echo hi\"]]\nskip = [\"linux\", \"mac\"]\n",
        )
        .expect("应解析");
        assert_eq!(f.manifest.len(), 3);
        assert_eq!(
            f.manifest["pwsh"].env_set.as_ref().unwrap()["POWERSHELL_TELEMETRY_OPTOUT"],
            "1"
        );
        assert_eq!(f.manifest["bun"].shims.as_ref().unwrap()["bunx"], "bun");
        assert!(f.manifest["demo"].post_install.is_some());
    }

    #[test]
    fn 高版本拒载() {
        let e = parse("schema_version = 2\n").expect_err("应拒载");
        assert!(e.contains("高于引擎支持"), "{e}");
    }

    #[test]
    fn 平台覆盖与选键() {
        // 当前平台的命令键（平台自适应，避免 win 视角写死）
        let cur = if cfg!(windows) {
            vec![vec!["cmd".to_string(), "/c".to_string(), "echo".to_string()]]
        } else {
            vec![vec!["echo".to_string()]]
        };
        let mut pi = PostInstall::default();
        if cfg!(windows) {
            pi.win = Some(cur);
        } else if cfg!(target_os = "macos") {
            pi.mac = Some(cur);
        } else {
            pi.linux = Some(cur);
        }
        assert!(platform_covered(&pi), "当前平台有命令即覆盖");
        assert_eq!(platform_commands(&pi).count(), 1);
        let empty = PostInstall::default();
        assert!(!platform_covered(&empty), "全空未覆盖");
        // skip 覆盖路径：当前平台无命令但显式 skip
        let mut skipped = PostInstall::default();
        let cur_name = if cfg!(windows) { "win" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
        skipped.skip = Some(vec![cur_name.to_string()]);
        assert!(platform_covered(&skipped), "显式 skip 即覆盖");
        assert_eq!(platform_commands(&skipped).count(), 0);
    }

    #[test]
    fn shims_生成与幂等() {
        let dir = tempfile::tempdir().expect("临时目录");
        let src = dir.path().join(if cfg!(windows) { "bun.exe" } else { "bun" });
        std::fs::write(&src, b"fake").expect("写源");
        let mut m = ToolManifest::default();
        m.shims = Some([("bunx".to_string(), "bun".to_string())].into());
        apply_shims(&m, dir.path()).expect("应成功");
        let dst = dir.path().join(if cfg!(windows) { "bunx.exe" } else { "bunx" });
        assert!(dst.exists(), "别名应生成");
        apply_shims(&m, dir.path()).expect("幂等二连应成功");
    }

    #[test]
    fn post_install_成功失败与覆盖缺失() {
        // 当前平台自适应命令（win=cmd /c echo、POSIX=echo），三键齐备形态
        let ok = if cfg!(windows) {
            vec!["cmd".to_string(), "/c".to_string(), "echo".to_string(), "ome-ok".to_string()]
        } else {
            vec!["echo".to_string(), "ome-ok".to_string()]
        };
        let mut m = ToolManifest::default();
        let mut pi = PostInstall::default();
        if cfg!(windows) {
            pi.win = Some(vec![ok]);
            pi.skip = Some(vec!["linux".into(), "mac".into()]);
        } else if cfg!(target_os = "macos") {
            pi.mac = Some(vec![ok]);
            pi.skip = Some(vec!["win".into(), "linux".into()]);
        } else {
            pi.linux = Some(vec![ok]);
            pi.skip = Some(vec!["win".into(), "mac".into()]);
        }
        m.post_install = Some(pi);
        run_post_install(&m, "t").expect("当前平台命令应成功");
        // 启动失败：当前平台一条不存在的命令
        let mut bad = ToolManifest::default();
        let mut bpi = PostInstall {
            ..Default::default()
        };
        let missing = vec!["definitely-missing-ome-bin".to_string()];
        if cfg!(windows) {
            bpi.win = Some(vec![missing]);
            bpi.skip = Some(vec!["linux".into(), "mac".into()]);
        } else if cfg!(target_os = "macos") {
            bpi.mac = Some(vec![missing]);
            bpi.skip = Some(vec!["win".into(), "linux".into()]);
        } else {
            bpi.linux = Some(vec![missing]);
            bpi.skip = Some(vec!["win".into(), "mac".into()]);
        }
        bad.post_install = Some(bpi);
        let e = run_post_install(&bad, "t").expect_err("应报失败");
        assert!(e.contains("失败"), "{e}");
        // 未覆盖：当前平台无命令且未 skip
        let mut uncovered = ToolManifest::default();
        let mut upi = PostInstall::default();
        if cfg!(windows) {
            upi.linux = Some(vec![vec!["echo".to_string()]]);
        } else {
            upi.win = Some(vec![vec!["cmd".to_string()]]);
        }
        uncovered.post_install = Some(upi);
        let e = run_post_install(&uncovered, "t").expect_err("未覆盖应报");
        assert!(e.contains("未覆盖"), "{e}");
    }
}

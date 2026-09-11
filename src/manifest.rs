//! manifest.toml：安装配置部署逻辑的数据面（R016 B 层，D39 第一波引擎）。
//!
//! 结构：`schema_version` 加每工具一节 `[manifest.<tool>]`，含 L1 声明原语
//! （`env_set` 用户级键值表、`shims` 别名表）与 L2 受控命令（`post_install`
//! 分平台 argv 数组——每条是参数数组非 shell 字符串，无元字符解释）。
//!
//! 生命周期：与 tools.toml 同目录（catalog sync 顺带拉取三件套，同锚同签）；
//! 文件或工具节缺失时零原语、零动作（内建双轨已于 2026-09-11 撤除，omc 数据面为唯一来源）。
//! 高 `schema_version` 拒载并提示升级 ome（R016 前进兼容红线）。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// 输出尾窗：滚动只留最近这么些字节（既抽干管道又不随输出无限吃内存；3 行足矣）。
const PIPE_TAIL_KEEP: usize = 64 * 1024;

/// 抽干线程回传尾窗的等待上限。子进程遗留的后台孙进程会一直持着写端不关，
/// 不能无限等（等不到就放弃尾行，不影响退出码与超时判定）。
const PIPE_DRAIN_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// 并发抽干一个管道并只回传尾窗：子进程输出超过管道缓冲（约 64KB）时写端会阻塞，
/// 若等它退出后再读，则子进程永不退出、轮询窗口耗尽而误判超时（M017 实证）。
fn drain_pipe<R: std::io::Read + Send + 'static>(mut r: R) -> std::sync::mpsc::Receiver<Vec<u8>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut keep: Vec<u8> = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            match r.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    keep.extend_from_slice(&buf[..n]);
                    if keep.len() > PIPE_TAIL_KEEP {
                        let cut = keep.len() - PIPE_TAIL_KEEP;
                        keep.drain(..cut);
                    }
                }
            }
        }
        let _ = tx.send(keep);
    });
    rx
}

/// 尾窗取尾 3 行拼一行（失败报告用；非 UTF-8 按损耗转换）。
fn tail_lines(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" | ")
}

/// manifest.toml 路径：与 catalog（tools.toml）同目录（R016 两件分离同批落位）。
/// 唯一推导，sync 落位、status 诊断与 install 消费共用，避免三处各拼一次漂移。
pub fn path_for(catalog: &Path) -> PathBuf {
    match catalog.parent() {
        Some(dir) => dir.join("manifest.toml"),
        None => PathBuf::from("manifest.toml"),
    }
}

/// 载入 manifest.toml（传 catalog 路径，落位与消费同一推导）；缺文件返回空（零原语）；
/// 高 schema 版本拒载（报错由调用方传导）。
pub fn load(catalog: &Path) -> Result<ManifestFile, String> {
    let path = path_for(catalog);
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
pub fn platform_commands(pi: &PostInstall) -> impl Iterator<Item = &Vec<String>> {
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

/// win `.cmd` 兜底内容（纯函数可测）：`%~dp0` 相对定位（
/// 目录含空格时整体加引号即可，路径不落进内容、无转义面），纯 ASCII 无 BOM（cmd 不认 BOM）。
pub fn shim_cmd_content(source: &str) -> String {
    format!("@\"%~dp0{source}.exe\" %*\r\n")
}

/// L1：生成别名（win=硬链接加 `.cmd` 兜底、POSIX=符号链接；目标已存在即跳过，幂等）。
/// `bin_dir` 必须是**目标二进制所在目录**（源与别名同目录；调用方传 exe 父目录而非工具根）。
/// win 硬链接要求同卷同 NTFS：跨卷或 exFAT 等不支持时自动回落到 `.cmd`。
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
            std::fs::write(&cmd, shim_cmd_content(source))
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
    run_post_install_with_timeout(
        m,
        tool,
        std::time::Duration::from_secs(POST_INSTALL_TIMEOUT_SECS),
    )
}

/// 超时可注入版（单测跑杀进程路径不必等 300s；语义与公开入口一致）。
fn run_post_install_with_timeout(
    m: &ToolManifest,
    tool: &str,
    timeout: std::time::Duration,
) -> Result<(), String> {
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
        // 两路管道并发抽干：不抽干则输出超管道缓冲的子进程写阻塞、永不退出（M017）
        let stdout_rx = child.stdout.take().map(drain_pipe);
        let stderr_rx = child.stderr.take().map(drain_pipe);
        let deadline = std::time::Instant::now() + timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        // Windows 下 kill 只杀直接子进程：cmd /c 的孙进程存活（D39 共识④），
                        // taskkill /T 按 PID 补杀整棵树（失败静默——目标可能已退出）
                        #[cfg(windows)]
                        {
                            let pid = child.id().to_string();
                            let _ = std::process::Command::new("taskkill")
                                .args(["/PID", &pid, "/T", "/F"])
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();
                        }
                        return Err(format!(
                            "{tool} post_install 超时（{}s）已终止: {}",
                            timeout.as_secs(),
                            argv.join(" ")
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => return Err(format!("{tool} post_install 等待失败: {e}")),
            }
        };
        // 失败报告：退出码加输出尾行（R016 三节，omc 评审建议；尾窗在抽干线程里已备好）
        if !status.success() {
            let mut tail_s = String::new();
            if let Some(rx) = &stdout_rx {
                if let Ok(buf) = rx.recv_timeout(PIPE_DRAIN_WAIT) {
                    tail_s.push_str(&tail_lines(&buf));
                }
            }
            if let Some(rx) = &stderr_rx {
                let mut err_tail = String::new();
                if let Ok(buf) = rx.recv_timeout(PIPE_DRAIN_WAIT) {
                    err_tail.push_str(&tail_lines(&buf));
                }
                if !err_tail.is_empty() {
                    if !tail_s.is_empty() {
                        tail_s.push_str(" ; ");
                    }
                    tail_s.push_str(&err_tail);
                }
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
#[allow(clippy::field_reassign_with_default)] // 平台自适应用例按需补键，字面量反而更难读
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
        // 链接而非拷贝：改源即见新内容（硬链接与符号链接同判，M016 平台自适应）
        std::fs::write(&src, b"changed").expect("改源");
        assert_eq!(std::fs::read(&dst).expect("读别名"), b"changed", "别名应与源同体");
        #[cfg(not(windows))]
        assert_eq!(
            std::fs::read_link(&dst).expect("应为符号链接"),
            src,
            "POSIX 别名应指向源"
        );
        apply_shims(&m, dir.path()).expect("幂等二连应成功");
    }

    #[test]
    fn post_install_成功失败与覆盖缺失() {
        // 当前平台自适应成功命令（win=cmd /c echo、POSIX=echo）
        let ok = if cfg!(windows) {
            vec!["cmd".to_string(), "/c".to_string(), "echo".to_string(), "ome-ok".to_string()]
        } else {
            vec!["echo".to_string(), "ome-ok".to_string()]
        };
        let mut m = ToolManifest::default();
        m.post_install = Some(pi_for(ok));
        run_post_install(&m, "t").expect("当前平台命令应成功");
        // 启动失败：一条不存在的命令
        let mut bad = ToolManifest::default();
        bad.post_install = Some(pi_for(vec!["definitely-missing-ome-bin".to_string()]));
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

    #[test]
    fn shim_cmd内容不嵌绝对路径() {
        // 内容用 %~dp0 相对定位：路径含空格靠引号兜住，不落盘绝对路径、无转义面（M017 面）
        let c = shim_cmd_content("bun");
        assert_eq!(c, "@\"%~dp0bun.exe\" %*\r\n");
        assert!(!c.contains('\\'), "内容不应含转义反斜杠: {c}");
        assert!(!c.contains('\u{feff}'), "cmd 不认 BOM");
    }

    #[test]
    fn post_install大输出不阻塞且报告失败尾行() {
        // 输出远超管道缓冲（此处约 1MiB）：不并发抽干则子进程写阻塞被误判超时（M017 实证）。
        // 生成器取「读一个大文件」而非 shell 循环：前者毫秒级且不吃负载（循环版在本机
        // 高负载下从 0.2s 漂到 21s，把有负载的机器变成假红）。
        let dir = tempfile::tempdir().expect("临时目录");
        let big_file = dir.path().join("big.txt");
        std::fs::write(&big_file, ("A".repeat(63) + "\r\n").repeat(16384)).expect("写大文件");
        assert!(
            std::fs::metadata(&big_file).expect("读元数据").len() > 512 * 1024,
            "输出必须远超管道缓冲才有判别力"
        );
        let path = big_file.display().to_string();
        let big = if cfg!(windows) {
            vec!["cmd".to_string(), "/c".to_string(), "type".to_string(), path]
        } else {
            vec!["cat".to_string(), path]
        };
        let mut m = ToolManifest::default();
        m.post_install = Some(pi_for(big));
        // 20s 窗口远小于 300s：真阻塞必超时，抽干后毫秒级成功
        run_post_install_with_timeout(&m, "t", std::time::Duration::from_secs(20))
            .expect("大输出应抽干不阻塞");
        // 失败路径：非零退出码加尾行（退出码与内容都进报告）
        let failing = if cfg!(windows) {
            vec!["cmd".to_string(), "/c".to_string(), "echo boom& exit /b 7".to_string()]
        } else {
            vec!["sh".to_string(), "-c".to_string(), "echo boom; exit 7".to_string()]
        };
        let mut bad = ToolManifest::default();
        bad.post_install = Some(pi_for(failing));
        let e = run_post_install_with_timeout(&bad, "t", std::time::Duration::from_secs(20))
            .expect_err("应报失败");
        assert!(e.contains("退出码 Some(7)"), "{e}");
        assert!(e.contains("boom"), "尾行应带输出: {e}");
    }

    #[test]
    fn post_install超时即杀进程() {
        // 短超时注入跑杀进程路径（真 300s 不可测）：应快速返回超时而非等命令自然结束
        let slow = if cfg!(windows) {
            vec!["cmd".to_string(), "/c".to_string(), "ping -n 20 127.0.0.1".to_string()]
        } else {
            vec!["sh".to_string(), "-c".to_string(), "sleep 20".to_string()]
        };
        let mut m = ToolManifest::default();
        m.post_install = Some(pi_for(slow));
        let started = std::time::Instant::now();
        let e = run_post_install_with_timeout(&m, "t", std::time::Duration::from_secs(1))
            .expect_err("应超时");
        assert!(e.contains("超时"), "{e}");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(10),
            "杀进程后应立即返回"
        );
    }

    /// 当前平台键的三键齐备构造（当前平台给命令、其余进 skip；M016 平台自适应纪律）。
    fn pi_for(cmd: Vec<String>) -> PostInstall {
        let mut pi = PostInstall::default();
        if cfg!(windows) {
            pi.win = Some(vec![cmd]);
            pi.skip = Some(vec!["linux".into(), "mac".into()]);
        } else if cfg!(target_os = "macos") {
            pi.mac = Some(vec![cmd]);
            pi.skip = Some(vec!["win".into(), "linux".into()]);
        } else {
            pi.linux = Some(vec![cmd]);
            pi.skip = Some(vec!["win".into(), "mac".into()]);
        }
        pi
    }
}

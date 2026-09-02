//! ome CLI 入口：query / pin（lock 别名）/ install / deploy / update / status / daily / init（self-deploy 别名）
//! / verify / heal / doctor / package / self。
//!
//! 输出纪律（吸收自 incurs 研究 S001，S003 扩展三格式）：
//! - stdout 只走数据：默认 key=value 逐行，`--format json|jsonl` 或 `--json` 切结构化，
//!   统一经 render 层输出，命令里不散写 println!；
//! - 人称提示（[INFO]/[OK]/[WARN]/[HINT]/[跳过] 等）一律 stderr；
//! - 错误出口为 OmeError（code/message/hint/exit_code），main 按 exit_code 退出；
//!   结构化模式下错误以单行 JSON 附 stderr 末行，stdout 保持纯数据。

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use ome::catalog::{self, Catalog};
use ome::install::{install_tool, InstallOptions, InstallOutcome};
use ome::omerr::OmeError;
use ome::render;
use ome::resolve::{resolve_tool, Resolution, ResolveOptions};
use ome::status::{self, DailyRow};

// ── 帮助示例元数据（各子命令示例集中于此，经 after_help 挂进帮助）──
const EX_QUERY: &str = "示例:\n  ome query gh --latest\n  ome query all";
const EX_PIN: &str = "示例:\n  ome pin\n  ome pin git --latest\n  ome lock git --version 2.55.0";
const EX_INSTALL: &str = "示例:\n  ome install git\n  ome install all --force";
const EX_DEPLOY: &str = "示例:\n  ome deploy gh\n  ome deploy gh --version 2.92.0";
const EX_UPDATE: &str = "示例:\n  ome update\n  ome update gh";
const EX_STATUS: &str = "示例:\n  ome status";
const EX_DAILY: &str = "示例:\n  ome daily --dry-run\n  ome daily --include-breaking";
const EX_INIT: &str = "示例:\n  ome init";
const EX_PACKAGE: &str =
    "示例:\n  ome package fnm --out ./deploy\n  ome package fnm --out ./deploy --latest";
const EX_VERIFY: &str = "示例:\n  ome verify\n  ome verify --check toolRoot,localbin16 --json";
const EX_HEAL: &str = "示例:\n  ome heal aria2\n  ome heal all --dry-run";
const EX_DOCTOR: &str = "示例:\n  ome doctor\n  ome doctor --json";
const EX_SELF: &str =
    "示例:\n  ome self update\n  ome self update --stable\n  ome self update --git";

#[derive(Parser)]
#[command(
    name = "ome",
    bin_name = "ome",
    about = "Oh My Env：全平台 Agent 工具及运行时依赖环境的部署、管理、验收与诊断 CLI"
)]
struct Cli {
    /// 环境根目录覆盖，默认读取 OHMYENV_ROOT 或平台默认路径
    #[arg(long, global = true)]
    env_root: Option<String>,

    /// 以 JSON 数组输出数据，等价 --format json
    #[arg(long, global = true, conflicts_with = "format")]
    json: bool,

    /// 输出格式：kv 逐行键值、json 整批数组、jsonl 逐块单行
    #[arg(long, global = true, value_enum)]
    format: Option<FormatArg>,

    #[command(subcommand)]
    command: Commands,
}

/// --format 取值（映射 render::Format）。
#[derive(clap::ValueEnum, Clone, Copy)]
enum FormatArg {
    Kv,
    Json,
    Jsonl,
}

impl From<FormatArg> for render::Format {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Kv => render::Format::Kv,
            FormatArg::Json => render::Format::Json,
            FormatArg::Jsonl => render::Format::Jsonl,
        }
    }
}

/// --latest / --tag / --version 三选项（query / pin / install / deploy 共用）。
#[derive(clap::Args, Clone, Default)]
struct VersionOpts {
    /// 解析最新版
    #[arg(long, conflicts_with_all = ["tag", "version"])]
    latest: bool,
    /// 指定 release tag
    #[arg(long, conflicts_with = "version")]
    tag: Option<String>,
    /// 指定版本号
    #[arg(long)]
    version: Option<String>,
}

impl VersionOpts {
    fn is_empty(&self) -> bool {
        !self.latest && self.tag.is_none() && self.version.is_none()
    }
}

#[derive(Subcommand)]
enum Commands {
    /// 解析工具版本与下载资产，不落盘
    #[command(after_help = EX_QUERY)]
    Query {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
    },
    /// 查看或设置版本锁定，未锁定的工具自动锁定最新版
    #[command(visible_alias = "lock", after_help = EX_PIN)]
    Pin {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
    },
    /// 安装工具到环境目录，按锁定版本，不注册 PATH
    #[command(after_help = EX_INSTALL)]
    Install {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
        /// 强制重装，跳过幂等检查
        #[arg(long)]
        force: bool,
    },
    /// 安装工具并注册用户 PATH
    #[command(after_help = EX_DEPLOY)]
    Deploy {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
        /// 强制重装，跳过幂等检查
        #[arg(long)]
        force: bool,
    },
    /// 更新工具到最新版并重新锁定
    #[command(after_help = EX_UPDATE)]
    Update {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        /// 强制重装，跳过幂等检查
        #[arg(long)]
        force: bool,
    },
    /// 对照锁定版本、已装版本与 PATH 三态
    #[command(after_help = EX_STATUS)]
    Status,
    /// 日常更新：同主版本自动升级，跨主版本保留待确认
    #[command(after_help = EX_DAILY)]
    Daily {
        /// 只预览不执行
        #[arg(long)]
        dry_run: bool,
        /// 跨主版本也强制更新
        #[arg(long)]
        include_breaking: bool,
    },
    /// 安装自身到用户程序目录，同步 catalog 并注册 PATH，幂等
    #[command(alias = "self-deploy", after_help = EX_INIT)]
    Init,
    /// 打包工具为可分发目录，不注册 PATH、不回写锁定
    #[command(after_help = EX_PACKAGE)]
    Package {
        /// 工具名
        tool: String,
        /// 输出目录，默认 <EnvRoot>/cache/deploy
        #[arg(short, long)]
        out: Option<String>,
        #[command(flatten)]
        opts: VersionOpts,
    },
    /// 按部署维度验收环境一致性，失败返回非零
    #[command(after_help = EX_VERIFY)]
    Verify {
        /// 只检查指定维度，逗号分隔
        #[arg(long)]
        check: Option<String>,
    },
    /// 幂等自愈指定部署维度（heal-map 嵌入注册表；agent 域键休眠、ohmypwsh 域键提示路由）
    #[command(after_help = EX_HEAL)]
    Heal {
        /// 维度名或 all（默认 all）
        #[arg(default_value = "all")]
        dim: String,
        /// 只打印将执行的动作，不执行
        #[arg(long)]
        dry_run: bool,
    },
    /// 诊断部署异常：版本漂移、PATH 死链重复、锁定缺失、缓存孤儿等，失败返回非零
    #[command(after_help = EX_DOCTOR)]
    Doctor,
    /// ome 自身管理
    #[command(name = "self", after_help = EX_SELF)]
    OmeSelf {
        #[command(subcommand)]
        cmd: SelfCmd,
    },
}

/// `ome self` 子命令面。
#[derive(Subcommand)]
enum SelfCmd {
    /// 升级自身：默认 dev 滚动源，--stable 拉 latest 正式版，--git 源码构建
    #[command(alias = "upgrade")]
    Update {
        /// 拉 latest 正式版（v* tag 封版产物）
        #[arg(long, conflicts_with = "git")]
        stable: bool,
        /// 源码安装：浅克隆仓库 cargo build 后替换（封版前通道，需 git 与 cargo）
        #[arg(long, conflicts_with = "stable")]
        git: bool,
    },
}

fn main() {
    match run() {
        Ok(()) => render::finish(),
        Err(e) => {
            // 错误前已产出的数据块照常上 stdout（如 daily 有保留项时的预览行）
            render::finish();
            if render::is_structured() {
                let mut obj = serde_json::Map::new();
                obj.insert("code".to_string(), serde_json::json!(e.code));
                obj.insert("message".to_string(), serde_json::json!(e.message));
                if let Some(hint) = &e.hint {
                    obj.insert("hint".to_string(), serde_json::json!(hint));
                }
                if let Ok(line) = serde_json::to_string(&serde_json::Value::Object(obj)) {
                    eprintln!("{line}");
                }
            } else {
                eprintln!("ome: {e}");
            }
            std::process::exit(e.exit_code);
        }
    }
}

fn run() -> Result<(), OmeError> {
    let cli = Cli::parse();
    let format = if cli.json {
        render::Format::Json
    } else {
        cli.format
            .map(render::Format::from)
            .unwrap_or(render::Format::Kv)
    };
    render::set_format(format);
    let env_root = catalog::resolve_env_root(cli.env_root.as_deref()).map_err(OmeError::from)?;
    let cat_path = catalog::resolve_catalog_path().map_err(OmeError::from)?;
    let cat = Catalog::load(&cat_path).map_err(OmeError::from)?;

    match cli.command {
        Commands::Query { tool, opts } => cmd_query(&cat, &tool, &opts).map_err(OmeError::from),
        Commands::Pin { tool, opts } => cmd_pin(&cat, &tool, &opts).map_err(OmeError::from),
        Commands::Install { tool, opts, force } => {
            cmd_install(&cat, &env_root, &tool, &opts, force, false).map_err(OmeError::from)
        }
        Commands::Deploy { tool, opts, force } => {
            cmd_install(&cat, &env_root, &tool, &opts, force, true).map_err(OmeError::from)?;
            if opts.is_empty() {
                eprintln!("[HINT] 已按锁定版本部署；如需升级到最新并锁定: ome update");
            }
            Ok(())
        }
        Commands::Update { tool, force } => {
            cmd_update(&cat, &env_root, &tool, force).map_err(OmeError::from)
        }
        Commands::Status => cmd_status(&cat, &env_root).map_err(OmeError::from),
        Commands::Daily {
            dry_run,
            include_breaking,
        } => cmd_daily(&cat, &env_root, dry_run, include_breaking),
        Commands::Init => cmd_init(&env_root).map_err(OmeError::from),
        Commands::Package { tool, out, opts } => {
            cmd_package(&cat, &env_root, &tool, out.as_deref(), &opts).map_err(OmeError::from)
        }
        Commands::Verify { check } => {
            cmd_verify(&cat, &env_root, check.as_deref()).map_err(OmeError::from)
        }
        Commands::Heal { dim, dry_run } => {
            cmd_heal(&cat, &env_root, &dim, dry_run).map_err(OmeError::from)
        }
        Commands::Doctor => cmd_doctor(&cat, &env_root).map_err(OmeError::from),
        Commands::OmeSelf {
            cmd: SelfCmd::Update { stable, git },
        } => {
            let channel = if git {
                ome::selfupdate::Channel::Git
            } else if stable {
                ome::selfupdate::Channel::Stable
            } else {
                ome::selfupdate::Channel::Dev
            };
            cmd_self_update(&env_root, channel).map_err(OmeError::from)
        }
    }
}

/// self update：升级自身（通道：dev 滚动 / stable 正式 / git 源码）。
fn cmd_self_update(env_root: &Path, channel: ome::selfupdate::Channel) -> Result<(), String> {
    let out = ome::selfupdate::self_update(env_root, channel)?;
    render::emit(&[
        kv("action", out.action),
        kv("channel", out.channel),
        kv("asset", &out.asset),
        kv("sha256", &out.sha256),
        kv("exe", &out.exe.display().to_string()),
        kv(
            "catalog",
            if out.catalog_synced {
                "synced"
            } else {
                "skipped"
            },
        ),
    ]);
    Ok(())
}

/// doctor：部署异常诊断，流式逐项输出。kv 输出 name=OK/WARN/FAIL（明细走 stderr）；
/// 结构化输出 check/status/detail 块（明细入字段，agent 免读 stderr）。FAIL 即 exit 1。
fn cmd_doctor(cat: &Catalog, env_root: &Path) -> Result<(), String> {
    let mut first = true;
    let rows = ome::doctor::run_doctor_with(cat, env_root, |r| {
        if render::is_structured() {
            let mut block = vec![kv("check", r.name), kv("status", r.status)];
            if !r.detail.is_empty() {
                block.push(kv("detail", &r.detail.join("; ")));
            }
            emit_block(&mut first, block);
        } else {
            render::emit(&[(r.name.to_string(), r.status.to_string())]);
        }
        for d in &r.detail {
            eprintln!("[{}] {}: {}", r.status, r.name, d);
        }
    })?;
    let (fails, warns, fail_names, _) = ome::doctor::summarize(&rows);
    eprintln!("[汇总] {} 项：FAIL {fails}、WARN {warns}", rows.len());
    if fails > 0 {
        return Err(format!(
            "诊断发现 {fails} 项 FAIL: {}",
            fail_names.join(", ")
        ));
    }
    Ok(())
}

/// verify：部署域验收维度检查，kv 输出 dim=PASS/FAIL/NA 收割行（ohmypwsh 按此正则消费），
/// 结构化输出 name/verdict 块。维度就绪即出（流式）。FAIL 即 exit 1。
fn cmd_verify(cat: &Catalog, env_root: &Path, check: Option<&str>) -> Result<(), String> {
    let filter: Vec<String> = check
        .map(|c| {
            c.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let mut first = true;
    let rows = ome::verify::run_verify_with(cat, env_root, &filter, |name, verdict| {
        if render::is_structured() {
            emit_block(
                &mut first,
                vec![kv("name", name), kv("verdict", verdict.as_str())],
            );
        } else {
            render::emit(&[(name.to_string(), verdict.as_str().to_string())]);
        }
        Ok(())
    })?;
    let (total, fails) = ome::verify::summarize(&rows);
    eprintln!("[汇总] {total} 项，FAIL {} 项", fails.len());
    if !fails.is_empty() {
        return Err(format!("验收失败 {} 项: {}", fails.len(), fails.join(", ")));
    }
    Ok(())
}

/// heal：部署维度幂等自愈。kv 输出 dim/action/params/result 收割行（明细走 stderr），
/// 结构化输出 dim/action/params/result/detail 块。有 fail/partial 结果即 exit 1。
fn cmd_heal(cat: &Catalog, env_root: &Path, dim: &str, dry_run: bool) -> Result<(), String> {
    let mut first = true;
    let rows = ome::heal::run_heal_with(cat, env_root, dim, dry_run, |r| {
        if render::is_structured() {
            let mut block = vec![
                kv("dim", &r.dim),
                kv("action", r.action),
                kv("params", &r.params),
                kv("result", &r.result),
            ];
            if !r.detail.is_empty() {
                block.push(kv("detail", &r.detail.join("; ")));
            }
            emit_block(&mut first, block);
        } else {
            let mut block = vec![
                kv("dim", &r.dim),
                kv("action", r.action),
                kv("result", &r.result),
            ];
            if !r.params.is_empty() {
                block.insert(2, kv("params", &r.params));
            }
            emit_block(&mut first, block);
        }
        for d in &r.detail {
            eprintln!("[{}] {}: {}", r.result, r.dim, d);
        }
        Ok(())
    })?;
    let healed = rows.iter().filter(|r| r.result == "healed").count();
    eprintln!(
        "[汇总] {} 项：healed {healed}、ok {}、fail/partial {}",
        rows.len(),
        rows.iter().filter(|r| r.result == "ok").count(),
        rows.iter()
            .filter(|r| r.result == "fail" || r.result == "partial")
            .count(),
    );
    let bad: Vec<String> = rows
        .iter()
        .filter(|r| r.result == "fail" || r.result == "partial")
        .map(|r| r.dim.clone())
        .collect();
    if !bad.is_empty() {
        return Err(format!(
            "自愈未完全成功 {} 项: {}",
            bad.len(),
            bad.join(", ")
        ));
    }
    Ok(())
}

/// 组装解析选项。
fn resolve_opts(opts: &VersionOpts) -> ResolveOptions {
    ResolveOptions {
        latest: opts.latest,
        tag: opts.tag.clone(),
        version: opts.version.clone(),
    }
}

/// query：只解析不下载，每工具输出 tool/tag/version/asset/size/url 六行 key=value。
fn cmd_query(cat: &Catalog, tool: &str, opts: &VersionOpts) -> Result<(), String> {
    let names = cat.select(tool)?;
    let ropts = resolve_opts(opts);
    let mut first = true;
    for name in &names {
        let def = cat.tool(name)?;
        if ome::vsbuild::is_vsbuild(def) {
            eprintln!("[INFO] {name} 为永续引导器条目，无远端版本解析");
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("version", "evergreen"),
                    kv("asset", ome::vsbuild::BOOTSTRAPPER),
                    kv("url", def.cdn_url().unwrap_or("")),
                ],
            );
            continue;
        }
        if ome::rustup::is_rustup(def) {
            eprintln!("[INFO] {name} 为 rustup 引导器条目（stable 滚动），无远端版本解析");
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("version", "evergreen"),
                    kv("asset", ome::rustup::INIT_EXE),
                    kv("url", def.cdn_url().unwrap_or("")),
                ],
            );
            continue;
        }
        if !ome::toolver::platform_managed(def) {
            eprintln!("[INFO] {name} 当前平台不适用（无本平台 exe 字段），跳过");
            emit_block(&mut first, vec![kv("tool", name), kv("action", "skipped")]);
            continue;
        }
        let r = resolve_tool(name, def, &ropts)?;
        emit_block(&mut first, resolution_rows(&r, true));
    }
    Ok(())
}

/// pin：无选项打印当前 pin（sha256 截前 16 位加 ...），未 pin 的自动解析最新并回写；
/// 有选项则解析并回写 tag/version/asset（版本变化时清 sha256）。对齐 ohmyenv.ps1 pin/lock。
fn cmd_pin(cat: &Catalog, tool: &str, opts: &VersionOpts) -> Result<(), String> {
    let names = cat.select(tool)?;
    let ropts = resolve_opts(opts);
    let mut first = true;
    for name in &names {
        let def = cat.tool(name)?;
        if ome::vsbuild::is_vsbuild(def) {
            eprintln!("[INFO] {name} 为 evergreen 引导器条目，无 pin 语义（install 幂等）");
            emit_block(&mut first, vec![kv("tool", name), kv("pin", "evergreen")]);
            continue;
        }
        if ome::rustup::is_rustup(def) {
            eprintln!(
                "[INFO] {name} 为 rustup 引导器条目（stable 滚动），无 pin 语义（install 即更新）"
            );
            emit_block(&mut first, vec![kv("tool", name), kv("pin", "evergreen")]);
            continue;
        }
        if !ome::toolver::platform_managed(def) {
            eprintln!("[INFO] {name} 当前平台不适用（无本平台 exe 字段），跳过");
            emit_block(&mut first, vec![kv("tool", name), kv("action", "skipped")]);
            continue;
        }
        // 版本锁定（hold）：pin 不动（显示当前锁定并提示解锁方式）
        if def.is_held() {
            eprintln!("[INFO] {name} 已锁定（hold），pin 不变；解锁需删 catalog 的 hold 字段");
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("pin", def.pin_version().unwrap_or("held")),
                ],
            );
            continue;
        }
        if opts.is_empty() && def.pin_tag().is_some() {
            // 已 pin：只打印当前锁定
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("tag", def.pin_tag().unwrap_or("")),
                    kv("version", def.pin_version().unwrap_or("")),
                    kv("asset", def.pin_asset().unwrap_or("")),
                    kv("sha256", &short_sha(def.pin_sha256())),
                ],
            );
            continue;
        }
        // 未 pin 且无选项：自动解析最新并回写；有选项：按选项解析并回写
        let eff = if opts.is_empty() {
            eprintln!("[INFO] {name} 未 pin，自动 pin 最新版");
            ResolveOptions {
                latest: true,
                ..ResolveOptions::default()
            }
        } else {
            ropts.clone()
        };
        let r = resolve_tool(name, def, &eff)?;
        let version_changed = catalog::write_pin(&cat.path, name, &r)?;
        eprintln!(
            "[OK] {name} 已 pin: {}{}",
            r.version,
            if version_changed {
                "（sha256 已清除，将在 install 时回填）"
            } else {
                ""
            }
        );
        emit_block(&mut first, resolution_rows(&r, false));
    }
    Ok(())
}

/// install/deploy：解析（默认锁定版本）→ 安装；deploy 额外注册用户 PATH。
fn cmd_install(
    cat: &Catalog,
    env_root: &Path,
    tool: &str,
    opts: &VersionOpts,
    force: bool,
    register_path: bool,
) -> Result<(), String> {
    let names = cat.select(tool)?;
    let ropts = resolve_opts(opts);
    let iopts = InstallOptions {
        register_path,
        update_lock: false,
        force,
    };
    let mut first = true;
    let mut errors: Vec<String> = Vec::new();
    for name in &names {
        let def = cat.tool(name)?;
        // 平台不适用（无本平台 exe，如 shellcheck 在 Windows、Windows-only 工具在 Linux）：跳过不安装
        if !ome::toolver::platform_managed(def) {
            eprintln!("[INFO] {name} 当前平台不适用（无本平台 exe 字段），跳过");
            emit_block(&mut first, vec![kv("tool", name), kv("action", "skipped")]);
            continue;
        }
        // vsbuild：evergreen 引导器（无版本解析、需提权、机器级 PATH），走专用安装模块
        if ome::vsbuild::is_vsbuild(def) {
            match ome::vsbuild::install(def, env_root) {
                Ok(out) => emit_block(&mut first, install_rows(name, &out)),
                Err(e) => skip_or_fail(tool, name, e, &mut errors)?,
            }
            continue;
        }
        // rust：rustup 引导器（rsproxy 直链、stable 滚动、EnvRoot 重定位），走专用安装模块
        if ome::rustup::is_rustup(def) {
            match ome::rustup::install(def, env_root) {
                Ok(out) => emit_block(&mut first, install_rows(name, &out)),
                Err(e) => skip_or_fail(tool, name, e, &mut errors)?,
            }
            continue;
        }
        // docker：static zip + Windows 服务注册 + daemon.json + compose 插件（set-docker.ps1 迁移），走专用模块
        if ome::docker::is_docker(def) {
            let step = resolve_tool(name, def, &ropts)
                .and_then(|r| ome::docker::install(def, env_root, &r));
            match step {
                Ok(out) => emit_block(&mut first, install_rows(name, &out)),
                Err(e) => skip_or_fail(tool, name, e, &mut errors)?,
            }
            continue;
        }
        // 版本锁定（hold）：带版本选项的安装拒绝漂移；无选项按 pin 走（幂等）
        if def.is_held() && !opts.is_empty() {
            eprintln!(
                "[INFO] {name} 已锁定（hold）：{}，拒绝按选项安装；解锁需删 catalog 的 hold 字段",
                def.pin_version().unwrap_or("")
            );
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("action", "skipped"),
                    kv("version", def.pin_version().unwrap_or("")),
                ],
            );
            continue;
        }
        let step = resolve_tool(name, def, &ropts)
            .and_then(|r| install_tool(cat, env_root, name, &r, &iopts).map(|out| (r, out)));
        match step {
            Ok((_, out)) => emit_block(&mut first, install_rows(name, &out)),
            Err(e) => skip_or_fail(tool, name, e, &mut errors)?,
        }
    }
    summarize_all_errors(&errors)
}

/// all 循环容错：单工具失败时跳过续跑（WARN 加 skipped 行），单工具显式调用即时失败。
/// 返回 Err(()) 仅用于中断循环（调用方以 ? 传播），实际错误信息已在 errors 中。
fn skip_or_fail(
    tool_arg: &str,
    name: &str,
    e: String,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    if tool_arg == "all" {
        eprintln!("[WARN] {name}: {e}（all 循环跳过继续）");
        errors.push(format!("{name}: {e}"));
        return Ok(());
    }
    Err(e)
}

/// all 循环收尾：有失败项则汇总报错（exit 非零），单工具路径恒 Ok。
fn summarize_all_errors(errors: &[String]) -> Result<(), String> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "all 循环 {} 项失败: {}",
            errors.len(),
            errors.join("; ")
        ))
    }
}

/// update：--latest 解析，同 tag 跳过（不看 --force，对齐 ohmyenv.ps1 update），否则装 + 注册 + 回写。
fn cmd_update(cat: &Catalog, env_root: &Path, tool: &str, force: bool) -> Result<(), String> {
    let names = cat.select(tool)?;
    let ropts = ResolveOptions {
        latest: true,
        ..ResolveOptions::default()
    };
    let iopts = InstallOptions {
        register_path: true,
        update_lock: true,
        force,
    };
    let mut first = true;
    let mut errors: Vec<String> = Vec::new();
    for name in &names {
        let def = cat.tool(name)?;
        if ome::vsbuild::is_vsbuild(def) {
            eprintln!("[INFO] {name} 为 evergreen 引导器条目，不走 update（install 幂等）");
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("action", "skipped"),
                    kv("version", def.pin_version().unwrap_or("evergreen")),
                ],
            );
            continue;
        }
        if ome::rustup::is_rustup(def) {
            eprintln!("[INFO] {name} 为 rustup 引导器条目，不走 update（install 即 rustup update stable）");
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("action", "skipped"),
                    kv("version", def.pin_version().unwrap_or("evergreen")),
                ],
            );
            continue;
        }
        if !ome::toolver::platform_managed(def) {
            eprintln!("[INFO] {name} 当前平台不适用（无本平台 exe 字段），跳过");
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("action", "skipped"),
                    kv("version", def.pin_version().unwrap_or("")),
                ],
            );
            continue;
        }
        // 版本锁定（hold）：update 拒绝（含 --force）
        if def.is_held() {
            eprintln!(
                "[INFO] {name} 已锁定（hold）：{}，不更新；解锁需删 catalog 的 hold 字段",
                def.pin_version().unwrap_or("")
            );
            emit_block(
                &mut first,
                vec![
                    kv("tool", name),
                    kv("action", "skipped"),
                    kv("version", def.pin_version().unwrap_or("")),
                ],
            );
            continue;
        }
        let step = resolve_tool(name, def, &ropts).and_then(|r| {
            if def.pin_tag() == Some(r.tag.as_str()) {
                eprintln!(
                    "[INFO] {name} 已是最新: {}",
                    def.pin_version().unwrap_or("")
                );
                emit_block(
                    &mut first,
                    vec![
                        kv("tool", name),
                        kv("action", "skipped"),
                        kv("version", def.pin_version().unwrap_or("")),
                    ],
                );
                return Ok(());
            }
            match install_tool(cat, env_root, name, &r, &iopts) {
                Ok(out) => {
                    emit_block(&mut first, install_rows(name, &out));
                    Ok(())
                }
                Err(e) => Err(e),
            }
        });
        if let Err(e) = step {
            skip_or_fail(tool, name, e, &mut errors)?;
        }
    }
    summarize_all_errors(&errors)
}

/// status：locked / installed / path 三态对照，按 核心基础/扩展 两层分组（组标题为 # 注释行）。
fn cmd_status(cat: &Catalog, env_root: &Path) -> Result<(), String> {
    render::header(&format!("环境根目录: {}", env_root.display()));
    let mut last_tier = String::new();
    let mut last_cat = String::new();
    let mut first = true;
    // 流式：每探完一个工具立即输出（探测要逐工具拉起 --version 子进程，整批探完才打印会被感知为卡顿）
    status::collect_status_with(cat, env_root, |row| {
        let tier = status::tier_of(&row.category);
        if tier != last_tier {
            render::header(&format!("[{tier}]"));
            last_tier = tier.to_string();
            last_cat = String::new();
        }
        if tier == "核心基础工具" && row.category != last_cat {
            render::header(&format!("  [{}]", status::category_label(&row.category)));
            last_cat = row.category.clone();
        }
        emit_block(
            &mut first,
            vec![
                kv("tool", &row.name),
                kv("locked", row.locked.as_deref().unwrap_or("")),
                kv("installed", row.installed.as_deref().unwrap_or("-")),
                kv("path", if row.path { "true" } else { "false" }),
                kv(
                    "exe",
                    &row.exe
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ],
        );
        Ok(())
    })?;
    Ok(())
}

/// daily：同主版本自动、跨主版本保留，流式逐工具出判定行；有保留项 exit 2（OmeError 通道）。
fn cmd_daily(
    cat: &Catalog,
    env_root: &Path,
    dry_run: bool,
    include_breaking: bool,
) -> Result<(), OmeError> {
    let mut first = true;
    let (_rows, outcome) =
        status::run_daily_with(cat, env_root, dry_run, include_breaking, |row| {
            emit_block(&mut first, daily_rows(row));
        })
        .map_err(OmeError::from)?;
    if outcome.held > 0 {
        return Err(OmeError::new(
            "daily-held",
            format!("{} 项跨主版本更新保留待人工确认", outcome.held),
        )
        .with_hint("ome daily --include-breaking 强制更新")
        .with_exit_code(2));
    }
    Ok(())
}

/// init：复制当前 exe 到用户程序目录，同步 catalog 到用户数据目录，注册用户 PATH（幂等；self-deploy 别名）。
fn cmd_init(env_root: &Path) -> Result<(), String> {
    let out = ome::selfdeploy::self_deploy(env_root)?;
    let catalog = out
        .catalog
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "none".to_string());
    render::emit(&[
        kv("action", if out.copied { "deployed" } else { "current" }),
        kv("exe", &out.exe.display().to_string()),
        kv("bin_dir", &out.bin_dir.display().to_string()),
        kv("catalog", &catalog),
        kv(
            "path",
            if out.path_registered {
                "registered"
            } else {
                "exists"
            },
        ),
    ]);
    Ok(())
}

/// package：把工具打包到指定目录，默认 <EnvRoot>/cache/deploy/<tool>。
fn cmd_package(
    cat: &Catalog,
    env_root: &Path,
    tool: &str,
    out: Option<&str>,
    opts: &VersionOpts,
) -> Result<(), String> {
    let names = cat.select(tool)?;
    let ropts = resolve_opts(opts);
    let default_out = env_root.join("cache").join("deploy");
    let out_dir = out
        .map(Path::new)
        .unwrap_or(&default_out)
        .canonicalize()
        .unwrap_or_else(|_| out.map(PathBuf::from).unwrap_or(default_out));
    let mut first = true;
    for name in &names {
        let def = cat.tool(name)?;
        if ome::vsbuild::is_vsbuild(def) {
            return Err(format!(
                "{name} 是安装器型条目（VS 引导器），不支持 package 分发"
            ));
        }
        if !ome::toolver::platform_managed(def) {
            return Err(format!(
                "{name} 当前平台不适用（无本平台 exe 字段），无法打包"
            ));
        }
        let res = resolve_tool(name, def, &ropts)?;
        let out = ome::package::package_tool(cat, env_root, name, &res, &out_dir)?;
        emit_block(
            &mut first,
            vec![
                kv("tool", &out.tool),
                kv("version", &out.version),
                kv("package_dir", &out.package_dir.display().to_string()),
                kv("bin_dir", &out.bin_dir.display().to_string()),
                kv("main_bin", &out.main_bin.display().to_string()),
            ],
        );
    }
    Ok(())
}

// ── 输出行构造（数据行统一收敛为 Vec<(key, value)>，经 render 层输出）──

fn kv(k: &str, v: &str) -> (String, String) {
    (k.to_string(), v.to_string())
}

/// 解析结果行：query 含 size/url，pin 回写只出 tool/tag/version/asset。
fn resolution_rows(r: &Resolution, full: bool) -> Vec<(String, String)> {
    let mut rows = vec![
        kv("tool", &r.tool),
        kv("tag", &r.tag),
        kv("version", &r.version),
        kv("asset", &r.asset_name),
    ];
    if full {
        rows.push(kv("size", &r.asset_size.to_string()));
        rows.push(kv("url", &r.asset_url));
    }
    rows
}

/// 安装结果行：tool/action/version/dir。
fn install_rows(name: &str, out: &InstallOutcome) -> Vec<(String, String)> {
    let mut rows = vec![
        kv("tool", name),
        kv("action", out.action.as_str()),
        kv("version", &out.version),
    ];
    if let Some(d) = &out.dir {
        rows.push(kv("dir", &d.display().to_string()));
    }
    rows
}

/// daily 行：tool/action/from/to。
fn daily_rows(row: &DailyRow) -> Vec<(String, String)> {
    vec![
        kv("tool", &row.tool),
        kv("action", row.action),
        kv("from", &row.from),
        kv("to", &row.to),
    ]
}

/// 输出一组行（多工具之间空行分隔）。
fn emit_block(first: &mut bool, rows: Vec<(String, String)>) {
    if !*first {
        render::blank();
    }
    *first = false;
    render::emit(&rows);
}

/// sha256 展示：截前 16 位加 ...，未回填则标注（对齐 ohmyenv.ps1 pin 展示）。
fn short_sha(sha: Option<&str>) -> String {
    match sha {
        Some(s) if !s.is_empty() => {
            let head: String = s.chars().take(16).collect();
            format!("{head}...")
        }
        _ => "(未回填)".to_string(),
    }
}

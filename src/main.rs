//! ome CLI 入口：query / pin（lock 别名）/ install / deploy / update 子命令。
//! 输出协议：key=value 逐行（对齐 ohmyagents 约定）。

use std::path::Path;

use clap::{Parser, Subcommand};

use ome::catalog::{self, Catalog};
use ome::install::{install_tool, InstallOptions, InstallOutcome};
use ome::resolve::{resolve_tool, ResolveOptions, Resolution};

#[derive(Parser)]
#[command(name = "ome", about = "Oh My Env：本机 Windows 环境部署管理 CLI")]
struct Cli {
    /// 环境根目录覆盖（默认：OHMYENV_ROOT > 存在 D:\ 则 D:\ohmyenv 否则 C:\ohmyenv）
    #[arg(long, global = true)]
    env_root: Option<String>,

    #[command(subcommand)]
    command: Commands,
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
    /// 解析工具版本与资产（只查询不下载）
    Query {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
    },
    /// 查看/设置工具 pin；无选项打印当前 pin，未 pin 的自动 pin 最新版
    #[command(visible_alias = "lock")]
    Pin {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
    },
    /// 安装工具到环境目录（不改 PATH，默认锁定版本）
    Install {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
        /// 强制重装（跳过幂等检查）
        #[arg(long)]
        force: bool,
    },
    /// 安装 + 注册用户 PATH（默认锁定版本）
    Deploy {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        #[command(flatten)]
        opts: VersionOpts,
        /// 强制重装（跳过幂等检查）
        #[arg(long)]
        force: bool,
    },
    /// 更新到最新版并锁定（安装 + 注册 PATH + 回写 pin）
    Update {
        /// 工具名或 all
        #[arg(default_value = "all")]
        tool: String,
        /// 强制重装（跳过幂等检查）
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ome: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let env_root = catalog::resolve_env_root(cli.env_root.as_deref())?;
    let cat_path = catalog::resolve_catalog_path()?;
    let cat = Catalog::load(&cat_path)?;

    match cli.command {
        Commands::Query { tool, opts } => cmd_query(&cat, &tool, &opts),
        Commands::Pin { tool, opts } => cmd_pin(&cat, &tool, &opts),
        Commands::Install { tool, opts, force } => {
            cmd_install(&cat, &env_root, &tool, &opts, force, false)
        }
        Commands::Deploy { tool, opts, force } => {
            cmd_install(&cat, &env_root, &tool, &opts, force, true)?;
            if opts.is_empty() {
                eprintln!("[HINT] 已按锁定版本部署；如需升级到最新并锁定: ome update");
            }
            Ok(())
        }
        Commands::Update { tool, force } => cmd_update(&cat, &env_root, &tool, force),
    }
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
        let r = resolve_tool(name, def, &ropts)?;
        if !first {
            println!();
        }
        first = false;
        println!("tool={}", r.tool);
        println!("tag={}", r.tag);
        println!("version={}", r.version);
        println!("asset={}", r.asset_name);
        println!("size={}", r.asset_size);
        println!("url={}", r.asset_url);
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
        if opts.is_empty() && def.tag.is_some() {
            // 已 pin：只打印当前锁定
            print_pin_head(&mut first);
            println!("tool={name}");
            println!("tag={}", def.tag.as_deref().unwrap_or(""));
            println!("version={}", def.version.as_deref().unwrap_or(""));
            println!("asset={}", def.asset.as_deref().unwrap_or(""));
            println!("sha256={}", short_sha(def.sha256.as_deref()));
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
        print_resolution_pin(&mut first, &r);
    }
    Ok(())
}

fn print_pin_head(first: &mut bool) {
    if !*first {
        println!();
    }
    *first = false;
}

/// 解析后回写的输出：tool/tag/version/asset 四行 key=value。
fn print_resolution_pin(first: &mut bool, r: &Resolution) {
    print_pin_head(first);
    println!("tool={}", r.tool);
    println!("tag={}", r.tag);
    println!("version={}", r.version);
    println!("asset={}", r.asset_name);
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
    for name in &names {
        let def = cat.tool(name)?;
        let r = resolve_tool(name, def, &ropts)?;
        let out = install_tool(cat, env_root, name, &r, &iopts)?;
        print_install_outcome(&mut first, &out, name);
    }
    Ok(())
}

/// update：--latest 解析，同 tag 跳过，否则装 + 注册 PATH + 回写 pin（对齐 ohmyenv.ps1 update）。
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
    for name in &names {
        let def = cat.tool(name)?;
        let r = resolve_tool(name, def, &ropts)?;
        // 同 tag 即已是最新，跳过（对齐 ohmyenv.ps1 update：此判定不看 -Force）
        if def.tag.as_deref() == Some(r.tag.as_str()) {
            eprintln!("[INFO] {name} 已是最新: {}", def.version.as_deref().unwrap_or(""));
            print_install_lines(
                &mut first,
                name,
                "skipped",
                def.version.as_deref().unwrap_or(""),
                None,
            );
            continue;
        }
        let out = install_tool(cat, env_root, name, &r, &iopts)?;
        print_install_outcome(&mut first, &out, name);
    }
    Ok(())
}

/// 安装结果输出：tool/action/version/dir 四行 key=value。
fn print_install_outcome(first: &mut bool, out: &InstallOutcome, name: &str) {
    print_install_lines(
        first,
        name,
        out.action.as_str(),
        &out.version,
        out.dir.as_deref(),
    );
}

fn print_install_lines(
    first: &mut bool,
    name: &str,
    action: &str,
    version: &str,
    dir: Option<&Path>,
) {
    print_pin_head(first);
    println!("tool={name}");
    println!("action={action}");
    println!("version={version}");
    if let Some(d) = dir {
        println!("dir={}", d.display());
    }
}

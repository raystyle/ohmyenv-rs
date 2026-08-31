//! ome CLI 入口：第一阶段实现 query / pin（lock 别名）两个子命令。
//! 输出协议：key=value 逐行（对齐 ohmyagents 约定）。

use clap::{Parser, Subcommand};

use ome::catalog::{self, Catalog};
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

/// --latest / --tag / --version 三选项（query 与 pin 共用）。
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
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ome: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    // 本阶段 query/pin 不下载，EnvRoot 解析仅做覆盖链校验（install 等后续命令使用）
    let _env_root = catalog::resolve_env_root(cli.env_root.as_deref())?;
    let cat_path = catalog::resolve_catalog_path()?;
    let cat = Catalog::load(&cat_path)?;

    match cli.command {
        Commands::Query { tool, opts } => cmd_query(&cat, &tool, &opts),
        Commands::Pin { tool, opts } => cmd_pin(&cat, &tool, &opts),
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

//! resolve：版本解析三分支（cdn_index_url / cdn_url / GitHub REST），
//! 语义对齐 helpers.ps1 的 Resolve-ToolVersion / Get-HashiCorpIndex / Get-GitHubRelease。
//! 本模块只解析不下载；网络调用统一 30s 超时、3 次指数退避（2^n 秒），
//! api.github.com 在 403/限流等失败时回退 `gh api`（认证通道）。

use std::cmp::Ordering;
use std::process::Command;
use std::time::Duration;

use regex::Regex;
use serde_json::Value;

use crate::catalog::Tool;

const UA: &str = "ome-bootstrap";
const MAX_ATTEMPTS: u32 = 3;

/// 版本选择：--latest / --tag / --version 三选一（都不给则用 pin 的锁定版本）。
#[derive(Debug, Clone, Default)]
pub struct ResolveOptions {
    pub latest: bool,
    pub tag: Option<String>,
    pub version: Option<String>,
}

/// 解析结果：与 pwsh Resolve-ToolVersion 的返回字段对应（Release/Shasums 等运行时对象除外）。
/// shasums_url 仅 cdn_index_url 分支有值（HashiCorp SHA256SUMS 清单地址），供 checksum 用。
#[derive(Debug, Clone)]
pub struct Resolution {
    pub tool: String,
    pub tag: String,
    pub version: String,
    pub asset_name: String,
    pub asset_size: u64,
    pub asset_url: String,
    pub shasums_url: Option<String>,
}

/// 解析工具目标版本与资产：uv-git > cdn_index_url > cdn_url > GitHub release 四分支。
pub fn resolve_tool(name: &str, tool: &Tool, opts: &ResolveOptions) -> Result<Resolution, String> {
    if tool.extract() == Some("uv-git") {
        resolve_uv_git(name, tool, opts)
    } else if tool.cdn_index_url.is_some() {
        resolve_cdn_index(name, tool, opts)
    } else if tool.cdn_url().is_some() {
        resolve_cdn_url(name, tool, opts)
    } else {
        resolve_github(name, tool, opts)
    }
}

/// 分支 (0)：uv tool install git 型（如 browser-harness）——无预编译资产，
/// 版本即仓库 tag（pin 驱动，升级改 catalog pin），url 为 git+ 安装源仅作展示。
fn resolve_uv_git(name: &str, tool: &Tool, opts: &ResolveOptions) -> Result<Resolution, String> {
    let repo = tool
        .repo()
        .ok_or_else(|| format!("{name} 缺少 repo 字段（uv-git 型必需）"))?;
    let prefix = tool.tag_prefix.as_deref().unwrap_or("");
    let (tag, ver) = if let Some(v) = &opts.version {
        (format!("{prefix}{v}"), v.clone())
    } else if let Some(pinned) = tool.pin_tag() {
        (
            pinned.to_string(),
            tool.pin_version().unwrap_or_default().to_string(),
        )
    } else if let Some(t) = &opts.tag {
        (
            t.clone(),
            t.trim_start_matches(prefix)
                .trim_start_matches('v')
                .to_string(),
        )
    } else {
        return Err(format!(
            "{name} 未 pin 且未给 --tag/--version（uv-git 型无 latest 解析）"
        ));
    };
    Ok(Resolution {
        tool: name.to_string(),
        tag,
        version: ver,
        asset_name: format!("{repo}（uv tool install git 源）"),
        asset_size: 0,
        asset_url: format!("git+https://github.com/{repo}"),
        shasums_url: None,
    })
}

/// 分支 (a)：HashiCorp 式 index.json（如 vault）。
/// latest 时滤掉含 + 的企业变体（+ent/+ent.hsm 等），按语义版本取最大。
fn resolve_cdn_index(name: &str, tool: &Tool, opts: &ResolveOptions) -> Result<Resolution, String> {
    let index_url = tool
        .cdn_index_url
        .as_deref()
        .ok_or_else(|| format!("{name} 缺少 cdn_index_url"))?;

    // 版本选择对齐 pwsh：-Version > -Latest > 已 pin version（平台分列）> 报错
    let pinned = if let Some(v) = &opts.version {
        Some(v.clone())
    } else if opts.latest {
        None
    } else if let Some(v) = tool.pin_version() {
        Some(v.to_string())
    } else {
        return Err(format!("{name} 需 --version 或先 pin（HashiCorp 来源）"));
    };

    let index = get_json_retried(index_url, false)?;
    let versions = index
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("index.json 缺少 versions 字段: {index_url}"))?;

    let ver = if opts.latest {
        let oss: Vec<String> = versions
            .keys()
            .filter(|k| !k.contains('+'))
            .cloned()
            .collect();
        pick_max_semver(&oss).ok_or_else(|| format!("index.json 无 OSS 版本: {index_url}"))?
    } else {
        pinned.ok_or_else(|| format!("{name} 需 --version 或先 pin（HashiCorp 来源）"))?
    };

    let info = versions
        .get(&ver)
        .ok_or_else(|| format!("HashiCorp index.json 无版本 {ver}"))?;
    let real_ver = info
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or(&ver)
        .to_string();

    // 资产名模板里的 {version} 以正则转义后的真实版本替换（对齐 pwsh）；
    // pattern 走平台访问器（vault 类工具各平台资产名不同）
    let pattern = tool
        .cdn_asset_pattern()
        .ok_or_else(|| format!("{name} 缺少 cdn_asset_pattern"))?
        .replace("{version}", &regex::escape(&real_ver));
    let re = Regex::new(&pattern).map_err(|e| format!("{name} cdn_asset_pattern 非法: {e}"))?;

    let builds = info
        .get("builds")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("index.json {real_ver} 无 builds 字段"))?;
    let build = builds
        .iter()
        .find(|b| {
            b.get("filename")
                .and_then(Value::as_str)
                .map(|f| re.is_match(f))
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("{name} {real_ver} 在 index.json 中未找到匹配构建: {pattern}"))?;

    let filename = build
        .get("filename")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} 构建缺少 filename"))?
        .to_string();
    let url = build
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} 构建缺少 url"))?
        .to_string();

    // SHA256SUMS 清单地址：构建 URL 中文件名替换为 shasums 字段值
    let shasums_url = info
        .get("shasums")
        .and_then(Value::as_str)
        .map(|s| url.replace(&filename, s));

    Ok(Resolution {
        tool: name.to_string(),
        tag: real_ver.clone(),
        version: real_ver,
        asset_name: filename,
        asset_size: 0,
        asset_url: url,
        shasums_url,
    })
}

/// 分支 (b)：cdn_url 直链模板（含 {version} 占位，如 dotnet/oscdimg/grok）。
fn resolve_cdn_url(name: &str, tool: &Tool, opts: &ResolveOptions) -> Result<Resolution, String> {
    let cdn_url = tool
        .cdn_url()
        .ok_or_else(|| format!("{name} 缺少 cdn_url"))?;

    // 版本选择对齐 pwsh：-Version > 已 pin version（平台分列）> -Tag（去 v 前缀）> 报错
    let ver = if let Some(v) = &opts.version {
        v.clone()
    } else if let Some(v) = tool.pin_version() {
        v.to_string()
    } else if let Some(t) = &opts.tag {
        t.trim_start_matches('v').to_string()
    } else {
        return Err(format!("{name} 需 --version 指定版本（CDN 来源）"));
    };

    let url = cdn_url.replace("{version}", &ver);
    let asset_name = url
        .rsplit('/')
        .next()
        .ok_or_else(|| format!("{name} cdn_url 无法取资产名: {url}"))?
        .to_string();

    // tag 取真实 pin tag；无 pin 时按 tag_prefix 组合（如 go1.27.0），再退裸版本号
    let tag = tool
        .pin_tag()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}{ver}", tool.tag_prefix.as_deref().unwrap_or("")));

    Ok(Resolution {
        tool: name.to_string(),
        tag,
        version: ver,
        asset_name,
        asset_size: 0,
        asset_url: url,
        shasums_url: None,
    })
}

/// 分支 (c)：GitHub REST（releases/latest 或 releases/tags/{tag}）。
fn resolve_github(name: &str, tool: &Tool, opts: &ResolveOptions) -> Result<Resolution, String> {
    // repo 用 effective 访问器（Linux 取 linux_repo 回退通用）——仅 linux_repo 的工具
    // （如 shellcheck）在 Linux 解析不应报「缺少 repo」（2026-09-01 WSL install all 实证）
    let repo = tool
        .repo()
        .ok_or_else(|| format!("{name} 缺少 repo 字段（非 cdn 工具必须有）"))?;
    // psd1 语义：TagPrefix 为空即 tag 无前缀（uv/nushell/zig 等 tag 就是裸版本号），
    // 勿默认补 v——只有显式写 tag_prefix 的工具才带前缀
    let prefix = tool.tag_prefix.as_deref().unwrap_or("");

    let url = if opts.latest {
        format!("https://api.github.com/repos/{repo}/releases/latest")
    } else if let Some(tag) = &opts.tag {
        format!("https://api.github.com/repos/{repo}/releases/tags/{tag}")
    } else if let Some(ver) = &opts.version {
        format!("https://api.github.com/repos/{repo}/releases/tags/{prefix}{ver}")
    } else {
        let tag = tool.pin_tag().ok_or_else(|| {
            format!("{name} 尚未 pin 版本。先执行: ome pin {name} --latest（或 --version <版本>）")
        })?;
        format!("https://api.github.com/repos/{repo}/releases/tags/{tag}")
    };

    let release = get_json_retried(&url, true)?;
    let tag_name = release
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("release 缺少 tag_name: {url}"))?
        .to_string();

    // 资产按 asset_pattern 正则筛选，取第一个匹配
    let pattern = tool
        .asset_pattern()
        .ok_or_else(|| format!("{name} 缺少 asset_pattern"))?;
    let re = Regex::new(pattern).map_err(|e| format!("{name} asset_pattern 非法: {e}"))?;
    let assets = release
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("release 缺少 assets: {url}"))?;
    let asset = assets
        .iter()
        .find(|a| {
            a.get("name")
                .and_then(Value::as_str)
                .map(|n| re.is_match(n))
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("在 {tag_name} 中未找到匹配资产: {pattern}"))?;

    let asset_name = asset
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} 资产缺少 name"))?
        .to_string();
    let asset_size = asset.get("size").and_then(Value::as_u64).unwrap_or(0);
    let asset_url = asset
        .get("browser_download_url")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} 资产缺少 browser_download_url"))?
        .to_string();

    // 版本号：pin 驱动解析（无 latest/tag/version 选项）时 pin 四键的 version 即锁定权威
    // （tag 形态不规则的条目如 openssh 运行时版本 10.0p2 不从 tag 10.0.0.0p2-Preview 推导）；
    // 新解析走 version_pattern 从资产名提取（如 python-build-standalone 的 tag 是日期），
    // 否则剥 tag_prefix 得 version（对齐 pwsh Resolve-ToolVersion）
    let version = if !opts.latest && opts.tag.is_none() && opts.version.is_none() {
        tool.pin_version()
            .map(str::to_string)
            .unwrap_or_else(|| strip_tag_prefix(&tag_name, prefix))
    } else {
        match &tool.version_pattern {
            Some(vp) => extract_version_by_pattern(vp, &asset_name)
                .unwrap_or_else(|| strip_tag_prefix(&tag_name, prefix)),
            None => strip_tag_prefix(&tag_name, prefix),
        }
    };

    Ok(Resolution {
        tool: name.to_string(),
        tag: tag_name,
        version,
        asset_name,
        asset_size,
        asset_url,
        shasums_url: None,
    })
}

/// tag_prefix 剥离（大小写不敏感，对齐 pwsh 的 OrdinalIgnoreCase）。
pub fn strip_tag_prefix(tag: &str, prefix: &str) -> String {
    if tag.len() >= prefix.len()
        && tag.is_char_boundary(prefix.len())
        && tag[..prefix.len()].eq_ignore_ascii_case(prefix)
    {
        tag[prefix.len()..].to_string()
    } else {
        tag.to_string()
    }
}

/// 用 version_pattern 正则从资产名提取版本（取第 1 捕获组）。
pub fn extract_version_by_pattern(pattern: &str, asset_name: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    let caps = re.captures(asset_name)?;
    caps.get(1).map(|m| m.as_str().to_string())
}

/// 从版本名列表里按语义版本取最大（对齐 pwsh Get-HashiCorpIndex 的
/// Sort-Object { [System.Version]($_ -replace '[^0-9.].*$', '') }：剥尾后按数值段比较）。
pub fn pick_max_semver(names: &[String]) -> Option<String> {
    names
        .iter()
        .filter_map(|n| parse_semver(n).map(|v| (n.clone(), v)))
        .max_by(|a, b| semver_cmp(&a.1, &b.1))
        .map(|(n, _)| n)
}

/// 剥掉首个非 [0-9.] 字符起的尾巴，再按 . 拆数值段；解析不出则视为非法版本。
fn parse_semver(s: &str) -> Option<Vec<u64>> {
    let stripped: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let stripped = stripped.trim_matches('.');
    if stripped.is_empty() {
        return None;
    }
    stripped.split('.').map(|p| p.parse::<u64>().ok()).collect()
}

/// 数值段比较，短版本按 0 补齐（1.2 == 1.2.0）。
fn semver_cmp(a: &[u64], b: &[u64]) -> Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

/// GET JSON：3 次指数退避（第 n 次失败后等 2^n 秒）；github 地址按需回退 gh api。
fn get_json_retried(url: &str, allow_gh_fallback: bool) -> Result<Value, String> {
    let mut last_err = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        let result = http_get_json(url).or_else(|e| {
            if allow_gh_fallback && should_fallback_gh(&e) {
                eprintln!("[INFO] api.github.com 直连失败，改用 gh api（认证通道）");
                gh_api(url)
            } else {
                Err(e)
            }
        });
        match result {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = e;
                if attempt < MAX_ATTEMPTS {
                    let wait = 2u64.pow(attempt);
                    eprintln!(
                        "[WARN] 查询失败，{wait}s 后重试（{attempt}/{MAX_ATTEMPTS}）: {last_err}"
                    );
                    std::thread::sleep(Duration::from_secs(wait));
                }
            }
        }
    }
    Err(format!("查询失败（{MAX_ATTEMPTS} 次）: {url}\n{last_err}"))
}

/// 单次 GET JSON：30s 超时 + User-Agent 头；api.github.com 带 GH_TOKEN/GITHUB_TOKEN 认证
/// （共享出口 IP 匿名限流 60 次/时是 CI 真网测试挂点，认证后 5000+；对齐 selfupdate 注入模式）。
fn http_get_json(url: &str) -> Result<Value, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout(Duration::from_secs(30))
        .build();
    let mut req = agent
        .get(url)
        .set("User-Agent", UA)
        .set("Accept", "application/vnd.github+json");
    if url.starts_with("https://api.github.com/") {
        if let Ok(tok) = std::env::var("GH_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN")) {
            let auth = format!("Bearer {tok}");
            req = req.set("Authorization", auth.as_str());
        }
    }
    let resp = req
        .call()
        .map_err(|e| format!("HTTP 请求失败: {url}: {e}"))?;
    // ureq 未开 json feature：读文本后走 serde_json 解析
    let body = resp
        .into_string()
        .map_err(|e| format!("读取响应体失败: {url}: {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败: {url}: {e}"))
}

/// 是否需要 gh api 兜底（对齐 pwsh Invoke-GitHubApi 的失败特征串）。
fn should_fallback_gh(err: &str) -> bool {
    let lower = err.to_lowercase();
    [
        "403",
        "rate limit",
        "502",
        "503",
        "504",
        "gateway",
        "ssl",
        "tls",
        "connect",
        "timed out",
        "timeout",
    ]
    .iter()
    .any(|k| lower.contains(k))
}

/// gh api 兜底：which 找 gh，请求路径剥掉 https://api.github.com 前缀。
fn gh_api(url: &str) -> Result<Value, String> {
    let path = url
        .strip_prefix("https://api.github.com")
        .ok_or_else(|| format!("非 api.github.com 地址，无法走 gh api: {url}"))?;
    let gh = which::which("gh").map_err(|_| "gh 不可用，无法回退 gh api".to_string())?;
    let out = Command::new(gh)
        .arg("api")
        .arg(path)
        .output()
        .map_err(|e| format!("gh api 执行失败: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gh api 失败（{:?}）: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("gh api 输出 JSON 解析失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_取最大_数值段比较而非字典序() {
        // 期望值来源：System.Version 数值比较语义（1.10 > 1.9，字典序则相反）
        let names = vec![
            "1.9.4".to_string(),
            "1.10.0".to_string(),
            "1.2.3".to_string(),
        ];
        assert_eq!(pick_max_semver(&names).as_deref(), Some("1.10.0"));
    }

    #[test]
    fn semver_剥后缀_短版本按0补齐() {
        let names = vec!["1.2.3-beta".to_string(), "1.2".to_string()];
        // 1.2 == 1.2.0 < 1.2.3（beta 尾巴剥掉后是 1.2.3）
        assert_eq!(pick_max_semver(&names).as_deref(), Some("1.2.3-beta"));
    }

    #[test]
    fn semver_空列表与全非法返回none() {
        assert_eq!(pick_max_semver(&[]), None);
        let names = vec!["beta".to_string(), "x.y.z".to_string()];
        assert_eq!(pick_max_semver(&names), None);
    }

    #[test]
    fn tag_prefix_剥离_大小写不敏感() {
        assert_eq!(strip_tag_prefix("v1.2.3", "v"), "1.2.3");
        assert_eq!(strip_tag_prefix("V1.2.3", "v"), "1.2.3");
        assert_eq!(strip_tag_prefix("rust-v0.149.1", "rust-v"), "0.149.1");
        assert_eq!(strip_tag_prefix("release-2.0", "release-"), "2.0");
    }

    #[test]
    fn tag_prefix_不匹配时原样返回() {
        assert_eq!(strip_tag_prefix("20250818", "v"), "20250818");
        assert_eq!(strip_tag_prefix("1.2.3", "v"), "1.2.3");
    }

    #[test]
    fn version_pattern_从资产名提取版本() {
        // python-build-standalone：tag 是日期，版本在资产名里
        let v = extract_version_by_pattern(
            "cpython-([0-9.]+)\\+",
            "cpython-3.12.11+20250818-x86_64-pc-windows-msvc-install_only.tar.gz",
        );
        assert_eq!(v.as_deref(), Some("3.12.11"));
    }

    #[test]
    fn version_pattern_不匹配返回none() {
        assert_eq!(
            extract_version_by_pattern("cpython-([0-9.]+)\\+", "other.zip"),
            None
        );
        assert_eq!(extract_version_by_pattern("(非法正则", "x"), None);
    }
}

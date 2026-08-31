//! checksum：sha256 校验源解析，语义对齐 helpers.ps1 的 Get-OfficialSha256。
//! 优先级（R001 四、2）：pin 的 sha256 > 官方校验源三型——
//! 1) cdn_index_url 的 HashiCorp SHA256SUMS 清单（shasums_url 来自解析结果）；
//! 2) sums_asset 统一校验清单（{version}/{tag} 占位，按 sums_pattern 取行）；
//! 3) asset_sha_suffix 逐资产后缀文件（如 <asset>.sha256）。
//! 校验清单类资产每次强制重下（删旧再下），结果统一大写。

use std::fs;
use std::path::Path;

use regex::Regex;

use crate::catalog::Tool;
use crate::download;
use crate::resolve::Resolution;

const HEX64: &str = "[0-9a-fA-F]{64}";

/// 本次下载应遵循的 sha256 基准：pin 的 sha256 优先，否则查官方校验源，都没有则 None。
pub fn expected_sha256(
    tool: &Tool,
    res: &Resolution,
    env_root: &Path,
) -> Result<Option<String>, String> {
    if let Some(sha) = &tool.sha256 {
        if !sha.trim().is_empty() {
            return Ok(Some(sha.to_uppercase()));
        }
    }
    official_sha256(tool, res, env_root)
}

/// 官方校验源三型（对齐 Get-OfficialSha256 的分支顺序）。
pub fn official_sha256(
    tool: &Tool,
    res: &Resolution,
    env_root: &Path,
) -> Result<Option<String>, String> {
    // 1) HashiCorp 来源：SHA256SUMS 清单，行内按资产名匹配取 64-hex
    if tool.cdn_index_url.is_some() {
        if let Some(sums_url) = &res.shasums_url {
            let sums_name = sums_url
                .rsplit('/')
                .next()
                .ok_or_else(|| format!("shasums_url 无法取文件名: {sums_url}"))?;
            let path = download::download_fresh(env_root, sums_name, sums_url)?;
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("读取校验清单失败: {}: {e}", path.display()))?;
            let needle = Regex::new(&regex::escape(&res.asset_name))
                .map_err(|e| format!("资产名正则构造失败: {e}"))?;
            let line = text
                .lines()
                .find(|l| needle.is_match(l))
                .ok_or_else(|| format!("{} 官方 SHA256SUMS 未找到匹配资产: {}", res.tool, res.asset_name))?;
            return extract_hex64(line)
                .map(Some)
                .ok_or_else(|| format!("{} 官方 SHA256SUMS 行内无 64-hex: {line}", res.tool));
        }
    }

    // 2) 统一校验清单资产：sums_asset 名带 {version}/{tag} 占位，按 sums_pattern 取行
    if let Some(sums_asset) = &tool.sums_asset {
        let sums_name = sums_asset
            .replace("{version}", &res.version)
            .replace("{tag}", &res.tag);
        let repo = tool
            .repo
            .as_deref()
            .ok_or_else(|| format!("{} 使用 sums_asset 但缺少 repo 字段", res.tool))?;
        let sums_url = format!("https://github.com/{repo}/releases/download/{}/{sums_name}", res.tag);
        let path = download::download_fresh(env_root, &sums_name, &sums_url)?;
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("读取校验清单失败: {}: {e}", path.display()))?;
        let pattern = tool
            .sums_pattern
            .as_deref()
            .ok_or_else(|| format!("{} 使用 sums_asset 但缺少 sums_pattern", res.tool))?;
        let re = Regex::new(pattern).map_err(|e| format!("{} sums_pattern 非法: {e}", res.tool))?;
        let line = text
            .lines()
            .find(|l| re.is_match(l))
            .ok_or_else(|| format!("{} 官方校验清单中未找到匹配资产: {pattern}", res.tool))?;
        return extract_hex64(line)
            .map(Some)
            .ok_or_else(|| format!("{} 官方校验清单行内无 64-hex: {line}", res.tool));
    }

    // 3) 逐资产后缀文件：<asset>.sha256，全文取第一个 64-hex
    if let Some(suffix) = &tool.asset_sha_suffix {
        let sha_name = format!("{}{suffix}", res.asset_name);
        let sha_url = format!("{}{suffix}", res.asset_url);
        let path = download::download_fresh(env_root, &sha_name, &sha_url)?;
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("读取校验文件失败: {}: {e}", path.display()))?;
        return extract_hex64(text.trim())
            .map(Some)
            .ok_or_else(|| format!("{} 官方 .sha256 解析失败: {sha_url}", res.tool));
    }

    Ok(None)
}

/// 从一行文本提取第一个 64 位 hex 并大写（对齐 pwsh 的 ([0-9a-fA-F]{64}) + ToUpperInvariant）。
pub fn extract_hex64(text: &str) -> Option<String> {
    let re = Regex::new(HEX64).ok()?;
    let m = re.find(text)?;
    Some(m.as_str().to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex64_提取_统一大写() {
        let line = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  abc.zip";
        assert_eq!(
            extract_hex64(line).as_deref(),
            Some("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD")
        );
    }

    #[test]
    fn hex64_不足64位返回none() {
        assert_eq!(extract_hex64("abc123"), None);
        assert_eq!(extract_hex64(""), None);
    }

    #[test]
    fn 优先级_pin的sha256高于官方源() -> Result<(), String> {
        // pin 有 sha256 时不应触发任何官方源下载（此处 vault 无网也应直接返回）
        let tool = Tool {
            sha256: Some("a4a9a398".to_string()),
            ..Tool::default()
        };
        let res = Resolution {
            tool: "demo".to_string(),
            tag: "v1".to_string(),
            version: "1".to_string(),
            asset_name: "demo.zip".to_string(),
            asset_size: 0,
            asset_url: "https://example.invalid/demo.zip".to_string(),
            shasums_url: None,
        };
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let got = expected_sha256(&tool, &res, dir.path())?;
        assert_eq!(got.as_deref(), Some("A4A9A398"), "pin sha 应直接返回并大写");
        Ok(())
    }

    #[test]
    fn 优先级_无pin且无官方源_返回none() -> Result<(), String> {
        let tool = Tool::default();
        let res = Resolution {
            tool: "demo".to_string(),
            tag: "v1".to_string(),
            version: "1".to_string(),
            asset_name: "demo.zip".to_string(),
            asset_size: 0,
            asset_url: "https://example.invalid/demo.zip".to_string(),
            shasums_url: None,
        };
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        assert_eq!(expected_sha256(&tool, &res, dir.path())?, None);
        Ok(())
    }
}

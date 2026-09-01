//! catalog：tools.toml 读写与路径解析。
//!
//! 数据契约见 `docs/references/R001`：读用 serde（字段同 R001），
//! 写（pin 回写）用 toml_edit DocumentMut 直接改文档树，保住字段顺序与注释。
//! 路径解析优先级与原 ohmyenv.ps1 / helpers.ps1 对齐：
//! - EnvRoot：`--env-root` 参数 > `OHMYENV_ROOT` 环境变量 > 存在 D:\ 则 D:\ohmyenv 否则 C:\ohmyenv
//! - catalog：`OME_CATALOG` 环境变量 > exe 上级的 catalog\tools.toml > cwd\catalog\tools.toml

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml_edit::{DocumentMut, Formatted, Item, Value};

use crate::resolve::Resolution;

/// 工具条目：字段与 R001 一一对应；可选字段为空时整行省略，故全部 Option。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Tool {
    // —— 静态元数据（ome 不回写）——
    pub category: Option<String>,
    pub deploy: Option<String>,
    pub dir: Option<String>,
    pub bin: Option<String>,
    pub exe: Option<String>,
    pub extract: Option<String>,
    pub repo: Option<String>,
    pub tag_prefix: Option<String>,
    pub asset_pattern: Option<String>,
    pub version_pattern: Option<String>,
    pub cdn_url: Option<String>,
    pub cdn_index_url: Option<String>,
    pub cdn_asset_pattern: Option<String>,
    pub cdn_version_url: Option<String>,
    pub linux_cdn_url: Option<String>,
    pub sums_asset: Option<String>,
    pub sums_pattern: Option<String>,
    pub asset_sha_suffix: Option<String>,
    pub bootstrap_asset: Option<String>,
    // —— Linux / macOS 平台专属字段（缺失时回退到通用字段）——
    pub linux_repo: Option<String>,
    pub linux_asset_pattern: Option<String>,
    pub linux_dir: Option<String>,
    pub linux_bin: Option<String>,
    pub linux_exe: Option<String>,
    pub linux_extract: Option<String>,
    pub linux_sums_pattern: Option<String>,
    pub linux_asset_sha_suffix: Option<String>,
    pub linux_bootstrap_asset: Option<String>,
    // —— pin 字段（ome pin/update 回写）——
    pub tag: Option<String>,
    pub version: Option<String>,
    pub asset: Option<String>,
    pub sha256: Option<String>,
}

impl Tool {
    /// 当前平台适用的 repo（Linux 优先 `linux_repo`，回退 `repo`）。
    pub fn repo(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_repo.as_deref() {
            return Some(v);
        }
        self.repo.as_deref()
    }

    /// 当前平台适用的 asset_pattern（Linux 优先 `linux_asset_pattern`，回退 `asset_pattern`）。
    pub fn asset_pattern(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_asset_pattern.as_deref() {
            return Some(v);
        }
        self.asset_pattern.as_deref()
    }

    /// 当前平台适用的安装目录字段（Linux 优先 `linux_dir`，回退 `dir`）。
    pub fn dir(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_dir.as_deref() {
            return Some(v);
        }
        self.dir.as_deref()
    }

    /// 当前平台适用的 PATH 目录字段（Linux 优先 `linux_bin`，回退 `bin`）。
    pub fn bin(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_bin.as_deref() {
            return Some(v);
        }
        self.bin.as_deref()
    }

    /// 当前平台适用的 exe 字段（Linux 优先 `linux_exe`，回退 `exe`）。
    pub fn exe(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_exe.as_deref() {
            return Some(v);
        }
        self.exe.as_deref()
    }

    /// 当前平台适用的 extract 字段（Linux 优先 `linux_extract`，回退 `extract`）。
    pub fn extract(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_extract.as_deref() {
            return Some(v);
        }
        self.extract.as_deref()
    }

    /// 当前平台适用的 sums_pattern（Linux 优先 `linux_sums_pattern`，回退 `sums_pattern`）。
    pub fn sums_pattern(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_sums_pattern.as_deref() {
            return Some(v);
        }
        self.sums_pattern.as_deref()
    }

    /// 当前平台适用的 asset_sha_suffix（Linux 优先 `linux_asset_sha_suffix`，回退 `asset_sha_suffix`）。
    pub fn asset_sha_suffix(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_asset_sha_suffix.as_deref() {
            return Some(v);
        }
        self.asset_sha_suffix.as_deref()
    }

    /// 当前平台适用的 bootstrap_asset（Linux 优先 `linux_bootstrap_asset`，回退 `bootstrap_asset`）。
    pub fn bootstrap_asset(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_bootstrap_asset.as_deref() {
            return Some(v);
        }
        self.bootstrap_asset.as_deref()
    }

    /// 当前平台适用的 cdn_url（Linux 优先 `linux_cdn_url`，回退 `cdn_url`）。
    pub fn cdn_url(&self) -> Option<&str> {
        #[cfg(not(windows))]
        if let Some(v) = self.linux_cdn_url.as_deref() {
            return Some(v);
        }
        self.cdn_url.as_deref()
    }
}

/// 已加载的 catalog：order 保工具书写顺序（即安装/更新顺序），tools 按键查值。
pub struct Catalog {
    pub path: PathBuf,
    pub order: Vec<String>,
    pub tools: HashMap<String, Tool>,
}

#[derive(Deserialize)]
struct RawCatalog {
    tools: HashMap<String, Tool>,
}

impl Catalog {
    /// 从磁盘加载 catalog。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("读取 catalog 失败: {}: {e}", path.display()))?;
        Self::parse(&text, path.to_path_buf())
    }

    /// 解析 catalog 文本（顺序取文档树、取值走 serde）。
    pub fn parse(text: &str, path: PathBuf) -> Result<Self, String> {
        // toml 0.8 风格 serde 序列化会丢字段顺序，故顺序另从 DocumentMut 取
        let doc: DocumentMut = text
            .parse()
            .map_err(|e| format!("catalog TOML 语法错误: {}: {e}", path.display()))?;
        let mut order = Vec::new();
        let tools_item = doc
            .get("tools")
            .ok_or_else(|| format!("catalog 缺少 [tools] 节: {}", path.display()))?;
        let tools_table = tools_item
            .as_table_like()
            .ok_or_else(|| format!("catalog [tools] 不是表: {}", path.display()))?;
        for (name, _) in tools_table.iter() {
            order.push(name.to_string());
        }
        let raw: RawCatalog = toml_edit::de::from_str(text)
            .map_err(|e| format!("catalog 字段解析失败: {}: {e}", path.display()))?;
        Ok(Catalog {
            path,
            order,
            tools: raw.tools,
        })
    }

    /// 子命令的工具选择：`all` 展开为全部（保序），单工具校验存在性。
    pub fn select(&self, name: &str) -> Result<Vec<String>, String> {
        if name == "all" {
            return Ok(self.order.clone());
        }
        if !self.tools.contains_key(name) {
            return Err(format!(
                "未知工具: {name}（catalog: {}）",
                self.path.display()
            ));
        }
        Ok(vec![name.to_string()])
    }

    /// 按名取工具条目。
    pub fn tool(&self, name: &str) -> Result<&Tool, String> {
        self.tools
            .get(name)
            .ok_or_else(|| format!("catalog 中无工具: {name}"))
    }
}

/// EnvRoot 解析：显式参数 > OHMYENV_ROOT > 平台默认。
/// 对齐 helpers.ps1 Get-DefaultEnvRoot：参数与环境变量都会裁掉尾部斜杠。
pub fn resolve_env_root(cli: Option<&str>) -> Result<PathBuf, String> {
    if let Some(v) = cli {
        let v = v.trim().trim_end_matches(['/', '\\']);
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    if let Ok(v) = std::env::var("OHMYENV_ROOT") {
        let v = v.trim().trim_end_matches(['/', '\\']);
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    Ok(crate::platform::default_env_root())
}

/// catalog 路径解析：OME_CATALOG > exe 上级的 catalog\tools.toml > cwd\catalog\tools.toml。
pub fn resolve_catalog_path() -> Result<PathBuf, String> {
    if let Ok(v) = std::env::var("OME_CATALOG") {
        let v = v.trim();
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // 自部署布局为 <ome>\bin\ome.exe，故 exe 上级（bin 的上一级）下找 catalog
            let cand = exe_dir.join("..").join("catalog").join("tools.toml");
            if cand.exists() {
                return Ok(cand);
            }
        }
    }
    let cand = PathBuf::from("catalog").join("tools.toml");
    if cand.exists() {
        return Ok(cand);
    }
    Err("未找到 catalog\\tools.toml（可设 OME_CATALOG 指定路径）".to_string())
}

/// pin 回写：用 toml_edit 直接改文档树，只动 tag/version/asset/sha256 四个键，
/// 保住字段顺序、缩进与注释。版本变化时删除 sha256 行（等 install 回填），同版本 re-pin 保留。
/// 对齐 helpers.ps1 Set-ToolPin。
pub fn write_pin(path: &Path, tool: &str, res: &Resolution) -> Result<bool, String> {
    let mut version_changed = false;
    update_tool_table(path, tool, |table| {
        let old_version = table
            .get("version")
            .and_then(|i| i.as_str())
            .map(str::to_string);
        version_changed = old_version.as_deref() != Some(res.version.as_str());
        set_string(table, "tag", &res.tag);
        set_string(table, "version", &res.version);
        set_string(table, "asset", &res.asset_name);
        if version_changed {
            // 版本真变才清 sha（同版本 re-pin 保留旧 sha）
            table.remove("sha256");
        }
    })?;
    Ok(version_changed)
}

/// sha256 回填：只写 sha256 一个键（install 成功后回填空 sha；统一大写）。
pub fn write_sha256(path: &Path, tool: &str, sha: &str) -> Result<(), String> {
    update_tool_table(path, tool, |table| {
        set_string(table, "sha256", &sha.to_uppercase())
    })
}

/// 共享回写助手：解析 DocumentMut、定位 [tools.<名>] 表交给 edit 闭包、写回时保持原行尾风格。
fn update_tool_table(
    path: &Path,
    tool: &str,
    edit: impl FnOnce(&mut dyn toml_edit::TableLike),
) -> Result<(), String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("读取 catalog 失败: {}: {e}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e| format!("catalog TOML 语法错误: {}: {e}", path.display()))?;

    let tools = doc
        .get_mut("tools")
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| format!("catalog 缺少 [tools] 节: {}", path.display()))?;
    let table = tools
        .get_mut(tool)
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| format!("catalog 中无工具节 [tools.{tool}]"))?;
    edit(table);

    // 保持原文件行尾风格：toml_edit 序列化统一出 LF，原文件为 CRLF 时整体换回，避免全文件换行噪音
    let mut out = doc.to_string();
    if text.contains("\r\n") {
        out = out.replace("\r\n", "\n").replace('\n', "\r\n");
    }
    fs::write(path, out).map_err(|e| format!("写回 catalog 失败: {}: {e}", path.display()))?;
    Ok(())
}

/// 设置字符串键：已存在则只换值本体、保留 decor（缩进与行内注释）；不存在则追加到节尾。
fn set_string(table: &mut dyn toml_edit::TableLike, key: &str, v: &str) {
    match table.get_mut(key) {
        Some(Item::Value(Value::String(s))) => {
            let decor = s.decor().clone();
            let mut f = Formatted::new(v.to_string());
            *f.decor_mut() = decor;
            *s = f;
        }
        Some(item) => {
            *item = toml_edit::value(v);
        }
        None => {
            table.insert(key, toml_edit::value(v));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 夹具原文即期望值来源（R001 字段契约的实例化），避免重言式断言。
    const FIXTURE: &str = include_str!("../tests/fixtures/tools.toml");

    fn fixture_catalog() -> Catalog {
        Catalog::parse(FIXTURE, PathBuf::from("tests/fixtures/tools.toml")).expect("夹具应能解析")
    }

    #[test]
    fn catalog_夹具解析_三工具保序且字段齐全() {
        let cat = fixture_catalog();
        assert_eq!(cat.order, vec!["age", "vault", "python"]);

        let age = cat.tool("age").expect("age 应存在");
        assert_eq!(age.repo.as_deref(), Some("FiloSottile/age"));
        assert_eq!(age.tag_prefix.as_deref(), Some("v"));
        assert_eq!(age.tag.as_deref(), Some("v1.3.1"));
        assert_eq!(age.version.as_deref(), Some("1.3.1"));
        assert_eq!(age.asset.as_deref(), Some("age-v1.3.1-windows-amd64.zip"));
        assert_eq!(
            age.sha256.as_deref(),
            Some("C56E8CE22F7E80CB85AD946CC82D198767B056366201D3E1A2B93D865BE38154")
        );

        let vault = cat.tool("vault").expect("vault 应存在");
        assert_eq!(
            vault.cdn_index_url.as_deref(),
            Some("https://releases.hashicorp.com/vault/index.json")
        );
        assert!(vault.repo.is_none(), "纯 cdn 工具无 repo 字段");

        let python = cat.tool("python").expect("python 应存在");
        assert_eq!(
            python.version_pattern.as_deref(),
            Some("cpython-([0-9.]+)\\+")
        );
    }

    #[test]
    fn select_all_展开为全部工具_按书写顺序() {
        let cat = fixture_catalog();
        assert_eq!(cat.select("all").expect("all 应可选"), cat.order);
    }

    #[test]
    fn dies_select_未知工具() {
        let cat = fixture_catalog();
        let err = cat.select("nonexistent").expect_err("未知工具应报错");
        assert!(err.contains("未知工具"), "错误应含提示: {err}");
    }

    #[test]
    fn env_root_显式参数优先_并裁尾斜杠() {
        let root = resolve_env_root(Some(r"E:\env\")).expect("显式参数应生效");
        assert_eq!(root, PathBuf::from(r"E:\env"));
    }

    /// 回写后重读文档树：字段顺序应与原文件一致（sha256 被版本变更删除除外）。
    fn key_order(text: &str, tool: &str) -> Vec<String> {
        let doc: DocumentMut = text.parse().expect("TOML 应能解析");
        let table = doc["tools"][tool].as_table_like().expect("工具节应为表");
        table.iter().map(|(k, _)| k.to_string()).collect()
    }

    #[test]
    fn pin_回写保序保注释_版本变更清sha() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("tools.toml");
        fs::write(&path, FIXTURE).map_err(|e| e.to_string())?;

        let res = Resolution {
            tool: "age".to_string(),
            tag: "v1.4.0".to_string(),
            version: "1.4.0".to_string(),
            asset_name: "age-v1.4.0-windows-amd64.zip".to_string(),
            asset_size: 0,
            asset_url: "https://example.invalid/age.zip".to_string(),
            shasums_url: None,
        };
        let changed = write_pin(&path, "age", &res)?;
        assert!(changed, "1.3.1 到 1.4.0 应判定为版本变更");

        let out = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // 注释（文件头与 sha256 上方的行注释）应保留
        assert!(out.contains("# 测试夹具"), "文件头注释应保留");
        assert!(out.contains("# age    = GitHub release"), "节上注释应保留");
        assert!(!out.contains("C56E8CE2"), "版本变更后旧 sha256 应被删除");
        assert!(out.contains("version = \"1.4.0\""), "version 应被更新");
        assert!(out.contains("tag = \"v1.4.0\""), "tag 应被更新");

        // 字段顺序：sha256 被删除，其余键顺序不变
        let before: Vec<String> = key_order(FIXTURE, "age")
            .into_iter()
            .filter(|k| k != "sha256")
            .collect();
        assert_eq!(key_order(&out, "age"), before, "回写不应打乱字段顺序");
        // 其它工具节完全不动
        assert!(out.contains("version = \"3.12.11\""), "python 节不应受影响");
        Ok(())
    }

    #[test]
    fn pin_同版本repin_保留sha256() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("tools.toml");
        fs::write(&path, FIXTURE).map_err(|e| e.to_string())?;

        let res = Resolution {
            tool: "age".to_string(),
            tag: "v1.3.1".to_string(),
            version: "1.3.1".to_string(),
            asset_name: "age-v1.3.1-windows-amd64.zip".to_string(),
            asset_size: 0,
            asset_url: "https://example.invalid/age.zip".to_string(),
            shasums_url: None,
        };
        let changed = write_pin(&path, "age", &res)?;
        assert!(!changed, "同版本 re-pin 不算变更");

        let out = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        assert!(out.contains("C56E8CE2"), "同版本 re-pin 应保留 sha256");
        Ok(())
    }

    #[test]
    fn pin_回写保持原文件crlf行尾() -> Result<(), String> {
        let src = FIXTURE.replace('\n', "\r\n");
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("tools.toml");
        fs::write(&path, &src).map_err(|e| e.to_string())?;

        let res = Resolution {
            tool: "age".to_string(),
            tag: "v1.4.0".to_string(),
            version: "1.4.0".to_string(),
            asset_name: "age-v1.4.0-windows-amd64.zip".to_string(),
            asset_size: 0,
            asset_url: "https://example.invalid/age.zip".to_string(),
            shasums_url: None,
        };
        write_pin(&path, "age", &res)?;

        let out = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let bare_lf = out.replace("\r\n", "").contains('\n');
        assert!(!bare_lf, "CRLF 文件回写后不应混入裸 LF");
        assert!(out.contains("version = \"1.4.0\"\r\n"), "内容应正常更新");
        Ok(())
    }

    #[test]
    fn pin_未pin工具_追加pin字段到节尾() -> Result<(), String> {
        // 未 pin 的工具没有 tag/version/asset 行（R001：可选字段为空整行省略）
        let src = "[tools.demo]\nrepo = \"owner/demo\"\ntag_prefix = \"v\"\n";
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let path = dir.path().join("tools.toml");
        fs::write(&path, src).map_err(|e| e.to_string())?;

        let res = Resolution {
            tool: "demo".to_string(),
            tag: "v2.0.0".to_string(),
            version: "2.0.0".to_string(),
            asset_name: "demo.zip".to_string(),
            asset_size: 0,
            asset_url: "https://example.invalid/demo.zip".to_string(),
            shasums_url: None,
        };
        write_pin(&path, "demo", &res)?;

        let out = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        assert_eq!(
            key_order(&out, "demo"),
            vec!["repo", "tag_prefix", "tag", "version", "asset"],
            "pin 字段应追加在静态元数据之后"
        );
        Ok(())
    }
}

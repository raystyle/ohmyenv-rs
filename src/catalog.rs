//! catalog：tools.toml 读写与路径解析。
//!
//! 数据契约见 `docs/references/R001`：读用 serde（字段同 R001），
//! 写（pin 回写）用 toml_edit DocumentMut 直接改文档树，保住字段顺序与注释。
//! 路径解析优先级与原 ohmyenv.ps1 / helpers.ps1 对齐：
//! - EnvRoot：`--env-root` 参数 > `OHMYENV_ROOT` 环境变量 > 存在 D:\ 则 D:\ohmyenv 否则 C:\ohmyenv
//! - catalog：`OME_CATALOG` 环境变量 > exe 上级的 catalog\tools.toml > cwd\catalog\tools.toml
//!   > 用户数据目录 catalog\tools.toml（自部署布局）

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
    pub linux_cdn_asset_pattern: Option<String>,
    pub linux_extra_bins: Option<String>,
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
    // —— macOS 专属字段（缺失时回退 linux_*，再回退通用；仅 darwin 构建生效）——
    pub mac_repo: Option<String>,
    pub mac_asset_pattern: Option<String>,
    pub mac_dir: Option<String>,
    pub mac_bin: Option<String>,
    pub mac_exe: Option<String>,
    pub mac_extract: Option<String>,
    pub mac_sums_pattern: Option<String>,
    pub mac_asset_sha_suffix: Option<String>,
    pub mac_bootstrap_asset: Option<String>,
    pub mac_cdn_url: Option<String>,
    pub mac_cdn_asset_pattern: Option<String>,
    pub mac_extra_bins: Option<String>,
    // —— pin 字段（ome pin/update 回写；按平台分列，通用四键即 Windows pin）——
    pub tag: Option<String>,
    pub version: Option<String>,
    pub asset: Option<String>,
    pub sha256: Option<String>,
    pub linux_tag: Option<String>,
    pub linux_version: Option<String>,
    pub linux_asset: Option<String>,
    pub linux_sha256: Option<String>,
    pub mac_tag: Option<String>,
    pub mac_version: Option<String>,
    pub mac_asset: Option<String>,
    pub mac_sha256: Option<String>,
    /// 版本锁定开关（静态元数据，跨平台生效）：true 时 update/daily/pin/带版本选项的 install 全部跳过，
    /// 用于钉死特定版本（如 bun 1.3.14——最后一个完全用 Zig 编写核心的版本，2026-09-01 用户裁决）。
    pub hold: Option<bool>,
}

impl Tool {
    /// 是否版本锁定（hold = true）。
    pub fn is_held(&self) -> bool {
        self.hold.unwrap_or(false)
    }

    /// 当前平台适用的 repo（mac 依次回退 `mac_repo`/`linux_repo`/`repo`；Linux 依次 `linux_repo`/`repo`；Windows 取 `repo`）。
    pub fn repo(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_repo.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_repo.as_deref() {
            return Some(v);
        }
        self.repo.as_deref()
    }

    /// 当前平台适用的 asset_pattern（回退链同 `repo()`）。
    pub fn asset_pattern(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_asset_pattern.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_asset_pattern.as_deref() {
            return Some(v);
        }
        self.asset_pattern.as_deref()
    }

    /// 当前平台适用的安装目录字段（回退链同 `repo()`）。
    pub fn dir(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_dir.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_dir.as_deref() {
            return Some(v);
        }
        self.dir.as_deref()
    }

    /// 当前平台适用的 PATH 目录字段（回退链同 `repo()`）。
    pub fn bin(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_bin.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_bin.as_deref() {
            return Some(v);
        }
        self.bin.as_deref()
    }

    /// 当前平台适用的 exe 字段（回退链同 `repo()`）。
    pub fn exe(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_exe.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_exe.as_deref() {
            return Some(v);
        }
        self.exe.as_deref()
    }

    /// 平台专属 exe 字段原始值（`mac_exe`/`linux_exe`，无通用回退）。
    /// 有值时 exe 相对 install_dir；为 None 时 `exe()` 必来自通用字段（Windows 风格，自带 dir 段、相对 EnvRoot）。
    pub fn platform_exe(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_exe.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_exe.as_deref() {
            return Some(v);
        }
        None
    }

    /// 当前平台适用的 extract 字段（回退链同 `repo()`）。
    pub fn extract(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_extract.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_extract.as_deref() {
            return Some(v);
        }
        self.extract.as_deref()
    }

    /// 当前平台适用的 sums_pattern（回退链同 `repo()`）。
    pub fn sums_pattern(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_sums_pattern.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_sums_pattern.as_deref() {
            return Some(v);
        }
        self.sums_pattern.as_deref()
    }

    /// 当前平台适用的 asset_sha_suffix（回退链同 `repo()`）。
    pub fn asset_sha_suffix(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_asset_sha_suffix.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_asset_sha_suffix.as_deref() {
            return Some(v);
        }
        self.asset_sha_suffix.as_deref()
    }

    /// 当前平台适用的 bootstrap_asset（回退链同 `repo()`）。
    pub fn bootstrap_asset(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_bootstrap_asset.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_bootstrap_asset.as_deref() {
            return Some(v);
        }
        self.bootstrap_asset.as_deref()
    }

    /// 当前平台适用的 cdn_url（回退链同 `repo()`）。
    /// 直链模板（平台严格）：Windows 取通用 `cdn_url`；Linux 只取 `linux_cdn_url`；
    /// macOS 取 `mac_cdn_url` 回退 `linux_cdn_url`。非 Windows **不回退通用 cdn_url**——
    /// 通用 cdn_url 是 Windows 资产直链（go/zig/dotnet/oscdimg），回退会让 Linux 侧
    /// 解析到 Windows 包（2026-09-01 WSL install all 实证踩坑）。
    pub fn cdn_url(&self) -> Option<&str> {
        #[cfg(windows)]
        {
            self.cdn_url.as_deref()
        }
        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            self.linux_cdn_url.as_deref()
        }
        #[cfg(target_os = "macos")]
        {
            self.mac_cdn_url
                .as_deref()
                .or(self.linux_cdn_url.as_deref())
        }
    }

    /// 当前平台适用的 cdn_asset_pattern（回退链同 `repo()`；vault 类 cdn 工具各平台资产名不同）。
    pub fn cdn_asset_pattern(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        if let Some(v) = self.mac_cdn_asset_pattern.as_deref() {
            return Some(v);
        }
        #[cfg(not(windows))]
        if let Some(v) = self.linux_cdn_asset_pattern.as_deref() {
            return Some(v);
        }
        self.cdn_asset_pattern.as_deref()
    }

    /// 当前平台的补充二进制叶子名列表（空格/逗号分隔）：`*-bin` 提取与 zip/targz 的 chmod
    /// 除 exe 主二进制外还覆盖这些成员（如 age 的 age-keygen、ast-grep 的 sg）。
    pub fn extra_bins(&self) -> Vec<&str> {
        #[cfg(target_os = "macos")]
        let raw = self
            .mac_extra_bins
            .as_deref()
            .or(self.linux_extra_bins.as_deref());
        #[cfg(all(not(windows), not(target_os = "macos")))]
        let raw = self.linux_extra_bins.as_deref();
        #[cfg(windows)]
        let raw: Option<&str> = None;
        raw.map(|s| s.split([',', ' ']).filter(|x| !x.is_empty()).collect())
            .unwrap_or_default()
    }

    /// 当前平台锁定的 tag（pin 按平台分列、无跨平台回退：Windows 通用、Linux `linux_*`、mac `mac_*`）。
    pub fn pin_tag(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        return self.mac_tag.as_deref();
        #[cfg(all(not(windows), not(target_os = "macos")))]
        return self.linux_tag.as_deref();
        #[cfg(windows)]
        self.tag.as_deref()
    }

    /// 当前平台锁定的 version。
    pub fn pin_version(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        return self.mac_version.as_deref();
        #[cfg(all(not(windows), not(target_os = "macos")))]
        return self.linux_version.as_deref();
        #[cfg(windows)]
        self.version.as_deref()
    }

    /// 当前平台锁定的 asset。
    pub fn pin_asset(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        return self.mac_asset.as_deref();
        #[cfg(all(not(windows), not(target_os = "macos")))]
        return self.linux_asset.as_deref();
        #[cfg(windows)]
        self.asset.as_deref()
    }

    /// 当前平台锁定的 sha256。
    pub fn pin_sha256(&self) -> Option<&str> {
        #[cfg(target_os = "macos")]
        return self.mac_sha256.as_deref();
        #[cfg(all(not(windows), not(target_os = "macos")))]
        return self.linux_sha256.as_deref();
        #[cfg(windows)]
        self.sha256.as_deref()
    }
}

/// 当前平台 pin 字段的 TOML 键名（Windows 无前缀，Linux/mac 加平台前缀）。
pub fn pin_key(base: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("mac_{base}")
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        format!("linux_{base}")
    }
    #[cfg(windows)]
    {
        base.to_string()
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

/// catalog 路径解析：`OME_CATALOG` > exe 上级的 catalog\tools.toml（仓库与旧自部署布局）
/// > cwd\catalog\tools.toml > 用户数据目录 catalog\tools.toml（新自部署布局，self-deploy 时同步）。
pub fn resolve_catalog_path() -> Result<PathBuf, String> {
    if let Ok(v) = std::env::var("OME_CATALOG") {
        let v = v.trim();
        if !v.is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    catalog_candidates(&catalog_search_roots())
        .into_iter()
        .find(|p| p.exists())
        .ok_or_else(|| "未找到 catalog\\tools.toml（可设 OME_CATALOG 指定路径）".to_string())
}

/// 候选根目录（按优先级）：exe 上两级（仓库 target\ 布局与旧自部署 `<ome>\bin\ome.exe` 布局）、
/// cwd（仓库根运行）、用户数据目录（新自部署布局）。cwd 先于用户数据目录，
/// 保证仓库内开发/测试永远命中仓库 catalog，不被部署副本遮蔽。
fn catalog_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if let Some(up) = exe_dir.parent() {
                roots.push(up.to_path_buf());
            }
        }
    }
    roots.push(std::env::current_dir().unwrap_or_default());
    roots.push(crate::platform::metadata_dir());
    roots
}

/// 由根目录列表生成 catalog 候选路径（保持入参顺序）。
fn catalog_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .map(|r| r.join("catalog").join("tools.toml"))
        .collect()
}

/// pin 回写：用 toml_edit 直接改文档树，只动当前平台的 tag/version/asset/sha256 四个键
/// （Windows 通用、Linux `linux_*`、mac `mac_*`，见 R001），保住字段顺序、缩进与注释。
/// 版本变化时删除 sha256 行（等 install 回填），同版本 re-pin 保留。
pub fn write_pin(path: &Path, tool: &str, res: &Resolution) -> Result<bool, String> {
    let mut version_changed = false;
    let (k_tag, k_version, k_asset, k_sha) = (
        pin_key("tag"),
        pin_key("version"),
        pin_key("asset"),
        pin_key("sha256"),
    );
    update_tool_table(path, tool, |table| {
        let old_version = table
            .get(&k_version)
            .and_then(|i| i.as_str())
            .map(str::to_string);
        version_changed = old_version.as_deref() != Some(res.version.as_str());
        set_string(table, &k_tag, &res.tag);
        set_string(table, &k_version, &res.version);
        set_string(table, &k_asset, &res.asset_name);
        if version_changed {
            // 版本真变才清 sha（同版本 re-pin 保留旧 sha）
            table.remove(&k_sha);
        }
    })?;
    Ok(version_changed)
}

/// sha256 回填：只写当前平台的 sha256 一个键（install 成功后回填空 sha；统一大写）。
pub fn write_sha256(path: &Path, tool: &str, sha: &str) -> Result<(), String> {
    let k_sha = pin_key("sha256");
    update_tool_table(path, tool, |table| {
        set_string(table, &k_sha, &sha.to_uppercase())
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
    fn catalog_candidates_按根目录顺序生成() {
        let roots = vec![
            PathBuf::from("repo-root"),
            PathBuf::from("cwd"),
            PathBuf::from("user-data"),
        ];
        let got = catalog_candidates(&roots);
        assert_eq!(
            got,
            vec![
                PathBuf::from("repo-root")
                    .join("catalog")
                    .join("tools.toml"),
                PathBuf::from("cwd").join("catalog").join("tools.toml"),
                PathBuf::from("user-data")
                    .join("catalog")
                    .join("tools.toml"),
            ]
        );
    }

    #[test]
    fn hold字段_解析与判定() -> Result<(), String> {
        let toml = "[tools.x]\nversion = \"1.0\"\nhold = true\n\n[tools.y]\nversion = \"2.0\"\n";
        let cat = Catalog::parse(toml, PathBuf::from("synthetic.toml"))?;
        assert!(cat.tool("x")?.is_held(), "hold=true 应判定锁定");
        assert!(!cat.tool("y")?.is_held(), "缺省不锁定");
        Ok(())
    }

    #[test]
    fn catalog_解析_取第一个存在的候选() -> Result<(), String> {
        let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let miss = dir.path().join("miss");
        let hit = dir.path().join("hit");
        std::fs::create_dir_all(hit.join("catalog")).map_err(|e| e.to_string())?;
        std::fs::write(hit.join("catalog").join("tools.toml"), "").map_err(|e| e.to_string())?;
        let got = catalog_candidates(&[miss.clone(), hit.clone(), miss.clone()])
            .into_iter()
            .find(|p| p.exists());
        assert_eq!(got, Some(hit.join("catalog").join("tools.toml")));
        Ok(())
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
        // 平台分列 pin：三套并存、互不覆盖（R001 平台边界）
        assert_eq!(
            age.linux_asset.as_deref(),
            Some("age-v1.3.1-linux-amd64.tar.gz")
        );
        assert_eq!(
            age.mac_sha256.as_deref(),
            Some("01120EA2CBF0463D4C6BD767F99F3271BBED1CDC8A9AA718A76BA1FE4F01998B")
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

    /// 三字段同设时各平台取值（期望值即 R001 平台边界契约：mac 专属 > linux > 通用）。
    fn chain_tool() -> Tool {
        Tool {
            repo: Some("o/win".to_string()),
            linux_repo: Some("o/lin".to_string()),
            mac_repo: Some("o/mac".to_string()),
            ..Tool::default()
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn mac_字段族_优先于linux与通用_缺失逐级回退() {
        let t = chain_tool();
        assert_eq!(t.repo(), Some("o/mac"), "mac 三字段同设应取 mac 专属");
        let no_mac = Tool {
            mac_repo: None,
            ..chain_tool()
        };
        assert_eq!(no_mac.repo(), Some("o/lin"), "mac 缺失应回退 linux");
        assert_eq!(
            chain_tool().platform_exe(),
            None,
            "无专属 exe 字段时 platform_exe 为 None"
        );
    }

    #[test]
    #[cfg(all(not(windows), not(target_os = "macos")))]
    fn linux_字段族_优先于通用_mac字段不生效() {
        let t = chain_tool();
        assert_eq!(t.repo(), Some("o/lin"), "linux 下 linux 字段优先于通用");
        assert_eq!(t.platform_exe(), None);
    }

    #[test]
    #[cfg(windows)]
    fn windows_仅通用字段_平台专属族不生效() {
        let t = chain_tool();
        assert_eq!(t.repo(), Some("o/win"), "Windows 下只认通用字段");
        assert_eq!(t.platform_exe(), None);
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

    /// 夹具 age 的当前平台 pin sha 键与旧值（期望值取自夹具文件，三平台各取其一）。
    fn age_pin_sha_old() -> (&'static str, &'static str) {
        #[cfg(windows)]
        {
            (
                "sha256",
                "C56E8CE22F7E80CB85AD946CC82D198767B056366201D3E1A2B93D865BE38154",
            )
        }
        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            (
                "linux_sha256",
                "BDC69C09CBDD6CF8B1F333D372A1F58247B3A33146406333E30C0F26E8F51377",
            )
        }
        #[cfg(target_os = "macos")]
        {
            (
                "mac_sha256",
                "01120EA2CBF0463D4C6BD767F99F3271BBED1CDC8A9AA718A76BA1FE4F01998B",
            )
        }
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

        let (sha_key, sha_old) = age_pin_sha_old();
        let out = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // 注释（文件头与 sha256 上方的行注释）应保留
        assert!(out.contains("# 测试夹具"), "文件头注释应保留");
        assert!(out.contains("# age    = GitHub release"), "节上注释应保留");
        assert!(!out.contains(sha_old), "版本变更后本平台旧 sha256 应被删除");
        assert!(out.contains("version = \"1.4.0\""), "version 应被更新");
        assert!(out.contains("tag = \"v1.4.0\""), "tag 应被更新");

        // 字段顺序：本平台 sha256 被删除，其余键顺序不变
        let before: Vec<String> = key_order(FIXTURE, "age")
            .into_iter()
            .filter(|k| k != sha_key)
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

        let (_, sha_old) = age_pin_sha_old();
        let out = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        assert!(out.contains(sha_old), "同版本 re-pin 应保留本平台 sha256");
        Ok(())
    }

    #[test]
    fn pin_回写保持原文件crlf行尾() -> Result<(), String> {
        // 夹具检出行尾随平台 autocrlf 变化，先归一 LF 再转 CRLF（同 write_pin 内 314 行惯用法）
        let src = FIXTURE.replace("\r\n", "\n").replace('\n', "\r\n");
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
            vec![
                "repo".to_string(),
                "tag_prefix".to_string(),
                pin_key("tag"),
                pin_key("version"),
                pin_key("asset")
            ],
            "本平台 pin 字段应追加在静态元数据之后"
        );
        Ok(())
    }
}

# R001：catalog 数据模式——tools.toml 字段与 pin 语义

> tools.toml 是 ome 的工具名录唯一 pin 源，由 `.tools\import-catalog.ps1` 从 ohmypwsh `scripts\catalog.psd1`（Win 侧）与 `helpers.ps1` New-ToolDef 静态元数据生成；pin 字段由 `ome update` / `ome pin` 回写。本文件是该文件的字段契约。

## 一、文件级约定

1. 路径：`catalog\tools.toml`，UTF-8 无 BOM。
2. 工具顺序与 ohmypwsh `helpers.ps1` 的 ToolNames 一致（即安装/更新顺序）。
3. 每个工具一节 `[tools.<名>]`；字段分两组：静态元数据在前、pin 字段在后。
4. 可选字段为空时整行省略，不写空字符串。

## 二、字段表

### 静态元数据

> 转换器写入，ome 不回写。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `category` | string | 权威分类，来自 New-ToolDef：key / agent / project / base / extras |
| `deploy` | string | Deploy.win：envroot / installer / official |
| `dir` | string | EnvRoot 下安装目录（official 工具可省） |
| `bin` | string | 注册进用户 PATH 的目录，相对 EnvRoot（official 可省） |
| `exe` | string | 版本探测 exe 路径，相对 EnvRoot；official 可含 `%VAR%` 环境变量 |
| `extract` | string | 解压/安装方式：zip / targz / copy / gsudo / 7z-extra / 7zsfx / msi / rmux / single |
| `repo` | string | GitHub 仓库 `owner/name`（纯 cdn 工具可省） |
| `tag_prefix` | string | tag 前缀，剥离后得 version（如 `v`、`release-`） |
| `asset_pattern` | string | 资产名匹配正则（GitHub release 资产筛选） |
| `version_pattern` | string | 可选，从资产名提取版本的正则（python 用） |
| `cdn_url` | string | 可选，直链模板，含 `{version}` 占位（dotnet/oscdimg） |
| `cdn_index_url` | string | 可选，HashiCorp 式 index.json（vault） |
| `cdn_asset_pattern` | string | 可选，cdn 系资产名正则 |
| `cdn_version_url` | string | 可选，cdn 系版本查询地址（当前无在用工具，为后续 cdn 系工具预留） |
| `sums_asset` | string | 可选，官方统一校验清单资产名模板（`{version}`/`{tag}` 占位） |
| `sums_pattern` | string | 可选，校验清单内匹配本工具资产行的正则 |
| `asset_sha_suffix` | string | 可选，逐资产校验文件后缀（如 `.sha256`） |
| `bootstrap_asset` | string | 可选，安装自举资产（7z 的 7zr.exe） |

### pin 字段

> ome update/pin 回写，保序保注释用 toml_edit。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `tag` | string | 锁定的 release tag |
| `version` | string | 锁定的版本号 |
| `asset` | string | 锁定的资产文件名 |
| `sha256` | string | 锁定资产 sha256，大写；未回填可省略 |

## 三、示例

```toml
[tools.age]
category = "tool"
deploy = "envroot"
dir = "age"
bin = "age"
exe = 'age\age.exe'
extract = "zip"
repo = "FiloSottile/age"
tag_prefix = "v"
asset_pattern = '^age-v[0-9.]+-windows-amd64\.zip$'
tag = "v1.3.1"
version = "1.3.1"
asset = "age-v1.3.1-windows-amd64.zip"
sha256 = "C56E8CE22F7E80CB85AD946CC82D198767B056366201D3E1A2B93D865BE38154"
```

## 四、语义要点

1. **唯一 pin 源**：tag/version/asset/sha256 只存在于本文件；解析最新版只读不写，`pin`/`update` 才回写。
2. **sha256 校验优先级**：pin 的 sha256 > 官方校验源（sums_asset / asset_sha_suffix / cdn_index_url 自带 SUMS）；安装成功后可回填空 sha256。
3. **版本变更清 sha**：pin 到不同 version 时清掉旧 sha256，等 install 回填。
4. **平台边界**：本文件当前只含 Windows 侧；Linux/mac 策略为系统标准目录（不进 ohmyenv 目录），后续以独立小节扩展，不混进 win 字段。

# R001：catalog 数据模式——tools.toml 字段与 pin 语义

> tools.toml 是 ome 的工具名录唯一 pin 源与静态字段权威（M0 起数据主权在 ome，2026-09-01）：托管 26 节的 Win 侧静态字段源自 ohmypwsh `scripts\catalog.psd1` 与 `helpers.ps1` New-ToolDef 的历史生成，Pos 侧 linux/mac 数据已于 2026-09-01 一次性回流完毕，此后 psd1 冻结只读，`.tools\import-catalog.ps1` 只校验不再生（冲突报错、ome 增补放行）。pin 字段由 `ome update` / `ome pin` 按平台回写。本文件是该文件的字段契约。

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
| `extract` | string | 解压/安装方式：zip / targz / targz-bin / tarxz-bin / zip-bin / zip-dir（Windows 版本目录树不展平，zig 用）/ targz-dir / tarxz-dir / copy / gsudo / 7z-extra / 7zsfx / msi / rmux / single / vsbuild（见五） |
| `repo` | string | GitHub 仓库 `owner/name`（纯 cdn 工具可省） |
| `tag_prefix` | string | tag 前缀，剥离后得 version（如 `v`、`release-`） |
| `asset_pattern` | string | 资产名匹配正则（GitHub release 资产筛选） |
| `version_pattern` | string | 可选，从资产名提取版本的正则（python 用） |
| `cdn_url` | string | 可选，直链模板，含 `{version}` 占位（dotnet/oscdimg；golang 与 ziglang 不走 GitHub Releases，Windows 侧同 mac 走 go.dev/dl 与 ziglang.org/download 直链） |
| `cdn_index_url` | string | 可选，HashiCorp 式 index.json（vault） |
| `cdn_asset_pattern` | string | 可选，cdn 系资产名正则 |
| `cdn_version_url` | string | 可选，cdn 系版本查询地址（当前无在用工具，为后续 cdn 系工具预留） |
| `sums_asset` | string | 可选，官方统一校验清单资产名模板（`{version}`/`{tag}` 占位） |
| `sums_pattern` | string | 可选，校验清单内匹配本工具资产行的正则 |
| `asset_sha_suffix` | string | 可选，逐资产校验文件后缀（如 `.sha256`） |
| `bootstrap_asset` | string | 可选，安装自举资产（7z 的 7zr.exe） |

### Linux 平台专属字段

> 缺失时回退到同名的通用字段。ome 不回写。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `linux_repo` | string | 可选，Linux 专属 GitHub 仓库（通常与 `repo` 相同） |
| `linux_asset_pattern` | string | 可选，Linux 资产名匹配正则 |
| `linux_dir` | string | 可选，Linux 安装目录（可含 `~` 与 `$VAR`） |
| `linux_bin` | string | 可选，Linux 下注册进 PATH 的目录 |
| `linux_exe` | string | 可选，Linux 下版本探测 exe 路径（相对 `linux_dir`） |
| `linux_extract` | string | 可选，Linux 解压方式；支持 `targz-bin`（从 tar.gz 查找二进制）、`tarxz-bin`、`zip-bin`（从 zip 按叶子名取二进制）、`targz-dir` / `tarxz-dir`（全量解压不展平，版本目录/运行时树用） |
| `linux_extra_bins` | string | 可选，补充二进制叶子名（空格/逗号分隔）：`*-bin` 提取与 zip/targz 的 chmod 除 exe 外还覆盖这些成员（如 age-keygen、sg、uvx） |
| `linux_sums_pattern` | string | 可选，Linux 校验清单行匹配正则 |
| `linux_asset_sha_suffix` | string | 可选，Linux 逐资产校验文件后缀 |
| `linux_bootstrap_asset` | string | 可选，Linux 安装自举资产 |
| `linux_cdn_url` | string | 可选，Linux 直链模板，含 `{version}` 占位；Linux 下优先于 `cdn_url` |
| `linux_cdn_asset_pattern` | string | 可选，Linux 的 cdn 索引资产名正则（vault 类各平台资产名不同） |

### macOS 平台专属字段

> 仅 darwin 构建生效；缺失时先回退同义 `linux_*` 字段、再回退通用字段（M1 设立，2026-09-01）。ome 不回写。
> 字段集与 Linux 族同名对称（`mac_*` 前缀，含 `mac_cdn_url`/`mac_cdn_asset_pattern`/`mac_extra_bins`）；语义同上表对应项。
> exe 双语义（`toolver::exe_path`）：有平台专属 exe（`mac_exe`/`linux_exe`）时 exe 相对 install_dir，
> 回退通用 `exe` 时为 Windows 名录风格——路径自带 dir 段、相对 EnvRoot 直接拼。

### pin 字段

> ome update/pin 回写，保序保注释用 toml_edit。**按平台分列（M0 起）**：通用四键即 Windows pin，
> Linux/macOS 各有 `linux_*` / `mac_*` 四键，互不覆盖、无跨平台回退——回写只动当前平台四键。

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `tag` | string | Windows 锁定的 release tag |
| `version` | string | Windows 锁定的版本号 |
| `asset` | string | Windows 锁定的资产文件名 |
| `sha256` | string | Windows 锁定资产 sha256，大写；未回填可省略 |
| `linux_tag` / `linux_version` / `linux_asset` / `linux_sha256` | string | Linux 侧对应四键（回源自 psd1 Pos 侧） |
| `mac_tag` / `mac_version` / `mac_asset` / `mac_sha256` | string | macOS 侧对应四键（回源自 psd1 Pos 侧 AssetMac/Sha256Mac） |
| `hold` | bool | 可选，版本锁定（静态元数据、跨平台生效）：true 时 `update`（含 `--force`）、`daily`、`pin`、带版本选项的 `install` 全部跳过；无选项 `install` 仍按 pin 幂等。解锁 = 删该字段。首个用例 bun 1.3.14（最后一个完全用 Zig 编写核心的版本，2026-09-01 用户裁决） |

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
linux_dir = "~/.local/bin"
linux_bin = "~/.local/bin"
linux_exe = "age"
linux_extract = "targz"
linux_asset_pattern = '^age-v[0-9.]+-linux-amd64\.tar\.gz$'
tag = "v1.3.1"
version = "1.3.1"
asset = "age-v1.3.1-windows-amd64.zip"
sha256 = "C56E8CE22F7E80CB85AD946CC82D198767B056366201D3E1A2B93D865BE38154"
```

## 四、语义要点

1. **唯一 pin 源**：tag/version/asset/sha256 只存在于本文件；解析最新版只读不写，`pin`/`update` 才回写。
2. **sha256 校验优先级**：pin 的 sha256 > 官方校验源（sums_asset / asset_sha_suffix / cdn_index_url 自带 SUMS）；安装成功后可回填空 sha256。
3. **版本变更清 sha**：pin 到不同 version 时清掉旧 sha256，等 install 回填。
4. **平台边界**：Windows 字段为默认；平台专属字段以 `linux_` / `mac_` 前缀并列——静态字段 Linux 取 `linux_*` 回退通用，mac 取 `mac_*` 回退 `linux_*` 再回退通用；pin 字段按平台分列无回退（平台无 pin 即未锁定，`install` 不带 `--latest` 会提示先 pin）。sha256 随当前平台安装的 asset 回填到本平台键；本平台 pin 的 asset 与解析资产不一致时，该 sha256 不当作校验基准。
5. **数据主权（M0，2026-09-01）**：本文件是唯一权威。psd1 Pos 侧一次性回流后冻结，托管 26 节静态字段与 psd1 的关系由校验器维护：冲突报错、ome 增补（psd1 空、ome 有值）放行、ome 缺失报错；pin 值不校验（ome 合法回写）。本地节（reader/vsbuild/go/zig/shellcheck）不参与 psd1 校验。
6. **平台不适用（2026-09-01）**：单平台工具是常态数据形状（shellcheck 仅 `linux_*`、aria2/git 仅 Windows）。当前平台 effective exe 缺失即「平台不适用」：status 出空态行（installed 与 exe 渲染为 -）、install/update/daily/pin/query 跳过、package 拒绝（见 M106 M003）。
7. **仓库与部署态同步纪律（M0 定案）**：仓库 `catalog\tools.toml` 是唯一源；`ome self-deploy` 把它同步到用户数据目录（`<data>\ohmyenv\catalog\tools.toml`，幂等覆盖）。部署态副本上的 pin 回写（部署态二进制在任何 cwd 跑 `ome update/pin/install` 落用户数据目录副本）视为缓存漂移，**不构成权威**——pin 变更须回仓库：开发态在仓库 cwd 重跑同命令（或手动同步回）后提交入库，再 `ome self-deploy` 收敛部署态。

## 五、evergreen 条目

> `extract = "vsbuild"` 型条目（当前仅 vsbuild，自 ohmypwsh `scripts\set-vsbuild.ps1` 接管）不走「版本解析、sha 校验、pin 回写」主流程，规则如下。

1. **无 pin 字段**：tag、version、asset、sha256 整组省略——源是永续直链（aka.ms 引导器），无可锁版本、无官方 sha。`pin`/`update`/`daily` 对其跳过（提示 evergreen），`query` 只报直链与 evergreen 标记，`package` 拒绝（安装器型不可绿色分发）。
2. **安装幂等语义**：cl.exe 在位（`<EnvRoot>\vsbuild\VC\Tools\MSVC\*\bin\Hostx64\x64\cl.exe`）即视为已装，只补机器 PATH；重装等于修复。
3. **需管理员**：未提权且 gsudo 在位时经 gsudo 重跑 `ome install vsbuild`（退出码透传）；两者皆无则报错给出两条出路。
4. **PATH 写机器级**（HKLM，REG_EXPAND_SZ）：MSBuild 与 cl.exe 两个目录，非用户级；status 的 path 态按机器级判定。
5. **版本探测**：`exe` 指向跨版本稳定的 MSBuild.exe，`MSBuild -version` stdout 首行裸版本号（如 17.14.51.32402）取前三段。
6. **Windows 专属**：非 Windows 平台安装即报错。
7. **边界**：Windows SDK 不随 VS 组件（ISO 分离装 Windows Kits，暂留 ohmypwsh `set-windows-sdk.ps1`）。

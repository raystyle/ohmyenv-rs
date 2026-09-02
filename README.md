# ome

一句话定位：**Oh My Env**——自适应承担**本地本系统**的工具与运行时环境部署、管理、诊断的 CLI
（2026-09-02 用户定调）：在当前平台本机管理 Agent 所依赖的工具与运行时（fnm/bun/python/uv/dev 工具链等）
的版本解析、下载、校验、解压、PATH 注册、pin 锁定、日常更新、部署域验收与异常诊断。
ome 自身部署安装到用户目录、拥有独立用户数据目录、自注册 PATH——与被管理环境根（EnvRoot）彻底解耦。
ome 不做远程与集成——那是 ohmypwsh 的编排域。

三仓分工（2026-09-02 用户裁决）：**ome** 只管本地本系统（平台自适配）的部署/管理/诊断；
**ohmypwsh** 集成 ome——本地或远程执行 ome（远程由其下发二进制与 catalog 后 ssh 执行）、
负责 ome 自身的部署分发，并在 ome 装好密钥工具（sops/age 等）后承担密钥管理；
**Agent 四件套（claude/codex/grok/kimi）本体部署执行器归 D:\ohmyagents**。

## 快速开始

Windows：

```powershell
cargo build --release
.\target\release\ome.exe init   # 安装到用户目录 %LOCALAPPDATA%\Programs\ome，自注册用户 PATH
ome status
```

Linux / WSL：

```bash
cargo build --release
./target/release/ome init        # 安装到 ~/.local/bin，自注册 PATH（写入 ~/.bashrc）
ome status
```

## 命令

| 命令 | 说明 |
| --- | --- |
| `ome query [tool\|all] [--latest\|--tag\|--version]` | 只解析版本与资产，不下载 |
| `ome install [tool\|all]` | 装入环境目录，不改 PATH |
| `ome deploy [tool\|all]` | 安装 + 注册用户 PATH（默认锁定版本） |
| `ome update [tool\|all]` | 更新到最新版并锁定 |
| `ome pin [tool\|all] [--latest\|--version]` | 查看/设置 pin（lock 为别名） |
| `ome status` | 锁定 vs 已安装 vs PATH 三态对照（流式输出） |
| `ome daily [--dry-run] [--include-breaking]` | 日常更新：同主版本自动，跨主版本保留（退出码 2） |
| `ome init` | 安装自身到用户目录（Windows：`%LOCALAPPDATA%\Programs\ome`；Linux/mac：`~/.local/bin`），同步 catalog 到独立用户数据目录并自注册 PATH（`self-deploy` 为兼容别名；幂等） |
| `ome package <tool\|all> [--out <dir>] [--latest\|--tag\|--version]` | 打包工具为可分发目录（默认 `<EnvRoot>/cache/deploy/<tool>`），供 scp 与镜像离线装料；sha 校验与 install 同口径；all 批量容错（安装器型 vsbuild/rust 与平台不适用跳过续跑，单工具显式调用仍即失败） |
| `ome verify [--check <维度,...>] [--json]` | 部署域验收维度检查（Windows 9 维、Linux/mac 7 维，catalog 三态驱动；流式输出；FAIL 即 exit 1） |
| `ome heal [<维度>\|all] [--dry-run]` | 部署维度幂等自愈（heal-map 42 键迁嵌入注册表）：install 类走原生安装（pin 驱动）、密钥载体与镜像源原生移植（heal-keys/heal-mirror）；agent 域 12 键休眠、ohmypwsh 域 8 键提示路由；有 fail/partial 即 exit 1 |
| `ome doctor [--json]` | 部署异常诊断：版本漂移、探测失败、PATH 死链与重复、pin/sha 缺失、缓存孤儿、EnvRoot 可写等九项（FAIL 即 exit 1，WARN 不拦） |
| `ome self update [--stable\|--git]` | 升级自身三通道：默认 dev 滚动源（CI push main 构建滚动挂 pre-release `dev`）；`--stable` 拉 latest 正式版（v* tag 封版触发）；`--git` 浅克隆源码 cargo build 后替换（封版前通道）。按资产 sha256 对比自身，不同则下载校验替换部署位并同步 catalog |

## 输出格式与退出码

全局 `--format kv|json|jsonl`（默认 kv），`--json` 为 json 简写：

- **kv**：`key=value` 逐行，块间空行，`#` 注释行为分组标题（可滤）；
- **json**：整批输出一个 JSON 数组文档，stdout 恒为合法 JSON（无数据为 `[]`）；
- **jsonl**：每块一行 JSON 对象，逐工具/逐维度即出（流式与结构化兼得）。

值一律字符串，字段序与 kv 行序一致。数据只走 stdout；进度与提示（`[INFO]/[OK]/[WARN]`）走 stderr；
结构化模式下错误以单行 JSON `{"code","message","hint"?}` 附 stderr 末行，退出码不变形。

**对外数据契约（issue #4，2026-09-02 冻结）**：`query` 与 `status` 的 `--format json` 字段集与退出码
（0 成功 / 1 失败）为对外契约，ohmypwsh 镜像链（build-wsl-image、download-posix-dev 替代）按此消费；
契约演进只做**增量字段**（消费方按名取值不受影响），删除或改名视为 breaking，需在提交与 diary 显式标注并
通知消费方。query 的 `sha256` 字段语义：解析 tag 与资产同 pin 时给锁定 sha256（未回填为空串），否则空串。

| 命令 | 数据块字段 |
| --- | --- |
| `query` | tool, tag, version, asset, size, url, sha256 |
| `pin` | tool, tag, version, asset, sha256 |
| `install` / `deploy` / `update` | tool, action, version, dir |
| `status` | tool, locked, installed, path, exe |
| `daily` | tool, action, from, to |
| `init` | action, exe, bin_dir, catalog, path |
| `package` | tool, version, package_dir, bin_dir, main_bin |
| `verify` | name, verdict |
| `heal` | dim, action, params, result, detail |
| `doctor` | check, status, detail |

退出码：`0` 成功；`1` 失败（verify/doctor 有 FAIL 项、heal 有 fail/partial、安装出错）；`2` daily 有跨主版本保留项。

## 边界

- 工具名录七类 taxonomy（2026-09-02 裁决定稿，catalog 节序即类序）：**操作编排依赖**（ome 自管条目、oma 待 ohmyagents 集成、herdr）、**运行时依赖**（pwsh/python/dotnet/bun/fnm/wsl/docker/nushell）、**编译器依赖**（go/zig/rust/vsbuild）、**运行时衍生依赖**（browser-harness 走 uv tool；omcf 待 ohmycloud 集成）、**多路复用依赖**（rmux）、**远程服务依赖**（openssh/vault）、**命令工具依赖**（其余 20 个）。ome 自身以 `extract = "ome-self"` 条目在册：无 pin 无资产，升级走 `ome self update` 三通道。
- 2026-09-01 起承接 ohmypwsh 部署、验收、自愈完整迁移（分域路线见 ohmypwsh 仓库 P0026 方案）：本机 Windows 与 Linux 部署、远端四端二进制下发、verify 与 heal 已迁移。heal 边界（P0026 M4 与 2026-09-01 裁决）：agent 域键休眠（归 ohmyagents）；secret-guard、旧残留清零、compileMatrix、POSIX 系统位 aria2 归 ohmypwsh / 系统域；dev-rust 待 rustup 建模后转原生。
- 定位定调（2026-09-02）：自适应承担**本地本系统**的工具与运行时环境部署、管理、诊断（平台自适配，落在哪台机器就管哪台）；远程执行与集成编排归 ohmypwsh（其本地调 ome、远程下发 ome 后 ssh 执行）；密钥工具由 ome 部署、密钥管理归 ohmypwsh；Agent 四件套本体归 D:\ohmyagents。
- VS Build Tools 已接管（`ome install vsbuild`，evergreen 引导器无 pin、需管理员、机器级 PATH，语义见 R001 五）；Windows SDK 仍走 ohmypwsh ISO 分离装。
- Rust 已接管（`ome install rust`，rustup 引导器型：rsproxy 直链无 pin、stable 滚动即更新、RUSTUP_HOME/CARGO_HOME 重定位 EnvRoot、cargo sparse 镜像、cargo bin 用户 PATH；自 ohmypwsh set-rust.ps1 完整迁移；Windows 专列，dev-rust heal/verify 维度已转原生）。
- Docker Engine 已接管（`ome install docker`，Windows 容器 static zip + 服务注册 + daemon.json + compose 插件 + 机器级 PATH，自 ohmypwsh set-docker.ps1 完整迁移；CDN pin 驱动，升级需改 catalog pin 后重装）。
- Windows：被管理工具集中安装到 `D:\ohmyenv`（EnvRoot），PATH 走注册表；ome 自身装在用户目录 `%LOCALAPPDATA%\Programs\ome`，用户数据目录 `%LOCALAPPDATA%\ohmyenv`（catalog 部署态副本），与 EnvRoot 解耦。
- Linux/mac：ome 元数据在用户数据目录（`~/.local/share/ohmyenv`；mac `~/Library/Application Support/ohmyenv`），各软件按标准目录安装（默认单二进制到 `~/.local/bin`），PATH 通过 shell profile 管理。
- 工具名录唯一 pin 源：`catalog\tools.toml`（模式见 `docs\references\R001`）。

## 在 mac 上开发

开发主机切换到本机 mac（仓库 `https://github.com/raystyle/ohmyenv-rs`）：clone 到家目录后跑六门禁（build/test/fmt/clippy/两 md 扫描）；mac 目录与自部署形态、平台门控现状、三端验证分工与接力状态见 `docs\references\R011-mac开发接管-环境准备与构建验证.md`。历史 WSL/Linux 接管文档见 `docs\references\R010-linux开发接管-环境准备与构建验证.md`。

## 文档导航

- `AGENTS.md` — 协作规则最高约束
- `GOAL.md` / `PLAN.md` / `TODO.md` — 三原语
- `INDEX.md` — 唯一索引（编号表 / 目录结构 / 代码位置）

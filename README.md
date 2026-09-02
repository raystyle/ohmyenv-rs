# ome

一句话定位：**Oh My Env**——全平台 Agent 工具及运行时依赖环境部署管理 CLI（2026-09-02 用户定调）：
自 ohmypwsh 五端控制总台剥离的 Rust 实现，管理 Agent 工具链及其运行时依赖的版本解析、下载、校验、
解压、PATH 注册、pin 锁定、日常更新、部署域验收与异常诊断，一个标准、一个配置。
ome 自身部署安装到用户目录、拥有独立用户数据目录、自注册 PATH——与被管理环境根（EnvRoot）彻底解耦。

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
| `ome package <tool> [--out <dir>] [--latest\|--tag\|--version]` | 打包工具到指定目录（默认 `<EnvRoot>/cache/deploy/<tool>`），供 scp 分发 |
| `ome verify [--check <维度,...>] [--json]` | 部署域验收维度检查（Windows 9 维、Linux/mac 7 维，catalog 三态驱动；流式输出；FAIL 即 exit 1） |
| `ome doctor [--json]` | 部署异常诊断：版本漂移、探测失败、PATH 死链与重复、pin/sha 缺失、缓存孤儿、EnvRoot 可写等九项（FAIL 即 exit 1，WARN 不拦） |

## 边界

- 2026-09-01 起承接 ohmypwsh 部署、验收、自愈完整迁移（分域路线见 ohmypwsh 仓库 P0026 方案）：本机 Windows 与 Linux 部署、远端四端二进制下发、verify 已迁移；heal 按里程碑推进。
- 定位定调（2026-09-02）：全平台 **Agent 工具及运行时依赖**环境部署管理——Agent 四件套（claude/codex/grok/kimi）的部署执行器归属本 CLI（接管按 P0026 里程碑推进），配置与密钥域留 ohmypwsh。
- VS Build Tools 已接管（`ome install vsbuild`，evergreen 引导器无 pin、需管理员、机器级 PATH，语义见 R001 五）；Windows SDK 仍走 ohmypwsh ISO 分离装。
- Windows：被管理工具集中安装到 `D:\ohmyenv`（EnvRoot），PATH 走注册表；ome 自身装在用户目录 `%LOCALAPPDATA%\Programs\ome`，用户数据目录 `%LOCALAPPDATA%\ohmyenv`（catalog 部署态副本），与 EnvRoot 解耦。
- Linux/mac：ome 元数据在用户数据目录（`~/.local/share/ohmyenv`；mac `~/Library/Application Support/ohmyenv`），各软件按标准目录安装（默认单二进制到 `~/.local/bin`），PATH 通过 shell profile 管理。
- 工具名录唯一 pin 源：`catalog\tools.toml`（模式见 `docs\references\R001`）。

## 在 mac 上开发

开发主机切换到本机 mac（仓库 `https://github.com/raystyle/ohmyenv-rs`）：clone 到家目录后跑六门禁（build/test/fmt/clippy/两 md 扫描）；mac 目录与自部署形态、平台门控现状、三端验证分工与接力状态见 `docs\references\R011-mac开发接管-环境准备与构建验证.md`。历史 WSL/Linux 接管文档见 `docs\references\R010-linux开发接管-环境准备与构建验证.md`。

## 文档导航

- `AGENTS.md` — 协作规则最高约束
- `GOAL.md` / `PLAN.md` / `TODO.md` — 三原语
- `INDEX.md` — 唯一索引（编号表 / 目录结构 / 代码位置）

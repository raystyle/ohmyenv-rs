# ome

一句话定位：**Oh My Env**——本机 Windows 与 Linux（WSL）环境部署管理 CLI，自 ohmypwsh 五端控制总台的 `ohmyenv.ps1` 剥离的 Rust 实现：27 个工具的版本解析、下载、校验、解压、PATH 注册、pin 锁定、日常更新，一个标准、一个配置。

## 快速开始

Windows：

```powershell
cargo build --release
.\target\release\ome.exe self-deploy   # 复制到 D:\ohmyenv\ome\bin 并注册用户 PATH
ome status
```

Linux / WSL：

```bash
cargo build --release
./target/release/ome self-deploy        # 复制到 ~/.local/bin 并注册 PATH（写入 ~/.bashrc）
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
| `ome status` | 锁定 vs 已安装 vs PATH 三态对照 |
| `ome daily [--dry-run] [--include-breaking]` | 日常更新：同主版本自动，跨主版本保留（退出码 2） |
| `ome self-deploy` | 自部署二进制（Windows：`D:\ohmyenv\ome\bin`；Linux：`~/.local/bin`）并注册 PATH |
| `ome package <tool> [--out <dir>] [--latest\|--tag\|--version]` | 打包工具到指定目录（默认 `<EnvRoot>/cache/deploy/<tool>`），供 scp 分发 |

## 边界

- 管理本机 Windows 与 Linux（WSL 为验证主机）；远端/五端域（check/heal/omp/ssh-mesh）留在 ohmypwsh。
- 智能体（codex/claude/grok）安装不管理，归 ohmyagents / ohmypwsh。
- Windows：工具集中安装到 `D:\ohmyenv`，PATH 走注册表。
- Linux：ome 元数据存 `~/.local/share/ohmyenv`，各软件按标准目录安装（默认单二进制到 `~/.local/bin`），PATH 通过 shell profile（`~/.bashrc`）管理。
- 工具名录唯一 pin 源：`catalog\tools.toml`（模式见 `docs\references\R001`）。

## 在 mac 上开发

开发主机切换到本机 mac（仓库 `https://github.com/raystyle/ohmyenv`）：clone 到家目录后 `cargo build` / `cargo test`；mac 目录/PATH 策略、平台门控现状与三端验证分工见 `docs\references\R011-mac开发接管-环境准备与构建验证.md`。历史 WSL/Linux 接管文档见 `docs\references\R010-linux开发接管-环境准备与构建验证.md`。

## 文档导航

- `AGENTS.md` — 协作规则最高约束
- `GOAL.md` / `PLAN.md` / `TODO.md` — 三原语
- `INDEX.md` — 唯一索引（编号表 / 目录结构 / 代码位置）

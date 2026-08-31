# ome

一句话定位：**Oh My Env**——本机 Windows 环境部署管理 CLI，自 ohmypwsh 五端控制总台的 `ohmyenv.ps1` 剥离的 Rust 实现：26 个工具的版本解析、下载、校验、解压、PATH 注册、pin 锁定、日常更新，一个标准、一个配置。

## 快速开始

```powershell
cargo build --release
.\target\release\ome.exe self-deploy   # 复制到 D:\ohmyenv\ome\bin 并注册用户 PATH
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
| `ome self-deploy` | 自部署二进制到 `D:\ohmyenv\ome\bin` 并注册 PATH |

## 边界

- 只管本机 Windows；远端/五端域（check/heal/omp/ssh-mesh）留在 ohmypwsh。
- 智能体（codex/claude/grok）安装不管理，归 ohmyagents / ohmypwsh。
- Linux/mac 不进 ohmyenv 目录（系统标准目录策略），后续扩展。
- 工具名录唯一 pin 源：`catalog\tools.toml`（模式见 `docs\references\R001`）。

## 在 WSL 上开发

开发主机是本机 WSL 的 Linux 发行版（仓库 `https://github.com/raystyle/ohmyenv`）：clone 到 WSL 家目录后 `cargo build` / `cargo test`；工具链准备、平台门控现状与两端验证分工见 `docs\references\R010-linux开发接管-环境准备与构建验证.md`。

## 文档导航

- `AGENTS.md` — 协作规则最高约束
- `GOAL.md` / `PLAN.md` / `TODO.md` — 三原语
- `INDEX.md` — 唯一索引（编号表 / 目录结构 / 代码位置）

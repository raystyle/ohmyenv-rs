# GOAL：任务目标管理

> 角色：**工作任务管理**，四个部分——**起点**（何时/为何发起）、**锚点**（当前锚定的目标 + 推进时间线）、
> **进程**（当前目标的进程）、**历史**（所有已完成目标的轨迹）。随工作实时更新。

## 起点

- **日期**：2026-08-31。
- **起点**：ome 首版本机 Windows 域已收盘并推送云端（github.com/raystyle/ohmyenv），开发主机由 WSL/Linux 切换到本机 mac 接管后续开发；需要把 mac 接管所需的环境准备、构建验证、平台边界落成文档，让 mac 侧照着即可接手。

## 锚点

> 当前锚定的目标 + 推进时间线：从起点到现在的关键推进节点（带日期），达成后整条移入「历史」轨迹。

- **锚定的目标**：先让 ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管。

### 推进时间线

> 倒序。

| 日期 | 进展 |
| --- | --- |
| 2026-08-31 | mac 接管文档已就绪：新增 R011、R010 归档、README/INDEX/三原语切换为 mac 开发主机 |
| 2026-08-31 | 完成 Linux 本机部署支持 + 部署包能力：剩余工具 Linux 字段补齐，`ome package` 落地，cargo test + Windows cross-compile + md 扫描全绿 |
| 2026-08-31 | 立项：由 WSL 接管开发升级为 Linux 部署支持；更新 GOAL/PLAN/TODO/AGENTS 边界 |
| 2026-08-31 | 完成 WSL 接管开发：cargo build/test 全绿（58 passed），winreg 收 target 依赖，selfdeploy/toolver/install 测试加平台门控 |

## 进程

> 当前目标的进程：只记录当前这一个目标的进行状态。

- 当前目标：ohmypwsh 与 ome 的 Linux/Windows 现状对齐。ome 侧已梳理需要对齐的 catalog 字段、部署包目录、跳过工具；待用户在 ohmypwsh 仓库完成 catalog/脚本同步后，回到 ome 做回归验证。

## 历史

> 所有已完成目标的轨迹，按日期倒序。

| 日期 | 目标 | 结果 |
| --- | --- | --- |
| 2026-08-31 | 扩展 ome 支持 Linux 本机软件部署（除 agent 外） | 达成：catalog 补齐 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl Linux 字段；新增 `tarxz-bin`、`linux_cdn_url`、UTF-16 校验清单、共享目录 flatten；query/install 闭环验证 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl；cargo test + `cargo check --target x86_64-pc-windows-gnu` + md 扫描全绿 |
| 2026-08-31 | WSL Linux 下接管 ome 开发 | 达成：WSL 侧 cargo build / cargo test 全绿（58 passed）；winreg 收进 target 依赖；selfdeploy/toolver/install 测试按平台 cfg 门控；R010 实测回填 |
| 2026-08-31 | ome 首版本机 Windows 域可用 | 达成：八命令落地，60 测试全绿；status 26/26、query 同 tag、daily 同判定全对齐 ohmyenv.ps1；self-deploy 幂等；推送 github.com/raystyle/ohmyenv |

## 维护规则

- **起点**：开工时写一句「何时发起 + 为什么发起」。
- **锚点**：推进中保持「锚定的目标 + 推进时间线」最新——每完成一个节点补一行（记日期 + 进展）。
- **进程**：只记当前目标的进行状态，达成后整条移入「历史」。
- **历史**：每个目标达成/变更时，记一条（日期 + 目标 + 结果），按日期倒序。

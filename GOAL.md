# GOAL：任务目标管理

> 角色：**工作任务管理**，四个部分——**起点**（何时/为何发起）、**锚点**（当前锚定的目标 + 推进时间线）、
> **进程**（当前目标的进程）、**历史**（所有已完成目标的轨迹）。随工作实时更新。

## 起点

- **日期**：2026-09-01。
- **起点**：用户裁决 ohmypwsh 的部署、验收、自愈全家桶**完整迁移**给 ome（取代原 R012 共存对齐），pwsh 部署链按域退役；仓库改名 raystyle/ohmyenv-rs、本地工作副本迁 `D:\ohmyenv-rs`；安装形态整改为用户目录自部署 + 独立 app 数据目录。

## 锚点

> 当前锚定的目标 + 推进时间线：从起点到现在的关键推进节点（带日期），达成后整条移入「历史」轨迹。

- **锚定的目标**：承接 ohmypwsh 部署、验收、自愈完整迁移，按 ohmypwsh 仓库 P0026 方案的 M0..M6 里程碑推进。

### 推进时间线

> 倒序。

| 日期 | 进展 |
| --- | --- |
| 2026-09-01 | mac 批次全部收官并交接：开发接力回 Windows（D:\ohmyenv-rs）做 ohmypwsh 联动验收（P0026 M6 口径）；R011 一.4 刷新回接要点（基线门禁、平台 pin 不可覆盖、校验器只读），10 笔提交推送远端 |
| 2026-09-01 | mac 完美收敛：目录型运行时布局实证回填（pwsh/rmux/zig/go 四型）+ zip-bin/dir 提取型 + extra_bins 多二进制 + cdn pattern 平台族 + go/zig version 子命令与无前缀 tag 修复；starship 真装、vault/uv 升级、sops/go pin 补齐；终态 18 工具三态全等、其余如实空态；全门禁绿 |
| 2026-09-01 | M0 收官：psd1 Pos 侧一次性回流（linux/mac 静态族 + 平台 pin 分列 19+3 工具、mac pin 16 工具）、pin runtime 按平台分列（访问器/回写/sha 基准/状态判定）、转换器改只校验不再生（冲突报错增补放行）、go/zig/shellcheck 补录（kimi 按 agent 裁决剔除，31 工具）、R001 主权与同步纪律定案；校验器与全门禁绿 |
| 2026-09-01 | mac 首开发批次收官：M1 mac 字段族（`mac_*` + 三分支回退链 + exe 双语义）、jq/fnm mac 补录、R011 六项真机验证全绿、ohmypwsh 已部署 26 工具幂等接管实证（jq skipped 零重装零改写）；真机门禁修三坑（exe 双拼、clippy cfg 门控、测试沙盒漏 catalog 记 M102/M002） |
| 2026-09-01 | 交接 mac：用户赴 mac 接管开发，R011 刷新（新仓址、安装形态、M0 依赖 ohmypwsh psd1）；两仓推送远端 |
| 2026-09-01 | vsbuild 接管：evergreen 引导器（无 pin）+ gsudo 提权 + 机器级 PATH + MSBuild 稳定探测；真机幂等空转验证过（status installed=17.14.51 path=true） |
| 2026-09-01 | 立项批次：新址 clone 基线全绿（修 CRLF 检出敏感测试）、安装形态整改（用户程序目录自部署 + 数据目录 catalog 同步 + 旧 PATH 残留清理）、旧 clone 清理、R012 降级标注、三原语切换 |
| 2026-09-01 | 立项：完整迁移裁决；仓库改名 raystyle/ohmyenv-rs，本地 `D:\ohmyenv-rs` |

## 进程

> 当前目标的进程：只记录当前这一个目标的进行状态。

- 当前目标：承接完整迁移。立项批次、vsbuild 接管、M1 mac 域、M0 数据主权与回流、mac 完美收敛已收盘；**2026-09-01 开发接力回 Windows 做 ohmypwsh 联动验收**（回接要点见 R011 一.4；验收口径 P0026 M6）。其后 M2 远端二进制下发。

## 历史

> 所有已完成目标的轨迹，按日期倒序。

| 日期 | 目标 | 结果 |
| --- | --- | --- |
| 2026-08-31 | ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管 | 终结：被完整迁移裁决取代（2026-09-01）；对齐待办关闭为 superseded，数据迁移改走 P0026 M0 单向回流，mac 接管并入 M1 |
| 2026-08-31 | 扩展 ome 支持 Linux 本机软件部署（除 agent 外） | 达成：catalog 补齐 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl Linux 字段；新增 `tarxz-bin`、`linux_cdn_url`、UTF-16 校验清单、共享目录 flatten；query/install 闭环验证 pwsh/7z/dotnet/fnm/bun/uv/python/mq/herdr/rumdl；cargo test + `cargo check --target x86_64-pc-windows-gnu` + md 扫描全绿 |
| 2026-08-31 | WSL Linux 下接管 ome 开发 | 达成：WSL 侧 cargo build / cargo test 全绿（58 passed）；winreg 收进 target 依赖；selfdeploy/toolver/install 测试按平台 cfg 门控；R010 实测回填 |
| 2026-08-31 | ome 首版本机 Windows 域可用 | 达成：八命令落地，60 测试全绿；status 26/26、query 同 tag、daily 同判定全对齐 ohmyenv.ps1；self-deploy 幂等；推送 github.com/raystyle/ohmyenv |

## 维护规则

- **起点**：开工时写一句「何时发起 + 为什么发起」。
- **锚点**：推进中保持「锚定的目标 + 推进时间线」最新——每完成一个节点补一行（记日期 + 进展）。
- **进程**：只记当前目标的进行状态，达成后整条移入「历史」。
- **历史**：每个目标达成/变更时，记一条（日期 + 目标 + 结果），按日期倒序。

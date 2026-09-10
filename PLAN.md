# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D33 软件清单云端化与实时刷新（用户方向 2026-09-10「软件清单放 env 云端可实时更新，
> 无需动 ome 即可配置播种新软件」，用户令「开工」按建议三项定稿）。

### 依据

- 现状（已实证）：镜像已有 catalog 本体加 `.sha256` 边车（`ome/catalog/tools.toml`，seed-mirror 路线 B 随 catalog 变更与每日自动推，D31/D32 两次入镜实测）；catalog 解析四级（`OME_CATALOG` > 二进制同级 > cwd > 用户数据目录）加全 miss 自举（官方 raw 优先、镜像边车锚回落）。
- 缺口：消费侧只在全 miss 时拉一次，部署机装完不再回头读云端，新增软件与 pin 变动要等下一次 `ome init` / `self update`。
- 三项定稿（用户「开工」）：权威落位取仓库副本为开发与离线兜底源、云端为运行态权威；刷新时机取 TTL 24h 自动（`OME_CATALOG_TTL` 秒级可调、0 关；`OME_OFFLINE=1` 关）加显式 `ome catalog status/sync`；信任锚取镜像自算 sha256 边车（先边车后资产），签名留后。

### 方案骨架

1. **两子功能落位**：主功能 catalog 独占入口，刷新与状态采集都在 `src/catalog.rs` 内（用户第 3 轮结构裁：不另立 catalogsync 概念）。sync 子功能先取边车锚（`?t=` 击穿），再下载资产（`?v=<锚>` 击穿），过 sha 校验与 `Catalog::load` 解析验证（防半截件）后，以同目录临时文件替换落位用户数据副本；TTL 判定走标记文件（记上次检查时刻与 sha，不碰 catalog 文件 mtime）；status 子功能采集解析面来源、本地与云端锚、年龄、TTL 与同步态。
2. **命令面**：`ome catalog [status|sync]`（缺省 status；sync 为显式通道，不受 TTL 与离线开关限制）：status 报解析面路径与 origin（repo / userdata / env）、本地与云端 sha、检查年龄、TTL 与离线态、是否同源。
3. **自动刷新接线**：main 在解析 catalog 后、加载前，仅当解析面**就是用户数据副本**时按 TTL 刷新；失败静默回落本地不拦命令，刷新成功打一行 stderr `[OK]`。
4. **开发态零干扰**：仓库 cwd、二进制同级、`OME_CATALOG` 指定面一律不读不改；`OME_CATALOG_TTL=0` 与 `OME_OFFLINE=1` 全关，`catalog sync` 仍可显式执行。
5. **文档同步**：R013 新命令数据块；README 命令表与 SKILL 命令图与 `--llms`（顺修 manifest 里「41 工具」旧计数）；AGENTS 意图路由加清单刷新一条；R001 四.7 部署副本语义升格（云端权威加本地回写仍是临时态）与入册 checklist 增补；CHANGELOG、PRD/GOAL/TODO、diary 一篇。

### 完成定义

- `ome catalog status` 三态可读：路径、origin、本地与云端 sha、年龄、TTL、同步态。
- `ome catalog sync` 幂等：锚同则不重写；锚变则拉取校验落位，坏件（解析失败或 sha 不符）拒收且不动本地。
- 部署面（用户数据副本）在 TTL 到期且云端有变更时自动更新；开发面（仓库 cwd）零改写。
- 纯函数单测加 `OME_TEST_MIRROR=1` 闸门真网测（云端拉取与锚校验端到端）。

### 验收

- `cargo test` 全绿（含新模块单测与闸门测）、`cargo clippy` 干净、`rumdl check .` 与 `.tools` 三扫描绿。
- 真机端到端：把镜像已入镜的 typst 刷进本机部署副本，再用**未含 D33 的旧部署二进制**在非仓库目录读到 typst（实证「不动 ome 即可配置播种新软件」）。
- 关闭通道实证：`OME_CATALOG_TTL=0` 与 `OME_OFFLINE=1` 时 status 不触发刷新；`--force` 显式仍可用。
- 黄金文件与夹具不破（夹具 catalog 不驻新命令面字段）。

# PRD：需求清单管理

> 角色：需求清单，四原语之首，需求驱动目标。GOAL 的每个目标应能回指本清单条目。
> 分工：PRD=要什么；GOAL=要达成什么；PLAN=怎么做；TODO=做到哪。
> 本清单于 2026-09-05 补建（evo 骨架重建）：D01 至 D05 按 GOAL 历史与 diary 时间线追溯登记，均标 [推断]；此后新需求走追问链逐条澄清后登记。

## 需求清单

| 编号 | 需求 | 状态 | 澄清轮次 | 派生去向 |
| --- | --- | --- | --- | --- |
| D01 | ome 立项：自 ohmypwsh 的 `ohmyenv.ps1` 剥离为本机跨平台环境部署管理 Rust CLI，Windows 域八命令（query/pin/install/deploy/update/status/daily/self-deploy）可用 | 已交付 | [推断] 追溯登记（2026-09-05） | GOAL 历史 2026-08-31；S001/S002 选型；R001 数据模式 |
| D02 | WSL Linux 下接管 ome 开发（构建、测试、平台门控） | 已交付 | [推断] 追溯登记（2026-09-05） | GOAL 历史 2026-08-31；R010（已归档） |
| D03 | 扩展 ome 支持 Linux 本机软件部署（agent 除外，归 ohmyagents/ohmypwsh） | 已交付 | [推断] 追溯登记（2026-09-05） | GOAL 历史 2026-08-31；平台抽象层与 linux 字段族 |
| D04 | ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管 | 已拒绝 | [推断] 追溯登记（2026-09-05） | 原因：2026-09-01 被完整迁移裁决取代（superseded）；对齐待办关闭，数据迁移改走 P0026 M0 单向回流；R012 降级为数据迁移参考。防复问：不再回到「双向对齐」路线 |
| D05 | 承接 ohmypwsh 部署、验收、自愈完整迁移（M0..M6 里程碑，验收口径 ohmypwsh P0026） | 已采纳 | 2026-09-01 用户裁决（完整迁移取代共存对齐） | GOAL 锚点（登记日 2026-09-01）；PLAN；TODO；M0..M4 已收盘，余量 M6 |
| D06 | 用 project-evo 重建文档体系：补 PRD/proven/ROADMAP/AGENTS 义务表，INDEX 对账磁盘修断链断号，diary 禁字清剿（PE-12 全绿） | 已交付 | 2026-09-05 用户两裁：全面重整档 + PRD 全量追溯 | evo check 13 项全绿（PE-06 合法 SKIP）；GOAL 时间线 2026-09-05；diary 同日 |
| D07 | 命令原语重构第一批：doctor 升 ome 核心命令做三层诊断（系统 / agent 二进制加版本加 token 可用性 / 依赖六类加环境错误检测），检测驱动安装，agent 二进制入册 ome catalog 由 install/deploy 幂等承载；oma 收窄为配置 agent、hook、编排（D06 方向反转） | 已采纳 | 第 3 轮共六裁（2026-09-05）：ome 承载、oma agents install 迁册 ome、六类（agent / 运行时 / 运行时管理器 uv 加 fnm / 编译器含 vsbuild / cli 工具 / 运行时衍生）、先 doctor 命令面大重构另立项、oma 登录态等四类检查归 agents 域、doctor 不取设置 | GOAL 起点 2026-09-05；PLAN 切片；oma 仓 D06 注记同步 |
| D08 | 全软件分发兜底渠道：download 层官方渠道失败回落 env.ohmygh.com 镜像（按 `<tool>/<version>/<asset>` 模板拼 URL，sha 校验同 catalog pin 值），为默认分发切换自建做准备（用户方向：Cloudflare Worker 加 S3 加自有域名自己管理分发）；D07 切片 3 的渠道兜底部分（grok GCS、kimi CDN manifest）并入本需求统一实现 | 已采纳 | 第 1 轮三裁（2026-09-05，随 ohmycloud D36 同轮：域名 env. 子域、全量一步到位、先手动种子后 Actions 自动） | ohmycloud D36 基建联动；D07 切片 3 渠道部分并入 |

## 维护规则

- 一条需求一行，D 编号两位接续（D01、D02…）。
- 新需求先入本表「待澄清」，经追问链澄清后流转「已澄清 / 已采纳」；状态生命周期：待澄清、已澄清、已采纳、已交付，任一状态可转「已拒绝」（记原因防复问）。
- 「派生去向」回指 GOAL 锚点 / PLAN 切片 / P 编号 / S 编号；目标交付后回填本表状态。
- 追溯条目必须标 [推断]；未标记者视为当期追问链产物。

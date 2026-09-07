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
| D07 | 命令原语重构第一批：doctor 升 ome 核心命令做三层诊断（系统 / agent 二进制加版本加 token 可用性 / 依赖六类加环境错误检测），检测驱动安装，agent 二进制入册 ome catalog 由 install/deploy 幂等承载；oma 收窄为配置 agent、hook、编排（D06 方向反转） | 已交付 | 第 3 轮共六裁（2026-09-05）：ome 承载、oma agents install 迁册 ome、六类（agent / 运行时 / 运行时管理器 uv 加 fnm / 编译器含 vsbuild / cli 工具 / 运行时衍生）、先 doctor 命令面大重构另立项、oma 登录态等四类检查归 agents 域、doctor 不取设置 | 四切片全收：九类 taxonomy 与入册（461a178）、doctor 三层（9179893）、全链适配与 D08 渠道（8c4953e、9c48903）、oma 迁册由 oma 侧 P0029 交付；双端验收绿 |
| D08 | 全软件分发兜底渠道：download 层官方渠道失败回落 env.ohmygh.com 镜像（按 `<tool>/<version>/<asset>` 模板拼 URL，sha 校验同 catalog pin 值），为默认分发切换自建做准备（用户方向：Cloudflare Worker 加 S3 加自有域名自己管理分发）；D07 切片 3 的渠道兜底部分（grok GCS、kimi CDN manifest）并入本需求统一实现 | 已交付 | 第 1 轮三裁（2026-09-05，随 ohmycloud D36 同轮：域名 env. 子域、全量一步到位、先手动种子后 Actions 自动） | download_asset_with_mirror 落码双端断源实测绿（8c4953e）；ohmycloud 种子 69/69；darwin 补种 ohmycloud#3 进行中 |
| D09 | 命令面 agent 友好化（参考 evo agent-native CLI 契约，用户定调 2026-09-07「所有命令参考 evo 的 agent 友好实践简化，命令主要给 agent 使用」）：发现层（根 SKILL.md 与 `ome --llms` 紧凑清单）、CTA（诊断缺口带下一步建议命令）、输出契约对齐（kv 紧凑已达、R013 在档）；命令面合并或收敛（增减裁决）另批 | 已采纳 | 第 0 轮（用户方向指令） | PLAN 三件套；命令面增减以 D10 原语口径为准 |
| D10 | ome 功能原语定义（2026-09-07 用户裁定：三原语加派生）：**三原语**为 doctor（检测诊断：三层加环境错误）、install（幂等安装：检测驱动，本地与目标一致即免装）、status（三态对照：锁定/已装/PATH）；**派生面**语义挂靠原语不另立：query 为解析前置（install 的 dry 态）、deploy 为 install 加 PATH、update 与 daily 为 install 时变、pin 为锚操作（数据面）、verify 与 heal 为断言与自愈组合、package 与 init 与 self 为辅助通道。后续命令面重构与 D09 实现以此为准 | 已采纳 | 第 1 轮（三原语加派生） | SKILL.md 与 R013 与 AGENTS 意图路由同步原语口径 |
| D11 | doctor 配置健康面扩展（用户 2026-09-07「诊断不仅要检测 PATH 变量、SHELL 使用命令情况，还要检测运行时配置、编译器配置和依赖等所有配置和使用正常不」）：在 PATH 卫生与命令探测之外加**配置健康节**：运行时配置（npm 源、bunfig 遥测、cargo sparse 镜像、GOPROXY 等 heal-mirror 域判据）、编译器配置（rust toolchain、go env 关键变量）、依赖与密钥载体（heal-keys 域）逐项 OK/WARN/FAIL；检测判据与 heal 自愈动作同源（检测-自愈闭环，heal-map 数据源复用）；密钥域不管（用户裁定归 ohmypwsh），关注面收敛为软件安装部署与默认配置（用户裁定） | 已交付 | 第 0 轮（用户方向指令） | PLAN 切片；判据清单从 heal-map mirror/keys 类提取 |

## 维护规则

- 一条需求一行，D 编号两位接续（D01、D02…）。
- 新需求先入本表「待澄清」，经追问链澄清后流转「已澄清 / 已采纳」；状态生命周期：待澄清、已澄清、已采纳、已交付，任一状态可转「已拒绝」（记原因防复问）。
- 「派生去向」回指 GOAL 锚点 / PLAN 切片 / P 编号 / S 编号；目标交付后回填本表状态。
- 追溯条目必须标 [推断]；未标记者视为当期追问链产物。

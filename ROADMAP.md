# ROADMAP：阶段与里程碑

> 阶段与里程碑状态（四态：已完成 / 进行中 / 已规划 / 待定）。只记版本级节点，不记任务流水；任务进度见 `TODO.md`，版本明细见 `CHANGELOG.md`。

## 开发流程 2026-09-08 裁 S005

分支：GitHub Flow 单干变体（直推 main：验证后推、一事一提交、Conventional Commits 前缀）；并行 agent 会话或危险大改开短命分支（<1 天）squash 进 main 后删；不设长命 develop；分支保护与 PR 门禁待开放协作者时启用。发版：tag 驱动手动封版（见阶段七）；release-please 等自动化攒单不引入（CHANGELOG 手工为 evo 权威）。

## 集成优先级 2026-09-10 裁

与 omc / oma 集成的功能排序（用户裁，D29）：首要诊断与检测（doctor：系统 / 依赖两层，D30 收窄），其次恢复与治愈（heal / verify），最后安装部署配置（install 面）。三仓共识细则（omc agent deploy 委托 ark、发版知会 ohmycloud）见 R014 六；ark-rs#10 类集成余量按此序排期（仓 D41 前名 ohmyenv-rs，旧 issue 号沿用）。

## 阶段总览

| 阶段 | 状态 | 里程碑 |
| --- | --- | --- |
| 一、立项与 Windows 域可用 | 已完成 | 2026-08-31：八命令落地、catalog 数据层、status 三态真机绿（60 测试全绿） |
| 二、Linux 域与开发接管 | 已完成 | 2026-08-31：WSL 接管开发（R010）、linux 字段族与平台抽象层 |
| 三、mac 开发接管 | 已完成 | 2026-09-01：M1 mac 字段族、R011 六项真机验证、mac 管理域全量实证收敛 |
| 四、部署域迁入 ome（D05） | 已完成 | M0 数据主权、M2 四端齐、M3 verify、M4 heal 与 rust 接管、self update 五端闭环已收盘；M6 随 D18 取消（源项目不存在） |
| 五、独立仓命令面（D18） | 已完成 | 2026-09-07：不再有 ohmypwsh；本机命令面可跑 |
| 六、生态 | 已规划 | ohmycloud 为资源分发基建兄弟仓（D08/D19）；种子清单走 ISSUE（D20 / R014 / ohmycloud#5）；D29 三仓共识：omc agent deploy 委托 ark、发版知会镜像锚、集成优先级诊断先行（R014 六）；oma/omcf catalog 预留条目待集成 |
| 七、封版发布 | 进行中 | 首个封版 v0.2.0（2026-09-10 急令当日发布，三仓对齐里程碑 D30）。流程七步：CHANGELOG 收口、ROADMAP 切状态、tag、push tag（CI v* 通道出正式 release 与双 stable 段直推）、herdr 知会、`self update --stable` 验收；hotfix 走 fix forward 出 patch tag，不维护多版本线 |
| 八、更名 Ark 与 v1.0.0（D41） | 进行中 | 2026-09-12：ome 更名 **Ark（Agent Runtime Kit）**，仓 raystyle/ark-rs，四阶段（A 身份核心、B 分发链、C 自举与存量兼容、D 文档发版）逐批 codex 对线落地；**v1.0.0 封版**（同日）：旧名读回兼容、镜像双写双附过渡（停 ome/ 面（段与资产名）判据为存量机水位清零，届时 omc 联动批切）；后续 1.x 演进待裁决 |

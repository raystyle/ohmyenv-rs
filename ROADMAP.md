# ROADMAP：阶段与里程碑

> 阶段与里程碑状态（四态：已完成 / 进行中 / 已规划 / 待定）。只记版本级节点，不记任务流水；任务进度见 `TODO.md`，版本明细见 `CHANGELOG.md`。

## 阶段总览

| 阶段 | 状态 | 里程碑 |
| --- | --- | --- |
| 一、立项与 Windows 域可用 | 已完成 | 2026-08-31：八命令落地、catalog 数据层、status 逐项对齐 ohmyenv.ps1（60 测试全绿） |
| 二、Linux 域与开发接管 | 已完成 | 2026-08-31：WSL 接管开发（R010）、linux 字段族与平台抽象层、package 分发 |
| 三、mac 开发接管 | 已完成 | 2026-09-01：M1 mac 字段族、R011 六项真机验证、mac 管理域全量实证收敛 |
| 四、ohmypwsh 完整迁移（D05） | 进行中 | M0 数据主权、M2 四端齐、M3 verify、M4 heal 与 rust 接管、self update 五端闭环均已收盘；余量 M6（ohmypwsh 部署链退役配合，验收口径 P0026 M6） |
| 五、生态集成 | 已规划 | oma/omcf 兄弟仓集成（catalog 预留条目）；跨仓 ISSUE 矩阵推进（ohmypwsh#6#7、ohmyagents#2#3、ohmycloud#1） |
| 六、封版发布 | 待定 | 首个正式版 v1.x（封版流程见 evo flow-release；触发条件待用户裁决） |

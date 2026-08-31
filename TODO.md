# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪进度（待办/进行中/已完成）。

## 当前目标

ome 首版本机 Windows 域可用——query/install/deploy/update/pin/status/daily/self-deploy 八命令，行为与产物对齐 ohmyenv.ps1（登记日 2026-08-31）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 脚手架与 git 仓库 | 已完成 | Cargo.toml、.gitignore、docs 骨架、git init | 2026-08-31 |
| 三原语与数据模式文档 | 已完成 | GOAL/PLAN/TODO/INDEX + R001 tools.toml 模式 | 2026-08-31 |
| catalog 转换器与 tools.toml | 进行中 | .tools\import-catalog.ps1 从 catalog.psd1 + New-ToolDef 生成 29 工具 | 2026-08-31 |
| 核心链路 query/pin | 进行中 | catalog/resolve/download/checksum 四模块 + query/pin 命令 | 2026-08-31 |
| 安装链路 install/deploy/update | 待办 | extract 九分派、envpath 注册表 PATH、toolver 版本探测 | |
| status/daily/self-deploy | 待办 | 三态对照、daily 退出码 2 语义、self-deploy 复制到 D:\ohmyenv\ome\bin 并注册 PATH | |
| 测试与真机对齐验收 | 待办 | tests\cli.rs 沙盒 + 与 ohmyenv.ps1 status/query/daily 对照 | |

# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪进度（待办/进行中/已完成）。

## 当前目标

ome 首版本机 Windows 域可用——query/install/deploy/update/pin/status/daily/self-deploy 八命令，行为与产物对齐 ohmyenv.ps1（登记日 2026-08-31）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 脚手架与 git 仓库 | 已完成 | Cargo.toml、.gitignore、docs 骨架、git init | 2026-08-31 |
| 三原语与数据模式文档 | 已完成 | GOAL/PLAN/TODO/INDEX + R001 tools.toml 模式 | 2026-08-31 |
| catalog 转换器与 tools.toml | 已完成 | .tools\import-catalog.ps1 生成 29 工具，已验证符合 R001 | 2026-08-31 |
| 核心链路 query/pin | 已完成 | catalog/resolve/download/checksum 四模块 + query/pin；build/test 全绿（23 单元 + 4 集成）；真实 tools.toml 烟测 jq/vault/python 通过 | 2026-08-31 |
| 安装链路 install/deploy/update | 已完成 | toolver/extract/envpath/install 四模块 + 三命令；47 测试全绿；真机 `update jq` 同 tag 跳过烟测通过 | 2026-08-31 |
| status/daily/self-deploy | 已完成 | status 三态分组、daily exit 2 语义、self-deploy 幂等实测通过（复制+PATH 注册二次全跳过） | 2026-08-31 |
| 测试与真机对齐验收 | 已完成 | 沙盒测试全绿；OME_TEST_REAL 闸门：status 29/29 一致、query jq 同 tag；M001 catalog 保真分歧已修正（转换器对齐 Get-EnvLock 合并语义） | 2026-08-31 |

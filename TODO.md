# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

先让 ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管（登记日 2026-08-31）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 梳理 ohmypwsh 需要对齐的接口 | 已完成 | 落成 `docs/references/R012-ohmypwsh与ome对齐清单-linux-windows.md` | 2026-08-31 |
| ohmypwsh catalog 与部署脚本对齐 | 待办 | 用户在 ohmypwsh 仓库执行 | |
| ome 侧回归验证 | 已完成 | cargo test 66 全绿、clippy 零告警、fmt 全仓统一；交叉 check 受 C 工具链限制（xz2/native-tls），记 R010 五、6 | 2026-09-01 |
| 文档记录对齐结果 | 待办 | diary + 必要时 references | |
| 文档扫描 | 待办 | rumdl + py 扫描 | |
| mac 真机构建验证 | 暂停 | 待 ohmypwsh 对齐完成后再推进 | |
| 新增 reader 工具安装 | 已完成 | raystyle/reader_rs v0.1.0 入名录（27 工具），转换器保留本地节；真机 deploy 通过 | 2026-09-01 |
| S002 测试三件套落地 | 已完成 | tests\expected 黄金文件 oracle（pin/status）+ tests\common helper；dies_ 负例补齐至 10 个可成组过滤；R004 补 expected 文件 oracle 段 | 2026-09-01 |

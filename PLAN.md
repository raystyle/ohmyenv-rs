# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**——当前目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标（对应 GOAL.md 登记日 2026-08-31）：先让 ohmypwsh 与 ome 的 Linux/Windows 现状对齐，再推进 mac 接管。任务项见 `TODO.md`。

1. **梳理需要对齐的接口**：从 ome 侧整理 ohmypwsh 需要同步的事项，落成 `docs/references/R012-ohmypwsh与ome对齐清单-linux-windows.md`。
2. **ohmypwsh 侧调整（用户在 ohmypwsh 仓库执行）**：同步 catalog.psd1 / helpers.ps1 / 部署脚本，使 ohmypwsh 的 Linux/Windows 行为与 ome catalog 一致。
3. **ome 侧复核**：ohmypwsh 改动后，回到 ome 跑 `cargo test` 与 `cargo check --target x86_64-pc-windows-gnu`，确认没有回归。
4. **文档记录**：在 `docs/diary/` 记录对齐决策与变更。
5. **验证**：跑 `rumdl check .` + `.tools` 两个 py 扫描；一事一提交。

## 完成定义

- ohmypwsh 的 catalog.psd1 与 ome 的 tools.toml 在 Linux/Windows 字段上无冲突、无遗漏。
- ohmypwsh 的部署脚本能消费 ome `package` 命令产出的目录结构（`<tool>/bin/<tool>`）。
- `cargo test` WSL 全绿；`cargo check --target x86_64-pc-windows-gnu` 通过；md 扫描全绿。
- 对齐结果落在 ome 的 diary 与/或 references 文档中。

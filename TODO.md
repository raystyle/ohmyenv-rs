# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

WSL Linux 下接管 ome 开发——文档就位，WSL 侧 clone 后 build/test 跑通，Windows 专属行为有明确的验证回路（登记日 2026-08-31）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| 接管文档更新（Windows 侧） | 已完成 | GOAL/PLAN/TODO 换锚、R010 接管细则、README/INDEX/diary/CHANGELOG 同步，三件套全绿，推送 origin | 2026-08-31 |
| 智能体工具剥离 | 已完成 | codex/claude/grok 移出名录（归 ohmyagents/ohmypwsh），29 降至 26；转换器加显式排除，文档同步 | 2026-08-31 |
| 开发主机锚定 WSL | 已完成 | 接管目标由「lan-linux 或 WSL」收窄为本机 WSL 发行版，GOAL/PLAN/TODO/R010 同步 | 2026-08-31 |
| WSL 侧工具链准备 | 待办 | rustup/cargo + uv/python（+ 可选 rumdl），按 R010 一节 | |
| WSL 侧首次 build/test | 待办 | WSL 家目录 clone 后 cargo build / cargo test；结果回填 R010 实测栏 | |
| Windows 门控补齐 | 待办 | 如首 build 暴露：winreg 收 target 专属依赖等，保证 WSL 全绿 | |
| 两端验证分工实证回填 | 待办 | WSL 跑非 Windows 逻辑测试；Windows 跑 OME_TEST_REAL 闸门 | |

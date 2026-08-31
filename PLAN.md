# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**——当前目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标（对应 GOAL.md 登记日 2026-08-31）：Linux 下接管 ome 开发。任务项见 `TODO.md`。

1. Windows 侧（本轮）：落定接管文档——GOAL/PLAN/TODO 换锚、R010 接管细则（环境准备、构建验证、平台门控现状、验证回路）、README 补开发入口、INDEX 登记、diary 记流水；验证三件套全绿后提交并推送 origin。
2. Linux 侧（接手后）：按 R010 准备工具链（rustup/cargo、uv + python、可选 rumdl），clone 仓库，跑 `cargo build` 与 `cargo test`。
3. 门控补齐（如首 build 暴露）：把 Windows 专属依赖（winreg）与代码收进 `cfg(windows)` /target 专属依赖，保证 Linux 下 build/test 全绿；不绿处逐个收编。
4. 验证回路确立：Linux 侧负责非 Windows 逻辑（解析/下载/校验/渲染/错误结构）的 build + test；Windows 专属行为（注册表 PATH、msi、self-deploy、真机对齐）回 Windows 跑 `OME_TEST_REAL=1 cargo test` 验收，两端分工写进 R010 实测回填。

## 完成定义

- R010 接管细则落库并推送，Linux 侧照文档可独立完成 clone 到 build/test。
- Linux 下 `cargo build` / `cargo test` 全绿（允许 Windows 专属测试按平台 cfg 跳过）。
- 两端验证分工在 R010 里有实证记录。

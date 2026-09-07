# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：审查缺陷收口（2026-09-07）。D09 已交付（SKILL / `--llms` / CTA / `ome skill`）。D07 已交付。

### 依据

整仓审查：安装 pin/sha 与 tag 不对齐、doctor 判据误报、POSIX PATH 单槽、文档四原语落后于 D09 至 D13。

### 方案骨架

1. 安装链：`expected_sha256` 同 tag；sha 回写仅同发行或 `update_lock`；下载 `.part`；CDN 显式 version 重算 tag。
2. doctor/verify/heal：probe-fail 看文件、死链按路径分量、rust 走 EnvRoot、D13 总超时、localbin16 去 vault。
3. 平台：Unix PATH 块多目录；mac 只认 `mac_exe`；self update 通道与 pin 保留。
4. 文档与冒烟：PRD/GOAL/INDEX/README/SKILL 对齐；`--llms` 与缺子命令测试。

### 完成定义

- 库测与 `tests/cli.rs` 全绿；clippy `-D warnings` 绿。
- 四原语与 INDEX 反映 D09/D12/D13 已交付与 40 工具。

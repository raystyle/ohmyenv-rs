# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D21 向 ohmycloud 派三面对齐 ISSUE（2026-09-07）已交付。

### 依据

用户裁定：种子资源清单用 ISSUE 和 ohmycloud 派任务和对齐。ohmycloud 是分发基建兄弟仓（D19）。

### 方案骨架

1. catalog 为唯一权威；`.tools\seed-inventory.py` 抽可入镜对象。
2. 流程落 R014；向 `raystyle/ohmycloud` 开 ISSUE，本仓开跟踪 ISSUE。
3. 差集写进 ISSUE：补种、下架、待 sha、latest 段保持。

### 完成定义

- R014 在档；脚本可从 catalog 再生清单。
- ohmycloud 种子 ISSUE 已开，含当期差集与 85 对象清单。

### 验收

已派 [ohmycloud#5](https://github.com/raystyle/ohmycloud/issues/5)；本仓跟踪 [ohmyenv-rs#8](https://github.com/raystyle/ohmyenv-rs/issues/8)。

2026-09-07 通报闭环：镜像 85 加 latest 6；`OME_TEST_MIRROR=1` 两测绿；差集三处出入已域面 HEAD 复核（M012：派单前先 HEAD）。

zoxide/ffmpeg `linux_sha256` 已回填（官方资产哈希与 GitHub digest 一致）。HEAD 404 后派补种 [ohmycloud#7](https://github.com/raystyle/ohmycloud/issues/7)。

D21：[ohmycloud#8](https://github.com/raystyle/ohmycloud/issues/8) 问如何对齐 ome 自身与在管软件的分发、更新、安装。

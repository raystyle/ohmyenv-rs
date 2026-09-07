# PLAN：当前目标规划指导

> 角色：**当前目标的规划指导**：当前这个目标怎么推进（步骤/标准/验收），随目标变化更新，不存历史目标。
> 与 `TODO.md` 分工：todo = 当前目标任务进度清单（做到哪）；本文件 = 当前目标怎么做（步骤/标准/流程）。

## 当前目标实施计划

> 当前目标：D09 命令面 agent 友好化（登记日 2026-09-07，用户定调「所有命令参考 evo 的 agent 友好实践简化，命令主要给 agent 使用」）。D07 已交付（四切片收口见 GOAL 时间线与 PRD）。

### 依据

evo `tool-cli-agents` 参考的契约对照 ome 现状：发现层（根 SKILL.md 与 `--llms`）缺失、CTA（下一步建议）缺失；输出契约基本达标（kv 紧凑默认、json/jsonl 结构化、错误单行 JSON，R013 在档）；管道代码逃生舱对部署 CLI 适用性存疑（无运行时语境）。

### 方案骨架

> 第一批三件，纯增量不动现有命令。

1. **根 SKILL.md**（oma/reader 家族同款）：何时用 ome（环境部署/诊断/更新场景一律走 ome，不手拼下载不裸 curl 官方源）加 13 命令图（一行摘要，细则指向 R013 与 README）加镜像兜底与幂等检测语义；`ome init` 同步 SKILL 到部署目录。
2. **`ome --llms`**：打印紧凑命令清单（markdown 表：命令、语义、关键输出字段、退出码要点），agent 零安装可用；JSON schema 形态留第二批（clap 定义一份多渲染）。
3. **CTA 下一步建议**：doctor 依赖层 missing 与 agent 层 binary=missing 带建议命令（`ome install <tool>`）；status 漂移行带 `ome update <tool>` 建议；错误 hint 已有（OmeError）本批补数据面。

第二批候选（另批追问用户裁决）：命令面合并收敛（增减）、TOON 表格化列表输出、管道代码逃生舱适用性评估。

### 完成定义

- 根 SKILL.md 在仓与 `ome init` 部署位同步；`ome --llms` 输出稳定可解析；doctor/status 的 CTA 真机可见。
- 测试与文档四件套全绿；R013 与 README 同步新面。

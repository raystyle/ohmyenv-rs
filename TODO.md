# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

承接 ohmypwsh 部署、验收、自愈完整迁移，按 ohmypwsh 仓库 P0026 方案的 M0..M6 里程碑推进（登记日 2026-09-01）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| M0 数据主权与回流 | 待办 | psd1 Pos 侧 26 工具 linux pin（含 sha256）回流 tools.toml 修复 linux 字段丢失（转换器再生时吞字段，`mac_*` 字段 jq/fnm 已有同列保护）；import-catalog.ps1 改「只校验不再生」；go/zig/shellcheck/kimi 补录；仓库与部署态 catalog 同步纪律定案 | |
| M1 后续：mac 逐工具补录 | 待办 | M0 数据到位后逐工具补 `mac_*`（pwsh 等 status 失真项优先，本机 `~/.local/bin` 26 工具为接管对象） | |
| M2 远端四端二进制下发 | 待办 | 交叉编译 linux-musl/darwin-arm64 单二进制 + catalog，scp 后远端 ome install/deploy | |
| M3 verify 移植 | 待办 | 维度族数据化，`ome verify` 与 verify-five-ends -Json 双跑对账零 diff | |
| M4 heal 移植 | 待办 | heal-map 42 键迁嵌入数据，`ome heal [--dry-run]` | |
| M5 远端通道与 agent 归属 | 待办 | ssh 调系统 ssh 复用 mesh；agent 四件套部署执行器接入 | |
| M6 ohmypwsh 链退役配合 | 待办 | 配合 ohmypwsh 逐域 deprecated，验收口径见 P0026 M6 | |

### 已完成批次

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| M1 mac 字段族与真机验证 | 已完成 | Tool 增 `mac_*` 族、`effective_*` 三分支回退链（mac→linux→通用）、exe 双语义修复；jq/fnm 首批 mac 补录；R011 六项真机验证全绿（70 测试、install/deploy 幂等接管在位 jq、self-deploy 部署态解析、package fnm、gnu 交叉 check）；真机门禁修三坑（exe 双拼、clippy cfg、测试沙盒漏 catalog 记 M102） | 2026-09-01 |
| vsbuild 接管 | 已完成 | evergreen 引导器（无 pin 无 sha）+ gsudo 自动提权 + 机器级 PATH + MSBuild 稳定探测；74 测试全绿、真机幂等空转验证；Windows SDK 仍留 ohmypwsh（ISO 分离） | 2026-09-01 |
| 新址 clone 与基线门禁 | 已完成 | D:\ohmyenv-rs 基线 993e77b；修 CRLF 检出敏感测试（夹具 CRLF 检出时 `\r\n` 双转）后门禁全绿 | 2026-09-01 |
| 安装形态整改 | 已完成 | 自部署进 `%LOCALAPPDATA%\Programs\ome`、catalog 同步 `%LOCALAPPDATA%\ohmyenv`、旧 PATH 残留清理；catalog 解析扩四级；68 测试全绿 | 2026-09-01 |
| 承接完整迁移登记 | 已完成 | R012 降级标注（被 P0026 取代）、三原语切换、INDEX 与 diary 登记 | 2026-09-01 |
| ohmypwsh catalog 与部署脚本对齐 | superseded | 被完整迁移裁决取代（2026-09-01）：数据改走 M0 单向回流，不再双向对齐 | 2026-09-01 |

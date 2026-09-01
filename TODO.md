# TODO：当前目标任务进度清单

> 角色：**当前目标的任务进度清单**——按 `GOAL.md` 当前目标列任务项，追踪每个任务的进度（待办/进行中/已完成）。

## 当前目标

承接 ohmypwsh 部署、验收、自愈完整迁移，按 ohmypwsh 仓库 P0026 方案的 M0..M6 里程碑推进（登记日 2026-09-01）。

## 任务进度清单

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| go/zig Windows 安装建模 | 待办 | golang/zig 不走 GitHub Releases（tag 404 实证），需按 mac 模式建 cdn_url 直链解析（go.dev/dl、ziglang.org/download）加 zip-dir 提取型（现有 dir 型只有 targz/tarxz，Windows 资产是 zip）；补 Windows pin 后 status 五项全完美 | |
| ohmypwsh 联动验收 | 待办 | 对齐 P0026 M6 口径：ohmypwsh 部署链逐域 deprecated 配合、ome 与 ohmypwsh 状态对账；mac 侧随时按 R011 一.4 复验 | |
| ohmypwsh 联动验收 | 待办 | 对齐 P0026 M6 口径：ohmypwsh 部署链逐域 deprecated 配合、ome 与 ohmypwsh 状态对账；mac 侧随时按 R011 一.4 复验 | |
| M2 远端四端二进制下发 | 待办 | 交叉编译 linux-musl/darwin-arm64 单二进制 + catalog，scp 后远端 ome install/deploy | |
| M3 verify 移植 | 待办 | 维度族数据化，`ome verify` 与 verify-five-ends -Json 双跑对账零 diff | |
| M4 heal 移植 | 待办 | heal-map 42 键迁嵌入数据，`ome heal [--dry-run]` | |
| M5 远端通道与 agent 归属 | 待办 | ssh 调系统 ssh 复用 mesh；agent 四件套部署执行器接入 | |
| M6 ohmypwsh 链退役配合 | 待办 | 配合 ohmypwsh 逐域 deprecated，验收口径见 P0026 M6 | |

### 已完成批次

| 任务项 | 进度 | 说明 | 日期 |
| --- | --- | --- | --- |
| Windows 回接与基线门禁 | 已完成 | mac 10 笔拉回后全门禁绿（76 测试、fmt/clippy、md 三件套、校验器「托管 26 节与 psd1 一致」）；修 Windows 真仓回归：仅 linux 字段工具（shellcheck）无通用 exe 致 status 全挂，加 platform_managed 容忍（status 空态行、install/update/daily/pin/query 跳过、package 拒绝），M106 记 M003 | 2026-09-01 |
| M1 后续：mac 完美收敛 | 已完成 | 目录型运行时布局实证回填（pwsh→powershell/7、rmux→~/.local 树、zig→~/zig 版本目录、go→~/.local/go，新增 zip-bin/targz-dir/tarxz-dir 提取型）；extra_bins 多二进制（age-keygen/sg/uvx 实证）；cdn pattern 平台族修 vault；go/zig version 子命令与无前缀 tag 修复；sops/uv/vault/go mac pin 补齐（starship 真装、vault/uv 升级接管）；终态 18 工具 locked=installed、pwsh 版本对齐、win-only 与 official 布局如实空态 | 2026-09-01 |
| M0 数据主权与回流 | 已完成 | psd1 Pos 侧一次性回流（19 在管工具 linux 静态族+pin、16 工具 mac pin，钉版本 pattern 解开为 [0-9.]+）；pin runtime 平台分列（访问器/回写/sha 基准/status/update 全链切平台键，黄金双 oracle）；import-catalog.ps1 改只校验不再生（冲突报错、ome 增补放行、平台族完整性）；go/zig/shellcheck 补录（kimi 按不管理 agent 裁决剔除，共 31 工具）；同步纪律定案入 R001 | 2026-09-01 |
| M1 mac 字段族与真机验证 | 已完成 | Tool 增 `mac_*` 族、`effective_*` 三分支回退链（mac→linux→通用）、exe 双语义修复；jq/fnm 首批 mac 补录；R011 六项真机验证全绿（70 测试、install/deploy 幂等接管在位 jq、self-deploy 部署态解析、package fnm、gnu 交叉 check）；真机门禁修三坑（exe 双拼、clippy cfg、测试沙盒漏 catalog 记 M102） | 2026-09-01 |
| vsbuild 接管 | 已完成 | evergreen 引导器（无 pin 无 sha）+ gsudo 自动提权 + 机器级 PATH + MSBuild 稳定探测；74 测试全绿、真机幂等空转验证；Windows SDK 仍留 ohmypwsh（ISO 分离） | 2026-09-01 |
| 新址 clone 与基线门禁 | 已完成 | D:\ohmyenv-rs 基线 993e77b；修 CRLF 检出敏感测试（夹具 CRLF 检出时 `\r\n` 双转）后门禁全绿 | 2026-09-01 |
| 安装形态整改 | 已完成 | 自部署进 `%LOCALAPPDATA%\Programs\ome`、catalog 同步 `%LOCALAPPDATA%\ohmyenv`、旧 PATH 残留清理；catalog 解析扩四级；68 测试全绿 | 2026-09-01 |
| 承接完整迁移登记 | 已完成 | R012 降级标注（被 P0026 取代）、三原语切换、INDEX 与 diary 登记 | 2026-09-01 |
| ohmypwsh catalog 与部署脚本对齐 | superseded | 被完整迁移裁决取代（2026-09-01）：数据改走 M0 单向回流，不再双向对齐 | 2026-09-01 |

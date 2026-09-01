# M106-catalog转换与数据保真-错误

> catalog 转换器与 tools.toml 数据保真类错误速查（一行一事，同根因合并聚合）。

## 行级条目

| 编号 | 日期 | 状态 | 现象 | 根因 | 正确处理 |
| --- | --- | --- | --- | --- | --- |
| M001 | 2026-08-31 | 已修正（2026-08-31） | ome status 对 rmux 报 installed=-，pwsh 报 0.10.0（tests\real.rs 真机闸门实测） | import-catalog.ps1 合并规则原为「psd1 Win 侧优先（仅 category/version_pattern 取 New-ToolDef）」，而 pwsh 运行时 Get-EnvLock 是 New-ToolDef 全量覆盖静态字段：rmux 有效 Exe=%LOCALAPPDATA%\rmux\bin\rmux.exe、Extract=rmux、SumsAsset=SHA256SUMS | 已把转换器合并规则对齐 Get-EnvLock（New-ToolDef 定义过的静态字段全量以其为准，含显式置空；未定义字段回退 psd1；pin 字段始终取 psd1），重生成后 diff 仅 rmux（official 生效）与 grok 两节。grok 的 New-ToolDef 资产模式与 CdnUrl 停在 1.0.5，落后于 pin 1.0.13，是 pwsh 侧自身滞后（不能改 ohmypwsh）：grok 走 cdn_url 直链解析，asset_pattern 闲置，按覆盖规则照常合并无害；若日后 grok 版本升级，需先在 ohmypwsh 侧刷新其 tooldef 的 1.0.5 硬编码再重跑转换器。tests\real.rs 的 rmux 豁免已移除，闸门 29/29 一致 |
| M003 | 2026-09-01 | 已修正（2026-09-01） | Windows 真仓 `ome status` 退出 1 报「工具缺少 exe 字段」（mac 三批次合入后首次复现） | M0 补录的 shellcheck 只有 `linux_*` 字段族无通用 `exe`；`exe()` 回退链平台不对称（mac 有 linux 兜底、Windows 仅通用），加夹具三工具全带通用 exe、真机闸门未跑 status，三重盲区 | 代码层凡消费 effective 字段处须容忍平台缺失：`toolver::platform_managed` 判定（effective exe 存在即本平台在管），status 出空态行（exe 渲染 -）、install/update/daily/pin/query 跳过、package 拒绝；测试用「全平台无 exe 的 ghost 工具」做可移植验证 |

## 范围注记

- 2026-08-31：智能体（codex/claude/grok）安装不属 ome 管理域，转换器显式剔除，ome 管理 26 个工具。M001 中 grok 的 New-ToolDef 滞后点对 ome 随之失效（grok 已不在名录），该前置条件仅对 ohmypwsh 侧自身有效。

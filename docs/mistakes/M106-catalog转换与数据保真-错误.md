# M106-catalog转换与数据保真-错误

> catalog 转换器与 tools.toml 数据保真类错误速查（一行一事，同根因合并聚合）。

## 行级条目

| 编号 | 日期 | 现象 | 根因 | 正确处理 |
| --- | --- | --- | --- | --- |
| M001 | 2026-08-31 | ome status 对 rmux 报 installed=-，pwsh 报 0.10.0（tests\real.rs 真机闸门实测） | import-catalog.ps1 合并规则是「psd1 Win 侧优先（仅 category/version_pattern 取 New-ToolDef）」，而 pwsh 运行时 Get-EnvLock 是 New-ToolDef 全量覆盖静态字段：rmux 有效 Exe=%LOCALAPPDATA%\rmux\bin\rmux.exe、Extract=rmux、SumsAsset=SHA256SUMS，tools.toml 却是相对 exe 加 zip | 转换器合并规则对齐 Get-EnvLock（New-ToolDef 定义的静态字段优先，psd1 只供 pin 与 tooldef 未定义字段）；注意 grok 的 New-ToolDef 资产模式停在 1.0.5 落后于 pin 1.0.13（pwsh 侧自身滞后），全量覆盖前需先在 ohmypwsh 侧刷新 grok 的 tooldef，否则把滞后值倒灌进 ome；修复后移除 tests\real.rs 的 rmux 已知分歧豁免 |

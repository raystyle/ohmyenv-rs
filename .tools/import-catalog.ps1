#Requires -Version 7.0
<#
.SYNOPSIS
    import-catalog.ps1 已退役（D18）。

.DESCRIPTION
    catalog\tools.toml 是唯一 pin 源与静态字段权威，不再对照外部 psd1。
    本脚本保留文件名以免旧门禁断链，运行即提示后退出 0。

.EXAMPLE
    pwsh -NoProfile -File .tools\import-catalog.ps1
#>
param(
    [string]$PwshRoot = '',
    [string]$OmeRoot = (Split-Path -Parent $PSScriptRoot)
)

Write-Host '[INFO] import-catalog.ps1 已退役（D18）：catalog\tools.toml 是唯一权威，不再对照外部 catalog。'
exit 0

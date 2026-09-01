#Requires -Version 7.0
<#
.SYNOPSIS
    import-catalog.ps1 - ohmypwsh 与 ome catalog 的一致性校验器（只校验不再生，M0 起 2026-09-01）。

.DESCRIPTION
    历史角色：从 ohmypwsh 生成 catalog\tools.toml（只取 psd1 Win 侧，Pos 侧忽略——
    该设计曾把手补的 linux_*/mac_* 字段整表再生吞掉，即 M0 立项根因，见 M106 与 M0 diary）。

    现角色（数据主权在 ome）：catalog\tools.toml 是唯一 pin 源与静态字段权威；
    psd1 Pos 侧数据已于 2026-09-01 一次性回流完毕，此后 psd1 冻结、本脚本只读校验，绝不写文件。

    校验内容（对托管 26 节；本地节如 reader/vsbuild/go/zig/shellcheck 只做存在性登记不校验）：
      1. 托管节齐全（ToolNames 剔除智能体 codex/claude/grok/kimi 后的 26 个）；
      2. Win 静态字段与 psd1 Win + New-ToolDef 合并结果逐字段一致（ome 不回写静态字段，漂移即错）；
      3. 平台族完整性：psd1 有 Pos 侧的工具必须有 linux_asset_pattern 与 linux pin 四键；
         有 AssetMac 的必须有 mac_asset_pattern 与 mac pin 四键（防再吞）；
      4. 平台 asset_pattern 与 psd1 一致（psd1 钉版本的形态按 [0-9.]+ 解开后比对）；
      5. pin 值不校验（ome update/pin 合法回写，主权在 ome）；sha256 形态须为 64 位大写 hex。

    数据源（严格只读）：
      - <PwshRoot>\scripts\catalog.psd1 与 <PwshRoot>\scripts\helpers.ps1
      - PwshRoot 缺省 D:\ohmypwsh（Windows），不存在则试 $HOME/ohmypwsh-src（mac/Linux clone）。

.EXAMPLE
    pwsh -NoProfile -File .tools\import-catalog.ps1          # 校验，漂移退出码 1
#>
param(
    [string]$PwshRoot = '',
    [string]$OmeRoot  = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if (-not $PwshRoot -or -not (Test-Path (Join-Path $PwshRoot 'scripts/catalog.psd1'))) {
    $PwshRoot = if (Test-Path 'D:\ohmypwsh\scripts\catalog.psd1') { 'D:\ohmypwsh' }
                elseif (Test-Path "$HOME/ohmypwsh-src/scripts/catalog.psd1") { "$HOME/ohmypwsh-src" }
                else { throw "找不到 ohmypwsh 数据源（D:\ohmypwsh 或 `$HOME/ohmypwsh-src）" }
}

# ---------- 1. 读取 catalog.psd1（只读） ----------
$catalog = Import-PowerShellDataFile (Join-Path $PwshRoot 'scripts/catalog.psd1')

# ---------- 2. 子进程 dot-source helpers.ps1，导出 ToolNames + New-ToolDef ----------
$helpersPath = Join-Path $PwshRoot 'scripts/helpers.ps1'
if (-not (Test-Path $helpersPath)) { throw "找不到数据源: $helpersPath" }
$extractor = @"
`$ErrorActionPreference = 'Stop'
. '$($helpersPath -replace "'","''")'
`$result = [ordered]@{ toolNames = @(`$script:ToolNames); defs = [ordered]@{} }
foreach (`$t in `$script:ToolNames) { `$result.defs[`$t] = New-ToolDef -Tool `$t }
`$result | ConvertTo-Json -Depth 6 -Compress
"@
$tmpScript = Join-Path ([System.IO.Path]::GetTempPath()) "ome-import-catalog-extract-$PID.ps1"
try {
    [System.IO.File]::WriteAllText($tmpScript, $extractor, (New-Object System.Text.UTF8Encoding($false)))
    $json = (& pwsh -NoProfile -File $tmpScript | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw "helpers.ps1 子进程提取失败（exit $LASTEXITCODE）" }
} finally {
    Remove-Item $tmpScript -ErrorAction SilentlyContinue
}
$meta = $json | ConvertFrom-Json
[string[]]$toolNames = @($meta.toolNames)

# ---------- 2.1 管理域：智能体不属 ome（codex/claude/grok；2026-09-01 追认 kimi 同类） ----------
$excludedTools = @('codex', 'claude', 'grok', 'kimi')
if ($toolNames.Count -ne 29) { throw "源 ToolNames 数量应为 29，实际 $($toolNames.Count)" }
$toolNames = @($toolNames | Where-Object { $_ -notin $excludedTools })
if ($toolNames.Count -ne 26) { throw "剔除智能体后应为 26 个工具，实际 $($toolNames.Count)" }

# ---------- 3. 读 ome tools.toml（只读） ----------
$omeToml = Join-Path $OmeRoot 'catalog/tools.toml'
if (-not (Test-Path $omeToml)) { throw "找不到 $omeToml" }
$omeText = [System.IO.File]::ReadAllText($omeToml)
# 节文本切分：[tools.x] 到下一个 [tools. 或文件尾
$sections = @{}
foreach ($m in [regex]::Matches($omeText, '(?ms)^\[tools\.([^\]]+)\]\r?\n(.*?)(?=^\[tools\.|\z)')) {
    $sections[$m.Groups[1].Value] = $m.Groups[2].Value
}

function Get-WinField($win, [string]$name) {
    if ($null -eq $win) { return '' }
    $v = $win[$name]
    if ($null -eq $v) { return '' }
    return [string]$v
}
function Get-DefProp($def, [string]$name) {
    $p = $def.PSObject.Properties[$name]
    if ($null -eq $p -or $null -eq $p.Value) { return '' }
    return [string]$p.Value
}
# 从节文本取键值（TOML 字符串剥引号；单引号字面串与双引号基本串）
function Get-TomlField([string]$section, [string]$key) {
    $m = [regex]::Match($section, "(?m)^(?:linux_|mac_)?$([regex]::Escape($key))\s*=\s*(.+)$")
    if (-not $m.Success) { return $null }
    $v = $m.Groups[1].Value.Trim()
    if ($v.StartsWith("'")) { return $v.Trim("'") }
    return $v.Trim('"')
}
function Get-SectionField([string]$section, [string]$key) {
    $m = [regex]::Match($section, "(?m)^$([regex]::Escape($key))\s*=\s*(.+)$")
    if (-not $m.Success) { return $null }
    $v = $m.Groups[1].Value.Trim()
    if ($v.StartsWith("'")) { return $v.Trim("'") }
    return $v.Trim('"')
}
# psd1 钉版本形态解开后比对（回流时的同一变换）
function Unbake-Pattern([string]$pattern, [string]$version) {
    if (-not $version) { return $pattern }
    $escapedForm = $version.Replace('.', '\.')
    return $pattern -replace [regex]::Escape($escapedForm), '[0-9.]+'
}

$errors = [System.Collections.Generic.List[string]]::new()

# psd1 Win 字段 -> tools.toml 静态字段（与 R001 字段表一致）
$staticMap = @(
    @{ Key = 'dir';               Psd1 = 'Dir' }
    @{ Key = 'bin';               Psd1 = 'Bin' }
    @{ Key = 'exe';               Psd1 = 'Exe' }
    @{ Key = 'extract';           Psd1 = 'Extract' }
    @{ Key = 'repo';              Psd1 = 'Repo' }
    @{ Key = 'tag_prefix';        Psd1 = 'TagPrefix' }
    @{ Key = 'asset_pattern';     Psd1 = 'AssetPattern' }
    @{ Key = 'cdn_url';           Psd1 = 'CdnUrl' }
    @{ Key = 'cdn_index_url';     Psd1 = 'CdnIndexUrl' }
    @{ Key = 'cdn_asset_pattern'; Psd1 = 'CdnAssetPattern' }
    @{ Key = 'cdn_version_url';   Psd1 = 'CdnVersionUrl' }
    @{ Key = 'sums_asset';        Psd1 = 'SumsAsset' }
    @{ Key = 'sums_pattern';      Psd1 = 'SumsPattern' }
    @{ Key = 'asset_sha_suffix';  Psd1 = 'AssetShaSuffix' }
    @{ Key = 'bootstrap_asset';   Psd1 = 'BootstrapAsset' }
)

foreach ($name in $toolNames) {
    if (-not $catalog.Contains($name)) { $errors.Add("$name`: 不在 catalog.psd1"); continue }
    if (-not $sections.Contains($name)) { $errors.Add("$name`: tools.toml 缺节"); continue }
    $entry = $catalog[$name]
    $win = $entry['Win']
    $def = $meta.defs.PSObject.Properties[$name]
    if ($null -eq $def) { $errors.Add("$name`: 无 New-ToolDef 定义"); continue }
    $def = $def.Value
    $sec = $sections[$name]

    # 静态字段：New-ToolDef 定义过的以其为准，否则 psd1 Win（与历史生成规则一致）。
    # 主权语义（M0）：冲突（两侧皆非空且不等）报错；ome 增补（psd1 空、ome 有值）放行；
    # ome 缺失（psd1 有值、ome 空）报错。
    foreach ($m in $staticMap) {
        $defHas = $null -ne $def.PSObject.Properties[$m.Psd1]
        $expect = if ($defHas) { Get-DefProp $def $m.Psd1 } else { Get-WinField $win $m.Psd1 }
        $actual = Get-SectionField $sec $m.Key
        if ($null -eq $actual) { $actual = '' }
        if ($expect -eq $actual) { continue }
        if ([string]::IsNullOrEmpty($expect)) { continue }   # ome 增补，放行
        $errors.Add("$name`: 静态字段 $($m.Key) 漂移（psd1 侧 '$expect'，tools.toml 侧 '$actual'）")
    }
    $deploy = ''
    if ($null -ne $entry['Deploy']) { $deploy = [string]$entry['Deploy']['win'] }
    $actualDeploy = Get-SectionField $sec 'deploy'
    if ($null -eq $actualDeploy) { $actualDeploy = '' }
    if ($deploy -ne $actualDeploy) { $errors.Add("$name`: deploy 漂移（'$deploy' vs '$actualDeploy'）") }

    # pin 形态：sha256 键若存在须为 64 位大写 hex（值不校验，主权在 ome）
    foreach ($shaKey in @('sha256', 'linux_sha256', 'mac_sha256')) {
        $v = Get-SectionField $sec $shaKey
        if ($null -ne $v -and $v -notmatch '^[0-9A-F]{64}$') { $errors.Add("$name`: $shaKey 形态非法（须 64 位大写 hex）: $v") }
    }

    # 平台族完整性：Pos 侧存在则 linux 族必须在；AssetMac 存在则 mac 族必须在
    $pos = if ($entry.Contains('Pos') -and $null -ne $entry['Pos']) { $entry['Pos'] } else { $null }
    if ($pos) {
        $expectPat = Unbake-Pattern ([string]$pos['AssetPattern']) ([string]$pos['Version'])
        $actualPat = Get-SectionField $sec 'linux_asset_pattern'
        if ([string]::IsNullOrEmpty($expectPat)) {
            if ($null -ne $actualPat) { $errors.Add("$name`: psd1 无 linux AssetPattern 但 tools.toml 有 linux_asset_pattern") }
        } elseif ($actualPat -ne $expectPat) {
            $errors.Add("$name`: linux_asset_pattern 漂移（期望 '$expectPat'，实际 '$actualPat'）")
        }
        foreach ($k in @('linux_tag', 'linux_version', 'linux_asset')) {
            if ($null -eq (Get-SectionField $sec $k)) { $errors.Add("$name`: 缺 $k（Pos 侧有数据，平台 pin 键不得缺失）") }
        }
        if ($pos.Contains('AssetMac') -and $pos['AssetMac']) {
            foreach ($k in @('mac_asset_pattern', 'mac_tag', 'mac_version', 'mac_asset')) {
                if ($null -eq (Get-SectionField $sec $k)) { $errors.Add("$name`: 缺 $k（AssetMac 在位，mac 族不得缺失）") }
            }
        }
    }
}

if ($errors.Count -gt 0) {
    Write-Host "校验失败（$($errors.Count) 处漂移）：" -ForegroundColor Red
    $errors | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
$localSections = @($sections.Keys | Where-Object { $_ -notin $toolNames })
Write-Host "校验通过：托管 $($toolNames.Count) 节与 psd1 一致；本地节 $($localSections.Count) 个（$($localSections -join ', ')）不校验" -ForegroundColor Green

#Requires -Version 7.0
<#
.SYNOPSIS
    import-catalog.ps1 - 从 ohmypwsh（pwsh 侧）生成 ome 的 catalog\tools.toml。

.DESCRIPTION
    数据源（严格只读，不修改 ohmypwsh 任何文件）：
      - D:\ohmypwsh\scripts\catalog.psd1 ：取每个工具的 Win 侧字段 + Deploy.win（Pos 侧忽略）。
      - D:\ohmypwsh\scripts\helpers.ps1  ：$script:ToolNames（29 个受管工具名单与顺序）与
        New-ToolDef 的权威静态元数据（Category 分类 key/agent/project/base/extras、VersionPattern）。

    合并规则（对齐 helpers.ps1 Get-EnvLock 的运行时语义）：
      - 静态元数据：New-ToolDef 定义过的字段全量以其为准（含 tooldef 显式置空的字段），
        tooldef 未定义的字段回退 psd1 Win 侧；
      - pin 字段（tag/version/asset/sha256）始终取 psd1 Win 侧；
      - sha256 统一大写；未 pin（空值）的字段整行省略，不写空字符串。

    New-ToolDef 数据获取方式说明（方式 a，子进程 dot-source）：
      helpers.ps1 顶层有 PATH 重建、Console 编码设置等副作用，但这些副作用仅限当前进程，
      因此派生一个 pwsh 子进程 dot-source 后导出 JSON，对本脚本与系统完全无害。
      不选方式 b（正则/AST 解析函数体）：New-ToolDef 是约 386 行的 switch 字面量表，
      文本解析脆弱且要随源文件写法变动维护，直接执行求值才是权威结果。

    幂等性：输出完全确定（工具顺序固定为 ToolNames 顺序、字段顺序固定、无时间戳），
    重复运行生成字节一致的 tools.toml，可随时安全重跑。

    生成物：catalog\tools.toml（UTF-8 无 BOM）。pin 字段后续由 ome update / ome pin 回写。

    管理域排除（2026-08-31 裁决）：智能体安装不属 ome 管理域（归 ohmyagents / ohmypwsh），
    codex / claude / grok 三个智能体工具从生成结果剔除，ome 管理 26 个工具。

    本地新增工具保留（2026-09-01）：ohmypwsh 名录之外、ome 自行新增的工具节
    （如 reader），重跑转换时整节原样保留在托管 26 节之后，不被覆盖。

.EXAMPLE
    pwsh -NoProfile -File D:\ohmyenv\ome\.tools\import-catalog.ps1
#>
param(
    [string]$PwshRoot = 'D:\ohmypwsh',
    [string]$OmeRoot  = 'D:\ohmyenv\ome'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# ---------- 1. 读取 catalog.psd1（只读） ----------
$psd1Path = Join-Path $PwshRoot 'scripts\catalog.psd1'
if (-not (Test-Path $psd1Path)) { throw "找不到数据源: $psd1Path" }
$catalog = Import-PowerShellDataFile -Path $psd1Path

# ---------- 2. 子进程 dot-source helpers.ps1，导出 ToolNames + New-ToolDef 为 JSON ----------
$helpersPath = Join-Path $PwshRoot 'scripts\helpers.ps1'
if (-not (Test-Path $helpersPath)) { throw "找不到数据源: $helpersPath" }

$extractor = @"
`$ErrorActionPreference = 'Stop'
. '$($helpersPath -replace "'","''")'
`$result = [ordered]@{ toolNames = @(`$script:ToolNames); defs = [ordered]@{} }
foreach (`$t in `$script:ToolNames) { `$result.defs[`$t] = New-ToolDef -Tool `$t }
`$result | ConvertTo-Json -Depth 6 -Compress
"@
$tmpScript = Join-Path $env:TEMP "ome-import-catalog-extract-$PID.ps1"
try {
    [System.IO.File]::WriteAllText($tmpScript, $extractor, (New-Object System.Text.UTF8Encoding($false)))
    $json = (& pwsh -NoProfile -File $tmpScript | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw "helpers.ps1 子进程提取失败（exit $LASTEXITCODE）" }
} finally {
    Remove-Item $tmpScript -ErrorAction SilentlyContinue
}
$meta = $json | ConvertFrom-Json
[string[]]$toolNames = @($meta.toolNames)

# ---------- 2.1 管理域排除：智能体工具不属 ome（归 ohmyagents / ohmypwsh） ----------
# 显式名单剔除而非按 category 过滤：New-ToolDef 的 agent 类还含 pwsh/rmux 等基础设施。
$excludedTools = @('codex', 'claude', 'grok')
if ($toolNames.Count -ne 29) {
    $errors0 = "源 ToolNames 数量应为 29，实际 $($toolNames.Count)"
    throw $errors0
}
$toolNames = @($toolNames | Where-Object { $_ -notin $excludedTools })
if ($toolNames.Count -ne 26) {
    throw "剔除智能体后应为 26 个工具，实际 $($toolNames.Count)"
}

# ---------- 3. 小工具函数 ----------
# 取 psd1 Win 侧字段（键可能缺失，缺失视为空）
function Get-WinField($win, [string]$name) {
    if ($null -eq $win) { return '' }
    $v = $win[$name]
    if ($null -eq $v) { return '' }
    return [string]$v
}
# 取 New-ToolDef 导出对象的可选属性
function Get-DefProp($def, [string]$name) {
    $p = $def.PSObject.Properties[$name]
    if ($null -eq $p -or $null -eq $p.Value) { return '' }
    return [string]$p.Value
}
# TOML 字符串：含反斜杠（正则/路径）用单引号字面串，其余用双引号基本串
function Format-TomlString([string]$v) {
    if ($v.Contains('\')) {
        if ($v.Contains("'")) { throw "值同时含反斜杠与单引号，无法安全写入 TOML: $v" }
        return "'$v'"
    }
    return '"' + ($v -replace '"', '\"') + '"'
}

# ---------- 4. 逐工具生成 TOML ----------
# psd1 Win 字段到 tools.toml 静态字段的映射（字段契约见 docs\references\R001）
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

$errors = [System.Collections.Generic.List[string]]::new()
$validCategories = @('key', 'agent', 'project', 'base', 'extras')

$sb = [System.Text.StringBuilder]::new()
[void]$sb.AppendLine('# tools.toml - ome 工具名录唯一 pin 源')
[void]$sb.AppendLine('# 托管 26 节由 .tools\import-catalog.ps1 从 ohmypwsh scripts\catalog.psd1（Win 侧）与 helpers.ps1 New-ToolDef 自动生成，勿手改。')
[void]$sb.AppendLine('# 托管之外的本地新增工具节（如 reader）重跑转换时原样保留在尾部。')
[void]$sb.AppendLine('# pin 字段（tag/version/asset/sha256）由 ome update / ome pin 回写；sha256 大写，未 pin 时省略。')
[void]$sb.AppendLine('')

foreach ($name in $toolNames) {
    if (-not $catalog.Contains($name)) { $errors.Add("ToolNames 中的 '$name' 不在 catalog.psd1"); continue }
    $def = $meta.defs.PSObject.Properties[$name]
    if ($null -eq $def) { $errors.Add("ToolNames 中的 '$name' 无 New-ToolDef 定义"); continue }
    $def = $def.Value

    $entry = $catalog[$name]
    $win   = $entry['Win']
    $deploy = ''
    if ($null -ne $entry['Deploy']) { $deploy = [string]$entry['Deploy']['win'] }

    # 静态元数据：New-ToolDef 定义过的字段全量以其为准（对齐 helpers.ps1 Get-EnvLock 的运行时合并：
    # `foreach ($k in $def.Keys) { $d[$k] = $def[$k] }`，含 tooldef 显式置空的字段）；
    # tooldef 未定义的字段才回退 psd1 Win 侧。pin 字段始终取 psd1。
    $category = Get-DefProp $def 'Category'
    $versionPattern = Get-DefProp $def 'VersionPattern'
    $fields = [ordered]@{ category = $category; deploy = $deploy }
    foreach ($m in $staticMap) {
        $defHas = $null -ne $def.PSObject.Properties[$m.Psd1]
        $fields[$m.Key] = if ($defHas) { Get-DefProp $def $m.Psd1 } else { Get-WinField $win $m.Psd1 }
        if ($m.Key -eq 'asset_pattern') { $fields['version_pattern'] = $versionPattern }
    }
    # pin 字段（来自 psd1 Win 侧）
    $pin = [ordered]@{
        tag     = Get-WinField $win 'Tag'
        version = Get-WinField $win 'Version'
        asset   = Get-WinField $win 'Asset'
        sha256  = (Get-WinField $win 'Sha256').ToUpperInvariant()
    }

    # ---------- 5. 校验 ----------
    if ($fields['category'] -notin $validCategories) { $errors.Add("$name`: category 非法 '$($fields['category'])'") }
    if ($deploy -notin @('envroot', 'installer', 'official')) { $errors.Add("$name`: Deploy.win 非法 '$deploy'") }
    if ($deploy -in @('envroot', 'installer')) {
        if ([string]::IsNullOrEmpty($fields['dir'])) { $errors.Add("$name`: $deploy 工具缺 dir") }
        if ([string]::IsNullOrEmpty($fields['exe'])) { $errors.Add("$name`: $deploy 工具缺 exe") }
    }
    if ([string]::IsNullOrEmpty($fields['extract'])) { $errors.Add("$name`: 缺 extract") }
    # 纯 GitHub 源工具（无 cdn 直链/索引）必须有 repo 与 asset_pattern
    $hasCdn = -not [string]::IsNullOrEmpty($fields['cdn_url']) -or -not [string]::IsNullOrEmpty($fields['cdn_index_url'])
    if (-not $hasCdn) {
        if ([string]::IsNullOrEmpty($fields['repo'])) { $errors.Add("$name`: github 源工具缺 repo") }
        if ([string]::IsNullOrEmpty($fields['asset_pattern'])) { $errors.Add("$name`: github 源工具缺 asset_pattern") }
    }

    # ---------- 6. 写节：静态元数据在前、pin 字段在后；空值整行省略 ----------
    [void]$sb.AppendLine("[tools.$name]")
    foreach ($k in $fields.Keys) {
        $v = $fields[$k]
        if ([string]::IsNullOrEmpty($v)) { continue }
        [void]$sb.AppendLine("$k = $(Format-TomlString $v)")
    }
    foreach ($k in $pin.Keys) {
        $v = $pin[$k]
        if ([string]::IsNullOrEmpty($v)) { continue }
        [void]$sb.AppendLine("$k = $(Format-TomlString $v)")
    }
    [void]$sb.AppendLine('')
}

if ($errors.Count -gt 0) {
    Write-Host "校验失败：" -ForegroundColor Red
    $errors | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}

# ---------- 6.5 保留本地新增工具节（非 ohmypwsh 托管的 [tools.x] 节原样保留在尾部） ----------
$outPath = Join-Path $OmeRoot 'catalog\tools.toml'
$localNames = @()
if (Test-Path $outPath) {
    $existing = [System.IO.File]::ReadAllText($outPath)
    $sectionRe = [regex]'(?ms)^\[tools\.([^\]]+)\]\r?\n.*?(?=^\[tools\.|\z)'
    foreach ($m in $sectionRe.Matches($existing)) {
        $localName = $m.Groups[1].Value
        if ($toolNames -contains $localName) { continue }
        $localNames += $localName
        [void]$sb.AppendLine($m.Value.TrimEnd())
        [void]$sb.AppendLine('')
    }
}

# ---------- 7. 写文件（UTF-8 无 BOM） ----------
[System.IO.File]::WriteAllText($outPath, $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))
$total = $toolNames.Count + $localNames.Count
Write-Host "已生成 $outPath（托管 $($toolNames.Count) 个 + 本地 $($localNames.Count) 个 = $total 个工具）"

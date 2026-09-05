# Supervised local acceptance recovery, not a public automatic resume command.
# Original campaign ledger, recipe and terminal receipt are never modified.
[CmdletBinding()]
param([ValidateSet('Inspect','Prepare','Uninstall','Install','Finish','Audit')][string]$Mode = 'Inspect')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$runRoot = 'C:\Users\chris\Games\CEBG-Curated-20260905-r5'
$gameRoot = Join-Path $runRoot 'game'
$recoveryRoot = Join-Path $runRoot '.chriz\recoveries\modpack-alpha5-20260906'
$stepRoot = Join-Path $runRoot '.chriz\attempts\attempt-aba3cbb32e1956c690b8\steps\0110-22791e178261b9e3\attempt-0001'
$oldArchiveHash = '16453c1e9d1ff76a1e697426cda33d1b6cac3243d7310f7fda6d90410fca243c'
$oldArchive = 'C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection\sha256\16\' + $oldArchiveHash + '.archive'
$newArchiveHash = '2278c839f60e019bedba355cb794176a052d24b68db2851248af926580840b33'
$newArchive = Join-Path $PSScriptRoot '..\target\alpha9-source-verification\chriz-bg-modpack-v0.2.0-alpha.5.zip'
$toolHash = 'ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a'
$toolPath = Join-Path $gameRoot 'Setup-chriz-bg-modpack.exe'
$requested = @(110,130,140,170,190,192,193,194,195,196,197,198,410,430,440,450)
$installed = @($requested | Where-Object { $_ -notin @(170,192) })
$prefix = @(Get-Content -LiteralPath (Join-Path $stepRoot 'before.log') | Where-Object { $_ -match '^~' })
function Hash([string]$Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
function New-Json([string]$Path, $Value) {
    $bytes = [Text.Encoding]::UTF8.GetBytes(($Value | ConvertTo-Json -Depth 14))
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try { $stream.Write($bytes,0,$bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
}
function Active-Lines([string]$Path) { @(Get-Content -LiteralPath $Path | Where-Object { $_ -match '^~' }) }
function Assert-Stack([int[]]$Tail, [string]$Version) {
    $lines = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log'))
    if ($prefix.Count -ne 383 -or $lines.Count -ne 383 + $Tail.Count) { throw 'Unexpected active component count' }
    for ($i=0; $i -lt $prefix.Count; $i++) {
        if ($lines[$i] -cne $prefix[$i]) { throw "Historical component entry changed at $i" }
    }
    for ($i=0; $i -lt $Tail.Count; $i++) {
        if ($lines[383+$i] -notmatch '^~SETUP-CHRIZ-BG-MODPACK\.TP2~ #0 #(\d+) // .+: (.+)$' -or
            [int]$Matches[1] -ne $Tail[$i] -or $Matches[2] -cne $Version) { throw "Unexpected modpack tail at $i" }
    }
}
function Game-Path([string]$Relative) {
    if ([IO.Path]::IsPathRooted($Relative) -or $Relative -match '(^|[\\/])\.\.([\\/]|$)|:') { throw "Unsafe relative path: $Relative" }
    $candidate = [IO.Path]::GetFullPath((Join-Path $gameRoot $Relative))
    if (-not $candidate.StartsWith($gameRoot + '\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Path escaped game root' }
    $cursor = $candidate
    while ($cursor.Length -ge $gameRoot.Length) {
        if ((Test-Path -LiteralPath $cursor) -and ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Reparse point: $cursor" }
        $cursor = Split-Path -Parent $cursor
    }
    $candidate
}
function Is-Published([string]$Name) {
    $Name -eq 'setup-chriz-bg-modpack.tp2' -or $Name.StartsWith('chriz-bg-modpack/',[StringComparison]::Ordinal)
}
function Check-Package([string]$Archive,[string]$ExpectedHash,[bool]$CheckInstalled) {
    if ((Hash $Archive) -ne $ExpectedHash) { throw 'Package checksum mismatch' }
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        foreach ($entry in $zip.Entries) {
            if ($entry.Name -eq '' -or -not (Is-Published $entry.FullName)) { continue }
            $path = Game-Path $entry.FullName
            if ($CheckInstalled) {
                $entryStream = $entry.Open()
                try { $expected = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($entryStream)).ToLowerInvariant() } finally { $entryStream.Dispose() }
                if ((Hash $path) -ne $expected) { throw "Published source differs: $($entry.FullName)" }
            }
        }
    } finally { $zip.Dispose() }
}
function Assert-Protected {
    $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'intent.json') -Raw | ConvertFrom-Json
    foreach ($item in $intent.protected_files) { if ((Hash $item.path) -ne $item.sha256) { throw "Historical metadata changed: $($item.path)" } }
    if ((Hash (Join-Path $recoveryRoot 'before-state.zip')) -ne $intent.backup_sha256) { throw 'Recovery backup changed' }
}
function Test-SrDerivedFile([string]$Relative, [string]$Archive) {
    # SR main_component mutates these inputs itself. Accept only the exact documented
    # resref substitution / six literal DS appends, not arbitrary changed data files.
    if ($Relative -notmatch '^spell_rev/(shared|sp[^/]+)/[^/]+\.(spl|itm|eff)$' -and $Relative -ne 'spell_rev/lib/ds_sr_extra.2da') { return $false }
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $entry = $zip.GetEntry($Relative)
        if ($null -eq $entry) { return $false }
        $inputStream = $entry.Open(); $memory = [IO.MemoryStream]::new()
        try { $inputStream.CopyTo($memory); $original = $memory.ToArray() } finally { $inputStream.Dispose(); $memory.Dispose() }
    } finally { $zip.Dispose() }
    $current = [IO.File]::ReadAllBytes((Game-Path $Relative))
    if ($Relative -eq 'spell_rev/lib/ds_sr_extra.2da') {
        $expectedLines = @([Text.Encoding]::UTF8.GetString($original).TrimEnd().Replace("`r",'').Split("`n"))
        foreach ($line in (Get-Content -LiteralPath (Game-Path 'spell_rev/components/main_component.tpa'))) {
            if ($line -match '^APPEND_OUTER ~spell_rev/lib/ds_sr_extra\.2da~ ~([^~]+)~') { $expectedLines += $Matches[1] }
        }
        $actualLines = [Text.Encoding]::UTF8.GetString($current).TrimEnd().Replace("`r",'').Split("`n")
        return (($expectedLines -join "`n") -ceq ($actualLines -join "`n"))
    }
    if ($original.Length -ne $current.Length) { return $false }
    $known = @(Get-Content -LiteralPath (Game-Path 'spell_rev/lib/manage_add_spell_references.tpa') | ForEach-Object { if ($_ -match '^\s*(sp[a-z0-9]{5})\s*=>') { $Matches[1] } })
    for ($offset=0; $offset -lt $original.Length; $offset++) {
        if ($original[$offset] -eq $current[$offset]) { continue }
        if ($offset+8 -gt $original.Length) { return $false }
        $oldRef = [Text.Encoding]::ASCII.GetString($original,$offset,8).TrimEnd([char]0)
        $newRef = [Text.Encoding]::ASCII.GetString($current,$offset,8).TrimEnd([char]0)
        if ($oldRef.ToLowerInvariant() -notin $known -or $newRef -cne ('dv' + $oldRef.Substring(2).ToLowerInvariant())) { return $false }
        $offset += 7
    }
    return $true
}
function Run-WeiDU([string]$Stage,[string[]]$Arguments) {
    $outPath = Join-Path $recoveryRoot ($Stage + '.stdout.log')
    $errPath = Join-Path $recoveryRoot ($Stage + '.stderr.log')
    foreach ($suffix in @('stdout.log','stderr.log','intent.json','debug.log','after.log','process.json','verified.json')) {
        if (Test-Path -LiteralPath (Join-Path $recoveryRoot ($Stage + '.' + $suffix))) { throw 'Operation already attempted; inspect its evidence before any retry' }
    }
    $argsAll = @('--language','0','--use-lang','en_US') + $Arguments + @('--no-exit-pause','--skip-at-view','--noautoupdate','--log',('"' + (Join-Path $recoveryRoot ($Stage + '.debug.log')) + '"'))
    New-Json (Join-Path $recoveryRoot ($Stage + '.intent.json')) @{program=$toolPath; tool_sha256=$toolHash; arguments=$argsAll; before_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log')); at=(Get-Date -Format o)}
    $worker = Start-Process -FilePath $toolPath -ArgumentList $argsAll -WorkingDirectory $gameRoot -WindowStyle Hidden -RedirectStandardOutput $outPath -RedirectStandardError $errPath -PassThru
    Write-Output "Started $Stage WeiDU PID $($worker.Id)"
    $worker.WaitForExit()
    $worker.Refresh()
    [IO.File]::Copy((Join-Path $gameRoot 'WeiDU.log'),(Join-Path $recoveryRoot ($Stage + '.after.log')),$false)
    New-Json (Join-Path $recoveryRoot ($Stage + '.process.json')) @{exit_code=$worker.ExitCode; at=(Get-Date -Format o); after_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'))}
    if ($worker.ExitCode -notin @(0,3)) { throw "WeiDU $Stage failed; inspect logs, do not retry blindly" }
}
if ((Hash (Join-Path $runRoot '.chriz\recipe\payload.zip')) -ne 'cb220e5de749a779ec0e87337534fd803c11805375011e4b2856984120b4da34') { throw 'Wrong historical recipe' }
if ((Hash $toolPath) -ne $toolHash) { throw 'Unverified WeiDU executable' }
$writers = @(Get-CimInstance Win32_Process | Where-Object {
    ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($gameRoot + '\',[StringComparison]::OrdinalIgnoreCase)) -or
    ($_.Name -eq 'chriz-bg-install.exe' -and $_.CommandLine -like ('*' + $runRoot + '*'))
})
if ($writers.Count -gt 0) { throw 'A process is using the target game; recovery refused' }
$lockPath = Join-Path $runRoot '.chriz\supervised-recovery.lock'
$lock = $null
if ($Mode -ne 'Inspect') { $lock = [IO.File]::Open($lockPath,[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None) }
try {
    if ($Mode -in @('Inspect','Prepare','Uninstall')) {
        Assert-Stack $installed 'v0.2.0-alpha.1'
        Check-Package $oldArchive $oldArchiveHash $true
        Check-Package $newArchive $newArchiveHash $false
    }
    if ($Mode -eq 'Inspect') { @{eligible=$true; prior_components=383; installed_tail=$installed; replacement_components=$requested} | ConvertTo-Json; return }
    if ($Mode -eq 'Prepare') {
        if (Test-Path -LiteralPath $recoveryRoot) { throw 'Recovery directory already exists' }
        $files = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        $rollbackFiles = @{}
        foreach ($rel in @('WeiDU.log','chitin.key','lang/en_US/dialog.tlk','setup-chriz-bg-modpack.tp2','Setup-chriz-bg-modpack.exe','SETUP-CHRIZ-BG-MODPACK.DEBUG')) { [void]$files.Add((Game-Path $rel)) }
        foreach ($dir in @('chriz-bg-modpack','weidu_external/backup/chriz-bg-modpack')) {
            Get-ChildItem -LiteralPath (Game-Path $dir) -File -Recurse | ForEach-Object { [void]$files.Add($_.FullName) }
        }
        foreach ($id in $installed) {
            $backupDir = Game-Path "weidu_external/backup/chriz-bg-modpack/$id"
            foreach ($kind in @('MOVE','OTHER')) { if ((Get-Item -LiteralPath (Join-Path $backupDir "$kind.$id")).Length -ne 0) { throw 'Non-file uninstall actions require separate review' } }
            foreach ($line in (Get-Content -LiteralPath (Join-Path $backupDir "MAPPINGS.$id"))) {
                $pair = $line -split 'TB#"SPACE"',2
                if ($pair.Count -ne 2 -or -not (Test-Path -LiteralPath (Game-Path $pair[1]))) { throw "Missing mapping backup for $id" }
                if (-not $rollbackFiles.ContainsKey($pair[0].ToLowerInvariant())) { $rollbackFiles[$pair[0].ToLowerInvariant()] = @{relative=$pair[0]; exists=$true; sha256=(Hash (Game-Path $pair[1]))} }
            }
            foreach ($line in (Get-Content -LiteralPath (Join-Path $backupDir "UNINSTALL.$id"))) {
                if ($line.Trim()) {
                    $path = Game-Path $line.Trim(); if (Test-Path -LiteralPath $path) { [void]$files.Add($path) }
                    if (-not $rollbackFiles.ContainsKey($line.Trim().ToLowerInvariant())) { $rollbackFiles[$line.Trim().ToLowerInvariant()] = @{relative=$line.Trim(); exists=$false; sha256=$null} }
                }
            }
        }
        New-Item -ItemType Directory -Path $recoveryRoot | Out-Null
        $zipPath = Join-Path $recoveryRoot 'before-state.zip'
        $zip = [IO.Compression.ZipFile]::Open($zipPath,[IO.Compression.ZipArchiveMode]::Create)
        try { foreach ($path in $files) { [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip,$path,$path.Substring($gameRoot.Length+1).Replace('\','/'),[IO.Compression.CompressionLevel]::Optimal) } } finally { $zip.Dispose() }
        $protected = @((Get-ChildItem -LiteralPath (Join-Path $runRoot '.chriz\ledger') -File).FullName) + @((Join-Path $runRoot '.chriz\recipe\payload.zip'),(Join-Path $runRoot '.chriz\recipe\envelope.json'),(Join-Path $runRoot '.chriz\attempts\terminal-0000000222-277ba8822bcb84fe\receipt.json'))
        New-Json (Join-Path $recoveryRoot 'intent.json') @{schema=1; kind='supervised-top-tail-recovery'; base_install='install-e6325c7c98451ad4901e'; base_recipe='0.1.0-alpha.8'; retained_bardic='v2.9c-balance.2'; old_artifact_sha256=$oldArchiveHash; replacement_artifact_sha256=$newArchiveHash; tool_sha256=$toolHash; requested=$requested; installed_tail=$installed; prior_components=383; backup_sha256=(Hash $zipPath); backup_files=$files.Count; rollback_files=@($rollbackFiles.Values); protected_files=@($protected | ForEach-Object { @{path=$_;sha256=(Hash $_)} }); at=(Get-Date -Format o)}
        @{backup_files=$files.Count; backup_bytes=(Get-Item -LiteralPath $zipPath).Length; path=$recoveryRoot} | ConvertTo-Json
        return
    }
    Assert-Protected
    if ($Mode -eq 'Uninstall') {
        $reverse = @($installed); [array]::Reverse($reverse)
        Run-WeiDU 'uninstall' (@('--force-uninstall-list') + @($reverse | ForEach-Object { [string]$_ }))
        Assert-Stack @() ''
        $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'intent.json') -Raw | ConvertFrom-Json
        foreach ($item in $intent.rollback_files) {
            $path = Game-Path $item.relative
            if ($item.exists) { if ((Hash $path) -ne $item.sha256) { throw "Rollback bytes differ: $($item.relative)" } }
            elseif (Test-Path -LiteralPath $path) { throw "Rollback left a newly-created file: $($item.relative)" }
        }
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'uninstall.verified.json') @{active_components=383; prefix_restored=$true; at=(Get-Date -Format o)}
        return
    }
    if ($Mode -eq 'Install') {
        Assert-Stack @() ''
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'uninstall.verified.json'))) { throw 'Uninstall has not been verified' }
        Check-Package $oldArchive $oldArchiveHash $true
        Check-Package $newArchive $newArchiveHash $false
        $oldZip = [IO.Compression.ZipFile]::OpenRead($oldArchive)
        try { $oldNames = @($oldZip.Entries | Where-Object { $_.Name -ne '' -and (Is-Published $_.FullName) } | ForEach-Object FullName) } finally { $oldZip.Dispose() }
        $zip = [IO.Compression.ZipFile]::OpenRead($newArchive)
        try {
            foreach ($entry in $zip.Entries) {
                if ($entry.Name -ne '' -and (Is-Published $entry.FullName) -and $entry.FullName -notin $oldNames -and (Test-Path -LiteralPath (Game-Path $entry.FullName))) { throw "New package file collides with existing file: $($entry.FullName)" }
            }
            foreach ($entry in $zip.Entries) {
                if ($entry.Name -eq '' -or -not (Is-Published $entry.FullName)) { continue }
                $destination = Game-Path $entry.FullName
                [IO.Directory]::CreateDirectory((Split-Path -Parent $destination)) | Out-Null
                [IO.Compression.ZipFileExtensions]::ExtractToFile($entry,$destination,$true)
            }
        } finally { $zip.Dispose() }
        Check-Package $newArchive $newArchiveHash $true
        Run-WeiDU 'install' (@('--force-install-list') + @($requested | ForEach-Object { [string]$_ }) + @('--safe-exit'))
        Assert-Stack $requested 'v0.2.0-alpha.5'
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'install.verified.json') @{active_components=399; prior_383_preserved=$true; components=$requested; modpack_version='0.2.0-alpha.5'; managed_campaign_complete=$false; at=(Get-Date -Format o)}
        Write-Output 'Modpack recovered: 16/16 installed; original 383 entries preserved. Managed campaign remains historical/terminal.'
    }
    if ($Mode -eq 'Finish') {
        Assert-Stack $requested 'v0.2.0-alpha.5'
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'install.verified.json'))) { throw 'Modpack repair not verified' }
        $frozen = Get-Content -LiteralPath (Join-Path $runRoot '.chriz\recipe\payload.zip') -Raw | ConvertFrom-Json
        $runs = @($frozen.plan.runs | Select-Object -Last 3)
        if (($runs.run_id -join ',') -cne 'cdtweaks-spell-save-penalties-bg2,spell-rev-npc-spellbooks-bg2,buffbot-bg2') { throw 'Unexpected remaining runs' }
        $publication = Get-Content -LiteralPath (Game-Path '.chriz-bg-collection/publication-manifest.json') -Raw | ConvertFrom-Json
        $derivedEvidence = @()
        $srHash = $frozen.artifacts.'spell-rev-4.21-chriz.3'.source.sha256
        $srArchive = 'C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection\sha256\' + $srHash.Substring(0,2) + '\' + $srHash + '.archive'
        if ((Hash $srArchive) -ne $srHash) { throw 'SR cache identity changed' }
        foreach ($run in $runs) {
            if ($run.args.Count -ne 0 -or $run.prompt_scripts.Count -ne 0) { throw 'This supervised continuation supports only these prompt-free runs' }
            foreach ($entry in @($publication.entries | Where-Object owner -EQ $run.artifact_id)) {
                $actualHash = Hash (Game-Path $entry.relative_path)
                if ($actualHash -ne $entry.sha256) {
                    if ($run.mod_id -ne 'spell-rev' -or -not (Test-SrDerivedFile $entry.relative_path $srArchive)) { throw "Remaining source differs beyond a proven installation transformation: $($entry.relative_path)" }
                    $derivedEvidence += @{path=$entry.relative_path; original_sha256=$entry.sha256; observed_sha256=$actualHash; verified_transformation='SR main-component temporary spell references or exact DS appends'}
                }
            }
        }
        New-Json (Join-Path $recoveryRoot 'continuation-source-evidence.json') @{derived_files=$derivedEvidence; other_published_files_unchanged=$true; at=(Get-Date -Format o)}
        foreach ($run in $runs) {
            $mod = $frozen.mods.($run.mod_id)
            $artifact = $frozen.artifacts.($run.artifact_id)
            $archiveHash = $artifact.source.sha256
            $archivePath = 'C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection\sha256\' + $archiveHash.Substring(0,2) + '\' + $archiveHash + '.archive'
            if ((Hash $archivePath) -ne $archiveHash) { throw 'Remaining artifact cache identity changed' }
            if ($mod.language -ne 0 -or $mod.invocation_mode -ne 'setup-name') { throw 'Unexpected remaining invocation mode/language' }
            $toolPath = Game-Path ([IO.Path]::GetFileNameWithoutExtension($mod.tp2) + '.exe')
            if (-not (Test-Path -LiteralPath $toolPath)) { [IO.File]::Copy((Game-Path 'Setup-chriz-bg-modpack.exe'),$toolPath,$false) }
            if ((Hash $toolPath) -ne $toolHash) { throw 'Remaining setup executable is not the pinned tool' }
            $before = @(Active-Lines (Game-Path 'WeiDU.log'))
            Run-WeiDU $run.run_id (@('--force-install-list') + @($run.components | ForEach-Object { [string]$_ }) + @('--safe-exit'))
            $after = @(Active-Lines (Game-Path 'WeiDU.log'))
            if ($after.Count -ne $before.Count + $run.components.Count) { throw 'Unexpected continuation component count' }
            for ($i=0; $i -lt $before.Count; $i++) { if ($before[$i] -cne $after[$i]) { throw 'Continuation changed an earlier active row' } }
            for ($i=0; $i -lt $run.components.Count; $i++) {
                if ($after[$before.Count+$i] -notmatch '^~([^~]+)~ #0 #(\d+) // (.+)$' -or $Matches[1].Replace('\','/') -ine $mod.tp2 -or [int]$Matches[2] -ne $run.components[$i]) { throw 'Continuation tail differs from frozen component order' }
            }
            $stdout = Get-Content -LiteralPath (Join-Path $recoveryRoot ($run.run_id + '.stdout.log'))
            $statuses = @($stdout | Where-Object { $_ -match '^(SUCCESSFULLY INSTALLED|INSTALLED WITH WARNINGS|NOT INSTALLED DUE TO ERRORS|SKIPPING)' })
            if ($statuses.Count -ne $run.components.Count -or @($statuses | Where-Object { $_ -match '^(NOT INSTALLED|SKIPPING)' }).Count -ne 0) { throw 'Continuation terminal status evidence is incomplete or failed' }
            Assert-Protected
            New-Json (Join-Path $recoveryRoot ($run.run_id + '.verified.json')) @{run_id=$run.run_id; components=$run.components; artifact_id=$run.artifact_id; artifact_sha256=$archiveHash; active_components=$after.Count; at=(Get-Date -Format o)}
        }
    }
    if ($Mode -in @('Finish','Audit')) {
        $frozen = Get-Content -LiteralPath (Join-Path $runRoot '.chriz\recipe\payload.zip') -Raw | ConvertFrom-Json
        $bg1Count = @(Active-Lines (Join-Path $runRoot 'bg1\WeiDU.log')).Count
        $bg2Count = @(Active-Lines (Game-Path 'WeiDU.log')).Count
        if ($bg1Count -ne 27 -or $bg2Count -ne 403) { throw 'Final full component counts differ' }
        foreach ($target in @('bg1','bg2')) {
            $logPath = if ($target -eq 'bg1') { Join-Path $runRoot 'bg1\WeiDU.log' } else { Game-Path 'WeiDU.log' }
            $actualRows = @(Active-Lines $logPath)
            $expectedRows = @($frozen.plan.runs | Where-Object target -EQ $target | ForEach-Object {
                $plannedRun = $_; $plannedMod = $frozen.mods.($plannedRun.mod_id)
                foreach ($component in $plannedRun.components) { @{tp2=$plannedMod.tp2.Replace('\','/');language=$plannedMod.language;component=$component} }
            })
            if ($actualRows.Count -ne $expectedRows.Count) { throw 'Final plan/log lengths differ' }
            for ($i=0; $i -lt $expectedRows.Count; $i++) {
                if ($actualRows[$i] -notmatch '^~([^~]+)~ #(\d+) #(\d+) // ' -or $Matches[1].Replace('\','/') -ine $expectedRows[$i].tp2 -or [int]$Matches[2] -ne $expectedRows[$i].language -or [int]$Matches[3] -ne $expectedRows[$i].component) { throw "Final $target component identity/order differs at $i" }
            }
        }
        $recordName = if ($Mode -eq 'Audit') { 'final-plan.verified.json' } else { 'recovered-install.json' }
        New-Json (Join-Path $recoveryRoot $recordName) @{schema=1; kind='supervised-recovered-installation'; base_install='install-e6325c7c98451ad4901e'; base_recipe='0.1.0-alpha.8'; replacement_modpack='0.2.0-alpha.5'; retained_bardic='v2.9c-balance.2'; bg1_components=$bg1Count; bg2_components=$bg2Count; total_components=430; exact_plan_order_verified=$true; bg1_log_sha256=(Hash (Join-Path $runRoot 'bg1\WeiDU.log')); bg2_log_sha256=(Hash (Game-Path 'WeiDU.log')); managed_campaign_complete=$false; gameplay_smoke_tested=$false; at=(Get-Date -Format o)}
        Write-Output 'Supervised installation complete: 430 components. Original managed receipt remains failed; composite managed-recovery integration is pending.'
    }
} finally { if ($null -ne $lock) { $lock.Dispose() } }

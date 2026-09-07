# Supervised, incident-specific recovery for ChrizEasyBG-No-SR-Test.
# The historical ledger, frozen recipe, failed attempt and terminal receipt are immutable.
[CmdletBinding()]
param(
    [ValidateSet('Inspect','Prepare','Uninstall','Install','Finish','Audit')]
    [string]$Mode = 'Inspect',
    [Parameter(Mandatory)]
    [ValidatePattern('^[0-9a-fA-F]{64}$')]
    [string]$ReplacementSha256,
    [Parameter(Mandatory)]
    [ValidateRange(1, 9223372036854775807)]
    [long]$ReplacementLength,
    [string]$ReplacementArchive = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$runRoot = 'C:\Users\chris\Games\ChrizEasyBG-No-SR-Test'
$gameRoot = Join-Path $runRoot 'game'
$recoveryId = 'tempus-v032-20260907'
$recoveryRoot = Join-Path $runRoot ".chriz\recoveries\$recoveryId"
$stepRoot = Join-Path $runRoot '.chriz\attempts\attempt-e6b6f3a5aacc11891051\steps\0106-fab86e09514d1f0a\attempt-0001'
$terminalReceipt = Join-Path $runRoot '.chriz\attempts\terminal-0000000214-224d39cd2de6c8d1\receipt.json'
$oldArchiveHash = '729be99e91f9fa2c9044783bf300998b987011a390f407e8cd0b6d4e9dc507bb'
$oldArchiveLength = 1364012L
$oldArchive = "C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection\sha256\72\$oldArchiveHash.archive"
$replacementVersion = '0.3.2'
$replacementReference = 'v0.3.2'
$replacementUrl = 'https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.3.2/chriz-bg-rebalance-v0.3.2.zip'
$replacementFilename = 'chriz-bg-rebalance-v0.3.2.zip'
$approvedReplacementSha256 = '25480a8e597d316d3cf1799f641971f3b6edb113eea24da7f45a8dd70b0a9ef4'
$approvedReplacementLength = 1369825L
if ([string]::IsNullOrWhiteSpace($ReplacementArchive)) {
    $ReplacementArchive = Join-Path $PSScriptRoot '..\target\alpha15-source-verification\chriz-bg-rebalance-v0.3.2.zip'
}
$ReplacementArchive = [IO.Path]::GetFullPath($ReplacementArchive)
$ReplacementSha256 = $ReplacementSha256.ToLowerInvariant()
if ($ReplacementSha256 -cne $approvedReplacementSha256 -or $ReplacementLength -ne $approvedReplacementLength) {
    throw 'Replacement identity is not the approved public v0.3.2 release'
}
$toolHash = 'ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a'
$toolPath = Join-Path $gameRoot 'Setup-chriz-bg-rebalance.exe'
$requested = @(101,121,400,401,404,405,407,408)
$installed = @(101,121,400,404,405,407,408)
$prefix = @(Get-Content -LiteralPath (Join-Path $stepRoot 'before.log') | Where-Object { $_ -match '^~' })

function Hash([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-DirectPath([string]$Path, [bool]$Directory) {
    $item = Get-Item -LiteralPath $Path -Force
    if (($Directory -and -not $item.PSIsContainer) -or (-not $Directory -and $item.PSIsContainer)) {
        throw "Unexpected path type: $Path"
    }
    $cursor = $item.FullName
    while (-not [string]::IsNullOrEmpty($cursor)) {
        $current = Get-Item -LiteralPath $cursor -Force
        if ($current.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "Reparse point: $cursor"
        }
        $parent = Split-Path -Parent $cursor
        if ($parent -eq $cursor) { break }
        $cursor = $parent
    }
}

function New-Json([string]$Path, $Value) {
    $bytes = [Text.Encoding]::UTF8.GetBytes(($Value | ConvertTo-Json -Depth 16))
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try {
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush($true)
    } finally {
        $stream.Dispose()
    }
}

function Active-Lines([string]$Path) {
    @(Get-Content -LiteralPath $Path | Where-Object { $_ -match '^~' })
}

function Game-Path([string]$Relative) {
    if ([IO.Path]::IsPathRooted($Relative) -or $Relative -match '(^|[\\/])\.\.([\\/]|$)|:') {
        throw "Unsafe relative path: $Relative"
    }
    $candidate = [IO.Path]::GetFullPath((Join-Path $gameRoot $Relative))
    if (-not $candidate.StartsWith($gameRoot + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Path escaped game root'
    }
    $cursor = $candidate
    while ($cursor.Length -ge $gameRoot.Length) {
        if (Test-Path -LiteralPath $cursor) {
            $item = Get-Item -LiteralPath $cursor -Force
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw "Reparse point: $cursor"
            }
        }
        $cursor = Split-Path -Parent $cursor
    }
    $candidate
}

function Is-Published([string]$Name) {
    $Name -ceq 'setup-chriz-bg-rebalance.tp2' -or
        $Name.StartsWith('chriz-bg-rebalance/', [StringComparison]::Ordinal)
}

function Read-PackageEntries([string]$Archive) {
    Assert-DirectPath $Archive $false
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        $result = @()
        foreach ($entry in $zip.Entries) {
            if ($entry.Name -eq '' -or -not (Is-Published $entry.FullName)) { continue }
            if ([IO.Path]::IsPathRooted($entry.FullName) -or $entry.FullName -match '(^|/)\.\.(/|$)|\\|:') {
                throw "Unsafe package path: $($entry.FullName)"
            }
            if (-not $seen.Add($entry.FullName)) {
                throw "Duplicate package path: $($entry.FullName)"
            }
            $result += $entry.FullName
        }
        if ('setup-chriz-bg-rebalance.tp2' -notin $result -or $result.Count -lt 2) {
            throw 'Replacement package is missing its exact published roots'
        }
        @($result)
    } finally {
        $zip.Dispose()
    }
}

function Check-Package([string]$Archive, [string]$ExpectedHash, [long]$ExpectedLength, [bool]$CheckInstalled) {
    Assert-DirectPath $Archive $false
    if ((Get-Item -LiteralPath $Archive).Length -ne $ExpectedLength -or (Hash $Archive) -ne $ExpectedHash) {
        throw 'Package release identity mismatch'
    }
    [void](Read-PackageEntries $Archive)
    if (-not $CheckInstalled) { return }
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        foreach ($entry in $zip.Entries) {
            if ($entry.Name -eq '' -or -not (Is-Published $entry.FullName)) { continue }
            $path = Game-Path $entry.FullName
            Assert-DirectPath $path $false
            $entryStream = $entry.Open()
            try {
                $expected = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($entryStream)).ToLowerInvariant()
            } finally {
                $entryStream.Dispose()
            }
            if ((Hash $path) -ne $expected) {
                throw "Published source differs: $($entry.FullName)"
            }
        }
    } finally {
        $zip.Dispose()
    }
}

function Assert-Stack([int[]]$Tail, [string]$Version) {
    $lines = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log'))
    if ($prefix.Count -ne 372 -or $lines.Count -ne 372 + $Tail.Count) {
        throw 'Unexpected active component count'
    }
    for ($i = 0; $i -lt $prefix.Count; $i++) {
        if ($lines[$i] -cne $prefix[$i]) {
            throw "Historical component entry changed at $i"
        }
    }
    for ($i = 0; $i -lt $Tail.Count; $i++) {
        if ($lines[372 + $i] -notmatch '^~SETUP-CHRIZ-BG-REBALANCE\.TP2~ #0 #(\d+) // .+: (.+)$' -or
            [int]$Matches[1] -ne $Tail[$i] -or $Matches[2] -cne $Version) {
            throw "Unexpected BG Rebalance tail at $i"
        }
    }
}

function Assert-IncidentIdentity {
    Assert-DirectPath $runRoot $true
    Assert-DirectPath $gameRoot $true
    Assert-DirectPath (Join-Path $runRoot '.chriz') $true
    $exact = @{
        '.chriz\recipe\payload.zip' = '38d783a4683933404054478bd3e229ee51b9c5bab0040c48ed3fa7fe86f3e848'
        '.chriz\attempts\attempt-e6b6f3a5aacc11891051\steps\0106-fab86e09514d1f0a\attempt-0001\before.log' = '276e7099180190ffd76ec53015ab36e46ca705c3848837a011fecca3a1597208'
        '.chriz\attempts\attempt-e6b6f3a5aacc11891051\steps\0106-fab86e09514d1f0a\attempt-0001\after.log' = 'd7d7eda47b886c03f5efff59764eddf32745f43621067277bb2a2202f493f699'
        '.chriz\attempts\attempt-e6b6f3a5aacc11891051\steps\0106-fab86e09514d1f0a\attempt-0001\stdout.log' = '574fdd7b9675cbe796aeb47ff408e27c6e1c0554aef2749c8a4a663bdd159485'
        '.chriz\attempts\attempt-e6b6f3a5aacc11891051\steps\0106-fab86e09514d1f0a\attempt-0001\invocation.json' = '4ace06fc58bf8763636692d2b105ba58f9cbce17cc142f96103603c1646c48b4'
        '.chriz\attempts\attempt-e6b6f3a5aacc11891051\steps\0106-fab86e09514d1f0a\attempt-0001\process-result.json' = '400187e390edccdd3f066a9311f0c080fe00dc0531bc2f724b09b89ff3d40b9f'
        '.chriz\attempts\terminal-0000000214-224d39cd2de6c8d1\receipt.json' = 'de25eb148a99ac7105d398da2785615dca4a9ca90e93c79248a70878422a0ff2'
        '.chriz\ledger\0000000214.json' = '18da15c3797e4f0ab05285f72a66b062e901684a46613800a4901486ae6bc265'
    }
    foreach ($pair in $exact.GetEnumerator()) {
        $path = Join-Path $runRoot $pair.Key
        Assert-DirectPath $path $false
        if ((Hash $path) -ne $pair.Value) { throw "Incident evidence changed: $($pair.Key)" }
    }
    $receipt = Get-Content -LiteralPath $terminalReceipt -Raw | ConvertFrom-Json
    if ($receipt.install_id -cne 'install-c9dc8c3e0e0b0a3476e0' -or
        $receipt.attempt_id -cne 'terminal-0000000214-224d39cd2de6c8d1' -or
        $receipt.versions.recipe -cne '0.1.0-alpha.12' -or
        $receipt.outcome.status -cne 'fresh_copy_required' -or
        $receipt.outcome.step_id -cne 'install:chriz-bg-rebalance-bg2') {
        throw 'Terminal receipt is not the approved incident'
    }
    $invocation = Get-Content -LiteralPath (Join-Path $stepRoot 'invocation.json') -Raw | ConvertFrom-Json
    if ($invocation.run_id -cne 'chriz-bg-rebalance-bg2' -or $invocation.attempt -ne 1 -or
        (($invocation.components -join ',') -cne ($requested -join ','))) {
        throw 'Failed invocation is not the approved component request'
    }
    $result = Get-Content -LiteralPath (Join-Path $stepRoot 'process-result.json') -Raw | ConvertFrom-Json
    if ($result.exit_code -ne 2 -or $result.terminal -cne 'exited') {
        throw 'Failed process result changed'
    }
    $frozen = Get-Content -LiteralPath (Join-Path $runRoot '.chriz\recipe\payload.zip') -Raw | ConvertFrom-Json
    $tail = @($frozen.plan.runs | Select-Object -Last 4)
    if (($tail.run_id -join ',') -cne 'chriz-bg-rebalance-bg2,chriz-bg-modpack-bg2,cdtweaks-spell-save-penalties-bg2,buffbot-bg2' -or
        @($tail | Where-Object { $_.target -ne 'bg2' -or $_.args.Count -ne 0 -or $_.prompt_scripts.Count -ne 0 }).Count -ne 0) {
        throw 'Frozen recovery tail is not the approved prompt-free BG2 shape'
    }
    if (($tail[0].components -join ',') -cne ($requested -join ',')) {
        throw 'Frozen replacement component list changed'
    }
}

function Assert-Protected {
    $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'intent.json') -Raw | ConvertFrom-Json
    foreach ($item in $intent.protected_files) {
        Assert-DirectPath $item.path $false
        if ((Hash $item.path) -ne $item.sha256) {
            throw "Historical metadata changed: $($item.path)"
        }
    }
    if ((Hash (Join-Path $recoveryRoot 'before-state.zip')) -ne $intent.backup_sha256) {
        throw 'Scoped recovery backup changed'
    }
    if ($intent.replacement.sha256 -cne $ReplacementSha256 -or
        [long]$intent.replacement.length -ne $ReplacementLength) {
        throw 'Caller-supplied replacement identity differs from prepared intent'
    }
}

function Assert-NoWriters {
    $writers = @(Get-CimInstance Win32_Process | Where-Object {
        ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($gameRoot + '\', [StringComparison]::OrdinalIgnoreCase)) -or
        ($_.CommandLine -and $_.CommandLine.Contains($runRoot, [StringComparison]::OrdinalIgnoreCase))
    })
    if ($writers.Count -gt 0) {
        throw "A process is using the recovery target: $($writers.ProcessId -join ',')"
    }
}

function Run-WeiDU([string]$Stage, [string[]]$Arguments) {
    Assert-NoWriters
    $outPath = Join-Path $recoveryRoot ($Stage + '.stdout.log')
    $errPath = Join-Path $recoveryRoot ($Stage + '.stderr.log')
    foreach ($suffix in @('stdout.log','stderr.log','intent.json','debug.log','after.log','process.json','verified.json')) {
        if (Test-Path -LiteralPath (Join-Path $recoveryRoot ($Stage + '.' + $suffix))) {
            throw "Operation $Stage was already attempted; inspect evidence and do not retry"
        }
    }
    Assert-DirectPath $toolPath $false
    if ((Hash $toolPath) -ne $toolHash) { throw 'Unverified WeiDU executable' }
    $argsAll = @('--language','0','--use-lang','en_US') + $Arguments +
        @('--no-exit-pause','--skip-at-view','--noautoupdate','--log',('"' + (Join-Path $recoveryRoot ($Stage + '.debug.log')) + '"'))
    New-Json (Join-Path $recoveryRoot ($Stage + '.intent.json')) @{
        program = $toolPath
        tool_sha256 = $toolHash
        arguments = $argsAll
        before_log_sha256 = (Hash (Join-Path $gameRoot 'WeiDU.log'))
        at = (Get-Date -Format o)
    }
    $worker = Start-Process -FilePath $toolPath -ArgumentList $argsAll -WorkingDirectory $gameRoot `
        -WindowStyle Hidden -RedirectStandardOutput $outPath -RedirectStandardError $errPath -PassThru
    Write-Output "Started $Stage WeiDU PID $($worker.Id)"
    $worker.WaitForExit()
    $worker.Refresh()
    [IO.File]::Copy((Join-Path $gameRoot 'WeiDU.log'), (Join-Path $recoveryRoot ($Stage + '.after.log')), $false)
    New-Json (Join-Path $recoveryRoot ($Stage + '.process.json')) @{
        exit_code = $worker.ExitCode
        at = (Get-Date -Format o)
        after_log_sha256 = (Hash (Join-Path $gameRoot 'WeiDU.log'))
    }
    if ($worker.ExitCode -ne 0) {
        throw "WeiDU $Stage exited $($worker.ExitCode); preserve evidence and do not retry"
    }
}

function Assert-FinalPlan {
    $frozen = Get-Content -LiteralPath (Join-Path $runRoot '.chriz\recipe\payload.zip') -Raw | ConvertFrom-Json
    $bg1Count = @(Active-Lines (Join-Path $runRoot 'bg1\WeiDU.log')).Count
    $bg2Count = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log')).Count
    if ($bg1Count -ne 27 -or $bg2Count -ne 402) { throw 'Final full component counts differ' }
    foreach ($target in @('bg1','bg2')) {
        $logPath = if ($target -eq 'bg1') { Join-Path $runRoot 'bg1\WeiDU.log' } else { Join-Path $gameRoot 'WeiDU.log' }
        $actualRows = @(Active-Lines $logPath)
        $expectedRows = @($frozen.plan.runs | Where-Object target -EQ $target | ForEach-Object {
            $plannedRun = $_
            $plannedMod = $frozen.mods.($plannedRun.mod_id)
            foreach ($component in $plannedRun.components) {
                @{tp2=$plannedMod.tp2.Replace('\','/'); language=$plannedMod.language; component=$component}
            }
        })
        if ($actualRows.Count -ne $expectedRows.Count) { throw "Final $target plan/log lengths differ" }
        for ($i = 0; $i -lt $expectedRows.Count; $i++) {
            if ($actualRows[$i] -notmatch '^~([^~]+)~ #(\d+) #(\d+) // ' -or
                $Matches[1].Replace('\','/') -ine $expectedRows[$i].tp2 -or
                [int]$Matches[2] -ne $expectedRows[$i].language -or
                [int]$Matches[3] -ne $expectedRows[$i].component) {
                throw "Final $target component identity/order differs at $i"
            }
        }
    }
    @{bg1=$bg1Count; bg2=$bg2Count}
}

Assert-IncidentIdentity
Check-Package $ReplacementArchive $ReplacementSha256 $ReplacementLength $false
Assert-DirectPath (Join-Path $runRoot '.chriz\ledger.lock') $false
$ledgerLock = [IO.File]::Open((Join-Path $runRoot '.chriz\ledger.lock'), [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
$recoveryLock = $null
try {
    if ($Mode -ne 'Inspect') {
        $recoveryLock = [IO.File]::Open((Join-Path $runRoot '.chriz\supervised-recovery.lock'), [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    }
    Assert-NoWriters

    if ($Mode -in @('Inspect','Prepare','Uninstall')) {
        Assert-Stack $installed 'v0.3.1'
        Check-Package $oldArchive $oldArchiveHash $oldArchiveLength $true
    } elseif ($Mode -eq 'Install') {
        Check-Package $oldArchive $oldArchiveHash $oldArchiveLength $true
    } else {
        Check-Package $ReplacementArchive $ReplacementSha256 $ReplacementLength $true
    }
    if ($Mode -eq 'Inspect') {
        @{
            eligible = $true
            recovery_id = $recoveryId
            prior_components = 372
            installed_tail = $installed
            replacement_components = $requested
            remaining_runs = @('chriz-bg-modpack-bg2','cdtweaks-spell-save-penalties-bg2','buffbot-bg2')
            final_components = @{bg1=27; bg2=402; total=429}
            replacement = @{version=$replacementVersion; sha256=$ReplacementSha256; length=$ReplacementLength}
        } | ConvertTo-Json -Depth 5
        return
    }

    if ($Mode -eq 'Prepare') {
        if (Test-Path -LiteralPath $recoveryRoot) { throw 'Recovery directory already exists' }
        $files = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
        $rollbackFiles = @{}
        foreach ($rel in @('WeiDU.log','chitin.key','lang/en_US/dialog.tlk','setup-chriz-bg-rebalance.tp2','Setup-chriz-bg-rebalance.exe','SETUP-CHRIZ-BG-REBALANCE.DEBUG','.chriz-bg-collection/publication-manifest.json')) {
            $path = Game-Path $rel
            Assert-DirectPath $path $false
            [void]$files.Add($path)
        }
        foreach ($dir in @('chriz-bg-rebalance','weidu_external/backup/chriz-bg-rebalance')) {
            $dirPath = Game-Path $dir
            Assert-DirectPath $dirPath $true
            Get-ChildItem -LiteralPath $dirPath -File -Recurse | ForEach-Object {
                Assert-DirectPath $_.FullName $false
                [void]$files.Add($_.FullName)
            }
        }
        foreach ($id in $installed) {
            $backupDir = Game-Path "weidu_external/backup/chriz-bg-rebalance/$id"
            foreach ($kind in @('MOVE','OTHER')) {
                if ((Get-Item -LiteralPath (Join-Path $backupDir "$kind.$id")).Length -ne 0) {
                    throw "Component $id has non-file uninstall actions requiring separate review"
                }
            }
            foreach ($line in (Get-Content -LiteralPath (Join-Path $backupDir "MAPPINGS.$id"))) {
                $pair = $line -split 'TB#"SPACE"', 2
                if ($pair.Count -ne 2) { throw "Malformed mapping backup for $id" }
                $backup = Game-Path $pair[1]
                Assert-DirectPath $backup $false
                if (-not $rollbackFiles.ContainsKey($pair[0].ToLowerInvariant())) {
                    $rollbackFiles[$pair[0].ToLowerInvariant()] = @{relative=$pair[0]; exists=$true; sha256=(Hash $backup)}
                }
            }
            foreach ($line in (Get-Content -LiteralPath (Join-Path $backupDir "UNINSTALL.$id"))) {
                if (-not $line.Trim()) { continue }
                $path = Game-Path $line.Trim()
                if (Test-Path -LiteralPath $path) {
                    Assert-DirectPath $path $false
                    [void]$files.Add($path)
                }
                if (-not $rollbackFiles.ContainsKey($line.Trim().ToLowerInvariant())) {
                    $rollbackFiles[$line.Trim().ToLowerInvariant()] = @{relative=$line.Trim(); exists=$false; sha256=$null}
                }
            }
        }
        New-Item -ItemType Directory -Path $recoveryRoot | Out-Null
        Assert-DirectPath $recoveryRoot $true
        $zipPath = Join-Path $recoveryRoot 'before-state.zip'
        $zip = [IO.Compression.ZipFile]::Open($zipPath, [IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($path in @($files | Sort-Object)) {
                [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
                    $zip, $path, $path.Substring($gameRoot.Length + 1).Replace('\','/'),
                    [IO.Compression.CompressionLevel]::Optimal)
            }
        } finally {
            $zip.Dispose()
        }
        $protectedRoots = @(
            (Join-Path $runRoot '.chriz\ledger'),
            (Join-Path $runRoot '.chriz\recipe'),
            $stepRoot
        )
        $protected = @($protectedRoots | ForEach-Object {
            Assert-DirectPath $_ $true
            Get-ChildItem -LiteralPath $_ -File -Recurse
        }) + @(Get-Item -LiteralPath $terminalReceipt)
        $protected = @($protected | Sort-Object FullName -Unique)
        New-Json (Join-Path $recoveryRoot 'intent.json') @{
            schema = 1
            kind = 'supervised-top-tail-recovery'
            recovery_id = $recoveryId
            base_install = 'install-c9dc8c3e0e0b0a3476e0'
            base_recipe = '0.1.0-alpha.12'
            failed_run = 'chriz-bg-rebalance-bg2'
            old_artifact = @{id='chriz-bg-rebalance-0.3.1'; version='0.3.1'; sha256=$oldArchiveHash; length=$oldArchiveLength}
            replacement = @{id='chriz-bg-rebalance-0.3.1'; version=$replacementVersion; reference=$replacementReference; url=$replacementUrl; expected_filename=$replacementFilename; sha256=$ReplacementSha256; length=$ReplacementLength}
            tool_sha256 = $toolHash
            requested = $requested
            installed_tail = $installed
            prior_components = 372
            backup_sha256 = (Hash $zipPath)
            backup_files = $files.Count
            rollback_files = @($rollbackFiles.Values | Sort-Object relative)
            protected_files = @($protected | ForEach-Object { Assert-DirectPath $_.FullName $false; @{path=$_.FullName; sha256=(Hash $_.FullName)} })
            at = (Get-Date -Format o)
        }
        @{backup_files=$files.Count; backup_bytes=(Get-Item -LiteralPath $zipPath).Length; path=$recoveryRoot} | ConvertTo-Json
        return
    }

    Assert-Protected
    if ($Mode -eq 'Uninstall') {
        $reverse = @($installed)
        [array]::Reverse($reverse)
        Run-WeiDU 'uninstall' (@('--force-uninstall-list') + @($reverse | ForEach-Object { [string]$_ }))
        Assert-Stack @() ''
        $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'intent.json') -Raw | ConvertFrom-Json
        foreach ($item in $intent.rollback_files) {
            $path = Game-Path $item.relative
            if ($item.exists) {
                Assert-DirectPath $path $false
                if ((Hash $path) -ne $item.sha256) { throw "Rollback bytes differ: $($item.relative)" }
            } elseif (Test-Path -LiteralPath $path) {
                throw "Rollback left a newly-created file: $($item.relative)"
            }
        }
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'uninstall.verified.json') @{active_components=372; prefix_restored=$true; rollback_bytes_verified=$true; at=(Get-Date -Format o)}
        return
    }

    if ($Mode -eq 'Install') {
        Assert-Stack @() ''
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'uninstall.verified.json'))) {
            throw 'Uninstall has not been byte-verified'
        }
        Check-Package $oldArchive $oldArchiveHash $oldArchiveLength $true
        Check-Package $ReplacementArchive $ReplacementSha256 $ReplacementLength $false
        $oldNames = @(Read-PackageEntries $oldArchive)
        $newNames = @(Read-PackageEntries $ReplacementArchive)
        foreach ($oldName in $oldNames) {
            if ($oldName -notin $newNames) { throw "Replacement omits old published file: $oldName" }
        }
        foreach ($newName in $newNames) {
            if ($newName -notin $oldNames -and (Test-Path -LiteralPath (Game-Path $newName))) {
                throw "New package file collides with existing file: $newName"
            }
        }
        $zip = [IO.Compression.ZipFile]::OpenRead($ReplacementArchive)
        try {
            foreach ($entry in $zip.Entries) {
                if ($entry.Name -eq '' -or -not (Is-Published $entry.FullName)) { continue }
                $destination = Game-Path $entry.FullName
                [IO.Directory]::CreateDirectory((Split-Path -Parent $destination)) | Out-Null
                [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $destination, $true)
            }
        } finally {
            $zip.Dispose()
        }
        Check-Package $ReplacementArchive $ReplacementSha256 $ReplacementLength $true
        Run-WeiDU 'install' (@('--force-install-list') + @($requested | ForEach-Object { [string]$_ }) + @('--safe-exit'))
        Assert-Stack $requested 'v0.3.2'
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'install.verified.json') @{
            active_components=380
            prior_372_preserved=$true
            components=$requested
            replacement_version=$replacementVersion
            replacement_sha256=$ReplacementSha256
            managed_campaign_complete=$false
            at=(Get-Date -Format o)
        }
        Write-Output 'BG Rebalance recovered: 8/8 installed; original 372 rows preserved. Three frozen runs remain.'
        return
    }

    if ($Mode -eq 'Finish') {
        Assert-Stack $requested 'v0.3.2'
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'install.verified.json'))) {
            throw 'BG Rebalance repair is not verified'
        }
        $frozen = Get-Content -LiteralPath (Join-Path $runRoot '.chriz\recipe\payload.zip') -Raw | ConvertFrom-Json
        $runs = @($frozen.plan.runs | Select-Object -Last 3)
        if (($runs.run_id -join ',') -cne 'chriz-bg-modpack-bg2,cdtweaks-spell-save-penalties-bg2,buffbot-bg2') {
            throw 'Unexpected remaining runs'
        }
        $publicationPath = Game-Path '.chriz-bg-collection/publication-manifest.json'
        Assert-DirectPath $publicationPath $false
        $publication = Get-Content -LiteralPath $publicationPath -Raw | ConvertFrom-Json
        foreach ($run in $runs) {
            if ($run.target -ne 'bg2' -or $run.args.Count -ne 0 -or $run.prompt_scripts.Count -ne 0) {
                throw 'Supervised continuation supports only the exact prompt-free BG2 tail'
            }
            $mod = $frozen.mods.($run.mod_id)
            $artifact = $frozen.artifacts.($run.artifact_id)
            $archiveHash = $artifact.source.sha256
            $archivePath = "C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection\sha256\$($archiveHash.Substring(0,2))\$archiveHash.archive"
            Assert-DirectPath $archivePath $false
            if ((Get-Item -LiteralPath $archivePath).Length -ne $artifact.source.expected_length -or (Hash $archivePath) -ne $archiveHash) {
                throw "Remaining artifact identity changed: $($run.artifact_id)"
            }
            foreach ($entry in @($publication.entries | Where-Object owner -EQ $run.artifact_id)) {
                $published = Game-Path $entry.relative_path
                Assert-DirectPath $published $false
                if ((Hash $published) -ne $entry.sha256) {
                    throw "Remaining published source changed: $($entry.relative_path)"
                }
            }
            if ($mod.language -ne 0 -or $mod.invocation_mode -ne 'setup-name') {
                throw 'Unexpected remaining invocation mode or language'
            }
        }
        New-Json (Join-Path $recoveryRoot 'continuation-source-evidence.json') @{remaining_runs=$runs.run_id; all_published_files_unchanged=$true; at=(Get-Date -Format o)}
        foreach ($run in $runs) {
            $mod = $frozen.mods.($run.mod_id)
            $artifact = $frozen.artifacts.($run.artifact_id)
            $toolPath = Game-Path ([IO.Path]::GetFileNameWithoutExtension($mod.tp2) + '.exe')
            if (-not (Test-Path -LiteralPath $toolPath)) {
                [IO.File]::Copy((Game-Path 'Setup-chriz-bg-rebalance.exe'), $toolPath, $false)
            }
            Assert-DirectPath $toolPath $false
            if ((Hash $toolPath) -ne $toolHash) { throw 'Remaining setup executable is not pinned WeiDU' }
            $before = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log'))
            Run-WeiDU $run.run_id (@('--force-install-list') + @($run.components | ForEach-Object { [string]$_ }) + @('--safe-exit'))
            $after = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log'))
            if ($after.Count -ne $before.Count + $run.components.Count) { throw 'Unexpected continuation component count' }
            for ($i = 0; $i -lt $before.Count; $i++) {
                if ($before[$i] -cne $after[$i]) { throw 'Continuation changed an earlier active row' }
            }
            for ($i = 0; $i -lt $run.components.Count; $i++) {
                if ($after[$before.Count + $i] -notmatch '^~([^~]+)~ #(\d+) #(\d+) // ' -or
                    $Matches[1].Replace('\','/') -ine $mod.tp2.Replace('\','/') -or
                    [int]$Matches[2] -ne $mod.language -or [int]$Matches[3] -ne $run.components[$i]) {
                    throw 'Continuation tail differs from frozen component order'
                }
            }
            $stdout = Get-Content -LiteralPath (Join-Path $recoveryRoot ($run.run_id + '.stdout.log'))
            $statuses = @($stdout | Where-Object { $_ -match '^(SUCCESSFULLY INSTALLED|INSTALLED WITH WARNINGS|NOT INSTALLED DUE TO ERRORS|SKIPPING)' })
            if ($statuses.Count -ne $run.components.Count -or
                @($statuses | Where-Object { $_ -match '^(NOT INSTALLED|SKIPPING)' }).Count -ne 0) {
                throw 'Continuation terminal status evidence is incomplete or failed'
            }
            Assert-Protected
            New-Json (Join-Path $recoveryRoot ($run.run_id + '.verified.json')) @{
                run_id=$run.run_id
                components=$run.components
                artifact_id=$run.artifact_id
                artifact_sha256=$artifact.source.sha256
                active_components=$after.Count
                at=(Get-Date -Format o)
            }
        }
        $counts = Assert-FinalPlan
        New-Json (Join-Path $recoveryRoot 'recovered-install.json') @{
            schema=1
            kind='supervised-recovered-installation'
            base_install='install-c9dc8c3e0e0b0a3476e0'
            base_recipe='0.1.0-alpha.12'
            replacement_bg_rebalance=$replacementVersion
            replacement_sha256=$ReplacementSha256
            bg1_components=$counts.bg1
            bg2_components=$counts.bg2
            total_components=429
            exact_plan_order_verified=$true
            bg1_log_sha256=(Hash (Join-Path $runRoot 'bg1\WeiDU.log'))
            bg2_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'))
            managed_campaign_complete=$false
            gameplay_smoke_tested=$false
            at=(Get-Date -Format o)
        }
        Write-Output 'Supervised installation tail complete: 429 components. Run the read-only Rust acceptance adapter next.'
        return
    }

    if ($Mode -eq 'Audit') {
        Assert-Protected
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'recovered-install.json'))) {
            throw 'Finish evidence is absent'
        }
        $counts = Assert-FinalPlan
        @{
            recovery_id=$recoveryId
            exact_plan_order_verified=$true
            bg1_components=$counts.bg1
            bg2_components=$counts.bg2
            total_components=429
            historical_campaign='fresh_copy_required'
            recovery_receipt_published=(Test-Path -LiteralPath (Join-Path $runRoot '.chriz\install-receipt.json'))
            gameplay_smoke_tested=$false
        } | ConvertTo-Json
    }
} finally {
    if ($null -ne $recoveryLock) { $recoveryLock.Dispose() }
    $ledgerLock.Dispose()
}

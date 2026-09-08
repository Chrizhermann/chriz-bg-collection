# Incident-specific append-only recovery for Combined-20260908.
# The original campaign ledger, frozen recipe, failed attempt and terminal receipt remain immutable.
[CmdletBinding()]
param(
    [ValidateSet('Inspect','Prepare','FinishPreparation','Install256','FinishTail','Audit')]
    [string]$Mode = 'Inspect'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$runRoot = 'C:\Users\chris\CEBG-Tests\Combined-20260908'
$gameRoot = Join-Path $runRoot 'game'
$recoveryId = 'sod-256-append-20260909'
$recoveryRoot = Join-Path $runRoot ".chriz\recoveries\$recoveryId"
$failedStep = Join-Path $runRoot '.chriz\attempts\attempt-656080e90d82e0a12f12\steps\0116-c656b8d09bdc1ad9\attempt-0001'
$terminalReceipt = Join-Path $runRoot '.chriz\attempts\terminal-0000000234-19d855a4d1b269e8\receipt.json'
$frozenRecipe = Join-Path $runRoot '.chriz\recipe\payload.zip'
$publicationManifest = Join-Path $gameRoot '.chriz-bg-collection\publication-manifest.json'
$sourceArchive = Join-Path $PSScriptRoot '..\target\combined-playtest-20260908\sources\chriz-sod-remix-20260908.zip'
$fixedRoot = Join-Path $PSScriptRoot '..\target\combined-playtest-20260908\recovery-20260909'
$fixedArchive = Join-Path $fixedRoot 'chriz-sod-remix-29e123a-recovery-20260909.zip'
$sourceEvidence = Join-Path $fixedRoot 'owner-handoff-manifest.json'
$compatibilityEvidence = Join-Path $fixedRoot 'owner-fixed-complete-report.json'
$provenanceEvidence = Join-Path $fixedRoot 'source-provenance.json'
$sourceRelative = 'chriz-sod-remix/lib/comp256_creatures.tpa'
$sourcePath = Join-Path $gameRoot ($sourceRelative -replace '/', '\')

$toolHash = 'ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a'
$recipeHash = '40712587d45b8361d9be1393bc5125a851dbd358b44a7a7aff620af822868a3d'
$terminalHash = '710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938'
$oldArchiveHash = '80210e083ff3c6c5644369b0e04961acdec63e13a8eceacd9208829d7392016d'
$oldArchiveLength = 557969L
$fixedArchiveHash = '453906a1157bdccfcf4778eafe86c891061bb5acf4a2f254afda94ec5b44fdeb'
$fixedArchiveLength = 558607L
$oldSourceHash = 'c450c2e6ac96555a5f41376b5ccb7356eee7f93371d835e0455d45dda7f68b5f'
$fixedSourceHash = 'b730bae065bc1d47fcadd117eb5eefe1438afb02831c94a6eac38a9a8340eeca'
$sourceCommit = '29e123a79b9f03334ab88ce93e28c300b287a8e0'
$sourceEvidenceHash = 'dcb7a772951578ee6d07b4b716e7fcd9bf8f67283221076aaabeee954b76c9b7'
$compatibilityEvidenceHash = 'b13efb10c573df8d6234b8c26f65d3d4a8d8236d670dd21def130cfd52ff09fd'
$provenanceEvidenceHash = '960b368e5966fe4d2edc522748cce7b641178cc29d2da626972c6b851d40a890'
$installedSoD = @(100,110,120,130,135,140,150,145,160,170,180,175,185,190,195,210,197,187,200,215,220,225,245,230,240,250,255,260,265,270,280,290,900,910)
$expectedNew = @('B200090.PVRZ','B2000N90.PVRZ','CSR256CM.CRE','CSR256FM.CRE','CSR256G1.CRE','CSR256G2.CRE','CSR26CMA.BCS','CSR26E1G.CRE','CSR26E1L.CRE','CSR26E1S.CRE','CSR26E2L.CRE','CSR26E2S.CRE','CSR26ELW.ITM','CSR26F1G.CRE','CSR26F1L.CRE','CSR26F1S.CRE','CSR26F2L.CRE','CSR26F2S.CRE','CSR26FLW.ITM','CSR26FMA.BCS','CSR26MEL.BCS')
$expectedEdited = @('BD2000.ARE','BD2000.BCS','BD2000.TIS','BD2000N.TIS','BDBWOOSH.ITM','BDPHOSSE.DLG')

function Hash([string]$Path) {
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function New-Json([string]$Path, $Value) {
    $bytes = [Text.Encoding]::UTF8.GetBytes(($Value | ConvertTo-Json -Depth 24))
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    try { $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
}

function Assert-DirectPath([string]$Path, [bool]$Directory) {
    $item = Get-Item -LiteralPath $Path -Force
    if (($Directory -and -not $item.PSIsContainer) -or (-not $Directory -and $item.PSIsContainer)) {
        throw "Unexpected path type: $Path"
    }
    $cursor = $item.FullName
    while (-not [string]::IsNullOrEmpty($cursor)) {
        $current = Get-Item -LiteralPath $cursor -Force
        if ($current.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point: $cursor" }
        $parent = Split-Path -Parent $cursor
        if ($parent -eq $cursor) { break }
        $cursor = $parent
    }
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
        if ((Test-Path -LiteralPath $cursor) -and ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw "Reparse point: $cursor"
        }
        $cursor = Split-Path -Parent $cursor
    }
    $candidate
}

function Active-Lines([string]$Path) {
    @(Get-Content -LiteralPath $Path | Where-Object { $_ -match '^~' })
}

function Assert-NoWriters {
    $writers = @(Get-CimInstance Win32_Process | Where-Object {
        ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($gameRoot + '\', [StringComparison]::OrdinalIgnoreCase)) -or
        ($_.CommandLine -and $_.CommandLine.Contains($runRoot, [StringComparison]::OrdinalIgnoreCase))
    })
    if ($writers.Count -gt 0) { throw "A process is using the recovery target: $($writers.ProcessId -join ',')" }
}

function Assert-ExactFile([string]$Path, [string]$ExpectedHash, [Nullable[long]]$ExpectedLength = $null) {
    Assert-DirectPath $Path $false
    if ($null -ne $ExpectedLength -and (Get-Item -LiteralPath $Path).Length -ne $ExpectedLength) {
        throw "Unexpected length: $Path"
    }
    if ((Hash $Path) -cne $ExpectedHash) { throw "Unexpected SHA256: $Path" }
}

function Assert-IncidentIdentity {
    Assert-DirectPath $runRoot $true
    Assert-DirectPath $gameRoot $true
    Assert-ExactFile $frozenRecipe $recipeHash
    Assert-ExactFile $terminalReceipt $terminalHash
    $exact = @{
        'before.log' = '491854354260eeb3d59f8349509dd56b7b15f311e706435bc35c144d019914b4'
        'after.log' = '29b5ba0e0338ae27db5fc1598f3af098f4f67864ce68999cd1f456e2a762239d'
        'stdout.log' = 'd231a034a82492ef75dc88c846f63f86648926438577e17b8733b22c549482b4'
        'stderr.log' = 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
        'weidu.debug.log' = 'a1cfeb5c5dfae32b30496244c1afd15d82b23de381c76402819cbd91512e83ff'
        'invocation.json' = 'e00c2670f55efda746ab7b876917a1b6996ed8cbaeb0736839e57cb30af605ea'
        'process-result.json' = 'd3075128762955b6a0e75b56db8196b07a17350d8add3402d9706464414eab4d'
    }
    foreach ($pair in $exact.GetEnumerator()) { Assert-ExactFile (Join-Path $failedStep $pair.Key) $pair.Value }
    $receipt = Get-Content -LiteralPath $terminalReceipt -Raw | ConvertFrom-Json
    if ($receipt.install_id -cne 'install-d976d775d15d76c8bdc0' -or
        $receipt.attempt_id -cne 'terminal-0000000234-19d855a4d1b269e8' -or
        $receipt.versions.recipe -cne 'local-40712587d45b' -or
        $receipt.outcome.status -cne 'fresh_copy_required' -or
        $receipt.outcome.step_id -cne 'install:chriz-sod-remix-bg2') {
        throw 'Terminal receipt is not the approved incident'
    }
    $invocation = Get-Content -LiteralPath (Join-Path $failedStep 'invocation.json') -Raw | ConvertFrom-Json
    $requested = @($installedSoD[0..26]) + @(256) + @($installedSoD[27..33])
    if ($invocation.run_id -cne 'chriz-sod-remix-bg2' -or $invocation.attempt -ne 1 -or
        (($invocation.components -join ',') -cne ($requested -join ','))) {
        throw 'Failed invocation is not the approved SoD request'
    }
    $result = Get-Content -LiteralPath (Join-Path $failedStep 'process-result.json') -Raw | ConvertFrom-Json
    if ($result.exit_code -ne 2 -or $result.terminal -cne 'exited') { throw 'Failed process result changed' }
}

function Get-Frozen {
    Get-Content -LiteralPath $frozenRecipe -Raw | ConvertFrom-Json
}

function Assert-Stack([int]$CompletedTailRuns) {
    $base = @(Active-Lines (Join-Path $failedStep 'after.log'))
    if ($base.Count -ne 364) { throw 'Frozen partial BG2 log no longer has 364 rows' }
    $actual = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log'))
    $expected = [Collections.Generic.List[object]]::new()
    foreach ($line in $base) { $expected.Add($line) }
    if ($CompletedTailRuns -ge 0) {
        $expected.Add('component-256-placeholder')
    }
    if ($CompletedTailRuns -gt 0) {
        $frozen = Get-Frozen
        foreach ($run in @($frozen.plan.runs | Select-Object -Last 10 | Select-Object -First $CompletedTailRuns)) {
            $mod = $frozen.mods.($run.mod_id)
            foreach ($component in $run.components) {
                $expected.Add([pscustomobject]@{ tp2=$mod.tp2; language=[int]$mod.language; component=[int]$component })
            }
        }
    }
    if ($actual.Count -ne $expected.Count) { throw "Unexpected BG2 component count: $($actual.Count), expected $($expected.Count)" }
    for ($i=0; $i -lt $base.Count; $i++) {
        if ($actual[$i] -cne $base[$i]) { throw "Historical component row changed at $i" }
    }
    if ($CompletedTailRuns -ge 0) {
        if ($actual[364] -notmatch '^~([^~]+)~ #(\d+) #(\d+) // ' -or
            $Matches[1].Replace('\','/') -ine 'chriz-sod-remix/setup-chriz-sod-remix.tp2' -or
            [int]$Matches[2] -ne 0 -or [int]$Matches[3] -ne 256) {
            throw 'The append-only component 256 row is missing or changed'
        }
    }
    for ($i=365; $i -lt $expected.Count; $i++) {
        $row = $expected[$i]
        if ($actual[$i] -notmatch '^~([^~]+)~ #(\d+) #(\d+) // ' -or
            $Matches[1].Replace('\','/') -ine $row.tp2.Replace('\','/') -or
            [int]$Matches[2] -ne $row.language -or [int]$Matches[3] -ne $row.component) {
            throw "Continuation component identity/order differs at $i"
        }
    }
}

function Assert-SourceArchives {
    Assert-ExactFile $sourceArchive $oldArchiveHash $oldArchiveLength
    Assert-ExactFile $fixedArchive $fixedArchiveHash $fixedArchiveLength
    Assert-ExactFile $sourceEvidence $sourceEvidenceHash
    Assert-ExactFile $compatibilityEvidence $compatibilityEvidenceHash
    Assert-ExactFile $provenanceEvidence $provenanceEvidenceHash
    $oldZip = [IO.Compression.ZipFile]::OpenRead([IO.Path]::GetFullPath($sourceArchive))
    $newZip = [IO.Compression.ZipFile]::OpenRead([IO.Path]::GetFullPath($fixedArchive))
    try {
        $old = @{}; $new = @{}
        foreach ($entry in $oldZip.Entries) {
            if ($entry.Name -eq '') { continue }
            if ($old.ContainsKey($entry.FullName)) { throw "Duplicate old archive member: $($entry.FullName)" }
            $old[$entry.FullName] = $entry
        }
        foreach ($entry in $newZip.Entries) {
            if ($entry.Name -eq '') { continue }
            if ($new.ContainsKey($entry.FullName)) { throw "Duplicate fixed archive member: $($entry.FullName)" }
            $new[$entry.FullName] = $entry
        }
        if ($old.Count -ne 129 -or $new.Count -ne 129 -or (($old.Keys | Sort-Object) -join "`n") -cne (($new.Keys | Sort-Object) -join "`n")) {
            throw 'Fixed source archive member set changed'
        }
        $deltas = @()
        foreach ($name in $old.Keys) {
            $oldStream = $old[$name].Open(); $newStream = $new[$name].Open()
            try {
                $oldHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($oldStream)).ToLowerInvariant()
                $newHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($newStream)).ToLowerInvariant()
            } finally { $oldStream.Dispose(); $newStream.Dispose() }
            if ($oldHash -cne $newHash) { $deltas += [pscustomobject]@{path=$name;old=$oldHash;new=$newHash} }
        }
        if ($deltas.Count -ne 1 -or $deltas[0].path -cne $sourceRelative -or
            $deltas[0].old -cne $oldSourceHash -or $deltas[0].new -cne $fixedSourceHash) {
            throw 'Fixed archive is not the approved single-runtime-file delta'
        }
    } finally { $oldZip.Dispose(); $newZip.Dispose() }
}

function Assert-Protected {
    $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'intent.json') -Raw | ConvertFrom-Json
    foreach ($item in $intent.protected_files) {
        Assert-ExactFile $item.path $item.sha256
    }
    Assert-ExactFile (Join-Path $recoveryRoot 'before-state.zip') $intent.backup_sha256
    foreach ($item in $intent.evidence_files) { Assert-ExactFile (Join-Path $recoveryRoot $item.name) $item.sha256 }
}

function Assert-PreparedIntent {
    $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'intent.json') -Raw | ConvertFrom-Json
    if ($intent.kind -cne 'supervised-append-missing-component' -or
        $intent.install_id -cne 'install-d976d775d15d76c8bdc0' -or
        $intent.recovery_id -cne $recoveryId -or $intent.run_id -cne 'chriz-sod-remix-bg2' -or
        $intent.component -ne 256 -or (($intent.installed_components -join ',') -cne ($installedSoD -join ',')) -or
        $intent.source_commit -cne $sourceCommit -or $intent.preserved_active_rows -ne 364 -or
        $intent.new_files -ne 21 -or $intent.edited_files -ne 6 -or $intent.removed_files -ne 0 -or
        $intent.later_sibling_write_overlaps -ne 0 -or $intent.old_source_sha256 -cne $oldSourceHash -or
        $intent.fixed_source_sha256 -cne $fixedSourceHash -or $intent.fixed_archive_sha256 -cne $fixedArchiveHash) {
        throw 'Prepared recovery intent differs from the approved incident'
    }
}

function Find-Archive($Artifact) {
    $hash = [string]$Artifact.source.sha256
    $name = [string]$Artifact.source.expected_filename
    $candidates = @(
        (Join-Path $PSScriptRoot "..\target\combined-playtest-20260908\sources\$name"),
        (Join-Path 'C:\CEBG-creator-full-cache\manual' $name),
        (Join-Path "C:\CEBG-creator-full-cache\sha256\$($hash.Substring(0,2))" "$hash.archive")
    )
    foreach ($path in $candidates) {
        if ((Test-Path -LiteralPath $path) -and (Hash $path) -ceq $hash -and (Get-Item -LiteralPath $path).Length -eq [long]$Artifact.source.expected_length) {
            return [IO.Path]::GetFullPath($path)
        }
    }
    throw "Frozen artifact archive unavailable or changed: $($Artifact.id)"
}

function Test-SrDerivedFile([string]$Relative, [string]$Archive) {
    if ($Relative -notmatch '^spell_rev/(shared|sp[^/]+)/[^/]+\.(spl|itm|eff)$' -and $Relative -ne 'spell_rev/lib/ds_sr_extra.2da') { return $false }
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $entry = $zip.GetEntry($Relative); if ($null -eq $entry) { return $false }
        $stream = $entry.Open(); $memory = [IO.MemoryStream]::new()
        try { $stream.CopyTo($memory); $original = $memory.ToArray() } finally { $stream.Dispose(); $memory.Dispose() }
    } finally { $zip.Dispose() }
    $current = [IO.File]::ReadAllBytes((Game-Path $Relative))
    if ($Relative -eq 'spell_rev/lib/ds_sr_extra.2da') {
        $expected = @([Text.Encoding]::UTF8.GetString($original).TrimEnd().Replace("`r",'').Split("`n"))
        foreach ($line in (Get-Content -LiteralPath (Game-Path 'spell_rev/components/main_component.tpa'))) {
            if ($line -match '^APPEND_OUTER ~spell_rev/lib/ds_sr_extra\.2da~ ~([^~]+)~') { $expected += $Matches[1] }
        }
        return (($expected -join "`n") -ceq ([Text.Encoding]::UTF8.GetString($current).TrimEnd().Replace("`r",'').Split("`n") -join "`n"))
    }
    if ($original.Length -ne $current.Length) { return $false }
    $known = @(Get-Content -LiteralPath (Game-Path 'spell_rev/lib/manage_add_spell_references.tpa') | ForEach-Object { if ($_ -match '^\s*(sp[a-z0-9]{5})\s*=>') { $Matches[1] } })
    for ($offset=0; $offset -lt $original.Length; $offset++) {
        if ($original[$offset] -eq $current[$offset]) { continue }
        if ($offset + 8 -gt $original.Length) { return $false }
        $oldRef = [Text.Encoding]::ASCII.GetString($original,$offset,8).TrimEnd([char]0)
        $newRef = [Text.Encoding]::ASCII.GetString($current,$offset,8).TrimEnd([char]0)
        if ($oldRef.ToLowerInvariant() -notin $known -or $newRef -cne ('dv' + $oldRef.Substring(2).ToLowerInvariant())) { return $false }
        $offset += 7
    }
    return $true
}

function Assert-RunSource($Frozen, $Run) {
    $artifact = $Frozen.artifacts.($Run.artifact_id)
    $archive = Find-Archive $artifact
    $publication = Get-Content -LiteralPath $publicationManifest -Raw | ConvertFrom-Json
    $entries = @($publication.entries | Where-Object owner -CEQ $Run.artifact_id)
    if ($entries.Count -eq 0) { throw "No publication entries for $($Run.artifact_id)" }
    foreach ($entry in $entries) {
        $path = Game-Path $entry.relative_path
        if (-not (Test-Path -LiteralPath $path)) { throw "Published source is missing: $($entry.relative_path)" }
        if ((Hash $path) -cne $entry.sha256) {
            if ($Run.mod_id -cne 'spell-rev' -or -not (Test-SrDerivedFile $entry.relative_path $archive)) {
                throw "Published source differs: $($entry.relative_path)"
            }
        }
    }
    $archive
}

function Run-WeiDU([string]$Stage, [string]$Tool, [string[]]$Arguments) {
    Assert-NoWriters
    foreach ($suffix in @('intent.json','stdout.log','stderr.log','debug.log','after.log','process.json','verified.json')) {
        if (Test-Path -LiteralPath (Join-Path $recoveryRoot "$Stage.$suffix")) {
            throw "Operation $Stage already has evidence; it will not be replayed"
        }
    }
    Assert-ExactFile $Tool $toolHash
    $all = @('--language','0','--use-lang','en_US') + $Arguments + @('--safe-exit','--no-exit-pause','--skip-at-view','--noautoupdate','--log',('"' + (Join-Path $recoveryRoot "$Stage.debug.log") + '"'))
    New-Json (Join-Path $recoveryRoot "$Stage.intent.json") @{program=$Tool;tool_sha256=$toolHash;arguments=$all;before_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'));at=(Get-Date -Format o)}
    $worker = Start-Process -FilePath $Tool -ArgumentList $all -WorkingDirectory $gameRoot -WindowStyle Hidden -RedirectStandardOutput (Join-Path $recoveryRoot "$Stage.stdout.log") -RedirectStandardError (Join-Path $recoveryRoot "$Stage.stderr.log") -PassThru
    Write-Output "Started $Stage WeiDU PID $($worker.Id)"
    $worker.WaitForExit(); $worker.Refresh()
    [IO.File]::Copy((Join-Path $gameRoot 'WeiDU.log'), (Join-Path $recoveryRoot "$Stage.after.log"), $false)
    New-Json (Join-Path $recoveryRoot "$Stage.process.json") @{exit_code=$worker.ExitCode;after_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'));at=(Get-Date -Format o)}
    if ($worker.ExitCode -ne 0) { throw "WeiDU $Stage failed; inspect its evidence and do not retry blindly" }
}

function Assert-Status([string]$Stage, [int]$Count) {
    $statuses = @(Get-Content -LiteralPath (Join-Path $recoveryRoot "$Stage.stdout.log") | Where-Object { $_ -match '^(SUCCESSFULLY INSTALLED|INSTALLED WITH WARNINGS|NOT INSTALLED DUE TO ERRORS|SKIPPING)' })
    if ($statuses.Count -ne $Count -or @($statuses | Where-Object { $_ -match '^(NOT INSTALLED|SKIPPING)' }).Count -ne 0) {
        throw "WeiDU terminal status evidence is incomplete or failed for $Stage"
    }
}

function Assert-HgoNoopEvidence {
    $expected = @{
        'hiddengameplayoptions-bg2.after.log' = 'e3a7ca35589191cdb0d08639c49512e89d5a9e98969e3500b7a79583444dabec'
        'hiddengameplayoptions-bg2.debug.log' = 'ac1983754b186f883e04a4f371cc3a7df408a783226074ce2621e3584dc7c91e'
        'hiddengameplayoptions-bg2.intent.json' = 'e5c0b511d0748f93e1a988256c1ee46e324533cbce021bbc4858e608f924c058'
        'hiddengameplayoptions-bg2.process.json' = 'c6221a52aed629948c14c00981552ab502c1c7f055d988f58b3de496880c6bd1'
        'hiddengameplayoptions-bg2.stderr.log' = 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
        'hiddengameplayoptions-bg2.stdout.log' = 'bb6640040ebe970be5d4e7ccfd6ea5be438b763ee1f5aabc069b8ce74ab3459c'
    }
    foreach ($pair in $expected.GetEnumerator()) { Assert-ExactFile (Join-Path $recoveryRoot $pair.Key) $pair.Value }
    $intent = Get-Content -LiteralPath (Join-Path $recoveryRoot 'hiddengameplayoptions-bg2.intent.json') -Raw | ConvertFrom-Json
    $process = Get-Content -LiteralPath (Join-Path $recoveryRoot 'hiddengameplayoptions-bg2.process.json') -Raw | ConvertFrom-Json
    if ($intent.program -notlike '*\HiddenGameplayOptions.exe' -or $intent.program -like '*\Setup-HiddenGameplayOptions.exe' -or
        $intent.before_log_sha256 -cne $process.after_log_sha256 -or $process.exit_code -ne 0 -or
        (Hash (Join-Path $gameRoot 'WeiDU.log')) -cne $process.after_log_sha256) {
        throw 'The preserved Hidden Gameplay Options no-op is not the reviewed zero-change attempt'
    }
}

Assert-IncidentIdentity
Assert-NoWriters
Assert-SourceArchives
$lock = $null
$lockPath = Join-Path $runRoot '.chriz\supervised-recovery.lock'
if ($Mode -ne 'Inspect') { $lock = [IO.File]::Open($lockPath,[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None) }
try {
    if ($Mode -eq 'Inspect') {
        if (Test-Path -LiteralPath $recoveryRoot) {
            Assert-Protected
            $verified = @(Get-ChildItem -LiteralPath $recoveryRoot -Filter '*.verified.json' -File | Select-Object -ExpandProperty BaseName)
            @{eligible=$true;prepared=$true;verified=$verified;active_bg2=(Active-Lines (Join-Path $gameRoot 'WeiDU.log')).Count} | ConvertTo-Json
        } else {
            Assert-Stack -1
            Assert-ExactFile $sourcePath $oldSourceHash
            @{eligible=$true;prepared=$false;preserved_components=391;missing_component=256;remaining_runs=10} | ConvertTo-Json
        }
        return
    }

    if ($Mode -eq 'Prepare') {
        Assert-Stack -1
        Assert-ExactFile $sourcePath $oldSourceHash
        if (Test-Path -LiteralPath $recoveryRoot) { throw 'Recovery directory already exists' }
        foreach ($name in $expectedNew) {
            if (Test-Path -LiteralPath (Game-Path "override\$name")) { throw "Reserved component 256 output already exists: $name" }
        }
        $staging = "$recoveryRoot.preparing"
        if (Test-Path -LiteralPath $staging) { throw 'A partial recovery preparation already exists' }
        [IO.Directory]::CreateDirectory($staging) | Out-Null
        foreach ($evidence in @(
            @{source=$sourceEvidence;name='source-evidence.json';sha=$sourceEvidenceHash},
            @{source=$compatibilityEvidence;name='compatibility-evidence.json';sha=$compatibilityEvidenceHash},
            @{source=$provenanceEvidence;name='source-provenance.json';sha=$provenanceEvidenceHash}
        )) {
            [IO.File]::Copy($evidence.source, (Join-Path $staging $evidence.name), $false)
        }
        $tracked = @('WeiDU.log','chitin.key','lang\en_US\dialog.tlk',$sourceRelative.Replace('/','\')) + @($expectedEdited | ForEach-Object { "override\$_" }) + @($expectedNew | ForEach-Object { "override\$_" })
        $state = @()
        $backup = Join-Path $staging 'before-state.zip'
        $zip = [IO.Compression.ZipFile]::Open($backup,[IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($relative in $tracked) {
                $path = Game-Path $relative
                $exists = Test-Path -LiteralPath $path
                $state += @{relative=$relative.Replace('\','/');exists=$exists;sha256=if($exists){Hash $path}else{$null}}
                if ($exists) { [void][IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip,$path,$relative.Replace('\','/'),[IO.Compression.CompressionLevel]::Optimal) }
            }
        } finally { $zip.Dispose() }
        $protected = @($frozenRecipe,$terminalReceipt) + @(Get-ChildItem -LiteralPath (Join-Path $runRoot '.chriz\ledger') -File | Select-Object -ExpandProperty FullName) + @(Get-ChildItem -LiteralPath $failedStep -File | Select-Object -ExpandProperty FullName)
        New-Json (Join-Path $staging 'intent.json') @{
            schema=1;kind='supervised-append-missing-component';install_id='install-d976d775d15d76c8bdc0';recovery_id=$recoveryId
            run_id='chriz-sod-remix-bg2';component=256;installed_components=$installedSoD;final_components=@($installedSoD+256)
            source_commit=$sourceCommit;preserved_active_rows=364;new_files=21;edited_files=6;removed_files=0;later_sibling_write_overlaps=0
            old_source_sha256=$oldSourceHash;fixed_source_sha256=$fixedSourceHash;fixed_archive_sha256=$fixedArchiveHash
            backup_sha256=(Hash $backup);tracked_state=$state
            evidence_files=@(@{name='source-evidence.json';sha256=$sourceEvidenceHash},@{name='compatibility-evidence.json';sha256=$compatibilityEvidenceHash},@{name='source-provenance.json';sha256=$provenanceEvidenceHash})
            protected_files=@($protected | ForEach-Object { @{path=$_;sha256=(Hash $_)} });at=(Get-Date -Format o)
        }
        [IO.Directory]::Move($staging,$recoveryRoot)
        $fixedZip = [IO.Compression.ZipFile]::OpenRead([IO.Path]::GetFullPath($fixedArchive))
        try {
            $entry = $fixedZip.GetEntry($sourceRelative); if ($null -eq $entry) { throw 'Fixed runtime member missing' }
            $temporary = "$sourcePath.cebg-recovery-new"
            if (Test-Path -LiteralPath $temporary) { throw 'Temporary source overlay already exists' }
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry,$temporary,$false)
            Assert-ExactFile $temporary $fixedSourceHash
            Move-Item -LiteralPath $temporary -Destination $sourcePath -Force
        } finally { $fixedZip.Dispose() }
        Assert-ExactFile $sourcePath $fixedSourceHash
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'prepare.verified.json') @{source_overlay=$sourceRelative;source_sha256=$fixedSourceHash;historical_rows=364;at=(Get-Date -Format o)}
        Write-Output 'Prepared the exact one-file SoD fix; all 391 installed components remain untouched.'
        return
    }

    if ($Mode -eq 'FinishPreparation') {
        if (-not (Test-Path -LiteralPath $recoveryRoot)) { throw 'Recovery preparation does not exist' }
        Assert-Protected
        Assert-PreparedIntent
        Assert-Stack -1
        Assert-ExactFile $sourcePath $oldSourceHash
        $temporary = "$sourcePath.cebg-recovery-new"
        Assert-ExactFile $temporary $fixedSourceHash
        if (Test-Path -LiteralPath (Join-Path $recoveryRoot 'prepare.verified.json')) { throw 'Recovery preparation is already complete' }
        Move-Item -LiteralPath $temporary -Destination $sourcePath -Force
        Assert-ExactFile $sourcePath $fixedSourceHash
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'prepare.verified.json') @{source_overlay=$sourceRelative;source_sha256=$fixedSourceHash;historical_rows=364;at=(Get-Date -Format o)}
        Write-Output 'Finished the interrupted source preparation; all 391 installed components remain untouched.'
        return
    }

    Assert-Protected
    Assert-PreparedIntent
    Assert-ExactFile $sourcePath $fixedSourceHash
    if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'prepare.verified.json'))) { throw 'Recovery preparation has not been verified' }

    if ($Mode -eq 'Install256') {
        Assert-Stack -1
        $tool = Game-Path 'Setup-chriz-sod-remix.exe'
        Run-WeiDU 'chriz-sod-remix-256' $tool @('--force-install-list','256')
        Assert-Stack 0
        Assert-Status 'chriz-sod-remix-256' 1
        $backupDir = Game-Path 'weidu_external\backup\chriz-sod-remix\256'
        $mapping = @(Get-Content -LiteralPath (Join-Path $backupDir 'MAPPINGS.256') | ForEach-Object { ($_ -split 'TB#"SPACE"',2)[0].Replace('\','/').ToUpperInvariant() })
        $uninstall = @(Get-Content -LiteralPath (Join-Path $backupDir 'UNINSTALL.256') | Where-Object { $_.Trim() } | ForEach-Object { $_.Trim().Replace('\','/').ToUpperInvariant() })
        $writes = @($mapping + $uninstall | Sort-Object -Unique)
        $expectedWrites = @(($expectedEdited + $expectedNew) | ForEach-Object { "OVERRIDE/$($_.ToUpperInvariant())" } | Sort-Object -Unique)
        if (($writes -join "`n") -cne ($expectedWrites -join "`n")) { throw 'Component 256 backup metadata differs from the approved 27-resource write set' }
        Assert-Protected
        New-Json (Join-Path $recoveryRoot 'chriz-sod-remix-256.verified.json') @{run_id='chriz-sod-remix-bg2';components=@(256);source_commit=$sourceCommit;active_components=365;after_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'));writes=$writes;at=(Get-Date -Format o)}
        Write-Output 'Component 256 appended successfully; the original 364 BG2 rows remain byte-for-byte unchanged.'
        return
    }

    $frozen = Get-Frozen
    $tailRuns = @($frozen.plan.runs | Select-Object -Last 10)
    if (($tailRuns.run_id -join ',') -cne 'hiddengameplayoptions-bg2,chriz-bg-rebalance-bg2,chriz-bg-modpack-bg2,cdtweaks-spell-save-penalties-bg2,safana-bg2,chriz-bg-modpack-late-companions-bg2,spell-rev-npc-spellbooks-bg2,spell-rev-lightning-bg2,klatu-armor-thieving-bg2,buffbot-bg2' -or
        @($tailRuns | Where-Object { $_.target -ne 'bg2' -or $_.args.Count -ne 0 -or $_.prompt_scripts.Count -ne 0 }).Count -ne 0) {
        throw 'Frozen continuation is not the approved ten-run prompt-free BG2 tail'
    }

    if ($Mode -eq 'FinishTail') {
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot 'chriz-sod-remix-256.verified.json'))) { throw 'Component 256 append has not been verified' }
        $completed = 0
        for ($i=0; $i -lt $tailRuns.Count; $i++) {
            $run = $tailRuns[$i]
            $verified = Join-Path $recoveryRoot "$($run.run_id).verified.json"
            if (Test-Path -LiteralPath $verified) {
                $checkpoint = Get-Content -LiteralPath $verified -Raw | ConvertFrom-Json
                if ($checkpoint.run_id -cne $run.run_id -or (($checkpoint.components -join ',') -cne ($run.components -join ','))) { throw "Checkpoint changed: $($run.run_id)" }
                $completed++
                continue
            }
            Assert-Stack $completed
            [void](Assert-RunSource $frozen $run)
            $mod = $frozen.mods.($run.mod_id)
            if ($mod.language -ne 0 -or $mod.invocation_mode -cne 'setup-name') { throw "Unsupported frozen invocation: $($run.run_id)" }
            $setupName = [IO.Path]::GetFileNameWithoutExtension($mod.tp2)
            if (-not $setupName.StartsWith('setup-', [StringComparison]::OrdinalIgnoreCase)) { $setupName = 'Setup-' + $setupName }
            $tool = Game-Path ($setupName + '.exe')
            if (-not (Test-Path -LiteralPath $tool)) { [IO.File]::Copy((Game-Path 'Setup-chriz-sod-remix.exe'),$tool,$false) }
            $operationStage = $run.run_id
            if ($i -eq 0 -and (Test-Path -LiteralPath (Join-Path $recoveryRoot 'hiddengameplayoptions-bg2.intent.json'))) {
                Assert-HgoNoopEvidence
                $operationStage = 'hiddengameplayoptions-bg2-attempt2'
            }
            Run-WeiDU $operationStage $tool (@('--force-install-list') + @($run.components | ForEach-Object { [string]$_ }))
            Assert-Status $operationStage $run.components.Count
            $completed++
            Assert-Stack $completed
            Assert-Protected
            New-Json $verified @{run_id=$run.run_id;operation_stem=$operationStage;components=$run.components;artifact_id=$run.artifact_id;artifact_sha256=$frozen.artifacts.($run.artifact_id).source.sha256;active_components=(Active-Lines (Join-Path $gameRoot 'WeiDU.log')).Count;after_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'));at=(Get-Date -Format o)}
        }
    }

    if ($Mode -in @('FinishTail','Audit')) {
        Assert-Stack 10
        $bg1 = @(Active-Lines (Join-Path $runRoot 'bg1\WeiDU.log'))
        $bg2 = @(Active-Lines (Join-Path $gameRoot 'WeiDU.log'))
        if ($bg1.Count -ne 27 -or $bg2.Count -ne 419) { throw 'Final full component counts differ from 446' }
        $expectedBg1 = @($frozen.plan.runs | Where-Object target -EQ 'bg1' | ForEach-Object { $run=$_;$mod=$frozen.mods.($run.mod_id);foreach($component in $run.components){[pscustomobject]@{tp2=$mod.tp2;language=[int]$mod.language;component=[int]$component}} })
        if ($expectedBg1.Count -ne $bg1.Count) { throw 'Frozen BG1 plan length differs' }
        for ($i=0;$i -lt $bg1.Count;$i++) {
            if ($bg1[$i] -notmatch '^~([^~]+)~ #(\d+) #(\d+) // ' -or $Matches[1].Replace('\','/') -ine $expectedBg1[$i].tp2.Replace('\','/') -or [int]$Matches[2] -ne $expectedBg1[$i].language -or [int]$Matches[3] -ne $expectedBg1[$i].component) { throw "BG1 plan/log differs at $i" }
        }
        $name = if ($Mode -eq 'Audit') {'final-plan-audit.json'} else {'recovered-install.json'}
        if (-not (Test-Path -LiteralPath (Join-Path $recoveryRoot $name))) {
            New-Json (Join-Path $recoveryRoot $name) @{schema=1;kind='supervised-recovered-installation';install_id='install-d976d775d15d76c8bdc0';recipe='local-40712587d45b';source_commit=$sourceCommit;authorized_physical_order='original 364 BG2 rows, appended SoD 256, frozen ten-run tail';bg1_components=27;bg2_components=419;total_components=446;original_terminal_receipt_sha256=$terminalHash;bg1_log_sha256=(Hash (Join-Path $runRoot 'bg1\WeiDU.log'));bg2_log_sha256=(Hash (Join-Path $gameRoot 'WeiDU.log'));managed_campaign_complete=$false;gameplay_smoke_tested=$false;at=(Get-Date -Format o)}
        }
        Assert-Protected
        Write-Output 'Supervised installation tail complete: 446 components; original failed receipt remains immutable.'
    }
} finally {
    if ($null -ne $lock) { $lock.Dispose() }
}

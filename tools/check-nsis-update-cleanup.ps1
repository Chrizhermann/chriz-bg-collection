<#
Checks that installing over an existing installation in update mode, as the in-app updater does,
leaves the folders an app bundles with exactly the new version's files.

The script installs the setup into a new folder, adds what an older version could have left in
its bundled folders (a stale file, a stale folder, an older copy of a recipe artifact and a
junction to a folder outside), then runs the same setup again with /UPDATE. Afterwards every
bundled folder must match the first install, the junction must be gone and its target untouched.
With -RecipeFolder and -ValidatorPath it also shows that the stale files break the bundled recipe
and that the update repairs it.

Installing registers the app for the current Windows user and closes a running copy of it, so
the script runs only on a disposable GitHub Actions runner.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SetupPath,

    # The app's Tauri configuration files in build order: tauri.conf.json, then an overlay such as
    # tauri.iwd.conf.json. Their bundle.resources name the folders the app bundles.
    [Parameter(Mandatory = $true)]
    [string[]]$TauriConfiguration,

    # A new folder for the test install. NSIS takes /D= unquoted, so it must not contain spaces.
    [Parameter(Mandatory = $true)]
    [string]$WorkDirectory,

    # The bundled recipe, one of the bundled folders, and chriz-bg-install.exe to validate it.
    [string]$RecipeFolder,
    [string]$ValidatorPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($env:GITHUB_ACTIONS -ne 'true') {
    throw 'This check installs the app for the current Windows user; run it only on a GitHub Actions runner.'
}
if ([string]::IsNullOrEmpty($RecipeFolder) -ne [string]::IsNullOrEmpty($ValidatorPath)) {
    throw 'Pass -RecipeFolder and -ValidatorPath together.'
}

# RFC 7396 JSON merge patch, as the Tauri CLI merges --config files.
function Merge-JsonPatch($Target, $Patch) {
    if ($Patch -isnot [Collections.IDictionary]) { return , $Patch }
    if ($Target -isnot [Collections.IDictionary]) { $Target = [ordered]@{} }
    foreach ($key in @($Patch.Keys)) {
        if ($null -eq $Patch[$key]) { $Target.Remove($key) }
        else { $Target[$key] = Merge-JsonPatch $Target[$key] $Patch[$key] }
    }
    return $Target
}
$configuration = $null
foreach ($path in $TauriConfiguration) {
    $configuration = Merge-JsonPatch $configuration (Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable)
}
$BundledFolders = @($configuration['bundle']['resources'].Values |
    Where-Object { $_ -is [string] -and $_.EndsWith('/') } |
    ForEach-Object { $_.TrimEnd('/') } | Sort-Object)
if ($BundledFolders.Count -eq 0) { throw 'The configuration bundles no folders.' }
if (-not [string]::IsNullOrEmpty($RecipeFolder) -and $BundledFolders -notcontains $RecipeFolder) {
    throw "RecipeFolder must be one of the bundled folders: $($BundledFolders -join ', ')."
}
$setup = (Resolve-Path -LiteralPath $SetupPath).Path
$work = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($WorkDirectory)
if ($work -match '\s') { throw 'WorkDirectory must not contain spaces.' }
if (Test-Path -LiteralPath $work) { throw "WorkDirectory '$work' must not exist yet." }
$installDir = Join-Path $work 'app'
$outside = Join-Path $work 'outside'
New-Item -ItemType Directory -Path $outside -Force | Out-Null
$keep = Join-Path $outside 'keep.txt'
Set-Content -LiteralPath $keep -Value 'outside the install folder' -Encoding utf8

function Invoke-Setup([string[]]$Arguments) {
    # NSIS reads /D= up to the end of the command line.
    $process = Start-Process -FilePath $setup -ArgumentList ($Arguments + "/D=$installDir") -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "$(Split-Path -Leaf $setup) $($Arguments -join ' ') exited with $($process.ExitCode)."
    }
}

function Get-BundledFiles {
    @(foreach ($folder in $BundledFolders) {
        $root = Join-Path $installDir $folder
        if (-not (Test-Path -LiteralPath $root -PathType Container)) { throw "The setup did not install '$folder'." }
        Get-ChildItem -LiteralPath $root -Recurse -Force | ForEach-Object {
            $relative = "$folder/$([IO.Path]::GetRelativePath($root, $_.FullName).Replace('\', '/'))"
            if ($_.PSIsContainer) { "$relative/" }
            else { "$relative  $((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash)" }
        }
    }) | Sort-Object
}

function Test-Recipe {
    $output = & $ValidatorPath validate (Join-Path $installDir $RecipeFolder) --profile public-alpha 2>&1
    [pscustomobject]@{ Valid = ($LASTEXITCODE -eq 0); Output = ($output | Out-String).Trim() }
}

Invoke-Setup @('/S')
$fresh = Get-BundledFiles
if (@($fresh | Where-Object { -not $_.EndsWith('/') }).Count -eq 0) { throw 'The first install left the bundled folders empty.' }
if ($RecipeFolder) {
    $check = Test-Recipe
    if (-not $check.Valid) { throw "The freshly installed recipe does not validate:`n$($check.Output)" }
}

foreach ($folder in $BundledFolders) {
    $root = Join-Path $installDir $folder
    Set-Content -LiteralPath (Join-Path $root 'stale-from-an-older-version.txt') -Value 'stale' -Encoding utf8
    $staleFolder = New-Item -ItemType Directory -Path (Join-Path $root 'stale-folder')
    Set-Content -LiteralPath (Join-Path $staleFolder.FullName 'stale.txt') -Value 'stale' -Encoding utf8
}
$junction = Join-Path (Join-Path $installDir $BundledFolders[0]) 'junction-to-outside'
New-Item -ItemType Junction -Path $junction -Target $outside | Out-Null
if ($RecipeFolder) {
    # An older release of one mod, as 0.1.3 left snowytoes-1.7 next to 0.1.6's snowytoes-1.7.1.
    $artifact = Get-ChildItem -LiteralPath (Join-Path (Join-Path $installDir $RecipeFolder) 'artifacts') -Filter '*.toml' -File |
        Sort-Object Name |
        Where-Object { (Get-Content -LiteralPath $_.FullName -Raw) -match '(?m)^sha256 = "[0-9a-f]{64}"' } |
        Select-Object -First 1
    if ($null -eq $artifact) { throw 'No bundled artifact names a SHA-256 to copy as an older release.' }
    $older = (Get-Content -LiteralPath $artifact.FullName -Raw) -replace '(?m)^id = "([^"]+)"', 'id = "$1-older"'
    $older = $older -replace '(?m)^(sha256 = ")[0-9a-f]{8}', '${1}00000000'
    Set-Content -LiteralPath (Join-Path $artifact.DirectoryName "$($artifact.BaseName)-older.toml") -Value $older -NoNewline -Encoding utf8
    $check = Test-Recipe
    if ($check.Valid) { throw 'The stale files did not break the installed recipe, so this check proves nothing.' }
    Write-Host "Before the update, as expected:`n$($check.Output)"
}

Invoke-Setup @('/S', '/UPDATE')
$updated = Get-BundledFiles
$difference = @(Compare-Object -ReferenceObject $fresh -DifferenceObject $updated)
if ($difference.Count -ne 0) {
    throw "After the update the bundled folders differ from a fresh install:`n$($difference | Out-String)"
}
if (Test-Path -LiteralPath $junction) { throw 'The junction inside a bundled folder survived the update.' }
if (-not (Test-Path -LiteralPath $keep)) { throw 'The update removed a file outside the install folder through a junction.' }
if ($RecipeFolder) {
    $check = Test-Recipe
    if (-not $check.Valid) { throw "The recipe does not validate after the update:`n$($check.Output)" }
}

[pscustomobject]@{
    Setup = Split-Path -Leaf $setup
    BundledEntries = $fresh.Count
    Result = 'The update left exactly the new version''s bundled files.'
}

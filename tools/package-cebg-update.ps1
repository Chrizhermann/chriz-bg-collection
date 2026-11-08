[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$')]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$SetupPath,

    [Parameter(Mandatory = $true)]
    [string]$SignaturePath,

    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory,

    # The app this setup is: Chriz Easy BG or Chriz Easy IWD. It names the distribution
    # repository the feed points to; each app reads its own feed (docs/iwd-release.md).
    [ValidateSet('bg', 'iwd')]
    [string]$Flavor = 'bg',

    [string]$PublishedSetupFilename,
    [string]$RecipeVersion = $Version,
    [string]$Notes,

    # Further release files, such as LICENSE, listed in SHA256SUMS under their own names.
    [string[]]$AdditionalAssets = @()
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$product = @{
    bg  = @{ Name = 'Chriz Easy BG'; Repository = 'Chrizhermann/chriz-easy-bg' }
    iwd = @{ Name = 'Chriz Easy IWD'; Repository = 'Chrizhermann/chriz-easy-iwd' }
}[$Flavor]
if ([string]::IsNullOrWhiteSpace($Notes)) { $Notes = "$($product.Name) $Version." }

$setup = Get-Item -LiteralPath $SetupPath
$signatureFile = Get-Item -LiteralPath $SignaturePath
if ($setup.PSIsContainer -or $setup.Length -eq 0 -or $setup.Extension -ine '.exe') {
    throw 'SetupPath must name a nonempty NSIS setup .exe.'
}
if ($signatureFile.PSIsContainer -or $signatureFile.Length -eq 0 -or $signatureFile.Extension -ine '.sig') {
    throw 'SignaturePath must name the nonempty Tauri .sig file for this setup.'
}
# Tauri names the setup after the product and version. A feed that announces this version
# with another app's or another version's setup would offer the same update again and again.
$namePattern = @($product.Name -split ' ' | ForEach-Object { [regex]::Escape($_) }) -join '[ .]'
$expectedSetup = "^$($namePattern)_$([regex]::Escape($Version))_x64-setup\.exe$"
if ($setup.Name -notmatch $expectedSetup) {
    throw "SetupPath must name the $($product.Name) $Version setup ('$($product.Name)_$($Version)_x64-setup.exe')."
}
$signature = [IO.File]::ReadAllText($signatureFile.FullName).Trim()
if ([string]::IsNullOrWhiteSpace($signature)) {
    throw 'The Tauri signature is empty.'
}
try {
    $signatureBytes = [Convert]::FromBase64String($signature)
    if ($signatureBytes.Length -eq 0) { throw 'Empty signature payload.' }
    $signature = [Convert]::ToBase64String($signatureBytes)
} catch {
    throw 'The Tauri .sig file must contain a nonempty base64 signature.'
}
if ([string]::IsNullOrWhiteSpace($RecipeVersion)) { throw 'RecipeVersion cannot be empty.' }
if ([string]::IsNullOrWhiteSpace($PublishedSetupFilename)) {
    $PublishedSetupFilename = $setup.Name -replace ' ', '.'
}
if ($PublishedSetupFilename -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]*\.exe$') {
    throw 'PublishedSetupFilename must be a portable ASCII .exe filename without directories.'
}
$publishedSignatureFilename = "$PublishedSetupFilename.sig"
$additional = @(foreach ($assetPath in $AdditionalAssets) {
    $asset = Get-Item -LiteralPath $assetPath
    if ($asset.PSIsContainer -or $asset.Length -eq 0) { throw "Additional asset '$assetPath' must be a nonempty file." }
    if ($asset.Name -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._-]*$') {
        throw "Additional asset '$($asset.Name)' must have a portable ASCII filename."
    }
    $asset
})
$publishedNames = @($PublishedSetupFilename, $publishedSignatureFilename, 'latest.json', 'SHA256SUMS') + @($additional | ForEach-Object Name)
if (@($publishedNames | Sort-Object -Unique).Count -ne $publishedNames.Count) {
    throw 'Every published release file needs its own name.'
}

$output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
$feedPath = Join-Path $output 'latest.json'
$checksumsPath = Join-Path $output 'SHA256SUMS'
foreach ($inputFile in @($setup.FullName, $signatureFile.FullName) + @($additional | ForEach-Object FullName)) {
    if ($inputFile -ieq $feedPath -or $inputFile -ieq $checksumsPath) {
        throw 'Generated metadata must not overwrite either input file.'
    }
}

$setupHash = (Get-FileHash -LiteralPath $setup.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$signatureHash = (Get-FileHash -LiteralPath $signatureFile.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$releaseTag = [Uri]::EscapeDataString("v$Version")
$assetName = [Uri]::EscapeDataString($PublishedSetupFilename)
$feed = [ordered]@{
    version = $Version
    recipe_version = $RecipeVersion
    notes = $Notes
    pub_date = [DateTimeOffset]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture)
    platforms = [ordered]@{
        'windows-x86_64' = [ordered]@{
            url = "https://github.com/$($product.Repository)/releases/download/$releaseTag/$assetName"
            signature = $signature
        }
    }
}

[void][IO.Directory]::CreateDirectory($output)
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($feedPath, ($feed | ConvertTo-Json -Depth 5) + "`n", $utf8)
$feedHash = (Get-FileHash -LiteralPath $feedPath -Algorithm SHA256).Hash.ToLowerInvariant()
$checksums = @("$setupHash  $PublishedSetupFilename", "$signatureHash  $publishedSignatureFilename", "$feedHash  latest.json")
foreach ($asset in $additional) {
    $checksums += "$((Get-FileHash -LiteralPath $asset.FullName -Algorithm SHA256).Hash.ToLowerInvariant())  $($asset.Name)"
}
[IO.File]::WriteAllText($checksumsPath, ($checksums -join "`n") + "`n", $utf8)

[pscustomobject]@{ FeedPath = $feedPath; ChecksumsPath = $checksumsPath; SetupSha256 = $setupHash; PublishedSetupFilename = $PublishedSetupFilename }

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

    [string]$RecipeVersion = $Version,
    [string]$Notes = "Chriz Easy BG $Version."
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$setup = Get-Item -LiteralPath $SetupPath
$signatureFile = Get-Item -LiteralPath $SignaturePath
if ($setup.PSIsContainer -or $setup.Length -eq 0 -or $setup.Extension -ine '.exe') {
    throw 'SetupPath must name a nonempty NSIS setup .exe.'
}
if ($signatureFile.PSIsContainer -or $signatureFile.Length -eq 0 -or $signatureFile.Extension -ine '.sig') {
    throw 'SignaturePath must name the nonempty Tauri .sig file for this setup.'
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

$output = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDirectory)
$feedPath = Join-Path $output 'latest.json'
$checksumsPath = Join-Path $output 'SHA256SUMS'
foreach ($inputFile in @($setup.FullName, $signatureFile.FullName)) {
    if ($inputFile -ieq $feedPath -or $inputFile -ieq $checksumsPath) {
        throw 'Generated metadata must not overwrite either input file.'
    }
}

$setupHash = (Get-FileHash -LiteralPath $setup.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$signatureHash = (Get-FileHash -LiteralPath $signatureFile.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$releaseTag = [Uri]::EscapeDataString("v$Version")
$assetName = [Uri]::EscapeDataString($setup.Name)
$feed = [ordered]@{
    version = $Version
    recipe_version = $RecipeVersion
    notes = $Notes
    pub_date = [DateTimeOffset]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ', [Globalization.CultureInfo]::InvariantCulture)
    platforms = [ordered]@{
        'windows-x86_64' = [ordered]@{
            url = "https://github.com/Chrizhermann/chriz-easy-bg/releases/download/$releaseTag/$assetName"
            signature = $signature
        }
    }
}

[void][IO.Directory]::CreateDirectory($output)
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($feedPath, ($feed | ConvertTo-Json -Depth 5) + "`n", $utf8)
$feedHash = (Get-FileHash -LiteralPath $feedPath -Algorithm SHA256).Hash.ToLowerInvariant()
$checksums = @("$setupHash  $($setup.Name)", "$signatureHash  $($signatureFile.Name)", "$feedHash  latest.json")
[IO.File]::WriteAllText($checksumsPath, ($checksums -join "`n") + "`n", $utf8)

[pscustomobject]@{ FeedPath = $feedPath; ChecksumsPath = $checksumsPath; SetupSha256 = $setupHash }

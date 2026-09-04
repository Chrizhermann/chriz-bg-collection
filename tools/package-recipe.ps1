[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$RecipeRoot,

    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory,

    [Parameter(Mandatory = $true)]
    [string]$SigningKeyPath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Test-PathWithin {
    param(
        [Parameter(Mandatory = $true)][string]$Candidate,
        [Parameter(Mandatory = $true)][string]$Parent
    )

    $normalizedParent = $Parent.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    return $Candidate.Equals($normalizedParent, [StringComparison]::OrdinalIgnoreCase) -or
        $Candidate.StartsWith($normalizedParent + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
}

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$resolvedRecipe = (Resolve-Path -LiteralPath $RecipeRoot).Path
$resolvedKey = (Resolve-Path -LiteralPath $SigningKeyPath).Path
$keyItem = Get-Item -LiteralPath $resolvedKey -Force
if ($keyItem.PSIsContainer -or $null -ne $keyItem.LinkType) {
    throw 'SigningKeyPath must name a regular, non-link file.'
}
if (Test-PathWithin -Candidate $resolvedKey -Parent $repositoryRoot) {
    throw 'SigningKeyPath must be outside the repository.'
}

$publicKeyPath = [IO.Path]::ChangeExtension($resolvedKey, '.pub')
if (-not (Test-Path -LiteralPath $publicKeyPath -PathType Leaf)) {
    throw "Expected the matching public key beside the signing key at: $publicKeyPath"
}
$keyId = [IO.Path]::GetFileNameWithoutExtension($publicKeyPath)

$resolvedOutput = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-PathWithin -Candidate $resolvedOutput -Parent $resolvedRecipe) {
    throw 'OutputDirectory must be outside RecipeRoot so package output cannot include itself.'
}

$minisign = Get-Command minisign -ErrorAction Stop
$appConfig = Get-Content -LiteralPath (Join-Path $repositoryRoot 'app/src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
$appVersion = [string]$appConfig.version

& cargo run --quiet --manifest-path (Join-Path $repositoryRoot 'Cargo.toml') `
    -p chriz-bg-engine --bin chriz-bg-author -- package-recipe `
    --recipe-root $resolvedRecipe --version $Version --output-directory $resolvedOutput --key-id $keyId
if ($LASTEXITCODE -ne 0) {
    throw 'Recipe payload/envelope packaging failed.'
}

$envelopePath = Join-Path $resolvedOutput 'envelope.json'
$signaturePath = Join-Path $resolvedOutput 'envelope.json.minisig'
& $minisign.Source -S -s $resolvedKey -m $envelopePath -x $signaturePath
if ($LASTEXITCODE -ne 0) {
    throw 'Minisign recipe-envelope signing failed.'
}

& cargo run --quiet --manifest-path (Join-Path $repositoryRoot 'Cargo.toml') `
    -p chriz-bg-engine --bin chriz-bg-author -- verify-recipe $resolvedOutput `
    --key-id $keyId --trusted-public-key-file $publicKeyPath --running-app-version $appVersion
if ($LASTEXITCODE -ne 0) {
    throw 'Immediate signed recipe verification failed.'
}

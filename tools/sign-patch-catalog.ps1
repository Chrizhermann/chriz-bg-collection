[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SigningKeyPath,

    # Existing package_config test binary; performs cryptographic verification.
    [Parameter(Mandatory = $true)]
    [string]$SignatureTestExecutable,

    [string]$CatalogPath = (Join-Path $PSScriptRoot '../patches/catalog.json'),
    [string]$PublicKeyPath = '',
    [switch]$EmptyPassword
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$resolvedKey = (Resolve-Path -LiteralPath $SigningKeyPath).Path
$keyItem = Get-Item -LiteralPath $resolvedKey -Force
if ($keyItem.PSIsContainer -or $null -ne $keyItem.LinkType) {
    throw 'SigningKeyPath must be an external regular, non-link file.'
}
if ($resolvedKey.StartsWith($repositoryRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar,
        [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Private signing keys must remain outside the repository.'
}
if ([string]::IsNullOrWhiteSpace($PublicKeyPath)) {
    $PublicKeyPath = $resolvedKey + '.pub'
}
$publicText = (Get-Content -LiteralPath $PublicKeyPath -Raw).Trim()
$config = Get-Content -LiteralPath (Join-Path $repositoryRoot 'app/src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
if ($publicText -cne ([string]$config.plugins.updater.pubkey).Trim()) {
    throw 'The supplied public key does not match the bundled updater trust key.'
}
$decodedPublic = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($publicText))
if (-not $decodedPublic.StartsWith('untrusted comment: minisign public key:')) {
    throw 'Expected a Tauri-encoded minisign public key.'
}
$resolvedCatalog = (Resolve-Path -LiteralPath $CatalogPath).Path
$catalogItem = Get-Item -LiteralPath $resolvedCatalog -Force
if ($catalogItem.PSIsContainer -or $null -ne $catalogItem.LinkType) {
    throw 'CatalogPath must be a regular, non-link file.'
}
$catalogHash = (Get-FileHash -LiteralPath $resolvedCatalog -Algorithm SHA256).Hash
$testExecutable = (Resolve-Path -LiteralPath $SignatureTestExecutable).Path
$testName = 'built_update_signature_matches_the_bundled_public_key'
$availableTests = & $testExecutable --list 2>&1
if ($LASTEXITCODE -ne 0 -or -not ($availableTests -match "^${testName}: test$")) {
    throw 'Verification executable does not expose the expected signature test.'
}

$cli = Join-Path $repositoryRoot 'app/node_modules/@tauri-apps/cli/tauri.js'
if (-not (Test-Path -LiteralPath $cli -PathType Leaf)) {
    throw 'Install the locked app dependencies before signing.'
}
$temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$temporary = New-Item -ItemType Directory -Path (Join-Path $temporaryRoot ('cebg-patch-sign-' + [Guid]::NewGuid().ToString('N')))
$oldSetup = [Environment]::GetEnvironmentVariable('CEBG_SIGNED_SETUP', 'Process')
try {
    $temporaryCatalog = Join-Path $temporary.FullName 'catalog.json'
    Copy-Item -LiteralPath $resolvedCatalog -Destination $temporaryCatalog
    # Pass only the key path, never its contents. Node preserves an explicitly
    # empty password argument across PowerShell/native argument handling.
    $signer = @'
const { spawnSync } = require('node:child_process');
const [cli, key, file, emptyPassword] = process.argv.slice(1);
const args = [cli, 'signer', 'sign', '--private-key-path', key];
if (emptyPassword === 'True') args.push('--password', '');
args.push(file);
const child = spawnSync(process.execPath, args, { stdio: 'inherit', windowsHide: true });
if (child.error) { process.stderr.write(child.error.message); process.exit(1); }
process.exit(child.status === null ? 1 : child.status);
'@
    & node -e $signer $cli $resolvedKey $temporaryCatalog ([string]$EmptyPassword.IsPresent)
    if ($LASTEXITCODE -ne 0) { throw 'Tauri catalog signing failed.' }
    [Environment]::SetEnvironmentVariable('CEBG_SIGNED_SETUP', $temporaryCatalog, 'Process')
    & $testExecutable --ignored --exact $testName --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'The new catalog signature failed immediate verification.' }
    if ((Get-FileHash -LiteralPath $resolvedCatalog -Algorithm SHA256).Hash -cne $catalogHash) {
        throw 'Catalog changed during signing; no signature was published.'
    }
    $encodedSignature = (Get-Content -LiteralPath ($temporaryCatalog + '.sig') -Raw).Trim()
    $decodedSignature = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($encodedSignature))
    if (-not $decodedSignature.StartsWith('untrusted comment:')) {
        throw 'Expected a Tauri-encoded minisign signature.'
    }
    # Generated public artifacts are raw minisign text, with LF and no BOM.
    $utf8 = New-Object Text.UTF8Encoding($false)
    [IO.File]::WriteAllText($resolvedCatalog + '.sig', $decodedSignature.Replace("`r`n", "`n"), $utf8)
    [IO.File]::WriteAllText((Join-Path $catalogItem.DirectoryName 'public-key.txt'), $decodedPublic.Replace("`r`n", "`n"), $utf8)
    Write-Output "Catalog signed and verified; SHA-256 $($catalogHash.ToLowerInvariant())"
} finally {
    [Environment]::SetEnvironmentVariable('CEBG_SIGNED_SETUP', $oldSetup, 'Process')
    $cleanupPath = [IO.Path]::GetFullPath($temporary.FullName)
    if (-not $cleanupPath.StartsWith($temporaryRoot.TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar,
            [StringComparison]::OrdinalIgnoreCase) -or $temporary.Name -notmatch '^cebg-patch-sign-[0-9a-f]{32}$') {
        throw 'Temporary cleanup path failed containment validation.'
    }
    Remove-Item -LiteralPath $cleanupPath -Recurse -Force
}

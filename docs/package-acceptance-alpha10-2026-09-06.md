# Local alpha.10 package acceptance (2026-09-06)

This is a local, unpublished Windows package of app `0.1.0-alpha.10` with bundled recipe
`0.1.0-alpha.11`. No installer, application, browser, game, or updater installation was
launched.

## Preconditions and build

The narrowed bundle-resource contract passed all three normal `package_config` tests. The
resources include the application and UnRAR notices plus production game profiles and only
the required runtime manifest files/directories; authoring reference/evidence TSV and map
data are excluded.

PowerShell supplied the existing private key by path with an explicitly empty password; key
bytes were neither read nor printed. The single build command was:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = 'C:\Users\chris\AppData\Local\Chriz Easy BG Developer\signing\cebg-updater.key'
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ''
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis
```

The release compile completed in 2m00s, followed by a successful NSIS bundle and updater
signature. Packaging evidence was finalized at `2026-09-06T04:42:10.574+09:00`
(`2026-09-05T19:42:10.574Z`).

- Setup: `target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.10_x64-setup.exe`
- Size: 5,392,890 bytes
- SHA-256: `02051c6d19708be3064ab05da5f3e5e03ba85620bb4670c90f4d851359718206`
- Setup completion time: `2026-09-05T19:41:18.0825423Z`
- Signature size: 436 bytes
- Signature SHA-256: `e2aaf689f882f365a2c30ac6962277c33ad1475fead733f771c3be4f596e1628`

The prior alpha.9 setup remains present at its original 5,252,740 bytes and SHA-256
`d3f04c79a1d701755b36907c4e10c6135c7063852b2a03a71d1930b9024f0f9f`.

## Headless updater verification

The freshly built setup passed the ignored `package_config` signature test against the
bundled updater public key: 1 passed, 0 failed.

The current dependency feature set does not expose `tauri::test` while recompiling
`updater_download_acceptance.rs`. As authorized for this acceptance, the existing compiled
headless harness
`target/debug/deps/updater_download_acceptance-d26df3b5ef40af79.exe` was run directly with
app version `0.1.0-alpha.10`, current version `0.1.0-alpha.9`, and the new setup. Its real
Tauri loopback update check, download, exact-byte comparison, signature verification, and
tamper rejection passed: 1 passed, 0 failed.

## Local update feed

`tools/package-cebg-update.ps1` generated the unpublished feed at
`target/cebg-release/0.1.0-alpha.10/` for app `0.1.0-alpha.10` and recipe
`0.1.0-alpha.11`.

- `latest.json` SHA-256: `f1141943f0d1eebd0671f4bd7a679702d59942ce0bd54de062a91b9c397580bc`
- `SHA256SUMS` SHA-256: `aa7cb062d41640c64ab50eb5ab352a28f41c99511feeddd9864a2d412060825f`

Automatic updater apply/restart, installer execution, native UI behavior, game startup,
and any full game installation remain untested. Nothing was published or committed.

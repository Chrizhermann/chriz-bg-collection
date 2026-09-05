# Local alpha.9 package acceptance (2026-09-06)

This is a local, unpublished Windows package of app `0.1.0-alpha.9` with bundled recipe
`0.1.0-alpha.10` and the current recovery/safe-pause lifecycle sources. No installer,
application, browser, game, or updater installation was launched.

## Build

PowerShell supplied the existing private key by path and an explicitly empty password; no
key bytes were read or printed. The single normal build command was:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = 'C:\Users\chris\AppData\Local\Chriz Easy BG Developer\signing\cebg-updater.key'
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ''
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis
```

The release compile, NSIS bundle, and updater signature completed successfully in 1m17s.
Packaging evidence was completed at `2026-09-06T03:42:26.840+09:00`
(`2026-09-05T18:42:26.840Z`).

- Setup: `target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.9_x64-setup.exe`
- Size: 5,252,740 bytes
- SHA-256: `d3f04c79a1d701755b36907c4e10c6135c7063852b2a03a71d1930b9024f0f9f`
- Setup completion time: `2026-09-05T18:41:53.477Z`
- Signature SHA-256: `3a8f33985369b7184d37a036cd83ddbc303a6986953cd03a434637524ce0850a`

## Headless updater verification

The existing compiled harnesses remained applicable: neither acceptance-test source nor the
bundled updater public key contract changed. Both were run directly against the new setup.

```powershell
$env:CEBG_SIGNED_SETUP = $setup
target\debug\deps\package_config-eb1c48ea32bc9612.exe built_update_signature_matches_the_bundled_public_key --ignored --exact

$env:CEBG_UPDATER_SETUP = $setup
$env:CEBG_UPDATER_VERSION = '0.1.0-alpha.9'
$env:CEBG_UPDATER_CURRENT_VERSION = '0.1.0-alpha.8'
target\debug\deps\updater_download_acceptance-d26df3b5ef40af79.exe real_tauri_updater_downloads_and_verifies_the_signed_nsis --ignored --exact
```

- Bundled-public-key signature test: 1 passed, 0 failed.
- Real Tauri loopback updater check/download/signature/exact-byte/tamper-rejection test:
  1 passed, 0 failed.

The local unpublished feed was regenerated with `tools/package-cebg-update.ps1` at
`target/cebg-release/0.1.0-alpha.9/`. It declares app `0.1.0-alpha.9` and recipe
`0.1.0-alpha.10`.

- `latest.json` SHA-256: `bfef04f595a2bbc557b50f59e53699f2aed398307869141c5462dd141f69d9e3`
- `SHA256SUMS` SHA-256: `a9a2b9dace50f16e2e156ba6cc14f4557cf2057c5f6352a7db74908cb7aa5f62`

Automatic updater apply/restart, installer execution, native UI behavior, and real WeiDU
pause acceptance remain untested. Nothing was published or committed.

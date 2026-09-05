# Signed updater download acceptance (2026-09-05)

`app/src-tauri/tests/updater_download_acceptance.rs` exercises the real
`tauri-plugin-updater` check and download path without opening a window or starting NSIS. It
serves a feed and selected setup from loopback, checks an old-to-new version transition,
downloads and verifies the complete setup against the public key in `tauri.conf.json`, and
asserts that the downloaded bytes equal the selected file. A second download changes one byte
while retaining the signature and must fail specifically in the plugin's Minisign verifier.

The loopback endpoint is accepted only in a debug test build. The test does not set Tauri's
insecure-transport option, and release builds continue to reject HTTP updater endpoints. It
does not read the private updater key.

## Result

The test passed for the signed alpha.3 setup as an alpha.2 to alpha.3 update:

- setup: `target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.3_x64-setup.exe`
- size: 5,093,355 bytes
- SHA-256: `B878EBA376C6C1C598700B8BD072B1181F33B8D721A5258D95A0F34F93C7D1A3`
- valid artifact: feed check, complete HTTP download, signature verification, and byte equality passed
- changed artifact: complete HTTP download reached signature verification and was rejected

The focused engine acquisition suites also passed: 3 cache tests and 8 HTTP tests, including
rehash-on-cache-hit, no publication on hash mismatch, interruption/resume, validator changes,
retry bounds, redirect handling, and HTTPS downgrade rejection.

## Alpha.4 result

The same test passed for the signed alpha.4 setup as an alpha.3 to alpha.4 update:

- setup: `target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.4_x64-setup.exe`
- size: 5,096,503 bytes
- SHA-256: `DF3110688FAD1B6F9AED0C51702867E15200E91FF7D90D807ABF2EDBA74DB944`
- valid artifact: feed check, complete HTTP download, signature verification, and byte equality passed
- changed artifact: complete HTTP download reached signature verification and was rejected

The local, unpublished feed is in `target/cebg-release/0.1.0-alpha.4/`. It identifies app and
recipe version `0.1.0-alpha.4` and describes the corrected full curated setup. Its checksum
manifest records:

- setup signature SHA-256: `E4D9BBDA1D1EDFEF21FD5D098FD21F6426D93A9583F31BF7E22491377D90F485`
- `latest.json` SHA-256: `444ED51FA1A859DE137640FE06F788E602B3D596D5F91938B722AC16998E72B3`

The exact PowerShell rerun follows. The linker flag embeds the Common Controls v6 dependency
required by Tauri's mock runtime in a Cargo test binary; it does not change the shipped
application.

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:CEBG_UPDATER_SETUP = (Resolve-Path -LiteralPath 'target\release\bundle\nsis\Chriz Easy BG_0.1.0-alpha.4_x64-setup.exe').Path
$env:CEBG_UPDATER_VERSION = '0.1.0-alpha.4'
$env:CEBG_UPDATER_CURRENT_VERSION = '0.1.0-alpha.3'
$env:CARGO_ENCODED_RUSTFLAGS = "-Clink-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
cargo test -p chriz-bg-app --features tauri/test --test updater_download_acceptance real_tauri_updater_downloads_and_verifies_the_signed_nsis -- --ignored --exact
```

This is meaningful signed check/download/verification acceptance, but it is not a complete
updater lifecycle. The harness intentionally never calls `install` or `download_and_install`:
automatic NSIS apply, process exit, relaunch into the new version, displayed version, and
preservation of the managed-install registry across that restart remain untested here.

## Alpha.6 result

The same real-plugin test passed for alpha.5-to-alpha.6 check, download, exact setup-byte
identity, and rejection of a one-byte-modified setup with the unchanged signature.

- Setup: `target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.6_x64-setup.exe`
- Size: 5,093,916 bytes
- SHA-256: `3c991e61049807bacd1f0ad9265a66156f656af2057c3f165447008e59dc1896`
- Local unpublished feed: `target/cebg-release/0.1.0-alpha.6/`; recipe version alpha.5
- Separate silent NSIS installation exited zero; installed EXE reports alpha.6.

The existing compiled acceptance harness was reused with `CEBG_UPDATER_SETUP`,
`CEBG_UPDATER_VERSION=0.1.0-alpha.6`, and `CEBG_UPDATER_CURRENT_VERSION=0.1.0-alpha.5`.
Automatic apply/restart remains untested; this manual local upgrade does not replace it.

## Alpha.7 result

The real-plugin check/download/byte-identity/tamper-rejection test passed again for
alpha.6-to-alpha.7. Signed setup size is 5,100,157 bytes; SHA-256
`4edeb0d2eb2a201edde82766acf64610616830f6a62cb9bf77d2ea0da28553ae`.
The setup is `target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.7_x64-setup.exe`.
The local unpublished feed is `target/cebg-release/0.1.0-alpha.7/`, with recipe alpha.6.
Separate silent NSIS installation exited zero and the installed EXE reports alpha.7.
Automatic updater apply/restart remains untested.

## Alpha.8 result

The existing real-plugin harness passed alpha.7-to-alpha.8 feed check, full download,
signature verification, exact setup-byte identity, and changed-artifact rejection
(one test passed, exit zero). The signed setup is
`target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.8_x64-setup.exe`:

- Size: 5,102,476 bytes.
- SHA-256: `151d4f7ac8faab61aa3024cef6c13ffe9c348038afacfed103ef9afd3354fa16`.
- Signature SHA-256: `8079a37e285b7f966ecbcf0afd654d855565fdda43319eb51b6569a2337c1c1f`.
- Local unpublished feed: `target/cebg-release/0.1.0-alpha.8/`, recipe alpha.7.
- Feed SHA-256: `6a37b0ff1e973186c44b64a84ca112d715c77708715c7bbfe8c839ed2b53a724`.
- All three checksum-manifest entries independently rehashed and matched.

A separate silent local NSIS upgrade exited zero and the installed EXE reports alpha.8
at `target/nsis-smoke-20260905-022935/chriz-bg-app.exe`. It remains closed while the
independent CLI game installation runs. Automatic updater apply/restart and the public
channel remain unproven/unpublished; the manual upgrade does not substitute for either.

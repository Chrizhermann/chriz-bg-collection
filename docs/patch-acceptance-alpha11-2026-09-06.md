# First public patch acceptance — app alpha.11 / recipe alpha.12

## Scope

Christopher authorized fixing the reported SoD Remix component 900 installation
failure and fully releasing the correction. The preceding approved clarity/support
changes and diagnostic summary are included in the patch. No existing game, stream
installation, source game, or save was modified; no full installation was restarted.

The user's supplied diagnostics establish a three-item `Container009` rejected by
SoD Remix v0.6.7's exact-one-sword guard, a zero-file rollback for 900, and subsequent
successful 910 installation. The earlier 310 active rows remain unchanged. See
[the bounded recovery assessment](issues/sod900-community-recovery-2026-09-06.md).
This is not evidence of a Steam-version defect or user error.

## Installer-side changes and verification

- Failure/restart views now distinguish unsupported automatic resume from proved
  irreparability and ask users to preserve the folder/export diagnostics. Resume
  guards, terminal seals and status codes are unchanged; there is no automatic repair
  button in this patch.
- The existing recovery planner has a regression for exactly the 31-of-32 SoD shape:
  rollback only the failed run's 31 active rows, reinstall the corrected 32, then finish
  the remaining runs. This is a planning/acceptance capability, not executed recovery.
- Diagnostic JSON/JSONL now preserves valid syntax when redacting secret-bearing
  URLs/values. Private paths in values and map keys remain protected; raw on-disk
  evidence is unchanged. The supplied ZIP was inspected in memory only and not committed.
- Frontend typecheck and all 109 tests passed. Native command contracts: 39 passed;
  normal packaging contracts: 3 passed. Engine diagnostics/receipt/recovery/update
  checks: 42 passed (10/10/8/7/7). Existing ignored signature acceptance requires the
  new signed installer and runs separately during packaging.
- The public-credit test initially caught an old expected app version after the version
  bump; app/recipe expectations were updated with the release markers. The map-key
  redaction regression caught over-redaction of an ordinary credential-key name and was
  fixed before the 42-test passing run.
- No new dependencies, authentication changes, endpoints or game mutation APIs were
  added. Security review focused on diagnostic redaction and preserving evidence.

## Release integration

SoD Remix v0.6.8 is publicly released at commit
`6c308f5facb6837b78f7e161613bb972c08b201b`. The official ZIP was freshly downloaded
and extracted with the production author verifier: 1,513,381 bytes, SHA-256
`29eb10537ebf759608da93b8764acfc678cd301bcef24e14cc860db01b33efbb`.
The mod owner reproduced the old three-item failure and checked the packaged fix,
including preservation of existing item records and 900/910 install ordering; 92
mod tests passed. No full-stack/gameplay acceptance is inferred from those fixtures.

CEBG public-alpha recipe validation has no findings. The actual Rust-resolved plan
has 43 runs / 434 components and matches every public inventory run/component in
order; BuffBot remains last with `[1, 0]`. Curated clarity tests: 4 passed. Production
recipe tests: 7 passed / 1 expected external-download test ignored; the new SoD
artifact was independently verified online above. Python tool tests: 35 passed.
The production-evidence test caught stale SoD archive identities/observations;
they were refreshed using the new ZIP with a separate September 6 verification note.

Signed Windows x64 NSIS build succeeded:

- App `0.1.0-alpha.11`, bundled recipe `0.1.0-alpha.12`.
- Setup `Chriz.Easy.BG_0.1.0-alpha.11_x64-setup.exe`: 5,355,536 bytes.
- SHA-256: `54e4b4c52838489ed8970170627ba192011bd0e37c3ffceb91f8870167818675`.
- Fresh package signature test passed against the bundled updater public key.
- Existing working headless real-Tauri updater harness offered alpha.11 over alpha.10,
  downloaded and verified the exact setup, and rejected a byte-tampered copy. This
  uses the unchanged updater code/key, not a newly built mock harness (private issue 2).
- Windows Defender custom file scan with remediation disabled reported no threats;
  this is not a guarantee of safety or Windows Authenticode signing.

App alpha.11 is required for the additive customization schema; recipe alpha.12 declares that minimum.
The patch is delivered through the signed application update with its bundled recipe;
it does not introduce a separately configured remote recipe-signing channel.

Native updater apply/restart, native-window visual acceptance, and another full game
installation are not claimed by the automated/package checks. The user's own test and
actual app update remain the next live acceptance. Existing failed installations are
not retroactively marked successful by installing the app patch.

## Public publication

Source integration commit `30b15e5` was pushed to the private collection branch
`codex/installer-v0-real-alpha`. Source repository visibility was rechecked as private.
The public [alpha.11 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.11)
is published (not draft, explicitly prerelease) with setup, signature, feed, SHA256SUMS,
434-component inventory, license and third-party notices. No third-party mod archives
or tester data were uploaded.

The publicly downloaded setup matches the exact length/hash above. Its adjacent
signature and versioned feed match, and the real updater harness passed again on the
publicly downloaded bytes, including tamper rejection. The mutable
[alpha feed](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/alpha/latest.json)
was replaced only after versioned assets passed; anonymous retrieval confirms app
alpha.11 / recipe alpha.12 and the exact correct setup URL/signature. Old versioned
assets remain unchanged. A verified copy is available in Christopher's Downloads.

Website owner `Build interactive BG run page` received the verified public URL/hash,
inventory and bounded release notes. Deployment confirmation is pending.

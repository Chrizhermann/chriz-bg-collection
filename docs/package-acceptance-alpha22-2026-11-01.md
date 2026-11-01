# Alpha.22 package verification — November 1

**Superseded the same evening:** Christopher decided that this release is **0.1.0**, with
the same content. This candidate was never published; the 0.1.0 candidate and its checks
are recorded in [0.1.0 package verification](package-acceptance-0.1.0-2026-11-02.md). This
record stays as the evidence for the alpha.22 binary, which is kept in the owner's test
folder.

App **0.1.0-alpha.22**, bundled collection **0.1.0-alpha.22**. Build source:
`52f797babeff5c878416b7b7d6bfc60ec9c46d69` (PR #76). This is a release candidate for
Christopher's own test; nothing is tagged, published or uploaded. Later documentation does
not alter the verified package.

## Why this release

The collection alpha.22 recipe needs app alpha.22: older apps reject its Challenge Mode
variants and the Tweaks Anthology configuration file, and its update ledger requires this
app version (#67). The candidate carries the new recommended mod setup and the installer
fixes since alpha.21; see the [release notes](release-notes-alpha22.md).

## Exact package

- Setup: `Chriz Easy BG_0.1.0-alpha.22_x64-setup.exe` (published name
  `Chriz.Easy.BG_0.1.0-alpha.22_x64-setup.exe`), **6,078,951 bytes**.
- Setup SHA256: `1101253fa38c2d90aad71c3f7d0a8fff7e6b80bfae73c8ebd1faab0fd45f45a0`.
- Signature SHA256: `34a28dc9101680269ec203b30f47903deffb6a1183a3cbd6352ca6f68d13e018`
  (436 bytes).
- Local feed SHA256: `bbffb315c55ac24a5c5f6a9d21236f8a99508314c9b717cbbc082f24039ef299`
  (`latest.json`, unpublished); `SHA256SUMS`:
  `084dafd4ad8299e422f02e1fd52057ebaa18eaa63fa9f5ed07ba6f7cd718a674`.
- Extracted app SHA256: `ddbb8d8ecc40e09f2d8175336c3787ffe4c2c3f7e2d706ff7b278da90a22d537`
  (25,042,432 bytes). It differs from the compiler output only in Tauri's bundle-type
  marker: three bytes, `UNK` → `NSS`.
- All **384 bundled resources** matched source bytes (alpha.21: 239), with none missing and
  no unexpected file: the license, the notices, 185 runtime manifest files and 197 files of
  the curated recipe.
- The player build carries no test hook: the app contains none of the
  `CHRIZ_BG_COLLECTION_TEST_*` names, and the production dependency tree enables none of
  the test features (`docs/BUILDING.md`). The setup has no Authenticode signature, as
  before.
- Public component inventory (`component-inventory.json`, 73,593 bytes, SHA256
  `0f0470c541e5dd4f5cc17e430b577483a70de26105507c3e31bbb63ce1a96cb1`): **476 default
  components, 70 runs, 56 mod entries** (alpha.21: 469 / 53 / 39).
- A verified copy of the setup and its signature was saved to the owner's Downloads folder
  for review; both hashes matched after copying.

### Component inventory against alpha.21

- Added: Sirene NPC for BG:EE (3 components) and the 18 player voicepacks (1 each).
- Left out of Chris's Recommended, now optional: BG1 Unfinished Business (16) and
  Unfinished Business for BG2 (18), and Tweaks Anthology `2292` (druids on the cleric XP
  table).
- New in existing runs: Modpack `135`, `230`–`234`, `239`, `460`; BG Rebalance `331`,
  `332`, `340`, `360`, `410`, `411`; SoD Remix `292`; SCS `3021`; Tweaks Anthology
  `3220`, `3500`, `3510`; Artisan's Kitpack tweaks `7205` and `1010000`.
- Renamed: Tweaks Anthology `1257` (the NPC move now names Ajantis) and `2210` (names the
  +3 to hit and +5 damage).
- Versions: Artisan's Kitpack fork chriz.1 → chriz.5 (all three installers), Bardic
  Wonders balance.7 → balance.8, BuffBot 1.10.0 → 1.12.6, Tweaks Anthology 18 → `7649ced`,
  Modpack alpha.11 → alpha.14, BG Rebalance 0.8.2 → 0.8.8, SoD Remix 0.6.13 → 0.6.16, EEex
  1.2.0 → 1.3.1, and the Spell Revisions fork and its Rogue Rebalancing compatibility tail
  chriz.5 → chriz.7.
- No component moved to another run. Net: 469 − 35 + 42 = 476.

## Automated checks

The final run completed with exit **0** on a clean checkout of the build source, at
BelowNormal priority. Tracked files were unchanged; the ignored build outputs of the
superseded run below were reused. Logs and the source record are under
`target/work/nov01-alpha22-rc`.

- Frontend typecheck, **452 tests / 37 files**, and production build passed.
- Native workspace: **1,155 passed**, zero failures; **17 explicitly opt-in tests
  ignored** (alpha.21: 14). The three new ones came with their features and need external
  inputs: the uninstaller hook test (makensis), the real loader-settings test (game-log
  switch) and the real-WeiDU bare-batch-name test (EET shortcut fix). No test was skipped
  to make a check pass.
- Formatting, Clippy with warnings denied, dependency notices and **265 Python tests**
  passed (the existing optional real-WeiDU pilot skipped).
- Native TP2-order rechecks: all 36 pins that changed since collection alpha.20 were
  downloaded anonymously and fresh, and each matched its pinned length and SHA-256. The
  order check (`tools/check_native_component_order.py`; for the `.iemod` archives, its own
  parsing functions) passed for every run whose TP2 it can read, in both recipes, with one
  exception below. Where it cannot read a TP2's component numbers (EEex, NTotSC, Sirene NPC
  for BG:EE, 14 voicepacks and two `.iemod` mods), the TP2 was byte-identical to the one
  the intake audited by hand. Every TP2 matched the hash its intake recorded.
- **One order failure, not new:** the `manifest/` profile ("Smaller downloadable alpha")
  requests the Artisan's Kitpack main components `10002` before `10001` and `1100` after
  `1009`, against the TP2's declaration order. It has done so since chriz.1, and three
  intake records note it. The default curated profile passes. With WeiDU installing a batch
  in TP2 order and the engine expecting the requested order, an installation with that
  profile would likely stop at this run; not reproduced. Tracked in
  [issue 77](https://github.com/Chrizhermann/chriz-bg-collection/issues/77).
- Package/signature checks passed, including the opt-in check against this exact setup and
  the bundled updater public key.
- The opt-in real Tauri updater download check passed from alpha.21 to alpha.22: version
  discovery, exact signed bytes accepted, deliberately altered bytes rejected. It did not
  install the update or test updater restart.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS` to
  `target/cebg-release/0.1.0-alpha.22`, next to the inventory; nothing was uploaded.

The first run, on `2933b8e`, failed only the dependency-notice check: the version bump
changed `Cargo.lock` and `app/package-lock.json`, whose LF-normalized digests
`THIRD_PARTY_NOTICES.md` records. Alpha.21 refreshed them in a separate commit after its
bump (`8d82c8e`); `52f797b` does the same, two digest lines. All its other checks passed
with the same counts. No package was built from `2933b8e`; its logs are under
`target/work/nov01-superseded-2933b8e`.

The Tauri build rewrote `app/src-tauri/Cargo.toml` with LF line endings in this
checkout, which uses CRLF. Its content equals the committed blob (`1a89782`), so the binary
corresponds to the build source; the checkout was restored afterwards.

The setup was signed with the existing updater key, passed to the signer by path only. No
key was created, copied or rotated.

## Defender and runtime boundaries

A single-file custom scan of the exact setup bytes returned **no threats**, exit 0, with
security intelligence **1.463.70.0** (engine 1.1.26100.4). The scan of the build output
itself reported the file as skipped, without a reason; the exclusion list needs
administrator rights to read and was not inspected. The scan therefore ran on a
byte-identical copy (same SHA-256) in a temporary folder. Antivirus, real-time,
behavior-monitor and downloaded-file protection remained enabled. No exclusions or security
preferences were changed. The extracted contents were not scanned: the owner was streaming,
and only the setup's single-file scan was wanted.

No setup, app or game was launched, and no window was opened. The uninstaller was not run.
No game, lab, stream installation or saved game was touched.

These are bounded clean-scan results, not a Microsoft guarantee or a Windows Authenticode
publisher certificate.

## Before publication

- Christopher's own test of this candidate, and his OK.
- The ledger's `published_at` is still the placeholder from #67. The app only requires it
  to be nonempty and shows it nowhere; setting it means a new build.
- The Updates page does not show the ledger's changes in this app version: its update check
  returns an empty change list for the collection, and the signed recipe channel that would
  fill it is not configured. The release notes say so.
- A decision on issue 77, and the lab run of the Recommended selection
  (`docs/curation/components/FOLLOW_UPS.md`).
- Not in this candidate: #69 (install what a mod's official link serves when it differs
  from the pin) and #53 (own-mod pin check), both open against the next-release branch.
- `npm audit` finds no vulnerability in the production dependencies; its four findings are
  in development tools (Vite's dev server, Vitest, source-map-js), which the setup does not
  contain.

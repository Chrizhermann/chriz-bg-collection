# 0.1.0 package verification — November 1

App **0.1.0**, bundled collection **0.1.0**. Build source:
`e91e9e2ed2c1e6e11038b5546663b90396b7a05b` (PR #76). This is a release candidate for
Christopher's own test; nothing is tagged, published or uploaded. Later documentation does
not alter the verified package.

## Why this release

Christopher decided on 2026-11-01 that the release prepared as alpha.22 is **0.1.0**,
without a pre-release suffix and with the same content. The collection recipe needs app
0.1.0: older apps reject its Challenge Mode variants and the Tweaks Anthology configuration
file, and its update ledger requires this app version (#67). The candidate carries the new
recommended mod setup and the installer fixes since alpha.21; see the
[release notes](release-notes-0.1.0.md).

## Differences from the alpha.22 candidate

The [alpha.22 candidate](package-acceptance-alpha22-2026-11-01.md) of the same evening is
superseded. Against it:

- **Version:** app and bundled collection are 0.1.0. The update ledger moved to
  `releases/v0.1.0/ledger.toml` in both profiles, needs app 0.1.0 and is published on
  2026-11-02 (`2026-11-02T00:00:00Z`; it was a placeholder). It still supersedes alpha.20,
  and its 231 recorded differences are unchanged, because `release.json` and the ledgers
  are not part of the recipe diff. The updater endpoint and the `alpha` channel are
  unchanged; 0.1.0 orders after every 0.1.0 alpha.
- **Issue 77 is fixed:** the `manifest/` profile, now called "Smaller downloadable setup",
  requests the Artisan's Kitpack main run and the Randomiser run in their TP2's
  declaration order. `tools/tests/test_native_run_orders.py` holds both bundled profiles to
  the generator's audited orders and to every native order an intake recorded.
- **Wording:** player-visible texts no longer call the product an alpha (the profile
  label, two Updates texts, the GOG and locale findings, one busy-installer detail, the
  troubleshooting guide). Internal names such as `public-alpha` stay.
- **Notices:** the lockfile digests are refreshed for the new lockfiles.
- **Same mods:** the public component inventory equals the alpha.22 candidate's except
  for its two version fields.

## Exact package

- Setup: `Chriz Easy BG_0.1.0_x64-setup.exe` (published name
  `Chriz.Easy.BG_0.1.0_x64-setup.exe`), **6,079,798 bytes**.
- Setup SHA256: `ff4f8d34d05cebfc4c39148d39ec795257ef1d2a4baaaf6adda4d4349a5c1f97`.
- Signature SHA256: `5c12f0a7230419ee79b6551585d2f3d2ab6b5a2e6775e7e78c49f95d8391beb3`
  (424 bytes).
- Local feed SHA256: `b0aa31ec506e330fadb301eccf7b7ba21a75b4f866fcd6f8f52c5f61fb56a34f`
  (`latest.json`, unpublished; it points to the `v0.1.0` release asset); `SHA256SUMS`:
  `1d685ee8fee1298058227d10a3dbafb0b58f0359716a57b882093c6ff9ee1f8c`.
- Extracted app SHA256: `00556bb9e0aacc48270458edf076f7e8d67b1b00efb4dbb5ca571e4a3f4d4819`
  (25,041,920 bytes). It differs from the compiler output only in Tauri's bundle-type
  marker: three bytes, `UNK` → `NSS`.
- All **384 bundled resources** matched source bytes, with none missing and no
  unexpected file: the license, the notices, 185 runtime manifest files and 197 files of
  the curated recipe.
- The player build carries no test hook: the app contains none of the
  `CHRIZ_BG_COLLECTION_TEST_*` names, and the production dependency tree enables none of
  the test features (`docs/BUILDING.md`). The setup has no Authenticode signature, as
  before.
- Public component inventory (`component-inventory.json`, 73,575 bytes, SHA256
  `b54d617118a63f62033e916e1d44723ec07b3a3b584c4142b7d4c7df0c38b35b`): **476 default
  components, 70 runs, 56 mod entries** (alpha.21: 469 / 53 / 39).
- A verified copy of the setup and its signature was saved to the owner's Downloads folder
  for review; both hashes matched after copying. The alpha.22 candidate's setup and
  signature were moved from there into the owner's test folder, with matching hashes.

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
- No component moved to another run. Net: 469 − 35 + 42 = 476. The inventory follows the
  curated recipe, so the issue 77 fix, which changes the order inside two `manifest/`
  runs, does not appear in it.

## Automated checks

The run completed with exit **0** in a new worktree: a fresh, clean checkout of the build
source with no build outputs, its own target folder, BelowNormal priority and capped
parallelism. Logs and the source record are under `target/work/nov02-0.1.0-rc`.

- Frontend typecheck, **452 tests / 37 files**, and production build passed.
- Native workspace: **1,155 passed**, zero failures; **17 explicitly opt-in tests
  ignored**, the same set as in the alpha.22 candidate. No test was skipped to make a
  check pass.
- Formatting, Clippy with warnings denied, dependency notices and **267 Python tests**
  passed (the existing optional real-WeiDU pilot skipped). The two new tests are the
  native-order checks of issue 77.
- Native TP2-order proof at the build source: the Artisan's Kitpack chriz.5 and Randomiser
  v8.1.1 archives were downloaded fresh, matched their pins, and
  `tools/check_native_component_order.py` accepted the Kitpack main, NPC and tweak runs
  and the Randomiser run in both profiles. Before the fix it rejected the `manifest/`
  Kitpack main and Randomiser runs. The other 35 pins that changed since collection alpha.20
  are identical to the alpha.22 candidate's and passed its fresh recheck the same day;
  `test_native_run_orders.py` now checks every run with a recorded native order (104 runs
  across both profiles) on every test run.
- Package/signature checks passed, including the opt-in check against this exact setup and
  the bundled updater public key.
- The opt-in real Tauri updater download check passed from **alpha.21 to 0.1.0**: the
  updater offered 0.1.0 to an alpha.21 app, accepted the exact signed bytes and rejected
  deliberately altered bytes. It did not install the update or test updater restart.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS` to
  `target/cebg-release/0.1.0`, next to the inventory; nothing was uploaded.

Before this acceptance, a development run of the same changes hit the known failure class
of [issue 9](https://github.com/Chrizhermann/chriz-bg-collection/issues/9) once: an
installation-removal contract test failed because a `weidu.exe` that it could not open was
running on the PC. It passed alone right afterwards, and this acceptance run had no
failure; the occurrence is recorded in that issue.

The Tauri build again rewrote `app/src-tauri/Cargo.toml` with LF line endings in this
checkout, which uses CRLF. Its content equals the committed blob (`c3e74fc`), so the binary
corresponds to the build source; the checkout was restored afterwards.

The setup was signed with the existing updater key, passed to the signer by path only. No
key was created, copied or rotated.

## Defender and runtime boundaries

A single-file custom scan of the exact setup bytes returned **no threats**, exit 0, with
security intelligence **1.463.70.0** (engine 1.1.26100.4). As for the alpha.22 candidate,
the scan of the build output itself reported the file as skipped, so the scan ran on a
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
- The Updates page does not show the ledger's changes in this app version: its update check
  returns an empty change list for the collection, and the signed recipe channel that would
  fill it is not configured. The release notes say so.
- The lab run of the Recommended selection (`docs/curation/components/FOLLOW_UPS.md`).
- Not in this candidate: #69 (install what a mod's official link serves when it differs
  from the pin) and #53 (own-mod pin check), both open against the next-release branch.
- The README still describes app and collection alpha.20 as the published release; update
  it together with the website at publication.
- `npm audit` finds no vulnerability in the production dependencies; its four findings are
  in development tools (Vite's dev server, Vitest, source-map-js), which the setup does not
  contain.

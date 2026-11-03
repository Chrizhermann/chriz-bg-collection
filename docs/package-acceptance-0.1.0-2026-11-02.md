# 0.1.0 package verification — November 2

App **0.1.0**, bundled collection **0.1.0**. Build source:
`50a42395be6112a687a9d0bf81dd2a23d1849b95` (PR #76). It was verified as a release candidate
for Christopher's own test and published on 2026-11-02 (see [Publication](#publication)).
Later documentation does not alter the verified package.

## Why this rebuild

Christopher took two content fixes into 0.1.0 on 2026-11-02, both released on 2026-11-01
(UTC): the Artisan's Kitpack fork chriz.6 and BG Rebalance v0.8.9. The version, the
ledger's date and the updater channel stay. The first 0.1.0 candidate
([November 1 record](package-acceptance-0.1.0-2026-11-01.md)) is superseded; see the
[release notes](release-notes-0.1.0.md) for the whole release.

## Differences from the first 0.1.0 candidate

- **Artisan's Kitpack fork chriz.6** replaces chriz.5 for the main, NPC and tweak
  installers in both profiles. The Shapeshifter's white and greater white werewolf forms
  had no attack 1 frames in the five base directions, which crashes BG2:EE 2.7.3.0 or
  corrupts memory near a wall; both files are now copies of the same fur's attack 2. The
  three TP2s change only their VERSION, so components, order and decisions stay.
- **BG Rebalance v0.8.9** replaces v0.8.8. Pierce Magic `210` and Spellstrike `211` get
  descriptions in Spell Revisions' layout that match their effects, and their scrolls, the
  Wand of Spell Striking and sibling spells show the same texts. No mechanic, component or
  selection changes: both profiles still install `210` and `211` as defaults, and the
  texts-only components `212`/`213` for existing installations ship in a separate package
  that neither profile offers or installs.
- **Ledger:** the pins add two recorded differences, the two new artifacts, so the 0.1.0
  ledger now covers **233**, from `e0b5116` (the recipe of app alpha.20 and alpha.21) to
  `9bb496a` (the pin commit). Packaging the curated trees of `e0b5116`, `e9d90e3` and
  `9bb496a` with `chriz-bg-author package-recipe` and comparing them with
  `diff_recipe_packages` reproduced the 231 earlier subjects and gave exactly the two new
  ones. Two new entries cover them (34 in all). Version 0.1.0, the publication date
  2026-11-02, the predecessor alpha.20 and the required app 0.1.0 are unchanged.
- **No app or engine code changed** since the first candidate's build source `e91e9e2`:
  only recipes, evidence, tests and documentation. The dependency notices are unchanged.
  The compiled app differs from the first candidate's in 24 bytes, the PE link timestamp
  and the PDB identifier.
- **Same components:** the public component inventory equals the first candidate's except
  for four version fields, Artisan's Kitpack chriz.5 → chriz.6 (main, NPC and tweak
  installers) and BG Rebalance 0.8.8 → 0.8.9.

## Exact package

- Setup: `Chriz Easy BG_0.1.0_x64-setup.exe` (published name
  `Chriz.Easy.BG_0.1.0_x64-setup.exe`), **6,081,271 bytes**.
- Setup SHA256: `295b6ac614430e8ca560d2801b2c4f57ad256229543e7777080d639f2e692d58`.
- Signature SHA256: `cb34f67fbfa7fac5b33d588641ff62423b801472ca737372e2599774a9e5bc2c`
  (424 bytes).
- Local feed SHA256: `4a8d8a2e8004afc4e9990a7a0b3b45b93d0c6894af3931c40a2a49e7e5713d06`
  (`latest.json`, unpublished; it points to the `v0.1.0` release asset, and its notes now
  also name the white werewolf crash fix); `SHA256SUMS`:
  `dbef35caea2f12b9830c3b29cc8b564c5fda9400d714f566cf51e6c6f44c4c06`.
- Extracted app SHA256: `bb208413bcb70b5c52876298a1acc9da14da4a6b4d1de9756b32646441f871cf`
  (25,041,920 bytes). It differs from the compiler output only in Tauri's bundle-type
  marker: three bytes, `UNK` → `NSS`.
- All **388 bundled resources** matched source bytes, with none missing and no
  unexpected file: the license, the notices, 187 runtime manifest files and 199 files of
  the curated recipe. Each profile has two files more than before, the two new artifact
  pins.
- The player build carries no test hook: the app contains none of the
  `CHRIZ_BG_COLLECTION_TEST_*` names, and the production dependency tree enables none of
  the test features (`docs/BUILDING.md`). The setup has no Authenticode signature, as
  before.
- Public component inventory (`component-inventory.json`, 73,575 bytes, SHA256
  `11106e4e7d4a68dd86fddff87e3ad889bbb7148c8b0ad70bc90bf70a9a3eb05c`): **476 default
  components, 70 runs, 56 mod entries** (alpha.21: 469 / 53 / 39). Against alpha.21 it
  differs exactly as the first candidate's did (see its record), with Artisan's Kitpack now
  chriz.1 → chriz.6 and BG Rebalance 0.8.2 → 0.8.9.
- A verified copy of the setup and its signature was saved to the owner's Downloads folder
  for review; both hashes matched after copying. The first 0.1.0 candidate's setup and
  signature (`ff4f8d34…`, `5c12f0a7…`) were moved from there into the owner's test folder,
  with matching hashes.

## Automated checks

The run completed with exit **0** in a new worktree: a fresh, clean checkout of the build
source with no build outputs, its own target folder, BelowNormal priority and at most four
parallel build jobs and test threads. Logs and the source record are under
`target/work/nov02-0.1.0-rc2`.

- Frontend typecheck, **452 tests / 37 files**, and production build passed.
- Native workspace: **1,155 passed**, zero failures; **17 explicitly opt-in tests
  ignored**, the same set as in the first candidate. No test was skipped to make a check
  pass, and no `weidu.exe` was running before or after the tests.
- Formatting, Clippy with warnings denied, dependency notices and **272 Python tests**
  passed (the existing optional real-WeiDU pilot skipped). The five new tests are the
  November 2 intake checks; the suite also passed at the pin commit `9bb496a` (271 tests).
- Native TP2-order proof at the build source: the Artisan's Kitpack chriz.6 and BG
  Rebalance v0.8.9 archives from the November 2 intake's fresh download (length and
  SHA-256 re-checked) were accepted by `tools/check_native_component_order.py` for the Kitpack
  main, NPC and tweak runs and the Rebalance run, in both profiles. No other pin changed
  since the first candidate.
- Package/signature checks passed, including the opt-in check against this exact setup and
  the bundled updater public key.
- The opt-in real Tauri updater download check passed from **alpha.21 to 0.1.0**: the
  updater offered 0.1.0 to an alpha.21 app, accepted the exact signed bytes and rejected
  deliberately altered bytes. It did not install the update or test updater restart.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS` to
  `target/cebg-release/0.1.0`, next to the inventory; nothing was uploaded.

The Tauri build again rewrote `app/src-tauri/Cargo.toml` with LF line endings in this
checkout, which uses CRLF. Its content equals the committed blob (`c3e74fc`), so the binary
corresponds to the build source; the checkout was restored afterwards.

The setup was signed with the existing updater key, passed to the signer by path only. No
key was created, copied or rotated.

## Defender and runtime boundaries

A single-file custom scan of the exact setup bytes returned **no threats**, exit 0, with
security intelligence **1.463.79.0** (engine 1.1.26100.4). As for the earlier candidates,
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
- The lab run of the Recommended selection, now from this build source
  (`docs/curation/components/FOLLOW_UPS.md`, step 3).
- Neither fix has been played in a game installed from these releases yet. The werewolf
  repair was lab-tested as a file copy into `override` (the greater white fur crashed after
  107 s before and ran 424 s without a crash after), and the white fur was checked offline
  only. The new Pierce Magic and Spellstrike texts passed the Rebalance owner's tests and
  dry runs but have not been seen in a running game.
- The Updates page does not show the ledger's changes in this app version: its update check
  returns an empty change list for the collection, and the signed recipe channel that would
  fill it is not configured. The release notes say so.
- Not in this candidate: #69 (install what a mod's official link serves when it differs
  from the pin) and #53 (own-mod pin check), both open against the next-release branch.
- The README still describes app and collection alpha.20 as the published release; update
  it together with the website at publication.

## Publication

Christopher gave the go on 2026-11-02 after testing the final lab install of this build
source. PR #76 merged into the next-release branch as `44235d8` (11:55 UTC), and
[0.1.0](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0) was published
seconds later as a normal release, marked Latest, with the tag `v0.1.0` on the distribution
repository's `main`, like every alpha. 0.1.0 drops the alpha label, so it is not marked as a
pre-release. All seven versioned assets were downloaded anonymously and matched the hashes
above: setup, signature, feed, checksums, component inventory, license and notices. The
license and notices are the committed file bytes.

The alpha channel, which every installed CEBG version reads, serves the 0.1.0 feed since
11:56 UTC; the previous alpha.21 feed is kept for rollback. An anonymous download of the
channel feed matches the feed hash above. Versioned assets are immutable.

The [collection website](https://bg.chrizfader.org/collection) was updated with website
PR 45, merged as `867438c` and live from 20:57 local time; its live content file names 0.1.0
with the setup URL and hash above. Issue #77 was closed by hand, because PR #76 merged into
the next-release branch rather than `main`.

# 0.1.1 package verification — November 3

App **0.1.1**, bundled collection **0.1.1**. Build source:
`f8554418dc3a73401b9719b27a24ac8488de8b32` (PR #98), merged into the next-release branch
as `161bb72e9b12efc04a59153c6b0d5672d88a03d6` with an identical tree (`c8503494…`), so
the package corresponds to the branch at that merge. This is a release candidate for
Christopher's own test; nothing is tagged, published or uploaded. Later documentation does
not alter the verified package.

## What this release changes

See the [release notes](release-notes-0.1.1.md). Against 0.1.0:

- **Installer:**
  - GOG, Beamdog and other copies (#89).
  - Retries while a file is held (#90).
  - Failed optional runs left out (#91).
  - Newer manual downloads (#92).
  - Names, folders and disk space (#93).
  - The modded-game question (#96).
  - The download questions (#69, #80).
  - The saves & portraits folder button (#82).
- **Collection:** BG1 NPC Project is optional and off in Chris's Recommended (#88), and
  Chriz BG Modpack v0.2.0-alpha.15 adds Marek's Boomerang Dagger `670` as a default (#97).
- **Version:** app 0.1.1 in the five files the 0.1.0 bump changed; the notices record the
  new lockfile digests.
- **Ledger:** `releases/v0.1.1/ledger.toml` in both profiles, two entries. It supersedes
  0.1.0 and needs app 0.1.0. Its four recorded differences run from `9bb496a` to
  `fa0edd6`.
- **Publication date:** the ledger's publication date, `2026-11-03T00:00:00Z`, is fixed
  before packaging because the setup bundles the ledger. If publication slips past
  2026-11-04 00:00 UTC, the setup is rebuilt with the real date rather than shipping a
  date that is a day off.

## Exact package

- **Setup:** `Chriz Easy BG_0.1.1_x64-setup.exe`, published as
  `Chriz.Easy.BG_0.1.1_x64-setup.exe`. **6,413,423 bytes**.
  - SHA256: `0d20eac8175ffecbf8a29f3c24e7c59cce156e949724e2ca5273f5723d631652`.
- **Signature:** 424 bytes.
  - SHA256: `7199df34f112b411ccde11ac545672d04023dc085692b4bafe461a037c776d31`.
- **Local feed** (`latest.json`, 1,129 bytes, unpublished; it points to the `v0.1.1` release
  asset):
  - SHA256: `d1261567a2bb77ce2e54f4eb83b3ef91fbe41175abb19d19bc923b4ba1478b94`.
  - `SHA256SUMS`: `c45b76692c5a1b9410a8895244a982e58b6e7233d64db7c6a4a5dce1c6775339`.
- **Extracted app** (26,537,472 bytes):
  - SHA256: `f1f3cf06ad56274016cccc9872dab11e34df333c706ac99e90da0cd039d34675`.
  - It differs from the compiler output only in Tauri's bundle-type marker: three bytes,
    `UNK` → `NSS`.
- **Bundled resources:** all **392** matched source bytes, with none missing and no
  unexpected file. They are the license, the notices, 189 runtime manifest files and 201
  files of the curated recipe.
- **No test hook:** the app contains none of the `CHRIZ_BG_COLLECTION_TEST_*` names, and
  the production dependency tree enables none of the test features (`docs/BUILDING.md`).
  The setup has no Authenticode signature, as before.
- **Public component inventory** (`component-inventory.json`, 72,468 bytes):
  - SHA256: `74d51f1cae3f1a38ff3e3770927b65971f0f2e2f31643a7d1860e20407b0f7a4`.
  - **468 default components, 69 runs, 55 mod entries** (0.1.0: 476 / 70 / 56).
- **Verified copies** of the setup, its signature, the feed, the checksums, the inventory,
  the license and the notices are in the owner's test folder for the lab install and the
  publication. The license and notices are the committed file bytes, and all hashes matched
  after copying. The prepared `alpha` channel feed equals the versioned feed. The rollback
  copy is the published 0.1.0 feed (`4a8d8a2e…`).

### Component inventory against 0.1.0

- Removed from Chris's Recommended, now optional: BG1 NPC Project (8 components, its own
  run) and the Kivan quest guard Modpack `130`, which follows it.
- New: Modpack `670` (Marek carries the Boomerang Dagger +2) in the main post-EET_end
  Modpack run.
- Versions: Chriz BG Modpack alpha.14 → alpha.15.
- No component moved to another run. Net: 476 − 9 + 1 = 468.

## Automated checks

The run completed with exit **0** in a new worktree. It was a fresh, clean checkout of the
build source with no build outputs, its own target folder, BelowNormal priority and at most
four parallel build jobs and test threads.

The signed build ran first, so that the lab could start; the package checks and the full
suite followed in the same checkout. Tauri again rewrote `app/src-tauri/Cargo.toml` with LF
line endings in this CRLF checkout. Its content equals the committed blob (`72c07cb`), so
the binary corresponds to the build source, and the file was restored before the suite.
The suite therefore ran with no tracked change and only the build's ignored outputs
present.

- Frontend typecheck, **478 tests / 38 files**, and production build passed.
- Native workspace: **1,336 passed**, zero failures; **17 explicitly opt-in tests
  ignored**, the same set as in 0.1.0. No test was skipped to make a check pass, and no
  `weidu.exe` was running before or after the tests.
- Formatting, Clippy with warnings denied, dependency notices and **281 Python tests**
  passed. The existing optional real-WeiDU pilot was skipped.
- Native TP2-order proof at the build source: the Chriz BG Modpack v0.2.0-alpha.15 archive
  from the November 3 intake's fresh download (length and SHA-256 re-checked) was accepted
  by `tools/check_native_component_order.py` for all nine Modpack runs, in both profiles.
  No other pin changed since 0.1.0.
- Package/signature checks passed, including the opt-in check against this exact setup and
  the bundled updater public key.
- The opt-in real Tauri updater download check passed from **0.1.0 to 0.1.1**. The updater
  offered 0.1.1 to a 0.1.0 app, accepted the exact signed bytes and rejected deliberately
  altered bytes. It did not install the update or test the updater restart.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS` to
  `target/cebg-release/0.1.1`, next to the inventory; nothing was uploaded.

The setup was signed with the existing updater key, passed to the signer by path only. No
key was created, copied or rotated.

## Defender and runtime boundaries

A single-file custom scan of the exact setup bytes returned **no threats**, exit 0, with
security intelligence **1.463.94.0** (engine 1.1.26100.4).

- As for the earlier candidates, the scan of the build output itself reported the file as
  skipped, so the scan ran on a byte-identical copy (same SHA-256) in a temporary folder.
- Antivirus, real-time, behavior-monitor and downloaded-file protection remained enabled.
  No exclusions or security preferences were changed.
- Only the setup's single-file scan ran; the extracted contents were not scanned.

No setup, app or game was launched, and no window was opened. The uninstaller was not run.
No game, lab, stream installation or saved game was touched.

These are bounded clean-scan results, not a Microsoft guarantee or a Windows Authenticode
publisher certificate.

## Before publication

- Christopher's own test of this candidate, and his OK.
- The lab install of this setup, with the Recommended selection.
- None of the new paths has been tried live from a released setup yet:
  - the download and modded-game questions;
  - a failed optional mod left out;
  - a GOG, Beamdog or Microsoft Store copy;
  - Marek's fights and the dagger drop.
- If publication slips past 2026-11-04 00:00 UTC, rebuild with the real publication date
  in the ledger.
- The Updates page does not show the ledger's changes in this app version: its update check
  returns an empty change list for the collection, and the signed recipe channel that would
  fill it is not configured. The release notes say so.
- Not in this candidate: #53 (own-mod pin check), still open against the next-release
  branch.
- The README describes 0.1.0 as the published release and the `alpha` channel as serving
  it; update both lines together with the website at publication.

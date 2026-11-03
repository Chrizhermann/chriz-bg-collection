# 0.1.1 package verification — November 3

App **0.1.1**, bundled collection **0.1.1**. Build source:
`f8554418dc3a73401b9719b27a24ac8488de8b32` (PR #98), merged into the next-release branch
as `161bb72e9b12efc04a59153c6b0d5672d88a03d6` with an identical tree (`c8503494…`), so
the package corresponds to the branch at that merge. Published on 2026-11-03 (UTC) after
Christopher's OK; see [Publication](#publication). Later documentation does not alter the
verified package.

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

## Lab install

On 2026-11-03 this setup installed the Recommended selection into a disposable lab without
errors: 69 runs, 468 components, the same set as the inventory above.

- Checked in the lab:
  - Marek drops the Boomerang Dagger +2 when killed from the console.
  - True Time Stop holds a weapon poison for about 18 seconds.
  - A slowed creature still takes its poison damage every second.
  - BG1 NPC Project is not installed.
- At load the game reports five missing files, the same five as 0.1.0.
- Not checked in the lab: Marek's fights in the story (checked in the files only), the
  Radar overlay and the saves & portraits button.

## Publication

Christopher OK'd the publication on 2026-11-04 (03:12 KST, 18:12 UTC on 2026-11-03), with
the Spell Revisions known issue below in the release notes. It went out before 2026-11-04
00:00 UTC, so the ledger's publication date stands.

**Own-mods release check.** `tools/own_mod_pins.py check --release` ran on 2026-11-03 at
18:29:45 UTC. Its tree was the next-release branch at `a0b16e6` with the BuffBot hold from
this PR, and Chriz Easy IWD was read from `claude/iwdee-app` at `cabb8f2`. Exit 0:

| Mod | Chriz Easy BG | Chriz Easy IWD | Newest | State |
|---|---|---|---|---|
| BuffBot | v1.12.6 | v1.12.6 | v1.15.0 | held |
| Chriz BG Modpack | v0.2.0-alpha.15 | – | v0.2.0-alpha.15 | newest |
| BG Rebalance | v0.8.9 | – | v0.8.9 | newest |
| SoD Remix | v0.6.16 | – | v0.6.16 | newest |
| EEex Remote Console | v0.2.1 (3d9899076) | v0.2.1 (3d9899076) | v0.2.1 | newest |
| Voicepacks | v1.1.0 | – | v1.1.0 | newest |
| Chriz IWD Encounters | – | v0.1.0 | v0.1.0 | newest |
| Artisan's Kitpack (our fork) | v4.81a-dev.85928f9-chriz.6 | – | v4.81a-dev.85928f9-chriz.6 | newest |
| Bardic Wonders (our fork) | v2.9c-balance.8 | – | v2.9c-balance.8 | newest |
| Spell Revisions (our fork) | v4.21-chriz.7 | – | v4.21-chriz.7 | newest |
| Item Randomiser (our fork) | v8.1.1 | – | v1.0.3 | held |

- **BuffBot v1.15.0 is held** (Christopher, 2026-11-04). 0.1.1 ships v1.12.6, the version
  he tested it with; v1.15.0 goes into both installers for 0.1.2.
- The check, the pin tool and the daily pin job (#53, #55, #72) merged into the
  next-release branch after this build. They change tooling and documentation only: between
  `161bb72` and `a0b16e6`, `manifest/` and `recipes/` differ only in the new
  `manifest/own-mods.toml`, which the app does not read.

**Release.** [v0.1.1](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.1)
was published on 2026-11-03 at 18:29:55 UTC: a normal release, marked Latest, with the
seven files above. An anonymous download of all seven matched:

- the setup, its signature and the feed, with the SHA-256 values above;
- the inventory `74d51f1c…`;
- `LICENSE` `b414bdb5…`;
- `SHA256SUMS` `c45b7669…`;
- `THIRD_PARTY_NOTICES.md` `46e789c7…`.

**Update channel.** Since 18:30:44 UTC the `alpha` release's `latest.json` is the 0.1.1
feed. An anonymous download returned `d1261567…`, the same file as the release asset. The
0.1.0 feed stays as the rollback copy.

**Known issue added at publication.** Without Spell Revisions, a new installation stops at
BG Rebalance `302` (Flail of the Ages balance).
- Cause: SCS writes its own Slow and Haste spells when Spell Revisions is missing, and
  v0.8.9 does not recognize them. The script-engine session's lab without Spell Revisions
  found it on 2026-11-03.
- A second lab without Spell Revisions and with `302` off finished cleanly (450
  components).
- Workaround: keep Spell Revisions ticked, or untick Flail of the Ages balance.
- Fix: BG Rebalance v0.8.10, for 0.1.2 (Christopher, 2026-11-04).

**Website.** The update for 0.1.1 is handed to the website chat.

**Not tried live from a released setup yet:**
- the download and modded-game questions;
- a failed optional mod left out;
- a GOG, Beamdog or Microsoft Store copy;
- Marek's fights in the story.

The Updates page does not show the ledger's changes in this app version. Its update check
returns an empty change list for the collection, and the signed recipe channel that would
fill it is not configured. The release notes say so.

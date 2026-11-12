# 0.1.3 package verification — November 12

App **0.1.3**, bundled collection **0.1.2**. Build source: `main` at
`7f266643b9c145c82b1e9fb2e235dd78642c0a92`, the merge of
PR [136](https://github.com/Chrizhermann/chriz-bg-collection/pull/136). Published on
2026-11-12 at 14:00 UTC (2026-11-12 23:00 KST) after Christopher's OK of 2026-11-12; see
[Publication](#publication). Later documentation does not alter the verified package.

## What this release changes

See the [release notes](release-notes-0.1.3.md). Against 0.1.2:

- **Installer & engine:**
  - Flexible game executable and `.bif` archive detection (`BaldurII.exe` in GOG's 2.7.3.0
    BG2:EE build, renamed executables, and older builds such as 2.6.6.0 lacking `PATCH27.bif`),
    plus an independent verified `Baldur.exe` / `icewind.exe` canonical copy inside the managed
    staging folder (#136).
  - Atomic no-overwrite publication (`fs_retry::rename_noclobber`) across all seven file
    publication sites instead of NTFS-only `std::fs::hard_link`, fixing `os error 1` on
    exFAT and FAT32 drives (#136).
  - Main choices (`Challenge Mode`) directly on the install screen (#130).
  - Campaign XP caps (`161,000` / `500,000` / `2,950,000` / `8,000,000`) with banked XP
    included inside `Challenge Mode` (#129).
  - In-app **Feedback** and **Report a problem**, plus Windows user-name and computer-name
    redaction and pre-receipt export in diagnostics bundles (#131, #135).
- **Collection:** bundled `recipes/curated-full-current` remains **0.1.2** (Recommended
  default selection unchanged: 466 components, 69 runs, 55 mods).
- **Version:** app **0.1.3** in `app/package.json`, `app/package-lock.json`,
  `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml`, `Cargo.lock`, and
  `THIRD_PARTY_NOTICES.md` (#136).

## Exact package

- **Setup:** `Chriz Easy BG_0.1.3_x64-setup.exe`, published as
  `Chriz.Easy.BG_0.1.3_x64-setup.exe`. **6,797,347 bytes**.
  - SHA256: `01f16036f7cfe871b033f5e5371b42d42ccb74dfd3a6e8c1b95440f92a2f7e6c`.
- **Signature:** 424 bytes.
  - SHA256: `a81af9ce93701af4a9c113075e2f4ac5d81618b0b0b5c4d508781424fd5a6eff`.
- **Feed** (`latest.json`, 903 bytes; points to the `v0.1.3` release asset, `pub_date`
  `2026-11-12T13:59:20Z`):
  - SHA256: `93fe9202204058aed79a1e4c8fce6ea001d43cae85d7d8f98d78278bee9fd76e`.
  - `SHA256SUMS`: `65bcdc8d55cbf488d563f0d5a7b2e785e80c566b8b65ef07a3e403f46f02600c` (536
    bytes).
- **No test hook:** the shipped app contains none of the `CHRIZ_BG_COLLECTION_TEST_*`
  names. The setup has no Authenticode signature, as before.
- **Public component inventory** (`component-inventory.json`, 72,277 bytes), written by
  `tools/public_component_credits.py` from the release source:
  - SHA256: `73d06f59a57146f57ef95aff56ec74d18539ad5da3a9784fcdf5ea8b658cd354`.
  - **466 default components, 69 runs, 55 mod entries** (unchanged from 0.1.2).
- **License and notices:** the committed file bytes, `LICENSE`
  `b414bdb57a4a78aca070c6e299cdcdbc6ed8f46dbe494f50281cc5f02af369ea` (1,076 bytes) and
  `THIRD_PARTY_NOTICES.md`
  `68c3defb94fcd14443c17fae5749a2023bddb8e155f750fe5e80ca238f0dc5be` (892,307 bytes).

## Automated checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 136 head `4da8511` | runs 43225815201, 43225829763 | runs 43225815314, 43225829884 |
| `main` at `7f26664` (build source) | run 43231498257 | run 43231498243 |

The build ran from a clean detached checkout of `7f266643b9c145c82b1e9fb2e235dd78642c0a92` at
BelowNormal priority. Tauri rewrote `app/src-tauri/Cargo.toml` with LF line endings in this
CRLF checkout; its content equalled the committed file and was restored before signing.

- `built_update_signature_matches_the_bundled_public_key` passed for the exact signed setup.
- `real_tauri_updater_downloads_and_verifies_the_signed_nsis` passed from **0.1.2 to 0.1.3**:
  the updater offered 0.1.3 to a 0.1.2 app, accepted the exact signed bytes, and rejected
  deliberately altered bytes.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS`.
- The setup was signed with the existing CEBG updater key passed by path only; no key was
  created, printed, copied into a repository, or rotated.

## Defender and runtime boundaries

A single-file custom scan (`MpCmdRun.exe -Scan -ScanType 3`) on an unexcluded copy of the
exact setup bytes returned **found no threats**, exit 0, with security intelligence
**1.463.257.0** (engine `1.1.26100.4`). Antivirus, real-time, behavior-monitor and
downloaded-file protection remained enabled; no exclusions or security preferences were
changed.

No setup or app was launched on the host, and no stream installation or saved game was
touched.

## Publication

**Own-mods release check.** `python -I tools/own_mod_pins.py check --release` ran on the
build source on 2026-11-12 at 13:53:05 UTC. Exit 0 (`OK: no installer is out of step.`, with
Item Randomiser v1.0.3 held per the 2026-10-25 decision).

**Release.** [v0.1.3](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.3)
was published on 2026-11-12 at 14:00:23 UTC: a normal release, marked Latest, with seven
files (setup, signature, feed, checksums, component inventory, license, notices). An
anonymous download of all seven matched the local files and `SHA256SUMS`.

**Update channel.** Since 14:00:38 UTC the `alpha` release's `latest.json` on
`Chrizhermann/chriz-easy-bg` is the 0.1.3 feed (`digest: sha256:93fe9202...`). The 0.1.2 feed
(`2f0862bb…`) was saved locally prior to upload as a rollback copy before cleaning `target/`.

**Website.** Published to `bg.chrizfader.org` via
`Chrizhermann/twitch-setup-chriz` PR #84 on 2026-11-12 and verified live (`version: 0.1.3`,
`sha256: 01f16036f7cfe871b033f5e5371b42d42ccb74dfd3a6e8c1b95440f92a2f7e6c`).

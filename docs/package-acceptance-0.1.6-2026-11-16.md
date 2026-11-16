# 0.1.6 package verification — November 16

App **0.1.6**, bundled collection **0.1.3**. Build source: `main` at
`3ff699301982171346c98d0db5c7163200a2e78f`, the merge of
PR [147](https://github.com/Chrizhermann/chriz-bg-collection/pull/147) following
PR [146](https://github.com/Chrizhermann/chriz-bg-collection/pull/146) (`bc5fc11`). Published
on 2026-11-16 at 08:55 UTC (17:55 KST) after PR #146 and PR #147 CI verification and
Christopher's release authorization; see [Publication](#publication). Later documentation does
not alter the verified package.

## What this release changes

See the [release notes](release-notes-0.1.6.md). Against 0.1.5, this ships **Chriz Overhead
Text v0.1.0** (#146, #147):

- **Spoken overhead text above Tab nameplates and HP Over Head (`Chriz Overhead Text v0.1.0`):**
  - Routes `CGameSprite::FloatingText` (Tab / Reveal Details) and `CGameSprite::FloatingHP`
    (`HP Over Head`) to a dedicated per-sprite `CGameObjectArray` slot and stacks spoken
    overhead lines (`DisplayStringHead*`, `VerbalConstant` subtitles) `4 px` cleanly above
    single-line or multi-line nameplates, scaled with area zoom.
  - Enabled by default (`decision = "default"`, `requires = ["feature:eeex:mandatory-components"]`)
    in `post-eet-end` directly before `eeexremote-bg2`, and optional under **Interface** in
    **Customize**.
- **Collection:** bundled `recipes/curated-full-current` goes from **0.1.2** to **0.1.3**
  (`467` default components, `70` runs, `56` mods in Chris's Recommended).
- **Version:** app **0.1.6** in `app/package.json`, `app/package-lock.json`,
  `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml`, `Cargo.lock`, and
  `THIRD_PARTY_NOTICES.md` (#147).

## Exact package

- **Setup:** `Chriz Easy BG_0.1.6_x64-setup.exe`, published as
  `Chriz.Easy.BG_0.1.6_x64-setup.exe`. **7,222,877 bytes**.
  - SHA256: `f1298d908586cb98ccef57e1b12d62e75061b488b2cdf6137aad8ceec273ec8a`.
- **Signature:** 424 bytes.
  - SHA256: `6af438bc6b854b26bbc6202dcd6fa48ca04deb799d9c9c319af666151a8fabb8`.
- **Feed** (`latest.json`, 939 bytes; points to the `v0.1.6` release asset, `recipe_version`
  `0.1.3`, `pub_date` `2026-11-16T08:54:00Z`):
  - SHA256: `fd78fd02c3d26a530fa52325dad8b96dcf9c5754572406a77774ae29e1933bcd`.
  - `SHA256SUMS`: `686fa0617e5ae74c462e93ae973cc60465d9ebb879565307bae0e97904f41e32` (536
    bytes).
- **No test hook:** `cargo tree` showed none of `test-documents`, `test-hooks`, or
  `test-application-data`, and the shipped `chriz-bg-app.exe` and setup contain none of the
  `CHRIZ_BG_COLLECTION_TEST_*` strings (`NO_TEST_HOOK_STRINGS_OK_BG`, ASCII and UTF-16LE). The
  setup has no Authenticode signature, as before.
- **Public component inventory** (`component-inventory.json`, 72,731 bytes), written by
  `tools/public_component_credits.py` from the release source:
  - SHA256: `d260e8513ffcb189c7c380abdefacaaa9445691f7fa856bfb8de4eeeb3c83638`.
  - **467 default components, 70 runs, 56 mod entries**.
- **License and notices:** the committed file bytes, `LICENSE`
  `554232c3c694355ec4d60fd6fd9d4a3cc30b9f03e280bfc5979d96b6d656db19` (1,097 bytes) and
  `THIRD_PARTY_NOTICES.md`
  `8dab71dfc6068a921195a8cffa28f835bb09228975ba7aef31d16ed31ee8026e` (892,416 bytes).

## Automated checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 146 head `ef8941b` | runs 43772844722, 43772826542 | runs 43772844739, 43772826538 |
| PR 147 head `d526c79` | runs 43829924284, 43829913895 | runs 43829924450, 43829913897 |
| `main` at `3ff6993` (build source) | run 43833927817 | run 43833927821 |

The build ran from a clean detached checkout of `3ff699301982171346c98d0db5c7163200a2e78f`.
Tauri rewrote `app/src-tauri/Cargo.toml` with LF line endings in this CRLF checkout; its content
equalled the committed file and was restored before signing.

- `built_update_signature_matches_the_bundled_public_key` passed for the exact signed setup.
- `real_tauri_updater_downloads_and_verifies_the_signed_nsis` passed from **0.1.5 to 0.1.6**:
  the updater offered 0.1.6 to a 0.1.5 app, accepted the exact signed bytes, and rejected
  deliberately altered bytes.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS`.
- The setup was signed with the existing CEBG updater key passed by path only; no key was
  created, copied into a repository, or rotated.

## Defender and runtime boundaries

A single-file custom scan (`MpCmdRun.exe -Scan -ScanType 3`) on an unexcluded copy of the
exact setup bytes returned **found no threats**, exit 0, with security intelligence
**1.463.319.0** (`AMProductVersion: 4.18.26100.4`, `AMEngineVersion: 1.1.26100.4`).
Real-time and behavior-monitor protection remained enabled; no exclusions or security
preferences were changed.

No setup or app was launched on the host, and no stream installation or saved game was
touched.

## Publication

**Own-mods release check.** `python -I tools/own_mod_pins.py check --release` ran on the
build source on 2026-11-16 at 08:46:10 UTC. Exit 0 (`OK: no installer is out of step.`, with
Chriz Overhead Text v0.1.0 on its newest release and Item Randomiser v1.0.3 held per the
2026-10-25 decision).

**Release.** [v0.1.6](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.6)
was published on 2026-11-16 at 08:55:16 UTC: a normal release, marked Latest, with seven
files (setup, signature, feed, checksums, component inventory, license, notices). An
anonymous download of all seven matched the local files and `SHA256SUMS`.

**Update channel.** Since 08:56 UTC the `alpha` release's `latest.json` on
`Chrizhermann/chriz-easy-bg` is the 0.1.6 feed (`digest: sha256:fd78fd02...`), verified via
GitHub API and uncached anonymous `curl.exe -fsSL`. The 0.1.5 feed (`b5aef212…`) was saved
prior to upload as a rollback copy before cleaning `target/`.

**Website.** Published to `bg.chrizfader.org` via
`Chrizhermann/twitch-setup-chriz` PR [101](https://github.com/Chrizhermann/twitch-setup-chriz/pull/101)
(`3a30d2dd530861db1dce9896e37004f03fdea4dc`).

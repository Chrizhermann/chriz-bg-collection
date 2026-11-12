# 0.1.4 package verification — November 13

App **0.1.4**, bundled collection **0.1.2**. Build source: `main` at
`032d623d4404a8bb5aa342b667b274020a7f7e71`, the merge of
PR [138](https://github.com/Chrizhermann/chriz-bg-collection/pull/138). Published on
2026-11-12 at 21:18 UTC (2026-11-13 06:18 KST) after Christopher's approval of the live UI
preview and release sign-off; see [Publication](#publication). Later documentation does not
alter the verified package.

## What this release changes

See the [release notes](release-notes-0.1.4.md). Against 0.1.3:

- **UI, UX & typography redesign (#138):**
  - Candlekeep granite stone-and-brass interface (`bg.css`, `stone-grain.svg`,
    `bg-medallion.svg`) replacing generic web-card styling.
  - Bundled multi-weight OFL fonts (`Alegreya` 400/400i/600/700 and `Alegreya Sans`
    400/500/700) eliminating synthetic font-weight smoothing and offline fallback drift;
    attributions in `app/src/themes/fonts/README.md` and `THIRD_PARTY_NOTICES.md`.
  - Spellbook/Journal chapter index sidebar and sticky toolbar in **Customize** (`setup.ts`).
  - Dedicated **Play** launch header card on **My installs** (`home.ts`), live activity line
    during **Build** (`build.ts`), full-width **Updates** layout (`updates.ts`), and compact
    two-column **Feedback / Report a problem** dialog (`report-dialog.ts`).
  - Multi-size `.ico` app icon (`16–256 px`) and 24-bit uncompressed NSIS sidebar/header
    bitmaps (`app/src-tauri/windows/bg-sidebar.bmp`, `bg-header.bmp`).
- **Collection:** bundled `recipes/curated-full-current` remains **0.1.2** (Recommended
  default selection unchanged: 466 components, 69 runs, 55 mods).
- **Version:** app **0.1.4** in `app/package.json`, `app/package-lock.json`,
  `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml`, `Cargo.lock`, and
  `THIRD_PARTY_NOTICES.md` (#138).

## Exact package

- **Setup:** `Chriz Easy BG_0.1.4_x64-setup.exe`, published as
  `Chriz.Easy.BG_0.1.4_x64-setup.exe`. **7,195,109 bytes**.
  - SHA256: `f5d51d43437fa7ea641204e1619e9a74c23cf44ec5e305d2aeeaca653cb04d74`.
- **Signature:** 424 bytes.
  - SHA256: `ad0c4c8fd718704c62c221f95c7dfee925de1b17777704f61663cfa38f94d526`.
- **Feed** (`latest.json`, 1,002 bytes; points to the `v0.1.4` release asset, `pub_date`
  `2026-11-12T21:17:31Z`):
  - SHA256: `7ed3f066752b863a5f6389e0165a8e9636ac7c3c10ffcd0c6af3f2bc59087627`.
  - `SHA256SUMS`: `55a2cfe7efca6022ba26cf3aea81ef198f7d686969d5b33e277c72f5b22d71b1` (536
    bytes).
- **No test hook:** the shipped app and setup contain none of the `CHRIZ_BG_COLLECTION_TEST_*`
  names. The setup has no Authenticode signature, as before.
- **Public component inventory** (`component-inventory.json`, 72,277 bytes), written by
  `tools/public_component_credits.py` from the release source:
  - SHA256: `83c4214afc2eff4e092a519c1ec5ab7948657affd92e1b1192dcb66822a623b2`.
  - **466 default components, 69 runs, 55 mod entries** (unchanged from 0.1.2).
- **License and notices:** the committed file bytes, `LICENSE`
  `b414bdb57a4a78aca070c6e299cdcdbc6ed8f46dbe494f50281cc5f02af369ea` (1,076 bytes) and
  `THIRD_PARTY_NOTICES.md`
  `dc663c021e5a0c83fcd2a7b90e9ad5958d8e78e28a9e3b1e8b7d3917c081da84` (892,416 bytes).

## Automated checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 138 head `8267ae3` | runs 43301760383, 43301749583 | runs 43301760738, 43301749576 |
| `main` at `032d623` (build source) | run 43307410482 | run 43307410540 |

The build ran from a clean detached checkout of `032d623d4404a8bb5aa342b667b274020a7f7e71` at
BelowNormal priority. Tauri rewrote `app/src-tauri/Cargo.toml` with LF line endings in this
CRLF checkout; its content equalled the committed file and was restored before signing.

- `built_update_signature_matches_the_bundled_public_key` passed for the exact signed setup.
- `real_tauri_updater_downloads_and_verifies_the_signed_nsis` passed from **0.1.3 to 0.1.4**:
  the updater offered 0.1.4 to a 0.1.3 app, accepted the exact signed bytes, and rejected
  deliberately altered bytes.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS`.
- The setup was signed with the existing CEBG updater key passed by path only; no key was
  created, copied into a repository, or rotated.

## Defender and runtime boundaries

A single-file custom scan (`MpCmdRun.exe -Scan -ScanType 3`) on an unexcluded copy of the
exact setup bytes returned **found no threats**, exit 0, with security intelligence
**1.463.269.0** (engine `1.1.26100.4`). Antivirus, real-time, behavior-monitor and
downloaded-file protection remained enabled; no exclusions or security preferences were
changed.

No setup or app was launched on the host, and no stream installation or saved game was
touched.

## Publication

**Own-mods release check.** `python -I tools/own_mod_pins.py check --release` ran on the
build source on 2026-11-12 at 21:09:27 UTC. Exit 0 (`OK: no installer is out of step.`, with
Item Randomiser v1.0.3 held per the 2026-10-25 decision).

**Release.** [v0.1.4](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.4)
was published on 2026-11-12 at 21:18:53 UTC: a normal release, marked Latest, with seven
files (setup, signature, feed, checksums, component inventory, license, notices). An
anonymous download of all seven matched the local files and `SHA256SUMS`.

**Update channel.** Since 21:19:05 UTC the `alpha` release's `latest.json` on
`Chrizhermann/chriz-easy-bg` is the 0.1.4 feed (`digest: sha256:7ed3f066...`). The 0.1.3 feed
(`93fe9202…`) was saved prior to upload as a rollback copy before cleaning `target/`.

**Website.** Published to `bg.chrizfader.org` via
`Chrizhermann/twitch-setup-chriz` PR [86](https://github.com/Chrizhermann/twitch-setup-chriz/pull/86)
on 2026-11-12 UTC (2026-11-13 KST) and verified live (`version: 0.1.4`,
`sha256: f5d51d43437fa7ea641204e1619e9a74c23cf44ec5e305d2aeeaca653cb04d74`).

# Chriz Easy IWD 0.1.9 package acceptance

Status: released on 2026-11-08 at 12:50 UTC, within the coverage documented below.
Christopher approved merging and releasing the overwrite round, the pins of our new mod
releases in both installers and a new Chriz Easy IWD version on 2026-11-08.

## Source and scope

- Release source: `main` at `3efbadac8fdb0e59e984e2a9b72b67d44e78137f` (merge of
  PR [124](https://github.com/Chrizhermann/chriz-bg-collection/pull/124), which changes
  only the Chriz Easy BG recipes).
- It is the first IWD release from `main`. PR
  [120](https://github.com/Chrizhermann/chriz-bg-collection/pull/120) (`4062a28`) brought
  the overwrite rule, PR [121](https://github.com/Chrizhermann/chriz-bg-collection/pull/121)
  (`2d5472a`) brought Chriz Easy IWD onto `main`, and PR
  [123](https://github.com/Chrizhermann/chriz-bg-collection/pull/123) (`a08c70b`) pinned the
  new IWD mod releases and numbered the app and recipe 0.1.9.
- App and recipe: 0.1.9. The previous public app and recipe are 0.1.8.
- Recipe changes: BuffBot 1.12.6 to 1.16.1 and Chriz IWD Encounters 0.2.1 to 0.2.2.
  `chris-recommended` still has 29 mods and 139 components. The release ledger is
  `recipes/iwd-current/releases/v0.1.9/ledger.toml`.
- Hotpatches: a new base `iwd-0.1.9` for the bundled catalog; no new hotpatch.

## Automated source checks

All required checks passed on the three PRs that changed IWD inputs or the shared code:

| PR | Windows x64 checks and unsigned package | Same newest release of our mods in every installer |
| --- | --- | --- |
| 121 | runs 42403866392, 42403871631 | runs 42403866380, 42403871645 |
| 123 | runs 42470221064, 42470226529 | runs 42470221113, 42470226439 |
| 124 | runs 42478687366, 42478692963 | runs 42478687359, 42478692804 |

The Windows job runs the Rust workspace, the IWD app flavor (including the check that the
bundled IWD hotpatch catalog fits the current release), the frontend, the Python tools,
formatting, Clippy, the locked third-party notices and unsigned BG and IWD packaging.

The first `main` run after PR 123 (run 42475885249) failed once in the unsigned packaging
step with `failed to bundle project: http status: 504`, a GitHub download error while the
bundler fetched its packaging tools; the failed job was rerun.

Locally, before PR 124 was merged, the Python tools passed (417 tests, 1 existing skip) and
the own-mod release gate reported that no installer is out of step.

## Full installation

A complete installation of the recommended setup finished on a fresh Steam IWD:EE 2.7.3.0
lab on 2026-11-08 (06:31 to 06:53 KST): 30 runs, 139 components, exit code 0. All 96
installer steps succeeded, including the final stack verification and the receipt.

- The installer was built from the PR 121 head `5eb50aa67daee922b4d0c26df7b38680d831d8ed`.
  The engine and app sources (`engine/src`, `app/src`, `app/src-tauri/src`) are identical
  between that commit and the release source. The recipes differ only by the two pins, the
  version numbers, the release ledger and the hotpatch base.
- BuffBot and Chriz IWD Encounters were local candidate archives built from BuffBot
  `e863d79` and Chriz IWD Encounters `02ff06b`. The published BuffBot 1.16.1 and Chriz IWD
  Encounters 0.2.2 differ from those commits only in version strings, changelogs, tests and
  one indentation-only Lua line. Their new archive needed a higher entry limit (175 entries;
  the limit is 256), which the released pin carries.

## Start test

The game was started through the quiet lab runner (muted, windowed, pointer guard) on
2026-11-08 at 07:27 KST:

- The EEex remote console came up and the START screen was reached.
- The save was continued; the world loaded with the party in `AR1006`.
- BuffBot loaded its 15 modules, its action bar and its HUD button.
- The game log had no new error lines. Its 13 error-like lines, about the missing
  `dialogf.tlk` and six music files, also appear in the baseline log of an earlier lab start.
- The game exited cleanly; all lab processes ended with exit code 0. Result: pass.

## Package

- `Chriz Easy IWD_0.1.9_x64-setup.exe` was built with the IWD flavor from the release
  source on 2026-11-08: 6,730,009 bytes, SHA-256
  `782255300b54289351d5150dd29f3fa07e6c90e30ca22dc276f2e00b44ad483b`.
- The package configuration test and the real updater test
  (`real_tauri_updater_downloads_and_verifies_the_signed_nsis`) passed against the signed
  setup.
- Microsoft Defender scanned the published setup and found no threats.
- Release [v0.1.9](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.9) has
  six assets. Each one was downloaded anonymously and matched `SHA256SUMS`.
- The `updates` channel serves `latest.json` for 0.1.9 (uploaded 12:51 UTC), so 0.1.7 and
  0.1.8 offer the update in the app.
- After publication, one sentence of the release text was corrected: it described the
  0.1.1 behaviour for a failed mod installer. 0.1.9 asks whether to leave out what did not
  install and continue, try again where that is safe, or stop.
- The distribution README describes 0.1.9 (`7dd35fd` in `Chrizhermann/chriz-easy-iwd`).

## Not covered

- The changed fights and BuffBot 1.16.1 were not played in IWD; the start test loads one
  save.
- Updating an installed 0.1.8 app through the in-app updater was not run end to end. The
  updater test downloads and verifies the signed setup.
- The existing hotpatches are unchanged; their apply and Undo were not run again.
- The website is maintained separately and is not part of this acceptance.

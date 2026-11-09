# Chriz Easy IWD 0.1.10 package acceptance

Status: released on 2026-11-09 at 20:28 UTC (2026-11-10 05:28 KST), within the coverage
documented below. Christopher asked for the change on 2026-11-09 and approved the release
the same day, after a full lab installation.

## Source and scope

- Release source: `main` at `8c51eec437f183829a1975c50a696d5fc36f8ce9` (merge of
  PR [127](https://github.com/Chrizhermann/chriz-bg-collection/pull/127), which prepares
  Chriz Easy BG 0.1.2 and changes no IWD input). The IWD change is
  PR [126](https://github.com/Chrizhermann/chriz-bg-collection/pull/126), merged as `0c688e5`;
  `recipes/iwd-current` and `hotpatches/iwd` are identical at both commits.
- App and recipe: 0.1.10. The previous public app and recipe are 0.1.9.
- Recipe change: Klatu Tweaks and Fixes 1.7.4 with its component `2150`, Allow Thievery in
  Armor, in its own run `klatu-armor-thieving-main` after the content mods and directly
  before BuffBot. It is on in `chris-recommended` and can be switched off under Quality of
  life in Customize. `chris-recommended` now has 30 mods and 140 components. The release
  ledger is `recipes/iwd-current/releases/v0.1.10/ledger.toml`.
- Hotpatches: a new base `iwd-0.1.10` for the bundled catalog; no new hotpatch.

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 126 head `eb1e6df` | runs 42634906093, 42634901223 | runs 42634906257, 42634901289 |
| `main` at `0c688e5` | run 42640948934 | run 42640949102 |
| `main` at `8c51eec` (release source) | run 42647837365 | run 42647837384 |

The Windows job runs the Rust workspace, the IWD app flavor (including the check that the
bundled IWD hotpatch catalog fits the current release), the frontend, the Python tools,
formatting, Clippy, the locked third-party notices and unsigned BG and IWD packaging. The
full suites were not run again on the build machine.

`python -I tools/own_mod_pins.py check --release` ran on the release source on 2026-11-09
at 20:13 UTC and reported that no installer is out of step: BuffBot v1.16.1 and Chriz IWD
Encounters v0.2.2 are the newest releases, and EEex Remote Console is at v0.2.1.

## Full installation

A complete installation of the recommended setup finished on a fresh Steam IWD:EE 2.7.3.0
lab on 2026-11-10 (05:03 to 05:27 KST): 31 runs, 140 components, exit code 0. All 99
installer steps succeeded, including the final stack verification and the receipt.

- The installer was built from the release source. The recipe was the copy of
  `recipes/iwd-current` from `0c688e5`, compared file by file with the release source.
- Each archive was checked against its pinned length and SHA-256 when it was acquired, so
  Klatu Tweaks and Fixes 1.7.4, BuffBot 1.16.1 and Chriz IWD Encounters 0.2.2 are the
  published archives.
- The install log's warning lines (81) are the same as in the 0.1.9 lab; none is new.
- A first start of this lab at 04:49 KST was cut off by a restart of the agent's host app
  while the game was being copied. That partial folder was set aside unused, and the
  installation above ran from an empty target.

**The armor change in the game files.** In the installed game's `override`, none of the 59
armor items (item type 2) carries a "disable button" effect (opcode 144) for stealth or
thieving. In the 0.1.9 lab, 13 of the 37 armor items in `override` did.

## Not started

The lab game was not started for this release. The 0.1.9 lab was started and loaded a save
on 2026-11-08; against it, this release adds one WeiDU component that edits armor items. The
same component was installed into a game in progress on 2026-11-09, and that game has been
played since.

## Package

- `Chriz Easy IWD_0.1.10_x64-setup.exe` was built with the IWD flavor from the release
  source on 2026-11-10: 6,728,972 bytes, SHA-256
  `02b4a33b51771382f22cd8846a9956493f4d6aa94facf16ebdb98c48e6bad961`. Its signature is 428
  bytes, SHA-256 `08dd6fab01c4694fec0b0753eb177ab83dbb5be3cb2500de07e23eb5f5c64fa0`.
- The setup was signed with the existing Chriz Easy IWD updater key, passed to the signer by
  path only. No key was created, copied or rotated.
- The signature test against the bundled public key and the real updater test
  (`real_tauri_updater_downloads_and_verifies_the_signed_nsis`, from 0.1.9 to 0.1.10) passed
  against the signed setup; the updater test also rejects the setup with one byte changed.
- The shipped app contains none of the `CHRIZ_BG_COLLECTION_TEST_*` names.
- Microsoft Defender scanned the exact setup bytes and found no threats (security
  intelligence 1.463.214.0, engine 1.1.26100.4; real-time, behavior and downloaded-file
  protection on, no exclusions changed).
- Release [v0.1.10](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.10) has
  six assets. Each one was downloaded anonymously and matched `SHA256SUMS`
  (`770f5b32528dad3dce706c8d709b8f64ca6c895d95a9101436d9d4fe6f9dc85b`); the feed's signature
  is the published `.sig`.
- The `updates` channel serves `latest.json` for 0.1.10 since 20:29 UTC
  (`f27bcfbe33d41dcb785c97b359a8b7ab6908b405774b3fef3f2bf12af14a335e`), so 0.1.7 to 0.1.9
  offer the update in the app. The 0.1.9 feed is kept as the rollback copy.
- The distribution README describes 0.1.10 (`eafff80` in `Chrizhermann/chriz-easy-iwd`).

## Not covered

- Thieving and stealth in armor were not played in a new 0.1.10 installation.
- Updating an installed 0.1.9 app through the in-app updater was not run end to end. The
  updater test downloads and verifies the signed setup.
- The existing hotpatches are unchanged; their apply and Undo were not run again.
- The website is maintained separately; its chat was told about 0.1.10 on 2026-11-10.

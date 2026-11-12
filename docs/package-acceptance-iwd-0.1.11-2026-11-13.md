# Chriz Easy IWD 0.1.11 package acceptance

Status: released on 2026-11-12 at 21:19 UTC (2026-11-13 06:19 KST), within the coverage
documented below. Christopher reviewed and approved the live UI redesign and authorized
end-to-end release of both installers on 2026-11-12 UTC.

## Source and scope

- Release source: `main` at `032d623d4404a8bb5aa342b667b274020a7f7e71` (merge of
  PR [138](https://github.com/Chrizhermann/chriz-bg-collection/pull/138)).
- App: **0.1.11** (previous public app: 0.1.10). Bundled recipe (`recipes/iwd-current`) and
  hotpatch catalog (`hotpatches/iwd`) remain **0.1.10** (`iwd-0.1.10`, 30 mods and 140
  components in `chris-recommended`; unchanged since 0.1.10).
- Changes reaching Chriz Easy IWD players against 0.1.10:
  - **Kuldahar dark stone and riveted timber UI/UX redesign (#138):** dark slate surfaces,
    riveted oak button planks (`iwd.css`, `timber-plank.svg`), glowing glacial Heartstone
    diamond motif (`iwd-diamond.svg`), bundled multi-weight OFL fonts (`Spectral` 400/600/700
    and `Alegreya Sans` 400/500/700), **Customize** chapter index sidebar and sticky toolbar,
    dedicated **Play** launch header card on **My installs**, live **Build** activity line,
    full-width **Updates** layout with flavor-aware copy, compact zero-scroll **Feedback /
    Report a problem** dialog, multi-size `.ico` app icon, and 24-bit uncompressed NSIS
    bitmaps (`iwd-sidebar.bmp`, `iwd-header.bmp`).
  - **Post-0.1.10 engine & app improvements already merged on `main`:** in-app **Feedback**
    in the header and **Report a problem** with Windows user-name and PC-name redaction in
    diagnostics (#131, #135), Main choices support on the install screen (#130), flexible
    executable and `.bif` probing plus canonical `icewind.exe` staging copy (#136), and
    atomic no-overwrite file publication (`fs_retry::rename_noclobber`) supporting exFAT and
    FAT32 installation drives (#136).

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 138 head `8267ae3` | runs 43301760383, 43301749583 | runs 43301760738, 43301749576 |
| `main` at `032d623` (release source) | run 43307410482 | run 43307410540 |

The Windows job runs the Rust workspace, the IWD app flavor (including the check that the
bundled IWD hotpatch catalog fits the current release), the frontend (including Vitest IWD
mode), the Python tools, formatting, Clippy, the locked third-party notices, and unsigned BG
and IWD NSIS packaging plus update-install replacement checks.

`python -I tools/own_mod_pins.py check --release` ran on the release source on 2026-11-12 at
21:09:27 UTC and reported `OK: no installer is out of step.` (BuffBot v1.16.1, Chriz IWD
Encounters v0.2.2, and EEex Remote Console v0.2.1 are on their newest releases).

## Package

- `Chriz Easy IWD_0.1.11_x64-setup.exe` (published as
  `Chriz.Easy.IWD_0.1.11_x64-setup.exe`) was built with the IWD flavor from the clean
  detached release source at BelowNormal priority: **7,115,234 bytes**, SHA-256
  `a06283bbd018855d4fa597c670b72ad073c97b67e1dde6f34197a662edb02779`.
- Signature (`.sig`): **428 bytes**, SHA-256
  `a91273650c9d1801991607a3a07d6fd1c9a99da05d1c84ed5c9ca737b0ee829b`.
- Feed (`latest.json`, 1,010 bytes, `pub_date` `2026-11-12T21:17:31Z`, `recipe_version`
  `0.1.10`): SHA-256 `1edba84b4b37792ef68159af79efd53aad5e5d67c9effd65d90bab46d1484704`.
- `SHA256SUMS` (449 bytes): SHA-256
  `f7252f92399f347c3ae56ceacf2bdb0f5d3fd05c3dae9c5abf599c2a7be9ff67`.
- `LICENSE` (1,076 bytes): `b414bdb57a4a78aca070c6e299cdcdbc6ed8f46dbe494f50281cc5f02af369ea`;
  `THIRD_PARTY_NOTICES.md` (892,416 bytes):
  `dc663c021e5a0c83fcd2a7b90e9ad5958d8e78e28a9e3b1e8b7d3917c081da84`.
- The setup was signed with the existing Chriz Easy IWD updater key, passed to the signer by
  path only. No key was created, copied into a repository, or rotated.
- `built_update_signature_matches_the_bundled_public_key` and
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` (from **0.1.10 to 0.1.11**)
  passed against the signed setup under the IWD `TAURI_CONFIG`; altered bytes were rejected.
- The shipped `chriz-easy-iwd.exe` and setup contain none of the `CHRIZ_BG_COLLECTION_TEST_*`
  names.
- Microsoft Defender scanned the exact setup bytes (`MpCmdRun.exe -Scan -ScanType 3`) and
  returned **found no threats**, exit 0 (security intelligence **1.463.269.0**, engine
  `1.1.26100.4`; real-time, behavior and downloaded-file protection on, no exclusions
  changed). No setup or app was launched on the host.
- Release [v0.1.11](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.11) was
  published at 21:19:10 UTC with six assets. Each asset was downloaded anonymously and
  matched `SHA256SUMS`; the feed's signature matched the published `.sig`.
- The `updates` channel serves `latest.json` for 0.1.11 since 21:19:20 UTC
  (`1edba84b4b37792ef68159af79efd53aad5e5d67c9effd65d90bab46d1484704`), verified
  anonymously, so 0.1.7 to 0.1.10 offer the update in the app. The 0.1.10 feed (`f27bcfbe…`)
  was saved prior to upload as a rollback copy.
- The distribution README in `Chrizhermann/chriz-easy-iwd` was updated at `fdd94aa`.
- Published to `bg.chrizfader.org/iwd` and `/` via `Chrizhermann/twitch-setup-chriz`
  PR [86](https://github.com/Chrizhermann/twitch-setup-chriz/pull/86) and verified live.

## Not covered

- Because `recipes/iwd-current` and `hotpatches/iwd` are byte-for-byte unchanged from 0.1.10,
  no new disposable IWD lab installation was run for 0.1.11 (the 0.1.10 lab completed all 31
  runs / 140 components on 2026-11-10).
- Updating an installed 0.1.10 copy through the live desktop window was not run on the host;
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` verified the real Tauri updater
  download and cryptographic verification path from 0.1.10 to 0.1.11.

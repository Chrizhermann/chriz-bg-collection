# Chriz Easy IWD 0.1.16 package acceptance

Chriz Easy IWD 0.1.16 was released on 2026-11-21 at 20:42:02 UTC
(2026-11-22 at 05:42:02 KST) after the release source passed both CI workflows.

## Source and scope

- The release was built from `main` at `6b93be33599a49cf4db166c65d8bce3527c5c481`,
  the merge of PR #159 (head `61abc5317f0edcda24037076223ddc463642ba1c`, pin commit
  `21ba55dbe0091c97df77886140b7f1765a0ce37a`).
  [PR #159](https://github.com/Chrizhermann/chriz-bg-collection/pull/159)
- App **0.1.16** bundles `recipes/iwd-current` and the `hotpatches/iwd` catalog at
  **0.1.17** (`iwd-0.1.17`, content digest
  `6aceb183900fb7b3640dbedc8f4dd84e931c7cfe1549aa0c31cf115726d397c1`).
  Chris's Recommended includes **33 mods, 145 default components, 34 install runs,
  and 18 recruitable companions**. [Release notes](release-notes-iwd-0.1.16.md)
- Changes since app 0.1.15
  - **Chriz IWD Encounters v0.3.1** unconditionally restores Belhifet's original Icewind
    Dale / Siege of Dragonspear `BELHIFET` sprite (`0xE256`, footprint, and combat/death
    voices) in component **300** instead of Stratagems' Pit Fiend replacement while keeping
    every **v0.3.0** Baldur's Gate spell rule (`120`), Chapter 2–6 overhaul (`420`),
    Belhifet/Yxunomei/Poquelin boss update (`300`/`310`/`360`), and optional HoW/TotLM
    expansion overhaul (`500`) intact.

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR #159 head `61abc53` | [44944044767](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44944044767) (success) · [44944038567](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44944038567) (success) | [44944044744](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44944044744) (success) · [44944038557](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44944038557) (success) |
| `main` at `6b93be3` (release source) | [44947239910](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44947239910) (success) | [44947239895](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44947239895) (success) |

`tools/own_mod_pins.py check --release` finished on the release source at
`2026-11-21T20:37:20.1815896+00:00` with exit 0 and `OK: no installer is out of step.`
The check confirmed BuffBot v1.16.2, BG Rebalance v0.8.12, Chriz IWD Encounters
v0.3.1, Chriz Overhead Text v0.1.0, and Chriz IWD Party Dialog v0.1.0 were on their
newest releases when the package was built.

## Package

- `Chriz Easy IWD_0.1.16_x64-setup.exe` (published as
  `Chriz.Easy.IWD_0.1.16_x64-setup.exe`) was built with the IWD flavor from the clean
  detached release source. The setup is **7,147,481 bytes** with SHA-256
  `e2833ad6625d635e3347f3789d7d7f06545bc5839d99ce76582bcd63c384b3cc`.
- `Chriz.Easy.IWD_0.1.16_x64-setup.exe.sig` is **428 bytes** with SHA-256
  `fbacff8dfa3e03650734c1000e7084e9c3eda781d1c9f0bd42870d17285f19d3`.
- `latest.json` is **1,021 bytes** with SHA-256
  `b4b450b214713daf8e62fd6137c6ef613b53645d9c3fdd5793c2d1f4f6e73d96`.
  It points to the versioned setup and records `recipe_version` as `0.1.17` and
  `pub_date` as `2026-11-21T20:41:56Z`.
- `SHA256SUMS` is **449 bytes** with SHA-256
  `5d9a17a6049fc3d5c8dde74a25d1b27aa1fa65ad3f947087017f675da9496e02`.
- `LICENSE` is **1,097 bytes** with SHA-256
  `554232c3c694355ec4d60fd6fd9d4a3cc30b9f03e280bfc5979d96b6d656db19`.
- `THIRD_PARTY_NOTICES.md` is **909,907 bytes** with SHA-256
  `4ea5ae48b829f83074df9574fa7a7c7078ca6d0e2eb885b129cdf2891d062ce8`.
- The setup was signed with the existing IWD Minisign updater key. The temporary key
  copy was removed after signing (`TEMP_IWD_KEY_REMOVED=True`).
- `package_config` passed `built_update_signature_matches_the_bundled_public_key`
  against the exact signed setup. `updater_download_acceptance` passed
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` from **0.1.15 to
  0.1.16** using the IWD configuration. The updater accepted the signed bytes and
  rejected altered bytes.
- The release build scan found no `CHRIZ_BG_COLLECTION_TEST_*` strings in
  `chriz-easy-iwd.exe` or the setup (`NO_TEST_HOOK_STRINGS_OK_IWD`, ASCII and UTF-16LE).
- Microsoft Defender scanned an unexcluded copy of the exact setup bytes outside
  `CEBG-Tests` and reported **found no threats**, exit 0 (`SCAN_DIR_REMOVED=True`).
  The Defender product was `4.18.26100.4`, engine `1.1.26100.4`, and security
  intelligence `1.463.404.0`. Real-time protection and Behavior Monitor were enabled.

## Publication

- Release **v0.1.16** was published at `2026-11-21T20:42:02Z`. All six anonymous
  asset downloads matched the staged files by byte count and SHA-256, and the feed's
  `url` pointed to the published `v0.1.16` setup (`ANON_V0116_ALL_6_MATCH_OK`).
  [GitHub release](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.16)
- The `updates` channel served the verified 0.1.16 feed at `2026-11-21T20:42:48Z`.
  An anonymous download matched the versioned `latest.json` SHA-256. The previous
  **0.1.15** feed was recorded before upload for rollback (1,024 bytes, SHA-256
  `93c506d6be037d8c23b6b82a5b05113c4c08d5e813da2dada6abe842922737ec`).
  [Update feed](https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/updates/latest.json)
- The distribution repository's `README.md` was updated at
  `b6665c6a4ea7ee22e3cf94a23dc4852aa9b65035`.
  [README commit](https://github.com/Chrizhermann/chriz-easy-iwd/commit/b6665c6a4ea7ee22e3cf94a23dc4852aa9b65035)
- Website PR #108 published the new download information and was merged at
  `ee81b3d3cb44731e4a93f7040f22bdad73a9fdd4` (`BG site check`
  [44947776701](https://github.com/Chrizhermann/twitch-setup-chriz/actions/runs/44947776701),
  [44947782044](https://github.com/Chrizhermann/twitch-setup-chriz/actions/runs/44947782044),
  and master deploy [44948171148](https://github.com/Chrizhermann/twitch-setup-chriz/actions/runs/44948171148)
  succeeded). The live production JSON at
  `https://bg.chrizfader.org/content/site-content.json` showed app `0.1.16` and the
  published setup hash.
  [Website PR #108](https://github.com/Chrizhermann/twitch-setup-chriz/pull/108) ·
  [Chriz Easy IWD](https://bg.chrizfader.org/iwd)

## Not covered

- The full combined 34-run app-driven WeiDU installation and a full IWD campaign
  playthrough remain pending for app 0.1.16.
- In `chriz-iwd-encounters v0.3.1`, all offline test suites (`test_templates`,
  `test_banter_guard`, and twice-installed `test_overwrite` scenarios `base`,
  `edited`, `embedded`, `stack`) passed after the final edit, and Christopher's local
  stream installation was updated to `v0.3.1` and verified with `check_install.py`
  (`Belhifet uses his BELHIFET sprite (0xE256)`, all checks passed).
- Host setup and desktop UI launches were outside package acceptance. The automated
  Tauri updater test covered the download and cryptographic verification path.

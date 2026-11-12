# Chriz Easy IWD 0.1.12 package acceptance

Status: released on 2026-11-12 at 23:32 UTC (2026-11-13 08:32 KST), within the coverage
documented below, following PR #141 CI verification and Christopher's release authorization.

## Source and scope

- Release source: `main` at `c53ab223485ced86c18573706d22df4152516df8` (merge of
  PR [141](https://github.com/Chrizhermann/chriz-bg-collection/pull/141)).
- App: **0.1.12** (previous public app: 0.1.11). Bundled recipe (`recipes/iwd-current`) and
  hotpatch catalog (`hotpatches/iwd`) remain **0.1.10** (`iwd-0.1.10`, 30 mods and 140
  components in `chris-recommended`; unchanged since 0.1.10).
- Changes reaching Chriz Easy IWD players against 0.1.11 (**17 flexibility & robustness fixes,
  #141**):
  - Flexible canonical/junction/8.3 path handling across source probing, install targets, and
    diagnostics matching.
  - Cross-drive/cross-volume moves (`ERROR_NOT_SAME_DEVICE`) and read-only attribute clearing
    in staging and cleanup.
  - Benign OS metadata files (`desktop.ini`, `Thumbs.db`, `.DS_Store`, `__MACOSX`) ignored in
    directory checks.
  - Expanded archive picker support (`.7z`, `.rar`, `.tar.gz`, `.tgz`), extra-file tolerance in
    hotpatch verification, smoother focus/cursor behavior in path inputs, and clearer antivirus
    guidance.

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 141 head `e2dbd1c` | runs 43323469626, 43323461170 | runs 43323469407, 43323461141 |
| `main` at `c53ab22` (release source) | run 43328750260 | run 43328750214 |

`python -I tools/own_mod_pins.py check --release` ran on the release source on 2026-11-12 at
23:25:35 UTC and reported `OK: no installer is out of step.` (BuffBot v1.16.1, Chriz IWD
Encounters v0.2.2, and EEex Remote Console v0.2.1 are on their newest releases).

## Package

- `Chriz Easy IWD_0.1.12_x64-setup.exe` (published as
  `Chriz.Easy.IWD_0.1.12_x64-setup.exe`) was built with the IWD flavor from the clean
  detached release source: **7,139,676 bytes**, SHA-256
  `5cebcdd7ed2baf95ab69de9b7628ad93a1532da2ccdde33af37f8a2f74703a32`.
- Signature (`.sig`): **428 bytes**, SHA-256
  `0ed797338b783e7a031ded310b6ae989809869e61fc2948030f875ce9f4c7934`.
- Feed (`latest.json`, 955 bytes, `pub_date` `2026-11-12T23:32:25Z`, `recipe_version`
  `0.1.10`): SHA-256 `fd830089173f96286d368c408d810656f684483ba08c4149026d2141e3d7d5b6`.
- `SHA256SUMS` (449 bytes): SHA-256
  `ed1aa2ab29e410fe9aef5dff8386f7ecdffa819c44b145c725f7060c39e5108f`.
- `LICENSE` (1,097 bytes): `554232c3c694355ec4d60fd6fd9d4a3cc30b9f03e280bfc5979d96b6d656db19`;
  `THIRD_PARTY_NOTICES.md` (892,416 bytes):
  `bfafce89a030722f00fa3ed3c819d8eeeb69953f9586a92eb2735c62230212a4`.
- The setup was signed with the existing Chriz Easy IWD updater key, passed to the signer by
  path only and removed immediately afterwards. No key was created, copied into a repository,
  or rotated.
- `built_update_signature_matches_the_bundled_public_key` and
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` (from **0.1.11 to 0.1.12**)
  passed against the signed setup under the IWD `TAURI_CONFIG`; altered bytes were rejected.
- The shipped `chriz-easy-iwd.exe` and setup contain none of the `CHRIZ_BG_COLLECTION_TEST_*`
  names (`NO_TEST_HOOK_STRINGS_OK_IWD`, ASCII and UTF-16LE).
- Microsoft Defender scanned an unexcluded copy of the exact setup bytes (`MpCmdRun.exe -Scan
  -ScanType 3`) and returned **found no threats**, exit 0 (`AMProductVersion: 4.18.26100.4`,
  `AMEngineVersion: 1.1.26100.4`, security intelligence **1.463.269.0**; real-time and
  behavior-monitor protection on, no exclusions changed). No setup or app was launched on the
  host.
- Release [v0.1.12](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.12) was
  published with six assets. Each asset was downloaded anonymously via `curl.exe -fsSL` and
  matched `SHA256SUMS`; the feed's signature matched the published `.sig`.
- The `updates` channel serves `latest.json` for 0.1.12
  (`fd830089173f96286d368c408d810656f684483ba08c4149026d2141e3d7d5b6`), verified anonymously,
  so 0.1.7 to 0.1.11 offer the update in the app. The 0.1.11 feed (`1edba84b…`) was saved
  prior to upload as a rollback copy.
- Published to `bg.chrizfader.org/iwd` and `/` via `Chrizhermann/twitch-setup-chriz`
  PR [87](https://github.com/Chrizhermann/twitch-setup-chriz/pull/87)
  (`474960fe544afc9c28ef64ee691f3caad8d941c9`) and verified live.

## Not covered

- Because `recipes/iwd-current` and `hotpatches/iwd` are byte-for-byte unchanged from 0.1.10,
  no new disposable IWD lab installation was run for 0.1.12 (the 0.1.10 lab completed all 31
  runs / 140 components on 2026-11-10).
- Updating an installed 0.1.11 copy through the live desktop window was not run on the host;
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` verified the real Tauri updater
  download and cryptographic verification path from 0.1.11 to 0.1.12.

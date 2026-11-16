# Chriz Easy IWD 0.1.13 package acceptance

Status: released on 2026-11-16 at 08:55 UTC (17:55 KST), within the coverage documented below,
following PR #146 and PR #147 CI verification and Christopher's release authorization.

## Source and scope

- Release source: `main` at `3ff699301982171346c98d0db5c7163200a2e78f` (merge of
  PR [147](https://github.com/Chrizhermann/chriz-bg-collection/pull/147)).
- App: **0.1.13** (previous public app: 0.1.12). Bundled recipe (`recipes/iwd-current`) and
  hotpatch catalog (`hotpatches/iwd`) advance from **0.1.10** to **0.1.13** (`iwd-0.1.13`,
  content digest `3540064fb278f30e240eedec4d0a2e43aebc6f16e3ba0fa5ee11fd4630c521b8`, **31 mods
  and 141 components** with **18 companions** in `chris-recommended`; existing `0.1.10`–`0.1.12`
  installations retain base `iwd-0.1.10`).
- Changes reaching Chriz Easy IWD players against 0.1.12 (**Chriz Overhead Text v0.1.0, #146,
  #147**):
  - Keeps spoken overhead lines (`DisplayStringHead*`, `VerbalConstant` subtitles) visible while
    **Tab / Reveal Details** or **`HP Over Head`** is active, stacking spoken lines `4 px`
    cleanly above nameplates or HP labels, scaled with area zoom.
  - Enabled by default (`decision = "default"`, requires EEex) right after `buffbot-main` and
    before `eeexremote-main`; switchable under **Interface** in **Customize**.

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 146 head `ef8941b` | runs 43772844722, 43772826542 | runs 43772844739, 43772826538 |
| PR 147 head `d526c79` | runs 43829924284, 43829913895 | runs 43829924450, 43829913897 |
| `main` at `3ff6993` (release source) | run 43833927817 | run 43833927821 |

`python -I tools/own_mod_pins.py check --release` ran on the release source on 2026-11-16 at
08:46:10 UTC and reported `OK: no installer is out of step.` (BuffBot v1.16.1, Chriz IWD
Encounters v0.2.2, Chriz Overhead Text v0.1.0, and EEex Remote Console v0.2.1 are on their
newest releases).

## Package

- `Chriz Easy IWD_0.1.13_x64-setup.exe` (published as
  `Chriz.Easy.IWD_0.1.13_x64-setup.exe`) was built with the IWD flavor from the clean
  detached release source: **7,139,482 bytes**, SHA-256
  `b761456bae5e5ccef7cec28b78be1f1207c7e7dde9ea53fef1af0d137273a3da`.
- Signature (`.sig`): **428 bytes**, SHA-256
  `04ca2783db73c51ad69a7a3667d34f27e10ee9265c1c6de56b1b8523f7aa3273`.
- Feed (`latest.json`, 932 bytes, `pub_date` `2026-11-16T08:54:00Z`, `recipe_version`
  `0.1.13`): SHA-256 `b04387d3d8a5e2bc341e4a05124fc98b58a84b76265d158d262acab88afd9e07`.
- `SHA256SUMS` (449 bytes): SHA-256
  `98f073be5d9f2ed2cbcd56a0c197136048bbdbf61015f4c89e0a76ddfab090c7`.
- `LICENSE` (1,097 bytes): `554232c3c694355ec4d60fd6fd9d4a3cc30b9f03e280bfc5979d96b6d656db19`;
  `THIRD_PARTY_NOTICES.md` (892,416 bytes):
  `8dab71dfc6068a921195a8cffa28f835bb09228975ba7aef31d16ed31ee8026e`.
- The setup was signed with the existing Chriz Easy IWD updater key retrieved temporarily from
  Christopher's vault, passed to the signer by path only and removed immediately afterwards
  (`TEMP_IWD_KEY_REMOVED=True`). No key was created, copied into a repository, or rotated.
- `built_update_signature_matches_the_bundled_public_key` and
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` (from **0.1.12 to 0.1.13**)
  passed against the signed setup under the IWD `TAURI_CONFIG`; altered bytes were rejected.
- The shipped `chriz-easy-iwd.exe` and setup contain none of the `CHRIZ_BG_COLLECTION_TEST_*`
  names (`NO_TEST_HOOK_STRINGS_OK_IWD`, ASCII and UTF-16LE).
- Microsoft Defender scanned an unexcluded copy of the exact setup bytes (`MpCmdRun.exe -Scan
  -ScanType 3`) and returned **found no threats**, exit 0 (`AMProductVersion: 4.18.26100.4`,
  `AMEngineVersion: 1.1.26100.4`, security intelligence **1.463.319.0**; real-time and
  behavior-monitor protection on, no exclusions changed). No setup or app was launched on the
  host.
- Release [v0.1.13](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.13) was
  published at 08:55:25 UTC with six assets. Each asset was downloaded anonymously via
  `curl.exe -fsSL` and matched `SHA256SUMS`; the feed's signature matched the published `.sig`.
- The `updates` channel serves `latest.json` for 0.1.13
  (`b04387d3d8a5e2bc341e4a05124fc98b58a84b76265d158d262acab88afd9e07`), verified via GitHub API
  and uncached anonymous download, so 0.1.7 to 0.1.12 offer the update in the app. The 0.1.12
  feed (`fd830089…`) was saved prior to upload as a rollback copy.
- `Chrizhermann/chriz-easy-iwd` `README.md` was updated at commit `f3b0663` (31 mods with 141
  components and Chriz Overhead Text v0.1.0 credited).
- Published to `bg.chrizfader.org/iwd` and `/` via `Chrizhermann/twitch-setup-chriz`
  PR [101](https://github.com/Chrizhermann/twitch-setup-chriz/pull/101)
  (`3a30d2dd530861db1dce9896e37004f03fdea4dc`).

## Not covered

- A combined 32-run full disposable IWD WeiDU installation was not re-run specifically for
  0.1.13; `Chriz Overhead Text v0.1.0` installs a single runtime Lua hook file
  (`override/M_CZOH.lua`), passed all 173 offline binary/hook/WeiDU checks against the `2.7.3.0`
  `Icewind.exe` executable and `EEex v1.3.1`, and was verified live in a `2.7.3.0` quiet lab.
- Updating an installed 0.1.12 copy through the live desktop window was not run on the host;
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` verified the real Tauri updater
  download and cryptographic verification path from 0.1.12 to 0.1.13.

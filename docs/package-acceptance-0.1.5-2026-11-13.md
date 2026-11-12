# 0.1.5 package verification — November 13

App **0.1.5**, bundled collection **0.1.2**. Build source: `main` at
`c53ab223485ced86c18573706d22df4152516df8`, the merge of
PR [141](https://github.com/Chrizhermann/chriz-bg-collection/pull/141). Published on
2026-11-12 at 23:31 UTC (2026-11-13 08:31 KST) after PR #141 verification and Christopher's
release authorization; see [Publication](#publication). Later documentation does not alter the
verified package.

## What this release changes

See the [release notes](release-notes-0.1.5.md). Against 0.1.4, this ships the **17 flexibility
and robustness fixes** across engine, bridge, and Tauri app (#141):

- **Path, junction, and volume flexibility (#141):**
  - Canonical `\\?\` / junction / 8.3 short-name path normalization (`canonicalize_flexible`)
    used uniformly across target validation, safe path checks, and diagnostic matching.
  - Cross-volume / cross-drive moves (`ERROR_NOT_SAME_DEVICE` / exFAT) and read-only file
    attributes handled resiliently during staging, archive extraction, and cleanup.
  - Benign OS metadata files (`desktop.ini`, `Thumbs.db`, `.DS_Store`, `__MACOSX`) ignored in
    emptiness and hotpatch checks.
- **Archives, hotpatches, UI input, and antivirus guidance (#141):**
  - Archive picker accepts `.7z`, `.rar`, `.tar.gz`, and `.tgz` alongside `.zip` / `.exe`.
  - Extra non-colliding user files tolerated in managed hotpatch verification.
  - Path input fields preserve cursor/focus cleanly while editing, and preflight/failure UI
    provides clearer antivirus exclusion guidance without turning protections off.
- **Collection:** bundled `recipes/curated-full-current` remains **0.1.2** (Recommended
  default selection unchanged: 466 components, 69 runs, 55 mods).
- **Version:** app **0.1.5** in `app/package.json`, `app/package-lock.json`,
  `app/src-tauri/tauri.conf.json`, `app/src-tauri/Cargo.toml`, `Cargo.lock`, and
  `THIRD_PARTY_NOTICES.md` (#141).

## Exact package

- **Setup:** `Chriz Easy BG_0.1.5_x64-setup.exe`, published as
  `Chriz.Easy.BG_0.1.5_x64-setup.exe`. **7,214,336 bytes**.
  - SHA256: `a4619b532db5d6d2ffe8ae82cb8c7206efec0ae09080dc39bc0161fdf11056ca`.
- **Signature:** 424 bytes.
  - SHA256: `b5302f7aa4c2c57dc838935d28b648026a3753353f9528c4633c47834b7d7495`.
- **Feed** (`latest.json`, 943 bytes; points to the `v0.1.5` release asset, `pub_date`
  `2026-11-12T23:29:14Z`):
  - SHA256: `b5aef212b22d5fd44a72a90a21c21d217f441e48f50daaf63886c813d6967c39`.
  - `SHA256SUMS`: `c2c18a1a94a3c2e25be3c7c61a7c72101612ec5f2e648ae60d628db671f44afd` (536
    bytes).
- **No test hook:** `cargo tree` showed none of `test-documents`, `test-hooks`, or
  `test-application-data`, and the shipped `chriz-bg-app.exe` and setup contain none of the
  `CHRIZ_BG_COLLECTION_TEST_*` strings (`NO_TEST_HOOK_STRINGS_OK`, ASCII and UTF-16LE). The setup
  has no Authenticode signature, as before.
- **Public component inventory** (`component-inventory.json`, 72,277 bytes), written by
  `tools/public_component_credits.py` from the release source:
  - SHA256: `bb31a2cbb0d4253e87a1dff684a5f868134f8ac0a2e862efc05d89ca2428eaae`.
  - **466 default components, 69 runs, 55 mod entries** (unchanged from 0.1.2).
- **License and notices:** the committed file bytes, `LICENSE`
  `554232c3c694355ec4d60fd6fd9d4a3cc30b9f03e280bfc5979d96b6d656db19` (1,097 bytes) and
  `THIRD_PARTY_NOTICES.md`
  `bfafce89a030722f00fa3ed3c819d8eeeb69953f9586a92eb2735c62230212a4` (892,416 bytes).

## Automated checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 141 head `e2dbd1c` | runs 43323469626, 43323461170 | runs 43323469407, 43323461141 |
| `main` at `c53ab22` (build source) | run 43328750260 | run 43328750214 |

The build ran from a clean detached checkout of `c53ab223485ced86c18573706d22df4152516df8`.
Tauri rewrote `app/src-tauri/Cargo.toml` with LF line endings in this CRLF checkout; its content
equalled the committed file and was restored before signing.

- `built_update_signature_matches_the_bundled_public_key` passed for the exact signed setup.
- `real_tauri_updater_downloads_and_verifies_the_signed_nsis` passed from **0.1.4 to 0.1.5**:
  the updater offered 0.1.5 to a 0.1.4 app, accepted the exact signed bytes, and rejected
  deliberately altered bytes.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS`.
- The setup was signed with the existing CEBG updater key passed by path only; no key was
  created, copied into a repository, or rotated.

## Defender and runtime boundaries

A single-file custom scan (`MpCmdRun.exe -Scan -ScanType 3`) on an unexcluded copy of the
exact setup bytes returned **found no threats**, exit 0, with security intelligence
**1.463.269.0** (`AMProductVersion: 4.18.26100.4`, `AMEngineVersion: 1.1.26100.4`).
Real-time and behavior-monitor protection remained enabled; no exclusions or security
preferences were changed.

No setup or app was launched on the host, and no stream installation or saved game was
touched.

## Publication

**Own-mods release check.** `python -I tools/own_mod_pins.py check --release` ran on the
build source on 2026-11-12 at 23:25:35 UTC. Exit 0 (`OK: no installer is out of step.`, with
Item Randomiser v1.0.3 held per the 2026-10-25 decision).

**Release.** [v0.1.5](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.5)
was published on 2026-11-12 at 23:31 UTC: a normal release, marked Latest, with seven
files (setup, signature, feed, checksums, component inventory, license, notices). An
anonymous download of all seven matched the local files and `SHA256SUMS`.

**Update channel.** Since 23:31:24 UTC the `alpha` release's `latest.json` on
`Chrizhermann/chriz-easy-bg` is the 0.1.5 feed (`digest: sha256:b5aef212...`), verified
anonymously via `curl.exe -sL`. The 0.1.4 feed (`7ed3f066…`) was saved prior to upload as a
rollback copy before cleaning `target/`.

**Website.** Published to `bg.chrizfader.org` via
`Chrizhermann/twitch-setup-chriz` PR [87](https://github.com/Chrizhermann/twitch-setup-chriz/pull/87)
(`474960fe544afc9c28ef64ee691f3caad8d941c9`) and verified live (`version: 0.1.5`,
`sha256: a4619b532db5d6d2ffe8ae82cb8c7206efec0ae09080dc39bc0161fdf11056ca`).

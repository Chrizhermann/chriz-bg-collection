# CEBG overnight implementation / acceptance

User priority: a simple attractive installer, a working full-mod installation, Radar,
versions/updates, and cheap consistency checks. Prefer real installation evidence over
another round of cosmetic refactoring. Preserve all older curation/owning-repository work.

## Workspace and scope

- Active worktree: `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- Branch: `codex/installer-v0-real-alpha`; do not work in stale `4d39`.
- The existing stream installation and all `C:\Games` references remain untouched.
- Both clean Steam 2.7.3 sources now pass detection. The older BG1-dirty handover is stale.

## Implemented

- Compact install, launcher, Customize, progress/failure and Updates views. Back navigation,
  searchable choices, smaller typography, sensible desktop fit, responsive narrow layouts.
- Fixed dotted artifact identities in campaign freezing **and** durable artifact evidence;
  actual `dlcmerger-2.1` now passes both paths.
- Human recipe release metadata is frozen into receipts; old local-hash receipts still
  resume. The native app records its own version separately from the engine version.
- Two packaged profiles: downloadable recommended alpha and full creator setup. Full uses
  the complete reference log plus explicitly recorded maintained-source substitutions.
  See `creator-full-install.md` and the generated `recipes/creator-full-current/PARITY.md`.
- Cheap receipt-vs-WeiDU.log checks on the launcher show counts/matches/changed/unavailable.
  Comments are ignored; ordered TP2/language/component identities are compared. This is
  **not** an override-file audit, and changes do not unnecessarily disable Play.
- BG Radar Overlay Latest check/download/native 7z install, separate owned-file receipt,
  Updates action, and automatic attempt after successful UI installation. Optional failure
  leaves the game playable. Play starts an intact installed overlay alongside the game.
- Signed Tauri updater, embedded public key, real check/install commands, startup check and
  update indicator. Collection releases currently ship with the app and apply to a **new
  installation**, never silently to current saves. No live-game hot patching is implemented.

## Verification already performed

- 63 frontend tests, typecheck and production web build passed.
- Native bridge/unit/package tests passed; profile selection includes path-injection rejection.
- Final native rerun: 39 tests passed; the environment-dependent signature test is run
  separately against the actual signed setup (and passed for the first alpha.2 build).
- Dotted production artifact IDs pass frozen-session and synthetic install-to-receipt tests.
- Real WeiDU log-redirection fix: the attempt-owned synchronized stdout supplies terminal
  markers only when the requested debug file has none; partial/contradictory debug remains
  authoritative. Focused regression, 26 CLI tests and all 4 real pinned-WeiDU integration
  tests passed. Exact mod-log suffix, exit status and prompt checks remain required.
- A human recipe version passes the full synthetic installation/receipt assertion.
- Headless viewport checks covered six screen sizes; desktop main screens fit without
  unnecessary scrollbars and no horizontal overflow was found.
- Radar 2.1.0.0 upstream identity and exact native extraction/publication passed. No claim
  of in-game Radar acceptance yet.
- NSIS alpha.2 builds with a Tauri signature. A separate Minisign test verifies the actual
  setup against the public key embedded in the app. Final rebuild includes the corrected
  full recipe and WeiDU evidence-source fix; its actual signature test passed.
- All 22 Python tooling tests passed after final generator normalization.

## First overnight follow-up: EET path compatibility

At 2026-09-04 21:22 UTC the full run had completed all acquisition/materialization,
all 28 BG1 components, and the two BG2 EE Fixpack components. EET then rolled back
with `End_of_file`; the verified failed receipt says the exact EET run is retryable.
There is no successful final installation receipt yet.

Root cause is the Windows path passed to EET, not missing SoD data: CEBG passed the
canonical `\\?\C:\Users\chris\Games\CEBG-Full-20260905\bg1` as a typed staged-root
argument. EET `lib/bgee_dir.tph` strips `?` and replaces backslashes, corrupting the
prefix before its movie-file check. The expected `movies/sodcin05.wbm` exists. The focused
Rust invocation fix now renders ordinary drive/UNC syntax only at the mod-script argument
boundary, retaining canonical path validation internally. The unit regression and a real
pinned-WeiDU synthetic EET-style movie-sentinel regression both passed.

The same managed installation resumed safely with the corrected optimized driver at
`target/full-install-20260905-r3/chriz-bg-install.exe`. The **current log** is now
`target/full-install-20260905-r3/resume.log`. Real EET accepted the ordinary BG1 path,
validated all 28 BG1 components, and reached TLK merging. No new game copy was needed.

The desktop build **0.1.0-alpha.3** completed successfully, while the unchanged collection
recipe stays **0.1.0-alpha.2**. Its actual setup signature passed verification against the
embedded public key. Keep the earlier alpha.2 package for an actual signed update-cycle
test; do not confuse a signature check with a tested download/apply/restart cycle.
No frozen recipe, ledger, game source, or mod source was edited for this repair.

## Real installation runs / cleanup

- Recommended alpha: `C:\Users\chris\Games\CEBG-alpha-test-20260905`,
  install id `install-3e091429c2b2d8889c15`.
  Logs: `target/real-install-20260905/resume-release.log` (current), earlier `resume.log`.
  Uses an isolated copied optimized driver, not `target/debug`, so builds cannot lock it.
  It successfully resumed an interrupted BG1 staging copy, then completed BG2 staging.
  All materialization completed and real DLC Merger exited 0 with the expected log row.
  CEBG then stopped with `fresh_copy_required`: setup-name WeiDU redirected its detailed
  debug output to `SETUP-DLCMERGER.DEBUG`, leaving the requested per-attempt debug file
  with only a header. The real integration tests used captured stdout but the CLI used
  that header-only debug file. The focused regression/fix now passes; do not
  edit this failed ledger or pretend this installation completed. The corrected full
  run will cover the same BG1 preparation path, so another parallel alpha copy is not
  needed. The regression tests have finished using this copy. Its cleanup command was
  also rejected by the execution policy before running, so the folder and exact registry
  record remain. Do not retry the same deletion through a different route; no diagnostic
  copy in `target/real-install-20260905/failed-evidence` was created by that rejected call.
- Retired first full attempt: `C:\CEBG-Full-20260905`, id
  `install-daea3ea2007ad86185b2`. Stopped during source staging before any WeiDU child ran,
  because old modpack component 600 does not exist in the current package. Do not resume
  its obsolete frozen recipe. Its generated folder and matching campaign registry JSON
  still exist: the cleanup command was rejected by the tool safety policy. Do not claim
  they were deleted or bypass that rejection. Download cache is reusable and should stay.
- Corrected full target: `C:\Users\chris\Games\CEBG-Full-20260905`.
  Install id: `install-8a3cab271f29d2c47f61`; active log:
  `target/full-install-20260905-r3/resume.log`. Copied optimized driver:
  `target/full-install-20260905-r3/chriz-bg-install.exe`.
  Started after the completion-evidence fix and final recipe normalization; freeze,
  preflight, all verified cache acquisition and both source copies succeeded. All
  materialization and BG1 preparation completed; see the EET follow-up above. Final plan: 90 runs,
  488 components, 31 artifacts. The recipe preserves maintained replacements once;
  explicit outcome differences are old modpack 600 and the legacy Safana-to-Abettor /
  Aura-to-Bard assignments. Sirene deliberately uses the approved native True Paladin.
  Cache: `C:\CEBG-creator-full-cache`. Keep a fully successful tested install for the user;
  dispose only of clearly identified failed/superseded test folders when permitted.
- For a successful CLI-driven run, add Radar with the same native implementation:
  `cargo run -p chriz-bg-engine --release --example install_radar -- <managed-root> <cache-root>`.
  The optimized helper is already built at `target/release/examples/install_radar.exe`.
  Do not launch a test game alongside another running Baldur/InfinityLoader process.

## Release boundary

App version is `0.1.0-alpha.3`; the bundled recipe remains `0.1.0-alpha.2`.
Signing private key lives outside the repository at
`%LOCALAPPDATA%\Chriz Easy BG Developer\signing\cebg-updater.key`; never print/commit it.
The proposed public distribution-only repository is `Chrizhermann/chriz-easy-bg`.
It has **not** been created/published, so the configured public update endpoint is not
live yet. The existing collection repository remains private. Public publication needs
explicit release authorization under the existing implementation plan; do not expose its
history or the private extras archive. See `cebg-release.md` for the prepared feed helper.

Current local Windows package:
`target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.3_x64-setup.exe`
(5,093,355 bytes; SHA-256
`B878EBA376C6C1C598700B8BD072B1181F33B8D721A5258D95A0F34F93C7D1A3`).
The signed NSIS build and actual package/public-key verification passed. Matching `.sig`
is beside it; local update feed and checksums are in `target/cebg-release/0.1.0-alpha.3/`.
This package includes the EET staged-path fix. Nothing has been published.

Retained previous package for update-cycle acceptance:
`target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.2_x64-setup.exe`
(5,092,074 bytes; SHA-256
`2B31F86F18C96115BE3535682B3FE2A6652047EBE7780BCA30C5C7578BC0DE17`).
Its matching `.sig` is beside it. Prepared update feed and checksums are at
`target/cebg-release/0.1.0-alpha.2/`; these are local only. The final source-only
normalization removes redundant blank lines at TOML EOF and does not change the
packaged recipe's parsed content. Tauri signatures are not Windows Authenticode signing.

## Finish line

1. The corrected full recipe, signed alpha.3 app and full managed run are prepared/launched.
   Continue the current run; do not start another copy just to repeat static checks.
2. Investigate actual install failures without skipping components or altering frozen
   evidence; resume safe failures, make a new copy only when required.
3. Verify successful receipt and exact final WeiDU logs; add Radar and do bounded boot /
   new-game / save-reload acceptance if available. Preserve the successful copy.
4. Record final paths/results here, hand over any owning-repo gap (notably legacy modpack
   600's Spellhold/Bodhi fix), and request public distribution authorization separately.

## Proportionate acceptance coverage

The user explicitly asked for more than an installation check. Use one complete real
installation, short packaged-app journeys, and small targeted failure fixtures; do not
repeat the whole mod stack for every edge case. Keep evidence categories distinct:

- Downloads: exercise an uncached download, verified cache reuse, and a bounded failed /
  interrupted acquisition case. The earlier recommended-profile cold acquisition covered
  30 artifacts; this full run uses the verified cache and is not a new cold-download test.
- Full installation: successful final receipt plus exact ordered WeiDU lists, followed by
  game boot, new-game and save/reload smoke acceptance. These remain pending.
- Packaged app: brief navigation/readiness/progress/failure/reopen-launcher journeys,
  including Play and Open game folder. Headless layout tests alone are not this acceptance.
- Updates: actual old-to-new signed check/download/apply/restart cycle, version display,
  unchanged managed game registration, and useful offline/no-update behavior. Signature
  verification has passed; the real update cycle has not. The public feed is not live.
- Radar and consistency: successful add-on installation/receipt and bounded launch check;
  receipt/WeiDU mismatch must show an understandable changed state without claiming a
  full override audit. Preserve the user's existing game and overlay processes.

Do not call the overall product accepted solely because unit tests or the final install
receipt pass. Record any unavailable interactive acceptance explicitly rather than looping.

## Bounded overnight continuation

App heartbeat `cebg-overnight-install-follow-up` returns to this task every 30 minutes,
for at most twelve checks. Normal progress should produce no repeated status messages or
test/review loops. Pause it after acceptance or when a user decision is needed. Inspect this
file first because the current run/driver can change after an actual failure. The process
is not considered successful until its successful receipt and final mod lists agree.

An existing `BG Radar Overlay.exe` process was observed before these tests; it belongs to
the user's existing setup. Do not terminate it to make a test convenient, or claim that
static extraction/signature checks prove in-game overlay acceptance.

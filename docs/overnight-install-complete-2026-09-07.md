# Overnight installation completion — 2026-09-07

The existing `C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` copy completed at
**07:57:05 KST**, after continuing with the alpha.14 recovery engine at 06:37.
Nothing was deleted or restarted. The successful copy is retained for Christopher.

## Verified result

- Identity: `install-5f63e5d7939d3f509139`; original frozen recipe alpha.12.
- Receipt outcome `succeeded`, 43 planned WeiDU runs and **434 components**.
  Every run was actually attempted; every requested component appears in its
  recorded installed order; no component rows were removed.
- Both current WeiDU logs match the receipt's exact hashes and ordered rows:
  BG1 27 components, BG2/EET 407. Engine `report` also accepts the completed state.
- Ledger 234 confirms final verification; ledger 236 confirms receipt completion.
- The original five successful mod runs still have only attempt 1. EET's one
  actual invocation was attempt 2, following the never-started preflight failure.
- All 32 SoD Remix components succeeded. BuffBot components 1 and 0 succeeded last.
- `game\InfinityLoader.exe` exists and is the recorded launch executable.
  Isolated save identity remains `Chriz Easy BG - 9053a55806fb`.

The receipt correctly keeps application alpha.13 as the original frozen session
version; updating the execution engine did not rewrite historical recipe/identity
fields. App alpha.14 is publicly released and its verified setup is in Downloads,
but the installed desktop app remains alpha.13 because unattended setup execution
was blocked by the tool environment before starting.

This proves complete installation/recorded consistency, not gameplay acceptance.
No game or save was launched or edited. Christopher's stream installation and clean
sources are untouched. Native GUI updater apply/restart and optional game-shortcut
creation were not performed during the background CLI continuation.

## Recorded mod-warning follow-up (not an installation abort)

Three runs returned WeiDU's installed-with-warnings exit 3; all requested rows are
present and the engine's final checks passed. Keep these findings for later mod
compatibility work rather than silently suppressing them or restarting the stack:

- RR component 7: no effects altered on `MISC2P.ITM` (root DEBUG line 1735).
- Bardic Wonders component 2004: attempt-owned stdout lines 4573/4621 report an
  unrecognized finite Abettor song controller, so **Symphony of the Dark Children
  was skipped**. This specific feature needs follow-up in the Bardic Wonders fork;
  do not mistake aggregate successful installation for proof that it was applied.
  Evidence: `.chriz/attempts/attempt-7da66eb7040f78cca7c1/steps/0091-c73ec1a85870a6db/attempt-0001/stdout.log`.
  Root `SETUP-BARDICWONDERS.DEBUG` was overwritten by a later invocation and does
  not contain this earlier warning; always use attempt-owned evidence.
- SCS components 2000/2510: no effects altered on `SPIN150.spl`/`SPPR116.spl`.
  SFO records inventory-shaping notices and one failed insertion of `dw#gphlp`
  above `gpmerc` in `OHRSGUA1` (root DEBUG line 218020). Its gameplay impact is
  unproven; keep it as a targeted later compatibility check, not a reason to
  declare all installation work failed or silently patch the completed copy.

## Launcher follow-through

The background continuation bypassed the frontend's automatic post-completion Radar
step. The existing `install_radar` operator helper invokes the same production
completed-state check, official Latest-release lookup and verified add-on installer
as the launcher. **BG Radar Overlay 2.5.0.0** (official Latest at lookup) was downloaded,
verified and installed successfully at `game/BG Radar Overlay/BG Radar Overlay.exe`.
Its executable hash matches `.chriz/addons/bg-radar-overlay.json`; both mod logs
still match their final receipt hashes. No Radar/game executable was launched.
A new Play shortcut can be created from the updated app later; the existing CEBG
app shortcut is preserved. The overnight heartbeat is paused after completion.

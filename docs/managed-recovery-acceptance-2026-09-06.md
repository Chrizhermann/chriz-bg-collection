# Managed recovery completion and Radar acceptance

The existing r5 copy is now registered as **0.1.0-alpha.8 (repaired)**. No game
clone, source staging, mod reinstall or selection change occurred in this slice.
The original failed attempt remains immutable history.

## Actual completion

- Root: `C:\Users\chris\Games\CEBG-Curated-20260905-r5`.
- Install: `install-e6325c7c98451ad4901e`; recovery: `modpack-alpha5-20260906`.
- Completion: 2026-09-06 02:35:58 KST (`1788629758053` epoch milliseconds).
- Separate `kind: supervised_recovery` receipt: `.chriz/install-receipt.json`.
  SHA-256: `b34f5291d842baf7295914159dc81f8a891160143ea8f14c87f121b55bbb94bb`.
- Original terminal receipt remains `fresh_copy_required`. All 226 protected
  historical metadata files are unchanged; the new receipt binds 273 evidence files.
- Exact stack: **27 BG1 + 403 BG2 = 430 components**. Modpack alpha.5 replaces
  alpha.1; earlier Bardic stays balance.2. This is not the pure alpha.9 recipe.
- Only missing game finalization was `game/engine.lua`: patched/read back to the
  already-reserved `CEBG Curated r5 - 8803ccf86178`. BG1 and ownership marker matched.
- Save folder: `C:\Users\chris\OneDrive\Documents\CEBG Curated r5 - 8803ccf86178`.
- Launch: `game/InfinityLoader.exe`. Completed registry record now exists under
  `LOCALAPPDATA\Chriz BG Collection\managed-installs`; existing same-id logic hides
  the original incomplete campaign card without deleting its history.

`engine/examples/accept_r5_recovery.rs` defaults to read-only proof. It independently
reconciles original failed snapshots, rollback prefix, pinned tools, all five recorded
supervised commands, terminal markers and final plan. `--apply` only finalizes save
isolation and publishes metadata; it never invokes WeiDU. A second apply was
byte-identical/idempotent, with no game identity rewrite.

CLI JSON `report` returns the original failed `receipt` plus separate `recovery`.
Human report identifies current recovery completion while retaining the old outcome.
Launcher consistency and the Radar helper accept normal success or explicit verified
recovery; neither accepts an ordinary failed receipt as complete.

## Radar

Normal add-on flow checked GitHub Latest, downloaded and installed **2.5.0.0** into
`game\BG Radar Overlay` at 02:41:05 KST. A second invocation checked the existing
owned files and Latest identity, returning `downloaded: false` without replacement.

- Archive: 68,222,139 bytes; SHA-256
  `b8622e03526be812fce22690508a59445687a7c4354a9f55bd1d3e4be88f8f9c`.
- Add-on receipt `.chriz/addons/bg-radar-overlay.json` owns 15 files.
- Executable SHA-256:
  `37dc2de78b770399e33e107308b99869d32ec889f18e07832d2372152188787d`.
- BG2 WeiDU.log remains byte-identical to the completed recovery log.
- Debug 7z extraction took several minutes; release-build timing is unmeasured.
  This is acquisition/extraction/publication acceptance, not an in-game overlay test.

## Tests and remaining scope

Passed: 8 recovery receipt TDD tests, 2 supervised command/path tests, 26 existing
CLI tests, 10 receipt tests, 10 registry tests and 3 launcher consistency tests.
Focused Clippy `-D warnings`, formatting and diff checks passed. Actual r5 read-only
proof, publication, idempotent completion and CLI report readback passed.

**No game process was launched in this slice.** Next: distinct native candidate with
current UI; existing-install launcher/Play/open-folder/Updates acceptance; game startup,
new-game/save-reload and in-game Radar smoke; app-update apply/restart. Installed native
alpha.8 predates these source changes. Safe pause/close and a public guarded Repair
action remain unfinished; the supervised adapter is not automatic public recovery.
Public Evandra standalone acquisition remains a separate alpha release blocker.

Do not rerun completed r5 recovery, resume stopped r6 or create a new full install.
R6 deletion was tool-rejected; do not retry another route. Stream game, Steam sources
and `C:\Games` remain untouched.

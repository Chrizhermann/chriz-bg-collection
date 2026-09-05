# Targeted recovery replaces repeated full rebuilds

Christopher explicitly approved targeted recovery after questioning the r6 restart.
This supersedes the earlier blanket read-only rule for the r5 **game files only within
this recovery**. Original recipe, ledger, seal and failed receipt remain immutable.
Stream game, Steam sources and C:\Games remain untouched. No further full rebuild
without discussing necessity and cost with Christopher.

## Current operation

**Game installation and managed recovery registration are COMPLETE.** See
[managed completion/Radar acceptance](managed-recovery-acceptance-2026-09-06.md):
separate recovery receipt, isolated save identity, registry and report readback passed;
Radar 2.5.0.0 is installed. Current native UI and gameplay acceptance remain pending.
**Do not rerun the completed recovery commands below.**

Final audit at 02:08 KST verified every TP2/language/component in exact
plan order: BG1 27 + BG2 403 = **430 components**. All five real WeiDU operations
(uninstall, modpack install, CDTweaks 2312, SR 60, BuffBot 1/0) exited zero.
No game launch/save-reload test occurred in the repair slice. Radar was installed
in the subsequent managed-completion slice linked above.

- R6 PID 47680 was deliberately stopped at 2026-09-06T01:38:53+09:00 during
  `stage:bg2`, ledger 73. BG1 staging had completed; no WeiDU child existed (only
  the Windows console host). No mod installation had started. Do not resume r6.
- Recover existing `C:\Users\chris\Games\CEBG-Curated-20260905-r5` in place.
  No game clone, source restaging, or replay of the first 383 active BG2 components.
- The known 14 modpack entries remain exactly at the top of the stack. The two
  failed components 170/192 already rolled back according to their process output.
- `tools/recover-r5-modpack.ps1` is an explicit supervised acceptance utility,
  **not** a public automatic resume feature. Default `Inspect` makes no writes.
  Modes `Prepare`, `Uninstall`, `Install` require known identities and exact stacks.
- Preparation completed: 882 files, **11,213,471-byte** `before-state.zip` under
  `.chriz/recoveries/modpack-alpha5-20260906/`. Includes scoped affected resources,
  mod source/backups, WeiDU executable/log, TLK and KEY. Intent records original
  metadata hashes, old/new artifact hashes and exact rollback-file expectations.
- Uninstall must use the original verified alpha.1 source and pinned WeiDU 249,
  without `--safe-exit`, targeting only the 14 installed top-tail components.
  Verify return to the 383-row prefix **and** earliest-backup file bytes/absence.
- Install alpha.5's same 16 selected components in authored order. No utility XP
  610 or new 220-223 defaults. Prior Bardic **balance.2 stays balance.2**; balance.3
  remains queued for future fresh recipes, not a reason to replay the earlier stack.
- All three remaining runs are now complete: CDTweaks 2312, Spell Revisions 60,
  BuffBot 1/0, with original frozen arguments/pins. Their per-operation evidence is
  retained alongside `recovered-install.json` and `final-plan.verified.json`.
- Uninstall restored all 367 tracked resource states (exact earliest-backup bytes or
  required absence) and the original 383 active entries. All 16 alpha.5 components
  installed successfully. The original 226 protected metadata files still hash-match.
- Final BG1 log SHA-256: `d89dc668b5adea7a1b22d74c8547107ccfb5df107b711411d23a769cf4ef725b`.
  Final BG2 log SHA-256: `b57792ae5d50cbef8d5f31622c75801847ce317667b0e0d8d5cf767a357c3156`.
- Continuation source checks identified **65 legitimate SR generated-file changes**.
  These were not blindly accepted or reset to archive bytes. Every binary difference
  was verified as SR's exact known `sp...` to `dv...` temporary resref substitution;
  `ds_sr_extra.2da` matched its original header plus the six literal authored appends.
  All other published files for the three remaining mods matched their ownership
  hashes. See `continuation-source-evidence.json`; no install script/config changes
  were allowed. The source proof is `ASSIGN_TEMP_NAMES_FOR_NEW_SPELL_REFERENCES` in
  `spell_rev/lib/manage_add_spell_references.tpa` plus `main_component.tpa` appends.
- Cleanup of stopped r6 was attempted once after exact-target checks were included
  in the command, but the tool rejected the entire command before execution. **No
  deletion or proposed ZIP creation happened.** Do not retry this rejected target by
  another route. `C:\Users\chris\CEBG-Tests\CEBG-Curated-20260906-r6` remains the
  stopped disposable copy; logs are already retained under worktree `target/`.

## Evidence and product limits

Original r5 terminal state must not be unsealed or falsely reported complete. The
supervised operation records create-once intent/process/verification sidecars; a
repaired game is not automatically an accepted managed campaign. The subsequent
managed-completion slice now supplies separate recovery lineage and registration,
without changing the original failed attempt or ledger seal.

The new pure engine planner `weidu::recovery` recognizes a provable ordered subset
at the end of an unchanged install stack, returns reverse-stack uninstall order,
and verifies exact active-prefix restoration. Six new tests plus 11 existing
WeiDU-verifier tests pass. It is not yet wired to an automatic recovery button.

## Next bounded installer work

1. Done: explicit recovery provenance/receipt and registry handling for the existing
   root, referencing failed history and verified operations. Native current UI
   acceptance remains; do not repeat completed metadata publication or mod installs.
2. Surface repair eligibility separately from genuinely unrecoverable state, then
   connect a guarded repair action. The new planner is a building block, not a UI
   completion claim. A changed mod version must be recorded, not hidden in alpha.8.
3. Complete launcher/game/Radar/update acceptance using this recovered copy. Do not
   reinstall it merely to obtain a different entrypoint or newer Bardic pin.

Product direction: retry/repair by default; offer optional-component omission only
with explicit user choice, dependency validation and truthful resulting selection.
Mandatory failures retain progress for repair. Full restart is a last resort when
known-good state cannot be restored, not a synonym for unsupported recovery.

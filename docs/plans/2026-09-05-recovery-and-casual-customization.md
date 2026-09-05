# Queued priorities: recovery, casual customization, optional SoD skip

Christopher's queued message adds these requirements without stopping the current full
installation or changing its frozen recipe. He accepts the short alpha-release scope.
Priority order: finish acceptance + clear dual updates; safe recovery; Customize QoL;
optional SoD skip in its owning repository for an early alpha/weekend release.

## 1. Both CEBG and a game setup have updates

Already implemented/tested in source (`9cecb26`), not yet in signed alpha.8: one visible
New badge and a tooltip such as **New updates available: CEBG app and collection.** Both
rows remain visible. If the setup ships with the newer app, explain **Update CEBG first**;
afterward, an older managed game still shows a separate new-installation action. Updating
CEBG does not silently upgrade existing game files. Radar can also appear in the tooltip.
See [visual acceptance](../updates-ui-acceptance-2026-09-05.md).

## 2. Recovery is the next installer priority

Read-only source audit; no crash/cancel experiment was performed on r4 or a live game.

| Interruption | Existing engine behavior | Player-facing contract |
|---|---|---|
| Internet loss during acquisition | Bounded retries; partial HTTP resume only with valid validators/range response, otherwise retry that archive from zero; completed cache entries are verified | Restore connection and Retry/Continue; do not lose all verified downloads |
| Retryable step failure | Frozen ledger skips completed steps and retries the failed/current step | Retry failed step |
| Crash between completed mod runs | Reopen indexed immutable identity and reconcile saved state | Continue installation after verification |
| Crash/kill during WeiDU | Reconcile invocation, logs/debug, expected prefix and postconditions; accept completed work, retry/continue a proven suffix, or fail closed | Never promise resume before verification; never ask users to rerun WeiDU manually |
| Contradictory evidence/failed postcondition | Permanent `fresh_copy_required` seal; retry forbidden | Start new installation in a new empty folder; preserve diagnostics and verified downloads |
| Moved/corrupt identity or another active worker | Refuse unsafe/concurrent mutation | Restore exact folder where possible, wait for the active worker, or start fresh |

The current **Cancel build** kills the active WeiDU process tree. It is NOT Pause.
Outside WeiDU, cancellation is not cooperatively observed during download/copy work and
can remain pending until the next WeiDU process starts. Closing the app has no close
interception; the worker is an in-process thread, not a durable background service.

### Required small recovery slice (before presenting pausing as supported)

- **Pause safely** requests a stop at the next verified pipeline boundary. During WeiDU,
  finish the whole current mod invocation and commit its verification/ledger first—not
  an arbitrary component, log line, or suspended process. Do not start another mod.
- Show **Pausing after [mod]…** and explicitly say the current mod can take a while.
  Only show **Paused — safe to close CEBG** after no child is active and the checkpoint
  is durable. Reopen must offer Continue with the same frozen choices/versions.
- Download/staging work needs cooperative boundary handling too. Do not defer cancellation
  until a new mod process is spawned merely to kill it immediately.
- Closing an active window must offer Keep running / Pause safely / Stop now. Stop now
  carries the truthful warning that the interrupted copy may need rebuilding. No force
  stop on silence alone; long-running SCS is not automatically a failed install.
- Resume must reconstruct the phase/progress display from frozen state; an empty phase
  array must not announce All phases complete.
- A fresh restart keeps diagnostics, cache and user choices, suggests a distinct empty
  destination and never silently deletes the failed folder or existing playable game.

### Bounded acceptance, separate from the full game build

Use small disposable fake-game/worker fixtures, not another 430-component run or an OS
reboot: interrupted download with and without range support; pause during download/copy;
pause requested during a controlled fake mod; app close at a boundary; process termination
mid-mod with intact versus contradictory evidence; reopen/resume; explicit fresh-restart
choice. Verify exact completed-prefix preservation, no duplicate mod execution, no next
mod started after pause, cache reuse, durable Paused state and truthful recovery labels.
Then one native close/reopen smoke. Existing lower-level tests cover portions of this;
the complete native pause/close lifecycle is not yet accepted.

Source pointers: `engine/src/acquire/cache.rs` (partial resume),
`engine/src/orchestrator.rs:1022` (interrupted attempt reconciliation),
`engine/src/weidu/runner.rs:53` (Cancel/ContinueWaiting, no Pause),
`app/src-tauri/src/bridge.rs:1768` (worker), `app/src-tauri/src/lib.rs` (window lifecycle),
`app/src/screens/build.ts` (Cancel/retry/fresh-copy actions).

## 3. Customize for a casual player (after recovery)

Update 2026-09-06: Christopher explicitly requested these grouped controls next. The
[implemented source slice and bounded acceptance](../customization-acceptance-2026-09-06.md)
supersede the unimplemented status below for common bundles/category actions. Safe
pause/close remains the next installer priority; typed advanced inputs remain unfinished.
Nothing changed in the running r5 recipe or the installed native package.

Read-only source-backed review, not a completed rendered UI audit: current full catalog
projects 493 controls across 25 categories into a flat checkbox view. Preserve advanced
power but do not ask someone changing two preferences to understand that catalog.

Proposed compact structure:

1. **Christopher's setup** remains the one-click default, with no forced customization.
2. **Common changes:** Spell Revisions, Item Randomiser, Artisan's kit overhaul, SoD Remix,
   and original companion classes/kits. Short explanations, compact type, collapsed details.
3. **Gameplay tweaks:** a small named CDTweaks selection, using actual input/choice controls.
4. **Advanced options** collapses the complete catalog with existing search/categories.
5. Show **Your changes: 2** with semantic changes and **Reset to Christopher's setup**;
   reflect those changes on the main install/review screen. Preserve dependencies and the
   existing curation decisions; no automatic unreviewed additions or omissions.

### Mapping/implementation requirements

- Spell Revisions: `mod:spell-rev`; Randomiser: `mod:randomiser`; SoD bundle:
  `mod:chriz-sod-remix` already supply useful parent controls.
- Artisan off must cover `mod:artisanskitpack`, `mod:artisanskitpack-npc`, and
  `mod:artisanskitpack-tweak`, plus truthful dependency effects. One parent is insufficient.
- **Original companion classes/kits** needs an audited semantic bundle across Artisan NPC
  changes, CHRIZ-BG-MODPACK conversions, Xan/Yeslick routes, CDTweaks and other contributors.
  It is a fresh-install selection, not surgery on recruited NPCs. Do not claim that merely
  turning off Artisan restores everyone. Clarify whether mod NPCs keep their own authors'
  original class/kit while BioWare NPCs return to game defaults; preserve other NPC fixes.
- `setup.ts` currently does not render `FeatureControl.inputs`; support authored choice,
  boolean and integer inputs before claiming every CDTweaks choice is editable.
- Replace context-free duplicate labels such as Use BG2 values with their parent tweak's
  meaning. Collapsing unavailable children should not hide an unexplained dependency change.
- Existing tiny synthetic UI fixtures do not prove findability or atomic bundle coverage
  for the real catalog. Use representative real-catalog UI tests and engine selection tests.

No Customize implementation or change to the running recipe occurred in this review.

## 4. Optional skip-SoD: delegated, not a first-alpha blocker

Handed to existing SoD task `019f6539-74ef-7260-93c3-00b63cee296a`, repository
`Chrizhermann/chriz-sod-rebalance`, for bounded prior-art research/design and implementation
where the EET transition is clear. Separate from released v0.6.5, component 290 and frozen r4.
See [the exact handoff](../handoffs/2026-09-05-optional-sod-skip.md).

- At the requested post-Sarevok bedroom moment, ask whether to skip SoD.
- Confirm both Yes and No choices; declining either confirmation loops to the question.
- Confirmed skip: **add 250,000 XP to the main character only, exactly once**, then use
  the supported EET transition to BG2. Not a party reward and not setting total XP.
- Confirmed play: normal SoD, no bonus, no repeated prompt.
- Verify the actual hook/timing, campaign/party/inventory transition and save/reload.
  If a meaningful timing or transition choice is needed, return that question rather
  than inventing a different story flow. No live-game mutation or automatic publication.

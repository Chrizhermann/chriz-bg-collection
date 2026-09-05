# Corrected full collection: installation acceptance

User request, 2026-09-05: complete the real full installation and test flow, including
downloads, updates, launcher/game checks and cleanup. Keep a successful install for review.

## Authority and current state

**Current run is r5, not r4.** R4 stopped after all 30 SoD Remix components installed
successfully: the recipe expected the root TP2 alias, whereas setup-name WeiDU logged
the byte-identical nested TP2. The immutable failure remains; no seal was removed.
See [the bounded diagnosis and correction](handoffs/2026-09-05-sod-tp2-identity.md).

- Root: `C:\Users\chris\Games\CEBG-Curated-20260905-r5`.
- Install ID: `install-e6325c7c98451ad4901e`; attempt `attempt-aba3cbb32e1956c690b8`.
- Frozen recipe `0.1.0-alpha.8`, SHA-256
  `cb220e5de749a779ec0e87337534fd803c11805375011e4b2856984120b4da34`.
- PID `45792`, started `2026-09-05T21:38:57.4010069+09:00`. Reuses the unchanged engine
  executable `target/full-install-20260905-r4/chriz-bg-install.exe`; verify both before use.
- Logs: `target/curated-r5-install.stdout.log` and `.stderr.log`. This human-output invocation
  writes progress to stderr; a nonempty stderr file alone is not an error.
- Exact same 43 runs / 430 components, sources and cache. Selection hash
  `c30cd756e7db86def60cca27425b5af27d358faf3c1eede79a4987e4a9dc15bd` and plan hash
  `ef683e075fa7a620cd68051d2c51d0b274bf497b4975cd66683c94aa9341d499` are unchanged.
- Recipe-only fix `b14c50a`; generated recipe commit `c4d607e`. 28 Python tool tests,
  4 production Chriz recipe tests and 11 WeiDU-verifier tests passed. Generic exact-path
  verification remains strict. Other remaining root-TP2 releases lack the duplicate alias.
- First verified checkpoint: ledger 71 (`stage:bg1`), all acquisitions complete, worker alive.
  This CLI-originated run has no originating desktop-app version. Do not invent one.
- R2/r3 cleanup was also tool-rejected before execution; nothing was deleted. Do not retry
  these paths by another route. Diagnostics are retained. R5's actual preflight passed:
  approximately 43.21 GB required versus 50.54 GB available before it began.

**Latest UI/release planning:** [short alpha path](plans/2026-09-05-alpha-release-short-path.md)
and [update-state visual acceptance](updates-ui-acceptance-2026-09-05.md). The source now
has explicit app/collection/Radar notifications, accessible tooltip, and current-bundle
new-install action (80 frontend tests and web build passed). These are not yet in the
installed signed alpha.8; package a new version before claiming native acceptance of them.
The original standalone Evandra download is available; its public acquisition contract
must replace the private aggregate contract before public release. Do not alter frozen r5.

### Historical r4 identity (terminal, not a resume target)

**R4 stopped at ledger 216 with a TP2 identity mismatch:** SoD v0.6.5 is published and independently
verified. Frozen alpha.7 retains the exact curated component choices; app alpha.8
also fixes the misleading Retry action for structured `fresh_copy_required` receipts
(65 frontend tests/typecheck passed, commit `a7bd8cf`).

- Root: `C:\Users\chris\Games\CEBG-Curated-20260905-r4`.
- Install ID: `install-83f1bf87d5eb83425ed7`.
- Attempt ID: `attempt-06284c52f0b43b3c6f2d`.
- Frozen recipe SHA-256:
  `e9b6b96e17f41a9033d5b9bede6acd06df7caad84b648c4177fc0ad15f50d247`.
- Preset: `chris-recommended`, recipe `0.1.0-alpha.7`, 43 runs / 430 components.
  Authored SoD pin commit `3357f1d`; generated recipe `3ec32ed`. Identity-masked plan
  hash is unchanged, proving no component selection/order change from the approved plan.
- CLI PID `87448`, started `2026-09-05T19:21:45.9712007+09:00`, executable
  `target/full-install-20260905-r4/chriz-bg-install.exe` in the active worktree below.
  Verify executable/start time before relying on this PID. Do not start another resume.
- Logs: `target/curated-r4-install.stdout.log`, `target/curated-r4-install.stderr.log`.
- Cache: `C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection`.
- Sources: clean Steam BG:EE+SoD and BG2:EE 2.7.3; both remain read-only.
- Final checkpoint: ledger 216, `fresh_copy_required` after SoD; worker exited.
  Inspect only the newest ledger records for progress; record zero is a large frozen payload.
- Signed app alpha.8 packaging passed; silent local NSIS upgrade exited zero and the
  installed EXE reports alpha.8 at `target/nsis-smoke-20260905-022935/chriz-bg-app.exe`.
  The app remains closed. This CLI-originated run has no originating desktop-app version;
  do not invent one. Rediscovery/launcher acceptance remains to do.
- Alpha.7-to-alpha.8 real updater check/download/signature/byte-identity/tamper-rejection
  passed. Signed setup is 5,102,476 bytes, SHA-256
  `151d4f7ac8faab61aa3024cef6c13ffe9c348038afacfed103ef9afd3354fa16`.
  Local unpublished feed: `target/cebg-release/0.1.0-alpha.8/`; see
  [updater evidence](update-acceptance-2026-09-05.md). Automatic apply/restart remains untested.
- Existing acceptance follow-up is active again, every 30 minutes for 12 checks. Its
  current prompt names this exact r4 identity, continues practical acceptance/cleanup,
  stays quiet on unchanged progress, and prohibits retries of rejected cleanup targets.

Radar Latest is now 2.5.0.0, published 2026-09-05. `BG2RadarOverlay.7z` is 68,222,139
bytes, SHA-256 `b8622e03526be812fce22690508a59445687a7c4354a9f55bd1d3e4be88f8f9c`.
Real download/layout checks and all seven focused engine Radar tests pass without code
changes. Verified archive is `target/radar-2.5.0.0-acceptance/BG2RadarOverlay.7z`.
Install this latest verified add-on only after the new game's completed receipt exists.

### Historical r3/package snapshot (not the current run)

- Worktree: `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- Frozen recipe: `recipes/curated-full-current`, preset `chris-recommended`,
  recipe version `0.1.0-alpha.6`; signed app alpha.7 is packaged and installed locally.
- Selection comes from curation and later approvals, never a replay of historical logs.
  See [the approved reconciliation plan](plans/2026-09-05-curation-reconciliation.md).
- The replacement installation **is stopped: SoD Remix 120 needs an owning-repo fix**.
  Target: `C:\Users\chris\Games\CEBG-Curated-20260905-r3`.
  Install id: `install-c117933c9f2de8edc020`.
  Recipe SHA-256: `e5ced746536e989e7482834b0f2bb40e18bc1fba595ac13a7e27112c96c2563a`.
  App cache: `C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection`.
  Former CLI PID: `33316` (exited), executable
  `target/full-install-20260905-r4/chriz-bg-install.exe`. This fresh attempt honestly has
  no originating desktop-app version; launcher rediscovery remains part of acceptance.
  CLI logs: `target/curated-r3-install.stdout.log` and `target/curated-r3-install.stderr.log`.
- The NSIS installer retained its earlier registered smoke-test location:
  `target/nsis-smoke-20260905-022935/chriz-bg-app.exe`. That installed app now reports
  alpha.7 and is closed; it is not the replacement run's owner. The old
  `%LOCALAPPDATA%\Chriz Easy BG` alpha.1 executable is also closed. Do not delete
  the NSIS test location while preparing/upgrading the packaged launcher.
- Durable progress is under the new root's `.chriz/ledger/` (latest numbered records),
  with frozen recipe under `.chriz/recipe/`. Do not dump record zero: it is a large payload.
  Use the engine report and bounded latest ledger records. Never start a concurrent resume
  while the CLI worker runs; the old native window is not the replacement's progress view.
- The invalid `creator-full-current` recipe and old failed installations (including r2)
  must not be resumed.

## Practical acceptance

- [x] Full ready recipe resolves; included/default/conditional/deferred reconciliation saved.
- [x] Packaged app defaults to corrected full recipe; historical replay absent from bundle.
- [ ] New isolated install completes using clean, unmodified BG1/BG2 source copies.
- [x] Download/cache/retry coverage includes real artifacts and bounded failure cases.
- [ ] Receipt version and exact installed WeiDU component list agree with the frozen plan.
- [ ] Launcher rediscovery, Play, game-folder action and update screen exercised.
- [x] Signed update check/download/rejection tested; apply/restart coverage stated separately.
- [ ] Latest verified BG Radar Overlay installed into the successful game copy.
- [ ] Bounded game launch/new-game/save-reload smoke, with no other game disturbed.
- [ ] Disposable test cleanup completed where permitted; successful copy retained.

The public update feed is not yet published. A local signed updater test is not evidence
that the public channel is live, and signature verification alone is not an apply/restart test.

## Pre-install evidence

- Actual engine plan: 43 runs / 430 concrete components; BuffBot is last. PublicAlpha
  validation has zero findings. Six selection findings are the recorded unavailable or
  incompatible choices, not silent omissions. The complete authored reconciliation is
  `recipes/curated-full-current/RECONCILIATION.md` (440 default/mandatory catalog rows).
- Authoring code and its regression tests are committed locally as `23fc059`; the recipe
  static evidence has been regenerated to reference that real commit, not an earlier base.
- Frontend: typecheck, 63 tests, production web build passed. Native bridge: 36 command
  contract tests plus two packaging tests passed. Full authoring tools: 27 tests passed.
- Four newly needed upstream source archives were freshly downloaded and verified. Eleven
  focused engine cache/HTTP tests passed. The actual signed alpha.3 installer passed the
  Tauri updater's check/download/verification and byte-tamper rejection. See
  [updater acceptance](update-acceptance-2026-09-05.md) for scope and the alpha.4 rerun.
- Both Steam source games independently pass the verified clean 2.7.3 profiles. The last
  native-window/process check found no running game; the stream copy's Radar stays untouched.
- NSIS alpha.4 installation exited zero. Native UI showed alpha.4, the Full curated setup
  selected by default, independently ready sources, and the exact new target before Install
  was clicked. Main/progress screens fit the default window with no scrollbar.
- Signed alpha.4 NSIS check/download and tamper rejection passed too; its setup SHA-256 is
  `df3110688fad1b6f9aed0c51702867e15200e91ff7d90d807abf2edba74db944`.
  Automatic updater apply/restart remains distinct from this manual NSIS installation.

## Continuation

### Resolved historical blocker: SoD Remix 120 (r3)

The following records the sequence before publication; the current r4 details above
supersede its waiting/pause instructions. Failed r3 remains immutable.

The bounded follow-up found r3 terminal at ledger 216. SCS, Randomiser, EET_END, and late
Bardic 1012/3001 passed. SoD Remix 120 failed because `csrhood.d` addresses BDSCRY state 4,
but the actual restored dialogue has only four states (0-3); dependent 225 was skipped.
The latest published source is still v0.6.4. This needs a source-repository compatibility
fix and a new pin, not another recipe-only tweak or a component omission.

Read the [complete owning-repo handoff](handoffs/2026-09-05-sod-remix-component-120.md).
Failure diagnostics are saved separately (553 files). No fourth full rebuild was started,
no source mod edits or publication were made, and no game-launch acceptance was attempted
on this incomplete copy. Christopher has now explicitly retained the full SoD Remix bundle
and authorized the source fix; the handoff was dispatched and the owning SoD task is active.
He can perform focused live tests there. The automatic installation follow-up remains
paused while the fix/test/release candidate is prepared. Full acceptance remains incomplete;
no component selections were changed and unrelated component 290 remains deferred.

The owning task has now returned local v0.6.5 candidate commit `3b21d6f` and reports
11 passing tests. It attributes the old state-4 assumption to the Aura-expanded dev
dialogue; Christopher explicitly does not require Aura support. Candidate archive hash
was checked locally; details are in the handoff. It is not published or collection-pinned,
and a complete fresh install/live acceptance remain pending. A focused new-code repair
on a disposable r3/game clone now passed: 120/225 installed with WeiDU exit 0, exact old
344-row prefix preserved (346 rows after), and semantic verifier reports zero failures.
See the handoff for evidence and test-copy ownership. No game was launched; the partial
tail clone is not a completed collection installation. The existing stream-copy read-only
verifier remains baseline-only evidence, distinct from this new native check.

The owning task has prepared a separate `CEBG SoD120 v065 Test` save/configuration profile
for that retained clone and a `LIVE-TEST.md` beside the local release candidate. Christopher
can now start a new disposable SoD character there for the focused pool check; no existing
save import is needed. Exact paths and cleanup ownership are in the handoff. No game has
been launched by either task, and publication still awaits his explicit decision.

Christopher has now accepted the focused gameplay test, including save/reload (relayed
with the owning task's bounded screenshot evidence). Details and remaining evidence limits
are in the handoff. Implementation is unchanged; the final release will refresh packaged
documentation. Wait for publication and verify its final artifact/hash before re-pin or
rebuild. Do not remove the retained test copy/profile yet. Full collection acceptance is
still incomplete.

### Historical r3 checkpoint

The conditional-prompt fix is committed as `6a2558b`, and recipe alpha.6 static evidence
references that commit. `cargo test -p chriz-bg-engine` passed all runnable tests (seven
environment-gated ignored), formatting passed, and all 28 authoring-tool tests passed.
The actual full-recipe plan has zero public-alpha validation findings and still contains
43 runs / 430 components, with every run's component sequence identical to r2. The only
run-order change is Randomiser immediately after SCS, per existing curation. The default
plan now emits exactly one `y\n` compatibility response; turning off Xan and RR12 emits
no Randomiser prompt and preserves its exact components. This is planning/automated
evidence, not proof that the new full install or game acceptance has completed.

R3 has already reused all verified acquisitions and reached BG1 staging (ledger 71).
It uses the freshly compiled, separately pinned CLI named above. Do not use the older
`target/full-install-20260905-r3` binary to load recipe alpha.6: it predates the new optional
prompt condition field. The old r2 installation remains terminal and is never a resume target.

App alpha.7 contains the conditional-prompt engine and recipe alpha.6. Its signed NSIS
package is 5,100,157 bytes, SHA-256
`4edeb0d2eb2a201edde82766acf64610616830f6a62cb9bf77d2ea0da28553ae`.
The actual updater check/download/byte-identity and tamper rejection passed from alpha.6
to alpha.7. Separate silent NSIS installation exited zero and the installed EXE reports
alpha.7; it remains closed while the CLI install runs. The local unpublished feed is
`target/cebg-release/0.1.0-alpha.7/`. Automatic apply/restart is still not proven by this.
The freshly rebuilt native app's 36 command-contract tests and two packaging tests passed;
the separate ignored package-signature test is superseded here by the actual updater's
successful verification of the full alpha.7 setup download.

### Randomiser compatibility prompt failure (r2)

R2 passed BG1 preparation, EET, the selected NPC/kit layers and ordinary Tweaks Anthology,
then stopped at ledger 206 on Randomiser `1100`. This is not another item-removal defect:
`lib/arrays.tpa` printed the compatibility question but the frozen recipe had no answer.
WeiDU received EOF, rolled back `1100`, installed the later requested Randomiser utilities,
and exited 2. Exact log verification correctly required a fresh copy. R2 must not resume.

The earlier approved installer design already records `y`: leave other mods' required
items in place. The question appears with Xan `0` or RR `12` in the supported curated
stack. The fix adds `when_any_features` to authored prompts, evaluated against effective
selection without activating any mods. The `y` response remains output-gated; disabling
both triggering features produces no pending prompt. Unknown feature IDs are rejected.

Recipe alpha.6 also restores the curation catalog's Randomiser-after-SCS position. Its
component choices remain unchanged. A focused audit of all 12 remaining r2 runs found no
other unmodeled reachable prompts: SCS READLN branches are debug/test-only in the pinned
configuration, other CDTweaks questions belong to unselected components, and the fresh
Randomiser run has no prior state requiring a preservation answer.

R2 diagnostics were exported to `target/curated-randomiser-prompt-failure-20260905/`
(498 files). R2 remains as failure evidence; no cleanup has been attempted on it.

### Native component-order failure and fresh replacement

The first curated UI run stopped at ledger record 154, `install:bg1npc-bg1`, with
`fresh_copy_required`: exact WeiDU suffix mismatch. Its WeiDU process exited **0** and
installed all eight requested components; the only difference was native `240, 200`
versus recipe `200, 240`. The curation catalog already documented the correct order.
Do not resume or edit that frozen alpha.4 copy.

A bounded read-only audit of 36 staged TP2 catalogs covered all 43 selected runs. It found
no missing IDs and two further order errors in Artisan main and Randomiser. All three are
fixed in recipe alpha.5. Before starting r2, every run's selected component set was compared
with the failed receipt: all are identical, only those three sequences changed. The plan
still contains 43 runs / 430 components and public-alpha validation has zero findings.
The red/green regressions and all 28 authoring-tool tests pass. Fix commits: `db1fbba` and
`42178af`; generated static evidence references the latter.

Failure diagnostics are retained separately in `target/curated-bg1npc-order-failure-20260905/`
(194 files). Compact source audit: `target/curated-component-native-order-audit-20260905.md`;
durable source findings are also in [source/order evidence](curated-full-sources-order.md).
The old managed folder is still present: its cleanup command was rejected before execution.
The proposed `frozen-state` copy inside the diagnostic directory was part of that rejected
command, so it was not made. Do not retry this deletion through another route.

Replacement r2 reused the verified acquisition cache and finished staging both games;
ledger 79 is materializing BG1 UB. The worker remains PID 81680 at this checkpoint.
Native UI start, error handling, manual acquisition and resume were exercised on the first
attempt; the replacement uses the CLI to keep the fresh rebuild in the background.
After it completes, exercise the packaged launcher against its registered install ID and
perform the remaining Radar/game/update acceptance, without substituting CLI start for a
claim that every UI lifecycle has passed.

The failed native view still offers a generic Retry action for `fresh_copy_required` and
announces an empty phase list as complete. The engine blocks an unsafe retry, but the UI
must explain the need for a new copy and remove this affordance before public release.

App alpha.6 was built and signed, then passed the real updater check/download/byte-identity
and one-byte tamper rejection test as an alpha.5-to-alpha.6 update. Its NSIS package is
5,093,916 bytes, SHA-256 `3c991e61049807bacd1f0ad9265a66156f656af2057c3f165447008e59dc1896`.
The old stopped native window was closed; silent NSIS installation exited zero and the
installed EXE now reports alpha.6. The local unpublished feed is
`target/cebg-release/0.1.0-alpha.6/` and labels recipe alpha.5 separately.
This is still not an automatic updater apply/restart acceptance claim.

### Earlier acquisition/progress fix (historical first run)

Live defect found and fixed before any WeiDU work: attempt 1 correctly stopped at the missing
manual Evandra archive, but every 64 KiB progress event rebuilt the whole UI, causing a large
display backlog. Tauri Channel GC was ruled out by its actual installed source. App alpha.5
now avoids redraws for closed-log progress/console events and caps open-log redraws at 10 Hz;
manual/error/completion transitions stay immediate. All 64 frontend tests/typecheck passed.
The new signed NSIS was installed successfully and its actual signature verified.

Native alpha.5 relaunched with `--install-id=install-66c8b55f3690bda2e3a5`. The selected
folder was visually verified before Continue. Attempt 2 immediately and correctly showed
the missing archive request; the actual file picker then supplied the exact local archive.
Attempt 3 has passed that acquisition and is downloading the remaining sources with a
responsive UI. This is historical; that alpha.4 copy later failed and must not be resumed.

Earlier checkpoint: all acquisitions completed, BG1 staging completed, BG2 staging started
(ledger record 77). The current bounded follow-up now points to replacement r2 above.

While downloading normally, do not poll repeatedly. The manual Evandra source may request
`C:\CEBG-creator-full-cache\manual\creator-full-private-extras-20260902.zip`; supply that
exact user-owned archive through the UI. Its narrowed frozen contract publishes only Evandra.
Do not copy the old extracted-layout cache or stage its other mods.

Small UX follow-ups observed, not reasons to interrupt a working installation:

- A restarted/resumed run lacks its phase list; the empty list incorrectly announces
  "All phases complete" to accessibility. Populate resume phases from its frozen plan.
- The manual-source card leaks the raw native error/path; retain friendly instructions and
  move the diagnostic text to Technical log.
- My installs/Updates remain visually enabled while navigation deliberately stays on the
  active build. Make that state clear; do not confuse it with a broken click.

These are recorded for later, not an invitation to restart the entire test for cosmetic fixes.

After acquisition/staging, let all 43 runs finish. On a failure, inspect the exact current
attempt and report; fix an in-scope installer/recipe defect with bounded verification, but
never alter frozen evidence or remove selections to force success. For a recipe change,
follow engine fresh-copy requirements and preserve failure evidence.

The app still shows old incomplete installations in the launcher. Do not use their Continue
buttons: no invalid historical recipe should be resumed. A future release should also hide
or quarantine legacy-replay resume affordances; profile selection itself is already blocked.

## Boundaries and cleanup inventory

Future NEW agent test installations belong under `C:\CEBG-Tests\<unique-run-name>`
(user requested a different destination after the cleanup backlog). Do not relocate any
existing blocked target there, and do not move/restart the active r5 installation. Keep
ordinary player destinations configurable and unchanged. There is only one local fixed
drive currently detected (C:); this is ownership separation, not a second-disk migration.
Changing the destination does not establish that a future cleanup command will be allowed.
Keep the active run and any successful copy promised to Christopher; preserve compact
failure diagnostics and clean obsolete full copies when allowed, reporting blockers early.

All clean Steam sources, `C:\Games` references, saves and stream installations remain
untouched. Do not weaken validation, edit a frozen recipe/ledger or omit a curated component
to obtain a passing run. Deferred mod design discussions remain deferred.

Seven cleanup targets were rejected by the tool policy before execution. Do not retry
those deletions by another route, and do not report them as removed:

- `C:\CEBG-Full-20260905` (`install-daea3ea2007ad86185b2`).
- `C:\Users\chris\Games\CEBG-alpha-test-20260905` (`install-3e091429c2b2d8889c15`).
- `C:\Users\chris\Games\CEBG-Curated-20260905` (`install-66c8b55f3690bda2e3a5`).
- `C:\Users\chris\Games\CEBG-Curated-20260905-r2` (`install-55cd391fcb89a92eac0c`).
- `C:\Users\chris\Games\CEBG-Curated-20260905-r3` (`install-c117933c9f2de8edc020`).
- `C:\Users\chris\Games\CEBG-Curated-20260905-r4` (`install-83f1bf87d5eb83425ed7`).
- `C:\Users\chris\Games\CEBG-Full-20260905` (`install-8a3cab271f29d2c47f61`).

Christopher explicitly requested removal of the seven obsolete copies after the inventory
found approximately 106.14 GB of files. Normal PowerShell deletion was attempted again
under that new request and was again rejected before execution (`blocked by policy`).
No deletion occurred and no alternative route was attempted. All seven identities were
checked, no reparse/save directories or exact-target processes were found, and diagnostic
archives were preserved in `target/cleanup-20260905/<install-id>.zip` (6,495,871 bytes total).
The initial aborted root had no receipt, so its entire small 2.4 MB folder was archived;
the other six use the engine's sanitized diagnostic export. These archives are local only.
The current r5, accepted focused SoD test, user's earlier Chriz Easy BG root, source games
and stream installation remain excluded from cleanup.

The separate invalid legacy full run `C:\Users\chris\Games\CEBG-Full-20260905`
(`install-8a3cab271f29d2c47f61`) remains historical evidence, not the corrected run.
Preserve the user's earlier `C:\Users\chris\Games\Chriz Easy BG` root.
Never delete a broad game directory, source, cache or workspace as test cleanup.

No public repository creation, push, tag, release publication, private archive upload or
signing-key exposure is part of this acceptance run.

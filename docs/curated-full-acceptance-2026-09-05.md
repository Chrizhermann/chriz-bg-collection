# Corrected full collection: installation acceptance

User request, 2026-09-05: complete the real full installation and test flow, including
downloads, updates, launcher/game checks and cleanup. Keep a successful install for review.

## Authority and current state

- Worktree: `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- Frozen recipe: `recipes/curated-full-current`, preset `chris-recommended`,
  recipe version `0.1.0-alpha.5`; signed app alpha.6 is packaged and installed locally.
- Selection comes from curation and later approvals, never a replay of historical logs.
  See [the approved reconciliation plan](plans/2026-09-05-curation-reconciliation.md).
- The replacement real installation **is running through the same engine CLI backend**.
  Target: `C:\Users\chris\Games\CEBG-Curated-20260905-r2`.
  Install id: `install-55cd391fcb89a92eac0c`.
  Recipe SHA-256: `43fb80da6240a24bd31fdb6a554f4d17b6e267463db1bf2413162af6be449187`.
  App cache: `C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection`.
  CLI PID: `81680` (verify identity before acting), executable
  `target/full-install-20260905-r3/chriz-bg-install.exe`. This fresh attempt honestly has
  no originating desktop-app version; launcher rediscovery remains part of acceptance.
  CLI logs: `target/curated-r2-install.stdout.log` and `target/curated-r2-install.stderr.log`.
- The NSIS installer retained its earlier registered smoke-test location:
  `target/nsis-smoke-20260905-022935/chriz-bg-app.exe`. That installed app now reports
  alpha.6 and is closed; it is not the replacement run's owner. The old
  `%LOCALAPPDATA%\Chriz Easy BG` alpha.1 executable is also closed. Do not delete
  the NSIS test location while preparing/upgrading the packaged launcher.
- Durable progress is under the new root's `.chriz/ledger/` (latest numbered records),
  with frozen recipe under `.chriz/recipe/`. Do not dump record zero: it is a large payload.
  Use the engine report and bounded latest ledger records. Never start a concurrent resume
  while the CLI worker runs; the old native window is not the replacement's progress view.
- The invalid `creator-full-current` recipe and old failed installations must not be resumed.

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

All clean Steam sources, `C:\Games` references, saves and stream installations remain
untouched. Do not weaken validation, edit a frozen recipe/ledger or omit a curated component
to obtain a passing run. Deferred mod design discussions remain deferred.

Three cleanup requests were rejected by the tool policy before execution. Do not retry
those deletions by another route, and do not report them as removed:

- `C:\CEBG-Full-20260905` (`install-daea3ea2007ad86185b2`).
- `C:\Users\chris\Games\CEBG-alpha-test-20260905` (`install-3e091429c2b2d8889c15`).
- `C:\Users\chris\Games\CEBG-Curated-20260905` (`install-66c8b55f3690bda2e3a5`).

The separate invalid legacy full run `C:\Users\chris\Games\CEBG-Full-20260905`
(`install-8a3cab271f29d2c47f61`) remains historical evidence, not the corrected run.
Preserve the user's earlier `C:\Users\chris\Games\Chriz Easy BG` root.
Never delete a broad game directory, source, cache or workspace as test cleanup.

No public repository creation, push, tag, release publication, private archive upload or
signing-key exposure is part of this acceptance run.

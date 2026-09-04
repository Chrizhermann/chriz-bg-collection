# Corrected full collection: installation acceptance

User request, 2026-09-05: complete the real full installation and test flow, including
downloads, updates, launcher/game checks and cleanup. Keep a successful install for review.

## Authority and current state

- Worktree: `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- Frozen recipe: `recipes/curated-full-current`, preset `chris-recommended`,
  recipe version `0.1.0-alpha.4`; app is now `0.1.0-alpha.5` after the live progress fix.
- Selection comes from curation and later approvals, never a replay of historical logs.
  See [the approved reconciliation plan](plans/2026-09-05-curation-reconciliation.md).
- The new real installation **is running from the installed alpha.4 native UI**.
  Target: `C:\Users\chris\Games\CEBG-Curated-20260905`.
  Install id: `install-66c8b55f3690bda2e3a5`.
  Recipe SHA-256: `3c0386d4033ea093ba861abd05ab080060eafca21d9169f21efb31bdff3f74b9`.
  App cache: `C:\Users\chris\AppData\Local\dev.chrizhermann.bgcollection`.
  Current app PID: `84760` (verify current executable identity before acting).
- The NSIS installer retained its earlier registered smoke-test location:
  `target/nsis-smoke-20260905-022935/chriz-bg-app.exe`. That installed app reports alpha.4
  (now upgraded to alpha.5) and is the active runner. The old `%LOCALAPPDATA%\Chriz Easy BG` alpha.1 executable is
  not the test runner and is closed. Do not delete the active NSIS test location.
- Durable progress is under the new root's `.chriz/ledger/` (latest numbered records),
  with frozen recipe under `.chriz/recipe/`. Do not dump record zero: it is a large payload.
  No CLI output log exists for this UI-started run. Use the engine report, bounded latest
  ledger records and the native UI. Never start a concurrent resume while the UI worker runs.
- The invalid `creator-full-current` recipe and old failed installations must not be resumed.

## Practical acceptance

- [x] Full ready recipe resolves; included/default/conditional/deferred reconciliation saved.
- [x] Packaged app defaults to corrected full recipe; historical replay absent from bundle.
- [ ] New isolated install completes using clean, unmodified BG1/BG2 source copies.
- [x] Download/cache/retry coverage includes real artifacts and bounded failure cases.
- [ ] Receipt version and exact installed WeiDU component list agree with the frozen plan.
- [ ] Launcher rediscovery, Play, game-folder action and update screen exercised.
- [ ] Signed update check/download/rejection tested; apply/restart coverage stated separately.
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
responsive UI. It is safe to continue monitoring this run; never rewrite its frozen recipe.

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

Two earlier cleanup requests were rejected by the tool policy before execution. Do not retry
those deletions by another route, and do not report them as removed:

- `C:\CEBG-Full-20260905` (`install-daea3ea2007ad86185b2`).
- `C:\Users\chris\Games\CEBG-alpha-test-20260905` (`install-3e091429c2b2d8889c15`).

The separate invalid legacy full run `C:\Users\chris\Games\CEBG-Full-20260905`
(`install-8a3cab271f29d2c47f61`) remains historical evidence, not the corrected run.
Preserve the user's earlier `C:\Users\chris\Games\Chriz Easy BG` root.
Never delete a broad game directory, source, cache or workspace as test cleanup.

No public repository creation, push, tag, release publication, private archive upload or
signing-key exposure is part of this acceptance run.

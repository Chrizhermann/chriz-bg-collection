# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## Alpha.15 published; no-SR installation recovered — 2026-09-07

Christopher approved publishing BG Rebalance v0.3.2, updating the collection pin,
and recovering `C:\Users\chris\Games\ChrizEasyBG-No-SR-Test` without a full restart.
The original alpha.12 recipe failed only Rebalance 401; its seven successful
siblings form the exact top tail after 372 BG2 components. Use the existing
supervised recovery lineage, not a rewritten ledger or an unsafe Resume bypass.
BG Rebalance v0.3.2 is published and independently verified. The existing no-SR
copy now has all 429 components (BG1 27 / BG2 402), including 401 and BuffBot,
with no full restart or repetition of the earlier 372 BG2 components. Its separate
supervised recovery receipt is published; the historical failure evidence is
unchanged. Launch `game/InfinityLoader.exe`; gameplay acceptance remains pending.
Do not run the recovery stages again. BG Radar Overlay 2.5.0.0 is installed and
verified too; final WeiDU logs remain unchanged. Collection alpha.13/app alpha.15
are published, the public updater feed is verified, and the website task has the
release tuple for deployment. Public setup is also in Downloads. See
[scope and evidence](issues/no-sr-tempus401-2026-09-07.md) and
[package acceptance](patch-acceptance-alpha15-2026-09-07.md).

The earlier Alpha13 test completed and Christopher subsequently reported that its
quick gameplay smoke test looked good. Bardic Wonders' agent captured the needed
compatibility evidence separately. Its separate fix is not included in this patch.
The no-SR two-hour follow-up is paused because its failure was inspected already.

## Overnight installation succeeded and retained — 2026-09-07, 07:57 KST

`C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` completed successfully:
43 actual WeiDU runs, all 434 frozen components, BG1 27 / BG2 407 exact final log
rows and hashes verified. SoD Remix's 32 components and BuffBot 1/0 are installed.
No restart/deletion or repetition of the five completed pre-EET runs occurred.
Worker 40540 has exited; final ledger is 236. Do not resume/reinstall this completed
copy. It is retained for Christopher's in-game check; gameplay has not been tested.

The launcher is `game/InfinityLoader.exe`. BG Radar Overlay 2.5.0.0 was also installed
through the existing verified add-on helper and its executable receipt hash checked;
mod logs stayed unchanged. Overnight heartbeat is paused. Desktop app itself still
needs the alpha.14 update from Downloads because unattended setup was blocked.
Do not confuse the receipt's frozen original app alpha.13 label with its unchanged
current collection alpha.12 or the alpha.14 engine used to recover it.

Nonblocking installed-with-warning findings are preserved for later compatibility
work, notably Bardic Wonders 2004 skipping Symphony of the Dark Children because
its finite Abettor controller was unrecognized. Do not silently fix the completed
copy. Exact evidence and acceptance limits: [completion report](overnight-install-complete-2026-09-07.md).

## Public source and release preparation — 2026-09-07

Christopher authorized publishing the existing source repository under MIT for
CEBG-owned code. The publication line integrates the finalized app alpha.14 source
and retains recipe alpha.12 / 434 recommended components. Current source/build instructions
are in [BUILDING.md](BUILDING.md); the [publication record](publication-source-2026-09-07.md)
tracks the public ref, hosted Windows checks and preserved binary/update URLs.

Public source preparation is complete. [Windows CI run 34064618951](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/34064618951)
passed at `36b59971107fe6ef02f1ed312fb904fe4f82fc93`: 620 normal native tests,
139 frontend tests, 42 Python tests, formatting/Clippy/notices and an unsigned NSIS
build. The downloaded CI setup's checksum and unsigned status were verified.
Issue 2's fresh updater-harness startup failure is fixed by test-only manifest
linkage. Build-readiness fixes preserve the recipe/pins and released alpha.14 bytes;
they include test corrections and one equivalent launcher consistency lint cleanup.

The original source audit is historical. EET excerpt attribution is recorded in
[the attribution review](audits/2026-09-07-eet-attribution.md), with upstream rights
preserved. The dependency notices cover separately licensed dependencies; MIT is
not a relicensing of those components. No third-party game/mod archive belongs here.

The installer task's subsequent shortcut/preferences/recovery work is independent
of this publication line. Do not replace its dirty worktree or the running alpha.13
installation. Publishing source does not publish a new app version or alter the
existing `chriz-easy-bg` downloads/updater. Authenticode enrollment remains separate.

## Alpha.14 published; overnight test continued without restart — 2026-09-07

- App **0.1.0-alpha.14** is [published](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.14).
  Signed public setup/download verified; live alpha update feed points to it.
  Collection **alpha.12**, all 434 recommended components/pins, and user selections
  are unchanged. Build source commit: `5610783590ad49b24101d5d3a6a85b018abadb7e`.
- Christopher's exact `C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` installation
  resumed at EET attempt 2 with the new release engine. Ledger 156 proves EET
  completed; EET components 0/100 are recorded. Main mods are now installing.
  Five prior successful runs were preserved. Nothing was deleted or restarted.
- Background CLI worker started as PID 40540; check its current identity/status,
  newest ledger and `target/cebg-overnight/alpha14-resume-20260907/stderr.log` before
  doing anything. Do not start a duplicate worker. Same native registry ID:
  `install-5f63e5d7939d3f509139`. The CLI uses the native app's frozen-campaign engine,
  but native GUI process controls cannot attach to this background worker.
- The old failed alpha.13 app closed normally. Unattended NSIS execution was blocked
  by the tool environment before starting, so the installed app remains alpha.13;
  do not retry that blocked operation by another route. The verified alpha.14 setup
  is in Downloads for Christopher to apply. This does not affect the running patched
  game-install engine. Native updater apply/restart acceptance remains unclaimed.
- Reused heartbeat `cebg-overnight-install-follow-up` checks every 20 minutes,
  staying quiet while progress is normal. On completion verify the receipt, retain
  the successful copy and pause the heartbeat. On failure preserve evidence and use
  bounded diagnosis/safe recovery. Cleanup/restart of this exact test is authorized
  only as a fallback; stream games, source games, saves and shared cache are protected.
- Website alpha.14 deployment is verified live at `/collection`: PR 9, merge
  `a47e11d6a170ad09a6b95cf28cddc5ea64bc3025`, all 76 site tests and public
  version/hash/download checks passed. No Discord/email was posted.
  The separate source audit task reports the source repository is now public and
  `origin/main` includes build source `5610783` plus audited docs/CI. Do not override
  its work or force-push. Remaining cleanup UI is documented, deferred, not implemented.

See [patch acceptance](patch-acceptance-alpha14-2026-09-07.md),
[incident](issues/alpha13-eet-pre-spawn-2026-09-07.md), and
[cleanup roadmap](plans/2026-09-07-managed-install-cleanup.md).

## Launcher friction patch + current EET recovery incident — 2026-09-07

**Overnight update:** the pre-spawn recovery/process fixes pass focused tests and
Christopher's exact `CEBG-Tests/Alpha13-20260907` copy resumed at EET attempt 2,
ledger 155, at 06:37 KST. Five previous mod runs were retained. Background worker
PID at launch is 40540; logs: `target/cebg-overnight/alpha14-resume-20260907/`.
Do not launch another worker or restart the stack. App alpha.14 is now published;
recipe alpha.12 is unchanged. Full install completion is not yet established.
Christopher authorized releasing a fix and only using cleanup/restart if recovery
cannot safely work. Cleanup UI is deferred: [roadmap](plans/2026-09-07-managed-install-cleanup.md).

Source-only launcher patch: separate Play shortcut names with safe collision handling,
unfinished setup preferences restored with fresh checks, and visible native error
details. Windows shortcut tests 7, native command contracts 39, frontend tests 139,
TypeScript and Vite build pass. These changes are included in alpha.14 without
changing the recipe. See [scope/acceptance](plans/2026-09-07-launcher-friction-patch.md).

Christopher's live alpha.13 test stopped before invoking EET: the process check saw
`game/EET/bin/win32/x86_64/weidu.exe`. The five preceding mod runs succeeded. That
process was absent during inspection; EET has only a before-log, identical to current
WeiDU.log. Alpha.13 mishandles this never-spawned evidence shape, so he was
asked to preserve the folder and hold off on Retry in that old app. Process-liveness
and pre-spawn recovery fixes are complete in the alpha.14 source. Do not restart the entire
installation or claim other agents caused the process match without evidence.
See [incident/evidence](issues/alpha13-eet-pre-spawn-2026-09-07.md).

## EET Documents-path and quiet-install fixes — 2026-09-07 (app alpha.13)

The community EET failure is an upstream Windows path parser truncating a spaced
Documents registry value, not a timeout or missing first game launch. A narrowly
hash-checked staged-file correction runs before EET core, preserving original
downloads/cache and recording exported per-attempt compatibility evidence.
Focused tests and a real verified WeiDU249 `--nogame` macro test pass.

The separate quiet-install notice now waits five minutes, clears on resumed output/
next successful step, truthfully says the process is still running and offers
**Keep waiting**. Frontend 111, Python 35 and focused native/engine checks pass.
App alpha.13 is publicly published, with signed package/public-download verification
and the live alpha Updates feed checked. Recipe alpha.12 and all 434 recommended
choices/pins remain unchanged. The website owner has the verified download and is
updating `/collection`; website deployment is not yet confirmed here.
See [acceptance](patch-acceptance-alpha13-2026-09-07.md)
and [release notes](releases/0.1.0-alpha.13.md).

Christopher chose that the affected tester can **start new with the fixed version**.
Do not spend more time on manual recovery instructions: the old frozen recipe also
contains SoD0.6.7, whereas the current recipe includes corrected SoD0.6.8. No game
or tester folder was modified or restarted by this task; no full install is queued.

## Installation-folder hotfix — 2026-09-07 (app alpha.12)

A community user had to manually create the default user Games directory. The exact
engine bug was reproduced: startup required the immediate install parent to exist,
although read-only path inspection accepted a new nested path. Startup now creates
missing ordinary ancestors and exclusively claims the final root, retaining reparse,
occupied-folder and other safety checks. Both location views explain automatic folder
creation, and the main Browse control remains available after a rejected location.

App alpha.12 is published and signature/download/tamper-checked, with all 110 frontend,
27 orchestrator, 19 preflight, 39 native command-contract, 3 package and 35 Python
checks passing. The recipe is unchanged at alpha.12: the same 434 recommended
components/source pins. No game install or native-window control was used.
See [folder-hotfix acceptance](patch-acceptance-alpha12-2026-09-07.md) and
[release notes](releases/0.1.0-alpha.12.md). The [public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.12)
and mutable alpha updater feed passed anonymous URL/hash/signature checks. A verified
setup is in Downloads. The [website](https://bg.chrizfader.org/collection) update is
deployed: site PR 5, merge `2f13d9196a0efb47f5c84fe97a227c2f15adbd85`. Owner confirmed
75 passing site tests, builds/CI, live content and an independent public setup hash
match. Full website handoff evidence is in the acceptance note above.

## Community SoD 900 fix — 2026-09-06 (published alpha.11)

The supplied diagnostics confirm an overly strict SoD Remix v0.6.7 chest guard,
not a Steam-version defect or user error. Component 900 rejected three existing
items, rolled back zero files, and WeiDU then installed 910. Earlier 310 active
components remain unchanged; BuffBot was not reached. Preserve the failed folder.
See [recovery assessment](issues/sod900-community-recovery-2026-09-06.md) and
[patch acceptance](patch-acceptance-alpha11-2026-09-06.md).

SoD Remix v0.6.8 is publicly released and freshly verified; CEBG now pins it without
changing the approved 434-component recommended selection. App alpha.11 / recipe
alpha.12 signed package is published and passed signature/download/tamper checks,
including a fresh anonymous public download. The mutable alpha feed now offers it.
[Public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.11)
and local Downloads copy are ready; the website owner is deploying the page update.
This patch includes the clarity/support work below and syntax-safe JSON diagnostic
redaction. It replaces misleading mandatory-restart wording but does not add an
unsafe Continue button: automatic targeted recovery is still private issue 3.
A supervised repair can target the SoD run and six remaining runs, without replaying
the historical 310 components, once the unchanged local evidence is available.
No game/save was modified and no full installation was restarted.

## Post-alpha clarity/support patch — 2026-09-06 (included in alpha.11)

Christopher approved a lightweight clarity/dependency pass, grouped alternatives,
installed-mod visibility, diagnostics from completed installs and first-play help.
Source implementation is in the `installer-v0-real-alpha` worktree; public app
alpha.10 / recipe alpha.11 is superseded by the published patch above; all game installations remain untouched.
See [the section-by-section audit](clarity-audit-2026-09-06.md).

- All 26 categories / 499 choices retain source/group context; ambiguous identification,
  stronghold and stacking options have concrete explanations. Repeated description text
  is suppressed, not expanded into boilerplate. Existing charcoal/brass styling stays.
- 33 explicit alternative groups use compact selectors with None/unchanged. Defaults
  remain optional, initially checked. Atomic switching only replaces group siblings;
  ordinary readiness, parent, dependency and external-conflict checks still apply.
  Three legacy BG1 NPC groups had inconsistent parents/conflicts; the authored curation
  establishes their exclusivity and those constraints are now consistent.
- My installs has a searchable receipt/current-WeiDU list, reported versions and
  missing/extra entries; it does not claim resource-byte or in-game activation checks.
  Diagnostics can be exported after restart for known installs with a terminal receipt.
  Exports stay local and warn about paths/logs before sharing. First-play tips collapse.
- Diagnostic ZIPs now start with a readable outcome/version/run-evidence summary.
  Empty BuffBot `attempts` means no finalized run record in that receipt, not proof
  that WeiDU never started or that the tester made a mistake. Raw evidence and later
  resume history must be checked. A crash before the first terminal receipt still
  prevents ZIP export; preserve the install folder and its `.chriz` evidence.
  See [diagnostic interpretation and verification](diagnostics-reading-2026-09-06.md).
- Christopher will run the install himself to check BuffBot. The community cause is
  established by its diagnostics above; do not run another full install or modify r5/
  the stream installation merely to reproduce this already confirmed failure.

Release integration: these additive presentation fields require the alpha.11 app binary;
old alpha.10 strictly rejects them. App/recipe versions are bumped together and the
new recipe ledger requires app alpha.11. Do not
publish this as a recipe-only update to alpha.10 or overwrite its immutable artifacts.
Native updater apply/restart and pause-close acceptance remain the previously recorded
follow-ups, not completed by the automated/package checks.

## Public alpha publication checkpoint — 2026-09-06

**Published and complete:** https://bg.chrizfader.org/collection now serves the
alpha download/guide/overview/roadmap/credits. Website owner verified production
merge `0cda80e65a0a9c0a5a2cb1b7a8b2b579ab8f92ca`, Cloudflare build
`7e3a258a-3af7-4593-a92f-a1ecaba12552`, 57 passing site tests, successful GitHub/
Workers builds, live page/content/header/route checks and an independent installer
hash match. Public setup and updater feed also passed this task's anonymous hash/
signature checks. See [publication evidence](publication-acceptance-2026-09-06.md).
No further full install or review loop is queued. Tomorrow's follow-ups are native
updater apply/restart and pause-close acceptance, public alpha reports, and the
already-approved post-alpha customization improvements. The fresh updater mock
harness's loader failure is private issue2, not a demonstrated app startup failure.

Christopher explicitly authorized publication after announcing the forthcoming
download on Discord. App `0.1.0-alpha.10` / recipe `0.1.0-alpha.11` is the candidate.
The separate public distribution repo `Chrizhermann/chriz-easy-bg` now exists;
**this collection source repository remains private**. Website task
`Build interactive BG run page` owns `/collection`, quick guide at top, overview,
roadmap, credits and deployment, gated on the exact verified public binary URL/hash.

Final source intake includes official Windows Evandra download-or-skip and SoD
Remix `v0.6.7`/component910. Christopher accepted the six-person skip Yes route;
the owner released commit `a6826ca1452cbe500c5e111f117b56a983a55d0d` with 82 Windows/
Linux tests. The first recipe's 31 SoD components now include 910 as the 32nd;
291/901 remain excluded. The early skip prompt is default inside the SoD toggle.
No full installation was restarted. R5 and all game/save sources remain untouched.

Frontend typecheck/all101 tests, 23 Python curation/generator tests, native command
contracts39, focused recipe/archive/materialization/validation tests passed.
RAR extraction safety review issues were fixed and rechecked; all183 Evandra files
match the accepted reference. Public bundle resources now exclude creator/live
reference TSVs and authoring inventories, and include application/UnRAR notices.
Signed packaging and public download/feed verification are next; do not claim
native updater apply/restart, close/pause interaction or another end-to-end install.
See `docs/releases/0.1.0-alpha.10.md` for the player-facing scope/limits.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Current priority — curation-derived recipe, not historical replay (2026-09-05)

**Latest approvals (2026-09-06):** [release intake and website coordination](plans/2026-09-06-release-intake-and-website-roadmap.md).
Christopher reports r5 gameplay without crashes/issues. Include released utility
XP610, Yoshimo/Hexxat choices220-223 and SoDv0.6.6 ending290 in the next recipe;
Hexxat's approved default is Shadowdancer221;222/223 remain alternatives. Imoen620 and dragon
work remain WIP. FullSoDskip is reportedly nearly ready: owning-task readiness
refresh requested before intake. Website task owns roadmap aggregation; later
UI-mod selection includes compatibility checks. Recipealpha10 source integration
and focused verification are complete; see [intake evidence](recipe-intake-2026-09-06.md).
Safe pause/app-close is implemented with focused automated checks;
native interaction and real WeiDU pause acceptance remain pending. These changes
were packaged and signed locally as app alpha.9 / recipe alpha.10; see
[package evidence](package-acceptance-2026-09-06.md). R5 is not being modified or rebuilt.
Christopher approved friendlier exclusive-choice dropdown/radio controls for a
post-alpha patch, not the initial release. Current conflict blocking/reasons remain.

**Evandra public acquisition:** automatic official download was approved, but the
ordinary HTTP probe received a Cloudflare browser challenge. Christopher then
approved an upfront download-or-skip exception and supplied official Windows
`evandra-v2.2.exe`. Recipe alpha.11 now pins its exact size/hash rather than the
private aggregate; explicit RAR SFX extraction never executes the downloaded EXE.
The early UI gate and native readiness checks are implemented and focused tests
pass; final payload comparison/package integration are underway. See the
[Evandra public acquisition checkpoint](handoffs/2026-09-06-evandra-public-acquisition.md).
Do not bypass the challenge, mirror the mod, or publish the older private contract.

**Latest instruction (2026-09-06 KST): [targeted r5 recovery](targeted-recovery-2026-09-06.md).**
Christopher approved repairing just the failed modpack rather than repeating the full
installation. **R5 has now completed all 430 components through supervised recovery**:
16/16 corrected modpack components and the three remaining commands passed, preserving
the prior 383 active BG2 entries. Final exact-order audit passed. Original managed
failure history remains intact. [Managed recovery completion](managed-recovery-acceptance-2026-09-06.md)
now passed too: separate receipt and registry, final isolated save identity, exact
versions, launcher consistency, CLI report and idempotent publication. Radar 2.5.0.0
is installed and its second current-release check reused the installation. Next is
current native package/launcher, game startup/save-reload and app-update apply/restart
acceptance, not another mod installation.
R6 was deliberately stopped before mod installation. Its cleanup was tool-rejected;
do not retry deletion by another route, resume it or start a full copy. The recovered
r5 retains earlier Bardic balance.2 and all approved selections. No game smoke yet.
The status below is the preceding r6
snapshot and is superseded by the targeted recovery record.

### Historical r6 context — superseded by completed r5 recovery above

The user has now requested the complete corrected installation/test/cleanup flow. Continue
from [the current acceptance run](curated-full-acceptance-2026-09-05.md), not the invalid
overnight replay. Recipe reconstruction and source/order verification run in parallel.

Start with [the approved reconciliation path](plans/2026-09-05-curation-reconciliation.md).
The overnight `creator-full-current` recipe bypassed recorded curation; it is invalid as
the curated collection. **Do not repair Bristlelick or resume that selection.** Curation
files are intact. Restore them as the authority, preserve deferred mod work, check changed
ordering against mod documentation/source. **Fresh r6 is running** at
`C:\Users\chris\CEBG-Tests\CEBG-Curated-20260906-r6` with recipe **alpha.9**.
Released modpack **alpha.5** fixes both r5 failures (170 Xan / 192 Viconia), with
public-component installation, repeat and uninstall checks on the actual captured r5
input resources. Bardic Wonders **balance.3** is also pinned. Both packages were
independently downloaded, hash/layout verified and extracted by the author verifier.
The resolved **43 runs / 430 components have identical selections, order and arguments**;
all 30 approved SoD components remain, Aura and 290 remain deferred. No new companion
or utility-XP defaults were invented. See the [current acceptance record](curated-full-acceptance-2026-09-05.md)
for the exact worker, startup retry and checkpoints. Full integration is not yet proven.

R5 remains read-only terminal evidence at `C:\Users\chris\Games\CEBG-Curated-20260905-r5`,
installation `install-e6325c7c98451ad4901e`. Ledger 222 is `fresh_copy_required`;
14 other modpack components succeeded and the previous 383 WeiDU rows were unchanged.
The [resolved owning-repo handoff](handoffs/2026-09-06-modpack-xan-viconia.md) retains
the original diagnosis. Do not resume r5 or change its recipe/seal. SoD and BG Rebalance passed.
SoD v0.6.5 is published, independently downloaded/hash-verified, and pinned. Its mod bytes
match the candidate Christopher accepted in focused gameplay/save-reload testing. See the
[owning-repo handoff](handoffs/2026-09-05-sod-remix-component-120.md).

The prior native-order and conditional Randomiser-answer failures are fixed. R4 then hit
a recipe TP2-alias mismatch after all 30 SoD components succeeded. The nested canonical
path is corrected with regression coverage; no component choices changed. Failed r3/r4
remain immutable and must not resume. App alpha.8 is built and installed locally and fixes
the misleading Retry option when a new copy is required. Radar Latest 2.5.0.0 download,
layout and seven engine tests passed; add it only to a successfully completed copy. The
[current acceptance record](curated-full-acceptance-2026-09-05.md) has the exact worker,
frozen hash, logs, remaining launch/update acceptance and cleanup boundaries. Full fresh
collection acceptance is **not complete**. Older packages/status below are historical;
there is still no public CEBG release/update channel. Installer acceptance is the priority.

Christopher's [short alpha-release path](plans/2026-09-05-alpha-release-short-path.md)
avoids another whole install merely to start it through a different entrypoint. Source
update notifications and the post-app-update/new-setup action are now visually tested
([evidence](updates-ui-acceptance-2026-09-05.md), 80 frontend tests); the installed alpha.8
predates this slice. Next signed candidate must include it. Public Evandra acquisition
needs the verified original standalone package, not the creator's private aggregate ZIP.
The explicit seven-copy cleanup request was tool-rejected too; see the acceptance record.
Christopher is now deleting those copies manually. Do not race his cleanup or assume each
folder still exists. No agent workaround deletion is allowed; the agent removed nothing.

Christopher requested a separate destination for future tests. Use
`C:\Users\chris\CEBG-Tests\<unique-run-name>` for NEW agent-owned disposable installations,
not the user's normal Games folder or any folder directly under C:. Christopher rejected
the earlier root-level proposal; it was not created or used. Keep frozen r5 where it is; no move,
rename or restart is implied. Public/player install defaults are unchanged. This separation
is for clear ownership and cleanup, not a workaround or guarantee about deletion policy.
Preserve small diagnostics and retire failed full copies when permitted; surface a blocked
cleanup backlog before starting additional large test copies.

Christopher also permits today's finished mod work in the first alpha. The
[cross-repo intake queue](plans/2026-09-05-first-alpha-mod-intake.md) records two
released modpack/Bardic updates now included in alpha.9, utility XP 610 awaiting
selection policy, unselected Yoshimo/Hexxat additions, and remaining WIP. New choices
belong in a later versioned candidate; do not mutate r6, invent defaults, or let WIP
become an installer blocker.

Latest queued requirements are captured in [recovery and casual customization](plans/2026-09-05-recovery-and-casual-customization.md).
Dual update notification is implemented in source. **Safe pause/app-close handling is
missing and is the next installer priority**: Cancel currently kills WeiDU and must not be
advertised as Pause. A read-only recovery matrix and bounded test plan are documented.
The [failure-detail slice](failure-ui-acceptance-2026-09-06.md) is now implemented
in source: retain the actual report reason, identify safely attributable missing
component log rows, keep fresh-copy recovery strict, and do not announce empty
progress as complete. 91 frontend / 9 focused CLI tests and responsive checks passed;
not in installed alpha.8. The fresh r6 CLI binary includes these error-detail improvements.
The requested [common Customize controls](customization-acceptance-2026-09-06.md) are now
implemented in source: six bundles, category bulk actions, preserved preferences and
visible collateral, with 88 frontend / 16 recipe-view / 4 authoring tests passing.
Headless real-catalog layout checks passed. Not in the installed alpha.8 yet; authoring
metadata is now in alpha.9 with no recommended-selection delta. Yeslick's original-class
route currently also excludes combined modpack dispel fix 410; owning-repo follow-up is
recorded. Typed advanced inputs and broader Customize QoL remain; no default curation changed.
Optional confirmed SoD skip with +250,000 protagonist-only XP was dispatched to
the owning SoD task; separate from component 290 and frozen r4, not a first-alpha blocker.
Its native ground-pile probe subsequently failed (owning commit 19e221b); that prototype
must not enter the recipe. Released SoD v0.6.5 is unaffected.

## Historical overnight continuation — CEBG app alpha.3 / recipe alpha.2 (2026-09-05)

Start with [`overnight-2026-09-05.md`](overnight-2026-09-05.md): current code changes,
real test paths, cleanup ownership, release boundary and remaining acceptance. The active
worktree is `installer-v0-real-alpha`, **not** `4d39`. Both Steam source games are now
verified clean. The dotted-artifact freeze/evidence failures are fixed. A compact UI,
full-creator recipe profile, Radar add-on, human receipt versions, lazy mod-list consistency
and signed-app updater are implemented; real full-install acceptance is still in progress.
The first overnight follow-up fixed EET's Windows staged-path argument; EET completed.
The second follow-up stopped on malformed Bristlelick source; the later curation audit
supersedes the proposed repair because Bristlelick was excluded. See the current priority
above. Signed app alpha.3 is built locally but its historical full profile must not be used.
No public release/channel has been published. The old status below is historical.

## Earlier Chriz Easy BG 0.1 alpha status (2026-09-05)

Branch `codex/installer-v0-real-alpha` is pushed. The installer engine and UI are
functional, and the public-alpha recipe validates and resolves **35 runs**. All **30
selected artifacts** have passed cold-cache acquisition, extraction, and payload
verification. Notable ready pins include public CHRIZ-BG-MODPACK `0.2.0-alpha.1`,
CHRIZ-SOD-REMIX `0.6.4`, Artisan's Kitpack `chriz-v1.3.1`, Spell Revisions
`v4.21-chriz.3`, and public CHRIZ-BG-REBALANCE `v0.3.1` with its ten-component curated
fresh selection.

Tasks 19, 20, and 23 are implemented: public-alpha omissions and evidence are
release-enforced; immutable recipe envelopes, update classification, and packaging are in
place; and the Tauri command surface includes restart-safe discovery and resume through the
immutable campaign index. Task 24 has a tested three-track update-center foundation, but
the production updater channel is deliberately still unconfigured.

The player-facing app is now **Chriz Easy BG (CEBG)**. With no registered installation it
opens directly on one compact install screen with detected sources, editable install name
and location, recommended choices, a clear readiness state, and one primary Install action.
With an existing installation it opens as a launcher with Play, Open game folder, install
switching, collapsed technical paths, and recovery for resumable or moved installs. The
optional CEBG desktop shortcut is checked by default and points back to the registry-verifying
launcher rather than directly to the game. Startup remains registry-first, so a completed
game can be launched even when its original source installs are unavailable.

Fresh verification after this UX slice passes 54 frontend tests, 37 native app tests, full
workspace tests and Clippy, and public-alpha validation with zero findings. Responsive checks
pass at 1920x1080, 1366x768, 768x1024, and 375x812: desktop keeps the primary action visible
without a scrollbar, and constrained screens use one column with normal internal scrolling.
The final local NSIS lifecycle smoke passed for the **4,413,788 byte**
`Chriz Easy BG_0.1.0-alpha.1_x64-setup.exe` with SHA-256
`7BF879133A0E49E67A811F85BCAFF98FC61AFA34B32207314F51375D4F42F227`: its branding and
version were correct, all 78 bundled manifest files hash-matched, the installed app stayed
alive for five seconds, and silent uninstall removed the isolated smoke directory.

There is **no full game-install acceptance and no public release yet**. The immediate E2E
blocker is a genuinely clean Steam BG:EE+SoD 2.7.3 source; the detected Steam BG2 source is
clean, while the available BG1 source is rejected for mod residue. These checks were
read-only and no game directory was modified. Production Minisign and Tauri updater keys
also remain to be provisioned, and Task 24's signed updater UI remains pending.

Immediate next actions only:

1. Obtain or restore a genuinely clean Steam BG:EE+SoD 2.7.3 source.
2. Run the first full installer-driven clean EET build and focused smoke before the public
   installer release.
3. Provision the production signing keys/channel and publish the first explicitly alpha
   build only after those gates pass.

Other blocked items remain later work and are not expanded here. Older status sections
below are retained as historical implementation context.

## Status (2026-09-02 — curation snapshot integrated; real-alpha implementation active)

The authoritative component-catalog snapshot from the dirty `main` checkout is preserved
in commit `1eedd8e` and integrated here without modifying that checkout. The normalized
decision semantics and per-component notes live under `docs/curation/components/`. Start
with [`docs/next-session.md`](next-session.md), use
[`FOLLOW_UPS.md`](curation/components/FOLLOW_UPS.md) for the complete evidence-backed queue,
and use [`COLLECTION_TAIL_FIXES.md`](curation/components/COLLECTION_TAIL_FIXES.md) for the
22-fix migration inventory. Explicit choices and source/release/acceptance gates remain;
do not treat every `default` row as currently installable.

The active implementation branch is `codex/installer-v0-real-alpha`. The approved current
milestone is defined by `docs/plans/2026-09-02-installer-v0-real-alpha-design.md` (`ae5855d`)
and `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md` plus its separate
27-task ledger (`fd73921`). This supersedes the fixture-only Recipe Preview as the first
release milestone. The new plan explicitly selects `ureq`'s `rustls`,
`platform-verifier`, and `win-system-proxy` features.

**Current RC status (2026-09-03):** `C:\BG-EET-RC-20260902` has been played for an
extended session without a crash, and BuffBot works. It is not suitable as the public
alpha: Item Randomiser v8.1's physical CRE-item removal left invalid inventory offsets
and references in 95 post-Randomiser creature resources, which can duplicate or omit loot
and equipment. There is no evidence of a crash or save-file corruption. Released v8.1.1
fixes the source defect; a clean rebuilt RC, new game, and Tarnesh loot smoke remain open.

## Engine Phase 1 baseline

The branch includes the `feat/engine-phase1` lineage through Task 6. Historical plan =
`docs/plans/2026-08-20-engine-phase1-implementation.md` plus its status ledger. Crate
`engine/` provides lib `bg_engine` and bin `chriz-bg-install`.

| Task | State | Notes |
|---|---|---|
| 0 scaffold, 1 schema types (`manifest.rs`) | done, reviewed | |
| 2 loader (`loader.rs`) | done, independently reviewed | original `a4cdb6a`; fixes `05fe9f7` + `05b8058`; 16 loader tests |
| 3 validators (`validate.rs`, 9 rules) | done, independently reviewed | original `5feb0a8` (merged `65e3bb8`); fix `a0c812b`; coverage `71f0387`; 41 tests |
| 4 resolve (`resolve.rs`) | done, independently reviewed | original `13df297` (merged `98c5515`); fix `4c61a8c`; 16 tests |
| 5 events (`events.rs`) | done, reviewed | |
| 6 snapshot session persistence (`session.rs`) | done | `39a6692`; nine focused tests |
| 10 fake-game builder (`tests/support/fakegame.rs`) | done, reviewed | KEY/BIF/TLK writer |
| 7 WeiDU invocation, 8 log-diff verify, 9 runner, 11–16 | not started | historical Phase-1 ledger only |

The real-alpha plan is authoritative for next-work order. Its Task 3 intentionally replaces
the Phase-1 Task 6 snapshot model with a create-once, append-only hash-chained campaign
ledger; do not confuse those two separately numbered tasks.

**Task 2 fix-up completed locally (2026-09-01):** schema is probed before strict parsing;
stem mismatch wins over duplicate-id defence; `.toml` matching is case-insensitive; file
symlinks are followed and broken ones return pathful I/O errors; roots are canonicalized;
non-UTF-8 stems have an explicit error; `Manifest::conventional_mod_path` clearly documents
that it is not an actual loaded-file lookup; fixture mods are copied as a directory. Follow-up
tests cover a real duplicate id on case-sensitive filesystems and case-variant extensions;
Windows symlink tests skip only unsupported/permission-denied link creation. Two independent
reviews found no blocking issues; their bounded follow-up requests are now covered. The
focused RED run failed in all five intended cases before implementation.

**Task 3 review completed locally (2026-09-01):** manual sources no longer inherit the
non-manual HTTPS/hash rule; hexadecimal hashes accept either case; one explicit order slot
cannot repeat a component; split `eet_end` entries must form the final contiguous main-phase
block; and aggregation is covered with two independent errors. The four intended regressions
failed before implementation and pass after `a0c812b`.

**Task 4 review completed locally (2026-09-01):** a selected choice may add a component to
the only explicit slot of an unsplit mod even when that slot did not list it originally.
Split mods still require unambiguous explicit placement. The regression failed before the
fix and passes after `4c61a8c`.

**Historical installer v0 preview:** `docs/plans/2026-09-01-installer-v0-design.md` and its
implementation plan established the guided wizard and engine-owned UI boundaries. The
static prototype at `docs/prototypes/installer-v0/index.html` remains a no-op Recipe Preview;
it does not download, copy, or install anything and is not the current release milestone.

**How to build (Windows):** Rust 1.97 stable-msvc via rustup; run cargo from **PowerShell**
with `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` — Git Bash's coreutils `link.exe`
shadows the MSVC linker. Work in an isolated worktree; never develop in the dirty primary
checkout or either protected game/archive directory. Repo files are CRLF.

**Conventions:** TDD per task (failing test → implement → green → commit "engine: …");
`#[serde(deny_unknown_fields)]` on all manifest structs; errors carry `PathBuf`s; no
`unwrap`/`expect` in library code; doc comments on public items; never write under
`C:\Games\…` (read-only reference); never hand-edit `manifest/install-order.tsv`;
curation content is Chris-only — the engine is curation-independent.

## Background (2026-08-19 — installer app designed)

- **Installer-app foundation:** `docs/plans/2026-08-19-installer-app-design.md` established
  the public EET-only, copy-then-install architecture with Tauri 2 and a headless Rust
  engine. The 2026-09-02 real-alpha design linked above is the current extension.
- `manifest/install-order.tsv` — recaptured by `fbd914b` from the reference WeiDU.log:
  **451 entries / 91 mods**. This resolved the earlier 414-vs-364 discrepancy; the 364
  figure was wrong. Never regenerate it except from the live reference install.
- `manifest/mod-sources.tsv` — 89 source rows are classified. The integrated curation
  snapshot points historical local fixes at the migration inventory, but several immutable
  pins and release artifacts still need refreshing before recipe freeze.
- `presets/` and `app/` are not started. The headless `engine/` contains the Phase-1
  baseline; the current real-alpha plan owns the remaining engine, UI, and release work.
- Parked: [#1 EET XP scaling fix](https://github.com/Chrizhermann/chriz-bg-collection/issues/1)
  (future chriz-layer component; home repo TBD).

## Hard guardrails (user directives)

1. **The game folder (`C:\Games\Baldur's Gate II Enhanced Edition modded\`) is a READ-ONLY
   reference.** Read WeiDU.log / EET_MODDING_GUIDE.md / mod folders freely; never write,
   install, or test there.
2. **Never redistribute third-party mods.** Private archiving of hard-to-find zips is
   acceptable in a private repo, but the public-facing design is links + versions. If the
   repo ever goes public, archived third-party content must be dropped/licensing-reviewed.
3. `gh` CLI auth is shared across concurrent agent sessions — `gh auth status` before any
   gh op; this repo needs `Chrizhermann`.

## Work queue

1. Follow the dependency graph and delivery waves in
   `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md.tasks.json`; do not use
   the older Phase-0 list or Recipe Preview as the schedule.
2. Resolve only Chris's remaining content choices in `docs/next-session.md`, then normalize
   those catalog rows. Its curation decisions remain authoritative, but its older engine/UI
   scheduling paragraphs are superseded by the real-alpha plan.
3. Refresh and freeze every selected immutable source, hash, license/provenance record, and
   split BG1/BG2 run. Keep blocked or unimplemented choices unavailable.
4. Implement engine and UI slices with TDD in isolated worktrees, integrating reviewed
   commits only after their focused and workspace checks pass.
5. Treat Christopher's cold-cache install, recovery rehearsal, InfinityLoader boot,
   BG1 start, and save/reload as separate live acceptance evidence—not as implied by static
   tests or catalog review.

## Known wrinkles for the driver (from the reference install's history)

- EET requires a BG1EE+SoD source install to import from; EET_end must be the last "core"
  entry, but ~50 additive components legitimately sit after it (see install-order.tsv tail).
- WeiDU v24900 template pattern: copy `Setup-Branwen.exe` as `Setup-<modname>.exe`.
- Some mods prompt interactively despite force-install flags — capture required extra args
  per mod in mod-sources.tsv `notes` as they're discovered.
- Test installs must set a distinct `engine_name` in `engine.lua` so they don't share the
  Documents user dir (saves/baldur.lua) with the live install.

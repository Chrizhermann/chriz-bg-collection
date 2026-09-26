# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## September 26 — recipe alpha.18: approved Emotion default

The prior app alpha.19 / recipe alpha.17 source was completed and committed first
as `d57b70a`. The next recipe, **0.1.0-alpha.18**, pins BG Rebalance **v0.6.0** and
makes component **301** default/ready under `mod:chriz-bg-rebalance`, with no extra
requirements. Recommended selection: **450 components / 50 runs**; the base
profile has 358 components. Bard progression `420`/`421` stays unselected.

Christopher's decision is dated **2026-09-26**: fix any issue in a follow-up,
without gating on more testing. Earlier live checks used standalone spells;
IWDification provider mode has fixture coverage. A quick Hope/Courage replacement
and refresh check on the next install is nonblocking. Source ordering and the
later CDTweaks/Modpack interactions are recorded in the
[recipe note](recipe-alpha18-2026-09-26.md).

The v0.6.0 archive was freshly downloaded, hashed and verified with the installer
authoring tool. The generated recipe, curation totals, source/evidence records,
golden digest and alpha.18 ledger are synchronized. App version stays alpha.19.
This is a source/recipe change for new installations or the next playthrough;
the September 23 package still contains alpha.17. No publication or live action.

Final source verification passed: **806 native tests**, 13 existing opt-in ignored;
formatting and Clippy with warnings denied; **105 Python tests**, one existing
opt-in skip. Native job `4e708c54-f961-425a-9878-fab6c6706bcd` exited 0; status and
logs in `target/work/sep26-alpha18-native` were inspected. Both public-alpha
profile validations returned zero findings. Planning resolves 450/358 components;
the existing 5/6 conditional-omission notices are unchanged, and disabling 301
removes only that component. Fresh generation reproduced all 101 recipe files
with normalized line endings. The semantic golden digest is
`9f5d6131c8c13887a80b6d82b592c440484d84043f38f0a0f5378ee177a29015`.
The app's 327 frontend tests/typecheck/build passed during the preceding source
closeout; this recipe change adds no frontend code.

## September 26 — finish recipe alpha.17 / app alpha.19 source

Christopher requested completion and a commit of the existing installer work
before adding BG Rebalance 301 to the next recipe. The existing comparison,
BuffBot update/Undo, naming, progress, diagnostics, Customize and recipe-record
changes are complete as source. A focused review found and corrected hidden
patch/Radar feedback on the default Updates selection; four regression cases
cover progress, success/error and switching installations.

Current checks: frontend typecheck, 327 tests and production build; Python
102 tests with one existing opt-in real-WeiDU skip; 803 native tests with 13
existing opt-in tests ignored,
formatting and Clippy with warnings denied; both recipe profiles pass
public-alpha validation with zero findings. Native job
`b38bf370-eda2-4cab-9366-b58677d4ea59` exited 0; saved results are in
`target/work/sep26-alpha17-final-native`.

The September 23 package record is historical and predates today's feedback
correction. No package rebuild, publication, installed-app or game action was
performed. Final packaged UI review remains user-driven. Existing accepted
BuffBot Apply/gameplay/Undo is not being repeated. The known concurrent-name
reservation limitation remains a follow-up. No new dependencies or public APIs
were introduced by the finishing correction; diagnostics tests contain deliberate
fake credential strings for redaction coverage, not credentials.

Completed as source in the separate recipe change above: Christopher's September
26 approved BG Rebalance 301 default with v0.6.0, leaving bard progression
420/421 unselected. The provider-mode check remains nonblocking.

## September 24 gameplay follow-ups: preparation only

September 25: Christopher authorized a parallel, source-backed Time Stop/Haste
feasibility plan, not implementation. Proposed owner: BG Rebalance. Check whether
EEex can safely separate periodic cadence and buff expiry from speed/time-stop
behavior; do not promise full tabletop fidelity or current-save compatibility.
See [research scope](plans/2026-09-24-effect-timing-traps-and-shift-dialogue.md#september-25-feasibility-research-requested).
The [source-backed feasibility report](plans/2026-09-25-timestop-haste-feasibility.md)
is now complete: credible route, but usual effect-list listeners run too late.
Next would be a separately authorized isolated proof before native expiry/pulses,
or a focused EEex extension. No implementation or runtime test occurred.

New backlog items: Time Stop/Haste effect timing, instant trap detection, and
optional off-by-default dialogue for shifted Shapeshifters/Hivemasters. See
[scope and preparation](plans/2026-09-24-effect-timing-traps-and-shift-dialogue.md).
These are not additions to the built review package or new release gates. No live
game/desktop control is authorized; Christopher is streaming. Keep the existing
installer acceptance work separate and user-driven.

## Current priorities — September 22, after accepted BuffBot Undo

Christopher lowered priority for (1) broader existing-mod updates, (2) a general
current-run/new-run/new-install classification system and (3) more hotpatches.
Keep the two existing supported routes and their protections; these expansions
are not release gates. See the updated [work index](plans/2026-09-20-next-work-index.md)
and [preserved requirements](plans/2026-09-20-existing-install-upgrades-priority.md).
He subsequently approved the four implementation lanes. Source now includes
compact Updates with real backup progress, persistent completion/Undo/error
feedback, unique display names and rename controls, contextual folder diagnostics,
and the completed Customize/typed-input frontend from `cebg-ui-options`.
Full native verification passed: **797 tests**, 13 existing opt-in tests ignored,
formatting and Clippy with warnings denied. Job
`cc6839dc-ab5b-476b-b2ff-31ff351e25cb` exited 0 at 2026-09-22T15:02:24Z;
saved status/stdout/stderr in `target/work/sep22-feedback-final-native` were read.
The older channel compile failure (`0ecc35d9...`) and generic-name test failure
(`f2299f45...`) are already fixed and superseded. Their delayed notifications do
not call for rerunning them.
A final naming review found that automatic destinations must stay free after
display-only rename. The native defaults helper now independently suffixes
occupied/registered destinations; six extra native tests cover it. The final job
passed all 38 app-library tests (eight naming), 40 command contracts, Clippy,
frontend 323 tests/typecheck/build, and four package tests including the signature.
The first package attempt stopped before packaging: two new test expectations
used mixed Windows path separators. Actual free-destination selection was correct;
the expectations now use separate native path joins. Job `a7af1479...` exit 101
is handled; do not rerun it unchanged.
Final job **`d75b1f80-08e3-4958-9b3d-bc28168d0a12`** exited **0** at
2026-09-22T15:13:36Z. Its saved status/stdout/stderr were read in
`target/work/sep23-feedback-review-package-final`. Normal release build, diagnostic
feature off. The updater signature matches the bundled public key.
Offline extraction matched all **191 resource files** to the current source,
with no extra recipe files. The executable differs from the raw build only by
Tauri's documented three-byte UNK→NSS bundle marker, with every other byte equal.
Verified setup/signature copies are in Downloads under the new, non-overwritten
name `Chriz Easy BG alpha19 - Review 2026-09-23-setup.exe` (adjacent `.sig`).
Setup SHA-256: `9cbae97c0f51505adc66cd36d897d96933982287c7487377c3d67e88b9528e6a`.
See [final package evidence](package-acceptance-alpha19-2026-09-22.md).
All subagent lanes are done; no command is still pending for this batch. The
package is ready for Christopher's finishing UI review, **not installed, launched
or published**. No further automated game/update test is requested.
UI provider **recipe** integration is still separate: the imported controls stay
hidden when the actual recipe has no verified provider group. Infinity UI++ and
EET SoD preparation remains in the owning fork; LeUI remains compatibility-blocked.
The website draft remains with its owning task. Do not describe these as shipped.
No new game actions, real-install deletion or publication is implied.
Frontend typecheck, 323 tests and web build passed. The full Python suite passed
102 tests with one existing opt-in real-WeiDU skip. Focused headless desktop/narrow
checks passed; evidence is in `target/work/sep22-feedback-layout/verification.md`.

### September 22 implementation notes

- Progress is observational only. Backup byte counts are published after verified
  copies, with final 100% after the source inventory recheck. Unknown work uses a
  spinner. Existing mutation checks, backups, recovery and save policy are intact.
- App-update success survives restart only when the requested different version
  is actually running. Mere download or an online up-to-date result is not success.
- Names live in separate AppData metadata keyed to the registered ID/root. Rename
  never rewrites receipts, frozen recipes, game/save folders or profile identity.
  Legacy duplicates remain resumable and are disambiguated visually. New creation
  and rename have cross-process uniqueness checks.
- Known narrow naming limitation: two independent CEBG processes starting the
  same name simultaneously allow only one registration, but the loser can leave
  an unindexed metadata-only destination because freeze precedes registration.
  No game copying has started. A future reservation across freeze/publication is
  needed before promising clean retry for that concurrency case; do not weaken the
  name check or delete an arbitrary destination to work around it.
- Folder codes preserve operation/path and original OS cause. Failed cleanup no
  longer hides the primary problem. This is error clarity, not a proven diagnosis
  of the viewer report or a new OneDrive/Documents fallback.

## No-SR lookup discrepancy explained — 2026-09-22, user-side repair pending

**Current acceptance: Apply, user gameplay and Undo passed.** Christopher applied
the BuffBot 1.8.3-to-1.8.4 update himself in the diagnostic app, accepted gameplay,
then used Undo. The installation is now back on **1.8.3-alpha**. The earlier Apply
disk checks confirmed successful installation of both components and
the expected new artifact. Transaction: `.chriz/mod-upgrades/v1/01790071515151257500`
inside `C:\Users\chris\Games\ChrizEasyBG-No-SR-Test`.
All 96 current update-file hashes, 96 affected-file backup hashes and 59 unchanged
guard hashes match. Original receipt and dialog.tlk are unchanged; all 400 other
active WeiDU rows match the pre-update log exactly. The prepared-journal hash is
valid. Optional backups also checked: full game has 159,300 files / 3,393 directories
/ 11,898,950,716 bytes, matching every pre-update path/type/size with no extra or
missing entries; all 12 profile files (1,456,735 bytes, five directories) match the
isolated profile byte-for-byte at the Apply audit, before gameplay. Full-game backup contents were not all
rehashed. Actual backup file handles resolve to normal AppData, not Codex LocalCache;
the applied journal and live TLK resolve to the intended No-SR game. Backup audit
helper: `target/work/buffbot-applied-verification-sep22/backup-check.ps1`.
Christopher subsequently reported **"its fine, it works"** after the requested
existing-save/BuffBot check. Record this as user-accepted gameplay for this narrow
upgrade; no further BuffBot gameplay testing is requested. Christopher then used
Undo, completed at 20:12:39 +09:00. Read-only checks confirm all 155 original guarded
game-file hashes and all 96 affected-file backup hashes match, the entire original
WeiDU.log is restored byte-for-byte, and the original receipt and dialog.tlk are
unchanged. The restored marker matches the prepared journal SHA-256
`4a56f5d5c96ce798c6935ac17601c5b611c31ecf5b632c4e2ebb0e89b217026b`.
No game process was running at verification. This accepts this exact Apply → user
gameplay → Undo flow, not other mod upgrades or publication. No agent game launch,
UI click, Apply, Undo or save edit was performed. Do not reapply automatically.
History wording is corrected in source to **Restored BuffBot 1.8.3-alpha** rather
than presenting the original upgrade arrow followed by "restored". It remains
inside closed history; applied, unfinished and unknown states also have explicit
wording. Typecheck and all 18 focused mod-upgrade tests passed. Not packaged yet.
The journal was created at 19:05:15; preparation finished at 19:37:29; the WeiDU
step ran approximately 19:37:32–19:37:35; the applied record was written 19:37:44.
Most waiting time was preparing the backups, supporting the progress-feedback request.

Additional UI requirements captured (not implemented by the compact-selector work):
- Lead with the supported action and applicability: current run / new run / new
  installation. Put comparison prose, dependency explanations and diagnostics in
  closed Details. Eligibility must be evidence-backed; unassessed work must not
  be labelled compatible. Keep consequential warnings beside confirmation.
- Reuse a compact progress display across installation, updates, downloads,
  backups and game discovery. Show the actual current stage (e.g. backing up,
  downloading, applying, verifying), a determinate bar when byte/file/step totals
  are known, and an indeterminate spinner otherwise. Never invent percentages or
  remaining time. Preserve access to optional activity details and clear final
  success/failure. This is queued UI work, not a change to the running update.
- Completion must be conspicuous for every update type: a persistent, dismissible
  **Update complete** banner at the top, installation/add-on name and new version,
  and the appropriate next action (Play / start a new run / open new installation).
  Keep Details and supported Undo accessible, remove obsolete pre-update wording,
  and distinguish success, partial completion and failure. App-update confirmation
  must survive restart and reflect the completed update, not merely "up to date".
  A newly built copy should say Installation ready, not imply the old copy was
  updated. Progress, completion and errors should be accessible announcements;
  do not rely on changed small text, colour alone or a short-lived toast.

**Latest: the user-side record repair succeeded at 16:57:21 +09:00.** Christopher
ran the supplied launcher from File Explorer. Its result file reports that the
single record was copied into normal AppData and verified at the expected SHA-256,
with the Codex copy preserved and no game/save/mod changes. No CEBG process was
running when this result was checked. He subsequently reopened it and supplied
the successful recognition/available-update screenshot recorded above.
The historical "pending" heading is retained for existing links. Do not rerun the
repair or mistake agent-side registry visibility for user-app acceptance.

The diagnostic build's real user-opened process (PID 73584) sees **one completed
record and three campaign pointers**, not the five completed records and 13
campaign pointers visible to agent-launched helpers. Its native lookup, IPC reply
and frontend acknowledgement agree. No-SR's completed-record probe returns
NotFound; its game root exists, but the remaining historical campaign is sealed
FreshCopyRequired. The UI's generic "folder unavailable" explanation is misleading.

**Proven cause: Windows redirects agent-side file reads to Codex's private AppData
copy.** Opening the apparently normal No-SR record and querying its actual file
handle with GetFinalPathNameByHandle resolves to:
`C:\Users\chris\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\Chriz BG Collection\managed-installs\install-c9dc8c3e0e0b0a3476e0.json`.
The normal user-opened CEBG process cannot see that completion record. This is why
helper checks and earlier agent-launched app checks contradicted Christopher's UI.
Do not claim a refresh or reinstall of the game fixes this. Package-identity API,
environment and directory-handle checks alone missed the redirection: check the
actual **file** handle. Do not copy all historical records, delete the package cache,
or change Codex's virtualization settings.

Prepared in Downloads, **subsequently executed successfully as recorded above**, for Christopher to double-click from
File Explorer outside Codex: `CEBG - Restore No-SR completion record.cmd` and its
same-named `.ps1`. The script restores only the 720-byte completed record into normal
AppData, verifies the exact record and existing game receipt hashes, rejects
redirected destinations and conflicting records, and never overwrites a record or
changes game files/saves/mods. Its syntax and read-only path helper were checked;
the user-run copy then succeeded. Result file: `CEBG - No-SR record repair result.txt`.
Record SHA-256: `16b3c35bf2221265eb336be9f22e4d951597af9f57db2522d727e73274f71d5d`.
Receipt SHA-256: `a06b6e0928ee0dc1463247252e4d118c21234e6bdba702ad18a3dd2061b35379`.
Verify the repair through the **user-opened app's trace**, not an agent helper.
Trace evidence: `%TEMP%\CEBG-installation-lookup-73584-1790060629093.jsonl`.

Christopher is using the PC and drives the app. No unsolicited app closing,
launching, clicking, Apply/Undo, or game actions. BuffBot update, gameplay and Undo
acceptance passed as recorded above. Latest UI request: a compact, obvious selected
installation at the top of Updates, less repeated text and smaller selection frame;
all game-specific actions must use that same selection. Independent user work:
draft the website's differences from vanilla, without publishing untested claims.

### Compact Updates selection — source implemented, not packaged

The selected game now appears directly below Updates in a compact bar. Short,
disambiguated labels replace path/ID-heavy choices; exact folder, identity and
status detail remain in a closed disclosure. Installed mods, current-game fixes
and Radar use the same selected ID. Changing games clears old comparison/fix
previews and errors; selection is locked during patch operations or mutations,
while obsolete read-only comparisons remain discardable. Radar no longer quietly
targets the independently selected My installs entry. App/collection actions are
separate below the selected game's controls. New-copy availability on this page
follows the selected game, rather than any other outdated installation.

Existing dark/gold styling retained with tighter, Updates-only spacing. Persistent
installation renaming remains separate roadmap work; these labels do not rename
game folders or rewrite records. Stale entries now say Needs attention in the
top bar rather than asserting a missing folder when other checks may have failed.

Validation: `npm run check` passed (typecheck, **223 tests**, frontend build).
Eight new tests cover shared selection, clearing old Apply controls, exact target
IDs for comparison/fix/Radar, operation locks, stale/empty states and duplicate
labels. One old Radar fixture was inconsistent (managed list `stream`, update list
`fixture-install`); its two mock lists now refer to the same game, without weakening
the expected Radar target. Headless synthetic previews at 1440x1000 and 390x844
had no horizontal overflow, including expanded long paths; controls remain at
least 44px. Evidence: `target/work/compact-updates-sep22/`. Lead inspected both
screenshots. No native app/game actions, NSIS rebuild, publication or real-game
update were performed. The user still has the earlier diagnostic build installed.

## Opt-in running-app lookup diagnostics — 2026-09-22, package verified

Christopher approved instrumenting the running app's lookup ("Then do that").
Source now has the opt-in Cargo feature `installation-lookup-diagnostics`.
Normal builds do not emit lookup traces. Diagnostic builds show
**Chriz Easy BG — Diagnostics** in the window title and automatically write
`%TEMP%\CEBG-installation-lookup-<pid>-<timestamp>.jsonl` (create-new, max 4 MiB).
No logging failure changes lookup acceptance or permits an update.

The trace captures process/executable SHA-256 and the specific LOCALAPPDATA/cwd,
actual registry directory entries, completed/campaign records and which wins,
original availability, separately labelled rechecks with exact rejected gate/IO
reason, and the final native list. Updates additionally records the native IPC
reply and a bounded frontend acknowledgement of its received IDs/names/states.
It does not copy saves/game content, dump other environment variables, upload
anything, reconstruct records, relax checks, or alter game files.

The engine now exposes `inspect_install_availability` and
`inspect_campaign_availability`; existing availability uses the same unchanged
checks, with reasons retained. Thirteen focused registry tests, six trace tests,
and 19 frontend adapter tests passed. A trace test's Windows separator expectation
was corrected; no safety assertion was removed.

Local job `6b13b22c-40c4-4edf-a231-a92af7d809dd` finished with **exit 0**, verified
at completion: 13 registry tests, 40 command contracts, six trace tests, 215
frontend tests, Clippy/formatting and four package/signature tests passed.
Results: `target/work/alpha19-lookup-diagnostics-sep22/`. Do not rerun the job.
The separately labelled setup in Downloads is ready:
`Chriz Easy BG alpha19 - Lookup Diagnostics-setup.exe` (5,872,109 bytes, plus `.sig`).
Setup SHA-256: `6a260c146873f7d9dd9c25edcd60d46eb28590b1be032a90d2fe681a232cd73b`.
Embedded executable SHA-256 (read directly from NSIS):
`316d570650ce4323223cb48951701d985e5da227c5e18ac9d09faf0e7d2844da`.
The normal Downloads candidate, installed app and public feed were not replaced.

Christopher installed and opened this diagnostic version himself. Its process
trace identified the AppData redirection described above; previous unknown-cause
notes below are historical evidence, not the current diagnosis.

## Unavailable No-SR entry recurred after candidate installation — 2026-09-22

Christopher installed the corrected candidate himself. The sole running app
(`chriz-bg-app.exe`, PID 63628 at the time of inspection) started at 14:55:25;
its executable hash matches the executable embedded in the corrected Downloads
setup: `07fbd0b8613861cc7b151af187e9472cf9b64c3d6769f75a30f85b20e09dd036`.
His 14:56 screenshot still shows No-SR as an unavailable **incomplete** campaign.
Do not ask him to reinstall the game or claim the recurrence is fixed.

Read-only checks found the No-SR root, launcher and completed receipt intact;
receipt SHA-256 still matches its immutable registry record
(`a06b6e0928ee0dc1463247252e4d118c21234e6bdba702ad18a3dd2061b35379`).
Both debug/release engine checks see five completed records and 13 campaign
pointers. A helper linked to the current release **app** library, using the
installed resource directory, projects 13 installations and No-SR as
`Ready to play`; its Update Center also recognizes the completed record.
The running app belongs to Christopher and its process environment has the
expected `LOCALAPPDATA`; its working directory is the installed app folder.

The incomplete display name comes from campaign projection, not from a completed
record merely becoming unavailable. Completed IDs normally suppress their old
campaign entries. No alternate frontend fixture/persisted-record source was found.
The discrepancy between the open window and these checks remains unexplained.
Christopher confirms **Check for updates has already been retried and does not
help**. Do not repeat refresh/restart/reinstall instructions as a substitute for
diagnosis. The test originally failed, but its `supervised_recovery` receipt
(`tempus-v032-20260907`) records completion after the tail repair on September 7;
current WeiDU.log reaches the final BuffBot 1.8.3 components 1 and 0. Its historical
failed campaign pointer is not evidence that the repaired game remains incomplete.
Do not launch/close/control his apps while he is working at the machine.

Diagnostics are local under `target/work/inspect_managed_registry*`,
`inspect_packaged_bridge*`, and `inspect-cebg-process-context.ps1`. The latter
reads only the app's working directory and LOCALAPPDATA; never dump its complete
environment. No game, save, receipt, registry record, or public release changed.
Actual BuffBot Apply/Undo and gameplay acceptance remain pending.

## Overnight update check and source-fingerprint fix — 2026-09-22

Christopher explicitly changed the earlier clicking restriction: "Just try it
yourself now, I am going to bed anyway." Automation is scoped to the No-SR test
below, not the stream installation. Windows UI action-time confirmations still
apply to installing software. No actual update/Undo/gameplay test has occurred.

Reopened the existing alpha.19 app. It now shows 13 installation records and
correctly recognizes `ChrizEasyBG-No-SR-Test`; the prior unavailable status cleared
on restart, but its cause is not established. The collection comparison works.
BuffBot preview failed before mutation: both hardcoded source-tree digests were
wrong for the exact hash-verified published ZIPs. Re-downloading was not a remedy.

Corrected both fingerprints in `engine/src/mod_upgrades/buffbot.rs` without
loosening the file inventory or byte checks. Added an explicit real-archive test:
`published_archives_match_reviewed_sources` (requires the two local archive paths;
no network/game execution). It failed before the fix; all **24 update-module tests**,
including that opt-in test, now pass. Changed and extra source files remain rejected.
All 41 installed source files and 31 copied runtime files on No-SR match 1.8.3;
the loader and backup metadata checks also found no drift in read-only inspection.

Read-only preview failures now have distinct wording: no update was applied and
existing recovery warnings still apply. Apply/Restore errors retain their recovery
guidance. **Six native bridge tests** pass. These fixes are not yet in the installed
alpha.19 app or the September 22 first candidate. No game, save or receipt changed.

The corrected engine's complete read-only preview on No-SR now returns
**available** (1.8.3 → 1.8.4), with 11,898,950,716 game-backup bytes,
1,456,735 profile-backup bytes and sufficient free space. This is eligibility
evidence, not a completed update. Replacement local build runs the real-archive
test before packaging; results belong in
`target/work/alpha19-preview-fix-sep22/`. Do not rerun/poll its watcher.

Replacement job `f78462ce-7d67-4d7d-8fb3-20dc1d3aef79` is now **verified complete,
exit 0**: 24 real-package/update tests, six native bridge tests, 211 UI tests,
Clippy/formatting and four package/signature checks passed. Corrected setup and
signature replaced the old candidate in Downloads after hash checks; see
[current hashes](package-acceptance-alpha19-2026-09-22.md#corrected-candidate--current-download).
The installed app is unchanged. Next step is action-time user confirmation to
install this corrected candidate; the computer-use skill requires it even after
general test authorization. No silent-install workaround, Apply or game launch.

## Old app cleanup and user-driven acceptance — 2026-09-22

Historical restriction, superseded by the overnight authorization above:
Christopher will do **all clicking** for the supervised update test. Do not launch
or control apps/games or apply updates on his behalf. The current running app was
verified as alpha.19 at `%LOCALAPPDATA%\Chriz Easy BG\chriz-bg-app.exe`; actual
BuffBot update/Undo and gameplay acceptance remain pending.

At his request, recycled 34 obsolete files (old downloaded/build installers,
signatures, alpha.1 app binaries and its Start Menu shortcut) and the exact old
alpha.8 smoke app directory. Removed its stale Windows uninstall entry after
exporting a `.reg` backup. Current alpha.19 app/setup/signature and all managed
install/campaign record hashes were unchanged; no game/profile/save was touched.
Evidence: `target/work/old-app-cleanup-20260922/` (local, not for publication).

**Naming migration remains open:** `%LOCALAPPDATA%\Chriz BG Collection` still holds
live registries and locks used by both app and engine. Do not delete/rename it or
run an old uninstaller. Move that state to the CEBG-named data location through a
backed-up, tested migration in a new build, with the app closed. Preserve install
identities, recovery histories and compatibility with existing records. Do not
blindly rename the stable updater identifier, game ownership markers or repo.

## Current-source alpha.19 test package ready — 2026-09-22

The fresh signed NSIS build succeeded, with 211 frontend tests and four package
checks including its exact updater signature. It includes the BuffBot update route
and interval-save fix below. See [package path/hashes and acceptance limits](package-acceptance-alpha19-2026-09-22.md).
No setup/app/game was launched and public alpha.18 remains unchanged.

Proposed supervised target: the already registered
`C:\Users\chris\Games\ChrizEasyBG-No-SR-Test`, whose completed receipt matches the
registry and whose final two WeiDU rows are BuffBot 1.8.3 components 1 then 0.
Remaining eligibility is untested. Christopher approved continuing but will drive
the app himself; guide his checks and inspect results without clicking for him.

## Interval saves repaired; installer regression covered — 2026-09-22

The current game had CDTweaks 3354 (15 minutes, four rotating slots) installed,
but its isolated profile lacked the required Script settings. EET temporarily
changes `engine_name`, so CDTweaks wrote those settings to the shared EET profile;
CEBG restored its private profile name without initializing the settings there.

With no game process running, added only `IntervalSaveEnabled=1` and
`IntervalSaveCombat=0` to
`C:\Users\chris\OneDrive\Documents\Chriz Easy BG - ec2c39f8d9c3\Baldur.lua`.
Original backup: `C:\Users\chris\Games\CEBG-Backups\Baldur-before-interval-save-fix-20260922.lua`.
No saves, mod resources, WeiDU.log or dialog.tlk changed. The settings-menu mod was
**not** added to this existing game. No game launch or timed-save test was performed.

Installer finalization now initializes missing interval-save flags in its reserved
profile, only when WeiDU.log confirms CDTweaks 3350–3357. It preserves explicit
preferences (including disabled saves), and final verification checks the settings.
Automated tests cover EET profile switching, all supported presets, exclusions,
existing preferences, repeat finalization, unsafe files and missing settings.

Christopher approved HGO 38's controls as default for future installations, dependent
on any supported interval-save choice rather than only 3354. Curated defaults are
now **449 components / 50 runs**. This is in the unpublished collection alpha.17
and the September 22 app alpha.19 local package; public alpha.18 does not include
it. Interactive acceptance and publication still wait for Christopher.

## Priority correction: real existing-install updates first — 2026-09-20

Christopher expected collection updates to upgrade the existing installed mod set,
not always create a fresh installation. The current `buildUpdatedCopy` flow and the
narrow description-link hotpatch are not that general capability. Make
[existing-install updates](plans/2026-09-20-existing-install-upgrades-priority.md)
the highest next-feature priority; distinguish app updates, targeted game patches
and collection upgrades clearly. Preserve the user's selected setup and truthful
partial-update history. Do not claim a general upgrade path is implemented.

[UI providers and manual checkpoints](plans/2026-09-20-ui-choices-and-manual-checkpoints.md)
are now parked, with design/source findings and the latest choices preserved.
No further research/implementation of those features now. Live testing of the
approved UI candidates may be left to users; Dragonspear UI++ waits for a request.
Parallel patch-delivery work has advanced beyond some historical entries below;
refresh its current owner/source status before redispatching old blockers.

## First whole-mod update implemented in source — 2026-09-21

The next slice is now implemented: Updates can offer **BuffBot 1.8.3-alpha →
1.8.4-alpha in the same game folder**, with affected/full/optional-save backups,
explicit confirmation, separate version history, Undo and interrupted Restore.
See [scope, checks and remaining acceptance](existing-install-upgrades-2026-09-21.md).
Original receipts and saves stay unchanged. Other whole-mod upgrades are not yet
supported; do not advertise this as Update All or a complete collection upgrade.

Automated fixture/frontend/native checks were run, and both release archives'
hashes/tree fingerprints were verified. **No real WeiDU upgrade, interactive
app/game test or publication was performed for this new route.**
The September 22 local package now includes it; Christopher must be present for
its acceptance test. Public alpha.18 remains untouched.

## Existing-install comparison groundwork — 2026-09-21

The parallel preparation is now partly implemented: Updates can explicitly compare
a registered installation with the bundled collection using its recorded choices,
including exclusions and settings. New optional content stays off; missing choices,
new requirements, changed mod lists and unknown history are reported rather than
silently accepted. This is **read-only groundwork, not a whole-mod updater**.
No Apply action or save-compatibility claim is attached to the comparison.

See [implementation and next steps](existing-install-comparison-2026-09-21.md).
The supported BuffBot transition and durable update/restore history are now in
source as described above. Do not publish the comparison alone as completion of
existing-install upgrades. UI providers and manual checkpoints remain parked.

**Interactive testing waits for Christopher.** Do not launch the app/game, apply
an update, or repeat the earlier disposable-fixture acceptance without his renewed
go-ahead. This turn changed repository source and ran automated fixture tests only.

## alpha.19 local candidate, not published — updated 2026-09-22

Source is at app **0.1.0-alpha.19**, collection **0.1.0-alpha.17** (record corrections
plus the approved interval-save controls default above; mod versions and order
are unchanged). The patch catalog's
`minimum_app_version` 0.1.0-alpha.18 still holds. [Release notes](release-notes-alpha19.md)
are prepared. The September 22 signed local Windows candidate now includes the
latest update-route and interval-save work; it supersedes the earlier September 20
local candidate. Packaged interactive acceptance and publication remain pending.
The earlier missing-key report was incorrect: the existing updater key is present
at the documented developer signing path, and its public companion matches the
app's configured key. No key was created or rotated. Christopher subsequently
stopped unattended interactive testing; the earlier instruction to launch a
disposable Updates fixture is superseded. Arrange acceptance with him before
continuing that step. Keep the published alpha.18 assets/channel unchanged until
publication is authorized.

Issues [#6](https://github.com/Chrizhermann/chriz-bg-collection/issues/6),
[#7](https://github.com/Chrizhermann/chriz-bg-collection/issues/7) and
[#8](https://github.com/Chrizhermann/chriz-bg-collection/issues/8) now have local
fixes: transcribed existing curation approvals, corrected update coverage and
duplicate history, and removed stale fallback metadata. The combined-playtest
tool retains published sources and rejects incompatible explicit local overrides.
No new curation decision or work from Christopher was needed. See the alpha.19
acceptance record for final check/build results when available; do not infer full
gameplay acceptance from these source checks.

## Updates patch flow implemented; packaging still pending — 2026-09-20

Christopher accepted the description-link pilot and loaded-save behavior, then
approved integrating it into Updates. The [production implementation record](existing-install-patches-acceptance-2026-09-20.md)
now covers per-install checks, signed catalog, exact pinned downloads, independent
backup choices, Apply/Undo/Restore, launch blocking during recovery, patch-aware
consistency and diagnostic export. This remains a **narrow description-link fix**,
not newer Assassin mechanics or a whole-mod upgrade. No RC/stream writes or full
installation took place in this implementation turn. Public alpha.18 is unchanged.

Verification: 24 engine patch tests plus a successful real-download/WeiDU
synthetic apply/repeat/Undo test; 15 native unit and 15 diagnostics tests;
179 frontend tests, typecheck and web build. Twenty isolated mock renders across
five viewport sizes had no horizontal overflow or page errors. Native packaged
Updates interaction and public publication are not yet claimed.

**Packaging blocker resolved — 2026-09-20.** The bundled selectable `manifest/`
public-alpha profile had six selected runs without accepted static evidence and
three without tail approvals, while the curated profile carried both. Root cause:
cb07eca added the runs to `manifest/` (the profile *and* the generator base) and
4c9199b recorded evidence only in the generated curated output, so alpha.16, .17
and .18 all shipped a fallback profile that failed validation. Fixed without
weakening anything: a new fallback-profile test covers those runs (303e629), six
accepted static-test records cite it, and Christopher extended his 2026-09-16
curated tail approval to public-alpha (c8f1593). Both profiles now validate with
no findings, the curated generator reconciles by identity and still reproduces the
committed recipe byte for byte, and the 40 command-contract tests pass.

See [alpha.19 status](#alpha19-local-candidate-not-published--updated-2026-09-21) above for the
current release state.

The remaining work is now indexed in [Next work](plans/2026-09-20-next-work-index.md):
more targeted patches, My installs/unique names, customization, randomiser ideas,
challenge mode, companion changes, poison XP, regeneration on rest, website vanilla
differences and deferred community reports. The index distinguishes delivered work
from ideas, and flags ownership/context gaps. Choose the next bounded batch with
Christopher after finishing this delivery; do not redispatch every old plan.

## Existing-install patch pilot — 2026-09-20

Christopher approved the pragmatic three-category design, full-backup recommendation
and no automatic save editing. The requested Opus 5 xhigh design review completed.
The [first pilot](hotpatch-pilot-acceptance-2026-09-20.md) applied the owner-maintained
Artisan description-link adapter to the named old RC, verified six intended table
cell changes/no TLK change, detected repeat application without invoking WeiDU, and
rolled back to the original log/resource hashes. Full game and separate profile
backups remain under the user's Games/CEBG-Backups directory. Twenty focused tests
pass. On Christopher's subsequent request, the same patch was **reapplied in a
second transaction** (`kit-links-native-test-20260920`), and Christopher accepted
its native descriptions and loaded-save behavior. The stream game remains untouched. A separate, backed-up engine-name
switch selects an isolated profile with 786 hash-verified copied files; see the
acceptance record for the launcher, test checklist and restore-first undo caveat.

The subsequent production Updates integration is recorded above; do not repeat
the accepted RC test. The supervised pilot is not itself a public Apply/Undo
release, and no public version/feed was changed. The agent did not launch the game
or edit original saves. See the [accepted plan](plans/2026-09-20-existing-install-patch-pilot.md)
for exact scope and guardrails; do not restart a full installation for this work.

## Alpha.18 published with reviewed Modpack fixes — 2026-09-20

Christopher approved the completed compatibility release. The
[alpha.18 source candidate](alpha18-candidate-2026-09-20.md) now pins Modpack
v0.2.0-alpha.7, with app alpha.18 / collection alpha.16 identities. Recommended
selection is unchanged at 448 components / 50 runs; the actual download/extraction
and focused recipe checks pass. The user's preceding alpha.17 installation has a
successful 50-run receipt; no repeat full install or live-game writes were made.
The background WeiDU fix is included in the published alpha.18 package. Its
[release and verification record](package-acceptance-alpha18-2026-09-20.md) covers
the signed download, tests, public assets and update feed. The alpha.18 setup is
also in Downloads; the website owner has the verified publication handoff.
Use [the release notes](release-notes-alpha18.md) for the included changes.
The wiki and newly proposed Baeloth work remain deferred, not release blockers.

## Next-release triage: Randomiser feedback — 2026-09-18

[Queued report and feature ideas](plans/2026-09-18-randomiser-next-release-triage.md):
unconfirmed Mode 2 / BG-style proficiency conflict; editable RR/EE item pools;
expanded cursed items; per-new-game randomisation; optional combat-focused reward
placement. Christopher explicitly requested recording only, no investigation now.
Discuss scope before promising the larger features for the next release.

## Queued UX follow-up: My installs and unique names — 2026-09-16

Christopher finds alpha.17's My installs view cluttered and its duplicate generic
incomplete-install labels ambiguous. The [cleanup roadmap follow-up](plans/2026-09-07-managed-install-cleanup.md#follow-up-roadmap-my-installs-clarity-and-unique-names--september-16)
records compact layout, persistent unique display names, automatic numbered
defaults, legacy-entry disambiguation and clear post-removal selection. Planned
only; no new app build, game changes or delay to his full-install test.

## Alpha.17: SoD fix, disclosure polish and explicit cleanup — 2026-09-16

The [alpha.17 candidate record](alpha17-candidate-2026-09-16.md) tracks app alpha.17 /
collection alpha.15 with SoD Remix v0.6.11, the previously unbundled download retries,
technical-log/scroll fixes, collapsible framed Customize categories and confirmed
managed-install deletion. Defaults remain 448 components / 50 runs. Static, fixture
and bounded headless UI checks pass; consult that record for package status.

Christopher has **chosen a fresh test install**, superseding the proposed supervised
197 recovery. Do not repair or delete his failed alpha.16 installation automatically.
He will test the full install and deletion later. No new live installation/deletion
test, stream-game modification, public release or updater-channel change is implied.
Automatic repair of sealed partial batches remains a separate feature.

## Incremental download retries and viewer follow-up — 2026-09-16

[Retry implementation](download-retry-acceptance-2026-09-16.md): five attempts with
0/2/10/30-second retry waits, ordinary technical-log notices, unchanged integrity
and resume protections. 47 focused tests pass. Source only; the earlier alpha.16
setup must be rebuilt before this can be advertised or published as included.
Viewer diagnostics 2 confirms Xan recovered and all 34 archives are ready, but
BG1 staging still fails at the Documents save child. The [Documents handoff](handoffs/2026-09-16-documents-folder-robustness.md)
records the evidence and remaining security-history question; root cause not proven.

## Deferred until tomorrow: player poison kill XP — reported 2026-09-16

Christopher reports that enemies killed by poison originating from the player
appear not to award kill experience. **Unconfirmed; do not investigate or fix now.**

Next-session plan: identify the exact poison source and affected installation,
compare a poison finishing tick with a direct-damage kill, and check kill/XP
attribution. Route the confirmed cause to the owning mod repo (owner unknown yet),
then make a focused fix and regression check. Do not assume all poison sources or
the base engine are affected. No game/save changes or agent dispatch requested now.

## Alpha.16 candidate assembled — 2026-09-16

The [candidate record](alpha16-candidate-2026-09-16.md) supersedes the preparation
states below. All owner releases are published and pinned, including modpack alpha.6
with Yeslick188/continuity199/Safana189/Imoen620. Recommended selection is 448
components / 50 runs; dragons remain opt-in. Public-alpha validation passes.
Christopher performs the installation/playtest; do not start another agent-run game
installation. See that record for package verification and publication state.

## Full candidate release first; web docs staged — 2026-09-16

Christopher's current decision is to include every **implemented** candidate in
the next new-install release, subject only to exact source/artifact, selection and
order validity before one full packaged-candidate install. Artisan and Bardic have
the required user acceptance and are releasing through their owners now. SoD Khalid
115 is released in v0.6.10 with bounded entry/briefing acceptance. SR/RR is included;
Lightning 80 is default and 81 is the mutually exclusive optional alternative.
Official Safana plus core 189 are included for Christopher's candidate test; unfinished
Bard/Abettor conversion is not. Yeslick 188 is implemented but awaits its owner
integration before 199/EET_end; it has no existing-save migration. Dragons 110/111
remain opt-in Challenge components.

**Latest wrap-up instruction:** Christopher will run the full installation and
playtest himself. Finish exact package intake, necessary recipe/dependency checks
and the installer build; do not start another automated full install, gameplay
round or broad review. Hotpatch implementation follows the new-install release.

Do not claim the provisional alpha.16/alpha.14 labels or historical 436/44 count as
final, and do not claim that every source is pinned, built, shipped or integration-tested.
The separate [full-candidate and web-docs handoff](handoffs/2026-09-16-full-candidate-and-web-docs.md)
has the exact release-first sequence. The web task may prepare/publish accurate docs
with upcoming changes clearly labelled, but the current download stays in place until
the new release is verified. Existing-game changes come later as opt-in guarded
patches—not generic override copying—and no game write/hotpatch work is authorized now.

## Documents-folder handling requirement — 2026-09-16

Christopher expects moved/OneDrive/missing Documents to be handled automatically
where Windows permits. A viewer's `stage:bg1` / `os error 2` points to the CEBG save
profile, not evidence that C:/D: separation is unsupported. Version/diagnostics are
pending; cause is unconfirmed. Current code resolves redirected Documents and
creates its own child but assumes the parent exists and returns generic I/O errors.
The [bounded hardening handoff](handoffs/2026-09-16-documents-folder-robustness.md)
records creation-at-install, clearer diagnostics, retry and ownership boundaries.
No fix/reproduction/release yet; do not advise deleting caches or disabling OneDrive.

## Yeslick kit gap and Dispel diagnosis — 2026-09-15

Christopher requests both for the next patch. The [owning-mod handoff](handoffs/2026-09-15-yeslick-bg1-kit-and-dispel.md)
records read-only source/save evidence: YeslickNPC 1 kits BG2 Yeslick only;
the current BG1 saved actor is an unkitted Fighter/Cleric. Add BG1 Alaghor coverage
in modpack before continuity 199/EET_end, respecting vanilla opt-out. Do not make
continuity reset player builds. Existing saved actors need a separate repair.

Dispel 410 is already in public modpack and the draft, but absent from the older
stream install. Its SPIN112 still affects allies and bypasses the dispel-level
contest; Combined's spell has the intended hostile-only 1.5x scaling at all 40
headers. No game/save changes or native casting test in this pass.

The extra final SoD bridge/Mislead 257/266 check is **waived as a release gate**
by Christopher, not recorded as passed. TLK changes remain high-risk and require
target-specific validation. Release and existing-install triage plans agree.

## Existing-install patch triage — 2026-09-14

[Patch candidates and remaining tests](plans/2026-09-14-existing-install-update-triage.md)
now complement the release plan. Separate component/resource compatibility from
saved-state applicability; never infer either from WeiDU.log alone. Dedicated
description, Lightning and Abettor tails exist, while full Artisan/Bardic saved-grant
migration does not. Modpack 199 remains new-campaign/pre-EET_end work; 189/620 and
SoD 115 have specific event boundaries. Local TLK appends are possible; copying
another install's TLK/compiled string numbers is not the proposed approach.
Source/planning checks only: no game changes or generic patch executor implemented.

## Expanded next-release plan — 2026-09-13

Christopher wants the next release within the next couple of days, including
implemented mod work whose live tests or owner releases are still pending. The
[next-release scope and acceptance plan](plans/2026-09-13-next-release-scope-and-acceptance.md)
is the current planning entry point. It inventories Artisan kits/modals/descriptions,
Bardic, both SoD work lines (bridge/filler plus Khalid 115), dragons, modpack
continuity/Imoen/Safana, the retained SR Lightning implementation, BuffBot 1.8.4 and
the current CEBG draft. Planned-only work is separated for discussion.

This expands the older narrow release proposal, **not the recipe yet**. Current
draft 436/44 and historical Combined 446/50 are not the final expanded count.
Reuse Combined for applicable native tests; consolidate sources and do one final
frozen packaged-candidate install, not a rebuild for every feature. September 16
resolved the earlier SR/RR, Safana and Lightning choices; the current top-of-file
handoff supersedes this historical discussion. No game/source-pin changes or
collection release were made by this planning pass.

## Challenge Mode recorded for later — 2026-09-13

Christopher requested future Challenge Mode components: SoD changes, stronger
dragons, wand-to-scroll replacements, selected-fight area locks and practical
enforcement of his run rules. The complete 16-rule screenshot inventory, new
requests, SoD filler exceptions, owner routing and unresolved policy details are
in [the Challenge Mode outlook](plans/2026-09-13-challenge-mode-outlook.md).
Documentation/preparation only: do not start implementation, change defaults or
delay the next patch on its account. Subsequent source inventory identifies
implemented optional SoD **257**, the Insane bridge sequencers, as an existing
challenge piece. Native acceptance of its final split is not recorded; Christopher
waived the extra check as a next-release gate on September 15. It does not mean
the full Challenge Mode rules are implemented.

## Acton Balthis no longer default — 2026-09-13

At Christopher's request, BG2 Unfinished Business **25: The Murder of Acton
Balthis** is now optional and unchecked in the draft alpha.16 / collection
alpha.14. Catalog, base manifest/preset and generated recipe agree. No other
component depends on it; all other selections and source pins remain unchanged.
The recommended plan is now **436 components / 44 runs**, retaining CDTweaks 2380.
37 focused tests pass; public-alpha validation has no findings. Opting into this
quest adds exactly `ub-bg2/25`, with no collateral selection changes.
Existing games, historical installation order and published release evidence were
not changed. This remains an unpublished next-patch change.

## Racial kit unlock default approved — 2026-09-12

Draft alpha.16 / collection alpha.14 now includes **CDTweaks 2380**, default-checked
and optional, after the main Artisan/Bardic kit additions. It fills the elf mage
specialist availability gap; internal/NPC-only kits and gnome mage policy stay
unchanged. Recommended plan: **437 components / 44 runs**. Switching it off removes
only `cdtweaks-bg2/2380` and returns 436. 36 focused tests pass; public-alpha
validation has no findings. Native WeiDU 249 fixtures using copied stream and
Combined tables pass install/uninstall with exact restoration; all 66 newly
eligible kit/race choices pass the ability-minimum checks. Completed in-engine
character creation is not yet tested. The pre-existing dwarf/Gallant CHA mismatch
is tracked separately in the community follow-up plan below. No user game/save
changes, packaging or publication.

## Optional choices and community follow-ups — 2026-09-11

Draft alpha.16 / collection alpha.14 now offers **24 additional SCS and 44
CDTweaks choices**, all unchecked. Recommended selections remain exactly unchanged:
**436 components / 44 runs**. Alternative groups have real mutual conflicts;
CDTweaks 2310/2311 remain alongside default 2312 in the late spell-scan run.
SCS 4130/4135/4140 have visible **Not recommended / experimental** cautions after
community feedback. 4130 also explains and enforces SCS's native SR exclusion.
Do not infer that source screening proves every combination works in-game.

35 focused Python tests and 12 Customize UI tests passed; public-alpha validation
has no findings; eight CLI planning cases and pinned SCS/CDTweaks native order were
checked. No game was modified, no installer test was started, and nothing was
packaged/published. Historical release acceptance records were preserved.

Christopher deferred the unresolved **Kivan/Jozzi** viewer case until he can
reproduce it; the existing component 130 guard was already shipped, not a promised
new fix for that case. Druid instant-cure availability is a future SR/BG Rebalance
balance discussion. **Automatic regeneration on rest** is a separate requested
QoL feature, researched but not implemented; owning-mod prototype scope is in
`docs/handoffs/2026-09-11-regeneration-on-rest.md`.

Next-release scope, source-screening caveats, Kivan follow-up, and the explicitly
deferred **website differences-from-vanilla comparison** are tracked in
`docs/plans/2026-09-11-optional-tweaks-and-community-followups.md`. The website list
must be discussed with Christopher/Twitch task after release, not published as an
unreviewed exhaustive claim now.

Also recorded there: Christopher's **elf Mage character-creation screenshot** shows
only Mage, Diviner, Enchanter and Wild Mage. Read-only checks of stream/combined
`K_M_E.2DA` and `mgsrcreq.2da` confirm exactly that list. Selected Artisan 1 unlocks
base classes, not the missing specialist choices. **CDTweaks 2380** is the
kit-unlock counterpart; default inclusion was subsequently approved and implemented
as recorded above. Gnome policy is separate. No game changes; not the earlier
Red Wizard/SR issue.

## Red Wizard default restored after independent review — 2026-09-10

Christopher asked another agent to investigate the Red Wizard/SR exclusion and
restore the kit as default unless a substantiated blocker was found. The review
found no basis for a blanket ban. Artisan NPC **5102** is now default-checked with
or without Spell Revisions, still optional. Other exclusions and source pins are
unchanged. Draft alpha.16 / collection alpha.14 has **436 components / 44 runs**.
This correction is not yet packaged or published.
Verification: 27 focused tests pass, public-alpha recipe validation has no findings,
and CLI plans confirm 436 components with Red Wizard / 435 without, retaining SR.

Evidence, original rule history and actionable mod-repo follow-ups are in
`docs/handoffs/2026-09-10-red-wizard-sr-default.md`. There are narrower starting-spell
and custom-kit spellbook-cleanup imperfections, plus an EET Edwin variant gap;
do not misrepresent them as a demonstrated universal SR incompatibility. Do not
silently enable the old Edwin amulet/slot patches.

Neither the stream nor Combined-20260908 has been modified: both still lack 5102.
The 446-component combined acceptance below does not cover Red Wizard. Focused
native Edwin testing and any existing-save migration remain separate work.

## Kivan quest fix default reconfirmed; already installed — 2026-09-09

Christopher requested default inclusion through the owning task **Fix Kivan Sahaguin
quest** (`01a08666-0eff-77e0-a178-bda4a232ff78`). This is already provided by the
normal hash-pinned `chriz-bg-modpack v0.2.0-alpha.5` dependency, component **130**;
do not add a duplicate standalone installer or select a stale checkout's stub.
The recipe keeps it `mandatory` when BG1NPC quest component **10** and SCS general
AI **6000** are selected, after EET import and SCS (public recipe: post-EET_end).
It is independent of Kivan's Archer choice. The dependency was tightened from
mere BG1NPC mod presence to component 10 after a regression test demonstrated that
turning quests off previously left the guard selected. The default component set
is unchanged; this authored/generated recipe correction is not yet published.

Read-only proof: Combined-20260908 `game/WeiDU.log:408` records modpack 130 once;
no standalone fix entry. Both `override/X#SAHA01.BCS` and `X#SAHA02.BCS` are 225 bytes,
SHA256 `f63e741a2b3c39eab66e34f18b3fa8830022465e2bf54c762b73267c282a45e7`, matching
the handoff's standalone verification exactly. No completed-install files, receipts,
saves, frozen recipes or source games were changed. The stale migration/stub
documentation has been corrected in `COLLECTION_TAIL_FIXES.md`.

Reference-only handoff package:
`C:\Users\chris\Documents\Codex\2026-09-09\kivan-quest-fix\KIVAN_QUEST_FIX-v1.0-verified.zip`,
verified SHA256 `7fc3769c0c18e32305c1ceefb9727b27a5331edb1165f04c786bb1c0eb035d23`.
Its `verification.json` records four passing installer cases, not new native
playtest acceptance. Do not use this local ZIP as a public dependency.
Test the quest from before the encounter: dialogue should finish before lower SCS
combat AI takes over. This prevents the interruption; it does not resurrect dead
quest actors or repair an already failed saved encounter. No new release authorized.

## Combined playtest completed and retained — 2026-09-09

**Installation accepted: 446/446 components (27 BG1 + 419 BG2).** All 391
previously completed components were preserved; no full reinstall or sibling
uninstall/replay was needed. Corrected SoD 256 and all ten remaining frozen runs
completed, including dragons, Safana 189, Imoen 620, SR 60/80, Klatu 2150 and
BuffBot 1/0. BG Radar Overlay **2.5.2.0** is also installed from official Latest.

Keep `C:\Users\chris\CEBG-Tests\Combined-20260908` for Christopher's testing.
Launch `game\InfinityLoader.exe`; Radar is
`game\BG Radar Overlay\BG Radar Overlay.exe`. The isolated save folder is
`C:\Users\chris\OneDrive\Documents\CEBG Combined Playtest 2026-09-08 - c2a832a75741`.
**Native launch/gameplay acceptance is still pending.** This is a local experimental
snapshot, not a public recipe release or a stream-install update. The source and
stream games remain untouched. No installation worker remains active.

Fresh CLI `report <root> --json` returns `ok: true`. The separately published
`.chriz/install-receipt.json` records recovery `sod-256-append-20260909`, version
`local-40712587d45b (repaired)`, exact final component order, source provenance and
attempt evidence. Receipt SHA256:
`51d670ab9fab44ca4fd6d282c4530fb352b1ebe49a6f6e2975585426ce7edcbd`.
The original failed receipt/ledger/frozen recipe were not rewritten; the original
terminal receipt SHA256 remains
`710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938`.
Keep `.chriz/recoveries/sod-256-append-20260909/` immutable: it is hash-bound evidence.
Do not rerun either installation worker or repair mode. The read-only follow-up
`cebg-overnight-install-follow-up` is now **PAUSED** after verified completion.

Focused recovery tests and example/CLI builds passed, followed by actual retained
installation, acceptance and fresh report verification. The acceptance example
uses canonical forward-slash evidence paths on Windows. Current CLI understands
the additive append-recovery receipt; public GUI compatibility with this local
recovery metadata has not been tested. The SoD owner received the successful 256
installation evidence; native bridge playtest and separate release approval remain
outstanding. Next: use the disposable checkpoints in
`docs/plans/2026-09-08-combined-playtest-and-alpha16.md`.

## Combined playtest completion authorization and execution history — 2026-09-09

Christopher explicitly requested: **"then finish the installation."** Retain
`C:\Users\chris\CEBG-Tests\Combined-20260908` and all 391 completed components.
Apply only the verified SoD 256 source fix below, append that missing component,
then execute the ten untouched frozen runs through BuffBot (446 total). Do not
rollback/replay the 34 successful SoD siblings or start a new installation.

Recovery executed in `installer-v0-real-alpha`: an incident-specific worker
and a narrowly authorized append-order recovery verifier. The original failure
receipt, ledger and recipe remain immutable; successful completion must publish
a separate recovery receipt with replacement-source and physical-order evidence.
The follow-up was active and read-only during execution; it is now paused.

**04:31 +09:00:** corrected 256 installed successfully, preserving all 364 old BG2
rows byte-for-byte (392 total including BG1). Evidence/backup is under the retained
root's `.chriz/recoveries/sod-256-append-20260909/`. `tools/recover-combined-bridge.ps1`
continues the frozen tail. A helper-only HGO executable-naming no-op was detected
with unchanged before/after logs; its evidence is retained, and the real HGO
operation uses stem `hiddengameplayoptions-bg2-attempt2`. Do not replay 256 or HGO.
The remaining runs and final recovery acceptance subsequently completed (see above).

## SoD 256 fix ready; minimal recovery rehearsal passed — 2026-09-09

The owning SoD task delivered commit `29e123a79b9f03334ab88ce93e28c300b287a8e0`
on `codex/issue-14-bridge-finale` / draft PR22. It fixes the missing review of
Artisan's MO1 equipment-policy pair, preserving the pair after validating its
delivery shape and SPLPROT semantics. It does not disable unknown-effect rejection
or require a Kitpack source change. Only runtime source changed:
`chriz-sod-remix/lib/comp256_creatures.tpa` (worktree SHA256
`6740293f96cb2d3d1c510b29984fc23106609eb3bd2f108f6099ccd9346b6387`;
committed LF blob `b730bae065bc1d47fcadd117eb5eefe1438afb02831c94a6eac38a9a8340eeca`).
The owner reports all 63 focused tests and exact-commit CI green.

Final disposable `fixed-complete` rehearsal with WeiDU249 succeeded: it preserved
all 364 original BG2 canonical log rows in order and appended only 256 as row365;
21 new assets, six intended existing-resource edits, zero removals/uninstalls.
The later installed SoD siblings have no conflicting recorded resource writes.
All 472 protected files and 129 read resources from the retained source were
verified unchanged. The source report and handoff are under the bridge worktree's
`research/data/issue14-fullstack-20260909/`; report SHA256
`b13efb10c573df8d6234b8c26f65d3d4a8d8236d670dd21def130cfd52ff09fd` was checked here.

This supports preserving **all 391 completed components** with a component-only
repair. It is not yet an accepted recovery of the retained installation: the
collection still needs narrow missing-component/source/order reconciliation and
evidence before continuing the untouched tail. Do not use blind Resume or alter
the original failed receipt. Its hash remains
`710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938` (rechecked here).
No retained game write, public release, new installation or native playtest was
performed by this handoff. The ready-fix notification has been delivered; the
check-only follow-up is paused pending the recovery decision, and must be
reactivated when a recovery worker starts.

## Combined playtest stopped; targeted recovery assessed — 2026-09-09

The 446-component local test stopped at **01:49 +09:00** in SoD Remix 256. No
installer process remains. Preserve `C:\Users\chris\CEBG-Tests\Combined-20260908`;
**do not delete it or start another full installation**. Current active rows:
27 BG1 + 364 BG2 = **391**. EET, SCS, modpack continuity and EET_end completed;
the ten later runs (including dragons, Safana/Imoen, Lightning, Klatu and BuffBot)
have not started. Public recipe and stream were not changed.

Cause: `BDOLONEI.CRE` contains Artisan MonkRevision-Mystic's opcode 326 delivery
`C0PR#MO1`; `comp256_creatures.tpa` omitted it from its reviewed donor-effect list.
The failure occurred during preflight and WeiDU reports zero component-256 files
to roll back. Other 34/35 SoD components completed. Root error is around line 10907
of the game's `SETUP-CHRIZ-SOD-REMIX.DEBUG`; retain that and the immutable terminal
receipt `.chriz/attempts/terminal-0000000234-19d855a4d1b269e8/receipt.json`.

Christopher asked whether the installation can be salvaged and to send the problem
to its owner.
The SoD task **Find next roadmap item** (`01a071d8-870a-7b02-bbde-e33d0f0b051f`)
received the exact logs, source commit, cause, focused regression request and
read-only-install boundary. It is working on the source fix; no public release or
game mutation was requested from that task.

Read-only recovery evidence: the SoD before-log matches the previous successful
run's after-hash. Its 330 BG2 active rows are retained exactly; the current log adds
only the 34 successful SoD siblings in relative authored order. The existing
supervised recovery mechanism can retain **all 27 BG1 + 330 pre-SoD BG2 components**
and redo only the SoD run plus the untouched tail. It requires reverse rollback of
those 34 SoD siblings, verification of the original prefix, a corrected 35-component
SoD run, complete tail evidence, and a separate recovery receipt. The existing
operator scripts are incident-specific, so this case still needs its own small
operator adapter; it is not a generic CLI recovery command. Ordinary Resume is
not valid, and no repair has been performed or accepted yet.

A smaller append-only 256 repair is potentially possible but not currently
supported by the recovery verifier. The owner must confirm compatibility after
the already-installed 260–910 siblings, not merely after 255; otherwise do not
invent an out-of-order success receipt or disable exact-order verification. The
owner was asked to clarify its 'never uninstall 255' note for both approaches.

Hourly check-only follow-up `cebg-overnight-install-follow-up` is now ACTIVE for
this fix/test (the old no-SR one-time follow-up had been paused). Stay quiet on this
already-reported failure; notify a ready fix, new failure, required decision or
verified completion. Pause when awaiting user direction or completed; reactivate
the same follow-up whenever an authorized recovery worker is launched.

## Combined playtest launched — 2026-09-08

Christopher confirmed modpack completion and asked us to continue. Bridge/filler
source is ready at `3b34eaee19dcb9043f3b72c77bf5b92adbed05ca`; clean modpack
integration is `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`. Preserve Sarah 190 and
existing companions; use Safana **189** (191 is reserved), continuity 199 and Imoen
620. The bridge hold and modpack source-integration blocker below are resolved.

The separate ignored `target/combined-playtest-20260908/recipe` is assembled from
seven frozen local sources. It preserves all 435 current recommended components
and adds exactly 11 experimental components, with no duplicates: **446 / 50 runs**.
Authoring validation has zero findings. Focused source/recipe tests pass; generated
archives are deterministic. Public recipe/feed and stream remain unchanged.

One hidden CLI installation started at **2026-09-08 22:54:18 +09:00**, PID **60364**,
managed root `C:\Users\chris\CEBG-Tests\Combined-20260908`, preset
`chris-recommended`, unique name/save profile `CEBG Combined Playtest 2026-09-08`.
It reuses `C:\CEBG-creator-full-cache`; seven local ZIPs and Evandra's existing
official EXE use the normal verified manual intake. CDTweaks/IWDification archives
were reused from the app cache. Initial source checks passed; the engine created
its managed ledger/frozen recipe and is acquiring/verifying the selected artifacts
at this checkpoint. Completion/native playtest are not yet claimed. **Do not start
another copy or edit this recipe during the run.**

Worker and full stdout/stderr are retained under ignored
`target/combined-playtest-20260908/run/`. Frozen worker SHA256:
`1e251c6af996bd64d039fa0a25ce65b02baae87353629e7756c9f70e7b937318`.
The first launch guard stopped before creating a game because the shared debug
binary had been rebuilt during recipe checks; the final copied binary was
revalidated and then launched. This was not a game-install restart.

Safana core v0.5 is added only as prerequisite for 189, addressing the inventory
gate documented in her curation. No Safana Bard/Abettor preset. Continuity 199
follows companion conversions immediately before EET_end; Safana/189/620 stay
late. SR/RR compatibility is after both core mods and before SCS; SR60's scans
target joinable NPCs, not those hostile RR actors. Lightning80 is after SR60 and
the other spell modifiers, then Klatu and BuffBot last. See the
[updated plan](plans/2026-09-08-combined-playtest-and-alpha16.md) for playtest saves.

## Combined test on hold for SoD bridge — 2026-09-08

Christopher wants to wait for the SoD bridge implementation (issue 14), then test
it together with the other SoD changes in one combined installation. Do not start
that installation until the bridge has a testable source snapshot. Refresh its
commit/component requirements before assembly; this is not a requirement to
publish the mod first. The prepared alpha.16 source remains saved, with no change
to public releases or the stream installation. See the
[combined playtest plan](plans/2026-09-08-combined-playtest-and-alpha16.md).

## Alpha.16 source prepared; combined playtest mapped — 2026-09-08

Proposed release is app alpha.16 / collection alpha.14: existing startup loading
feedback, default-optional Klatu armor QoL, and released Bardic `v2.9c-balance.4`.
435 recommended components / 44 runs; no other curation/pin changes. App version
markers and notices agree. The Bardic ZIP was fetched from its official release
and independently checked (5,177,696 bytes, SHA256
`ca7bb2b70ad50b5b6c0fa59a051e53b90c42d3cc9f98fd187a5c1ed40fc1efa7`).
Draft [release notes](release-notes-alpha16.md) are ready. Public remains app
alpha.15 / collection alpha.13: no new setup package, upload or feed update yet.

Checks: frontend typecheck + 146 tests + production build; 30 Python recipe/credits/
updater tests; 16 native curated/release tests, 8 update-ledger tests and 3 package
contract tests. Rust formatting passes. The package signature test remains unrun
until a new signed setup exists. No fresh game install or new live acceptance.
The draft ledger now covers the actual artifact/run/feature update addresses;
the focused regression reproduced the missing coverage before correction.

The [combined-playtest plan](plans/2026-09-08-combined-playtest-and-alpha16.md) shows
how most implemented gated work can share one fresh SR-on EET install with several
disposable saves. Do not bulldoze a stream clone. Important assembly fix: the old
modpack continuity snapshot reuses Sarah's published component 190 and lacks the
newer 192–198 conversions. Port it onto current modpack, preserve those IDs and
allocate a new continuity ID (199 proposed), integrating 620 deliberately. Split
its required pre-EET_end work from the existing late repairs. Keep SCS and SoD in
their current curated phases. SoD PR21 filler changes have code; bridge issue 14
does not. Mutually exclusive Lightning variants and no-SR Darkbloom are not all
simultaneously testable. No mod repo, stream files or saves changed here.

## Armor-thieving QoL — 2026-09-08 (integrated, not published)

Approved: default-checked optional thief skills in armor, **no added penalties**.
CDTweaks 2100 stays excluded. Klatu 2150 is integrated in draft collection alpha.14,
after armor/kit changes and immediately before BuffBot (still last): 435 recommended
components across 44 runs. No other Klatu component or unrelated mod update was added.
The real WeiDU synthetic install/uninstall fixture passed, as did 28 Python tests,
16 native curation/release tests and Rust formatting. No full reinstall was needed.
Exact scope and limits are [recorded here](plans/2026-09-08-armor-thieving-qol.md).
Published app remains alpha.15 / collection alpha.13; alpha.14 release metadata is
draft, including its provisional timestamp. No live game or save was modified.

The requested quick [mod-repo and patch-candidate audit](plans/2026-09-08-mod-repo-readiness.md)
is complete. Bardic balance.4 is now pinned in draft alpha.14; SR/RR's released tail
patch remains subject to the earlier collection deferral. Artisan's merged work is
not release-approved. Armor QoL and the separate kit-description repair are the
first existing-game patch candidates, not already applied patches.

## Visible startup while detecting games — 2026-09-08 (source only)

The blank window during slow first-launch checks is fixed in source: early HTML
fallback, staged loading text/spinner and retryable startup errors. The recommended
recipe and installed games are unchanged. Frontend checks and a delayed browser
preview pass; include this in the next app patch, not a collection recipe update.
Published app remains alpha.15 / collection alpha.13. See
[cause and acceptance](issues/startup-black-window-2026-09-08.md).

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

# September 22 release readiness

**Current status, September 29:** refreshed app alpha.19 / collection alpha.18 is
built and verified. One combined fresh installation and the short manual check
are next. No setup/game was launched and no CEBG release/feed/website was published.
See [current package evidence](../package-acceptance-alpha19-2026-09-29.md).
The dated earlier milestones below are historical where superseded by this status.

This is a working checklist, not a publication announcement. App **alpha.19** and
collection **alpha.17** are the current local candidate versions. The normal local
package built successfully on September 23 (KST); it has not been installed,
launched or published. Christopher's finishing UI review is next.

September 27: those opening versions describe the existing binary, not current
source. Recipe source is alpha.18 and two further small source additions are now
implemented. Read the [fresh inventory](2026-09-27-pre-package-inventory.md) before
the next package: four public mod pins are behind, Modpack 630/Artisan 51010 still
need owner releases and optional integration, and UI providers are not yet wired
into the production recipe. No new package or publication has occurred.

September 29 scope update: **Dragonspear UI++ joins the approved optional UI
providers for the next release**, for new installations only. Wire its pinned
components into the recipe, the exclusive interface choice and any source-proven
conflict rules before packaging; leave the recommended interface unchanged.
Basic selection/install-contract checks remain required; general live UI testing
may be left to users as agreed. Do not claim it already ships. Praetor's manual
installation/finalization report is captured in the
[UI and checkpoint plan](2026-09-20-ui-choices-and-manual-checkpoints.md#september-29-report-manual-ui-addition-rejected-at-finalization);
manual breakpoints are also explicitly in the next-release scope, although their
implementation is distinct from official provider integration.

### September 29: bounded delivery and community testing

Active delivery: Modpack **v0.2.0-alpha.8** is published and verified, with
224/225/611/630 and the bounded legacy-UI string-helper adapter 640. Its final
recipe integration is complete. BG Rebalance
302 is published as v0.7.0 (verified archive; 382 automated cases with 26 existing
fixture skips, exit 0), and Artisan's newly named release is also published.
Both are integrated into the recipe. The real UI providers and advanced manual
breakpoints have focused coverage; all six optional interface choices are now
selectable, with classic providers including the Modpack adapter automatically.
Recommended curated count is 453 components / 50 runs. The original combined job
stopped before packaging on stale UI test expectations. Both affected files were
corrected and their 11 tests pass; no checks/features were weakened. Continuation
job `a820c812-eac4-479e-bdad-68f1fb19645c` passed, including the full workspace and
frontend checks, package build/signature tests and all 221 bundled-resource
comparisons. The verified dated Downloads copy is ready for the user. No fresh
game install or desktop operation has started.

Use the [short combined smoke check](../next-release-smoke-check.md) for the next
package. It supersedes treating the detailed per-component checklists below as
mandatory manual gates: one fresh install, a short targeted check, and community
coverage for the wider matrix. Focused automated safety checks remain required.

- [x] Keep **Emotion Courage/Hope (BG Rebalance 301)** selected by default in
  the next recipe. Integrated in alpha.18 and the September 29 package.
  Christopher reconfirmed inclusion; no additional manual testing gate.
- [x] Bounded upstream reviews completed September 29: Artisan, Bardic,
  Randomiser and SR have no incoming upstream changes to merge.
- [x] Integrate the explicitly approved Artisan `chriz-v1.6.0` and Bardic
  `v2.9c-balance.6` fresh-install pins, retaining existing selections. Other
  published refreshes remain tracked in the pre-package inventory. Both archives
  pass engine verification, 27 Python/24 Rust checks pass, both profiles validate,
  and all 101 generated recipe files reproduce byte-for-byte. Artisan's later
  upstream-based version below supersedes this intermediate pin; both current
  owner packages are included in the September 29 installer.
- [x] Integrate SoD Remix **v0.6.12** and BuffBot **v1.9.0 stable** for fresh
  installs. Actual artifacts verified; follow-up 27 Python/19 Rust checks pass,
  both profiles validate, all 102 generated files reproduce exactly. No selected
  IDs changed. No SoD 258 retrofit or extension of the existing BuffBot updater.
- [x] Publish and integrate new Modpack **611**, story-based utility XP:
  BG1 rates through SoD; BG2 rates from SoA onward. Make it the default, with
  existing 610 unchanged as an exclusive optional smooth-curve alternative.
  Party-average scaling is deferred. Source implemented in `companion-spellbooks`;
  376 focused offline checks pass. Christopher approved EE Fixpack's **155-XP**
  correction for BG1/SoD level-16+ locks. Reconcile pending Modpack
  branches before publication. Published alpha.8 and the exclusive choice are now
  verified/integrated, alongside 224/225 defaults, optional 630 and UI helper 640.
- [x] Publish and integrate BG Rebalance **302**, the Flail of the Ages change
  as default: +5 Haste compatibility and tiered breath saves against Slow.
  Published v0.7.0 with verified download identity; 13 real-WeiDU disposable
  tests and the full owner suite pass. The current source recipe includes it;
  fresh-install/live-game acceptance still follows the short combined test.
  Keep broader Free Action and Dispelling Screen redesign outside this release.
- [x] Migrate Artisan's next owner release to upstream-version-plus-our-revision
  naming; legacy `chriz-v1.6.0` remains immutable. Verify the upstream release or
  explicitly identified snapshot, all three VERSION fields and the new CEBG pin.
  Published/integrated `v4.81a-dev.85928f9-chriz.1`, including optional 51010.
  See [versioning decision](../fork-versioning.md).
- [x] Implement the advanced, off-by-default manual breakpoint flow from the
  agreed plan: multiple completed-component boundaries, persisted acceptance of
  added mods, successful continuation/finalization without deleting WeiDU rows,
  and game hotpatches disabled after manual additions.
- [x] Run focused automated and basic package/safety checks for the new UI choices
  and checkpoint flow. Preserve safeguards and correct known deterministic defects.
- Broader compatibility and gameplay testing can be performed by viewers/users.
  Do not require exhaustive agent-led full-install matrices or make Christopher
  test every option personally before release. State what was actually checked.

Additional September 29 boundaries: unidentified-item visuals reuse existing game
assets, not newly created art. Instant trap detection is deferred beyond this
release. Story-based utility XP and the bounded Flail change are now approved
above. Artisan Fighter proficiency grants remain a discussion, not an approved
nerf. No new blanket gameplay-testing gate was added.

September 26 source closeout: the initial Updates selection now shows patch/Radar
progress and results correctly. Four new regression cases pass; the full frontend
has 327 passing tests, and the native workspace has 803 passing tests with 13
existing opt-in tests ignored. Python remains 102 tests with one opt-in skip;
both recipe profiles pass public-alpha validation. Formatting and Clippy pass.
The September 23 binary predates this correction; a new package is needed before
calling the corrected source packaged. Christopher requested the source commit
first, followed by a separate recipe update for approved BG Rebalance 301.

## Accepted: do not repeat without a new reason

- The existing Artisan description-only patch has its own
  [accepted pilot](../hotpatch-pilot-acceptance-2026-09-20.md). It does not add kit
  mechanics or edit saved characters.
- BuffBot **1.8.3-alpha → 1.8.4-alpha** passed user Apply, existing-save gameplay and
  Undo. The No-SR test copy is now back on 1.8.3. Original guarded files, full
  WeiDU.log, receipt and dialog.tlk passed read-only restoration checks. See
  [exact package/acceptance evidence](../package-acceptance-alpha19-2026-09-22.md).
- The compact Updates selector and clearer restored-history text passed their
  source checks and are part of the new local package, not the user's earlier
  diagnostic package.

## Local package built; user review next

The approved parallel changes are integrated. Combined native checks, the naming
follow-up and the normal local package build passed. This is not yet a packaged
UI playtest or publication; no game files were changed.

- Updates: concise, shared selection; real progress stages; clear completion,
  failure and Undo feedback. No invented percentages or success after failure.
- Installation names: stable, distinct names, including failures and legacy
  duplicates. Renaming a label must not move game folders or saves.
- Folder failures: name the actual path/problem, preserve choices on retry, and
  cover missing/unavailable/denied cases. The viewer's root cause is not proven.
- Documentation: current acceptance, short changelog and
  [troubleshooting/backup guide](../troubleshooting.md).
- Existing UI/Customize fork: integrate only completed changes. Coordinate with
  that task; do not reimplement its work. Known UI compatibility problems must be
  fixed or the affected choice left unavailable, not described as tested.

The completed Customize frontend is integrated. Alternative UI provider recipe
integration is not: controls remain absent when the bundled recipe does not
declare the verified group. Do not mistake the preview recipe for release data.
No receipt/save/folder rename is performed by the new naming controls. The narrow
cross-process same-name creation limitation is recorded in the handover.

Checks so far: frontend typecheck, **323 tests** and production web build passed.
The synthetic headless review passed desktop/narrow progress, success, failure,
Undo/recovery controls and Customize (20 final screenshots, no overflow or console
errors). Evidence: `target/work/sep22-feedback-layout/verification.md`.
The full Python suite passed **102 tests, one existing opt-in real-WeiDU skip**.
Two old 448-component expectations were corrected: the approved interval-save
menu (HGO 38) is the sole extra selection, making 449. Recipe/approvals unchanged.
Combined native tests passed **797 tests, 13 existing opt-in ignored**, plus
formatting and Clippy with warnings denied. Evidence:
`target/work/sep22-feedback-final-native` (exit 0). The earlier Tauri channel
compile error and legacy generic-name assertion were corrected.
A final rename-to-new-install destination correction passed the follow-up app
library check (**38 tests, including 8 naming tests**), **40 command-contract
tests**, Clippy and formatting. The normal package built and **4 package tests**,
including updater-signature verification, passed. Job
`d75b1f80-08e3-4958-9b3d-bc28168d0a12` finished with exit 0 at
`2026-09-22T15:13:36Z`; evidence is in
`target/work/sep23-feedback-review-package-final`.
The preceding attempt's two mixed-separator Windows test expectations were
corrected with native path joins. Offline package inspection also passed: all
191 resource files match source hashes, no extra recipe files, and the executable
matches except for Tauri's documented NSIS marker. Verified setup/signature copies
are in Downloads; exact paths/hashes are in the package-acceptance record.

## Next release-install playtest — additions agreed September 27

Christopher wants these checked together with the next release installation,
not in a separate test install now. This records the test plan, not acceptance
or a change to the current recipe.

- [ ] **Generic unidentified-item artwork (Modpack 630):** include it in the
  next combined test selection, with Randomiser's conflicting appearance option
  **570 off** (ordinary loot randomization may stay on). Check inventory, shops,
  bags/containers, ground, dragging and description pictures; include cursed and
  mod-added items. Identify one item and confirm its original art returns while
  an unidentified duplicate remains generic. Check quick slots and save/reload.
  Confirm item powers/text are unchanged. Verify uninstall/restoration later on
  the disposable test copy, after the other gameplay checks.
  Source: Modpack `codex/unidentified-item-appearance`, commit `6eefd2c`;
  full checklist: owning repo `docs/unidentified-items.md`.
  [Implementation and integration boundary](2026-09-18-randomiser-next-release-triage.md#september-27-generic-artwork-implementation-unreleased).

Prepare the Modpack pin and CEBG mutual-exclusion metadata before assembling that
test recipe. Do not apply this to the stream installation or claim it already
ships in the current installer.

## Final package and publication boundary

1. Complete: the refreshed September 29 combined checks/build pass. Both stale
   exact UI test expectations were corrected; no validation was weakened.
2. Confirm app alpha.19 / collection alpha.18 and included feature scope. Update
   [alpha.19 release notes](../release-notes-alpha19.md), package evidence,
   README/current-status text and the website handoff consistently.
3. Complete: refreshed package, updater signature and bundled recipe verified;
   hashes and the dated Downloads copy are in the September 29 package record.
   The old September 23 binary is not this candidate.
4. Use one combined fresh installation and the [short check](../next-release-smoke-check.md)
   for the new component scope. Do not repeat the accepted BuffBot Apply/Undo or
   add a personal UI/kit matrix. Let Christopher assess finishing touches there.
5. Publish the approved package, updater feed and website together after the
   release decision. A successful local build and signature check are not
   publication or user acceptance.

Updater signatures are not Windows Authenticode publisher signing. Don't promise
the Windows unknown-publisher warning has gone away.

## Short announcement draft

Use only after publication; add the finished UI changes when their final package
is verified. This draft deliberately omits work still in progress.

> Small but useful CEBG update:
>
> - BuffBot can now be updated without reinstalling your whole game, if your
>   installation supports it. Includes backups and Undo; both have been tested.
> - Added a fix for some missing Artisan kit descriptions in existing installs.
> - Fixed automatic saves being left inactive in the wrong game profile. New
>   recommended installs also include the in-game save settings.
>
> Updating CEBG itself won't change your game. Pick your installation in Updates
> for the supported game fixes. This isn't an "update every mod" button yet :)

## Small cleanup inventory — read only

September 22 agent-visible directory check, limited to the named test/backup
locations; no files were deleted or moved and no full size/hash scan was run:

- `C:\Users\chris\Games\CEBG-Tests` exists; no immediate child installation
  directories were returned. This is not a scan of other test locations.
- `C:\Users\chris\Games\CEBG-Backups` contains
  `RC-20260903-before-kit-links-20260920`. Review after the RC patch work is closed;
  it is not approved for removal by this inventory.
- `C:\Users\chris\Games\ChrizEasyBG-No-SR-Test` and
  `C:\BG-EET-RC-20260903\game` still exist. Keep them and the update/recovery
  backups until the lead/user selects the copies to retire.

## Not release gates

- A general multi-mod updater, collection-wide current-run/new-run/new-install
  classification and additional hotpatches: explicitly lower priority now.
- Legacy app-data migration and cleanup conveniences: separate follow-ups, not
  automatic additions to this release. Manual breakpoints were moved into the
  release scope by Christopher's September 29 clarification above.
- New gameplay/default choices or unconfirmed owning-mod work.
- Deleting old real installations/backups. A read-only inventory is preparation;
  deletion requires a confirmed keep/remove scope. Keep the accepted test's
  recovery backups for now and do not touch the stream installation.

The [work index](2026-09-20-next-work-index.md) retains the broader queue. The
[handover](../handover.md) is the live status if this checklist becomes older than
the ongoing work.

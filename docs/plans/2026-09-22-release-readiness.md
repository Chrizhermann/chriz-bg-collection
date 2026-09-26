# September 22 release readiness

This is a working checklist, not a publication announcement. App **alpha.19** and
collection **alpha.17** are the current local candidate versions. The normal local
package built successfully on September 23 (KST); it has not been installed,
launched or published. Christopher's finishing UI review is next.

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

## Final package and publication boundary

1. Done: combined source checks, naming follow-up and local package build. The
   earlier compile/test failures were corrected, not waived.
2. Confirm app/collection versions and included feature scope. Update
   [alpha.19 release notes](../release-notes-alpha19.md), package evidence,
   README/current-status text and the website handoff consistently.
3. Done: normal local package, updater signature and packaged-resource inspection;
   artifact hashes and the verified Downloads copy are recorded.
4. Let Christopher assess the finishing touches in that package. No repeat
   overnight full installation or BuffBot Apply/Undo is required solely for copy
   changes. Reassess only if later changes affect the accepted behavior.
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
- Manual install breakpoints, legacy app-data migration and cleanup conveniences:
  separate follow-ups, not automatic additions to this release.
- New gameplay/default choices or unconfirmed owning-mod work.
- Deleting old real installations/backups. A read-only inventory is preparation;
  deletion requires a confirmed keep/remove scope. Keep the accepted test's
  recovery backups for now and do not touch the stream installation.

The [work index](2026-09-20-next-work-index.md) retains the broader queue. The
[handover](../handover.md) is the live status if this checklist becomes older than
the ongoing work.

# Short path to a public CEBG alpha

Christopher asked for a practical alpha release tonight, clarification of CLI versus
desktop installation acceptance, and a visual test of app/collection update notifications.
This is the release plan, not authorization to publish a repository, tag or update feed.

Christopher subsequently agreed to this alpha scope and queued recovery/customization/
skip-SoD requirements. [The captured priorities](2026-09-05-recovery-and-casual-customization.md)
keep safe pause/close handling ahead of Customize QoL; the optional SoD skip remains
parallel owning-repo work for an early alpha, not a reason to restart the running install.

## Reuse the real work already running

Current r5 replaces r4 after a diagnosed recipe-only TP2-alias mismatch, not to repeat a
button click. All 30 SoD components succeeded in r4; its immutable failure stays intact.
The corrected alpha.8 recipe has unchanged component selections/order and plan hash.

Fresh r5 uses the same engine preparation and `execute_frozen_campaign` pipeline as the
desktop app. The desktop route additionally validates its reviewed identity/token,
handles native events/dialogs, and records its application version. R5 was started by
the CLI, so it correctly has no originating app version. Do not fabricate one.

Do not repeat all 43 runs merely to say a button started them. Previous native attempts
already exercised actual start, manual archive supply, progress, failure and resume.
Complete r5 and separately verify the packaged app's remaining lifecycle seams. Report
these evidence scopes accurately rather than claiming an uninterrupted native end-to-end
run that did not happen.

## Minimum useful release sequence

1. Complete r5; verify successful receipt, frozen version and exact WeiDU component list.
2. Add the newest verified Radar release to that successful game copy.
3. Rediscover it in packaged CEBG; test Play, Open game folder, Back and Updates.
4. Launch a new game and save/reload. Christopher can perform the gameplay smoke once
   the copy is ready; the agent can do it if needed. A second gameplay run is not mandatory.
5. Check the new update visuals with simulated app-only, collection-only, combined,
   app-required, offline and current states. Keep simulations out of production entrypoints.
6. Exercise actual self-update apply/restart and preservation of registered game installs;
   signed download/tamper rejection and a manual NSIS upgrade have already passed.
7. Resolve public acquisition, then obtain publication authorization. Build/sign the
   release candidate, publish the installer and alpha feed, and verify public download,
   version/checksum/signature and feed reachability. Keep third-party mods out of the package.
8. Retain the successful game for review; clean only authorized disposable tests while
   preserving evidence and the explicitly protected/rejected cleanup targets.

Tonight is a target, not a promise before the running full install and distribution checks
pass. Do not expand this into another general code-review or mod-curation project.

## Distribution decision still needed

The full creator acceptance recipe currently acquires Evandra from the user's 1.26 GB
aggregate archive. That archive must never be published or bundled. Public users need
the original standalone archive and an honest download/manual-supply flow with its own
verified contract. The [official Evandra 2.2 page](https://www.gibberlings3.net/files/file/1008-evandra/)
and its ordinary download chooser were checked: Windows `evandra-v2.2.exe` (12.81 MB),
`lin-evandra-v2.2.zip`, and `osx-evandra-v2.2.zip` are offered. A normal manual download
route exists; availability is not the blocker. Obtain/verify the standalone package,
its digest/layout and full relevant payload equality, then freeze that acquisition
contract. Do not silently drop Evandra or other approved choices to get a release out.
This does not change the frozen r5 installation.

## Updates in the alpha

The signed app self-updater already exists. Recipe changes initially arrive inside signed
CEBG app packages rather than through a separate recipe channel. The interface must still
distinguish an update to CEBG itself from a newer setup for an existing game installation.
An app update changes the installer/launcher, not installed game files. Collection updates
create a separate installation; next-playthrough changes are nonurgent and do not alter
the current save. Hot-patching remains outside this alpha path.

The header notification must be visible without hover, identify the update types in a
keyboard-accessible tooltip, and never turn stale/offline version fields into an available
update. Versions/changelogs and a clear action belong in the compact Updates screen.

The notification/source UI slice is now implemented and visually checked; see
[focused acceptance](../updates-ui-acceptance-2026-09-05.md). All 80 frontend tests and the
web build passed. The next signed package must include this slice; installed alpha.8
predates it. No public publication or actual self-update apply occurred during this check.

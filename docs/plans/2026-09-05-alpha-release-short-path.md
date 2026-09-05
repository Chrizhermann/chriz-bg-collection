# Short path to a public CEBG alpha

Christopher asked for a practical alpha release tonight, clarification of CLI versus
desktop installation acceptance, and a visual test of app/collection update notifications.
Publication was explicitly authorized on 2026-09-06 after Christopher's Discord
announcement. Publish the verified alpha binary/update feed in the separate public
`Chrizhermann/chriz-easy-bg` repository; keep this collection source/history private.
The existing website task owns the collection page and deployment, after receiving
the exact verified release URL. The earlier snapshots below are historical.

Current finish line: app alpha.10 / recipe alpha.11, official Windows Evandra
download-or-skip and accepted SoD v0.6.7 skip910. No second full install is required
for these bounded acquisition/recipe changes. Record the remaining alpha acceptance
limits honestly in the release guide.

## Latest user follow-up (2026-09-06)

Christopher is playing recovered r5 and reports no crashes or issues so far.
This is user-reported initial gameplay acceptance, not confirmation of every
save/reload, Radar or installer-update check. Do not interrupt his session.

- Radar convenience: Christopher notes the executable can live in the game root
  and accepts a small launcher button as an alternative. Queue **Open Radar**
  beside the other launcher actions, using the selected installation's managed
  add-on path. Existing r5 files need not be moved while he plays. This button
  is a requested follow-up, not yet implemented.
- Website distribution belongs with the existing **Build interactive BG run page**
  task in `twitch-setup-chriz` (task `01a061fe-a68c-74b1-8272-5bfa9251a731`).
  Coordinate the CEBG alpha download/version/changelog link there once the signed
  public artifact and URL are ready; no website deployment was performed here.
- SoD full-campaign skip is still separate from the newly released **v0.6.6**
  victory-ending component **290**. The recipe and r5 still use **v0.6.5**;
  the optional-skip prototype is not release-ready. See the intake queue.
- A public download still needs the current signed package and self-update
  apply/restart acceptance, safe pause/app-close work, and a verified standalone
  Evandra acquisition contract. A playable r5 does not establish those seams.

The alpha.9 app compiled and NSIS generation completed. Its original password
prompt was cancelled; the signing issue is now resolved by explicitly supplying
the existing key's empty password. The built candidate now has a verified Tauri
signature, and the real updater's loopback download/tamper-rejection check passed.
It predates the newly approved recipe/lifecycle changes and must be rebuilt and
re-signed after integration. No installation or apply/restart acceptance occurred.
Keep current desktop-control restrictions and all no-rebuild boundaries.

**2026-09-06 update:** [targeted recovery](../targeted-recovery-2026-09-06.md)
supersedes the r6 path below. R6 was stopped before mod installation; the existing
r5 game now has all 430 components after localized modpack replacement and the
remaining three commands. Preserve earlier Bardic balance.2. [Managed recovery completion](../managed-recovery-acceptance-2026-09-06.md)
has now passed, with separate receipt/registry, isolated saves and Radar 2.5.0.0 installed.
Next perform current native launcher, gameplay and app-update apply/restart acceptance.
No new full rebuild without discussing necessity/cost with Christopher.

Christopher subsequently agreed to this alpha scope and queued recovery/customization/
skip-SoD requirements. [The captured priorities](2026-09-05-recovery-and-casual-customization.md)
keep safe pause/close handling ahead of Customize QoL; the optional SoD skip remains
parallel owning-repo work for an early alpha, not a reason to restart the running install.

Today's finished owning-repo work may enter the first alpha. Consult the
[bounded mod intake queue](2026-09-05-first-alpha-mod-intake.md) once before freezing
the next release candidate. R5 remains immutable, terminal failure evidence. Released
modpack alpha.5 repairs and Bardic balance.3 are now pinned in alpha.9 for r6; the
43-run / 430-component selection, order and arguments are unchanged. New components
still need their own selection policy; do not hold the installer for unfinished work.

## Reuse the real work already running

Current r6 replaces terminal r5 after the owning repository released and tested both
modpack fixes. R4's TP2-alias mismatch and r5's Xan/Viconia failures remain historical
evidence, not resume targets. Alpha.9 retains the approved selected components.

Fresh r6 uses the same engine preparation and `execute_frozen_campaign` pipeline as the
desktop app. The desktop route additionally validates its reviewed identity/token,
handles native events/dialogs, and records its application version. R6 was started by
the CLI, so it correctly has no originating app version. Do not fabricate one.

Do not repeat all 43 runs merely to say a button started them. Previous native attempts
already exercised actual start, manual archive supply, progress, failure and resume.
Complete r6 and separately verify the packaged app's remaining lifecycle seams. Report
these evidence scopes accurately rather than claiming an uninterrupted native end-to-end
run that did not happen.

## Minimum useful release sequence

1. Complete r6; verify successful receipt, frozen version and exact WeiDU component list.
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
This does not change the frozen running installation; a public acquisition change needs
its own versioned recipe.

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

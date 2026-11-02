# Existing-install updates: preserved requirements, lower priority

## September 22 priority change

Christopher lowered the priority of expanding updates beyond BuffBot, general
current-run/new-run/new-install classification, and additional hotpatch adapters.
Defer those expansions; they do not block the coming release. BuffBot's exact
1.8.3-to-1.8.4 Apply, user gameplay and Undo passed; retain that supported route
and the accepted description patch. Existing safety checks and accurate operation
labels remain required. Do not present these two routes as a general updater.

Favor useful parallel work that does not need gameplay/curation decisions, with
the proposed batch presented first. Progress/completion feedback for existing
operations remains worthwhile; a classification audit of all changed components
is deferred. The original requirements below remain the later design reference,
not current scheduling instructions to prioritize this over all other work.

## Historical September 20 priority decision

Christopher's September 20 priority correction. Requirements capture only; not a
completed design, implementation or release claim.

## Product gap

Christopher expected Updates to be able to update his existing installed mod set,
not always redirect to building another installation. The current collection
update action does the latter. App self-updates, Radar updates and a narrowly
approved existing-install patch are different capabilities and must not be
described as general collection upgrades.

Source checkpoint: `app/src/screens/updates.ts` exposes `buildUpdatedCopy` and
explicitly describes collection updates as creating a new installation.
`app/src/app.ts::#buildUpdatedCopy` enters `#beginNewInstallation`; preserving a
compatible setup draft is not equivalent to reconstructing the selected existing
installation's choices. Include receipt-based selection continuity in the next
design rather than assuming the current new-install flow already provides it.
`patches/catalog.json` contains the Artisan campaign-description-link adapter;
its own summary excludes kit-ability upgrades, description-text replacement and
save changes. That pilot and its Updates integration do not deliver the broader
upgrade capability requested here. Parallel delivery work may advance; refresh
its status rather than repeat its already completed implementation/tests.

## Priority and intended outcome

- Put a genuine existing-install update path ahead of UI providers and manual
  installation checkpoints. Reuse existing patch/backup/history infrastructure
  where appropriate; do not confuse shipping the narrow pilot with closing this gap.
- Target the user's selected existing installation and installed selections;
  evaluate available updates against what they actually installed. Preserve their
  choices instead of resetting to the latest default recipe or silently adding mods.
- Explain exactly which installed components will change, which remain unchanged,
  and which updates require a new run or a rebuilt installation. A fresh copy is
  a fallback for changes that need it, not the only generic Update action.
- Preserve truthful version/history reporting: partial updates must not label an
  entire installation as the newest collection. Keep app version, collection
  baseline and applied game changes distinct and understandable.
- Carry forward game-closed, backups, recovery/undo and no automatic save-editing
  requirements. Existing-install upgradability and existing-save compatibility
  need separate assessments; neither follows merely from a matching WeiDU.log.
- Manual-mod additions remain excluded from automatic game hotpatches in the first
  supported policy. App updates remain independent.

Next work, when resumed: inspect current delivery status, choose one useful bounded
installed-component update path, design its eligibility/recovery/user-facing
behavior, implement and validate it proportionately. No universal updater or safe
arbitrary mid-stack reinstall is promised by this note. No game mutation/release
was performed to capture these priorities.

## Parked work, preserved for later

[UI choices and manual checkpoints](2026-09-20-ui-choices-and-manual-checkpoints.md)
retain the source findings, dependencies, UX outline and latest user decisions:
Infinity UI++, EET SoD-style UI and LeUI accepted as initial candidates; live UI
acceptance can be community testing. Dragonspear UI++ waits for a user request.
New-install-only UI choices and the limited advanced manual-mod workflow remain.
Do not keep researching, implementing or dispatching those parked features now.

Existing update design context:

- [Accepted narrow patch design](2026-09-20-existing-install-patch-pilot.md)
- [Production patch implementation record](../existing-install-patches-acceptance-2026-09-20.md)
- [Component-by-component update triage](2026-09-14-existing-install-update-triage.md)
- [Next-work index](2026-09-20-next-work-index.md)

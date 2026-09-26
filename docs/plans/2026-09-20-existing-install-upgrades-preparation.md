# Next implementation: update this installation

Original preparation: September 20, 2026. The read-only comparison and preserved
choices are now implemented in source; see the [September 21 implementation
record](../existing-install-comparison-2026-09-21.md). The findings below retain
the preparation snapshot, not a claim that every listed gap remains untouched.
The [first BuffBot whole-mod route](../existing-install-upgrades-2026-09-21.md) now
implements application and its specific update/restore history in source; general
multi-mod upgrading and real-execution acceptance remain open. No interactive
app/game testing, installation writes, version changes, packaging or publication
are authorized by this plan; interactive testing waits for Christopher.

## Outcome

Select an existing installation, keep its choices, and update supported installed
components **in that same folder**. Clearly show anything that cannot be updated.
Creating a new installation remains a separate fallback, not the update button's
hidden behavior. A preview alone is useful groundwork, but does not deliver this
feature; the first delivery must include an actual supported component update.

This implements the [priority decision](2026-09-20-existing-install-upgrades-priority.md).
The existing [description-patch pilot](2026-09-20-existing-install-patch-pilot.md)
is reusable groundwork, not proof that whole mods can already be upgraded.

## Source findings

Reviewed the current `installer-v0-real-alpha` working tree, including its pending
alpha.19 changes. These are source findings, not results from a user's game.

| What exists | What is missing |
|---|---|
| `InstallReceipt` retains normalized choices, typed inputs, executed plan, payload hashes and final logs (`engine/src/receipt.rs`). | An installed-set update planner; `app/src/app.ts::#buildUpdatedCopy` still calls the new-install flow. |
| `NormalizedSelection::to_selection` and `evaluate` can replay semantic choices (`engine/src/recipe_view.rs`). | Upgrade-specific migration: evaluation initializes **new recipe defaults** before applying old choices. Blind replay can add new content or enforce newly mandatory features. |
| Release ledgers and `diff_recipe_packages` explain recipe changes (`engine/src/updates.rs`). | Filtering/mapping to this installation's actual selected components, input values, recovered artifacts and completed patches. A release-wide summary is not that comparison. |
| Session state retains the frozen recipe payload/envelope and verifies their identity (`engine/src/session.rs::verify_frozen_recipe`). | Check availability and verified lineage before using these for an old-to-new diff. Missing old metadata is not permission to guess the baseline from current defaults. |
| Registry/receipt validation plus explicit supervised recovery receipts (`engine/src/registry.rs`, `recovery_receipt.rs`). | A common effective-state reader that retains original selections and applies verified recovery replacements/order changes. `read_completed_state` returns final state, not all that lineage. |
| Signed catalog, review-token recheck, target lock, affected-file backup, interrupted-patch recovery and guarded undo (`engine/src/patches/`). | More than the one hard-coded Artisan description adapter. `verified_log_suffix` explicitly rejects multiple active patch receipts; its suffix model cannot describe a mod replacement. |

Do not loosen those restrictions and call that support. Add a separate supported
transition contract, retaining compatibility with existing patch receipts.

## First implementation slice: accurate preview

Add a read-only `InstalledUpdatePlan` (proposed name), addressed by installation ID
and trusted target recipe identity. Do not reuse a setup draft or latest defaults.

1. Verify the registry target and immutable receipt. Ordinary successful installs
   and explicitly recovered installs are distinct supported inputs; failed or
   incomplete installs go to recovery, not updates. Unknown receipt schemas are
   reported as unsupported, never silently interpreted as the latest schema.
2. Reconstruct effective state from the original receipt, verified recovery
   replacements/order authorizations and completed update history. Compare actual
   active WeiDU rows. Unexpected manual changes disable automatic game updates for
   now, but not CEBG app updates. A matching log is necessary, not proof that every
   game file is unchanged; adapter-local resource checks remain required.
3. Preserve every known selection and input, including exclusions. New optional
   features stay off. A newly mandatory feature, missing/renamed ID, changed input
   meaning or new dependency requires an explicit migration/decision; do not force
   it on or silently discard the previous choice. Explicitly disable new features
   before evaluation and compare the result with the intended preserved set.
4. Map chosen features to actual component identities, language, order, settings
   and artifact digests. Compare with the target recipe. Do not use version-string
   changes alone, or assume one artifact change affects every component equally.
   Changed settings with the same archive still matter. Shared prerequisites and
   generated resources can make an apparently local update depend on other changes.
5. Produce entries for unchanged, already delivered, available update, needs new
   run, needs new installation, and unassessed/unsupported. Keep delivery method
   separate from save compatibility. Unselected content is not an update to install.
6. Bind the preview to its baseline/history digest, target recipe, chosen actions
   and relevant resource checks. Apply must recheck that identity under the target
   lock; a stale preview requires another check.

The desired UI is small: **Updates for [install name]**, a short change list, the
existing/new component versions where known, and **Update this installation** only
when supported actions exist. Details explain save limits and unavailable changes.
The user can apply an independent supported subset without accepting new content.

## First delivery: one real component upgrade

Preferred investigation candidate: an installed **BuffBot** version transition.
It is the final run in the current curated recipe (`buffbot-bg2`, components
`[1, 0]`), and both 1.8.3-alpha and 1.8.4-alpha have recorded artifact descriptors.
This makes it a bounded candidate, **not an approved upgrade route**. The actual
installed log must confirm its tail position; a recipe position alone is insufficient.

Before enabling that transition, the owning mod's source must establish:

- Exact old/new payload hashes and selected components/language; no upgrade of an
  unselected mod and no silent settings/default changes.
- What installed/generated files and WeiDU backup records change, and how removed
  old files are handled. Overlaying a release archive is not sufficient.
- Whether the update needs additional runtime dependencies, touches TLK/UI/shared
  tables, or changes saved configuration/innates. Continuing existing saves needs
  its own supported statement; otherwise label it new-run or unsupported.
- A bounded execution route that preserves the selected suffix, does not trigger
  an unassessed middle-of-stack reinstall, and has a complete recoverable write set.
- Verified postconditions that prove the new component actually ran. Exit code 0
  or an already-present WeiDU row alone is not enough.

If that transition is unsuitable, keep it unavailable and select another narrow
owner-supported upgrade. Do not broaden to arbitrary reinstalls to meet a deadline.
The existing SR protection-refresh adapter is a separate gameplay-fix candidate,
not a substitute for claiming a whole SR or collection version was upgraded.

## History and recovery before extending Apply

- Keep original install receipts immutable. Add append-only update records linked
  to the preceding effective state, with exact old/new identities, delivered change
  IDs, component coverage, settings, write set and backup locations.
- Record partial fixes separately from complete component upgrades. Keep the base
  collection label plus applied updates; never rename the entire installation to
  the newest recipe because one selected fix was applied.
- Support the existing `cebg-v1` patch history explicitly. Two consecutive updates
  must work: verify the current effective state, not every old intermediate file
  image against today's files. Unrecognized history means no automatic mutation.
- Pre-download and verify payloads; acquire the target lock, require game/writers
  closed and recheck the plan. Complete and verify affected-file backups before
  the first write. Keep the recommended full-install backup and optional separate
  save snapshot. A selected backup failing stops the update.
- Persist pending state before writes. On failure/interruption, offer verified
  restore or a specifically supported continuation. Never retry a partially
  applied install command blindly, and never treat WeiDU's failed reinstall as
  proof the previous installed version was restored.
- Undo is initially latest-update-only with current post-image checks. Restoring
  game files does not undo progress saved after the update. No save editing or
  shared-profile migration in this feature. TLK-writing adapters require additional
  old-reference preservation/rollback design and are not enabled by this plan.

## Small acceptance set for implementation later

1. Choices: non-default install, SR off, mutually exclusive selection and typed
   input remain unchanged despite new defaults; new mandatory/unknown IDs produce
   a visible decision or unsupported result, never a default reset.
2. Provenance: ordinary and supervised-recovery receipts, an existing Artisan
   patch, partial coverage and already-current components yield accurate previews.
   Unknown schema/manual log or relevant-file drift cannot gain an Apply action.
3. Delivery: one supported old-to-new component update in a disposable fixture;
   settings, unrelated files, original receipt and saves remain intact. Repeat is
   a no-op; then a second supported update exercises the history chain.
4. Failure: changed review token, bad payload, insufficient/failed backup and
   interruption during mutation. No unverified launch, no new-install fallback
   presented as recovery, and restoration is checked rather than merely reported.
5. User-present acceptance: review/apply/reopen/undo and the changed feature in a
   disposable game or explicitly approved copy. Scope this to the delivered
   component, not another full collection install/playthrough by default.

These checks are proposed, **not executed in this preparation**. Automated fixture
evidence and native/game acceptance must be reported separately.

## Work split and parked items

- Lead: own the transition/selection rules and integrate findings.
- Sol: completed a read-only source map; no code or runtime changes.
- Opus: completed the bounded read-only transition/recovery assessment. No code or
  runtime changes, no recursive delegation. Its useful history/footprint findings
  are incorporated; no extra review round is needed for this preparation.
- User + parallel fork: current packaged build testing and any associated release
  work. This preparation does not edit their release files or use their fixtures.
- Keep [UI choices/manual breakpoints](2026-09-20-ui-choices-and-manual-checkpoints.md)
  parked. Other recorded work stays in the [next-work index](2026-09-20-next-work-index.md).

No additional product choice is needed to start the read-only planner. A concrete
upgrade candidate still needs its bounded source assessment before Apply work.

### Lead resolutions from the two reviews

- Retain Opus's warning that changing the hard-coded patch ID would also make old
  transaction records unreadable. New adapters must coexist with the old adapter
  identity and verifier; a folder namespace change alone does not solve history.
- Do not ship only another narrow fix and call existing-mod upgrades finished.
  Opus proposed SR protection refresh as a low-complexity second adapter; it is a
  useful later candidate, while the preferred complete-upgrade investigation is
  BuffBot as described above. Neither has received new compatibility acceptance.
- Undo ordering is global per installation: after A then B, A remains recorded,
  but only B is offered for undo. After verified undo of B, reassess A. Do not
  promise independently undoing A while B remains applied.
- A manually altered mod list gets an explanation and no automatic game update
  under the current agreed policy, even if a narrow adapter's local inputs happen
  to match. Supervised exceptions remain separate explicit work.
- Current descriptions use live resources appropriately, but that is not enough
  to reconstruct intended whole-mod choices. Preserve receipt choices **and**
  verify effective installed state; neither replaces the other.

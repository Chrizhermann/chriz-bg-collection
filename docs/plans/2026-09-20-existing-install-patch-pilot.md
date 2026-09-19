# Existing-install patch pilot

Status: supervised apply/no-op/rollback pilot completed, then reapplied in a second
transaction and **user live accepted**, September 20. Christopher confirmed the
descriptions worked and a loaded existing save behaved normally. The public
Updates/Rust/Tauri flow is now **implemented in source**, not packaged or released.
The requested Opus 5 xhigh design review completed; it was not an implementation
review. No automatic public patch delivery exists yet. This does not authorize
writes to other installations, shared saves or `C:\Games`.

See the [pilot acceptance record](../hotpatch-pilot-acceptance-2026-09-20.md).
The first RC cycle was restored; the second, accepted test cycle remains applied
with its separate test-profile harness. Full game/profile backups remain. Do not
mistake the first cycle's rollback record for the current RC state.

The [production-flow acceptance record](../existing-install-patches-acceptance-2026-09-20.md)
records 25 engine patch tests, 15 native-app tests, 176 frontend tests and passing
frontend typecheck/build. Its real public-download/WeiDU apply-repeat-undo test
used a synthetic game and preserved receipt/save bytes; it did not modify the RC
or stream install. Public packaging and native packaged-UI interaction remain
separate acceptance work. Alpha.18 app/recipe versions and feeds are unchanged.

## Accepted refinements and design review

Christopher approved proceeding with the bounded pilot. Save files must never be
edited by the patch executor. An existing-save repair is a separate, explicitly
approved workflow, not an automatic part of installing a current-run patch.

The initial public UI should always create affected-file backups and offer
"Back up the entire installation first", recommended and checked by default,
with the required/free space shown. A failed selected backup stops before patching.
Offer a separate snapshot of the installation's save directories outside cloud
sync (do not guess one "relevant" save). No save conversion, deletion or rewriting.
Explain that rolling game files back cannot undo progress saved after a patch;
returning completely to the old state requires the matching pre-patch save.

Opus 5 xhigh completed the requested read-only design review in approximately
14 minutes. No additional review loop is required before this bounded pilot.
The following resolutions apply to the implementation:

- Only the committed Artisan description-link adapter is the first candidate.
  Klatu 2150 is absent on this RC: adding it would be new optional content, not
  an update of a selected component. The Artisan adapter is committed in release
  source `d16d35ba29c68aa18025a1c5323a8973986a97d9`; the earlier triage's
  uncommitted-worktree observation is historical.
- Perform eligibility and already-fixed checks before launching WeiDU. For this
  pilot require the needed 2DAs physically in override; BIF-only inputs are
  unsupported. Validate all selected references against the target TLK first.
- Pin adapter file hashes and the WeiDU executable. Review the tiny adapter's
  full source; no shell hooks, prompts, TLK writes or new kit/spell registration.
  Use non-interactive/no-autoupdate flags and an external DEBUG log, not quick-log.
  Declare adapter staging/backup files and WeiDU.log in addition to resource writes.
- Pilot state lives in `C:\BG-EET-RC-20260903\.chriz\patches`, outside game and
  the profile. Import a create-once observed baseline; retain the legacy receipt.
  Record each transaction's preceding record digest, adapter identity, before/after
  hashes and newly-created paths. One patch per transaction; failures never become
  a successful latest patch. Rollback is latest-patch-only, guarded by unchanged
  post-images, and recorded as a new event.
- Reject symlinks/junctions and hardlinked write targets. Check game/loader/WeiDU
  processes and exclusive access to affected files and TLK, then lock pilot state.
  This does not prevent direct EXE launches; check pending state before a manual test.
- Capture a game-tree metadata manifest around the pilot (not every app startup).
  Undeclared changes mean blocked/unverified: restoring the declared set is not
  evidence that an unknown write has been undone.
- Native acceptance, if performed, uses a temporary unique engine profile name as
  a separately backed-up harness change. Never launch against the original RC
  profile. Cloud placeholders in its save profile are not assumed unsafe junctions;
  backups must be complete before any test uses a copied save.
- Public patch metadata will be a separate signed catalog referencing existing
  update change IDs. Do not extend the immutable, strict recipe ledger schema.
  A local hash-pinned definition is sufficient for the supervised pilot.
- Evaluate eligibility per installation, and report partial coverage. The imported
  RC has no complete recipe delta: other updates remain unassessed, not implicitly
  applied. Existing `Before*` classifications need proven guards or no Apply button.
- For v1, "Needs a new run" offers "Create an updated installation and start a new
  game there"; a separate patched-copy workflow is not implemented yet.
- A later text adapter must preserve existing TLK entries. Automatic rollback
  cannot truncate appended strings that post-patch saves might reference.

The pilot remains supervised tooling. The separate production Updates integration
now exists in source; it is not yet a shipped public action.

## Small scope

Support straightforward, individually described fixes whose supported inputs can
be checked cheaply. Uncertain updates stay unavailable for in-place application;
do not spend days making every old stack work. Reuse receipts, the existing
update ledger and the verified downloader. Mod owners supply versioned adapters;
the collection does not absorb independent mod implementations.

## Three player-facing categories

| Label | Meaning | Action |
|---|---|---|
| Can be applied to your current run | The particular patch supports the detected installation and continuing existing saves, including any explained restart/recast limits. | Apply selected fixes, with the game closed. |
| Needs a new run | The existing installed stack can be patched without rebuilding all mods, but the changed content requires a fresh campaign. This does not imply TLK corruption. | Prepare a separate patched copy for a new run; leave the old game/profile usable. |
| Needs a new installation | No supported incremental route exists for this combination; install order, generated resources or an unassessed change require a rebuild. | Build a separate updated installation. Old saves are not automatically portable. |

Internally keep delivery (`targeted-patch` / `rebuild`) separate from save support
(`current-run` / `new-run` / `unknown` plus optional guarded event conditions).
An unknown assessment is not a proven incompatibility: show "Not assessed for
patching; use a new installation". Already fixed, mod not selected and unsupported
inputs are separate eligibility results, not extra save categories.

For v1, only implement the current-run Apply path. New-run/rebuild rows explain
the requirement and retain the existing new-install action until isolated-copy
preparation is implemented. Never expose a destructive new-run patch action against
the user's active playthrough merely because they accepted a warning.

## Per-change coverage, not blanket mod-version upgrades

- Every changed selected component in a recipe delta must have coverage: current-run
  patch, new-run update, rebuild, unchanged/no relevant effect, or unassessed.
- A change can cover multiple components, or one component can have several patch
  units. Evaluate prerequisites/conflicts and the accumulated patches on the actual
  target. Never label a whole mod upgraded after applying only one fix from it.
- A safe subset may be offered separately when independent; never split a dependent
  batch so half of an interdependent correction is accepted.
- Preserve user choices. Optional gameplay/balance changes need explicit selection;
  absent mods are not added automatically. Difficulty changes are not bug fixes.
- Display the base collection plus patch IDs/revisions, not a fictitious complete
  newest recipe version. Legacy RCs with no CEBG receipt get an explicit imported
  baseline, never a fabricated original receipt or inferred full-release identity.

## Minimal requirements before any write

1. Resolve the selected target and reject unsafe roots, unexpected links/junctions,
   store-managed sources and unsupported shared writable paths. Preserve configured
   source games. The pilot is the exact user-named RC, not a new destination under C:.
2. Ensure its game/loader and writers are closed, then acquire an installation lock.
   A game running from a different path is not blanket evidence this target is locked,
   but shared files/profile directories must be accounted for before mutation/launch.
3. Check installed components/order, versions, relevant runtime prerequisites and
   patch-local effective resources. "Similar to our main install" means these explicit
   conditions, not a percentage of matching mods. No full disk scan on startup.
4. Exact file replacement is allowed only for known whole-file inputs and output
   payloads with installation-local IDs/text references accounted for. Otherwise use
   a tested semantic edit preserving unrelated content, or decline the patch.
5. Authenticate patch metadata and archive hashes before extraction. No generic
   downloaded shell scripts or broad folder overlays. Use an allowlisted adapter
   with a declared write set; public signed metadata is not runtime sandboxing.
6. Record a durable pending transaction and back up every affected existing file,
   the absence of newly created files, and touched WeiDU/patch bookkeeping. For
   the pilot, prove the adapter's write set with before/after comparison. Reject
   missing space/permissions before mutation. Snapshot the relevant pre-patch save
   separately before native acceptance, without editing originals or shared profiles.
7. Apply only a purpose-built late patch, never reinstall a mid-stack component.
   Verify expected results and that no undeclared writes occurred; append the patch
   receipt only on success. Interrupted/failed writes block CEBG launch until verified
   recovery/restore. CEBG cannot prevent users launching the EXE outside the launcher.
8. Restore backups on failure; verify the restoration. Later rollback is offered only
   while current post-images/bookkeeping and dependent-patch state still match.
   Restoring game files cannot undo XP, quest progress or grants saved afterward.

## Text and dialog.tlk

TLK writes do not automatically mean "new run". The danger is changing the meaning
of existing string references, copying another installation's table/compiled IDs,
or removing text still referenced by saves/resources. A fresh campaign does not
repair a mismatched installed TLK.

A narrow description patch can use WeiDU to allocate/reuse the intended text in
the target installation and repoint the relevant resource field, preserving old
entries and unrelated references. Prefer adding/reusing text and changing intended
consumers over editing a shared existing entry. Resolve target language/female TLK
where applicable; back up and validate both resources and text, including old-entry
preservation, before allowing that adapter into the current-run category.

Mechanics and descriptions may be separate patch units when truly independent.
If only mechanics are supported, explicitly show "Gameplay fixed; in-game description
still shows the old text" and give the correct behavior in the patch details. Do not
silently ship misleading text or describe the entire component as updated. If a
simple local-text patch is available, prefer delivering both together. The first
pilot should avoid TLK writes; a later dedicated text adapter can demonstrate this
path without widening every patch's permissions.

## Original pilot sequence and minimum evidence

Historical checklist: apply/repeat/rollback evidence and the subsequent user live
acceptance are recorded separately above. Christopher did not explicitly report
the suggested new-slot save/reload sequence; do not promote it into a passed test.

1. Read-only inventory of the named RC, process paths and profile linkage.
2. Use the owner-maintained Artisan kit-description table repair (reuses existing
   local strings), subject to patch-local eligibility. Already-fixed inputs must
   return without invoking WeiDU. No new optional components are added by this pilot.
3. Define the exact inputs/write set, backup/restore and unsupported-input behavior.
4. Review the bounded design with requested Opus 5 xhigh, read-only and no delegation.
   Allow up to an hour initially; do not kill it merely at 5/15 minutes. A review
   does not substitute for execution evidence. Honor quota/model errors without fallback.
5. Exercise apply, repeat/no-op, unsupported/tampered input rejection and interrupted
   transaction restoration in fixtures. Then back up and patch only the named RC.
6. Restart/load/use/save/reload against an isolated test profile; compare relevant
   resources and rollback. Do not use shared production saves as the scratch profile.
7. Only after this, connect the supported Apply path to the Updates UI and publish
   that specific tested patch. Broader NPC/save migrations and quest updates remain
   separate work, not prerequisites for useful first fixes.

## Historical read-only intake result

At intake, the named RC was a plain directory with BG2EE 2.7.3.0. No process was
observed running from it. Its dedicated profile lived under redirected Documents
and required explicit test-profile isolation; that separate harness was completed
before the subsequent accepted test. The legacy replay receipt recorded 386 rows
and the observed pre-patch WeiDU.log had 394. Those were post-receipt changes, not
corruption or a reason to reject every local resource patch.

The first concrete candidate is the owning Artisan repo's committed
`live-patch/AKCB_KIT_DESCRIPTIONS` component 0, version 1.0 (release-source commit
`d16d35ba29c68aa18025a1c5323a8973986a97d9`). It reuses local KITLIST HELP references
and edits only campaign description-table links, not kit abilities or saves.
Read-only inspection found relevant stale BG1/SoD table links for the RC's installed
Assassin, Archer and Beast Master components; BG2's corresponding links already
matched. Those intake findings established relevance only; later application and
live acceptance are evidenced in the linked records, not inferred from inspection.

Before execution, verify all selected HELP references are valid against the actual
TLK and that the resource tables have the expected fields; the adapter itself only
checks that a returned reference is numeric/nonnegative. Capture actual table and
TLK pre-images, then prove only the intended table fields and normal declared WeiDU
bookkeeping change. No TLK text should change in this first pilot. Do not infer that
these description repairs deliver the newer versions' kit balance or abilities.

References: [prior component triage](2026-09-14-existing-install-update-triage.md),
`engine/src/updates.rs`, [TLK format](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/tlk_v1.htm),
[WeiDU documentation](https://weidu.org/WeiDU/README-WeiDU.html).

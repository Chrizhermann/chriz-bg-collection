# Chriz Easy IWD — curation

Started 2026-09-10. This is the IWDEE collection's curation workspace in the shared
source repository. Product name: **Chriz Easy IWD**. Proposed stable collection ID:
`chriz-easy-iwd` (not yet implemented in the app or recipe schema).

Follow the same process as the BG collection: mod worksheet, full versioned component
menus, Christopher's selections, explicit mapping into recipe features, and a separate
follow-up list for unresolved compatibility or delivery work.

## Working records

- [Mod worksheet](WORKSHEET.md): shared-mod intake and recorded direction.
- [Additional mods](ADDITIONAL-MODS.md): new NPC, quest and item candidates.
- [Follow-ups](FOLLOW_UPS.md): source, curation, implementation and acceptance tasks.
- [SCS research](SCS-RESEARCH.md): spoiler-free findings about existing caster coverage
  and the scope for conservative enhancements.
- [Collection design](../../plans/2026-09-08-iwdee-collection-direction.md): architecture,
  previous technical checks and agreed scope.

## Procedure

1. Inventory the reviewed BG collection's mods and exact source versions. Compare them
   with the current source/menu for each IWD candidate. Existing BG choices are a
   starting comparison, not automatic IWDEE selections.
2. For additional mods, first record mod-level interest, author/source, IWDEE and
   expansion support, acquisition route, likely compatibility and balance impact.
   After intake, use the same component procedure as for shared mods.
3. Enumerate full component menus from the exact source version using WeiDU's JSON
   listing in a scratch workspace. Verify IWDEE availability separately from listing:
   a listed component is not proof its runtime/install predicates can succeed.
4. Keep unreviewed inventories outside the authoritative `components/` catalogs. Show
   original component ID/name/group/subgroup, previous BG decision, IWD applicability,
   source/version and explicit review state. Do not turn unanswered choices into drops.
5. Review with Christopher in manageable batches. Record his choices without reordering
   into an agent-selected preset. Keep recommendations and compatibility findings in
   notes, separate from decisions. Preserve already settled decisions across refreshes.
6. Publish reviewed per-mod catalogs with the same normalized component semantics as
   BG, then map them explicitly to collection features, prerequisites and choice groups.
   Pending compatibility work stays recorded even when an intended feature is unavailable.
7. Resolve and verify a separate IWD install order from the selected sources. Build a
   self-contained versioned recipe, reconcile every selected outcome to its decision,
   then perform isolated installation and live acceptance checks.

## Component decision semantics

The reviewed component catalogs use the existing six-column format:
`# / Component / Group / Subgroup / checkmark / Decision`.

| Decision | Meaning |
|---|---|
| blank | Excluded; not offered in the installer. |
| `optional` | Offered, initially unchecked. |
| `default` | Offered, initially checked. |
| `mandatory` | Included whenever its parent mod is selected; no separate component toggle. |

Blank must not mean "pending". Pending entries remain in inventories/worksheets until
reviewed; those are not executable curation inputs. Mod-level direction does not select
all components or determine whether the parent mod itself is mandatory.

The checkmark column is evidence of installation in an explicitly identified reference
copy, never a substitute for a decision. There is no accepted IWD reference installation
yet. A BG reference checkmark must not appear as an IWD installed mark.

Keep availability and decisions separate: a desired component needing a port remains
desired with a technical follow-up. Mutually exclusive groups, dependencies, ordering
and unavailability reasons belong in their corresponding notes/mapping fields.

## Reuse and boundaries

- Reuse the parser/audit model in `tools/curation_audit.py`; it already takes a catalog
  directory and curation-map path. An IWD curation map and executable recipe do not exist yet.
- Reuse authoring helpers after reviewing their BG-specific assumptions. Do not run a
  BG recipe generator against unfinished IWD worksheets.
- Preserve existing BG catalogs and decisions. Store IWD catalogs and their own mapping
  under an explicit collection identity when implementation adds that support.
- Store source links and recipe metadata, not third-party archives or private game data.
- Report NPCs and quests without recruitment locations, story events or rewards.
  Encounter work is different since 2026-10-25: Christopher wants to see everything in
  the fight audit (enemy names, places, final boss) and designs the big strokes.

Current stage (2026-10-25): see the [v1 plan](../../plans/2026-10-25-chriz-easy-iwd-v1.md).
Neutral inventory, fight audit and the engine standalone mode are in progress.

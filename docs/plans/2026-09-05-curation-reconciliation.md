# CEBG: restore curation as the recipe authority

User-approved direction, 2026-09-05. Installer delivery remains the priority; do not reopen
all mod discussions or turn this into another mod-review marathon.

## What went wrong

`recipes/creator-full-current` was generated from 28 BG1 and 456 BG2 reference-log rows,
with selected maintained-source substitutions. Its single all-components feature produced
90 runs / 488 components without applying the curation worksheet or component decisions.
That historical replay is not a curated full collection and must not be offered or resumed
as one. Bristlelick's syntax error is incidental: the mod was explicitly dropped already.

The source catalogs were not lost. Do not ask Christopher to repeat the curation.
The failed root `C:\Users\chris\Games\CEBG-Full-20260905` and its frozen evidence are
historical test evidence only. Do not rewrite their identity or modify the stream install.

## Authoritative inputs

1. Later explicit user decisions and recorded release/curation updates take precedence.
2. `docs/curation-worksheet.md` contains mod-level keep/drop decisions, including Bristlelick,
   Wings and the separately offered voice packs. Per-component catalogs contain the detailed
   decisions and dependency notes. Preserve their semantics: blank = excluded; optional =
   unchecked; default = checked; mandatory = required when its parent is enabled.
3. `manifest/curation-map.toml` maps those decisions to features and explicit deferrals.
   Validate the mapping against the catalogs; the mapping does not override the decisions.
4. The pinned mod's source/menu defines the current component identities and prerequisites.
   A historical numeric ID is not proof that a new version supplies the same feature.
5. The old WeiDU logs provide provenance and a starting order, never an inclusion list.
   Do not hand-edit or regenerate `manifest/install-order.tsv` for this correction.

## Implementation sequence

- [x] Quarantine legacy replay authoring and its desktop profile with regression tests.
  Keep historical recipes/helpers/evidence; do not patch a blacklist into the generator and
  describe that as curation support. Existing alpha.3 binaries are not retroactively changed.
- [ ] Derive the complete installable selection from the curated feature/preset model.
  Build on `manifest/` and its existing feature choices, requirements, prompt handling and
  explicit unavailability. Do not return to a single unconditional all-components feature.
  The existing smaller public-alpha profile remains a limited profile, not a substitute
  to relabel as the completed full collection.
- [ ] Add the remaining approved, release-ready sources/components to that model. A private
  manually supplied archive may satisfy provenance/acquisition only; its contents cannot
  authorize components. Keep exclusions, readiness deferrals and private-source limitations
  separate. Retain optional components as choices without turning them on automatically.
- [ ] Produce a concrete reconciliation report: each curated default/mandatory outcome is
  installed, conditionally inactive, or explicitly deferred with its existing reason; each
  selected component maps back to a decision and the pinned version's semantic identity.
  Include unchanged selections as well as additions/removals. Do not treat old replacement
  patches as additional features or automatically equate missing IDs with missing effects.
- [ ] Resolve order from current mod documentation, pinned source prerequisites and reliable
  modder documentation where needed. Use old order only as a starting point. Preserve EET's
  BG1 preparation/import/main/finalization boundaries, authored late patches, and BuffBot last.
  Record only changed/ambiguous order decisions and their evidence, not a full research essay.
- [ ] Run the existing curation audit and focused resolved-selection regressions, then one
  new isolated full install. Do not resume the invalid frozen legacy selection or mutate its
  ledger. Keep downloads/cache reusable where their identities still match.
- [ ] Finish the existing practical acceptance list: downloads/retry, completed receipt and
  WeiDU consistency, packaged launcher, signed update cycle, Radar and bounded game smoke.

## Required regression examples from the failed recipe

- No Bristlelick, Wings, Ajantis BG2 or the 21 dropped voice/soundset installers.
- No historical BG2EE-EET-Fixpack parent, UB3, Bardic Wonders2001, Artisan30001, standalone
  Berserker replay, migration-only BG Rebalance409, or upstream EET Tweaks2042.
- SoD Remix selects all 30 approved v0.6.4 components, including the nine absent from the
  failed recipe: 187, 197, 215, 225, 245, 255, 260, 270, 280. Source order places210 before197.
- New default selections are preserved: UB13/14, Ascension60, CDTweaks2720/3121,
  Randomiser500, BG Rebalance100, and Artisan NPC9101/99001 where prerequisites are met.
- Garrick uses the recorded single assignment route (Bardic1008 prompt No, Artisan99001).
- Maintained modpack400 replaces the old Branwen Spiritual Hammer tail when applicable;
  do not falsely report its functionality missing merely because the old identity existed.
- Never select both restored-Bhaalspawn-power providers while that decision is deferred.
- Source-version compatibility and approved later decisions override stale numeric labels.

## Deferred mod work: do not block the installer

Christopher explicitly deferred reviewing Aura and the other unresolved mod choices until
the installer works. Preserve their entries/reasons and keep them unavailable or unchecked
as already documented; do not silently promote them to ready or drop their follow-ups.
Do not expand this turn into implementations in owning mod repositories.

## Verification of the initial correction

- Curation map audit passed for all 1,181 catalog rows: 583 excluded, 158 optional,
  299 default, 141 mandatory. This verifies authored coverage, not full install acceptance.
- All 35 native command-contract tests passed, including the new regression preventing
  a historical full recipe from being offered/selected merely because its files exist.
- All 25 Python tooling tests passed, including rejection before any legacy generator
  source access or output creation. The CLI returns a controlled error without a traceback.
- The corrected full recipe has not yet been generated, and no new install was started.
  The alpha.3 setup binary has not been rebuilt by this initial correction.

### Progressive XP replacement (annotation clarification)

This is the replacement for **EET Tweaks2042**, not the historical BG2EE-EET-Fixpack.
It is not implemented/released/pinned. The recorded intention is sensible BG1 rewards for
traps, learning spells and lockpicking, full BG2 rewards in the BG2 portion, and a working
transition/scaling mechanism. No exact numeric schedule is recorded; this is not approval
for a blanket kill/quest-XP multiplier.

Keep issue https://github.com/Chrizhermann/chriz-bg-collection/issues/1 and the curation
follow-up open. The owning mod repo remains to be settled (a balance-layer implementation,
not gameplay code absorbed into the installer). Once designed, implemented, tested and
released, the collection should offer it optional/default-checked. Upstream2042 remains out.

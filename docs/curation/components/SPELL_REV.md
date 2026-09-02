# SPELL_REV — components

Listed at installed version **v4.19**. Upstream **v4.21** and the intended fork
**v4.21-chriz.1** retain the same eight-component menu. The fork contains the current
protection-refresh fix but has no release tag yet, so `manifest/mod-sources.tsv` still
targets upstream v4.21 pending a fetchable fork release.
8 entries, 7 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Whenever the parent Spell Revisions mod is selected, component `0` is included without
  a separate component toggle.
- Component `60` requires component `0` and must run after every selected NPC mod and
  kit/class assignment. Its captured reference position is not late enough for the
  future generated order.
- SCS component `4240` is unavailable while Spell Revisions is selected; that rule is
  recorded in the SCS catalog.
- Known cross-catalog follow-up: while Spell Revisions is selected, Artisan's Kitpack
  components `8101`/`8102`, Artisan's Kitpack NPC components `5102`/`10004`, and Bardic
  Wonders component `1006` must be unavailable when those catalogs are reviewed.
- The separate local `SR_SUBSPELL_FIX` component `0` is automatically included while
  normal all-on Spell Revisions is selected and unavailable when Spell Revisions is off.
  If granular SR features are exposed later, gate the fix on the elemental-protection
  feature rather than merely on SR component `0`.
- Components `20`, `30`, `55`, `60`, and `65` were marked highly recommended during
  curation; `default` carries their selection state without overloading the Decision
  value.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Spell Revisions |  |  | ✓ | mandatory |
| 10 | Deva and Planetar Animations |  |  | ✓ | default |
| 20 | Mirror Image Fix |  |  | ✓ | default |
| 30 | Dispel Magic Fix |  |  | ✓ | default |
| 55 | Spell Deflection blocks AoE spells |  |  | ✓ | default |
| 60 | Update Spellbooks of Joinable NPCs |  |  | ✓ | default |
| 65 | Revised Warrior HLAs |  |  | ✓ | default |
| 70 | Revised Saving Throws |  |  |  | |

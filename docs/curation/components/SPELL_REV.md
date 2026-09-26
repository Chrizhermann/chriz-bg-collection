# SPELL_REV — components

The captured install used **v4.19**. The production recipe now pins the immutable
**v4.21-chriz.5** successor release, including the approved Lightning Bolt alternatives
and the reviewed protection-refresh and hidden-subspell repairs.
10 entries, 7 historically installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Whenever the parent Spell Revisions mod is selected, component `0` is included without
  a separate component toggle.
- Component `60` requires component `0` and runs after every selected NPC mod and
  kit/class assignment, before the Lightning Bolt run and final BuffBot run.
- SCS component `4240` is unavailable while Spell Revisions is selected; that rule is
  recorded in the SCS catalog.
- While Spell Revisions is selected, Artisan's Kitpack `8101` and Artisan's Kitpack NPC
  `10004` retain their existing exclusions. The unsupported Red Wizard Edwin `5102`
  exclusion was removed on 2026-09-10: it is default-checked with SR. Independent
  source review found spellbook cleanup imperfections, not a reason to block the kit;
  see `../../handoffs/2026-09-10-red-wizard-sr-default.md`.
- The former standalone `SR_SUBSPELL_FIX` is retired: its maintained implementation is
  part of component `60`, so no legacy tail run is authored.
- The [September 16 candidate](../../alpha16-candidate-2026-09-16.md) approved component
  `80` (classic bouncing Lightning Bolt) by default, with `81` (non-bouncing spells and
  wands, retaining bouncing traps) as its unchecked, mutually exclusive alternative.
  Both require core `0` and run after NPC spellbook processing, before BuffBot.
- The same approval includes the separate [SR/RR compatibility installer](SRCB_RR_COMPAT.md),
  automatically dependent on SR and either selected RR encounter, before SCS.
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
| 80 | Classic bouncing Lightning Bolt |  | Lightning Bolt behavior |  | default |
| 81 | Non-bouncing Lightning Bolt |  | Lightning Bolt behavior |  | optional |

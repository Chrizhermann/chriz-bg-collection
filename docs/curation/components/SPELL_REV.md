# SPELL_REV — components

The captured install used **v4.19**. The production recipe now pins the immutable
**v4.21-chriz.3** successor release, which retains the same eight-component menu and
contains the reviewed protection-refresh and hidden-subspell repairs.
10 catalog entries: 8 released and 2 development options; 7 installed.
✓ = installed. Subgroup = choose one.

## UI/dependency notes

- The Lightning Bolt selector has three states: **Spell Revisions** (unchanged
  default; install neither `80` nor `81`), **Original** (`80`), and **Non-bouncing**
  (`81`). Both development components are optional, mutually exclusive and blocked
  until a verified successor release includes them; they are not part of the pinned
  eight-component menu. `81` affects the wizard spell and Wand of Lightning, with
  path hits/friendly fire retained and traps unchanged. Do not substitute CDTweaks
  `2530`, whose shared-projectile change includes traps. Ordering and release gates:
  [integration handoff](../../handoffs/2026-09-07-lightning-bolt-options.md).

- Whenever the parent Spell Revisions mod is selected, component `0` is included without
  a separate component toggle.
- Component `60` requires component `0` and runs after every selected NPC mod and
  kit/class assignment, immediately before BuffBot in the current recipe.
- SCS component `4240` is unavailable while Spell Revisions is selected; that rule is
  recorded in the SCS catalog.
- While Spell Revisions is selected, Artisan's Kitpack `8101` and Artisan's Kitpack NPC
  `5102`/`10004` are unavailable. The conflict edges are applied with the Artisan recipe
  slice; this isolated pin predates those feature definitions.
- The former standalone `SR_SUBSPELL_FIX` is retired: its maintained implementation is
  part of component `60`, so no legacy tail run is authored.
- Spell Revisions/Rogue Rebalancing compatibility remains deferred and does not block
  either mod in this alpha.
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
| 80 | Original | Spells | Lightning Bolt |  | optional |
| 81 | Non-bouncing | Spells | Lightning Bolt |  | optional |

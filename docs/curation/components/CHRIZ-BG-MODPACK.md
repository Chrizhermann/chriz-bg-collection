# CHRIZ-BG-MODPACK — components

Refreshed for public release **v0.2.0-alpha.1**.
34 current or historical entries; the public installer exposes 18 and the collection selects
17 because component `195` already includes the work from standalone component `160`.
The four ✓ marks record the old reference install, not the new default recipe.

## Implementation status

- The public alpha implements `110`, `130`, `140`, `160`, `170`, `190`, `192`–`198`,
  `400`, `410`, `430`, `440`, and `450`. It contains no advertised FAIL placeholders.
- The collection exposes every implemented component except standalone `160`, whose exact
  skill repair is invoked atomically by the default Skie component `195`.
- Components `140`, `170`, `400`, and `430` remain visible but unavailable until their
  respective Artisan, Spell Revisions, or Tweaks Anthology prerequisites enter the recipe.
- Components `120`, `200`, `210`, and `300` are retired/superseded. The other historical
  numbers below remain excluded; component `600` is intentionally outside the fresh preset.
- See [Collection tail fixes](COLLECTION_TAIL_FIXES.md) for the complete mapping of all
  22 historical local-fix installers, their activation rules, and the items that live
  outside the modpack or do not yet have a maintained home.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 100 | Aura + Safana: add Bard spells to CRE | chriz-bg-modpack: NPC fixes |  |  | |
| 110 | Fade NPC: Fighter/Thief multiclass conversion | chriz-bg-modpack: NPC fixes |  |  | default |
| 120 | Fade NPC: proficiency pips + amulet usability | chriz-bg-modpack: NPC fixes |  |  | |
| 130 | BG1NPC Kivan Sea Elf Quest — SCS compatibility fix | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 140 | Mazzy: cap Short Bow at 4 pips, redistribute to Short Sword | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 150 | Safana: remove Thief Set Snare (SPCL412) from CRE | chriz-bg-modpack: NPC fixes |  |  | |
| 160 | Skie: redistribute Move Silently to Open Locks (Swashbuckler fix) | chriz-bg-modpack: NPC fixes |  |  | |
| 170 | Xan Eldritch Knight for EET (XAN_/4/6/TTXAN) | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 180 | Remove SR hidden subspells from joinable NPC spellbooks | chriz-bg-modpack: NPC fixes |  |  | |
| 190 | Sarah: Archer conversion and curated proficiencies | chriz-bg-modpack: NPC fixes |  |  | default |
| 192 | Viconia: permanent Cleric/Thief conversion | chriz-bg-modpack: NPC fixes |  |  | default |
| 193 | Shar-Teel: Wizard Slayer kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 194 | Kagain: Dwarven Defender kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 195 | Skie: Swashbuckler kit plus compatible thief skills | chriz-bg-modpack: NPC fixes |  |  | default |
| 196 | Faldorn: Avenger kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 197 | Dynaheir: learn the installed Haste spell | chriz-bg-modpack: NPC fixes |  |  | default |
| 198 | Kivan: Archer kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 200 | Artisan Kitpack: fix missing multiclass proficiency stats | chriz-bg-modpack: Kit fixes |  |  | |
| 210 | Druid/Ranger: remove Elemental Prince Call (SPPR724) from CLAB tables | chriz-bg-modpack: Kit fixes |  |  | |
| 300 | Red Bearskin Mail: Rashemi Berserker kit usability fix | chriz-bg-modpack: Item fixes |  |  | |
| 310 | Edwin Amulet (MISC89): remove bonus spell slots | chriz-bg-modpack: Item fixes |  |  | |
| 400 | SR Branwen Spiritual Hammer (SPIN113): fix param1 for Create Weapon | chriz-bg-modpack: Spell fixes |  |  | mandatory |
| 410 | Dispel Magic (Yeslick + Keldorn): friendly-fire + Yeslick cap fix | chriz-bg-modpack: Spell fixes |  |  | mandatory |
| 420 | SPWI910 Imprisonment: extend duration via maze-reuse hack (BINARY REPLACE — install LAST) | chriz-bg-modpack: Spell fixes |  |  | |
| 430 | Use Any Item: scrolls cast at a fair caster level for non-casters (CDTweaks #2170 fix) | chriz-bg-modpack: Spell fixes |  | ✓ | mandatory |
| 440 | Ascension Improved Slayer Transformation: repair the EE Fixpack branch (fizzled transformation kills the player) | chriz-bg-modpack: Spell fixes |  | ✓ | mandatory |
| 450 | SCS shapeshift spell tweak: restore the five forms broken by EE Fixpack (Mind Flayer/Iron Golem/Giant Troll/Spider/Mustard Jelly) | chriz-bg-modpack: Spell fixes |  | ✓ | mandatory |
| 500 | NPC kit changes (Khalid/Sirene/Skie/Sarah/etc.) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 510 | Historical Ajantis BG2 kit-transfer migration slot (do not install) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 511 | Historical Branwen BG2 kit-transfer migration slot (do not install) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 512 | Historical EEex NPC stat-transfer migration slot (redesign required) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 513 | Historical evil-NPC reputation migration slot (superseded by CDTweaks `3121`) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 514 | Historical Edwin spell-slot migration slot (audit required) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 600 | Disable UB "Cat and Mouse" Bodhi hunts in the Spellhold maze (and fix the ultimatum deadlock under SCS) | chriz-bg-modpack: Gameplay tweaks |  | ✓ | |

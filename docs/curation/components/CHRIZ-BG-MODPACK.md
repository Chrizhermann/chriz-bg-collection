# CHRIZ-BG-MODPACK — components

Listed at installed version **v0.1.0**.
26 entries, 4 installed. ✓ = installed. Subgroup = choose one.

## Implementation status

- Current maintained source implements only `430`, `440`, `450`, and `600`.
- Components `100`–`420`, `500`, and `510`–`514` are explicit FAIL stubs and must stay
  unavailable until their migrations are implemented and reviewed.
- Component `120` is redundant with the full Fade Fighter/Thief conversion and should be
  retired. Components `200`, `210`, and `300` appear superseded upstream and should be
  retired after fresh-install verification rather than reimplemented.
- See [Collection tail fixes](COLLECTION_TAIL_FIXES.md) for the complete mapping of all
  22 historical local-fix installers, their activation rules, and the items that live
  outside the modpack or do not yet have a maintained home.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 100 | Aura + Safana: add Bard spells to CRE | chriz-bg-modpack: NPC fixes |  |  | |
| 110 | Fade NPC: Fighter/Thief multiclass conversion | chriz-bg-modpack: NPC fixes |  |  | |
| 120 | Fade NPC: proficiency pips + amulet usability | chriz-bg-modpack: NPC fixes |  |  | |
| 130 | BG1NPC Kivan Sea Elf Quest — SCS compatibility fix | chriz-bg-modpack: NPC fixes |  |  | |
| 140 | Mazzy: cap Short Bow at 4 pips, redistribute to Short Sword | chriz-bg-modpack: NPC fixes |  |  | |
| 150 | Safana: remove Thief Set Snare (SPCL412) from CRE | chriz-bg-modpack: NPC fixes |  |  | |
| 160 | Skie: redistribute Move Silently to Open Locks (Swashbuckler fix) | chriz-bg-modpack: NPC fixes |  |  | |
| 170 | Xan Eldritch Knight for EET (XAN_/4/6/TTXAN) | chriz-bg-modpack: NPC fixes |  |  | |
| 180 | Remove SR hidden subspells from joinable NPC spellbooks | chriz-bg-modpack: NPC fixes |  |  | |
| 200 | Artisan Kitpack: fix missing multiclass proficiency stats | chriz-bg-modpack: Kit fixes |  |  | |
| 210 | Druid/Ranger: remove Elemental Prince Call (SPPR724) from CLAB tables | chriz-bg-modpack: Kit fixes |  |  | |
| 300 | Red Bearskin Mail: Rashemi Berserker kit usability fix | chriz-bg-modpack: Item fixes |  |  | |
| 310 | Edwin Amulet (MISC89): remove bonus spell slots | chriz-bg-modpack: Item fixes |  |  | |
| 400 | SR Branwen Spiritual Hammer (SPIN113): fix param1 for Create Weapon | chriz-bg-modpack: Spell fixes |  |  | |
| 410 | Dispel Magic (Yeslick + Keldorn): friendly-fire + Yeslick cap fix | chriz-bg-modpack: Spell fixes |  |  | |
| 420 | SPWI910 Imprisonment: extend duration via maze-reuse hack (BINARY REPLACE — install LAST) | chriz-bg-modpack: Spell fixes |  |  | |
| 430 | Use Any Item: scrolls cast at a fair caster level for non-casters (CDTweaks #2170 fix) | chriz-bg-modpack: Spell fixes |  | ✓ | |
| 440 | Ascension Improved Slayer Transformation: repair the EE Fixpack branch (fizzled transformation kills the player) | chriz-bg-modpack: Spell fixes |  | ✓ | |
| 450 | SCS shapeshift spell tweak: restore the five forms broken by EE Fixpack (Mind Flayer/Iron Golem/Giant Troll/Spider/Mustard Jelly) | chriz-bg-modpack: Spell fixes |  | ✓ | |
| 500 | NPC kit changes (Khalid/Sirene/Skie/Sarah/etc.) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 510 | Historical Ajantis BG2 kit-transfer migration slot (do not install) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 511 | Historical Branwen BG2 kit-transfer migration slot (do not install) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 512 | Historical EEex NPC stat-transfer migration slot (redesign required) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 513 | Historical evil-NPC reputation migration slot (superseded by CDTweaks `3121`) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 514 | Historical Edwin spell-slot migration slot (audit required) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 600 | Disable UB "Cat and Mouse" Bodhi hunts in the Spellhold maze (and fix the ultimatum deadlock under SCS) | chriz-bg-modpack: Gameplay tweaks |  | ✓ | |

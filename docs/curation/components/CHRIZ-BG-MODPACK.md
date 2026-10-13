# CHRIZ-BG-MODPACK — components

Refreshed for the October 7 target **v0.2.0-alpha.9**, retaining the choices
approved in the [September 16 candidate](../../alpha16-candidate-2026-09-16.md).
Rows `237`, `238`, `650`, `651` and `660` and the reworked `430` come from the
unpublished **v0.2.0-alpha.11** candidate (October 13 intake, see
[next batch intake](../../plans/2026-10-13-next-batch-intake.md)); the recipe pin
moves to that release only when it is published.
55 current or historical entries; the collection exposes 38. Component `195` already includes the work from standalone component
`160`; Hexxat's three class routes are one at-most-one choice.
The four ✓ marks record the old reference install, not the new default recipe.

## Implementation status

- Baeloth `235` is optional: legal Sorcerer spells known with or without SR,
  applied after SR's NPC spellbooks (`60`). It does not enable a full Challenge Mode.
- Edwin `236` is default only with Artisan's Red Wizard assignment `5102`.
  It runs immediately after that assignment, before SR NPC spellbooks `60`,
  and makes the amulet removable while leaving extra slots to the kit.
  Other Edwin kits and saved actors are not rewritten by this fresh-install option.

- Evandra `224` (Sorcerer) and `225` (owner-created portrait) are released and
  pinned independent defaults requiring Evandra core. They run after class and
  NPC providers, before continuity and EET_end; the generator and packaging
  check enforce the delivery contract. See
  [Evandra handoff](../../handoffs/2026-09-29-evandra-default-restoration.md).
- The public alpha implements `110`, `130`, `140`, `160`, `170`, `190`, `192`–`198`,
  `188`, `189`, `199`, `220`–`225`, `400`, `410`, `430`, `440`, `450`, `610`, `611`, `620`, `630`, and `640`. It contains no advertised
  FAIL placeholders. The alpha.11 candidate adds `237`, `238`, `650`, `651` and `660`.
- The collection exposes every implemented component except standalone `160`, whose exact
  skill repair is invoked atomically by the default Skie component `195`.
- Component `400` remains visible but conditionally inactive unless its Branwen /
  Spell Revisions prerequisites are selected.
- Scrolls `430` (alpha.11) is default and no longer uses Tweaks Anthology `2170`: it
  requires Windows EEex and sets the casting level for every scroll reader (the reader's
  arcane or divine caster level; other readers, such as Use Any Item users, half their
  highest class level, minimum 1). At install time it hashes every scroll and the spells
  they cast, so it runs in its own final run `chriz-bg-modpack-scrolls-bg2`, after BuffBot
  and every other spell/item writer.
- Fade `237` and Evandra `238` are default whenever Fade or Evandra is selected: at the
  first SoA recruitment they raise the companion to the highest protagonist XP threshold
  reached (125,000 to 1,250,000), never lower XP and pay once. They run in the
  late-companion run in TP2 order, `189, 237, 238, 620`.
- Campaign XP caps `650` (BG1 161,000, SoD 500,000; excess banked) and the SoA cap `651`
  (2,950,000, requires `650`) are optional, unchecked and experimental. Both require EEex
  and run in their own run `chriz-bg-modpack-campaign-xp-bg2` right after the
  late-companion run.
- Degrodel `660` is default: refusing to hand over the Helm of Balduran makes Degrodel
  and his servants attack; handing it over keeps the normal reward. It runs in the main
  post-EET_end Modpack run after `630`.
- Kivan quest guard `130` is mandatory with BG1NPC Banters/Quests/Interjections `10`
  and SCS general AI `6000`, after EET import and SCS. It is independent of Kivan's
  Archer choice `198`; never add standalone `KIVAN_QUEST_FIX` alongside it. Its
  default inclusion was explicitly reconfirmed on 2026-09-09.
- Yoshimo `220` is a default companion conversion. Hexxat `221` (Shadowdancer) is the
  default; `222` and `223` are unchecked alternatives, mutually exclusive with each other
  and with Artisan's Invisible Blade component `7104`.
- Story-stage utility XP `611` is the default; unchanged party-progress curve `610`
  remains optional. They are mutually exclusive, and either or neither can be chosen.
  Both require EEex and run after EET_end and earlier reward/XP tweaks; `611` requires EET.
- Unidentified appearance `630` is optional/off-default, Windows/EEex only, after
  item/art writers. Only Randomiser's appearance `570` conflicts; ordinary loot
  randomization remains compatible.
- Legacy text helper `640` is mandatory only when LeUI or Dragonspear is selected,
  after the final full UI and before Bubb. It has no HGO, EEex or optional Modpack
  gameplay-parent dependency. See [game interfaces](../game-interfaces.md).
- Yeslick `188` is automatic only with Yeslick NPC's Fighter/Alaghor component `1`;
  the vanilla class choice omits it. It runs with the other companion build changes
  before continuity `199`, which immediately precedes EET_end.
- Continuity `199` is default-checked and preserves Xan/Yeslick progression across EET,
  including vanilla class choices. It requires EET and is for new installations;
  it does not migrate saved actors.
- Safana `189` is automatic with official Safana core `0`, after that core and the
  final arrival-script writers. It clears inherited inventory once at first SoA arrival,
  preserving later equipment, and does not change her class or kit.
- Imoen `620` is default-checked, requires EEex, and runs in the late-companion slot
  before Spell Revisions spellbook processing. At first Spellhold recruitment it raises
  mage XP to the party average, capped at 3 million, without repeating on rejoin.
- Components `120`, `200`, `210`, and `300` are retired/superseded. The other historical
  numbers below remain excluded; component `600` is intentionally outside the fresh preset.
- See [Collection tail fixes](COLLECTION_TAIL_FIXES.md) for the complete mapping of all
  22 historical local-fix installers, their activation rules, and the items that live
  outside the modpack or do not yet have a maintained home.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 235 | Baeloth: legal Sorcerer spells known | chriz-bg-modpack: NPC fixes |  | | optional |
| 236 | Edwin Red Wizard: removable amulet and corrected spell slots | chriz-bg-modpack: NPC fixes |  | | default |
| 100 | Aura + Safana: add Bard spells to CRE | chriz-bg-modpack: NPC fixes |  |  | |
| 110 | Fade NPC: Fighter/Thief multiclass conversion | chriz-bg-modpack: NPC fixes |  |  | default |
| 120 | Fade NPC: proficiency pips + amulet usability | chriz-bg-modpack: NPC fixes |  |  | |
| 130 | BG1NPC Kivan Sea Elf Quest — SCS compatibility fix | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 140 | Mazzy: cap Short Bow at 4 pips, redistribute to Short Sword | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 150 | Safana: remove Thief Set Snare (SPCL412) from CRE | chriz-bg-modpack: NPC fixes |  |  | |
| 160 | Skie: redistribute Move Silently to Open Locks (Swashbuckler fix) | chriz-bg-modpack: NPC fixes |  |  | |
| 170 | Xan Eldritch Knight for EET (XAN_/4/6/TTXAN) | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 180 | Remove SR hidden subspells from joinable NPC spellbooks | chriz-bg-modpack: NPC fixes |  |  | |
| 188 | Yeslick: apply the selected Alaghor kit in BG1 | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 189 | Safana: clear carried-over arrival inventory once | chriz-bg-modpack: NPC fixes |  |  | mandatory |
| 190 | Sarah: Archer conversion and curated proficiencies | chriz-bg-modpack: NPC fixes |  |  | default |
| 192 | Viconia: permanent Cleric/Thief conversion | chriz-bg-modpack: NPC fixes |  |  | default |
| 193 | Shar-Teel: Wizard Slayer kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 194 | Kagain: Dwarven Defender kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 195 | Skie: Swashbuckler kit plus compatible thief skills | chriz-bg-modpack: NPC fixes |  |  | default |
| 196 | Faldorn: Avenger kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 197 | Dynaheir: learn the installed Haste spell | chriz-bg-modpack: NPC fixes |  |  | default |
| 198 | Kivan: Archer kit | chriz-bg-modpack: NPC fixes |  |  | default |
| 199 | Xan and Yeslick: preserve progression across EET | chriz-bg-modpack: NPC fixes |  |  | default |
| 220 | Yoshimo: Swashbuckler | chriz-bg-modpack: NPC fixes |  |  | default |
| 221 | Shadowdancer | chriz-bg-modpack: NPC fixes | Hexxat: choose a class or kit |  | default |
| 222 | Fighter/Thief multiclass | chriz-bg-modpack: NPC fixes | Hexxat: choose a class or kit |  | optional |
| 223 | Assassin | chriz-bg-modpack: NPC fixes | Hexxat: choose a class or kit |  | optional |
| 224 | Evandra: Sorcerer build and starting spellbook | chriz-bg-modpack: NPC fixes | | | default |
| 225 | Evandra: custom portrait | chriz-bg-modpack: NPC fixes | | | default |
| 200 | Artisan Kitpack: fix missing multiclass proficiency stats | chriz-bg-modpack: Kit fixes |  |  | |
| 210 | Druid/Ranger: remove Elemental Prince Call (SPPR724) from CLAB tables | chriz-bg-modpack: Kit fixes |  |  | |
| 300 | Red Bearskin Mail: Rashemi Berserker kit usability fix | chriz-bg-modpack: Item fixes |  |  | |
| 310 | Edwin Amulet (MISC89): remove bonus spell slots | chriz-bg-modpack: Item fixes |  |  | |
| 400 | SR Branwen Spiritual Hammer (SPIN113): fix param1 for Create Weapon | chriz-bg-modpack: Spell fixes |  |  | mandatory |
| 410 | Dispel Magic (Yeslick + Keldorn): friendly-fire + Yeslick cap fix | chriz-bg-modpack: Spell fixes |  |  | mandatory |
| 420 | SPWI910 Imprisonment: extend duration via maze-reuse hack (BINARY REPLACE — install LAST) | chriz-bg-modpack: Spell fixes |  |  | |
| 430 | Scrolls: use the reader's active caster level, or half their active class level | chriz-bg-modpack: Spell fixes |  | ✓ | default |
| 440 | Ascension Improved Slayer Transformation: repair the EE Fixpack branch (fizzled transformation kills the player) | chriz-bg-modpack: Spell fixes |  | ✓ | mandatory |
| 450 | SCS shapeshift spell tweak: restore the five forms broken by EE Fixpack (Mind Flayer/Iron Golem/Giant Troll/Spider/Mustard Jelly) | chriz-bg-modpack: Spell fixes |  | ✓ | mandatory |
| 500 | NPC kit changes (Khalid/Sirene/Skie/Sarah/etc.) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 510 | Historical Ajantis BG2 kit-transfer migration slot (do not install) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 511 | Historical Branwen BG2 kit-transfer migration slot (do not install) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 512 | Historical EEex NPC stat-transfer migration slot (redesign required) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 513 | Historical evil-NPC reputation migration slot (superseded by CDTweaks `3121`) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 514 | Historical Edwin spell-slot migration slot (audit required) | chriz-bg-modpack: Meta / multi-fix |  |  | |
| 600 | Disable UB "Cat and Mouse" Bodhi hunts in the Spellhold maze (and fix the ultimatum deadlock under SCS) | chriz-bg-modpack: Gameplay tweaks |  | ✓ | |
| 610 | Progressive utility XP for locks, traps, and learning spells | chriz-bg-modpack: Gameplay tweaks | Utility XP scaling | | optional |
| 611 | Story-stage utility XP for locks, traps, and learning spells | chriz-bg-modpack: Gameplay tweaks | Utility XP scaling | | default |
| 620 | Imoen: bring mage XP up to the party average at Spellhold | chriz-bg-modpack: Gameplay tweaks |  |  | default |
| 237 | Fade: first-recruitment SoA XP catch-up (new recruitment actors) | chriz-bg-modpack: Gameplay tweaks |  |  | default |
| 238 | Evandra: first-recruitment SoA XP catch-up (new recruitment actors) | chriz-bg-modpack: Gameplay tweaks |  |  | default |
| 630 | Hide unidentified item appearance | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 650 | Campaign XP caps - keep excess XP for later (BG1/SoD; experimental) | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 651 | Campaign XP caps - also cap SoA until ToB (requires 650; experimental) | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 660 | Degrodel: hand over the Helm of Balduran or keep it and fight him | chriz-bg-modpack: Gameplay tweaks | | | default |
| 640 | Legacy UI translation helper compatibility | chriz-bg-modpack: Compatibility | | | mandatory |

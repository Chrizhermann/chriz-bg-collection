# CHRIZ-BG-MODPACK — components

Refreshed for the October 7 target **v0.2.0-alpha.9**, retaining the choices
approved in the [September 16 candidate](../../alpha16-candidate-2026-09-16.md).
Rows `237`, `238`, `650`, `651` and `660` and the reworked `430` came with
**v0.2.0-alpha.11** (October 13 intake, see
[next batch intake](../../plans/2026-10-13-next-batch-intake.md)). **v0.2.0-alpha.12**
(2026-10-14) added the companion stat rows `230`–`234`. **v0.2.0-alpha.13**
(2026-10-20) added EET join XP `239`, Dorn's ambush fix `135`, the party AI fix `460` and
chapter XP caps `652`, and made campaign XP banking `650`/`651` work. **v0.2.0-alpha.14**
(published 2026-10-26) keeps scrolls `430` on for the whole session and stops `650` from
losing script variables; it adds and removes no component. With the alpha.14 pin the
collection also offers `135`, `460` and `652` (see below). The recipe now pins
**v0.2.0-alpha.15** (published 2026-11-03), which adds only Marek's Boomerang Dagger `670`,
a default for 0.1.1 (see below).
The recipe now pins **v0.2.0-alpha.16** (published 2026-11-08, annotated tag on public-main
squash commit `ee82f6b` of Modpack PR #19, built by the tagged release workflow). It keeps the
same components, order and Recommended choices. Every component writes the result it decides
regardless of edited game values, another mod's version or missing owned pieces; the bundled XP
table is copied by its tracked name `xplevel.2da`, so a case-sensitive WeiDU finds it. Fresh
source/package evidence is
`manifest/evidence/release-intake-20261108-chriz-bg-modpack-chriz-easy-bg.toml`, not a WeiDU
installation or gameplay acceptance.
The recipe now pins **v0.2.0-alpha.17** (published 2026-11-22, annotated tag on public-main
commit `5d93338` of Modpack PRs #20 and #21, built by the tagged release workflow). It adds
component `680` (EET story-consistent campaign boundaries and SoD backtracking fixes; optional,
on only with Challenge Mode)
after `670` and before `650` while keeping every earlier component and choice. Fresh source/package
evidence is `manifest/evidence/release-intake-20261122-chriz-bg-modpack-chriz-easy-bg.toml`,
not a full collection WeiDU installation or gameplay acceptance.
The recipe now pins **v0.2.0-alpha.18** (published 2026-11-22 21:13 UTC, annotated tag on public-main
commit `533913c` of Modpack PRs #24 and #25, built by the tagged release workflow). It adds
default component `470` (RAW specialist spell-scroll learning fix, requiring EEex) after `460`
in both Chriz Easy BG and Chriz Easy IWD (`470` is the sole Modpack component in IWD). Modpack
alpha.18 also ships component `690` (`cbm_fast_traps`, integrated in Modpack PR #23 with its
typed-write fix in PR #25), which remains unoffered and off in both installers until explicitly
approved. Fresh source/package evidence is
`manifest/evidence/release-intake-20261123-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml`,
not a combined collection WeiDU installation or full campaign gameplay.
The recipe now pins **v0.2.0-alpha.19** (published 2026-11-26 15:26 UTC, annotated tag on public-main
commit `6e6b364` of Modpack PR #26, built by the tagged release workflow). It adds default component
`695` (`cbm_bg1_early_party_xp`, BG1 early-party XP boost, requiring EEex) after `680` in Chriz
Easy BG only and omits it in Chriz Easy IWD (`470` stays the sole Modpack component in IWD; `690`
stays unoffered and off in both installers). Fresh source/package evidence is
`manifest/evidence/release-intake-20261127-chriz-bg-modpack-chriz-easy-bg-chriz-easy-iwd.toml`,
not a combined collection WeiDU installation or full campaign gameplay.
68 current or historical entries; the collection exposes 51. Component `195` already includes the work from standalone component
`160`; Hexxat's three class routes are one at-most-one choice.
The four ✓ marks record the old reference install, not the new default recipe.

## Implementation status

- Baeloth `235` is optional: legal Sorcerer spells known with or without SR,
  applied after SR's NPC spellbooks (`60`). It does not enable a full Challenge Mode.
- Edwin `236` is default only with Artisan's Red Wizard assignment `5102`.
  It runs immediately after that assignment, before SR NPC spellbooks `60`,
  and makes the amulet removable while leaving extra slots to the kit.
  Other Edwin kits and saved actors are not rewritten by this fresh-install option.

- Companion stats `230`–`234` (alpha.12) are **default (Christopher's decision,
  2026-10-14)**: Rasaad DEX 17/CON 16, Dorn CON 16, Eldoth STR 17/CON 16/WIS 9/INT 14,
  Garrick DEX 17/INT 15 and Quayle CON 15. Each is independent and needs no class, kit
  or spell mod. They run at the end of the pre-continuity run, after `225` in TP2 order,
  so they follow the companion class/kit conversions and precede continuity `199` and
  EET_end, as the Modpack's companion stat guide requires. Stored HP and saved actors
  are unchanged.
- Evandra `224` (Sorcerer) and `225` (owner-created portrait) are released and
  pinned independent defaults requiring Evandra core. They run after class and
  NPC providers, before continuity and EET_end; the generator and packaging
  check enforce the delivery contract. See
  [Evandra handoff](../../handoffs/2026-09-29-evandra-default-restoration.md).
- The public alpha implements `110`, `130`, `140`, `160`, `170`, `190`, `192`–`198`,
  `188`, `189`, `199`, `220`–`225`, `400`, `410`, `430`, `440`, `450`, `610`, `611`, `620`, `630`, and `640`. It contains no advertised
  FAIL placeholders. Alpha.11 added `237`, `238`, `650`, `651` and `660`; alpha.12 added
  `230`–`234`; alpha.13 added `135`, `239`, `460` and `652`; alpha.14 added none;
  alpha.15 adds `670`; alpha.16 added none; alpha.17 adds `680`; alpha.18 adds offered default `470` (while `690` remains unoffered/off); alpha.19 adds offered default `695` in Chriz Easy BG (while `690` stays unoffered/off and IWD keeps `470` only).
- BG1 early-party XP boost `695` (alpha.19) is **default (Christopher's decision, 2026-11-26)** in Chriz Easy BG only (never offered/installed in IWD) and requires EEex and EET. For party-divided XP routed through `CInfGame::AddPartyXP` in BG1 (`STATE_DEAD` clear count `N`), it adds `+70%` per additional living companion to the party XP pool (`1.0x` solo, `1.7x` duo, `2.4x` trio, `3.1x` four, `3.8x` five, `4.5x` six -> `100%` down to `75%` per member) until each living character reaches `16,000` total XP (sum of active XP and any retired dual-class threshold / Challenge Mode banked old-class XP for both `Player1` and companions), clipping only the bonus at `16,000` without lowering normal `1/N` shares. Verified offline with real `EEex_PreprocessAssembly` + Keystone/Unicorn emulation and synthetic WeiDU 249 install/uninstall, and verified live in quiet labs (`2026-11-27`) across standalone `BG:EE 2.7.3.0` (`6/6`) and `EET 2.7.3.0` (`21/21`: `N = 2..6`, dead slot `1`, dual-class `XPLEVEL.2DA`, SoD/`CHAPTER` gates, and Challenge Mode `650`/`652` coexistence) with `EEex v1.3.1`; full combined-collection WeiDU installation and full campaign playthrough remain separate.
- Spell scroll learning RAW fix `470` (alpha.18) is **default (Christopher's decision, 2026-11-23)** in both Chriz Easy BG and Chriz Easy IWD and requires EEex. Restricts the ±15% specialist scribing modifier (`opcode 147`) to Mage-class characters carrying one or more of the 8 wizard specialist bits (`0x0040`–`0x2000`, including custom multi-bit specialists); all Bards (Blade, Jester, Skald, mod kits), Wild Mages, kitted dual-class Mages, and non-Mages stay at 0%. Verified live in quiet labs (`2026-11-23`) on BG2:EE `2.7.3.0` and IWD:EE `2.7.3.0` with EEex `v1.3.1` (`BG:EE / SoD` live lab pass pending).
- Dorn's ambush fix `135` (alpha.13) is **default (Christopher's decision,
  2026-10-25)**. When Dorn catches up with Senjak and Dorotea in BG1, the cutscene no
  longer locks the camera on Senjak, and Senjak's script releases a lock once he turns
  hostile. Leaving the area after a stalled conversation therefore no longer crashes the
  game; a completed conversation still ends in the fight. Its TP2 needs `CUTDN1B.BCS`,
  which this BG2-based install gets from EET's BG1 import, so the feature requires EET's
  core. It runs in the main post-EET_end Modpack run right after `130`, in TP2 order, so
  it follows EET and SCS, as its
  [guide](https://github.com/Chrizhermann/chriz-bg-modpack/blob/v0.2.0-alpha.14/docs/dorn-ambush-camera.md)
  requires (after the mods that change `CUTDN1B.BCS` or `SENJACK.BCS`).
- Party AI fix `460` (alpha.13) is **default (Christopher's decision, 2026-10-25)**: the
  party AI scripts use Defensive Stance only while "Use special abilities" is on. EET's
  BG1 stance block in the default party AI script `BDDEFAI` ignored that option
  ([Gibberlings3/EET#210](https://github.com/Gibberlings3/EET/issues/210)). It patches
  the scripts `PARTYAI.2DA` names and needs no other mod. It runs in the main
  post-EET_end Modpack run after `450`, in TP2 order, so it follows EET and every party
  AI writer in the earlier runs, as the Modpack README requires.
- Chapter XP caps `652` (alpha.13) are **optional, unchecked and experimental
  (Christopher's decision, 2026-10-25)**, because they need `650`; since 2026-11-10 they
  belong to Challenge Mode with `650` (see below). Instead of `651`'s
  single SoA cap they cap usable XP per chapter across BG1, SoD, SoA and ToB, from 16,000
  in BG1 Chapters 1-2 up to 8,900,000 in ToB Chapter 10, and bank the excess. The TP2
  makes `651` and `652` mutually exclusive, and so do their features. `652` also makes
  Baeloth's Larswood entrance (`BAINVI.BCS`) wait for Chapter 5, so his 40,000 starting XP
  fits under that chapter's cap; if another mod replaced the scene, it leaves the script
  unchanged and warns. It runs after `650` and `651` in the campaign XP run. The ToB
  Chapter 10 cap of 8,900,000 needs an XP cap remover such as Tweaks Anthology `2090`, a
  default in the curated recipe.
- The collection exposes every implemented component except standalone `160`, whose exact
  skill repair is invoked atomically by the default Skie component `195`.
- Component `400` remains visible but conditionally inactive unless its Branwen /
  Spell Revisions prerequisites are selected.
- Scrolls `430` (alpha.11) is default and no longer uses Tweaks Anthology `2170`: it
  requires Windows EEex and sets the casting level for every scroll reader (the reader's
  arcane or divine caster level; other readers, such as Use Any Item users, half their
  highest class level, minimum 1). At install time it hashes every scroll and the spells
  they cast, so it runs in its own final run `chriz-bg-modpack-scrolls-bg2`, after BuffBot
  and every other spell/item writer. Alpha.11 to alpha.13 turned it off for the session on
  2.7.3.0 with EEex v1.3.1 after a load, a new game, an NPC joining or opening a
  container, with an EEex message box; alpha.14 fixes this, and `430` stays a default
  (Christopher, 2026-10-25). Its fallback information goes to the loader log only; one
  box remains for the rare case that it cannot start cleanly and cannot undo its start.
  The loader-log check in the Recommended lab run is still open; see
  [FOLLOW_UPS](FOLLOW_UPS.md).
- Fade `237` and Evandra `238` are default whenever Fade or Evandra is selected: at the
  first SoA recruitment they raise the companion to the highest protagonist XP threshold
  reached (125,000 to 1,250,000), never lower XP and pay once. They run in the
  late-companion run in TP2 order, `189, 237, 238, 239, 620`.
- EET join XP `239` (alpha.13) is **default (Christopher's decision, 2026-10-18)**. It
  adds EET's own SoA join-XP ladder and ToB raise to joinable NPCs whose scripts lack
  them, found from EET's campaign tables and transition scripts, never lowers XP and
  leaves existing ladders of any owner unchanged. Its TP2 requires EET (`GAME_IS eet`); it
  needs no spell, class, kit, Spell Revisions or EEex component. It runs in the
  late-companion run after `238`, so it follows the NPC mods, SCS, EET_end and Safana,
  and precedes campaign XP `650` as the Modpack's
  [EET join-XP guide](https://github.com/Chrizhermann/chriz-bg-modpack/blob/v0.2.0-alpha.13/docs/eet-join-xp.md)
  requires. With `237`/`238` either order works.
- Campaign XP caps `650` (BG1 161,000, SoD 500,000; excess banked) and the SoA cap `651`
  (2,950,000, requires `650`) are optional, unchecked and experimental. Both require EEex
  and run in their own run `chriz-bg-modpack-campaign-xp-bg2` right after the
  late-companion run. Alpha.11 and alpha.12 refused every permanent XP award with `650`
  installed; alpha.13 fixes the banking, lab-verified on 2.7.3.0 with EEex v1.3.1. Up to
  alpha.13 an XP change the bank could not take stopped the rest of that character's
  effects, so script variables set behind it were lost at the next save
  ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19)); alpha.14 fixes
  this and writes `650`'s fallbacks to the loader log only. `651` and chapter caps `652`
  are either-or.
- **Challenge Mode includes `650` and `652` (Christopher, 2026-11-10)**: switching the
  [group](../../plans/2026-10-13-challenge-mode-group.md) on turns on campaign XP caps with
  chapter caps; `652` comes on with `650`, which it requires. `651` stays outside the group
  as a separate optional choice. A player who ticked `651` keeps it when switching Challenge
  Mode on, and `652` then stays off with its conflict reason. All three stay optional,
  unchecked and experimental, and Chris's Recommended keeps Challenge Mode off.
- Degrodel `660` is default: refusing to hand over the Helm of Balduran makes Degrodel
  and his servants attack; handing it over keeps the normal reward. It runs in the main
  post-EET_end Modpack run after `630`.
- Marek's Boomerang Dagger `670` (alpha.15) is **default (Christopher, 2026-11-03)**. Since
  BG1 NPC Project left the defaults, BG1 and SoD had no Boomerang Dagger +2 (`DAGG11`): its
  only source there was that mod's Kivan sea-elf quest. Both of Marek's creature files,
  `MAREK.CRE` (the first meeting in the north-east district of Baldur's Gate) and
  `MAREK1.CRE` (the antidote fight at the Blushing Mermaid), now carry one unequipped
  dagger in the backpack; only one Marek can die, so one dagger drops. When the Kivan
  quest's Shipwreck's Coast script already creates the dagger, `670` changes nothing, so
  ticking BG1 NPC Project needs no conflict rule. Its TP2 needs `MAREK.CRE` or `MAREK1.CRE`
  and `DAGG11.ITM`, which this BG2-based install gets from EET's BG1 import, so the feature
  requires EET's core. It runs in the main post-EET_end Modpack run after `660`, in TP2
  order, after SCS and every other mod of the recipe that changes Marek's files. The
  Modpack's guide asks for `670` before mods that read creature items, such as the Item
  Randomiser; the recipe's Randomiser fork v8.1.1 runs earlier, but `DAGG11` is only in its
  BG2 lists, so the Randomiser never handles Marek's dagger and the order changes nothing. If
  the dagger joins the Randomiser's BG1 pool (see [FOLLOW_UPS](FOLLOW_UPS.md)), `670` has to
  move before it.
  Not yet tested in a game: the two Marek fights and the drop.
- EET world progression and SoD backtracking fixes `680` (alpha.17). **Christopher's decision, 2026-11-22: on only with Challenge Mode, off by default.** It is an optional, unchecked feature and a member of the [Challenge Mode group](../../plans/2026-10-13-challenge-mode-group.md); Chris's Recommended leaves it off. It implements his 2026-11-01 Challenge Mode idea of no travel back into an earlier campaign (see [FOLLOW_UPS](FOLLOW_UPS.md)). Requires EET (`feature:eet:mandatory-components`) and `mod:eet-end`, and runs in `chriz-bg-modpack-bg2` after `670`. It preserves EET's single install and BG1/SoD/SoA/ToB new-game starts while closing cross-campaign backtracking exploits (`AR0020 <-> BG4800`, `BD1000 <-> BG0900`, `AR3000 -> Amn` on ToB entry, and post-SoD `BD4000.BCS` reopening), keeping the SoD Ch.8-10 road open until the Dragonspear assault, and fixing abandoned SoD camp NPC stranding, chest forwarding, Baeloth's pit cleanup, camp sentries, siege audio and duplicate Jegg. All area rules live in `eet_world_progression.2da`.
  Its Siege of Dragonspear camp fixes do not depend on the travel lock and are planned as their
  own default component in the next Modpack release (see [FOLLOW_UPS](FOLLOW_UPS.md)); until then
  they stay off in Recommended.
  Not yet tested in a game: no full playthrough across the campaign transitions.
- Kivan quest guard `130` is mandatory with BG1NPC Banters/Quests/Interjections `10`
  and SCS general AI `6000`, after EET import and SCS. It is independent of Kivan's
  Archer choice `198`; never add standalone `KIVAN_QUEST_FIX` alongside it. Its
  default inclusion was explicitly reconfirmed on 2026-09-09. Since BG1 NPC Project left the
  defaults (Christopher, 2026-11-03), `130` installs only when BG1NPC `10` is ticked; the
  known limitations record that omission.
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
| 135 | Dorn's BG1 ambush: no camera lock on Senjak (prevents a crash after leaving the area) | chriz-bg-modpack: NPC fixes |  |  | default |
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
| 230 | Rasaad: Dexterity 17 and Constitution 16 | chriz-bg-modpack: NPC fixes | | | default |
| 231 | Dorn: Constitution 16 | chriz-bg-modpack: NPC fixes | | | default |
| 232 | Eldoth: Strength 17, Constitution 16, Wisdom 9 and Intelligence 14 | chriz-bg-modpack: NPC fixes | | | default |
| 233 | Garrick: Dexterity 17 and Intelligence 15 | chriz-bg-modpack: NPC fixes | | | default |
| 234 | Quayle: Constitution 15 | chriz-bg-modpack: NPC fixes | | | default |
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
| 239 | EET: SoA and ToB join-XP catch-up for joinable NPCs whose scripts lack it | chriz-bg-modpack: Gameplay tweaks | | | default |
| 630 | Hide unidentified item appearance | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 650 | Campaign XP caps - keep excess XP for later (BG1/SoD; experimental) | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 651 | Campaign XP caps - also cap SoA until ToB (requires 650; experimental) | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 652 | Campaign XP caps - chapter-by-chapter XP caps across BG1, SoD, SoA, and ToB (requires 650; experimental) | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 660 | Degrodel: hand over the Helm of Balduran or keep it and fight him | chriz-bg-modpack: Gameplay tweaks | | | default |
| 670 | Marek: carry the Boomerang Dagger +2 in BG1 (drops it when he dies) | chriz-bg-modpack: Gameplay tweaks | | | default |
| 680 | EET: story-consistent campaign boundaries and SoD backtracking fixes | chriz-bg-modpack: Gameplay tweaks | | | optional |
| 695 | BG1 early-party XP boost: +70% party XP pool per extra companion up to 16,000 XP (BG:EE and EET; Windows EEex) | chriz-bg-modpack: Gameplay tweaks | | | default |
| 640 | Legacy UI translation helper compatibility | chriz-bg-modpack: Compatibility | | | mandatory |
| 460 | Party AI Defensive Stance: obey the "Use special abilities" option | chriz-bg-modpack: Compatibility | | | default |
| 470 | Spell Learning: restrict +/-15% specialist modifier to specialist mages (BG:EE, SoD, BG2:EE, EET, IWD:EE; Windows EEex) | chriz-bg-modpack: Spell and compatibility fixes | | | default |

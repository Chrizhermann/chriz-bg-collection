# Curation worksheet — Phase 0.4 (hard human gate)

**This file is Chris's.** Agents regenerate the data tables on request but never fill a
decision column, never reorder rows into recommendations, never preselect. Legend for the
`Decision` column — freeform, suggested vocabulary: `core` (always installed),
`toggle` (optional, on by default), `toggle-off` (optional, off by default),
`drop`, `?` (park). Add anything else you like; the worksheet is parsed by a human.

Data source: `manifest/install-order.tsv` (2026-08-19 re-capture, 451 entries / 91 mods)
+ archive inventory. Component-level detail stays in the TSV — this sheet decides at mod
level; component-level curation happens when `mods/*.toml` gets authored from your calls.
This is now a living document — decisions are preserved, tables are not regenerated.

## 0. Decisions recorded from Chris (2026-08-19, dictated in session)

- **Versions**: installed versions are mostly outdated — the compilation targets *current*
  mod versions (Phase 0.3 pin-list proposes the upgrades for review).
- **BRISTLELICK**: drop ("wasn't that good anyway").
- **EVANDRA**: keep as a **toggle whose GUI label states it requires a manual download**
  (default on, per the recommended install). Licensing checked 2026-08-20: no license
  (repo `license: null`, no LICENSE/readme grant) → default copyright, we may NOT
  rehost her. Long-term fix: ask Rhaella/G3 for a GitHub release. The sorcerer
  conversion (`EVANDRA_SORCERER` local fix) becomes its own **optional toggle**.
  With Bristlelick dropped, Evandra is the only manual-download step in the install.
- **Voice packs / soundsets**: excluded from the collection entirely — offered separately
  later (many are self-created). Covers all local soundsets + the third-party voice packs
  (`AW_PF_SOUNDSETS`, `ZG_BGNPC_VOICES_BGS`, `BASTIL`). *Open:* does `HQ_SOUNDCLIPS_BG2EE`
  count as a voice pack (it restores HQ audio for BG2EE content rather than adding NPC
  voices) — Chris to rule.
- **SAFANA**: keep — the Roxanne association is a non-issue. Known problem to fix: she
  kept her SoD items in the EET game (hard no-go; needs a fix component in the manifest).
  The bard conversion (via local patches, e.g. `BARD_SPELL_FIX`, `SAFANA_LATE_SPELLS`)
  becomes an **optional toggle**.
- **WINGS**: drop.

## 1. Third-party mods currently installed (35)

| Mod folder | Comps | Version | Identifies as | Decision |
|---|---|---|---|---|
| AJANTISBG2 | 1 | 21 | Sir Ajantis NPC for BGII | drop and maybe look for something else later |
| ARTISANSKITPACK | 47 | 6.0 | Artisan's Kitpack | Important part of my collection, BUT we will use our own custom version, which has it's own repo (which is a fork, that we keep in sync with upstream |
| ARTISANSKITPACK_NPC | 6 | — | Artisan's Kitpack | same as above |
| ARTISANSKITPACK_TWEAK | 7 | — | Artisan's Kitpack | will have to do a quick check together later |
| ASCENSION | 16 | 2.1.0 | Rewritten Final Chapter of Throne of Bhaal | newest version essential |
| AURA_BG1_2_EET | 4 | — | Aura NPC for Baldur's Gate | optional (default not, mark as overpowered currently, I will have to nerf it more later) |
| BARDICWONDERS | 21 | — | Bardic Wonders | Important part of my collection, BUT we will use our own custom version, which has it's own repo (which is a fork, that we keep in sync with upstream |
| BGGO | 1 | v3.5 | Baldurs Gate Graphical Overhaul Core | optional, included by default |
| BRANWEN | 1 | v8pre | Branwen BG2 NPC mod for players and modders | optional, included by default |
| BRISTLELICK | 1 | v2.4 | Bristlelick—the gnoll companion for BGEE | drop |
| BUBB_SPELL_MENU_EXTENDED | 1 | v5.1 | Bubb's Spell Menu Extended | newest version essential |
| C0WARLOCK | 1 | 3.0 | Warlock Kit | |
| CDTWEAKS | 62 | v18 | Change Viconia's Skin Color to Dark Blue | newest version essential |
| CROSSMODBG2 | 3 | v30 | Crossmod Banter Pack for Shadows of Amn Content | essential |
| EEEX | 8 | v0.11.0-alpha | EEex | essential, we will 1000% use the newest version, not the one you mentioned |
| EEFIXPACK | 2 | Alpha 3 | Core Fixes | newest version essential |
| EET | 2 | v14.0 | EET core (resource importation) | newest version essential |
| EET_END | 1 | — | EET end (last mod in install order) -> Standard installation | newest version essential |
| EET_TWEAKS | 1 | 1.12 | XP for Traps, Spells and Lockpicking -> Vanilla friendly pro | Drop, we will replace this with our own version before v1 of the collection |
| EVANDRA | 2 | v2.2 | Evandra NPC | keep — toggle, labeled "manual download required" |
| FADE | 1 | 5.6 | Fade | optional, included by default |
| HIDDENGAMEPLAYOPTIONS | 27 | 5.0 | Add in-game option "Enable Debug Mode" | optional, included by default |
| IEPBANTERS | 7 | v5.9 | Extended NPC-NPC Interaction SoA | optional, included by default |
| IWDIFICATION | 8 | v11 | Icewind Dale Casting Graphics (Andyr) | newest version essential |
| PAINA | 1 | — | Pai'Na NPC for BG2 | optional, included by default |
| RANDOMISER | 9 | 7 | Randomise scrolls | newest version essential with my own tweaks, I think we already have a fork for this? |
| RR | 7 | v4.92 | Proper dual-wielding implementation for Thieves and Bards | essential |
| SARAHTOB | 1 | v8 | Sarah NPC Romance Mod for BG2 | optional, included by default |
| SIRENE_BG2 | 3 | — | Sirene NPC for BG2 | optional, included by default --- BG1 version missing |
| SPELL_REV | 7 | v4.19 | Spell Revisions | optional, included by default --- including or excluding has impact on a lot of other mods. |
| STRATAGEMS | 76 | 35.21 | Install all spell tweaks (if you don't select this, you will | essential, will later replace with my own version |
| UB | 22 | v28 | The Kidnapping of Boo by Cliffette |  |
| WINGS | 2 | — | Wings for BG2 | drop |
| XAN | 2 | v19 | Xan NPC MOD for Baldur's Gate II | optional, included by default, also needs optional change that makes him a fighter/mage in both games (and maybe even a eldritch knight, if Artisans is involved, all active by default) |
| YESLICKNPC | 1 | v5.0 | Yeslick NPC for BGII | optional, included by default |

!!! Lots of stuff missing here and a lot of options from SCS and CDTweaks and UB, etc etc, are not listed. !!!

## 2. Uncertain provenance (5) — research in progress

| Mod folder | Comps | Version | Identifies as | Decision |
|---|---|---|---|---|
| AW_PF_SOUNDSETS | 1 | — | Pathfinder Soundset | drop (voice packs excluded) |
| BASTIL | 1 | — | Bastila Shan Soundset | drop (voice packs excluded) |
| HQ_SOUNDCLIPS_BG2EE | 1 | 1.3 | Install high quality soundclips for new BG2EE content | audio restoration, have a subagent check for peoples opinion, but imo it should be essential |
| SAFANA | 1 | v0.5 | Safana in Amn | keep (fix SoD-items carryover; bard conversion = toggle) --- optional, included by default (including all changes) |
| ZG_BGNPC_VOICES_BGS | 1 | 0.3.1-bgs | Baldur's Gate NPC Voice Pack for EE 2.6+ | drop (voice packs excluded) |

## 3. Chriz-layer repos (9)

| Mod folder | Comps | Version | Identifies as | Decision |
|---|---|---|---|---|
| ABETTORHLAREBALANCE | 1 | 1.0.0 | Abettor of Mask | |
| AKCB_BERSERKER | 1 | 2.0 | Berserker Overhaul Rebalance | |
| AKCB_SHAPESHIFTER | 1 | 1.0 | Shapeshifter Overhaul Rebalance | |
| AURA_BALANCE_PATCH_SPELLS | 1 | v1.0 | Aura Balance Patch - Spell Effects | |
| BG2EE-EET-FIXPACK | 5 | 1.0 | Branwen BG2 | |
| CHRIZ-BG-MODPACK | 4 | v0.1.0 | Use Any Item | |
| CHRIZ-BG-REBALANCE | 7 | v0.1.0 | ?????? ->  | |
| CHRIZ-SOD-REMIX | 21 | v0.5.0 | SoD remix | |
| EEEXREMOTE | 1 | v0.2.0 | EEex Remote Console | |

## 4. Local hotfix micro-mods (22)

These exist only in the reference install. Besides keep/drop, each needs a **home**:
absorb into a chriz repo (modpack/rebalance/balance-patch), or become a tracked micro-mod
of its own. Put the intended home in the Decision column.

| Mod folder | Comps | Version | Identifies as | Decision |
|---|---|---|---|---|
| AK_MULTICLASS_PROFS_FIX | 1 | 2 | Fix missing multiclass proficiency stats in Artisan's Kitpac | |
| AURA_BALANCE_PATCH | 1 | v1.0 | Aura NPC Balance Patch | |
| BARD_SPELL_FIX | 1 | 1 | Add Bard spells to Aura and Safana CRE files | part of Safana-bard toggle |
| BEARSKIN_MAIL_FIX | 1 | 1.0 | Red Bearskin Mail - Rashemi Berserker Kit Usability Fix | |
| BRANWEN_HAMMER_FIX | 1 | 1.0 | SR Branwen Spiritual Hammer (SPIN113) - Fix param1 for Creat | |
| CBM_UAI_SCROLL | 1 | 1.0 | Use Any Item | |
| EDWIN_AMULET_FIX | 1 | 1.0 | Remove bonus spell slots from Edwin's Amulet (MISC89) | |
| ELEM_PRINCE_CLAB_FIX | 1 | 1.0 | Remove Elemental Prince Call (SPPR724) from Druid/Ranger CLA | |
| EVANDRA_SORCERER | 1 | 2 | Convert Evandra from Illusionist Mage to Sorcerer | toggle (optional sorcerer conversion) |
| FADE_FT_FIX | 1 | 2 | Convert Fade NPC to Fighter/Thief multiclass | |
| FADE_FT_PATCH | 1 | 1 | Fade F/T patch | |
| KIVAN_QUEST_FIX | 1 | 1.0 | BG1NPC Kivan Sea Elf Quest - SCS Compatibility Fix | |
| MAZZY_PROF_FIX | 1 | 1 | Cap Mazzy Short Bow to 4 pips, redistribute to Short Sword | |
| NPC_KIT_CHANGES | 1 | v1.0 | NPC Kit & Class Changes (Tier 1-3) | |
| PRIEST_DELIVERY_FIX | 1 | 1.0 | Priest spell CLAB delivery fix (AK 0x00/0x40/0x80 mis-sort) | |
| SAFANA_LATE_SPELLS | 1 | 1 | Safana BG1 template | part of Safana-bard toggle |
| SAFANA_SNARE_FIX | 1 | 1 | Remove Thief Set Snare (SPCL412) from Safana CRE files | |
| SKIE_SKILL_FIX | 1 | 1 | Redistribute Skie Move Silently points to Open Locks (Swashb | |
| SR_SUBSPELL_FIX | 1 | 1.0 | Remove SR hidden subspells from joinable NPC spellbooks | |
| VICONIA_MULTICLASS | 1 | 1 | Convert Viconia to Cleric/Thief multiclass | |
| XAN_EK_FIX | 1 | 1 | Fix Xan Eldritch Knight for EET (patches XAN_, XAN4, XAN6, T | |
| YESLICK_KELDORN_DISPEL_FIX | 1 | v1.0 | Fix Yeslick and Keldorn dispel magic (friendly fire + uncapp | |

## 5. Local soundsets (18)

Built locally from archive assets (`soundsets/`, `bastilla sound/`, portrait packs…).
For a public release these need a packaging/licensing decision (voice clips from other
games/media may not be distributable at all).

**Decision (Chris, 2026-08-19): ALL dropped from the collection — voice packs are excluded and offered separately later.**

| Mod folder | Comps | Version | Identifies as | Decision |
|---|---|---|---|---|
| ALORA | 1 | — | Alora Soundset | |
| CD_AERIE | 1 | — | Aerie Soundset | |
| CD_AJANTI | 1 | — | Ajantis Soundset | |
| CD_DRANGER | 1 | — | Dranger Soundset (Sylvanas Windrunner) | |
| CD_EDWNBG1 | 1 | — | Edwin BG1 Soundset | |
| CD_EDWNBG2 | 1 | — | Edwin BG2 Soundset | |
| CD_IMOBG1 | 1 | — | Imoen (BG1) Soundset | |
| CD_IMOBG2 | 1 | — | Imoen (BG2) Soundset | |
| CD_JAHEBG1 | 1 | — | Jaheira BG1 Soundset | |
| CD_JAHEBG2 | 1 | — | Jaheira BG2 Soundset | |
| CD_NEERA | 1 | — | Neera Soundset | |
| CD_RUMIKP | 1 | — | Rumi KPop Soundset | |
| CD_SAFANA | 1 | — | Safana Soundset | |
| CD_SAFSOD | 1 | — | Safana SoD Soundset | |
| CD_SAREVOK | 1 | — | Sarevok Soundset | |
| CD_YESLICK | 1 | — | Yeslick Soundset | |
| RONDAK | 1 | — | Rondak | |
| XAN_SOUNDSET | 1 | — | Xan | |

## 6. Previously dropped (in archive, deliberately not installed)

Reconfirm or reconsider — from EET_MODDING_GUIDE:

SkitiaNPCs, Brage's Redemption, Imoen 4 Ever, Angelo, Dvaradime, Drake,
Juniper & the Stone Leech, Kitanya, all friendship mods, Transitions, Romance Expanded,
Morpheus562's Kitpack (author removed from GitHub), Solaufein Romance, Xan BG1 Voice,
Crucible, Oblivion Guard Soundset

Decision(s):

## 7. Never-installed archive extras

Present in the mod archive, never part of any install:

ItemRevisions (2 variants), InfinityUI-1.17, eeuitweaks,
lefreuts-enhanced-ui, enhanced-powergaming-scripts, RemasteredSpellIcons,
Portraits-Portraits-Everywhere, House-Rule-Tweaks

Decision(s):

## 8. Open curation questions (from planning, in Chris's words)

- NPC mods: ship "less than I have installed" — which ones make the cut?
- Toggles named so far: Spell Revisions, Artisan's Kitpack, SoD remix — final list?
- Common-customization toggles: which small options does the GUI expose?
- Game-UI choice group: which UI mods are offered; default = the UI Chris plays.
- SCS: reference uses stock 35.21 now; own modified SCS planned later.
- Hard prohibitions to encode as manifest rules (from the guide): Divine Remix,
  Wheels of Prophecy, Sandrah Saga; never change Imoen's/Mazzy's kits.

Notes:

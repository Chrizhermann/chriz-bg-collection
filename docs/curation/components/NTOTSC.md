# NTOTSC — components

Listed at the pinned upstream `master` commit **f0ef953** (Spellhold-Studios/NTotSC,
2024-12-13); its TP2 still reports `VERSION ~5.0.0~`. Not in the reference install.
17 entries, 0 installed. ✓ = installed. Subgroup = mutually exclusive; choose zero or one.

## UI/dependency notes

- Christopher's decision (2026-10-23): Northern Tales of the Sword Coast is an optional,
  unchecked third-party quest mod in "Quest mods", labelled experimental until the
  combined lab install, and offered as is. Recommended does not include it.
- Inside the mod only the provisional default is decided: the main component `0` starts
  selected when the mod is ticked, everything else starts unchecked. Christopher has not
  yet chosen the other defaults; each is one `Decision` cell here and one `decision`
  line in the recipe.
- Components `1`–`14` require `0`; `8` (Will O'Hara's soundset) requires `7`.
  `15`/`16` ("Spell Scrolls") have no requirement in the TP2.
- `13`/`14` ("Reduce Number of Shaman Cloaks") and `15`/`16` ("Spell Scrolls") are WeiDU
  subcomponent pairs, so each pair is exclusive.
- In v5.0.0 the two spell scrolls (Skeleton Guard and Fiery Cloud, level 7/8) came with
  the main component. At f0ef953 they are the separate choice `15`/`16`, so "main
  component only" installs NTotSC without them. The readme still describes the v5.0.0
  layout.
- `4` and `6` refuse to install when BG1 Unfinished Business has already restored the same
  encounters (`FORBID_FILE override/ubnimfur.CRE` and `override/UBSVLAST.CRE`, TP2
  L2516 and L2592), and BG1UB `30`/`32` create exactly those files before EET. CEBG
  therefore marks NTotSC `4` unavailable while BG1UB `30` (Nim Furlwing Encounter) is
  selected and `6` while BG1UB `32` (Svlast, the Fallen Paladin Encounter) is selected; the
  reason tells the player to untick the BG1UB component to get NTotSC's version.
- Refusals that CEBG's stack cannot trigger: `0` refuses BP-BGT Worldmap `0` (TP2 L237)
  and `2`–`6` refuse Jarl's Adventure Pack creatures (`JA#*.CRE`); CEBG ships neither.
  `0` requires `GAME_IS bgt eet bgee`.
- Facts for later curation, not fixes: with SCS the generic orc casters in `AR01PB` are
  very strong (upstream #68; PR #79 unmerged); its rat quest item `NTMISC07` is added to
  the shared `RAT.CRE`, so BG2 rats carry it too (cosmetic); upstream #82 (Magnad and
  Winorma missing on EET) is open without reply.

## Collection behavior

- Run `ntotsc-bg2` installs on the EET (BG2) side in the `main` phase, after EET import and
  both EE Fixpack runs and before the NPC mods, SCS, the Item Randomiser and EET_end. Its
  ini declares `After = bg2fixpack, eefixpack, EET` and `Before = EET_end`. It runs before
  Ascalon's Questpack (BWS-NG order) and Thalantyr Item Upgrade (Thalantyr's readme).
- The run lists the components in TP2 order (`DESIGNATED 0`–`16`, ascending; checked by
  reading the TP2, because `tools/check_native_component_order.py` stops at an ALWAYS-block
  `ACTION_DEFINE_ASSOCIATIVE_ARRAY` that it reads as a component).
- The Item Randomiser duplicate-source audit at f0ef953 found no hits, so no `no_random`
  rows are needed.

## Changes over v5.0.0 (35 commits up to the pin)

- The spell scrolls moved from the main component into `15`/`16`, with a new revised
  level-6 version (b2b9cf1).
- The `NTKROTAN.CRE` level/HD fix that SCS's proficiency assignment needs (upstream #64).
- The Island of the Sirines (`AR9PBA`) on the worldmap and an updated Northern Citadel icon
  (21c7ec6).
- Reedrig's spawn script in Ulgoth's Beard patched with `EXTEND_TOP` (9730efd, PR #70) and
  the AR25PB salamander scripts (PR #53).
- Typo and bug fixes (e350649, 07e8321), lowercase file names (9d59428), and French,
  Spanish and Simplified Chinese translations.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Northern Tales of the Sword Coast (NTotSC) for BGT-Weidu, BG:EE, and EET |  |  |  | default |
| 1 | Spawn less Monsters depending on Game Difficulty | Nerfs |  |  | optional |
| 2 | Keelor the Dwarf | Quests and Encounters |  |  | optional |
| 3 | Llindellyn's Lucky Arrow | Quests and Encounters |  |  | optional |
| 4 | Nim Furlwing's Hunting Hounds | Quests and Encounters |  |  | optional |
| 5 | Pilar and Gheldehar | Quests and Encounters |  |  | optional |
| 6 | Svlast's Torment | Quests and Encounters |  |  | optional |
| 7 | Will O'Hara NPC | NPC |  |  | optional |
| 8 | Will O'Hara: Install English Soundset | NPC |  |  | optional |
| 9 | Fighting Encounters | Tactical Challenges |  |  | optional |
| 10 | Improved Lesser Fiends | Tactical Challenges |  |  | optional |
| 11 | Improved Haeball | Tactical Challenges |  |  | optional |
| 12 | Improved Ghotal | Tactical Challenges |  |  | optional |
| 13 | Reduce Number of Shaman Cloaks -> Remove Cloaks from generic Orc Priests | Nerfs | Reduce Number of Shaman Cloaks |  | optional |
| 14 | Reduce Number of Shaman Cloaks -> Make Cloaks Undroppable | Nerfs | Reduce Number of Shaman Cloaks |  | optional |
| 15 | Spell Scrolls -> Revised Version (level 6 spells) | Quests and Encounters | Spell Scrolls |  | optional |
| 16 | Spell Scrolls -> Original Version (level 7/8 spells) | Quests and Encounters | Spell Scrolls |  | optional |

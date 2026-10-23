# PPE — components

Listed at the pinned DaftHunk fork commit **417226ea** (Portraits Portraits Everywhere,
2025-12-31; the README calls it v1.03, the TP2 still reports `VERSION ~1.02~`). Not in
the reference install.
21 entries, 0 installed. ✓ = installed. Subgroup = mutually exclusive; choose zero or one.

## UI/dependency notes

- Christopher's decisions (2026-10-23): PPE is an optional, unchecked third-party mod in
  "Quest mods", labelled experimental until the combined lab install, and offered as is.
  Recommended does not include it. Components `310` and `320` are not offered because
  they are expected to fail on EET. Pai'Na's portrait is left as PPE sets it.
- Inside the mod only the provisional default is decided: the core component `0` starts
  selected when the mod is ticked, everything else starts unchecked. Christopher has not
  yet chosen the other defaults.
- `100`/`150` (category portraits, sequenced or random) are exclusive and require `0`.
  The portrait-style subgroups for Imoen (`300`–`302`), Viconia (`311`/`312`), Minsc
  (`330`/`332`) and Edwin (`340`/`342`) are exclusive choices. Jaheira keeps only `322`,
  so it is a single optional component.
- `310` (Viconia, BG1 style) and `320` (Jaheira, BG1 style) build the source names
  `VICONIAIAbg1?.bmp` and `JAHEIRAAbg1?.bmp` (TP2 L206, L229), which the mod does not ship;
  both are expected to fail with a missing-file COPY error.
- The Viconia and Jaheira choices change only their BG1 portraits: PPE writes the SoD and
  BG2 copies as `BDVICONIA*`, `NVICONIA*` and `NJAHEIRA*`, while the game reads
  `BDVICON*`, `NVICON*` and `NJAHEIR*`. The Imoen, Minsc, Edwin and Aerie names match.
- `25` refuses only the old BG2 Tweaks `setup-bg2_tweaks.tp2` `1180`, which CEBG does not
  ship, so the refusal never triggers. CEBG's curated recipe has Tweaks Anthology `1180`
  "Female Edwina" (a default), which already gives Edwina her portraits; with both, PPE's
  effects come later and which portrait shows is unverified. `25` is redundant there.
- `20` replaces small portraits of major SoA/ToB characters (`copy/BG2/spawn`). Ascension's
  curated defaults `2000`, `2100` and `2300` write some of the same files (Bodhi,
  Balthazar, Gromnir, Abazigal); PPE installs later, so those characters get Ascension's
  large and medium portraits with PPE's small one.
- `30` text-patches vanilla scripts and dialogues (AR2102, BALDUR, DRAGSILV, UDCBACK,
  CUT44C, UDARDUL, UDDEMON); it covers only the game's own companions, and where another mod
  changed the matched text the patch silently does nothing.
- `10` requires a non-IWD game; `20`, `25`, `30`, `40` and `350` require BG2:EE or EET.
- Size: a 387 MB download, and an estimated 0.4 GB of BMPs copied to override.
- Licensing fact for Christopher: the README says the portraits came from NWN and web
  searches, the fork has no license file, and fork issue #12 (an artist asking for credit
  for some portraits) is open.

## What the core component overwrites (fixture facts)

- Core `0` writes the dialogue/party portrait field of every CRE in its table, even where
  another mod already set one; category components `100`/`150` only fill CREs without a
  portrait. In the reference install 30 CREs carry another stack mod's portrait that
  PPE replaces:
  - Rogue Rebalancing: 14 CREs (`RR#…S` portraits).
  - UB `15` NPC Portrait Restorations (optional in CEBG): 13 CREs, plus Suna Seni.
  - Pai'Na (`SPPAIN`): her SoA party and dialogue portrait becomes PPE's picture of the
    vanilla Spellhold inmate; her record screen keeps `PainaL`, and her ToB CRE is not in
    PPE's table. This is the one visible party-portrait change; it is kept as PPE sets it.
- Not affected: Sirene, Xan, Branwen and Safana in Amn use their own CREs, Evandra's
  portrait is not in PPE's table, and EET's transition portraits are untouched.

## Collection behavior

- Run `ppe-bg2` installs in the `main` phase after every NPC and content mod in `main`,
  SCS, the Item Randomiser and the Modpack's companion conversions
  (`chriz-bg-modpack-pre-continuity-bg2`), so their NPCs can get category portraits. It
  runs directly before Modpack continuity `199`, which the Modpack requires after all
  NPC, content and kit mods and immediately before EET_end. This matches PPE's advice to
  install late; the community order sheet also puts PPE before EET_end (mleduque's
  eet-modda order puts it after). Only collection patch layers may run after EET_end
  ([EET_END.md](EET_END.md)).
- PPE and continuity `199` share no files: `199` reads the Xan and Yeslick BG2 templates
  (`O#Xan09`, `lk#yesl`) for dialogue, allegiance, scripts, inventory and sounds, not
  portraits, and patches `AR1000.bcs`, `AR2010.bcs` and `FATESP.dlg`.
- Of the companion conversions just before PPE, only Evandra's portrait `225` sets a
  portrait (`rh#evaL` in both fields of `RH#EVA`, `RH#EV25` and `RH#EVAMP`, plus
  `rh#eval.bmp`; Modpack alpha.13). PPE does not overwrite it: no PPE table, script or
  file names those creatures or that image, and category portraits `100`/`150` only fill
  creatures whose small-portrait image does not exist in the game (`S9MP-CLib.tph`
  L114–115).
- Safana in Amn is the one NPC mod that installs later (`post-eet-end`). PPE's table does
  not list Safana's own creatures (`SAFANA*.CRE`; it lists only `C6SAFA`), so her party
  portraits stay hers. Creatures Safana in Amn adds without a portrait get no PPE category
  portrait. This is cosmetic.
- The run lists the components in the TP2's `DESIGNATED` order
  (`tools/check_native_component_order.py` passes).
- The Item Randomiser duplicate-source audit found no hits, so no `no_random` rows are
  needed.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Portraits Portraits Everywhere (PPE) - core component |  |  |  | default |
| 100 | Category Portraits (Recommended) -> Sequenced |  | Category portraits |  | optional |
| 150 | Category Portraits (Recommended) -> Random |  | Category portraits |  | optional |
| 10 | Isandir's CHARNAME Portraits |  |  |  | optional |
| 20 | Replace Low Quality TOB & SOA Portraits | BG2 Companion Portraits |  |  | optional |
| 25 | Edwin/Edwina | BG2 Companion Portraits |  |  | optional |
| 300 | Change Imoen portraits -> BG1 Portrait | BG2 Companion Portraits | Change Imoen portraits |  | optional |
| 301 | Change Imoen portraits -> SOD Portrait | BG2 Companion Portraits | Change Imoen portraits |  | optional |
| 302 | Change Imoen portraits -> BG2 Portrait | BG2 Companion Portraits | Change Imoen portraits |  | optional |
| 310 | Change Viconia portraits -> BG1 Portrait | BG2 Companion Portraits | Change Viconia portraits |  | |
| 311 | Change Viconia portraits -> SOD Portrait | BG2 Companion Portraits | Change Viconia portraits |  | optional |
| 312 | Change Viconia portraits -> BG2 Portrait | BG2 Companion Portraits | Change Viconia portraits |  | optional |
| 320 | Change Jaheira portraits -> BG1 Portrait | BG2 Companion Portraits | Change Jaheira portraits |  | |
| 322 | Change Jaheira portraits -> BG2 Portrait | BG2 Companion Portraits | Change Jaheira portraits |  | optional |
| 330 | Change Minsc portraits -> BG1 Portrait | BG2 Companion Portraits | Change Minsc portraits |  | optional |
| 332 | Change Minsc portraits -> BG2 Portrait | BG2 Companion Portraits | Change Minsc portraits |  | optional |
| 340 | Change Edwin portraits -> BG1 Portrait | BG2 Companion Portraits | Change Edwin portraits |  | optional |
| 342 | Change Edwin portraits -> BG2 Portrait | BG2 Companion Portraits | Change Edwin portraits |  | optional |
| 350 | Longhair Aerie | BG2 Companion Portraits |  |  | optional |
| 30 | Ust Natha Drow NPCs | BG2 Companion Portraits |  |  | optional |
| 40 | Vampire NPCs | BG2 Companion Portraits |  |  | optional |

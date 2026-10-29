# SIRENE_BG1 — components

Listed at the pinned upstream commit **`1c8600029a9ac31395a61b5d4d03f98337dcf83e`**
(2026-01-01, unreleased `master`; the TP2 declares no `VERSION`).
10 entries, 0 installed in the reference. ✓ = installed. Subgroup = choose one.

## Decision and source notes

- Sirene NPC for BG:EE adds Sirene to BG1 and Siege of Dragonspear. It is a separate mod
  by the author of [Sirene NPC for BG2:EE](SIRENE_BG2.md) (Artemius_I / TheArtisanBG),
  and both use the death variable `C0Sirene` so that one Sirene carries on into BG2.
  Christopher, 2026-10-29: include it as a default in alpha.22 after an EET lab test.
- Pin exact commit `1c8600029a9ac31395a61b5d4d03f98337dcf83e`, not release v3.1
  (2024-04). v3.1 supports BG:EE only: it hard-codes `AR3400.bcs` and the BG:EE dialog
  tables, so on EET Sirene would never appear. Since 2025-09, `master` requires
  `GAME_IS ~bgee eet~`, includes EET's `EET/other/cpmvars/eet_cpmvars.tpa`, extends
  `%Temple%.bcs` (the Song of the Morning temple, `BG3400` on EET) and appends to EET's
  `bgdialog.2da` and `bgbanter.2da`. The CEBG pin of the unversioned BG2 mod is the same
  kind of exact commit pin.
- Gate every portrait and class option behind core component `0`.
- Components `1`–`5` are an at-most-one portrait choice. Only `3` (Sirick, Light Armor)
  is exposed, as a fixed child, matching SIRENE_BG2 `2`. Components `6`–`9` are an
  at-most-one class choice; True Paladin (`6`) is the default and Cavalier, Inquisitor and
  Undead Hunter are optional, matching SIRENE_BG2 `5`–`8`. A Sirene who travelled with
  the party keeps her BG1 class in BG2, so the class descriptions ask for the same choice
  in both mods. There is no `DESIGNATED`; WeiDU numbers the components `0`–`9`.

## Install order

- After Artisan's Kitpack's main run. The Martyr kit now comes from the Kitpack: both
  Sirene mods skip their own copy of the kit when Kitpack component `3005` is already
  installed. This mod's own copy adds the kit with `visible = 0`, hidden from character
  creation. Installed before the Kitpack, it would leave the Kitpack's Martyr kit (a
  default) unselectable, because the Kitpack and Sirene BG2 skip a kit name that already
  exists. The three copies add the same kit (same class, weapon and usability values),
  and the Kitpack's Martyr rewrites an existing copy's description, ability table and
  spells anyway. Without a class component Sirene herself would be a Martyr; the curated
  class `6` makes her a True Paladin.
- In this order Sirene NPC for BG2 replaces one unused Kitpack Martyr file
  (`c0ilms2a.spl`, no in-game effect; see [SIRENE_BG2](SIRENE_BG2.md)). This mod ships
  no such file.
- Before Sirene NPC for BG2. On EET that mod continues Sirene from SoD into SoA only if
  `c0sirene.cre` from this mod exists when it installs (`AR0903_e.baf`); otherwise it
  spawns its own Sirene.
- Before IEP Banters and Crossmod, like Sirene NPC for BG2. Crossmod detects Sirene by
  `bc0siren.dlg`, which this mod creates; its only Sirene content needs Lava's Will NPC,
  which the collection does not include ([CROSSMODBG2](CROSSMODBG2.md)).
- Like any mod installed after them, this mod's files are outside the reach of the global
  item, spell and creature passes of Spell Revisions' main component, Bardic Wonders'
  main run and the Kitpack's main run. Sirene NPC for BG2's files left their reach with
  the move ([SIRENE_BG2](SIRENE_BG2.md)). Spell Revisions' NPC spellbooks `60` runs after
  EET_end and still finds Sirene by her death variable, and the Kitpack's `C0PR#CL`
  weapon grant still applies when she joins.
- Branwen, Ascension, Spell Revisions, Bardic Wonders and the Kitpack do not detect
  Sirene at install time, so installing both Sirene mods after them loses no crossmod
  content (static audit, 2026-10-29).

## Lab test (2026-10-29)

A disposable lab with the full Recommended plan on EET 2.7.3.0, with SoD Remix's v0.6.16
candidate. It ran on a hidden desktop through the quiet lab runner. The EEex remote
console started dialogues, moved the party between areas and set the plot variables
listed below; everything else ran natively.

- **Install.** Clean.
  - WeiDU.log has Sirene BG1 `0`, `3` and `6` after Kitpack `3005` and before Sirene BG2.
  - KITLIST has exactly one Martyr (`C0ILM`) row. Every one of the seven `K_P_*` tables
    lists it, and its ability table and help text are the Kitpack's.
  - `AR0903.bcs` has Sirene BG2's EET block.
  - No creature in the Recommended set gets the Martyr kit.
- **BG1.** Sirene stood at the Song of the Morning (`BG3400`).
  - She was a True Paladin (kit `0x4000`) with the Sirick portrait, splint mail and her
    sword Brimstone +1.
  - She joined a chaotic good character through her own dialogue.
- **End of BG1.** Simulated: Sarevok's death marker and chapter 7. The temple's own script
  then ran the transition into SoD, and SoD Remix took the party to the palace.
  - The palace removed Sirene from the party, as the mod intends.
- **SoD.**
  - The Iron Throne building (`BD0111`) brought back the same creature.
  - She rejoined through "It hasn't been that long".
  - At the first camp (`BD1000`) she stayed in the party, with no second copy.
- **SoD, never recruited.** In a game where she was never recruited:
  - The Iron Throne building created her from `C02SIREN`: a True Paladin with the Sirick
    portrait.
  - The first camp created a second copy (see the upstream notes), and that copy joined.
- **SoD to SoA, with Remix `290`.** Simulated: chapter 12, plot 581 and the start of the
  victory cutscene `bdcut59a`. From there it ran natively:
  - Sirene's remark set the plot to 587, and the talks with de Lancie and Dazzo ended
    SoD. The walk to de Lancie got stuck in the crowd, so the character was moved next
    to de Lancie, and later next to Dazzo, within the same area.
  - SoA's opening played to the end.
  - In the Order of the Most Radiant Heart (`AR0903`), Sirene BG2 brought back the same
    creature, with her SoD experience, kit and portrait and new gear. She joined.
  - A save census found exactly one Sirene in the game.
  - Her SoD armour and Brimstone +1 go where EET puts everything the party still
    carries at the SoD-to-SoA transition: the invisible container `K#ImportContainer`
    in the first SoA dungeon (`AR0602`; container type non-visible, in the map corner).
    No script in the Recommended install hands its contents back, so no second
    Brimstone reaches the player. Without `290`, her own `bdcut61` script makes her
    leave the party and destroys her equipment before that move (read from the script,
    not run). No change to `290`.
- **Not covered:**
  - the Cavalier, Inquisitor and Undead Hunter options;
  - a Sirene recruited and then left at a camp;
  - the vanilla ending without `290`;
  - other portraits;
  - anything a player would only see on screen.

## Upstream notes (third-party mod, installed as-is)

- The class components require `~Sirene.tp2~` component `0`, while the file is
  `Setup-Sirene.tp2`; WeiDU treats the `setup-` prefix as optional. In the lab, `6`
  installed and made Sirene a True Paladin.
- Both `lib/Sirene.tpa` and `lib/Sirene_sod.tpa` extend `BDINTRO.bcs` with the same
  `BDINTRO.baf`, and `scripts/c0dornf.baf` checks `Dead("C@Sirene")`.
- The camp scripts (`bd1000`, `bd3000` and `bd7100`) create Sirene whenever she has
  never been in the party. They do not check whether she already exists elsewhere. In
  the lab the first camp made a second copy, and the one in the Iron Throne building
  stayed there.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Sirene NPC for Baldur's Gate: Enhanced Edition |  |  |  | default |
| 1 | BG2 Default (Isandir) |  | Choose an alternate portrait for Sirene? |  | |
| 2 | Original BG1 Default (sporeboy) |  | Choose an alternate portrait for Sirene? |  | |
| 3 | Alternate 1 (Sirick, Light Armor) |  | Choose an alternate portrait for Sirene? |  | default |
| 4 | Alternate 1 (Sirick/Lava, Heavy Armor) |  | Choose an alternate portrait for Sirene? |  | |
| 5 | Alternate 2 (Lodaligae) |  | Choose an alternate portrait for Sirene? |  | |
| 6 | True Paladin |  | Choose an alternate class for Sirene? |  | default |
| 7 | Cavalier |  | Choose an alternate class for Sirene? |  | optional |
| 8 | Inquisitor |  | Choose an alternate class for Sirene? |  | optional |
| 9 | Undead Hunter |  | Choose an alternate class for Sirene? |  | optional |

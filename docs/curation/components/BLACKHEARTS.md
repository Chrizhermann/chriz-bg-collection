# BLACKHEARTS — components

Listed at release version **v2.05** (A-Wizard-Did-It/Black-Hearts, 2026-10-13). Not in the
reference install.
2 entries, 0 installed. ✓ = installed.

## UI/dependency notes

- Christopher's decision (2026-10-23): Black Hearts is incompatible with the mandatory
  SoD Remix. CEBG offers it as an optional, experimental third-party mod in "Quest mods"
  with an incompatibility declaration: while CHRIZ-SOD-REMIX is selected, the component is
  shown as unavailable with that reason, so it is not installed together with the Remix.
- Why: its Siege of Dragonspear part ("Pilgrimage", chapters 9–10) extends BD0109, BD3000,
  BD6000 and BD7100 scripts and adds a region, entrance and animation to BD7230, all areas
  the Remix restructures. The Remix install guards still pass statically, but the runtime
  chain is unverified.
- Component `1` covers BG:EE and EET (`GAME_IS bgee eet`); on EET it also adds a short ToB
  dialogue for Illasera. Component `20005` is for BG2:EE without EET and is excluded.
- Facts, not fixes: `COPY ~Blackhearts/Graphics~ ~override~` (TP2 L136) silently replaces
  three vanilla files, ToB Illasera's small portrait `ILLISARS.BMP` and the Bastard Sword +2
  icons `ISW1H42.BAM` and `CSW1H42.BAM`, which other items also use. On EET it creates a
  game-root `Worldmap/` folder for BP-BGT Worldmap tables that EET does not use.

## Collection behavior

- Run `blackhearts-bg2` installs in the `main` phase after `ac-quest-bg2` and before the
  NPC mods, SCS, the Item Randomiser and EET_end. Its ini says `After = EET` and
  `Before = finchnpc, c#brandock, bp-bgt-worldmap`; none of those is in CEBG.
- The Item Randomiser duplicate-source audit found no hits, so no `no_random` rows are
  needed.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1 | Install Blackhearts for "BGEE" or "EET including BGEE and BG2EE" |  |  |  | optional |
| 20005 | Install Blackhearts for "BG2EE only" |  |  |  | |

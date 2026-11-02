# AC_QUEST — components

Listed at release version **v7.0** (Gitjas/Ascalons_Questpack, 2026-07-25; English is
language index 1). Not in the reference install.
13 entries, 0 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Christopher's decisions (2026-10-23): Ascalon's Questpack is an optional, unchecked
  third-party quest mod in "Quest mods", labelled experimental until the combined lab
  install, and offered as is. Recommended does not include it. Its three pairs are choices
  with the first variant preselected: `2` (Jumper inside the cave), `5` (Of Wolves and Men
  in High Hedge) and `10` (The Serpents of Abbathor, original).
- The mod has no main component. Every other component is provisionally unchecked, so
  ticking the mod preselects only `2`, `5` and `10`; Christopher has not yet chosen the
  other defaults.
- The pairs `2`/`3`, `5`/`6` and `10`/`11` are WeiDU subcomponents and exclusive.
- Every component refuses to install after `EET_end.tp2` `0` or `1` (TP2 L279–280,
  repeated on all 13). `0`–`11` require `GAME_IS bgt bgee tutu_totsc eet`; `12` requires
  `GAME_IS bgt tob bg2ee eet`.
- Alternatives that exist only for mods CEBG does not ship: `3` places Ugh and Jumper
  outside the cave for TheArtisan's Shadow Magic, and `6` moves the couple to Beregost for
  Glam's NPC Pack.
- `11` (tweaked Serpents of Abbathor) adds a BG2 follow-up through Cromwell (`WSMITH01`
  state 13) and Cespenar (`BOTSMITH` state 4). Item Randomiser `9000` also extends
  Cespenar's recipe menu; both append, so both sets of replies show (cosmetic, no change
  needed).
- On EET, ACQ's BG1 creatures carry the real BG2 items it would otherwise copy as
  replicas (for example Darkmail +3, The Night's Gift +5 and the Hammer of Thunderbolts);
  that is by design.

## Collection behavior

- Run `ac-quest-bg2` installs on the EET (BG2) side in the `main` phase, after
  `ntotsc-bg2` and before the NPC mods, SCS, the Item Randomiser and EET_end. Its ini says
  `After = EET` and `Before = ACBre, c#brandock, EET_end`; its readme says to install it
  before NPC mods. ACBre and Brandock are not in CEBG.
- The TP2 numbers components by position (no `DESIGNATED`); the 13 `BEGIN` blocks appear
  in the order `0`–`12`, and the run lists them in that order.
- The Item Randomiser duplicate-source audit found no hits, so no `no_random` rows are
  needed.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | A Job Well-Paid |  |  |  | optional |
| 1 | A Feast for the Gnolls |  |  |  | optional |
| 2 | Jumper -> Original version: Ugh and Jumper are inside the cave. |  | Jumper |  | default |
| 3 | Jumper -> Alternative for compatibility: Ugh and Jumper are outside. |  | Jumper |  | optional |
| 4 | The Lost Son |  |  |  | optional |
| 5 | Of Wolves and Men -> Quest uses hut near Thalantyr in High Hedge. |  | Of Wolves and Men |  | default |
| 6 | Of Wolves and Men -> Quest uses house in Beregost. |  | Of Wolves and Men |  | optional |
| 7 | The Great Carlini |  |  |  | optional |
| 8 | A Home for the Gibberlings |  |  |  | optional |
| 9 | Fangirls |  |  |  | optional |
| 10 | The Serpents of Abbathor -> Original version |  | The Serpents of Abbathor |  | default |
| 11 | The Serpents of Abbathor -> Tweaked version: quest can be finished in BG1. |  | The Serpents of Abbathor |  | optional |
| 12 | A Halfling among the Eyeless |  |  |  | optional |

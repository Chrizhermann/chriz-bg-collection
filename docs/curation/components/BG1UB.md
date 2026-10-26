# BG1UB — components

The collection targets **v17.1**. The full menu was re-listed with WeiDU 249 on
2026-09-03 and matches the version installed in the isolated BG1 release candidate.
Christopher approved that exact installed set as the current alpha default; the approval
does not make it permanent curation for later releases.

35 entries, 16 release-candidate-installed. ✓ = installed in the verified BG1 candidate.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Ice Island Level Two Restoration | Restored content |  | ✓ | optional |
| 1 | The Mysterious Vial | Restored content |  |  | |
| 2 | Additional Elminster Encounter | Restored content |  |  | |
| 3 | Angelo Notices Shar-teel | Restored content |  |  | |
| 4 | Finishable Kagain Caravan Quest | Restored content |  |  | |
| 5 | Coran and the Wyverns | Restored content |  |  | |
| 6 | Kivan and Tazok | Restored content |  |  | |
| 7 | Branwen and Tranzig | Restored content |  |  | |
| 8 | Safana the Flirt | Restored content |  |  | |
| 9 | Appropriate Albert and Rufie Reward | Restored content |  |  | |
| 10 | Place Entar Silvershield in His Home | Restored content |  |  | |
| 11 | Scar and the Sashenstar's Daughter | Restored content |  | ✓ | optional |
| 12 | Quoningar, the Cleric | Restored content |  | ✓ | optional |
| 13 | Shilo Chen and the Ogre-Magi | Restored content |  | ✓ | optional |
| 14 | Edie, the Merchant League Applicant | Restored content |  | ✓ | optional |
| 15 | Flaming Fist Mercenary Reinforcements | Restored content |  |  | |
| 16 | Creature Corrections | Fixes and restorations |  | ✓ | optional |
| 17 | Creature Restorations | Fixes and restorations |  | ✓ | optional |
| 18 | Creature Name Restorations | Fixes and restorations |  | ✓ | optional |
| 19 | Minor Dialogue Restorations | Fixes and restorations |  | ✓ | optional |
| 20 | Audio Restorations | Fixes and restorations |  |  | |
| 21 | Store, Tavern and Inn Fixes and Restorations | Fixes and restorations |  | ✓ | optional |
| 22 | Item Corrections and Restorations | Fixes and restorations |  | ✓ | optional |
| 23 | Area Corrections and Restorations | Fixes and restorations |  |  | |
| 24 | Permanent Corpses | Tweaks |  |  | |
| 25 | Elven Charm and Sleep Racial Resistance | Tweaks |  |  | |
| 26 | The Original Saga Music Playlist Corrections | Fixes and restorations |  |  | |
| 27 | Sarevok's Diary Corrections | Fixes and restorations |  |  | |
| 28 | Prism and the Emeralds Tweak | Tweaks |  |  | |
| 29 | Duke Eltan in the Harbor Master's Building | Restored content |  | ✓ | optional |
| 30 | Nim Furlwing Encounter | Restored content |  | ✓ | optional |
| 31 | Restored Elfsong Tavern Movie | Restored content |  |  | |
| 32 | Svlast, the Fallen Paladin Encounter | Restored content |  | ✓ | optional |
| 33 | Mal-Kalen, the Ulcaster Ghost | Restored content |  | ✓ | optional |
| 34 | Chapter 6 Dialogue Restorations | Fixes and restorations |  | ✓ | optional |

## UI/dependency notes

- All offered BG1 Unfinished Business components are optional and unchecked by default
  at Christopher's request (2026-10-23), like Unfinished Business for BG2; keep them
  available for opt-in in Customize. Recommended no longer runs `bg1ub-bg1`.
- `30` (Nim Furlwing) and `32` (Svlast) restore the same encounters as NTotSC `4` and `6`,
  and NTotSC refuses those two when BG1UB's creatures exist. When both mods are selected,
  BG1UB's version is kept and NTotSC `4`/`6` show as unavailable with the reason; unticking
  `30` or `32` here makes NTotSC's version available.

## Collection behavior

- Run the selected components on staged BGEE+SoD after DLC Merger and the first EE Fixpack
  run, but before EET imports the staged BG1 game.
- The menu reports no mutually exclusive subcomponent groups. Blank decisions remain
  excluded from the alpha UI; they were not inferred from either the historical or the
  candidate log.
- The exact approved component order is recorded in
  [`manifest/reference/bg1-premerge-approved.toml`](../../../manifest/reference/bg1-premerge-approved.toml).

## Evidence and follow-up

- All 16 selected components installed in the isolated release candidate. One known
  `REPLACE WEIGHT` warning was reviewed as non-fatal; there were no missing or extra log
  entries.
- Re-list and re-curate only changed or newly introduced choices if the v17.1 pin moves.

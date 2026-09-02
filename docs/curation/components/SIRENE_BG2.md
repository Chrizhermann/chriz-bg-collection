# SIRENE_BG2 — components

Listed at installed version **—**.
9 entries, 3 installed. ✓ = installed. Subgroup = choose one.

## UI/source notes

- Gate every portrait and class option behind core component `0`.
- Components `1`–`4` are an at-most-one portrait choice. Components `5`–`8` are an
  at-most-one class choice; True Paladin (`5`) is the new default, replacing the
  reference Cavalier (`6`) choice.
- Pin the unversioned BG2 source to exact commit
  `00beda909a4a791a35016069b4ebb77f27279723`.
- Sirene BG1 is a separate mod, not a component of this package. Add its own catalog and
  choose between published v3.1 and later unreleased `master`; do not attach the BG1
  archive to the BG2 source row.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Sirene NPC for BG2:EE |  |  | ✓ | default |
| 1 | BG1 Default (sporeboy) |  | Choose an alternate portrait for Sirene? |  | |
| 2 | Alternate 1 (Sirick, Light Armor) |  | Choose an alternate portrait for Sirene? | ✓ | default |
| 3 | Alternate 1 (Sirick/Lava, Heavy Armor) |  | Choose an alternate portrait for Sirene? |  | |
| 4 | Alternate 2 (Lodaligae) |  | Choose an alternate portrait for Sirene? |  | |
| 5 | True Paladin |  | Choose an alternate class for Sirene? |  | default |
| 6 | Cavalier |  | Choose an alternate class for Sirene? | ✓ | optional |
| 7 | Inquisitor |  | Choose an alternate class for Sirene? |  | optional |
| 8 | Undead Hunter |  | Choose an alternate class for Sirene? |  | optional |

# RANDOMISER — components

Listed at installed version **7**. The maintained fork target is **8.1**; it preserves
the same 26 component IDs. `manifest/mod-sources.tsv` still needs to be repinned from
upstream v7 to that fork release during manifest authoring.
26 entries, 9 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Whenever the parent Randomiser mod is selected, component `1100` is included without
  a separate component toggle. Components `500`–`570` must be installed before it.
- Component `510` needs an integer value from 0 through 100 when enabled; the UI must
  collect that value rather than exposing only a checkbox.
- Component `570` is unavailable with Cursed Item Revisions component `0`.
- Component `10300` remains a default choice, but is unavailable while SCS component
  `8040` is selected. Keep it visible and disabled with the note: **Already provided by
  SCS component 8040 (Improved random spawns).**
- Components `500`, `530`, `540`, `560`, `570`, `9000`, `10200`, `10210`, and `10300`
  were marked highly recommended during curation; `default` carries their selection
  state without overloading the Decision value.

## Integration notes

- Fork v8.1's safer manifest-driven Mode 1 backend does not yet support EEex v1.x's
  root `EEex_scripts` layout. With the collection's EEex v1.2 target, component `1100`
  falls back to legacy BCS delivery. It remains installable, but does not get the fork's
  safer delivery path until the fork gains EEex v1.x support.
- The collection uses Mode 1. SCS's weapon-proficiency concern about installing
  Randomiser first applies to Mode 2's install-time equipment shuffle, so it does not
  create an ordering conflict for this preset. Keep the fresh-stack Randomiser-after-SCS
  order so its order-sensitive item and store patches see the final stack.
- Planned fork work will extend Mode 1's item pool to cover items currently exclusive to
  Mode 2. This is not implemented in v8.1; do not advertise the expanded pool yet.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 500 | Randomly replace the WIS tome normally found in TotSC with one of the 6 available types | Randomisation options |  |  | default |
| 510 | Randomly not randomise items | Randomisation options |  |  | optional |
| 520 | Kangaxx further sealed away | Randomisation options |  |  | |
| 530 | Randomise scrolls | Randomisation options |  | ✓ | default |
| 540 | Randomise the heads of the Flail of Ages | Randomisation options |  | ✓ | default |
| 560 | More Spell Shield scrolls | Randomisation options |  | ✓ | default |
| 570 | Randomise the appearance of cursed items | Randomisation options |  | ✓ | default |
| 1100 | Mode 1: Randomise with in-game scripts. No items are lost |  | Randomise items | ✓ | mandatory |
| 1200 | Mode 1: Randomise with in-game scripts. Some items are lost |  | Randomise items |  | |
| 1300 | Mode 2: Randomise with WeiDU. No items are lost |  | Randomise items |  | |
| 1400 | Mode 2: Randomise with WeiDU. Some items are lost |  | Randomise items |  | |
| 5005 | Beholders have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5015 | Demi-liches have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5025 | Dragons have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5035 | Elementals have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5045 | Fiends have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5055 | Golems have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5065 | Master Brains have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5075 | Slimes have no items equipped | Components for unequipping items from creature groups |  |  | |
| 5085 | Trolls have no items equipped | Components for unequipping items from creature groups |  |  | |
| 9000 | Cespenar can forge SoA items |  |  | ✓ | default |
| 9050 | Make Gromnir a proper Barbarian |  |  |  | |
| 10100 | All scrolls from all stores |  | Remove Protection from Undead scrolls from stores |  | |
| 10200 | All scrolls from 9 out of 10 stores |  | Remove Protection from Undead scrolls from stores | ✓ | default |
| 10210 | Duergar merchants |  |  | ✓ | default |
| 10300 | Prevent Watcher's Keep statues from disappearing |  |  | ✓ | default |

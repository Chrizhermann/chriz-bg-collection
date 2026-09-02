# XAN — components

Listed at installed version **v19**.
7 entries, 2 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Whenever the parent Xan mod is selected, component `0` is included without a separate
  component toggle.
- Components `1`–`4` form an optional choice group and all require component `0`. The UI
  needs an explicit **Keep Xan as an Enchanter** choice that installs none of them;
  Fighter/Mage (`1`) is the default choice.
- Artisan's Kitpack NPC component `20002` requires Xan component `1` and Artisan's
  Kitpack component `20000`. On EET, that selection also requires the separate local
  `XAN_EK_FIX` component `0` so the BG1 and transition CRE variants receive the
  Eldritch Knight conversion. That local fix is not catalogued in this file.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Xan NPC MOD for Baldur's Gate II |  |  | ✓ | mandatory |
| 1 | Change Xan's class to Fighter/Mage |  | Install alternate class for Xan? | ✓ | default |
| 2 | Change Xan's class to Mage |  | Install alternate class for Xan? |  | optional |
| 3 | Change Xan's class to Sorcerer |  | Install alternate class for Xan? |  | optional |
| 4 | Change Xan's class to Wild Mage (ToB only) |  | Install alternate class for Xan? |  | optional |
| 5 | BG1-style flaming swords |  |  |  | |
| 6 | Xan's Alternate Voice by Joey Bracken |  |  |  | |

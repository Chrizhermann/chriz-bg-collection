# ARTISANSKITPACK_TWEAK — components

Refreshed for the selected Artisan fork release **chriz-v1.2.0**
(`f623045f58cb5c84ebb438f9ce32b1741405c637`).
17 entries, 7 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

## UI/dependency notes

- `20101→2010`, `12012→2012`, `1209→1008/1009`, `3202/3302→3002+EEex`,
  `7203→7003`, `8204→8004+EEex`, and `20021→2002+2000+EEex`.
- `20101/12012` run after item-adding mods; `7203` runs after all kit mods.
- `1110` is intended as an automatic late repair only when Samurai `1010` and EEex are
  selected. It stays unavailable while `1010` is uncurated; do not treat it as globally
  `mandatory` merely because this TP2 is included.
- `1010000` is intended as an automatic late repair only when the Artisan
  weapon-usability infrastructure exists. It stays unavailable until the manifest can
  express that predicate; it is not a gameplay option or a global mandatory component.
- `300010` is unavailable while its unsafe parent `30001` is unavailable.
- `2000001` requires EEex.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 20101 | Artisan's Kitpack: Archer - Apply Manyshot to mod-added items (install this after any mods that add mod bows and crossbows) |  |  | ✓ | default |
| 12012 | Artisan's Kitpack: Beastmaster - Modify Restrictions for Mod Items (install this after any mods that add mod axes, daggers, spears, armor, etc.) |  |  | ✓ | default |
| 20021 | Artisan's Kitpack: Dark Hunter - Allow Disarm Traps and Open Locks |  |  |  | |
| 3210 | Artisan's Kitpack: Cavalier - Rename Kit to Chevalier |  |  |  | |
| 1209 | Artisan's Kitpack: Dreadnought - Set Dreadnought and Siegemaster to 1 APR (install this if the Dreadnought or Siegemaster is getting bonus APR from other mods) |  |  | ✓ | default |
| 1110 | Artisan's Kitpack: Samurai - Add important effects to mod-added items installed after the kit |  |  |  | |
| 3202 | Artisan's Kitpack: Mystic Fire - Allow Wands and Mage Scrolls |  |  | ✓ | default |
| 3302 | Artisan's Kitpack: Mystic Fire - Add spells from HLAs directly to spellbook up to maximum paladin casting level |  |  | ✓ | default |
| 7203 | Artisan's Kitpack: Trickster - Mimic Mod Kit Abilities (install this after other kits!) |  |  | ✓ | default |
| 8204 | Artisan's Kitpack: Arcane Trickster (Sorcerer) - Enable Thief Equipment |  |  | ✓ | default |
| 110000 | Artisan's Kitpack: Adjust kit THAC0 progression to match tweaked THAC0 tables |  |  |  | |
| 1 | Artisan's Kitpack: Mage - Rename to Wizard |  |  |  | |
| 2 | Artisan's Kitpack: Thief - Rename to Rogue |  |  |  | |
| 300010 | Artisan's Kitpack: Favored Soul - Patch Items and Spells from mods installed after the kitpack |  |  |  | |
| 1010000 | Artisan's Kitpack: Update EEex-based Weapon Usability Changes |  |  |  | |
| 1020000 | Artisan's Kitpack: Dual-Classes Keep Kit Names in Character Record |  |  |  | |
| 2000001 | Artisan's Kitpack: Modal Abilities Menu (EEex) |  |  |  | |

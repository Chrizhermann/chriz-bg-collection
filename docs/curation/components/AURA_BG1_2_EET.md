# AURA_BG1_2_EET — components

Listed at installed version **—**; refreshed against upstream
`285dabbcd5faee9f77cea34d9d51d53b0c08999b`.
15 entries, 4 installed. ✓ = installed. Subgroup = choose one.

## Version 1 policy

Aura is deferred from version 1 of the collection. The installer must not download,
offer, or select Aura or any of its child components. The installed checkmarks below
remain only as evidence of the historical reference stack; blank decisions mean excluded.
Keep this catalog for a later release rather than deleting the reference evidence.

## UI/dependency notes

- `0`, `1`, and `2` are BGEE-only, BG2EE-only, and EET-only parent installs. Only `2`
  would be valid for this collection if Aura is reconsidered after version 1.
- `3`–`6` are a required single class choice whenever Aura is enabled. Because none is
  default, the UI must ask explicitly instead of silently choosing one. `6` requires EEex.
- `7` and `14` require BG2 content (`1` or `2`). `8`–`13` are a separate mutually
  exclusive portrait subgroup requiring any main install (`0`, `1`, or `2`); they do not
  depend on `7`.
- Chris's balance fork is not yet a safe “latest” pin: it has unreleased local work but
  is missing seven newer upstream commits, including quest/cutscene fixes. Reconcile and
  publish it before the manifest uses it.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Aura NPC for Baldur's Gate: Enhanced Edition |  |  |  | |
| 1 | Aura NPC for Baldur's Gate 2: Enhanced Edition |  |  |  | |
| 2 | Aura NPC for Baldur's Gate: Enhanced Edition Trilogy |  |  | ✓ | |
| 3 | Artificer (Thief) |  | Choose a class for Aura: | ✓ | |
| 4 | Illusionist / Artificer (Mage / Thief) |  | Choose a class for Aura: |  | |
| 5 | Priestess / Artificer (Cleric / Thief) |  | Choose a class for Aura: |  | |
| 6 | Artificer (Bard, EEex required) |  | Choose a class for Aura: |  | |
| 7 | Aura NPC for Baldur's Gate 2: Enhanced Edition-NPC Portraits |  |  | ✓ | |
| 8 | Alternate 1 (Omar Diaz) |  | Choose an alternate portrait for Aura: |  | |
| 9 | Alternate 2 (Pantalion) |  | Choose an alternate portrait for Aura: |  | |
| 10 | Alternate 3 (The Artisan, edited from Neverwinter Nights) |  | Choose an alternate portrait for Aura: |  | |
| 11 | Original BG1 Default (Lava Del'Vortel) |  | Choose an alternate portrait for Aura: |  | |
| 12 | Second BG1 Default (Lava Del'Vortel) |  | Choose an alternate portrait for Aura: |  | |
| 13 | Original BG2 Default (The Artisan) |  | Choose an alternate portrait for Aura: |  | |
| 14 | Aura NPC for Baldur's Gate 2: Enhanced Edition-Import BG1 upgrades into BG2 |  |  | ✓ | |

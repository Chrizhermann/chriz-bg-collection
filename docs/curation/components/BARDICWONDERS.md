# BARDICWONDERS — components

Listed at reference version **—**; refreshed for Chris's fork release
**v2.9c-balance.2** (`db0cf81504fd3f84e4e74eb8ab30e65499135512`).
26 entries, 21 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

## UI/dependency notes

- `2001` is not a fix component: it creates the Trademeet vendor Leanne, a store, and a
  large item pack including the powerful Dirge +5 and Lament +5. It is excluded.
- `1006` is unavailable with Spell Revisions. The fork's `1008`, `1009`, and `2008`
  still need semantic Spell Revisions fixture coverage before broad compatibility is claimed.
- `1012` and `2009` require EEex. `3001` requires `1012` or `2009` and installs late.
- Component `1008` stays selected for the Troubadour kit, but its interactive Garrick
  assignment must be scripted as **No**; Artisan NPC `99001` is the single deterministic
  Garrick assignment.
- The released fork does not yet contain the later local Abettor finite-HLA rebalance.
  Keep that work separate until it is integrated, tested, and released.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1001 | Bardic Wonders: Blade Overhaul | Kits |  | ✓ | default |
| 1002 | Bardic Wonders: Jester Overhaul | Kits |  | ✓ | default |
| 1003 | Bardic Wonders: Skald Overhaul | Kits |  | ✓ | default |
| 1004 | Bardic Wonders: Abettor of Mask Kit | Kits |  | ✓ | default |
| 1005 | Bardic Wonders: Dancer Kit | Kits |  | ✓ | default |
| 1006 | Bardic Wonders: Darkbloom Bard Kit | Kits |  | ✓ | default |
| 1007 | Bardic Wonders: Storm Drummer Kit | Kits |  | ✓ | default |
| 1008 | Bardic Wonders: Troubadour Kit | Kits |  | ✓ | default |
| 1009 | Bardic Wonders: Deathsinger Kit | Kits |  | ✓ | default |
| 1010 | Bardic Wonders: Strategist Kit | Kits |  | ✓ | default |
| 1011 | Bardic Wonders: Kapellmeister Kit | Kits |  | ✓ | default |
| 1012 | Bardic Wonders: Gallant Paladin Kit | Kits |  | ✓ | default |
| 2001 | Bardic Wonders: Items | Tweaks & Additions |  | ✓ | |
| 2002 | Bardic Wonders: Inspirations | Tweaks & Additions |  | ✓ | default |
| 2007 | Bardic Wonders: High Level Abilities | Tweaks & Additions |  | ✓ | default |
| 2008 | Bardic Wonders: New Bard Spells | Tweaks & Additions |  | ✓ | default |
| 2003 | Bardic Wonders: Armored Casting for Bards | Tweaks & Additions |  | ✓ | default |
| 2004 | Bardic Wonders: Bard Song Mechanics Tweak | Tweaks & Additions |  | ✓ | default |
| 2005 | Bardic Wonders: Item Restriction Tweaks—Bards may use mage items and priest scrolls | Tweaks & Additions |  | ✓ | default |
| 2012 | Bardic Wonders: Item Restriction Tweaks—Bards may use Small and Medium Shields | Tweaks & Additions |  |  | |
| 2013 | Bardic Wonders: Item Restriction Tweaks—Bards may use Helmets | Tweaks & Additions |  |  | |
| 2006 | Bardic Wonders: Bard Song Overhead Visual Effect | Tweaks & Additions |  | ✓ | default |
| 2009 | Bardic Wonders: Mage/Bard and Fighter/Mage/Bard multi-class (EEex, under thief multis) | Tweaks & Additions |  |  | |
| 2010 | Bardic Wonders: 3e Alignment for Bards (any non-lawful) | Tweaks & Additions |  |  | |
| 2011 | Bardic Wonders: Bard Casting Level Begins from 2nd level | Tweaks & Additions |  |  | |
| 3001 | Bardic Wonders: Patch Dialogue Class Checks for: Gallant, Mage/Bard and Fighter/Mage/Bard (install late) | Patches |  | ✓ | default |

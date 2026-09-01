# ARTISANSKITPACK — components

Listed at reference version **6.0**; refreshed for the selected fork release
**chriz-v1.2.0** (`f623045f58cb5c84ebb438f9ce32b1741405c637`).
59 entries, 47 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

## UI/dependency notes

- `2/3`, `8101/8102`, `8002/8003`, `1007/1002`, and `5110/5111` are
  mutually exclusive choices.
- `8101/8102` require `8001` and are unavailable with Spell Revisions.
- `10003/10004` require `10001` and EEex; `8004` also requires EEex.
- `1001` and `2003` require `1000`; `5110/5111` require `5100`.
- `5110/5111` retain Chris's preference but must be unavailable until the local
  Shapeshifter pathfinding-footprint fix is included in a fetchable fork release.
- `30001` is unavailable for every recommended collection route. Its global priest
  delivery is based on a stale snapshot and remains unsafe even without Spell Revisions.
- `1` genuinely broadens ordinary class/race availability, but deliberately does not
  remove every kit, dual-class, or gnome Mage restriction. No currently selected
  Tweaks Anthology component duplicates `1/2`; future overlapping `2350/2351/2353/2357/2358`
  and `2550`–`2552` choices must be mutually exclusive in the UI.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1 | Artisan's Kitpack: Enable All Classes for All Races | Rule Tweaks |  | ✓ | default |
| 2 | All Races | Rule Tweaks | Artisan's Kitpack: Enable Multi-Classes for | ✓ | default |
| 3 | All Races Except Human | Rule Tweaks | Artisan's Kitpack: Enable Multi-Classes for |  | |
| 20000 | Artisan's Kitpack: Eldritch Knight (Fighter / Mage Kit) | Multi-Class Kits |  | ✓ | default |
| 20001 | Artisan's Kitpack: Arcane Trickster (Mage / Thief Kit) | Multi-Class Kits |  | ✓ | default |
| 8001 | Artisan's Kitpack: Pale Master Sorcerer Kit | Sorcerer Kits |  | ✓ | default |
| 8101 | 'Unholy' Necromancy Spells Only (Cause Wounds, Disease, Poison, etc.) | Sorcerer Kits | Artisan's Kitpack: Pale Master Spells | ✓ | default |
| 8102 | All Necromancy Divine Spells (including Cure Wounds, Neutralize Poison, etc.) | Sorcerer Kits | Artisan's Kitpack: Pale Master Spells |  | |
| 8002 | Original Stat Bonuses | Sorcerer Kits | Artisan's Kitpack: 3e-accurate Dragon Disciple | ✓ | default |
| 8003 | Nerfed stat bonuses (net +2 to STR) | Sorcerer Kits | Artisan's Kitpack: 3e-accurate Dragon Disciple |  | |
| 8004 | Artisan's Kitpack: Arcane Trickster Sorcerer Kit—EEEx required | Sorcerer Kits |  | ✓ | default |
| 10002 | Artisan's Kitpack: Brawler Kit | Monk Kits |  | ✓ | default |
| 10001 | Artisan's Kitpack: Monk Revisions | Monk Kits |  | ✓ | default |
| 10003 | Artisan's Kitpack: Sacred Fist (Cleric) and Enlightened Fist (Sorcerer) Kits—EEEx required | Monk Kits |  | ✓ | default |
| 10004 | Artisan's Kitpack: Ninja "Class" (Monk Kit)—EEEx required | Monk Kits |  | ✓ | default |
| 1003 | Artisan's Kitpack: Berserker Overhaul | Fighter Kits |  | ✓ | default |
| 1006 | Artisan's Kitpack: Wizard Slayer Overhaul | Fighter Kits |  | ✓ | default |
| 1004 | Artisan's Kitpack: Kensai Overhaul | Fighter Kits |  | ✓ | default |
| 1005 | Artisan's Kitpack: Barbarian Overhaul | Fighter Kits |  | ✓ | default |
| 1007 | Artisan's Kitpack: Dwarven Defender Overhaul + Vanguard Fighter Kit | Fighter Kits |  | ✓ | default |
| 1002 | Artisan's Kitpack: Vanguard Fighter Kit | Fighter Kits |  |  | |
| 1000 | Artisan's Kitpack: Arcane Archer Fighter Kit | Fighter Kits |  | ✓ | default |
| 1001 | Artisan's Kitpack: Arcane Archer (Fighter/Mage) | Fighter Kits |  | ✓ | default |
| 1008 | Artisan's Kitpack: Siegemaster Fighter Kit | Fighter Kits |  | ✓ | default |
| 1009 | Artisan's Kitpack: Dreadnought Fighter Kit | Fighter Kits |  | ✓ | default |
| 1010 | Artisan's Kitpack: Samurai Fighter Kit | Fighter Kits |  |  | |
| 1100 | Artisan's Kitpack: Fighter Overhaul | Fighter Kits |  | ✓ | default |
| 2000 | Artisan's Kitpack: Ranger Overhaul | Ranger Kits |  | ✓ | default |
| 2010 | Artisan's Kitpack: Archer Overhaul | Ranger Kits |  | ✓ | default |
| 2011 | Artisan's Kitpack: Stalker Overhaul | Ranger Kits |  | ✓ | default |
| 2012 | Artisan's Kitpack: Beast Master Overhaul | Ranger Kits |  | ✓ | default |
| 2002 | Artisan's Kitpack: Dark Hunter Ranger Kit | Ranger Kits |  | ✓ | default |
| 2003 | Artisan's Kitpack: Arcane Archer (Ranger variant) | Ranger Kits |  |  | |
| 3000 | Artisan's Kitpack: Paladin Overhaul | Paladin Kits |  | ✓ | default |
| 3010 | Artisan's Kitpack: Cavalier Overhaul | Paladin Kits |  | ✓ | default |
| 3003 | Artisan's Kitpack: Inquisitor Overhaul | Paladin Kits |  | ✓ | default |
| 3011 | Artisan's Kitpack: Undead Hunter Overhaul | Paladin Kits |  | ✓ | default |
| 3004 | Artisan's Kitpack: Blackguard Overhaul | Paladin Kits |  | ✓ | default |
| 3001 | Artisan's Kitpack: Divine Champion Paladin Kit | Paladin Kits |  | ✓ | default |
| 3002 | Artisan's Kitpack: Mystic Fire Paladin Kit | Paladin Kits |  | ✓ | default |
| 3005 | Artisan's Kitpack: Martyr Paladin Kit | Paladin Kits |  | ✓ | default |
| 5300 | Artisan's Kitpack: Totemic Druid Overhaul | Druid Kits |  |  | |
| 5100 | Artisan's Kitpack: Shapeshifter Overhaul | Druid Kits |  | ✓ | default |
| 5110 | Shapeshifter Form Only | Druid Kits | Artisan's Kitpack: Shapeshifter Overhaul – Custom Werewolf Sprite | ✓ | default |
| 5111 | Replace all NPC Werewolf Sprites | Druid Kits | Artisan's Kitpack: Shapeshifter Overhaul – Custom Werewolf Sprite |  | |
| 5200 | Artisan's Kitpack: Avenger Overhaul | Druid Kits |  |  | |
| 5001 | Artisan's Kitpack: Elementalist Druid Kit | Druid Kits |  | ✓ | default |
| 5002 | Artisan's Kitpack: Hivemaster Druid Kit | Druid Kits |  | ✓ | default |
| 7000 | Artisan's Kitpack: Thief Overhaul | Thief Kits |  |  | |
| 7004 | Artisan's Kitpack: Assassin Overhaul | Thief Kits |  | ✓ | default |
| 7007 | Artisan's Kitpack: Bounty Hunter Overhaul | Thief Kits |  |  | |
| 7006 | Artisan's Kitpack: Swashbuckler Overhaul | Thief Kits |  | ✓ | default |
| 7008 | Artisan's Kitpack: Shadowdancer Overhaul | Thief Kits |  |  | |
| 7001 | Artisan's Kitpack: Rogue Archer Thief Kit | Thief Kits |  | ✓ | default |
| 7002 | Artisan's Kitpack: Magekiller Thief Kit | Thief Kits |  | ✓ | default |
| 7003 | Artisan's Kitpack: Trickster Thief Kit | Thief Kits |  | ✓ | default |
| 7005 | Artisan's Kitpack: Invisible Blade Thief Kit | Thief Kits |  | ✓ | default |
| 9001 | Artisan's Kitpack: Warhorn Shaman Kit | Shaman Kits |  | ✓ | default |
| 30001 | Artisan's Kitpack: Favored Soul (EEex only, under Shaman in chargen) | Other |  | ✓ | |

The selected source is Chris's released fork, which already contains the Berserker
rebalance. Do not install the standalone `AKCB_BERSERKER` component as well.

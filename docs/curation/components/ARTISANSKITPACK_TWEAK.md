# ARTISANSKITPACK_TWEAK — components

Refreshed for the selected Artisan fork release **v4.81a-dev.85928f9-chriz.1**
(`0798dc4ee3d09b5d97c56b69c80d9d2ac2317911`). The recipes pin
**v4.81a-dev.85928f9-chriz.4** (tag commit `afa3316`, pinned 2026-10-27): its tweak TP2
declares the same 18 components in the same order. The tweak installer uses the main
installer's libraries, so the sorcerer and kit weapon fixes (see
[ARTISANSKITPACK](ARTISANSKITPACK.md)) reach `8204` (Arcane Trickster sorcerer thief
equipment, selected in Recommended) and `1010000` (Update EEex-based Weapon Usability
Changes, selected in Recommended since 2026-10-29). Chains that mods installed after
Kitpack register in `C0PR#XX`, such as Bardic Wonders' Mage/Bard `2009` (excluded here),
join `C0PR#KG` only through `1010000`; without it they behave as before. Besides its version,
chriz.4's tweak TP2 only ends `1020000` (Dual-Classes Keep Kit Names, not offered) with
the pass that puts "apply `C0PR#CL`" back into every class and kit ability table; with
chriz.4, `1010000` also gives that entry to ability tables that later mods added or
replaced. No component or decision changes.
18 entries, 7 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

## UI/dependency notes

- `20101→2010`, `12012→2012`, `1209→1008/1009`, `3202/3302→3002+EEex`,
  `7203→7003`, `8204→8004+EEex`, and `20021→2002+2000+EEex`.
- `20101/12012` run after item-adding mods; `7203` runs after all kit mods.
- `1110` is intended as an automatic late repair only when Samurai `1010` and EEex are
  selected. It stays unavailable while `1010` is uncurated; do not treat it as globally
  `mandatory` merely because this TP2 is included.
- `1010000` is a late repair: it sweeps weapons that mods installed after the Kitpack
  add into the Kitpack's EEex-based weapon rules (`C0PR#CL`/`C0PR#KG`). Christopher
  approved it on 2026-10-26 ("Ja, aufnehmen": take it in); a default since 2026-10-29.
  Its TP2 installs it only when `C0PR#CL.SPL` exists. Within the collection only
  Eldritch Knight `20000` (with EEex) and Arcane Trickster sorcerer equipment `8204`
  (which requires EEex) create that file; Samurai `1010` and Favored Soul `30001` would
  too but are not offered. The feature therefore requires EEex plus `20000` or `8204`.
  It is the last component of the late run, after `8204`, after Tweaks Anthology and
  after SCS; weapons that runs after it add are not swept.
- What `1010000` changed in the October 29 EET lab (curated Recommended with Sirene NPC
  for BG:EE from PR #58; every swept file compared with its state before the sweep): 20
  items came under the Kitpack's weapon rules. A second, complete Recommended lab the
  same day gave the identical result. The static analysis of 2026-10-26 named Tweaks
  Anthology's two-handed axes and the Eldritch Knight with the Drow Katana; the lab data
  corrects both.
  - IWDification's eight two-handed axes (`60` and `140`, curated recipe only) are no
    longer closed to clerics, mages and thieves, so the Priest of Tempus, the Eldritch
    Knight and the Eldritch Knight sorcerer can use them.
  - The SCS Drow Katana becomes usable for the Arcane Trickster sorcerer. The fighter/mage
    Eldritch Knight and the mage/thief Arcane Trickster could use it before. The Eldritch
    Knight sorcerer still cannot: `8204` gave the katana a check that keeps out every
    single-class mage and sorcerer except the Arcane Trickster sorcerer (Kitpack
    behaviour, kept as-is; open question in `FOLLOW_UPS.md`).
  - Sirene NPC for BG2's own weapons (Celestial Fire, Harbinger of Pain) and Tweaks
    Anthology's two-handed copies of Celestial Fire, six items, but only because PR #58
    installs Sirene after the Kitpack's main run. In the current order that run already
    covers them. Celestial Fire stays lawful good only.
  - SCS's sahuagin Spear +2 and a Spell Revisions creature's spear, now also open to kits
    with spear access such as the Priest of Talos. IWDification's Decastave (a staff check
    that every class passes) and two SCS arrows (clerics and druids get no bow) change
    nothing in practice.
  - Tweaks Anthology's two-handed axes (`2035`) did not change: they are copies of axes
    the Kitpack had already converted. The other 806 swept items only got a second,
    identical weapon check, because the sweep adds its `SPLPROT.2DA` rows again; no
    in-game effect.
  - Unchanged: the plain dagger, quarterstaff and dart, the mage and sorcerer grants for
    daggers, quarterstaves, darts and slings, the monk's quarterstaff grant, and the
    Eldritch Knight and Arcane Trickster sorcerer weapon chains.
- `300010` is unavailable while its unsafe parent `30001` is unavailable.
- `2000001` requires EEex.
- `51010` is optional and off by default; requires Shapeshifter `5100` OR
  Hivemaster `5002`, with no EEex dependency. It is first in the released tweak
  TP2, so the late run selects it before `12012` when enabled. Five real-WeiDU
  synthetic checks were reported by the owner; no live-game acceptance is implied.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 51010 | Allow talking in Shapeshifter and Hivemaster forms | Quality of life |  |  | optional |
| 20101 | Artisan's Kitpack: Archer - Remove legacy Manyshot from mod-added items (compatibility cleanup; chriz-v1.6.0) |  |  | ✓ | default |
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
| 1010000 | Artisan's Kitpack: Update EEex-based Weapon Usability Changes |  |  |  | default |
| 1020000 | Artisan's Kitpack: Dual-Classes Keep Kit Names in Character Record |  |  |  | |
| 2000001 | Artisan's Kitpack: Modal Abilities Menu (EEex) |  |  |  | |

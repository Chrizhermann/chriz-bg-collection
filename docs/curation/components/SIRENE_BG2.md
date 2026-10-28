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
- Sirene BG1 is a separate mod, not a component of this package. It has its own catalog,
  [SIRENE_BG1](SIRENE_BG1.md), and its own source row: Christopher included it as a
  default on 2026-10-29, pinned to unreleased `master` commit `1c86000` because release
  v3.1 supports BG:EE only. On EET this mod continues a Sirene who travelled with the
  party from SoD into SoA (`AR0903_e.baf`) only when Sirene BG1 is installed first.
- Since 2026-10-29 both Sirene mods install after Artisan's Kitpack's main run, so the
  Martyr kit comes from the Kitpack (`3005`): each Sirene mod skips its own copy of the
  kit when `3005` is installed. See SIRENE_BG1's install-order notes.
- The move puts this mod after Spell Revisions' main component, Bardic Wonders' main run
  and the Kitpack's main run, so their global passes no longer reach its files. In the
  October 18 Recommended lab (WeiDU backups) they changed 11 of them: Spell Revisions `0`
  `C0ILMN4.SPL`, `C0SBLIND.SPL` and `C0SRING2.ITM`; Bardic Wonders `2003` the armor
  `C02PLAT`/`C0SLEAT` and `2005` the robe `C0SROBE`; Kitpack `20000` `C02PLAT`, `C0SLEAT`,
  `C0SBLUN`, `C0SS1H1` and `C0SS1H2`, `10001`/`10003` `C0SW2H`/`C0SW2H2`, `1006`
  `C0SRING2`, `7005` `C02PLAT`/`C0SLEAT` and `7006` `C02PLAT`. Spell Revisions' NPC
  spellbooks `60` runs after EET_end and still reaches Sirene.

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

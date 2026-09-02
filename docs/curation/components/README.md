# Component catalogs (Phase 0.4, per mod)

Full component menus as WeiDU lists them at the *installed* version, generated 2026-08-21,
with explicit target-version refreshes called out in individual catalogs.
✓ = installed in the reference unless a refreshed catalog explicitly labels a target/dev
preset; the index counts the marks shown in that file. `Subgroup` = mutually exclusive
options (pick one).
Decision column is Chris's. Where the pin-list targets a newer version the menu may differ — re-list after upgrade.

This directory indexes the full menus generated for the mods listed below; it is not one
file per manifest entry because soundsets and pseudo-installers are intentionally outside
the component UI. The 22 historical single-component local fixes are all covered by the
[collection tail-fix migration inventory](COLLECTION_TAIL_FIXES.md). Every unresolved
release, installer, and live-acceptance check is indexed separately in
[curation follow-ups](FOLLOW_UPS.md); detailed evidence stays in each mod catalog.
For the prioritized next working session, start with [the short checklist](../../next-session.md).

## Decision semantics

Use exactly one of these values in the `Decision` column:

- blank — exclude the component; do not expose it as an option
- `optional` — expose it, unchecked by default
- `default` — expose it, checked by default
- `mandatory` — include it without a component toggle whenever the parent mod is included

Choice groups, prerequisites, incompatibilities, and UI explanations belong in `Subgroup`
or a file-specific UI/dependency note, not in the `Decision` value.

| Mod | Listed | ✓ count | File |
|---|---|---|---|
| DLCMERGER | 4 | 1 | [DLCMERGER.md](DLCMERGER.md) |
| BG1UB | 35 | 16 | [BG1UB.md](BG1UB.md) |
| BG1NPC | 49 | 9 | [BG1NPC.md](BG1NPC.md) |
| EEFIXPACK | 3 | 2 | [EEFIXPACK.md](EEFIXPACK.md) |
| EET | 2 | 2 | [EET.md](EET.md) |
| EEEX | 9 | 8 | [EEEX.md](EEEX.md) |
| BUBB_SPELL_MENU_EXTENDED | 1 | 1 | [BUBB_SPELL_MENU_EXTENDED.md](BUBB_SPELL_MENU_EXTENDED.md) |
| BGGO | 4 | 1 | [BGGO.md](BGGO.md) |
| HIDDENGAMEPLAYOPTIONS | 44 | 27 | [HIDDENGAMEPLAYOPTIONS.md](HIDDENGAMEPLAYOPTIONS.md) |
| RR | 14 | 7 | [RR.md](RR.md) |
| BRANWEN | 1 | 1 | [BRANWEN.md](BRANWEN.md) |
| EVANDRA | 2 | 2 | [EVANDRA.md](EVANDRA.md) |
| FADE | 3 | 1 | [FADE.md](FADE.md) |
| PAINA | 1 | 1 | [PAINA.md](PAINA.md) |
| SARAHTOB | 2 | 1 | [SARAHTOB.md](SARAHTOB.md) |
| AURA_BG1_2_EET | 15 | 4 | [AURA_BG1_2_EET.md](AURA_BG1_2_EET.md) |
| UB | 24 | 22 | [UB.md](UB.md) |
| XAN | 7 | 2 | [XAN.md](XAN.md) |
| YESLICKNPC | 2 | 1 | [YESLICKNPC.md](YESLICKNPC.md) |
| AJANTISBG2 | 9 | 1 | [AJANTISBG2.md](AJANTISBG2.md) |
| SIRENE_BG2 | 9 | 3 | [SIRENE_BG2.md](SIRENE_BG2.md) |
| ASCENSION | 19 | 16 | [ASCENSION.md](ASCENSION.md) |
| SPELL_REV | 8 | 7 | [SPELL_REV.md](SPELL_REV.md) |
| ARTISANSKITPACK | 59 | 47 | [ARTISANSKITPACK.md](ARTISANSKITPACK.md) |
| BARDICWONDERS | 26 | 21 | [BARDICWONDERS.md](BARDICWONDERS.md) |
| ARTISANSKITPACK_NPC | 18 | 6 | [ARTISANSKITPACK_NPC.md](ARTISANSKITPACK_NPC.md) |
| IWDIFICATION | 23 | 8 | [IWDIFICATION.md](IWDIFICATION.md) |
| IEPBANTERS | 12 | 7 | [IEPBANTERS.md](IEPBANTERS.md) |
| CROSSMODBG2 | 3 | 3 | [CROSSMODBG2.md](CROSSMODBG2.md) |
| ARTISANSKITPACK_TWEAK | 17 | 7 | [ARTISANSKITPACK_TWEAK.md](ARTISANSKITPACK_TWEAK.md) |
| HQ_SOUNDCLIPS_BG2EE | 1 | 1 | [HQ_SOUNDCLIPS_BG2EE.md](HQ_SOUNDCLIPS_BG2EE.md) |
| C0WARLOCK | 1 | 1 | [C0WARLOCK.md](C0WARLOCK.md) |
| RANDOMISER | 26 | 9 | [RANDOMISER.md](RANDOMISER.md) |
| CDTWEAKS | 429 | 62 | [CDTWEAKS.md](CDTWEAKS.md) |
| STRATAGEMS | 141 | 76 | [STRATAGEMS.md](STRATAGEMS.md) |
| EET_END | 2 | 1 | [EET_END.md](EET_END.md) |
| AURA_BALANCE_PATCH_SPELLS | 1 | 1 | [AURA_BALANCE_PATCH_SPELLS.md](AURA_BALANCE_PATCH_SPELLS.md) |
| BG2EE-EET-FIXPACK | 5 | 5 | [BG2EE-EET-FIXPACK.md](BG2EE-EET-FIXPACK.md) |
| EET_TWEAKS | 64 | 1 | [EET_TWEAKS.md](EET_TWEAKS.md) |
| CHRIZ-BG-MODPACK | 26 | 4 | [CHRIZ-BG-MODPACK.md](CHRIZ-BG-MODPACK.md) |
| SAFANA | 1 | 1 | [SAFANA.md](SAFANA.md) |
| AKCB_BERSERKER | 1 | 1 | [AKCB_BERSERKER.md](AKCB_BERSERKER.md) |
| CHRIZ-SOD-REMIX | 31 | 30 | [CHRIZ-SOD-REMIX.md](CHRIZ-SOD-REMIX.md) |
| EEEXREMOTE | 1 | 1 | [EEEXREMOTE.md](EEEXREMOTE.md) |
| CHRIZ-BG-REBALANCE | 12 | 7 | [CHRIZ-BG-REBALANCE.md](CHRIZ-BG-REBALANCE.md) |
| ABETTORHLAREBALANCE | 1 | 1 | [ABETTORHLAREBALANCE.md](ABETTORHLAREBALANCE.md) |
| AKCB_SHAPESHIFTER | 1 | 1 | [AKCB_SHAPESHIFTER.md](AKCB_SHAPESHIFTER.md) |

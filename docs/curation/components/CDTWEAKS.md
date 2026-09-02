# CDTWEAKS — components

Listed at installed version **v18**.
429 entries, 62 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Every non-empty `Subgroup` is an at-most-one choice. The `default` row is initially
  selected; `optional` alternatives remain visible and choosing none remains valid unless
  the parent feature says otherwise.
- `2312` scans every arcane and divine spell visible at install time and adds
  caster-level-based save penalties. Keep it selected by default, but resolve its final
  order against Spell Revisions, IWDification, and SCS so later spell copies do not escape
  the patch.
- `2420` and `2430` are optional broad usability changes. Their upstream implementation
  also updates `CLASWEAP.2DA` and `WEAPPROF.2DA`, but the Enhanced Edition still has a
  documented dual-class proficiency-picker limitation unless the separate configuration
  workaround is enabled. Validate them with the final Artisan proficiency/item rules;
  ordinary multiclasses such as the planned Viconia Cleric/Thief are not affected by that
  dual-class UI limitation.
- `3347` is optional. When selected, ship the curated `cdtweaks.txt` values from the
  reference setup: movement enabled at `125` percent, casting-speed increase disabled,
  and the portrait icon enabled.
- `3390` is optional. EET satisfies its upstream availability predicate; it extends the
  final tracking-area scripts, so validate its order with IWDification and SCS.
- `2720` and `3121` are deliberate new defaults rather than reference-install choices.
  `2720` can help an existing campaign only before the Yoshimo-to-Imoen Spellhold transfer
  has occurred. `3121` is the maintained happiness implementation and replaces the
  duplicate BG2EE-EET-FIXPACK component `300`; characters already below the leave-party
  threshold need a separate one-time save repair.
- Keep `260` unavailable with SCS; the combination has a known invisible-hostile/save
  failure path and is not part of this preset.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Batch Installer -- EXPERIMENTAL, use at own risk. Check the readme. | Cosmetic Changes, Content Changes, Rule Changes, Convenience Tweaks/Cheats, NPC Tweaks |  |  | |
| 10 | Remove Helmet Animations | Cosmetic Changes |  |  | optional |
| 20 | Change Imoen's Avatar to Mage | Cosmetic Changes |  |  | optional |
| 30 | Change Nalia's Avatar to Thief | Cosmetic Changes |  |  | optional |
| 40 | Change Viconia's Skin Color to Dark Blue | Cosmetic Changes |  | ✓ | default |
| 50 | Avatar Morphing Script | Cosmetic Changes |  |  | optional |
| 60 | Weapon Animation Tweaks | Cosmetic Changes |  | ✓ | default |
| 70 | Icewind Dale Casting Graphics [Andyr] | Cosmetic Changes |  |  | |
| 72 | Baldur's Gate Casting Graphics [Andyr] | Cosmetic Changes |  |  | |
| 80 | Restore SoA Load Screen Logo | Cosmetic Changes |  |  | |
| 82 | Restore IWD Loading Screens [icelus] | Cosmetic Changes |  |  | |
| 90 | Disable Portrait Icons Added by Equipped Items | Cosmetic Changes |  |  | |
| 100 | Commoners Use Drab Colors | Cosmetic Changes |  |  | optional |
| 110 | Icon Improvements | Cosmetic Changes |  | ✓ | default |
| 130 | Force All Dialogue to Pause Game | Cosmetic Changes |  | ✓ | default |
| 140 | Fix Boo's Squeak | Cosmetic Changes |  |  | |
| 150 | Remove "+x" From Unique Item Names | Cosmetic Changes |  |  | |
| 160 | Make Magic Shields Glow [plainab/grogerson] | Cosmetic Changes |  |  | |
| 170 | Only replace icons that aren't already unique | Cosmetic Changes | Unique Icons [Lava] | ✓ | default |
| 171 | Replace all icons | Cosmetic Changes | Unique Icons [Lava] |  | |
| 180 | Fixes only | Cosmetic Changes | Unique Containers [Miloch] | ✓ | default |
| 181 | Unique icons only | Cosmetic Changes | Unique Containers [Miloch] |  | optional |
| 182 | Unique icons and names | Cosmetic Changes | Unique Containers [Miloch] |  | optional |
| 190 | For all shields and helmets | Cosmetic Changes | Use Character Colors Instead of Item Colors |  | optional |
| 191 | For non-magical shields and helmets | Cosmetic Changes | Use Character Colors Instead of Item Colors |  | optional |
| 192 | For all helmets | Cosmetic Changes | Use Character Colors Instead of Item Colors |  | optional |
| 193 | For non-magical helmets | Cosmetic Changes | Use Character Colors Instead of Item Colors |  | optional |
| 194 | For all shields | Cosmetic Changes | Use Character Colors Instead of Item Colors |  | optional |
| 195 | For non-magical shields | Cosmetic Changes | Use Character Colors Instead of Item Colors |  | optional |
| 200 | Remove blur effect | Cosmetic Changes | Remove Annoying Visual Effects from Equipped Items |  | optional |
| 3150 | Remove spell trap and reflection effects | Cosmetic Changes | Remove Annoying Visual Effects from Equipped Items |  | |
| 3151 | Remove all of the above | Cosmetic Changes | Remove Annoying Visual Effects from Equipped Items |  | |
| 2010 | Separate Resist Fire/Cold Icon into Separate Icons [Angel] | Cosmetic Changes |  | ✓ | default |
| 220 | Enhanced Overlays for Colorblind Players [Fouinto] | Cosmetic Changes |  |  | |
| 230 | Restore IWD Tooltips | Cosmetic Changes |  |  | |
| 240 | Add Black Outline | Cosmetic Changes | Outline White Spell Icons for Accessibility |  | |
| 241 | Add Gray Outline | Cosmetic Changes | Outline White Spell Icons for Accessibility |  | |
| 250 | Normal brightness | Cosmetic Changes | Colorize NPC Names and Tooltips |  | |
| 251 | Slightly increased brightness | Cosmetic Changes | Colorize NPC Names and Tooltips |  | |
| 252 | Moderately increased brightness | Cosmetic Changes | Colorize NPC Names and Tooltips |  | |
| 253 | Slightly decreased brightness | Cosmetic Changes | Colorize NPC Names and Tooltips |  | |
| 254 | Moderately decreased brightness | Cosmetic Changes | Colorize NPC Names and Tooltips |  | |
| 255 | Original color | Cosmetic Changes | Colorize NPC Names and Tooltips |  | |
| 260 | Hide Visual Effects from Invisible Enemies [Luke] | Cosmetic Changes |  |  | |
| 270 | Visualize health and status changes | Cosmetic Changes | Static PsT Character Portraits |  | |
| 271 | Don't visualize health and status changes | Cosmetic Changes | Static PsT Character Portraits |  | |
| 1010 | More Interjections | Content Changes |  | ✓ | default |
| 1020 | Alter HP Triggers for NPC Wounded Dialogues | Content Changes |  | ✓ | default |
| 1030 | Reveal Wilderness Areas Before Chapter Six | Content Changes |  |  | |
| 1035 | First area only | Content Changes | Make Cloakwood Areas Available Before Completing the Bandit Camp |  | |
| 1036 | All of Cloakwood except the mines | Content Changes | Make Cloakwood Areas Available Before Completing the Bandit Camp |  | |
| 1040 | Improved Athkatlan City Guard | Content Changes |  | ✓ | default |
| 1050 | Gradual Drow Item Disintegration | Content Changes |  |  | |
| 1060 | Breakable Iron Non-Magical Shields, Helms, and Armor | Content Changes | Breakable Iron Non-Magical Shields, Helms, and Armor |  | |
| 1061 | Shields and helms break, armor degrades once | Content Changes | Breakable Iron Non-Magical Shields, Helms, and Armor |  | |
| 1062 | Shields and helms break, armor progressively degrades and breaks | Content Changes | Breakable Iron Non-Magical Shields, Helms, and Armor |  | |
| 1070 | Improved Multi-Player Kick-Out Dialogues | Content Changes |  |  | |
| 1075 | Send BioWare NPCs to an Inn [DavidW/Zed Nocear] | Content Changes |  |  | |
| 1080 | Add Bags of Holding | Content Changes |  | ✓ | default |
| 1085 | Portable Containers [Zed Nocear] | Content Changes |  |  | |
| 1100 | Reveal City Maps When Entering Area | Content Changes |  |  | |
| 1101 | Do Not Reveal City Maps When Entering Area | Content Changes |  |  | |
| 1110 | Add Map Notes | Content Changes |  |  | |
| 1120 | Stores Sell Higher Stacks of Items | Content Changes |  | ✓ | default |
| 1130 | Reputation Resets in BG2 | Content Changes |  |  | |
| 1140 | Gems and potions | Content Changes | Gems and Potions Require Identification |  | |
| 1141 | Just gems | Content Changes | Gems and Potions Require Identification |  | |
| 1142 | Just potions | Content Changes | Gems and Potions Require Identification | ✓ | default |
| 1150 | Shapeshifter Rebalancing [Weimer] | Content Changes |  |  | |
| 1160 | No restrictions | Content Changes | Multiple Strongholds [Sabre, Baldurdash, Weimer] |  | optional |
| 1161 | Keep class restrictions | Content Changes | Multiple Strongholds [Sabre, Baldurdash, Weimer] | ✓ | default |
| 1340 | Make the Planar Sphere Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1341 | Make the de'Arnise Keep Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1342 | Make the Temple Strongholds Available to All Classes | Content Changes |  |  | optional |
| 1343 | Make the Thieves Guild Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1344 | Make the Playhouse Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1345 | Make the Noble Order of the Radiant Heart Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1346 | Make the Druid Grove Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1347 | Make the Imnesvale Cabin Stronghold Available to All Classes | Content Changes |  |  | optional |
| 1170 | Bonus Merchants [Baldurdash] | Content Changes |  |  | |
| 1180 | Female Edwina [Davide Carte, Wendy Yung, Weimer] | Content Changes |  | ✓ | default |
| 1190 | Romance Bug Fixes [Sabre, Richardson] | Content Changes |  |  | |
| 1200 | Imoen ToB Dialogue Fix [jcompton] | Content Changes |  |  | |
| 1210 | Use BG Walking Speeds | Content Changes |  |  | |
| 1220 | Allow Cromwell to Upgrade Watcher's Keep Items | Content Changes |  | ✓ | default |
| 1225 | Instant forging (original BG2 default) | Content Changes | Adjust Cromwell's Forging Time |  | optional |
| 1226 | Eight hours | Content Changes | Adjust Cromwell's Forging Time |  | optional |
| 1227 | Full 24 hours (BG2EE default, includes sleep) | Content Changes | Adjust Cromwell's Forging Time | ✓ | default |
| 1230 | Allow Cespenar to Use Cromwell Recipes | Content Changes |  | ✓ | default |
| 1240 | Friendly Arm Inn Hidden Container Restoration [plainab] | Content Changes |  |  | |
| 1257 | Move NPCs to Convenient Locations: Alora, Eldoth, Quayle, Shar-Teel, Tiax, and Viconia | Content Changes |  | ✓ | default |
| 1251 | Move NPCs to Convenient Locations: Move Alora to Gullykin | Content Changes |  |  | |
| 1252 | Move NPCs to Convenient Locations: Move Eldoth to the Coast Way | Content Changes |  |  | |
| 1253 | Move NPCs to Convenient Locations: Move Quayle to the Nashkel Carnival | Content Changes |  |  | |
| 1254 | Move NPCs to Convenient Locations: Move Shar-Teel to North Nashkel Road | Content Changes |  |  | |
| 1255 | Move NPCs to Convenient Locations: Move Tiax to Beregost | Content Changes |  |  | |
| 1256 | Move NPCs to Convenient Locations: Move Viconia to South Beregost Road | Content Changes |  |  | |
| 1260 | Bardic Reputation Adjustment | Content Changes |  |  | |
| 1270 | Change Cloakwood Mine Chapter End Change Trigger to Non-TotSC Behavior [plainab] | Content Changes |  |  | |
| 1280 | Game Ends When the Main Character Dies | Content Changes |  |  | |
| 1290 | NPCs Respond to the Main Character, Not to Whichever Character Talks to Them | Content Changes |  |  | |
| 1300 | Make Heart of Winter Accessible at Any Level | Content Changes |  |  | |
| 1310 | Restore (Most) BG2 Spells and Make Scrolls Available  WARNING: They will look very out of place | Content Changes |  |  | |
| 1330 | NPCs Cannot Use Doors | Content Changes |  |  | |
| 1350 | Remove Silver Weapon Requirement to Hit Karoug | Content Changes | Karoug Weapon Vulnerability Adjustments |  | |
| 1351 | Remove Silver Weapon Requirement to Hit Karoug, But Require +2 Weapons | Content Changes | Karoug Weapon Vulnerability Adjustments |  | |
| 1352 | NPCs With Wolfsbane Charm Can Harm Karoug | Content Changes | Karoug Weapon Vulnerability Adjustments |  | |
| 1353 | Options 1 AND 3 | Content Changes | Karoug Weapon Vulnerability Adjustments |  | |
| 1354 | Options 2 AND 3 | Content Changes | Karoug Weapon Vulnerability Adjustments |  | |
| 1360 | Don't change the main game | Content Changes | Automatic Transition to the "Heart of Winter" Expansion |  | |
| 1361 | Remove Hjollder from Kuldahar | Content Changes | Automatic Transition to the "Heart of Winter" Expansion |  | |
| 1370 | Allow Monks to Change Damage Types with Hand Weapons | Content Changes |  |  | |
| 2020 | Two-Handed Bastard Swords | Rule Changes |  | ✓ | default |
| 2030 | Two-Handed Katanas | Rule Changes |  | ✓ | default |
| 2035 | Two-Handed Axes | Rule Changes |  | ✓ | default |
| 2040 | Universal Clubs | Rule Changes |  |  | |
| 2060 | Weapon Styles for All | Rule Changes |  |  | |
| 2080 | Delay High Level Abilities | Rule Changes |  |  | |
| 2090 | Remove experience cap | Rule Changes | Change Experience Point Cap | ✓ | default |
| 2091 | Level 20 experience point cap | Rule Changes | Change Experience Point Cap |  | |
| 2092 | Level 30 experience point cap | Rule Changes | Change Experience Point Cap |  | |
| 2720 | Yoshimo to Imoen Experience Transfer [dark0dave] | Rule Changes |  |  | default |
| 2100 | Allow Thieving and Stealth in Heavy Armor per PnP | Rule Changes |  |  | |
| 2120 | Allow Arcane Spellcasting in Heavy Armor | Rule Changes |  |  | |
| 2140 | Expanded Dual-Class Options | Rule Changes |  |  | |
| 2150 | PnP restrictions | Rule Changes | Wear Multiple Protection Items |  | |
| 2151 | No restrictions | Rule Changes | Wear Multiple Protection Items |  | |
| 2152 | Allow armor plus one protection item [Angel] | Rule Changes | Wear Multiple Protection Items | ✓ | default |
| 2160 | Rebalanced weapon proficiencies | Rule Changes | Alter Weapon Proficiency System |  | |
| 2161 | BG-style weapon proficiencies, with weapon styles [the bigg] | Rule Changes | Alter Weapon Proficiency System |  | |
| 2162 | BG-style weapon proficiencies, without weapon styles [the bigg] | Rule Changes | Alter Weapon Proficiency System |  | |
| 2163 | IWD-style proficiencies with weapon styles | Rule Changes | Alter Weapon Proficiency System |  | |
| 2164 | IWD-style proficiencies without weapon styles | Rule Changes | Alter Weapon Proficiency System |  | |
| 2170 | Cast Spells from Scrolls (and Other Items) at Character Level | Rule Changes |  | ✓ | default |
| 2190 | Only mage and bard storekeepers can identify items | Rule Changes | Limit Ability of Storekeepers to Identify Items |  | |
| 2191 | Identification ability is based on storekeeper's lore | Rule Changes | Limit Ability of Storekeepers to Identify Items |  | |
| 2192 | Hybrid of both methods | Rule Changes | Limit Ability of Storekeepers to Identify Items | ✓ | default |
| 2200 | Multi-Class Grandmastery [Weimer] | Rule Changes |  |  | |
| 2210 | True grandmastery [Baldurdash] | Rule Changes | Change Grandmastery Bonuses | ✓ | default |
| 2211 | BG2 grandmastery rules | Rule Changes | Change Grandmastery Bonuses |  | |
| 2220 | Change Magically Created Weapons to Zero Weight | Rule Changes |  |  | |
| 2230 | Make +x/+y Weapons Consistent | Rule Changes |  |  | |
| 2240 | Un-Nerfed THAC0 Table | Rule Changes |  |  | |
| 2250 | Un-Nerfed Sorcerer Spell Progression Table | Rule Changes |  | ✓ | default |
| 2260 | Un-nerfed table [Blucher] | Rule Changes | Alter Mage Spell Progression Table | ✓ | default |
| 2261 | PnP table | Rule Changes | Alter Mage Spell Progression Table |  | |
| 2270 | Un-nerfed table [Blucher] | Rule Changes | Alter Bard Spell Progression Table | ✓ | default |
| 2271 | PnP table | Rule Changes | Alter Bard Spell Progression Table |  | |
| 2280 | Un-nerfed table [Blucher] | Rule Changes | Alter Cleric Spell Progression Table | ✓ | default |
| 2281 | PnP table | Rule Changes | Alter Cleric Spell Progression Table |  | |
| 2288 | No level progression changes, normal cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2289 | No level progression changes, un-nerfed cleric spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2290 | No level progression changes, un-nerfed druid spell table only [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2291 | No level progression changes, PnP druid/cleric spell table only | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2292 | Use cleric level progression changes with normal druid spell table | Rule Changes | Alter Druid Spell and Level Progression Tables | ✓ | default |
| 2293 | Use cleric level progression changes with un-nerfed druid spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2294 | Use cleric level progression changes with pnp druid/cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2295 | Use cleric level progression changes with normal cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2296 | Use cleric level progression changes with un-nerfed cleric spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2730 | Use IWD druid level progression changes with normal druid spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2731 | Use IWD druid level progression changes with un-nerfed druid spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2732 | Use IWD druid level progression changes with pnp druid/cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2733 | Use IWD druid level progression changes with normal cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2734 | Use IWD druid level progression changes with un-nerfed cleric spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2735 | Use BG/BG2 druid level progression changes with normal druid spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2736 | Use BG/BG2 druid level progression changes with un-nerfed druid spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2737 | Use BG/BG2 druid level progression changes with pnp druid/cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2738 | Use BG/BG2 druid level progression changes with normal cleric spell table | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2739 | Use BG/BG2 druid level progression changes with un-nerfed cleric spell table [Blucher] | Rule Changes | Alter Druid Spell and Level Progression Tables |  | |
| 2297 | Use cleric level progression changes with normal druid spell table | Rule Changes |  |  | |
| 2580 | Use PnP/PsT Table | Rule Changes | Alter Wisdom-Based Divine Bonus Spell Table |  | |
| 2581 | Use BG/BG2/IWD Table | Rule Changes | Alter Wisdom-Based Divine Bonus Spell Table |  | |
| 2300 | Triple-Class HLA Tables | Rule Changes |  |  | |
| 2310 | Arcane magic only | Rule Changes | Add Save Penalties for Spells Cast by High-Level Casters |  | |
| 2311 | Divine magic only | Rule Changes | Add Save Penalties for Spells Cast by High-Level Casters |  | |
| 2312 | Arcane and divine magic | Rule Changes | Add Save Penalties for Spells Cast by High-Level Casters | ✓ | default |
| 2320 | Trap Cap Removal [Ardanis/GeN1e] | Rule Changes |  |  | |
| 2330 | Remove Delay for Magical Traps [Ardanis/GeN1e] | Rule Changes |  |  | |
| 2339 | Remove Summoning Cap for Regular Summons | Rule Changes |  |  | |
| 2340 | Remove Summoning Cap for Celestials [Ardanis/GeN1e] | Rule Changes |  |  | |
| 2360 | Remove Racial Restrictions for Single-Classes | Rule Changes |  |  | |
| 2380 | Remove Racial Restrictions for Kits | Rule Changes |  |  | |
| 2371 | Allow non-humans to dual-class | Rule Changes | Alter Dual-Class Restrictions |  | |
| 2370 | Humans can no longer dual-class | Rule Changes | Alter Dual-Class Restrictions |  | |
| 2372 | Both options: *only* non-humans can dual-class | Rule Changes | Alter Dual-Class Restrictions |  | |
| 2350 | Allow humans to multi-class | Rule Changes | Alter Multi-Class Restrictions |  | |
| 2351 | Allow non-humans access to all multi-class combinations | Rule Changes | Alter Multi-Class Restrictions |  | |
| 2353 | Allow non-humans access to a multi-classes only if they can access the single-classes | Rule Changes | Alter Multi-Class Restrictions |  | |
| 2357 | Install options one and two (everyone can multi-class anything) | Rule Changes | Alter Multi-Class Restrictions |  | |
| 2358 | Install options one and three (everyone can multi-class anything they can single-class) | Rule Changes | Alter Multi-Class Restrictions |  | |
| 2550 | Keep gnome illusionist multi-class, enable all kits for single-class | Rule Changes | Alter Gnome Mage Kit/Multi-Class Options |  | |
| 2551 | Generic mage multi-class, keep only illusionist kit for single-class | Rule Changes | Alter Gnome Mage Kit/Multi-Class Options |  | |
| 2552 | Generic mage multi-class, enable all kits for single-class | Rule Changes | Alter Gnome Mage Kit/Multi-Class Options |  | |
| 2390 | Paladins Use Icewind Dale-Heart of Winter Spell Tables [grogerson] | Rule Changes | Alter Paladin Spell Progression Table |  | |
| 2391 | BG2-Style Progression (Up To Level Six Spells) [TotoR] | Rule Changes | Alter Paladin Spell Progression Table | ✓ | default |
| 2400 | Rangers Use Icewind Dale-Heart of Winter Spell Tables [grogerson] | Rule Changes | Alter Ranger Spell Progression Table | ✓ | default |
| 2401 | BG2-Style Progression (Up To Level Five Spells) [TotoR] | Rule Changes | Alter Ranger Spell Progression Table |  | |
| 2410 | Druids Use 3E Alignment Restrictions | Rule Changes |  | ✓ | default |
| 2420 | Loosen Equipment Restrictions for Cleric Multi- and Dual-Classes | Rule Changes |  | ✓ | optional |
| 2430 | Loosen equipment restrictions | Rule Changes | Change Equipment Restrictions for Druid Multi- and Dual-Classes | ✓ | optional |
| 2431 | Tighten equipment restrictions [Angel] | Rule Changes | Change Equipment Restrictions for Druid Multi- and Dual-Classes |  | |
| 2440 | Everyone Gets Bonus APR from Specialization [subtledoctor] | Rule Changes |  |  | |
| 2450 | Enforce PnP Proficiency Rules on Dual-Classed Characters [subtledoctor] | Rule Changes |  |  | |
| 2500 | Exceptional Strength Weight Limit Changes [sarevok57] | Rule Changes |  | ✓ | default |
| 2510 | Level-Lock Spell Scrolls [Angel] | Rule Changes |  |  | |
| 2520 | Allow Mages to Use Bucklers and Thieves to Use Small Shields [Angel] | Rule Changes |  | ✓ | default |
| 2530 | Lightning Bolts Don't Bounce [Angel] | Rule Changes |  |  | |
| 2540 | Speed Up de'Arnise Keep Stronghold Quests | Rule Changes |  | ✓ | default |
| 2560 | Allow Monks to Wear Helmets | Rule Changes | Expand Usable Monk Equipment |  | |
| 2561 | Allow Monks to Use Staves | Rule Changes | Expand Usable Monk Equipment |  | |
| 2562 | Allow Monks to Wear Helmets and Use Staves | Rule Changes | Expand Usable Monk Equipment |  | |
| 2590 | Thieves Can Backstab With More Weapons With "Use Any Item" or as Dual- and Multi-Classes | Rule Changes |  | ✓ | default |
| 2620 | Make Certain Creatures Immune to Backstab/Sneak Attack [Luke] | Rule Changes |  |  | |
| 2630 | Improved Cure/Cause Wounds [Luke] | Rule Changes |  |  | |
| 2640 | PnP Potions [Luke] | Rule Changes |  |  | |
| 2650 | Make Grease ignitable [Luke] | Rule Changes |  |  | |
| 2660 | Make Bears and Boars Continue Fighting after Reaching 0 Hit Points [Luke] | Rule Changes |  |  | |
| 2670 | Limit Resting Mechanic [Luke] | Rule Changes |  |  | |
| 2680 | Make Infravision Useful [Luke] | Rule Changes |  |  | |
| 2690 | v1 (check readme) | Rule Changes | Alternate Concentration Check [Luke] |  | |
| 2691 | v2 (check readme) | Rule Changes | Alternate Concentration Check [Luke] |  | |
| 2692 | v3 (check readme) | Rule Changes | Alternate Concentration Check [Luke] |  | |
| 2693 | v4 (check readme) | Rule Changes | Alternate Concentration Check [Luke] |  | |
| 2700 | Revised Troll Regeneration [Luke] | Rule Changes |  |  | |
| 2710 | NWN-Style Summons [Luke] | Rule Changes |  |  | |
| 2740 | Allow Shapeshifters to Wear Armor But Prevent Shapeshifting While Worn | Rule Changes |  |  | |
| 2999 | Max HP at Level One | Convenience Tweaks/Cheats |  | ✓ | default |
| 3000 | Maximum | Convenience Tweaks/Cheats | Higher HP on Level Up | ✓ | default |
| 3001 | NWN-style | Convenience Tweaks/Cheats | Higher HP on Level Up |  | optional |
| 3002 | Average rolls | Convenience Tweaks/Cheats | Higher HP on Level Up |  | optional |
| 3008 | Allow HP Rolls Through Level 20 [Angel] | Convenience Tweaks/Cheats |  | ✓ | optional |
| 3010 | For all creatures in game | Convenience Tweaks/Cheats | Maximum HP Creatures [the bigg] | ✓ | default |
| 3011 | For non-party-joinable NPCs only | Convenience Tweaks/Cheats | Maximum HP Creatures [the bigg] |  | optional |
| 3012 | For party-joinable NPCs only | Convenience Tweaks/Cheats | Maximum HP Creatures [the bigg] |  | optional |
| 3020 | Identify All Items | Convenience Tweaks/Cheats |  |  | |
| 3030 | 100% learn spells | Convenience Tweaks/Cheats | Easy Spell Learning |  | |
| 3031 | 100% learn spells and no maximum cap | Convenience Tweaks/Cheats | Easy Spell Learning |  | |
| 3040 | Make Bags of Holding Bottomless | Convenience Tweaks/Cheats |  | ✓ | default |
| 3050 | Remove Fatigue from Restoration Spells | Convenience Tweaks/Cheats |  |  | |
| 3060 | Remove "You Must Gather Your Party..." Sound [Weimer] | Convenience Tweaks/Cheats |  |  | |
| 3070 | Low Reputation Store Discount [Sabre] | Convenience Tweaks/Cheats | Change Effect of Reputation on Store Prices |  | |
| 3071 | Reputation has no effect, stores price fixed at 100% [Luiz] | Convenience Tweaks/Cheats | Change Effect of Reputation on Store Prices |  | |
| 3072 | Reputation has no effect, stores price fixed at 80% [Luiz] | Convenience Tweaks/Cheats | Change Effect of Reputation on Store Prices |  | |
| 3073 | Reputation has no effect, stores price fixed at 60% [Luiz] | Convenience Tweaks/Cheats | Change Effect of Reputation on Store Prices |  | |
| 3080 | Unlimited ammo stacking | Convenience Tweaks/Cheats | Increase Ammo Stack Size | ✓ | default |
| 3081 | Stacks of 40 | Convenience Tweaks/Cheats | Increase Ammo Stack Size |  | optional |
| 3082 | Stacks of 80 | Convenience Tweaks/Cheats | Increase Ammo Stack Size |  | optional |
| 3083 | Stacks of 120 | Convenience Tweaks/Cheats | Increase Ammo Stack Size |  | optional |
| 3090 | Unlimited jewelry, gem, and miscellaneous item stacking | Convenience Tweaks/Cheats | Increase Jewelry, Gem, and Miscellaneous Item Stacks | ✓ | default |
| 3091 | Stacks of 40 | Convenience Tweaks/Cheats | Increase Jewelry, Gem, and Miscellaneous Item Stacks |  | optional |
| 3092 | Stacks of 80 | Convenience Tweaks/Cheats | Increase Jewelry, Gem, and Miscellaneous Item Stacks |  | optional |
| 3093 | Stacks of 120 | Convenience Tweaks/Cheats | Increase Jewelry, Gem, and Miscellaneous Item Stacks |  | optional |
| 3100 | Unlimited potion stacking | Convenience Tweaks/Cheats | Increase Potion Stacking | ✓ | default |
| 3101 | Stacks of 40 | Convenience Tweaks/Cheats | Increase Potion Stacking |  | optional |
| 3102 | Stacks of 80 | Convenience Tweaks/Cheats | Increase Potion Stacking |  | optional |
| 3103 | Stacks of 120 | Convenience Tweaks/Cheats | Increase Potion Stacking |  | optional |
| 3110 | Unlimited scroll stacking | Convenience Tweaks/Cheats | Increase Scroll Stacking | ✓ | default |
| 3111 | Stacks of 40 | Convenience Tweaks/Cheats | Increase Scroll Stacking |  | optional |
| 3112 | Stacks of 80 | Convenience Tweaks/Cheats | Increase Scroll Stacking |  | optional |
| 3113 | Stacks of 120 | Convenience Tweaks/Cheats | Increase Scroll Stacking |  | optional |
| 3120 | NPCs are never angry about reputation | Convenience Tweaks/Cheats | Happy Patch - Alter How Party NPCs Complain About Reputation |  | optional |
| 3121 | NPCs can be angry about reputation but never leave [Salk] | Convenience Tweaks/Cheats | Happy Patch - Alter How Party NPCs Complain About Reputation |  | default |
| 3122 | NPCs are always neutral about reputation [Anomaly] | Convenience Tweaks/Cheats | Happy Patch - Alter How Party NPCs Complain About Reputation |  | optional |
| 3123 | NPCs Don't Fight | Convenience Tweaks/Cheats |  |  | optional |
| 3124 | Stop Haer'Dalis-Aerie Romance from Starting | Convenience Tweaks/Cheats |  |  | optional |
| 3125 | Neutral Characters Make Happy Comments at Mid-Range Reputation [Luiz] | Convenience Tweaks/Cheats |  |  | optional |
| 3130 | Remove traps completely | Convenience Tweaks/Cheats | No Traps or Locks [Weimer, argent77] |  | optional |
| 3131 | Remove only harmful trap effects | Convenience Tweaks/Cheats | No Traps or Locks [Weimer, argent77] |  | |
| 3132 | Remove harmful trap effects and creatures summoned by alarms | Convenience Tweaks/Cheats | No Traps or Locks [Weimer, argent77] |  | |
| 3140 | Originals from Ease-of-Use mod [Karzak, Blucher, aVENGER, Weimer] | Convenience Tweaks/Cheats | Faster Chapter One and Two Cutscenes and Dreams |  | |
| 3141 | Non-silly version | Convenience Tweaks/Cheats | Faster Chapter One and Two Cutscenes and Dreams |  | |
| 3160 | Keep Drizzt's Loot, Disable Malchor Harpell [Weimer] | Convenience Tweaks/Cheats |  |  | |
| 3170 | No Drow Avatars On Party In Underdark [Weimer] | Convenience Tweaks/Cheats |  |  | |
| 3175 | Disable Romances | Convenience Tweaks/Cheats |  |  | |
| 3176 | Accelerate/Decelerate Romances | Convenience Tweaks/Cheats |  |  | |
| 3183 | Romance Cheats [Sabre, Richardson, Weimer] | Convenience Tweaks/Cheats |  |  | |
| 3190 | Rest Anywhere [japheth] | Convenience Tweaks/Cheats |  |  | |
| 3191 | Disable Non-Hostile Rest Spawns | Convenience Tweaks/Cheats |  |  | |
| 3194 | Disable completely | Convenience Tweaks/Cheats | Alter Hostile Rest Spawns |  | |
| 3195 | Decrease frequency by 50% | Convenience Tweaks/Cheats | Alter Hostile Rest Spawns |  | |
| 3196 | Increase frequency by 50% | Convenience Tweaks/Cheats | Alter Hostile Rest Spawns |  | |
| 3197 | Double frequency | Convenience Tweaks/Cheats | Alter Hostile Rest Spawns |  | |
| 3198 | Quadruple frequency | Convenience Tweaks/Cheats | Alter Hostile Rest Spawns |  | |
| 3200 | Sellable Items [icelus] | Convenience Tweaks/Cheats |  |  | |
| 3205 | Stores Purchase All Item Types | Convenience Tweaks/Cheats |  |  | |
| 3210 | Minimum Stats Cheat | Convenience Tweaks/Cheats |  |  | |
| 3220 | Sensible Entrance Points | Convenience Tweaks/Cheats |  |  | |
| 3230 | Taerom Makes Additional Ankheg Armor [Icendoan/grogerson] | Convenience Tweaks/Cheats |  |  | |
| 3240 | Randomize on reload | Convenience Tweaks/Cheats | Friendly Random Drops |  | |
| 3241 | Choose your drop | Convenience Tweaks/Cheats | Friendly Random Drops |  | |
| 3242 | Exchange with merchants | Convenience Tweaks/Cheats | Friendly Random Drops |  | |
| 3250 | Never Lose Access to Orrick's Trade Goods | Convenience Tweaks/Cheats |  |  | |
| 3260 | 25% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3261 | 50% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3262 | 75% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3263 | 100% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3264 | 25% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3265 | 50% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3266 | 75% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3267 | 100% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Ammunition [argent77] |  | |
| 3270 | 25% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3271 | 50% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3272 | 75% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3273 | 100% chance to recover after a successful hit | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3274 | 25% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3275 | 50% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3276 | 75% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3277 | 100% chance to recover after a successful hit, vs. enemies only | Convenience Tweaks/Cheats | Recoverable Throwing Weapons [argent77] |  | |
| 3280 | Give Every Class/Kit Four Weapon Slots | Convenience Tweaks/Cheats |  |  | |
| 3290 | Use scheme: 000000000-Protagonist-Save-Name | Convenience Tweaks/Cheats | Personalize Automatic Save Names | ✓ | default |
| 3291 | Use scheme: 000000000-Protagonist Save-Name | Convenience Tweaks/Cheats | Personalize Automatic Save Names |  | optional |
| 3292 | Use scheme: 000000000-(Protagonist)-Save-Name | Convenience Tweaks/Cheats | Personalize Automatic Save Names |  | optional |
| 3293 | Use scheme: 000000000-(Protagonist) Save-Name | Convenience Tweaks/Cheats | Personalize Automatic Save Names |  | optional |
| 3294 | Use scheme: 000000000-[Protagonist]-Save-Name | Convenience Tweaks/Cheats | Personalize Automatic Save Names |  | optional |
| 3295 | Use scheme: 000000000-[Protagonist] Save-Name | Convenience Tweaks/Cheats | Personalize Automatic Save Names |  | optional |
| 3300 | Death Cam | Convenience Tweaks/Cheats |  | ✓ | default |
| 3310 | Start New Games with Party AI Turned Off | Convenience Tweaks/Cheats |  | ✓ | default |
| 3320 | No Depreciation in Stores | Convenience Tweaks/Cheats |  |  | |
| 3330 | Make Party Members Less Likely to Die Irreversibly | Convenience Tweaks/Cheats |  |  | |
| 3340 | Movement speed by 50 percent | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3341 | Movement speed by 100 percent | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3342 | Movement speed by 150 percent | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3343 | Movement speed by 50 percent and casting speed | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3344 | Movement speed by 100 percent and casting speed | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3345 | Movement speed by 150 percent and casting speed | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3346 | Casting speed only | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] |  | optional |
| 3347 | Customize (via cdtweaks.txt) | Convenience Tweaks/Cheats | Increase Party Movement Speed and/or Casting Speed Outside of Combat [argent77] | ✓ | optional |
| 3350 | Every 15 minutes (one save only) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3351 | Every 30 minutes (one save only) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3352 | Every 60 minutes (one save only) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3353 | Every 120 minutes (one save only) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3354 | Every 15 minutes (cycle through four saves) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] | ✓ | default |
| 3355 | Every 30 minutes (cycle through four saves) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3356 | Every 60 minutes (cycle through four saves) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3357 | Every 120 minutes (cycle through four saves) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | optional |
| 3358 | Customize (via cdtweaks.txt) | Convenience Tweaks/Cheats | Create Interval Saves [argent77] |  | |
| 3360 | Reset UnderSigil Fog of War | Convenience Tweaks/Cheats |  |  | |
| 3370 | Automatic "Chapter Saves" in PsT:EE | Convenience Tweaks/Cheats |  |  | |
| 3380 | Sensible Regeneration When Traveling | Convenience Tweaks/Cheats |  |  | |
| 3390 | Automatic Use of Ranger Tracking Skill | Convenience Tweaks/Cheats |  | ✓ | optional |
| 3400 | Restored Loading Hints | Convenience Tweaks/Cheats |  |  | |
| 3410 | Mage and priest spells only | Convenience Tweaks/Cheats | Explicit Cast Warnings |  | |
| 3411 | Mage, priest, and innate spells only | Convenience Tweaks/Cheats | Explicit Cast Warnings |  | |
| 3412 | All spells (check readme) | Convenience Tweaks/Cheats | Explicit Cast Warnings |  | |
| 3490 | Start at 300k XP, select items below | Convenience Tweaks/Cheats | Better Shadows of Amn Start [dark0dave] |  | |
| 3491 | Start at 500k XP, select items below | Convenience Tweaks/Cheats | Better Shadows of Amn Start [dark0dave] |  | |
| 3492 | Start at 300k XP plus all items | Convenience Tweaks/Cheats | Better Shadows of Amn Start [dark0dave] |  | |
| 3493 | Start at 500k XP plus all items | Convenience Tweaks/Cheats | Better Shadows of Amn Start [dark0dave] |  | |
| 3494 | Better Shadows of Amn Start: Add All BG Tomes | Convenience Tweaks/Cheats |  |  | |
| 3495 | Better Shadows of Amn Start: Add Golden Pantaloons | Convenience Tweaks/Cheats |  |  | |
| 3496 | Better Shadows of Amn Start: Add Drizzt Items | Convenience Tweaks/Cheats |  |  | |
| 3497 | Better Shadows of Amn Start: Add Balduran's Cloak | Convenience Tweaks/Cheats |  |  | |
| 3420 | More Sensible Cowled Wizards [Luke] | Convenience Tweaks/Cheats |  |  | |
| 3430 | More Stoneskin Feedback [Luke] | Convenience Tweaks/Cheats |  |  | |
| 3450 | More Sensible Fireshield [Luke] | Convenience Tweaks/Cheats |  |  | |
| 3460 | More Sensible Blade Barrier [Luke] | Convenience Tweaks/Cheats |  |  | |
| 3470 | Dynamic FPS Change [Luke] | Convenience Tweaks/Cheats |  |  | |
| 3480 | More Sensible Morentherene [Luke] | Convenience Tweaks/Cheats |  |  | |
| 4000 | Adjust Evil Joinable NPC Reaction Rolls | NPC Tweaks |  |  | |
| 4010 | Improved Fate Spirit Summoning | NPC Tweaks |  |  | |
| 4020 | ToB-Style NPCs | NPC Tweaks |  |  | |
| 4025 | Allow NPC Pairs to Separate | NPC Tweaks |  |  | |
| 4030 | Use BG values | NPC Tweaks | Consistent Stats: Edwin |  | |
| 4031 | Use BG2 values | NPC Tweaks | Consistent Stats: Edwin | ✓ | default |
| 4040 | Use BG values | NPC Tweaks | Consistent Stats: Jaheira |  | |
| 4041 | Use BG2 values | NPC Tweaks | Consistent Stats: Jaheira | ✓ | default |
| 4050 | Change Jaheira to Neutral Good Alignment | NPC Tweaks |  | ✓ | default |
| 4060 | Use BG values | NPC Tweaks | Consistent Stats: Minsc |  | |
| 4061 | Use BG2 values | NPC Tweaks | Consistent Stats: Minsc | ✓ | default |
| 4070 | Use BG values | NPC Tweaks | Consistent Stats: Viconia |  | |
| 4071 | Use BG2 values | NPC Tweaks | Consistent Stats: Viconia | ✓ | default |
| 4080 | Make Khalid a Fighter-Mage [Domi] | NPC Tweaks |  |  | |
| 4090 | Make Montaron an Assassin [Andyr] | NPC Tweaks |  |  | |
| 4100 | Change Korgan to Neutral Evil Alignment | NPC Tweaks |  |  | |
| 4110 | Give Kagain a Legal Constitution Score of 19 | NPC Tweaks |  |  | |
| 4120 | Give Coran a Legal Dexterity Score of 19 | NPC Tweaks |  |  | |
| 4130 | Make Xan a Generalist Mage [Mike1072] | NPC Tweaks |  |  | |
| 4131 | Make Dynaheir a Generalist Mage [Angel] | NPC Tweaks |  |  | |
| 4132 | Make Xzar a Generalist Mage [Angel] | NPC Tweaks |  |  | |
| 4133 | Make Edwin a Generalist Mage [Angel] | NPC Tweaks |  |  | |
| 4150 | Move Boo Into Minsc's Pack | NPC Tweaks |  |  | |
| 4160 | Allow Yeslick to Use Axes | NPC Tweaks |  |  | |
| 4170 | Ensure Shar-Teel Doesn't Die in the Original Challenge | NPC Tweaks |  |  | |
| 4140 | Don't Auto-Assign Advanced AI Scripting to Party | NPC Tweaks |  |  | |
| 4180 | Removable NPC Items | NPC Tweaks |  |  | |
| 4200 | Make Rancor +1 a Bit More Reliable | NPC Tweaks |  |  | |
| 4210 | To Thief level 5 | NPC Tweaks | Adjust Nalia's Thief Level |  | |
| 4211 | To Thief level 6 | NPC Tweaks | Adjust Nalia's Thief Level |  | |
| 4212 | To Thief level 7 | NPC Tweaks | Adjust Nalia's Thief Level |  | |
| 4213 | To Thief level 8 | NPC Tweaks | Adjust Nalia's Thief Level |  | |
| 4220 | Delay until the Nashkel mines are cleared | NPC Tweaks | Delay Neera's Appearance in Beregost |  | |
| 4221 | Delay until PC has 5000 XP | NPC Tweaks | Delay Neera's Appearance in Beregost |  | |
| 4230 | Increase Yoshimo's Trapsetting Ability | NPC Tweaks |  |  | |
| 6000 | Disarm Class Feat for Rogues [Luke] | NWN-Style Feats |  |  | |
| 6010 | Knockdown Class Feat for Fighters and Monks [Luke] | NWN-Style Feats |  |  | |
| 6020 | Spellcaster Skill/Counterspell Class Talent [Luke] | NWN-Style Feats |  |  | |
| 6030 | Spontaneous Casting for Clerics [Luke] | NWN-Style Feats |  |  | |
| 6040 | Weapon Finesse Class Feat for Thieves [Luke] | NWN-Style Feats |  |  | |
| 6050 | Dual-Wield Class Feat for Rangers [Luke] | NWN-Style Feats |  |  | |
| 6060 | NWN-Style Armor vs. Dexterity [Luke] | NWN-Style Feats |  |  | |
| 6070 | Defensive Roll Class Feat for Thieves [Luke] | NWN-Style Feats |  |  | |
| 6080 | Divine Grace/Dark Blessing Feat for Paladins/Blackguards [Luke] | NWN-Style Feats |  |  | |
| 6090 | Good Aim Racial Feat for Halflings [Luke] | NWN-Style Feats |  |  | |
| 6100 | Cleave Class Feat for Fighters and Monks [Luke] | NWN-Style Feats |  |  | |
| 6110 | Sneak Attack Class Feat for Blackguards [Luke] | NWN-Style Feats |  |  | |
| 6120 | Fearless Racial Feat for Halflings [Luke] | NWN-Style Feats |  |  | |
| 6130 | Poison Save Class Feat for Assassins [Luke] | NWN-Style Feats |  |  | |
| 6140 | Planar Turning Class Feat for Clerics/Paladins [Luke] | NWN-Style Feats |  |  | |
| 6150 | Circle Kick Class Feat for Monks [Luke] | NWN-Style Feats |  |  | |
| 6160 | NWN-style Barbarian Rage [Luke] | NWN-Style Feats |  |  | |
| 6170 | "Force" the Archer Kit to Use Bows [Luke] | NWN-Style Feats |  |  | |
| 6180 | Blind Fight Innate Feat for Berserkers [Luke] | NWN-Style Feats |  |  | |
| 6190 | Dirty Fighting Class Feat for Chaotic-Aligned Rogues [Luke] | NWN-Style Feats |  |  | |
| 6200 | Overwhelming/Devastating Critical Class Feat for Trueclass Fighters [Luke] | NWN-Style Feats |  |  | |
| 6210 | Self Concealment Class Feat for Monks [Luke] | NWN-Style Feats |  |  | |
| 6220 | Parry Mode Kit Feat for Blades and Swashbucklers [Luke] | NWN-Style Feats |  |  | |
| 6230 | Armored Caster Class Feat for Bards [Luke] | NWN-Style Feats |  |  | |
| 6240 | Trackless Step Class Feat for Rangers [Luke] | NWN-Style Feats |  |  | |
| 6250 | Nature Sense Class Feat for Druids [Luke] | NWN-Style Feats |  |  | |
| 6260 | Uncanny Dodge Class Feat for Barbarians and Rogues [Luke] | NWN-Style Feats |  |  | |
| 6270 | NWN-style Sneak Attack/Crippling Strike Class Feat for Thieves and Stalkers [Luke] | NWN-Style Feats |  |  | |
| 6280 | Evasion Class Feat for Monks and Rogues [Luke] | NWN-Style Feats |  |  | |
| 6290 | Spell Penetration Class Feat for Spellcasters [Luke] | NWN-Style Feats |  |  | |
| 6300 | Smite Evil/Good Class Feat for Paladins/Blackguards [Luke] | NWN-Style Feats |  |  | |
| 6310 | Woodland Stride Class Feat for Druids [Luke] | NWN-Style Feats |  |  | |
| 6320 | Bane of Enemies Class Feat for Rangers [Luke] | NWN-Style Feats |  |  | |
| 6330 | Cantrips Class Feat for Spellcasters [Luke] | NWN-Style Feats |  |  | |
| 6340 | Animal Companion Class Feat for Beastmasters [Luke] | NWN-Style Feats |  |  | |
| 6350 | Opportunist Class Feat for Chaotic-Aligned Rogues [Luke] | NWN-Style Feats |  |  | |

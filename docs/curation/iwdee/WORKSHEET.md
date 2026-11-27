# Chriz Easy IWD — mod worksheet

This worksheet records Christopher's direction from 2026-09-08 through 2026-09-10 and
his mod-level selection of 2026-10-25. It does not supply executable component
decisions. Source versions and all applicable component menus must be reviewed before
the IWD recipe is frozen.

## Mod-level selection (2026-10-25)

Christopher chose from the neutral [inventory](INVENTORY.md). "On" and "off" mean
selected or unselected by default in the installer. Component menus are still to be
curated per mod.

| Decision | Mods |
|---|---|
| In, on by default | EEex; EE Fixpack; Bubb's Spell Menu Extended; BuffBot (after IWD:EE enablement); SCS; our own combat mod; Tweaks Anthology; IWD Unfinished Business; IWD Graphical Overhaul (IWDGO); companions Minerva, Karihi, Athanasios, Guldruk, Nirik, Cassia, Bethany, Dendjelion, Dusky, Hommet, Ina, L'anna, Oak-Maw, Orra, T'viy, Urchin, Vito; IWD Crossmod Banter Pack |
| In, off by default | EEex Remote Console; LeUI-IWDEE and Infinity UI++ (alternatives, so the installer offers at most one); Lava's quests Below & Below Inn, Night of the Blinking Dead, Snowytoes Hamlet, Tale of Our Lady Dreamless, Terror of the Skineater, The Rediscovery of Kuldahar, The War of Bloom and Rot, Will of the Wisps |
| In, default open | Hidden Gameplay Options |
| Later | Frostforged and The Artisan's Armory ("look at it later"); IWDEE Tweaks and Fixes (wants a closer look) |
| Out | Artisan's Kitpack and Bardic Wonders (changed from the September "if compatible"); Spell Revisions; all other inventory entries, including Kulyok's IWD NPC Project, Tipps and Geomantic Sorcerer, Talents of Faerûn, the subtledoctor and argent77 kit mods, Deities of Faerûn, Afaaq, Turald, Xikâsz, HoWMod, IWD:EE Reborn, the item mods, EEUITweaks, the lighting pack, the font pack and Premade Party Adder |
| Stream rule only | Reroll limit (at most 10 rerolls, 2 stored rolls); no installer feature |

Added later:
- Arszyk's Wonderful Decay, 2026-11-07, for the next version and not for his stream game
  ("Kannst du es noch hinzufügen für die nächste Version?"). On by default, as the other
  companions. It is a beta, and its quest also works without recruiting him.
- Klatu Tweaks and Fixes 1.7.4, component 2150 (thieves use their skills in any armor, no added
  penalties), 2026-11-09: on by default as in the BG installer ("Es ist einfach Quality of
  Life"). It runs after every mod that adds armor. His stream game got it the same day.
- Chriz Overhead Text 0.1.0, component 0 (keep spoken overhead text visible above Reveal Details
  nameplates and overhead HP labels via EEex), 2026-11-16: on by default in both installers. It
  copies only `override/M_CZOH.lua` after EEex.
- Chriz IWD Party Dialog 0.1.0, component 0 (party-wide class, kit, and race checks in world and
  quest dialogues with speaker/token handover via EEex), 2026-11-17: on by default in Chriz Easy
  IWD. It copies only `override/M_CIPDLG.lua` after EEex and keeps personal companion talks bound
  to `Player1`.
- BG Rebalance v0.8.12, selected 2026-11-18. Both installers enable Round Clock (361) by default
  with EEex. It is the sole component in the IWD run, immediately after Chriz IWD Party Dialog.
- Chriz Multiplayer Fix 0.1.0, 2026-11-22: on by default in both installers. Component 0 (the
  crash fix after leaving the multiplayer lobby) always comes with the mod; component 1 (a Copy
  button per save on the Load screen, between single player and multiplayer) is on by default
  and can be turned off. Both need EEex and run directly after Chriz Overhead Text. Existing
  installations get both through the hotpatch `chriz-multiplayer-fix-0.1.0`.
- Chriz BG Modpack v0.2.0-alpha.18, 2026-11-23 (Christopher's decision): enables the RAW
  spell-scroll learning fix (470) by default with EEex (restricting ±15% specialist modifiers to
  specialist mages; bards, Wild Mages, kitted dual-class mages, and non-mages stay at 0%) in
  the IWD run, immediately after BG Rebalance (690 Fast Traps stays unoffered). Pinned release
  updated to v0.2.0-alpha.19 on 2026-11-27 (`0.1.20`), v0.2.0-alpha.21 on 2026-11-27 (`0.1.21`), and v0.2.0-alpha.22 on 2026-11-27 (`0.1.23`), adding optional mutually exclusive EEex unpaused-inventory choices 699 (always) and 700 (combat only), both off by default in Chris's Recommended; components 695, 696, 697, and 698 remain excluded from IWD.

Notes from the selection:
- The quest mods' compatibility with each other can be tested later; Christopher said
  it has no priority now.
- The Rediscovery of Kuldahar needs IWDGO installed first and a new game.
- Lava's downloads use per-visit tokens; automatic fetching is untested and needed for
  eleven companions and eight quests.

## Shared mods and adaptations

| Mod / family | Recorded direction | Next curation work |
|---|---|---|
| EE Fixpack | Include. | Verify the exact IWDEE core/text-update menu and supported game build. |
| EEex | Carryover candidate for the agreed feature set. | Review tiers/options and matched IWDEE build. |
| Bubb's Spell Menu | Carryover candidate for the agreed feature set. | Verify UI integration and conflicts. |
| BuffBot | Desired; source review suggests a small enablement task. | Enable/test IWDEE in its owning repo, then curate the supported release. |
| SCS | Include; additional conservative enhancements are desired for investigation. | Review IWD components and optional spell changes; audit existing coverage before designing patches. |
| Artisan's Kitpack | Desired if compatible. | Review kits, base-class changes and applicable late tweaks; omit BG companion assignments. |
| Bardic Wonders | Desired if compatible. | Review kits/songs/spells separately from BG-only content; check chosen NPCs. |
| CDTweaks | Include; re-curate components. | Compare BG choices, filter game availability, review unfamiliar IWD options. |
| Hidden Gameplay Options | Previously proposed candidate, component decisions open. | Review IWD options and any duplicates with shared UI/settings. |
| Existing Chriz patches | Investigate applicable fixes, particularly caster readiness. | Map each effect to an owning mod and verify the defect/need on IWDEE. |
| Other BG-list gameplay/UI mods | Inventory and assess individually. | Do not assume compatibility or include them merely because they were installed in BG. |
| New NPC/quest/item mods | Intake and curation desired; individual choices vary. | Use [the additions worksheet](ADDITIONAL-MODS.md). |

## Explicit omissions and deferrals

| Mod / family | Recorded direction |
|---|---|
| Spell Revisions | Omit; retain variety and enable a comparison with IWD's spell system. |
| Item Randomiser | Omit. |
| EET / EET_end / SoD and BG campaign content | Omit from IWD; game-specific stages remain available to BG. |
| BG NPC mods and BG companion kit assignments | Omit; choose IWD companions separately. |
| Frostforged | Deferred from the initial proposal because balance is a concern; no balance verdict established. |

DLC Merger is a conditional shared preparation tool, not a blanket omission or mandatory
IWD component. Decide from the supported source layout and exact archive requirements.

## First review batches

1. Source/build compatibility, EE Fixpack and EEex prerequisites.
2. SCS baseline: AI, optional spell changes, difficulty and prebuff behavior.
3. CDTweaks: convenience/cosmetics, then native rules/progression and IWD-specific options.
4. Artisan's/Bardic gameplay choices and supported UI/buff automation.
5. Additional NPCs, crossmod content and any selected quest/item candidates.
6. Proposed Chriz fixes/enhancements, independently from baseline mod selections.

The sequence is a workflow suggestion, not a change to existing curation. New discoveries
can add follow-ups without reopening decisions the user has already settled.

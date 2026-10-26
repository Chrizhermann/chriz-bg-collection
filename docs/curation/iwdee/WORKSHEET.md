# Chriz Easy IWD — mod worksheet

This worksheet records Christopher's direction from 2026-09-08 through 2026-09-10, his
mod-level selection of 2026-10-25 and his component selection of 2026-10-26. The
executable decisions live in the recipe (`recipes/iwd-current`).

## Mod-level selection (2026-10-25)

Christopher chose from the neutral [inventory](INVENTORY.md). "On" and "off" mean
selected or unselected by default in the installer. The component menus follow in the
next section.

| Decision | Mods |
|---|---|
| In, on by default | EEex; EE Fixpack; Bubb's Spell Menu Extended; BuffBot (after IWD:EE enablement); SCS; our own combat mod; Tweaks Anthology; IWD Unfinished Business; IWD Graphical Overhaul (IWDGO); companions Minerva, Karihi, Athanasios, Guldruk, Nirik, Cassia, Bethany, Dendjelion, Dusky, Hommet, Ina, L'anna, Oak-Maw, Orra, T'viy, Urchin, Vito; IWD Crossmod Banter Pack |
| In, off by default | EEex Remote Console; LeUI-IWDEE and Infinity UI++ (alternatives, so the installer offers at most one); Lava's quests Below & Below Inn, Night of the Blinking Dead, Snowytoes Hamlet, Tale of Our Lady Dreamless, Terror of the Skineater, The Rediscovery of Kuldahar, The War of Bloom and Rot, Will of the Wisps |
| In, default open | Hidden Gameplay Options |
| Later | Frostforged and The Artisan's Armory ("look at it later") |
| Out | Artisan's Kitpack and Bardic Wonders (changed from the September "if compatible"); Spell Revisions; IWDEE Tweaks and Fixes (after a closer look; its BG2-style HLAs maybe later for the community, since IWD is not built for HLAs); all other inventory entries, including Kulyok's IWD NPC Project, Tipps and Geomantic Sorcerer, Talents of Faerûn, the subtledoctor and argent77 kit mods, Deities of Faerûn, Afaaq, Turald, Arszyk, Xikâsz, HoWMod, IWD:EE Reborn, the item mods, EEUITweaks, the lighting pack, the font pack and Premade Party Adder |
| Stream rule only | Reroll limit (at most 10 rerolls, 2 stored rolls); no installer feature |

Notes from the selection:
- The quest mods' compatibility with each other can be tested later; Christopher said
  it has no priority now.
- The Rediscovery of Kuldahar needs IWDGO installed first and a new game.
- Lava's downloads use per-visit tokens; automatic fetching is untested and needed for
  eleven companions and eight quests. (Solved 2026-10-26: the installer fetches them
  unattended.)

## Component selection (2026-10-26)

Christopher went through the component menus of the selected mods on a private selection
page: 434 installable parts, each marked mandatory, on by default, offered (off by default)
or not offered. Where he made no pick, the page's suggestion applies: the BG installer's
decision when the version and label are the same, otherwise a mod's main part on and the
rest as the page proposed. The recipe `recipes/iwd-current` (branch `claude/iwdee-recipe`)
holds every resulting decision. His own choices:

| Mod | Christopher's choices |
|---|---|
| SCS | All spell tweaks, after asking for SCS's own recommendation (they also bring Baldur's Gate versions of some IWD spells), so the per-category choices are not offered. On by default: Baldur's Gate-style Insect Plague and Creeping Doom, Baldur's Gate-style Mordenkainen's Sword next to the IWD one, revised elementals and elemental summoning, wider availability of wizard spell scrolls, improved shapeshifting. Mandatory: revised IWD undead. |
| Tweaks Anthology | On: NPCs answer the main character, not whoever talks to them. Offered: Baldur's Gate casting graphics, restored IWD tooltips, game ends when the main character dies, Heart of Winter at any level, both variants of the Hjollder tweak, two-handed bastard swords, katanas and axes, the three random-drop variants. Not offered: the five Baldur's Gate druid progression variants. |
| EEex | Offered: the attack-info module (to-hit chance on the attack cursor). |
| Hidden Gameplay Options | Offered: improved "Quit Game" options. Its option for Tweaks Anthology's restored loading hints is left out, since that tweak is not chosen. Whether the mod is on by default is still open. |
| Infinity UI++ | Four quicksave slots on by default; one, two, three, five or six slots offered instead (chat, 2026-10-26). Offered: transparent tooltip background, its Dialog.tlk string updates. |
| IWD Crossmod Banter Pack | Banter timer of 30 minutes on by default, 15 minutes (the mod's recommendation) offered; crossmod romance conflicts on. |
| IWD Unfinished Business | On: Marketh's Ring, actual shadows in the Vale of Shadows, the wardstone for Lower Dorn's Deep. |
| Will of the Wisps | On: new items for shamans and undead NPCs. Offered: the Witchlight Shaman and Town Medium kits, the optional new enemies. |
| Athanasios | Fighter/Cleric. |
| Cassia | The grey default portrait; the installer answers the mod's question. |

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

# Optional weapon groups and companion customization

Christopher relayed DeeRainJed's suggestion on 2026-09-07: simplified weapon
proficiencies and choosing companions' weapon pips. Research/planning only; the
recommended recipe, released pins, and existing installations remain unchanged.
Current curation excludes CDTweaks 2160-2164 and SCS 4099/4100.

September 27: [fresh source check](2026-09-27-pre-package-inventory.md#viewer-request-simplified-weapon-proficiencies)
confirms 2161 as the best first candidate, with Randomiser/Artisan/final-NPC-pip
compatibility still to address. SCS 4100 is broader than a pip picker and also
needs the Infinity UI interaction checked. No options were enabled.

## Two separate player choices

- **Weapon proficiency groups:** keep the current system by default; investigate
  an optional broader grouping system, retaining fighting styles. One selector,
  not competing checkboxes. No need to expose every upstream variant initially.
- **Choose companion weapon pips:** preserve the chosen companion class/kit by
  default. Full class/kit respec is a distinct, broader choice, not implied by
  permission to redistribute pips. Explain any unavoidable XP/spellbook changes.

## Existing candidates

**Tweaks Anthology v18, Alter Weapon Proficiency System:** 2161 is BG-style
grouping with fighting styles; 2163 is the less concentrated IWD-style alternative
with styles. Their 2162/2164 variants remove styles. Component 2160 merely moves
wakizashis, ninja-tos and morningstars between existing proficiencies: it is not
the broad simplification the viewer describes. The official readme lists EET
support. [Documentation](https://gibberlings3.github.io/Documentation/readmes/readme-cdtweaks.html).

Do not infer compatibility with all collection components from EET support.
Artisan has its own proficiency/item-usability machinery. Before offering this,
check the pinned implementation against final weapon categories, kit proficiency
caps, custom usability checks, CLAB/kit abilities, summoned weapons and companion
pip allocations. Late item or companion patches can invalidate an earlier remap.
This is a compatibility question, not evidence that the current recipe is broken.

**SCS v35.21, 4100 Improved NPC customization and management:** strongest existing
candidate for recruitment-time allocation on EET, including ordinary mod NPCs.
Recruits start at level zero; class/kit changes require the optional Customize
power. It is not pip-only: it also matches recruitment/rejoining XP, replaces
mage/bard spellbooks, removes normally illegal bonus pips, and has special
dual-class handling. Baeloth, Caelar, Hexxat and Wilson are excluded. The legacy
4099 component is not an EET substitute.
[Readme](https://gibberlings3.github.io/Documentation/readmes/readme-stratagems.html),
[pinned TP2](https://github.com/Gibberlings3/SwordCoastStratagems/blob/v35.21/stratagems/setup-stratagems.tp2),
[leveller source](https://github.com/Gibberlings3/SwordCoastStratagems/blob/v35.21/stratagems/leveller/leveller_2.tpa).

- SCS 4100 and selected 4115 forbid each other: 4100 already includes the
  multiples-of-five thief-skill behavior. Model this as a replacement/subsumed
  feature, never ask WeiDU to install both. CDTweaks 2370/2372 are also forbidden
  alongside 4100; they are currently excluded.
- Our SCS run precedes chriz-bg-modpack NPC conversions and a late
  Artisan NPC conversion. SCS captures class/dual-class metadata and clears
  spellbooks; later converters may overwrite its level-zero setup. Inspect the
  ordering/metadata interaction before enabling it. Preserve Dynaheir's Haste,
  Sarah's intended identity, Viconia's multiclass and Artisan kit abilities.
  September 27 correction: the current main Modpack NPC conversions occur before
  EET_end; the earlier post-EET_end description was stale. The SCS-before-conversion
  ordering concern remains.

**NPC_EE 2000:** alternative optional-tome respec that keeps XP and does not
automatically rebuild a companion until used. Broader class/stat/kit changes and
dual-class/starting-pip limitations make it another integration candidate, not a
drop-in pip picker. Select one customization system, not both it and SCS 4100.
[Official source/readme](https://github.com/UnearthedArcana/NPC_EE).

**Level 1 NPCs:** useful prior art, but confirm modern EE/EET coverage before
considering it; not the first candidate for this one-click EET collection.
[Readme](https://gibberlings3.github.io/Documentation/readmes/readme-level1npcs.html).

## Bounded next work, not an alpha-release gate

1. Inspect the pinned CDTweaks/Artisan implementations together. Decide which
   grouping can preserve intended kit restrictions without a large overhaul.
2. Prototype recruitment customization with final NPC conversions; determine
   whether SCS's extra behavior is acceptable or a narrower mod-owned component
   is needed. Keep game-content implementation in the appropriate mod repo.
3. Use disposable focused fixtures plus recruit/level/save-reload checks for a
   normal fighter, Viconia, Sarah, Dynaheir, dual-class Imoen and an Artisan kit.
   Cover both proficiency modes and weapons from later mods. Check pip budgets,
   allowed equipment and abilities; no repeated full install just for research.
4. Only after that add default-off options, truthful descriptions, dependencies,
   exclusions and source/order pins. Never promise every companion/all-mod
   compatibility from documentation alone. No existing-save respec or hot patch
   is part of this proposal.

Evidence so far is documentation/pinned-source research and current curation/order
inspection. No prototype, live game test, new dependency, release or selection
change has been made.

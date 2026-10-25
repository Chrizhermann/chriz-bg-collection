# Chriz Easy IWD — collection direction

Status: agreed direction and initial research; not an executable recipe or a tested stack.
Created: 2026-09-08. Architecture recommendation revised: 2026-09-10.

Product name chosen by Christopher: **Chriz Easy IWD**. Use the
[IWD curation workspace](../curation/iwdee/README.md) for the BG-style mod worksheet,
versioned component review, additional-mod intake and follow-up tracking. The name is
recorded here; app branding and executable collection identity are not implemented yet.

## Purpose and repository structure

Create a standalone IWDEE collection with the convenience, class variety and smarter
combat of the BG collection, while preserving IWD's own spell and equipment identity.
Cover the base campaign, Heart of Winter and Trials of the Luremaster. Keep research
and player-facing reports free of encounter, recruitment-location and plot spoilers.

Recommended home: this repository, with one shared engine/app and separate named
BG/EET and IWDEE collections. This supersedes the initial sibling-recipe-repository
recommendation. The same maintainer, heavily shared mod sources, installer behavior
and testing needs favor keeping coordinated changes in one commit and review.

Separate collection identity, curation, version pins, release metadata and acceptance
evidence do not require separate Git repositories. Share mod/artifact definitions
where identical, but let each collection pin different versions when compatibility
requires it. Build self-contained immutable recipe snapshots so later catalog edits
cannot change an existing installation's frozen inputs.

Conceptual boundaries, with final directory names left to implementation:

- Shared engine/app: acquisition, validation, safe staging, WeiDU execution, progress,
  recovery, receipts, updates and launcher behavior.
- Game profiles: executable/source discovery, supported builds, required source roles,
  resource layout, launch/save conventions and applicable preparation steps.
- Collection recipes: selected mods and components, order, defaults, exclusions,
  compatibility notes, independent versions and release/playtest evidence.
- Presets: choices within one collection, not substitutes for a distinct game target.

BG/EET uses two source roles and its import/finalization steps; IWDEE uses one source
role and standalone installation. Express that variation explicitly through shared
contracts instead of copying the engine or spreading game-name conditionals throughout
it. BuffBot and existing reusable mod fixes retain their independent repositories.

The existing app bundle names one manifest root in `app/src-tauri/tauri.conf.json`.
`RecipeEnvelope` already carries recipe identity, version and minimum app version,
providing useful foundations for independent collection updates. Multiple-collection
selection, routing and persisted identity still need implementation and regression
tests. A source-repo split would not remove that work and would add coordinated
engine/schema/recipe release work across repos. Revisit separation if collections
later gain independent maintainers or materially different release/governance needs.

The current engine is EET-specific: `GameRoot` and `Phase` in
`engine/src/manifest.rs`, paired source checks in `engine/src/preflight.rs`, and the
session, staging, receipt and app contracts all need standalone-game support. Existing
artifact acquisition, integrity checks, selection resolution and WeiDU execution are
reusable. A second collection folder alone does not make the current installer support IWD.
Preserve existing frozen BG campaigns and serialized recipes when generalizing it.

DLC preparation is a shared optional capability. [DLC Merger](https://github.com/Argent77/A7-DlcMerger)
supports arbitrary Enhanced Edition DLC archives as well as its SoD-specific mode.
Select it when the supported source layout and recipe identify an archive that needs
merging; do not require it solely because IWDEE includes expansions. Retain exact
source/ordering checks and deterministic archive selection rather than silently
merging every archive found in a user's search paths.

## User decisions

Use the same curation procedure as the BG collection. Shared mods get their own IWD
component decisions, with existing BG choices visible for comparison. Additional mods
(primarily NPCs, possibly quests and items) first enter the intake worksheet and then
follow that same per-mod component review. Scope for conservative SCS improvements is
an explicit investigation track, including caster-role recipes and prebuff behavior.

| Area | Direction | Remaining work |
|---|---|---|
| EE Fixpack | Include. | Pin a compatible release and select applicable components. |
| SCS | Include IWDEE components. Investigate conservative caster-readiness improvements and existing patches. | Audit installed AI and spell resources; choose individual spell/gameplay changes. |
| Spell Revisions | Omit. | Keep the native spell system as the comparison baseline. |
| Artisan's Kitpack | Desired, conditional on compatibility. | Audit the exact fork/release and selected kits, including IWD progression and abilities. |
| Bardic Wonders | Desired, conditional on compatibility. | Check songs, custom kits, NPC interactions and the exact release. |
| CDTweaks | Include the mod; component choices remain open. | Produce an IWDEE-filtered review with current BG choices shown only as comparison. |
| BuffBot | Desired carryover; likely bounded compatibility enablement. | Verify game-specific assumptions and test on IWDEE before enabling support. |
| EEex / Bubb's Spell Menu | Carryover candidates supporting the intended UI and kits. | Pin a compatible game/EEex pair and verify the final UI combination. |
| NPCs | Tentative roster below; user will review more later. | Verify releases, expansion coverage, banters and kit conflicts. |
| Frostforged | Defer from the initial proposal because of balance concerns. | No conclusion that it is overpowered has been established. |
| Other IWD content | Discover selectively; prefer modest changes. | Review gameplay impact before adding new quests, rewards or overhauls. |
| Item Randomiser / EET / SoD / BG content and NPC conversions | Omit. | Build an independent IWD order; do not edit the captured BG install order. |

Including SCS does not select every optional spell overhaul. Explicitly review its
BG-inspired IWD spell changes, additional spells and broad rebalance groups so that
the resulting no-SR profile remains a useful comparison. Including a mod is also not
blanket approval for all of its components or an untested port.

Sources: [EE Fixpack](https://gibberlings3.github.io/Documentation/readmes/readme-eefixpack.html),
[SCS](https://gibberlings3.github.io/Documentation/readmes/readme-stratagems.html),
[Bardic Wonders](https://theartisanbg.github.io/The-Artisans-Corner/bardic-wonders),
[Artisan compatibility-patch history](https://forums.beamdog.com/discussion/75663/kit-mod-the-artisans-kitpack/p4).
The old Artisan patch is historical evidence of areas to inspect, not a patch to
apply blindly to today's kitpack. Local BG live acceptance does not establish IWD acceptance.

## Tentative NPC roster

These are the five companions the user named or described. This is a shortlist, not
a locked party or a decision to include every available NPC.

| NPC | Spoiler-free identification | Primary source |
|---|---|---|
| Minerva | Artisan's gnome fighter/artificer. | [Author](https://artisans-corner.com/minerva/) |
| Karihi | Artisan's fire genasi pyromancer. | [Author's IWD NPCs](https://artisans-corner.com/iwdee-npcs/) |
| Ina | The ghost bard: Lava's female spectre bard. | [Author](https://downloads.weaselmods.net/download/ina/) |
| Cassia | The tiefling cleric: Arvendor's Priest of Helm. | [Author](https://forums.beamdog.com/discussion/87511/mod-cassia-npc-for-iwd-ee) |
| Oak-Maw | The deer-like companion: Lava's dark fey Stalker. | [Author](https://downloads.weaselmods.net/download/oak-maw/) |

All five are named in the [IWD Crossmod Banter Pack v5 readme](https://gibberlings3.github.io/Documentation/readmes/readme-iwdcrossmodpack.html).
Include the corresponding banter content after the selected NPC mods; review optional
romance-conflict choices separately. Listing establishes coverage, not equal dialogue
volume for every pairing. Check HoW/TotL content per NPC rather than assuming every
companion has the same expansion coverage. Kulyok's pack and other companions remain
unselected candidates for the user's later review.

## Conservative caster improvements

Treat the user's concern that some casters lack prebuffs as a hypothesis to investigate.
First establish what the selected SCS build already does on IWDEE. Report anonymous
counts and caster categories; omit enemy names, locations, encounter order and rewards.

Distinguish assignment of AI from generation of spellbooks, prebuff routines and
individual encounter/caster overrides. Do not assume that a lack of bespoke encounter
changes means the shared spell/prebuff systems are absent. Source findings belong in
[the SCS audit](../curation/iwdee/SCS-RESEARCH.md); installed coverage is a separate check.
The 2026-09-10 v35.21 source audit confirms IWD-specific mage/priest dispatch, generated
spellbooks and preparation scripts, plus authored individual customization data. Thus
the enhancement scope is coverage, choices and behavior on the actual stack, not an
assumed absence of those systems. Effective override application remains to be verified.

Compare vanilla, SCS-only and proposed patched states on an isolated test copy. Separate
missing SCS coverage, delayed first AI execution, legitimately unprepared enemies and
actual spellbook omissions. Do not automatically grant a full BG-style defensive suite.

Candidate design limits:

- Preserve caster identity, level and available resources. Correct a demonstrated gap
  before considering additional spells or powers.
- Reuse existing readiness/prebuff mechanics only where the effective IWD spells and
  AI scripts satisfy their assumptions. Keep targeted compatibility changes optional.
- Preserve genuine casting costs, finite durations, once-only behavior and difficulty
  controls; do not repeatedly apply defenses or grant free defensive stacks.
- Exercise initial contact, ambush, invisibility, interrupted casting, death and save/load.
  Do not use stat inflation, new enemies or stronger loot as a substitute for better AI.

The current collection pins CHRIZ-BG-REBALANCE v0.3.2. Component 121 (ambient caster
readiness) is the initial candidate. Component 120 is SR-specific and does not belong
in this no-SR plan. Component 101 is BG-store-specific and does not transfer. Other
spell repairs need proof that the affected spell and defect actually exist on the IWD
stack. No whole-package carryover is implied.

Release-source audit at `011c50ab333ea07cfe2c8303fe7308866677381e` established:

- Component 121 has no SR/120 dependency. It requires BG2EE/EET, EEex, SCS Smarter
  Mages and the installed SCS instant-prebuff mapping.
- Runtime eligibility depends on SCS caster classification. A successful installation
  does not establish priest coverage or the coverage of newly added creatures.
- Its compiler requires a Project Image safety resource and at least one qualifying
  urgent weapon-protection spell. Ambient and urgent behavior can be switched off
  separately at runtime, but the released compiler does not install them independently.
- If those requirements do not fit IWDEE, separate the applicable ambient mode from
  the urgent mode while preserving their safety checks. Do not add BG spells merely
  to satisfy an installation assumption or simply remove the guard.
- The existing urgent reaction initiates an ordinary cast with normal casting time;
  it does not establish blanket instant prebuffs for every caster.

Evidence: [v0.3.2 installer](https://github.com/Chrizhermann/chriz-bg-rebalance/blob/v0.3.2/setup-chriz-bg-rebalance.tp2),
[readiness compiler](https://github.com/Chrizhermann/chriz-bg-rebalance/blob/v0.3.2/chriz-bg-rebalance/lib/ambient_readiness.tpa),
[runtime](https://github.com/Chrizhermann/chriz-bg-rebalance/blob/v0.3.2/chriz-bg-rebalance/lua/M_CBRRDY.lua).
The working checkout differed from this release; findings use the pinned tag's blobs.

## BuffBot compatibility findings

Read-only source review at BuffBot commit `2c3c7e9` (v1.8.3-alpha) found a likely
small enablement task, not evidence that a redesign is required. Both TP2 components
exclude IWDEE through their game predicate. Core scanning uses EEex spell/resource
enumeration; execution uses normal spell/item actions. No campaign, area, quest or
named BG companion dependency was found in the reviewed paths.

Read-only resource inspection of an available IWDEE 2.7.3.0 installation confirmed
`WORLD_ACTIONBAR` and its open/close callbacks, the shared `GUIOSTUL`/`GUISCRC` graphics
and relevant fonts. These match the existing UI integration. BuffBot also bundles its
own main graphics and innate icons. This removes an obvious compatibility concern,
but is not a runtime test.

Before enabling support, check UI layout, mage/priest/innate and item enumeration,
actual cast/slot accounting, F12/recharge, save/load, portrait reorder, summoned targets
and dismissal/replacement of the join-order-zero character used for summon settings.
Confirm item-slot offsets against IWDEE rather than relying on BG-only test evidence.

Evidence: [installer](https://github.com/Chrizhermann/bg-eeex-buffbot/blob/2c3c7e9/buffbot/setup-buffbot.tp2),
[UI](https://github.com/Chrizhermann/bg-eeex-buffbot/blob/2c3c7e9/buffbot/BfBotUI.lua),
[scanner](https://github.com/Chrizhermann/bg-eeex-buffbot/blob/2c3c7e9/buffbot/BfBotScn.lua),
[persistence](https://github.com/Chrizhermann/bg-eeex-buffbot/blob/2c3c7e9/buffbot/BfBotPer.lua).

## CDTweaks review queue

Use the [v18 component documentation](https://gibberlings3.github.io/Documentation/readmes/readme-cdtweaks.html)
and the chosen release's installer predicates. Generate the full applicable menu later;
these are review priorities, not selected defaults.

| Topic | Initial review position |
|---|---|
| BG NPC/quest changes | Filter out unavailable components. |
| Existing inventory/UI conveniences | Compare individually; retain familiar choices where suitable. |
| Paladin/ranger IWD progression imports | Already native to IWD; these BG-oriented options do not carry over. |
| Spell save penalties (BG choice 2312) | Reconsider; materially changes the no-SR baseline. |
| Spell slots, XP/progression, proficiency and equipment changes | Assess against native rules and selected kits. |
| Friendly Random Drops (3240-3242) | Review separately; controls native random drops, distinct from Item Randomiser. Leave unselected initially. |
| Early expansion access / rest changes | Leave unselected initially; affect campaign pacing and attrition. |
| Summon/trap cap removal and added feats | Leave unselected initially; material rules changes. |
| Cantrips / Bubb's Spell Menu | Documentation reports incompatibility; exclude this combination. |

## Work sequence and acceptance

1. Use the IWD mod worksheet and additions intake to start the existing BG-style
   curation procedure. Preserve already settled directions and track unfinished choices.
2. Review exact shared-mod versions and generate full component inventories with
   independently verified IWDEE availability. Investigate SCS, Artisan/Bardic and BuffBot
   compatibility in parallel; record source/version evidence.
3. Review component menus with Christopher in batches and finish the selected additions.
   Author the independent IWD decision mapping and specify collection/game-profile
   contracts, preserving existing BG recipes and saved campaigns during migration.
4. Build the independent IWD order and adapt shared installer contracts, collection
   selection and recipe packaging for both one-source and two-source installations.
5. Install on a clean, isolated IWDEE copy and verify the resulting WeiDU log and receipt.
6. Test character creation/levelling, kit abilities, songs, spell menu, buff execution,
   configuration persistence, NPC interaction and representative anonymous caster cases.
7. Obtain gameplay evidence through the campaigns before calling the collection broadly
   accepted. Keep installation success distinct from gameplay and balance acceptance.

This research has not installed IWDEE, modified any game directory, ported a mod,
changed the BG recipe, created a sibling repository or published a release.

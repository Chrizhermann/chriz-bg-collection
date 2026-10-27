# SCS source audit for Chriz Easy IWD

Checked: 2026-09-10. Scope: upstream source inspection only; no game access,
installation, generated-resource comparison or gameplay acceptance.

BG pins SCS **35.21** in `manifest/artifacts/stratagems-35.21.toml`. Inspected
upstream tag `v35.21`: `5dc240e9ff85d3909b6436e54b56297d3b61e33b`, verified with
`git ls-remote`. Official release-archive parity remains unchecked. This research
does not select an IWDEE pin or components.

## Finding

SCS 35.21 does more on IWDEE than assign generic combat AI. Its mage and priest
components have explicit IWD assignment routes that invoke shared spellbook
allocation and script generation, including defensive choices and preparation
blocks. IWD-specific customization data and script additions also exist.

This corrects the broad absence hypothesis. Coverage, effective overrides, balance
and timely defenses remain installation/runtime questions.

## Source trace

| Layer | Verified source behavior | Primary evidence at the inspected ref |
|---|---|---|
| Component entry | Smarter Mages 6030 and Smarter Priests 6040 invoke their respective installers; both require the general-AI component 5900. | [Installer, lines 1057-1069](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/setup-stratagems.tp2#L1057-L1069) |
| IWD dispatch | Both installers load spell-choice, individual-override and instant-prebuff data, then explicitly dispatch to IWD routines. | [Mage entry, lines 24-82](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage.tpa#L24-L82); [priest entry, lines 24-75](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest.tpa#L24-L75) |
| AI assignment | IWD routines scan area-actor and creature script slots against authored IWD script lists, then invoke the shared caster transformation. This is script-list matching, not proof that every caster is selected. | [Mage route, lines 172-295](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage_iwd.tpa#L172-L295); [priest route, lines 242-361](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest_iwd.tpa#L242-L361) |
| Generated spellbooks | Shared transformations determine caster type and level, allocate spells, and generate a script from those choices. Allocation replaces existing spellbooks, chooses defenses, fills available slots from spell lists, and writes the resulting spells. | [Mage transformation, lines 8-45](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage_shared.tph#L8-L45), [allocation, lines 545-612](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage_shared.tph#L545-L612), [spell write, line 853](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage_shared.tph#L853); [priest transformation, lines 8-61](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest_shared.tph#L8-L61), [allocation/write, lines 340-468](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest_shared.tph#L340-L468) |
| Generated prebuff scripts | Shared script builders load the instant-prebuff map and pass each caster's spells/overrides to the compiler. Setup templates include short, medium, long and very-long preparation blocks. | [Mage builder, lines 313-366](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage_shared.tph#L313-L366); [mage setup, lines 49-56](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/ssl/main/magesetup.ssl#L49-L56); [priest builder, lines 160-229](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest_shared.tph#L160-L229); [priest setup, lines 10-17](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/ssl/main/priestsetup.ssl#L10-L17) |
| Game-specific overrides | The loader explicitly reads the active game's spellchoice, kit, level, alignment, boolean and variable tables, including IWD. Both IWD routes can append authored BAF/SSL additions to generated scripts. | [Override loader, lines 241-275](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/caster_shared/caster_shared.tph#L241-L275); [mage additions, lines 233-239](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/mage/mage_iwd.tpa#L233-L239); [priest additions, lines 301-307](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest_iwd.tpa#L301-L307) |

## Anonymous source inventory

Counts describe authored source only, not affected actors or successful patches.

| IWD source inventory | Mage | Priest |
|---|---:|---:|
| Nonblank, non-comment rows in `override/iwd/spellchoice.2da` | 9 | 34 |
| Populated override categories out of six (alignment files are empty) | 5 | 5 |
| BAF/SSL files in `iwd/top/` | 14 | 23 |

Tables were read directly; snippets were counted in the tag tree. These establish
some bespoke work, not encounter-redesign breadth or parity with BG.

## Follow-up before designing enhancements

1. Compare the selected official release payload with this source and freeze IWD
   component choices and preparation/difficulty settings.
2. Privately verify assignment coverage, spellbooks, emitted prep blocks and actual
   individual-override effects in generated output.
3. Resolve an apparent producer/consumer naming difference: the shared loader
   writes the three-key `override` array, while `individual_choices` reads
   `override_spellchoice`. Aliasing elsewhere and release-payload differences were
   not checked. This is an investigation lead, **not a confirmed defect**.
   [Consumer, lines 354-365](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/caster_shared/caster_shared.tph#L354-L365).
4. Measure first-contact behavior: existing prebuff actions still have preparation,
   combat and sight/detection conditions. Consider CBR readiness only for a
   demonstrated gap, preserving costs, durations, difficulty and interruption.
   [Action conditions, lines 470-508](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/caster_shared/caster_definitions.ssl#L470-L508).

Publish aggregate results only. No patch or recipe selection is made here.

## Install-time findings (2026-10-28)

Checked in a private, disposable IWD:EE 2.7.3.0 lab built from the Chriz Easy IWD
recipe, which installs SCS 35.21 with its general AI and its smarter mages and
priests. Counts are aggregate; encounter details stay private.

- **One creature variant has no combat AI.** The priest route sets a default script
  that does not exist, a one-letter typo of an existing SCS priest script, before
  SCS's creature pass
  ([line 109](https://github.com/Gibberlings3/SwordCoastStratagems/blob/5dc240e9ff85d3909b6436e54b56297d3b61e33b/stratagems/priest/priest_iwd.tpa#L109)).
  That variant keeps its original spells and runs no combat script; its sibling
  variant is converted normally.
- **Two priest scripts are missed.** One entry of SCS's IWD priest list misspells an
  IWD priest script and another IWD priest script is absent. SCS ships top-files for
  both, but their priests keep the original AI and get no SCS precasts.
- **Melee AI runs ahead of the caster AI on five placements.** Where an IWD caster
  also carries a generic melee script in an earlier script slot, the general AI turns
  that script into SCS's melee AI. It attacks every round within melee range and
  walks to the nearest enemy when idle, so the caster AI acts only at a distance.
  Measured at Core Rules: a caster placed next to the party attacked in 154 of 180
  samples and cast nothing.
- **Priest precasts start on Hard.** With SCS's default preparation setting, priests
  precast their strongest self-protection only on Hard and Insane (on Insane only,
  for priests created out of sight) and buff during combat below that; SCS mages
  precast from Normal. In the lab, priests precast Armor of Faith on Hard and Insane,
  not on Core Rules.

Christopher's own add-on, Chriz IWD Encounters, repairs the first three and adds
modest priest preparation from Core Rules up. It is released as
[v0.1.0](https://github.com/Chrizhermann/chriz-iwd-encounters/releases/tag/v0.1.0);
its recipe entry is [draft PR #42](https://github.com/Chrizhermann/chriz-bg-collection/pull/42)
and merges after alpha.22.

# Bard spell progression: selected collection policy

Date: 2026-09-19; updated 2026-09-20. Status: source release integration record; the user reported
"Works great" after the BG2EE/EET live smoke test on 2026-09-20. This is bounded live
acceptance, not exhaustive kit/mod coverage. IWDEE native-signature support is included
from binary inspection, but its runtime has not been tested. This records the source
release policy; adoption by the runnable collection recipe remains a separate integration.

## Source release versions

- [chriz-bg-rebalance v0.5.0](https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/v0.5.0):
  shared provider `420` and base-game policy `421`.
- [Bardic Wonders v2.9c-balance.6](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.6):
  kit adapter and descriptions `3010`.

These are the source versions selected for this feature. Collection artifact pins and
release-asset digests are not assigned by this documentation change.

## User-selected default

| Group | Base progression | Maximum normal spellbook level |
| --- | --- | --- |
| Most ordinary bards; includes unkitted Bard and Jester | IWD/un-nerfed, with tiers 8–9 disabled | 7, first available at bard 21 |
| Blade, Bardic Wonders Dancer, Skald | Vanilla BG2 bard progression | 6 |
| Bardic Wonders Kapellmeister and Darkbloom, when installed | Full IWD/un-nerfed progression | 8, first available at bard 29 |

"IWD through seventh level" is not CDTweaks' PnP table: PnP first grants seventh-level
slots at bard 25. The policy changes base slot capacity; existing kit modifiers remain
unless separately rebalanced. Preserve Dancer's -1 slots, Kapellmeister's +2 slots,
Skald's early casting restrictions and caster-level penalty, and Strategist's stance.
Avoid double-counting those adjustments in the new tables.

The caps concern normal spellbook progression. Scrolls and separate innate/HLA abilities
are not implicitly prohibited. In particular Kapellmeister's Song of Universal Harmony
currently offers spell-level 1–9 selections; its balance is a separate decision.

## Additional kits and compatibility boundaries

- Darkbloom is selected for the eighth-level group when the kit is installed. Its extra
  slots and expanded druidic book support the classification.
  The Bardic fork currently excludes Darkbloom from recommended Spell Revisions installs;
  that is a separate copied-spell issue, not a restriction enforced by the progression
  component. Darkbloom is optional: absent kits are skipped, and installed Darkbloom
  receives IWD8 without changing the other kits' mappings.
- Troubadour, Deathsinger and Strategist should initially use IWD7. Their specialized
  spell lists or casting stance do not by themselves justify full eighth-level access.
- Storm Drummer and Abettor of Mask fit the general IWD7 default unless separately revised.
- There is no additional dedicated spellcasting bard kit in vanilla or the inspected SCS
  bard changes. Artisan's ordinary bard kits live in Bardic Wonders, not its main Kitpack.

Register recognized ordinary spellbook bards explicitly. Do not automatically change
Warlock, Aura's Artificer, unfamiliar bard-based casting systems, Gallant, or the
Mage/Bard and Fighter/Mage/Bard multiclass implementations. They need separate adapters
or unchanged native fallback.

## Repository responsibilities and standalone use

Implemented ownership split:

- `chriz-bg-rebalance`: independently selectable neutral EEex progression provider
  component `420`, plus separate base-game bard defaults and descriptions in component
  `421`. These source component numbers are not yet runnable collection rows.
- `Bardic-Wonders-Chriz-Balance-Patch`: late component `3010` for Bardic Wonders kit
  registrations and matching descriptions, requiring provider `420`. Baseline `421` is
  optional for standalone use; the intended collection order is `420` → `421` → `3010`.
- `The-Artisan-s-Kitpack-Chriz-Balance-Patch`: only its own consumers/integrations. Its
  Garrick Troubadour assignment should inherit the Bardic Wonders mapping automatically.
- `chriz-bg-collection`: recipe, exact version pins, prerequisites and feature selection.
  No absorption of another mod's runtime implementation.

The Bardic patch remains usable without the collection. Progression-dependent features
require EEex and the small shared provider component; other existing features need not.
Standalone `420` → `3010` changes the supported Bardic kits while retaining the user's
chosen native Bard progression. Native-kit description notes are refreshed only when
their actual registered table matches that note.
Prefer this shared dependency to independent copies of native hooks. A one-download
distribution can be considered later by bundling the same separately installed dependency,
with one versioned WeiDU identity and a clear reuse/update rule.

Only one provider may own the runtime hook. Different patches contribute data through the
same interface. Detect provider capabilities/version, not only an arbitrary filename;
reject conflicting registrations or an unsupported provider instead of silently mixing
versions. The BG2EE/EET user smoke test is complete. Runnable collection keys and pins
must be added together in the recipe integration, using the verified release assets.

## Collection integration

The old `CDTWEAKS 2270` default applies the un-nerfed table to all bards. The new policy
supersedes that as the intended future default. When the released feature enters the
recipe, remove 2270/2271 from its automatic selections. The provider ships its own named
progression resources and does not require CDTweaks just to obtain those tables.
Do not silently install the old global IWD8 behavior while describing it as the new policy.

Keep the feature and its matching description changes atomic in the collection UI.
Selecting a dependent Bardic change must select the supported core automatically; declining
the core makes that dependent feature unavailable. Independent Bardic components remain
selectable. A final kit-registration/description step must observe the installed kits and
run after mods that would overwrite those descriptions. Audit SCS enemy spell-allocation
reads separately if the policy is intended to govern generated enemy spellbooks too.

The collection target remains BG2EE/EET **2.7.3.0**. The provider also recognizes the
native bard-table lookup in the inspected **IWDEE 2.7.3** binary. Compatibility is checked
through the required EEex loader/APIs and a unique native signature, without a fixed
executable-version, file-offset or EEex pattern-database-filename gate. The runtime selects
persistent native table pointers without per-slot Lua calls. Binary recognition does not
establish IWDEE runtime acceptance. The BG2EE/EET user smoke test passed; focused LuaJIT
and disposable WeiDU checks supplement it. Source release publication alone does not
activate new components in the existing collection recipe.

### Runnable recipe integration still required

At collection commit `02080107aaf7f76c3e8126c09dce681b4a36a64a`, the curated recipe pins
Rebalance `v0.3.2` and Bardic Wonders `v2.9c-balance.3`, and selects CDTweaks `2270`.
This documentation commit does not change that published behavior. Integrating the new
default requires coordinated release-asset manifests with verified digests, mod component
lists, ordered runs, an atomic feature/preset selection, generated recipe reconciliation,
focused recipe checks, and a collection release. Preserve the captured
`manifest/install-order.tsv`; it is historical reference, not the editable future recipe.

In particular, the current late Bardic dialogue run precedes the Rebalance run. A new
`3010` run must follow provider `420` and policy `421`; adding `3010` to that earlier run
would violate the provider dependency. Treat separately released mod defaults and an
installed collection default as distinct completion states.

Descriptions must explain the seven-level Bard baseline, six-level kit disadvantage and
eight-level specialist advantage. Keep separate existing extra-slot, caster-level and
spell-access rules accurate. Use wording about spellbook progression so the text does not
accidentally promise to restrict scrolls or innate HLAs.

## Lightweight acceptance

Use focused static checks for the selected tables, registrations, prerequisites and
description changes. The user supplied the BG2EE/EET manual result, "Works great", on
2026-09-20. Do not expand that result into exhaustive compatibility or IWDEE acceptance.
Do not launch or modify the reference game on the user's behalf. No exhaustive cross-mod
validation matrix is required for this first implementation.

Three disposable real-WeiDU fixtures selected the actual Bardic Wonders component `3010`
with Darkbloom absent, present, and present with provider `420` but without baseline `421`.
They verified identical other-kit mappings/descriptions with or without Darkbloom,
Darkbloom's IWD8 registration and byte-for-byte preservation of its bonus-slot SPL, and
unchanged native mappings/descriptions when `421` was omitted. These are installer checks,
not live verification of Darkbloom's imported spells.

For a repeat of the manual check, set a test bard's **total XP to 5,000,000**, complete level-up, and
inspect the normal spellbook slots for the selected progression. Compare a regular bard
(through seventh level), a restricted kit (through sixth), and a spell specialist
(through eighth), accounting for the retained kit slot bonuses or penalties. Report
manual acceptance only after the user supplies the result.

The read-only reference's `override/XPLEVEL.2DA` BARD row sets level 29 at **4,180,000 XP**,
level 32 at **4,840,000 XP**, and level 33 at **5,060,000 XP**. Thus 5,000,000 total XP is
enough for bard level 32 on that table. Recheck the test install's BARD row if another mod
changes XP thresholds; the required target is bard level 29 or higher, not a fixed XP
amount for every installation.

## Evidence

- [Full engine and integration investigation](https://github.com/Chrizhermann/chriz-bg-rebalance/blob/v0.5.0/research/09-kit-spell-progression-and-caster-level.md).
- Local Bardic fork: `BardicWonders/lib/{kapellmeister,darkbloom,strategist,skald,spell_level}.tpa`
  and README's Darkbloom/Spell Revisions exclusion.
- Existing curation: `docs/curation/components/BARDICWONDERS.md`, `CDTWEAKS.md`,
  `ARTISANSKITPACK_NPC.md`, and `CHRIZ-BG-REBALANCE.md`.
- [Primary Bardic Wonders descriptions](https://theartisanbg.github.io/The-Artisans-Corner/bardic-wonders).
- [Tweaks progression comparison](https://gibberlings3.github.io/Documentation/readmes/readme-cdtweaks_tables.html#bard).

This collection update changes documentation only. Game files, captured install order,
release pins and runnable manifest entries are unchanged.

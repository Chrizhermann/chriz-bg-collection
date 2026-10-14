# CHRIZ-BG-REBALANCE — components

The October 7 target is **v0.8.1**. For the next release, Christopher moved
component `301` out of normal defaults into the planned Challenge Mode choices.
It stays available, but unchecked; all other existing defaults stay unchanged.
The historical September/October release decisions below still describe those
released versions. Christopher approved component
`301` as a recommended default on **2026-09-26**, based on earlier in-game testing;
fix any issue in a follow-up rather than gating on more testing. The
[September 16 candidate](../../alpha16-candidate-2026-09-16.md) approved dragon
components `110` and `111` as optional, unchecked additions. On September 29,
Christopher changed `111` to a recommended default; `110` stays unchecked.
The release retains component `120`
weapon-protection repair, component `121` EEex/SCS ambient-readiness bridge, and the
component `401` correction for Artisan CLAB cells packed into `ABILITY1`.

The recipe pins **v0.8.3** (published 2026-10-14), which adds potion uniqueness
`330`/`331` and the Heroism family `332` (see below).

32 curated entries, 23 recommended selections for the next release. Bard spell-level
ceilings `420`/`421` are one checked choice: **default (Christopher's decision,
2026-10-13)** (see below).
✓ = selected and receipt-verified in the isolated
2026-09-03 release candidate. Subgroup = choose one.

## Bard progression release and intended default

[Rebalance v0.5.0](https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/v0.5.0)
adds the shared EEex provider `420` and base Bard policy `421`. The intended collection
order is `420` → `421` → Bardic Wonders `3010`, after kit installation and competing
description edits. Treat progression and its descriptions as one default-on feature.

`421` gives Bard and Jester IWD progression through spell level 7, with Blade and Skald
retaining BG progression through level 6. Provider `420` alone changes no kit policy;
standalone Bardic users may select `420` → `3010` without `421`. The provider requires
compatible EEex capabilities and a unique supported native signature, without a fixed
EEex version or executable-offset requirement.

Christopher reported "Works great" after the BG2EE/EET live smoke test on 2026-09-20.
IWDEE 2.7.3 has binary and offline installer coverage, not live runtime acceptance.
See the [selected policy and release integration boundary](../../research/2026-09-19-bard-progression-policy.md).
On 2026-10-13 Christopher made `420`/`421` a recommended default; see the bard
spell-level ceilings section below. The table below retains its existing
release-candidate provenance.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 130 | EET elemental arrows: BG2 damage | Item balance |  | | default |
| 200 | Hardiness: 40% physical resistance | Spell balance | Hardiness physical resistance | | default |
| 201 | Hardiness: 30% physical resistance | Spell balance | Hardiness physical resistance | | optional |
| 202 | Hardiness: 20% elemental and magic-damage resistance | Spell balance |  | | optional |
| 210 | Spell Revisions Pierce Magic: bounded resistance reduction | Spell balance |  | | default |
| 211 | Spell Revisions Spellstrike: short spell failure | Spell balance |  | | default |
| 220 | Bracers of Blinding Strike: SR Improved Haste | Item balance |  | | default |
| 310 | Party physical resistance: diminishing returns above 80%, maximum 90% | Combat balance |  | | default |
| 320 | Combat cloud forms: replace damage immunity with 75% resistance | Combat balance |  | | default |
| 100 | SCS: Telekinetic Storm — restore save vs. spell for half damage (+ bypass Mirror Image) | SCS adjustments |  | ✓ | mandatory |
| 101 | SCS: Adventurer's Mart — restore the five Freedom scrolls (spell tweak orphaned in v35) | SCS adjustments |  | ✓ | default |
| 110 | SCS: dragon melee delivery and death-protection rebalance | SCS adjustments |  |  | optional |
| 111 | SCS: dragon wing-buffet spacing rebalance | SCS adjustments |  |  | default |
| 120 | SCS + Spell Revisions: repair false weapon-protection semantics | SCS adjustments |  | ✓ | mandatory |
| 121 | EEex + SCS: ambient caster readiness and one honest first-contact defense | SCS adjustments |  | ✓ | default |
| 301 | Emotion, Courage / Emotion, Hope — mutually exclusive beneficial emotions | Cross-cutting audits |  |  | optional |
| 302 | Flail of the Ages — tiered Slow saves and haste-compatible +5 | Item balance |  |  | default |
| 400 | Cleric of Tempus: weapon training — axe, longsword, crossbow, and two-pip mastery | Class and kit revisions |  | ✓ | default |
| 401 | Cleric of Tempus: Holy Power — automatic semantic detection (recommended) | Class and kit revisions | Cleric of Tempus: Holy Power compatibility | ✓ | default |
| 402 | Cleric of Tempus: Holy Power — force true-doubling compatibility | Class and kit revisions | Cleric of Tempus: Holy Power compatibility |  | |
| 403 | Cleric of Tempus: Holy Power — force additive compatibility | Class and kit revisions | Cleric of Tempus: Holy Power compatibility |  | |
| 404 | Cleric of Tempus: Chaos of Battle — one announced battle tide per cast | Class and kit revisions |  | ✓ | default |
| 405 | Cleric of Tempus: Divination toll — Tempus grants no future sight | Class and kit revisions |  | ✓ | default |
| 406 | Engine-table variant: full warrior attack progression (CLSWPBON, stronger than advertised) | Class and kit revisions | Cleric of Tempus: weapon specialization grants extra attacks |  | |
| 407 | EEex variant: exactly +1/2 attack while wielding a specialized weapon (recommended) | Class and kit revisions | Cleric of Tempus: weapon specialization grants extra attacks | ✓ | default |
| 409 | Cleric of Tempus: EEex specialization APR — re-ship the listener (migration repair for the v0.1.0 runaway-APR build) | Class and kit revisions |  |  | |
| 408 | Cleric of Tempus: updated kit and ability descriptions | Class and kit revisions |  | ✓ | default |
| 420 | EEex: independent kit spell-progression provider | Class and kit revisions |  |  | default |
| 421 | Bard progression: IWD through level 7; Blade and Skald use original progression through level 6 | Class and kit revisions |  |  | default |
| 330 | Potion uniqueness: all temporary potion types | Cross-cutting audits | Potion uniqueness |  | optional |
| 331 | Potion uniqueness: all temporary potion types except Potions of Genius | Cross-cutting audits | Potion uniqueness |  | default |
| 332 | Heroism family: Heroism, Greater Heroism and the Potion of Heroism do not combine | Cross-cutting audits |  |  | default |

## Atomic Tempus UI behavior

- Show one default-on checkbox named **Cleric of Tempus rework**.
- On selects exactly `400`, `401`, `404`, `405`, `407`, and `408`, in source order. Off
  selects none of them. Do not expose the six rows as independent toggles.
- Keep `402`, `403`, and `406` unavailable. Component `409` is a migration-only repair
  for an existing v0.1.0 installation and is never selected on a fresh installation.
- The rows retain `default` rather than `mandatory` because excluding the atomic Tempus
  checkbox must remain possible while using the independent SCS adjustments.

## Dependencies and order

- Component `100` requires SCS `2500` (`extra_arcane_spells`); the curated SCS preset
  includes it. Component `101` requires SCS `2000` or `5900`. Both run after SCS.
- Components `120` and `121` inspect the effective SCS/SR/EEex stack and therefore run as
  late compatibility components. On the researched stack, run `120` before `121`.
- Dragon `110` requires Smarter Dragons and EEex; `111` requires Smarter Dragons and
  spaces wing buffets 18 seconds apart rather than 6. `111` is default-on and can
  be disabled; `110` stays unchecked. They are independent options.
  Their inclusion is source-screened; native combat acceptance
  remains pending as recorded in the September 16 candidate.
- Run the Tempus bundle after SCS, final Spell Revisions processing, Artisan's Kitpack
  proficiency infrastructure, and EEex.
- Component `400` requires Artisan's `C0PR#C4`, its permission spells, and its custom
  proficiency stat. Component `407` requires EEex.
- Component `408` consumes artifacts from `400`, `404`, `405`, and `407` and assumes the
  `401` family. The atomic bundle guarantees that chain. Component `401` supports either
  SR or non-SR spell mappings, but must inspect the final installed resources.

## Bard spell-level ceilings — default (Christopher's decision, 2026-10-13)

- `420` (provider) and `421` (base bard policy) form one atomic choice,
  **Bard spell-level ceilings**: Bard and Jester reach 7th-level spells at bard
  level 21; Blade and Skald stop at 6th. Requires EEex. Existing characters get
  the new capacity at their next normal level-up.
- Bardic Wonders `3010` extends it to Bardic Wonders kits and requires this choice
  (see `BARDICWONDERS.md`). Deselecting Bardic Wonders leaves `420`/`421` valid.
- Both are **default (Christopher's decision, 2026-10-13)**: checked in the
  recommended preset and still deselectable. `3010` is selected whenever Bardic
  Wonders and the base ceilings are selected. Its late run carries its own
  release-tail approval and static evidence in the curated profile's release gate.
- Order follows the v0.8.1 TP2: `...408, 302, 420, 421, 310, 130, 220, 320` in the
  existing post-EET-end run.

## Potion uniqueness and the Heroism family — v0.8.3 (Christopher's decision, 2026-10-14)

- `330` and `331` are one at-most-one choice. Each temporary potion type works once at
  a time on its drinker: max-HP potions (Heroism, Power and similar) keep the first dose,
  every other type restarts its duration instead of stacking. `331` lets Potions of
  Genius stack as before; `330` does not.
- **`331` is default** in the recommended preset; `330` is offered and unchecked.
  Christopher wants `330` instead of `331` in Challenge Mode. The current feature-group
  model cannot switch a choice-group variant (members must be optional and outside any
  choice group), so that switch is not built yet; pick `330` by hand until it is.
- **`332` is default**: Heroism, Greater Heroism (Bardic Wonders), their copies and the
  Potion of Heroism do not combine; the first one active wins. It works with or without
  `330`/`331`.
- No EEex. Potions are recognised by what they do, so potions from other mods are
  included; they run at the end of the existing Rebalance run in TP2 order
  (`..., 320, 330/331, 332`). Later runs (Modpack, Safana, Klatu, BuffBot, EEex Remote)
  add no potion types. Effects already active in an existing save can stack once more.

## Emotion — optional for the next release

The October 9 decision supersedes the September 26 default: `301` remains an
independent, ready option and is off in the normal recommended preset. Baeloth's
separate challenge choice also remains optional/off. The broader Challenge Mode
group is separate work; this change does not move any other default.

### Historical September 26 release decision

Component `301` replaces the other beneficial Emotion on each target; the latest
wins and recasting refreshes. In the collection it rebuilds IWDification's Courage
and Hope in place. The post-EET-end run follows IWDification, SCS `5900` and Spell
Revisions. Later CDTweaks `2312` keeps hostile save-coupled removers aligned and
does not change Courage/Hope; Modpack `430` leaves their fixed-level scrolls alone.
The offered run follows source order:
`100, 101, 110, 111, 120, 121, 301, 400, 401, 404, 405, 407, 408`.
Dragon `110` remains unchecked; `111` is default-on following the September 29 decision.

The v0.4.0-to-v0.6.0 intake adds `301` and the unselected `420`/`421`; all previously
selected components are unchanged. This default applies to new installations /
the next playthrough, not an automatic edit to an existing game. Earlier live
checks used standalone spells; IWDification provider mode has installer and
transformer fixture coverage only. A quick Hope/Courage replacement and refresh
check in the next install is worthwhile, explicitly **not a release gate**.

## Remaining follow-up

September 29: Flail component `302` is an approved default in released v0.7.0.
The +5 Free Action permits Haste/Improved Haste. Slow saves versus Breath at
-2 for +3, -4 for both +4 variants, and -6 for +5; proc chance, duration and damage
remain unchanged. It follows `301` in the existing post-EET-end run, after SR,
SCS and item-description changes, without an EEex dependency. Thirteen focused
real-WeiDU synthetic/resource checks and the owner's full suite passed; this is
not live gameplay acceptance. Existing `301` stays selected and `420`/`421`
stay unselected.

- Stage the fixed APR implementation against EEex v1.2 and verify 1.5 APR with a two-pip
  weapon, 2.5 under Holy Power, no cycling, and prompt return to normal with a zero-pip
  weapon. The earlier installed test used EEex v0.11, so it is not target acceptance.
- Components `120` and `121` are now curated and selected. Current-version component `121`
  passed ambient delivery/accounting and the neutral-to-hostile urgent path; its older-EEex
  fallback remains a nonblocking compatibility check.
- Hardiness `200` is the 40% physical-resistance default; `201` is an exclusive
  30% alternative. Optional `202` adds 20% elemental/magic-damage resistance to
  either physical choice. All require SR `65`; this is not magic resistance.
- Pierce Magic `210` requires SR main and EEex; Spellstrike `211` and bracers
  `220` require SR main. These are approved defaults, independently selectable.
- Physical resistance `310` requires EEex: half effectiveness above 80%, capped
  at 90%, for party members only. The unimplemented 95% alternative is not offered.
- EET arrows `130` follows SCS `2000` and item writers. Cloud `320` replaces
  cloud-granted damage immunity with 75% resistance; other values and scripted
  escapes stay unchanged.
- The offered batch follows TP2 declaration order, including its nonnumeric tail:
  `...407, 408, 302, 310, 130, 220, 320`. Never sort component IDs numerically.
- Recheck whether the final SCS release supersedes `100` or `101`, and refresh the source
  repository's stale component documentation before publishing.

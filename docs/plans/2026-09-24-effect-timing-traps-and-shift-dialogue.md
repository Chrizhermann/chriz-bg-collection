# Effect timing, instant trap detection and shapeshift dialogue

Requested September 24, before returning to the existing installer tests.
Status: backlog and bounded parallel preparation, not implemented or selected for
the current release. Do not modify the stream install, saves, live game, desktop
or game settings. Interactive tests remain user-driven.

## 1. Effect timing under Time Stop and Haste

Requested outcome:

- Periodic effects must not keep ticking during Time Stop.
- Buff durations must not run out during Time Stop.
- Haste on the affected character must not accelerate effect ticks.

Keep these as separate regression cases. A fix for custom Lua callbacks does not
establish a fix for native poison/regeneration, repeating SPL/EFF effects or
ordinary buff expiry. Faster attacks, movement and casting are not the requested
target. Do not change the global engine speed to address this report.

Preparation:

1. Identify representative affected resources and their actual final owner. Check
   native effects, repeated effects and custom Lua timers separately; compare
   regular Haste, Improved Haste, Slow and unmodified speed.
2. Decide the intended clock for the Time Stop caster and Time-Stop-immune actors
   before a broad implementation. Never freeze Time Stop's own end timer. Record
   explicit exceptions rather than accidentally changing encounter mechanics.
3. Prefer an existing verified engine fix where it covers the exact behavior.
   Otherwise route shared mechanics to BG Rebalance and source-specific defects
   to the owning mod. CEBG integrates releases; it does not absorb their code.
4. Check expiry as well as tick cadence. After Time Stop, no accumulated burst of
   missed ticks, duplicate application or permanent extension of a buff is wanted.

Minimum later acceptance: one damaging periodic effect, one regenerating effect
and one short buff; normal/Haste/Improved Haste/Slow/Time Stop; caster, frozen
target and immune actor; pause and save/reload. Include the effective SR/non-SR
variants and SCS delivery where relevant. Synthetic clock tests are preparation,
not proof of the native engine's timing behavior.

Defaults and release inclusion are not decided by this backlog entry.

### September 25: feasibility research requested

Christopher explicitly requested a parallel research pass and a feasible plan for
the Time Stop/Haste work before implementation. He wants more coherent timing,
inspired by AD&D second edition. Do not label all proposed behavior as proven
tabletop rules: the caster, immune creatures and ordinary frozen creatures need
separate policy decisions, as do creature effects versus area spells/projectiles.

Proposed owner is **chriz-bg-rebalance**, whose current guidance explicitly covers
cross-cutting spell-behavior changes. CEBG will integrate a tested release later;
the modpack is not needed as another implementation layer. Prefer a standalone
optional component or separate cadence/expiry components sharing one runtime,
depending on what the source investigation establishes. No number/default has
been allocated and no new repository is required at this stage.

The bounded research deliverable is a source-backed feasibility judgment: usable
EEex hooks versus missing native hooks, supported executable/EEex versions,
known mod interactions, and the smallest isolated proof-of-concept with explicit
stop/go criteria. Do not silently replace native poison/regeneration with a new
damage implementation, rewrite saves, or extend every timer indiscriminately.
Any later prototype and in-game tests need separate authorization. Research is
not a new gate for the already-built installer review package.

**Research completed:** [feasibility report](2026-09-25-timestop-haste-feasibility.md).
There is a credible EEex route, but the inspected ordinary/deferred effect-list
listeners run after evaluation. A universal Lua-only fix is not established.
Next gate: demonstrate a version-verified interception before expiry and periodic
pulses on an isolated copy, or identify a focused EEex extension. Start with one
nonimmune actor, one timed buff and one native periodic family. No prototype or
live test was performed. This is feasibility evidence, not a compatibility claim.

## 2. Instant trap detection

Working scope: remove the wait between valid detection checks, not grant universal
trap knowledge or disarm traps automatically. Preserve Find Traps skill checks,
detectability, normal range and visibility constraints. Whether detection remains
an active mode or also becomes passive is a separate choice; do not silently bundle
the latter into this request.

First check available EEex/tweak components before writing another implementation.
CDTweaks' trap removal and magical-trap placement-delay options are not equivalent
to faster detection. CEBG owns selection/compatibility wiring if a suitable existing
component exists; any new shared implementation belongs in an owning mod.

Minimum later acceptance: immediate discovery of a detectable trap with sufficient
skill; a difficult or intentionally undetectable trap remains hidden; range and
visibility boundaries still matter; detection does not remove the trap; ordinary
disarming and its XP still work. Check area traps and trapped containers/doors.

Default selection and a release promise remain open.

## 3. Talking while shapeshifted

Allow Shapeshifters and Hivemasters to talk in their shifted forms as an independent
**optional QoL component, unchecked by default**. Preserve the form throughout the
conversation; do not automatically revert or consume another transformation.

Start with the effective Artisan kit/form implementation and its balance-patch
owner. Check player-initiated speech, NPC-initiated dialogue and companion
interjections. Limit any patch to the intended forms, including upgraded forms.

Do not remove every polymorph restriction globally. Dialogue permission must not
accidentally allow spellcasting, item use or unrelated actions, bypass genuine
Silence/incapacitation, or change unrelated shapeshifts. Selection should clearly
explain the affected kits and remain independently optional.

Minimum later acceptance: both kits and their upgraded forms; manual and scripted
dialogue; no forced reversion, lost form statistics or extra transformation costs;
the same casting/item restrictions remain. Include the selected SCS/SR form
interactions and save/reload only where implementation affects persistent state.

## Preparation status

Two independent read-only checks cover effect timing and reusable
trap-detection/shapeshift-dialogue components. No implementation or runtime
acceptance is claimed by this document.

### Effect timing: source findings

- [IESDP Time Stop](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op231)
  documents that frozen creatures' frequency-based effects stop firing while
  their durations continue. The caster is immune to its own Time Stop. Thus
  changing buff expiry is an intentional engine-behavior change; the report does
  not by itself prove a regression in our mods.
- [Repeating EFF timing](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op272)
  is explicitly affected by Haste/Slow. Distinguish periodic cadence from expiry.
- `chriz-spell-revisions-patch/spell_rev/components/main_component.tpa` installs
  Time Stop from `spell_rev/spwi9##/spwi909.spl`; that copy operation is not a Lua
  clock implementation. BG Rebalance's `research/05-eeex-apr-cap.md` also records
  why SR Improved Haste's APR bonus must not be equated with every other Haste
  implementation simply from the displayed name.
- BG Rebalance's `chriz-bg-rebalance/lib/tempus_holy_power.tpa` uses opcode 272
  for strength upkeep, with deliberately overlapping child-effect lifetimes.
  A blanket timing patch could break this infrastructure as well as change
  damage/healing. Its `lua/M_CBRAPR.lua` also explains why callback counts are not
  elapsed game time.

Proposed lead owner: **BG Rebalance** for the shared timing design. No existing
general fix surfaced in this bounded pass. Exact resource-specific bugs still
route to their source owner. Reproduce on an isolated copy after the stream, not
through runtime experiments in the active campaign.

### Trap detection: reuse candidate

[BGforge Tweaks and Tricks, Improved Traps](https://github.com/BGforgeNet/bg2-tweaks-and-tricks/blob/master/docs/traps.md)
already provides near-instant detection while retaining a sufficient-skill
requirement. Increased range, intuition and trap fixes are separate options, not
implicitly included in this request. Its Core must follow the selected Improved
Traps options. Start by reviewing the `g_tnt_traps_instant` and `g_tnt_traps_core`
labels against a pinned release, rather than copying this mod into ours or relying
on unpinned numeric component positions. Full collection compatibility and the
range/visibility boundaries above remain to be checked before integration.

The upstream detection script checks TRAPS skill and the active DETECTTRAPS mode;
the separate range option is not needed. Core changes trap scripts/area assignments
and uses class filters, so this is not a proven drop-in for every modded form or
existing saved area. Check Artisan form detection, unusual traps, containers/doors,
priest Find Traps and actual line-of-sight behavior. Do not describe it as a safe
current-save hotpatch from documentation alone.

### Shifted dialogue: narrow source candidate

Owner: **The-Artisan-s-Kitpack-Chriz-Balance-Patch**. Existing source components
are Shapeshifter 5100, its custom sprite option 5110 and Hivemaster 5002; these are
not component numbers for the proposed new QoL option.

The source form items contain a specific Talk-button restriction, opcode 144 with
parameter 2 = 7 and equipped timing 2:

- `ArtisansKitpack/Druid/Shapeshifter/spells/C0SS-W.ITM`
- `ArtisansKitpack/Druid/Hivemaster/spells/c0phive1.itm` through `c0phive4.itm`

[IESDP's disable-button reference](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op144)
identifies that exact button value. A separate off-by-default component can target
just this restriction while preserving the other equipped effects. Do not modify
the base kit choice or unlock other action buttons. The inspected `C0SS-F.ITM`
has no explicit Talk blocker, so do not pretend this one edit explains every
form/dialogue case. Preserve the existing Natural Spell level-14/15 progression.

This is a promising implementation candidate, not a tested fix: incoming scripted
conversations and companion interjections still need the acceptance checks above.

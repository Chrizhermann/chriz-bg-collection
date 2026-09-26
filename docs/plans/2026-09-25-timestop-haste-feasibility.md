# Time Stop and Haste: research-only feasibility

September 25, 2026. Requested by Christopher; source inspection only. No code,
game actions, installations, save edits, runtime probes, or release changes.

## Decision in plain language

**Worth pursuing, but not a proven small Lua fix.** There is a credible EEex-based
route: separate effect time from attack/movement time and pause the appropriate
effect clocks during Time Stop. The hard part is intercepting the engine *before*
it expires an effect or fires a periodic pulse. The ordinary convenient EEex
listeners inspected here run after effect processing; they are not sufficient
evidence for a reliable universal fix.

Proposed owner: **chriz-bg-rebalance**, as shared spell-behavior work. Keep the
engine-hook provider reusable; any necessary EEex capability should preferably
be contributed upstream rather than silently modifying third-party binaries.
CEBG would later expose/pin accepted components, not own this implementation.

Do not advertise complete AD&D 2e compliance from this research. The intended
rules for caster/immune actors, Slow, and area spells need an explicit policy.

## Sources and versions actually inspected

- CEBG's current manifest pins **EEex 1.2.0**, not latest: `manifest/mods/eeex.toml:2`
  and `manifest/artifacts/eeex-1.2.0.toml:3`. Its official ZIP is pinned to SHA-256
  `ac2d81bc5e6fdba04a686338a969507e5beca9cb082b2809a556e0874e690a88`.
- Official [EEex 1.2.0 source](https://github.com/Bubb13/EEex/tree/204040502ac920161d60046d90b766b8d2d08152)
  is commit `204040502ac920161d60046d90b766b8d2d08152`.
- Latest published release checked was [EEex 1.3.0](https://github.com/Bubb13/EEex/releases/tag/v1.3.0),
  September 16, source commit `9e8098ee696ca4b0bc055e7ff74b283704a8c02e`.
  Master was `a0502166ea7aaeacc39c6349efe2fb40bbbe203e`; do not install master.
- Native EEex implementation also lives in
  [InfinityLoader](https://github.com/Bubb13/InfinityLoader/tree/f14a00527286f819b8034411bc92ca719cfe3c86),
  inspected at `f14a00527286f819b8034411bc92ca719cfe3c86`. This checkout was not
  proven byte-identical to the native DLL inside either release ZIP.
- Source packages contain separate 2.6.6.0/2.7.3.0 loader material; this is not a
  claim that a proposed new hook supports both. Prove against the selected
  executable/build before enabling it. No live executable was inspected here.

## What exists, and the important missing proof

1. **An appropriate game clock exists.** EEex's own
   [B3TimeStep.lua:48-54](https://github.com/Bubb13/EEex/blob/204040502ac920161d60046d90b766b8d2d08152/EEex/copy/EEex_scripts/B3TimeStep.lua#L48-L54)
   reads `m_worldTime.m_gameTime`. This is the gameplay clock, not wall time or
   render frames. B3TimeStep itself is a manual paused-game stepping module,
   **not a Time Stop effects fix**. Its special write to global time is not a
   pattern to reuse for changing every actor's effect clock.

2. **Effect-list observers exist, but are late.** Both inspected release sources
   expose ordinary and deferred ListsResolved listeners in
   [EEex_Opcode.lua:8-22](https://github.com/Bubb13/EEex/blob/204040502ac920161d60046d90b766b8d2d08152/EEex/copy/EEex_scripts/EEex_Opcode.lua#L8-L22).
   The [hook placement:44-101](https://github.com/Bubb13/EEex/blob/204040502ac920161d60046d90b766b8d2d08152/EEex/copy/EEex_scripts/EEex_Opcode_Patch.lua#L44-L101)
   is after both effect lists, with the deferred call near ProcessAI's return.
   An already-expired buff cannot safely be restored by adding duration there;
   a poison tick cannot be undone by merely adjusting the next timestamp.
   Frozen actors' callback coverage also remains unproven. Do not confuse a
   once-per-AI-pass callback with a once-per-elapsed-tick clock.

3. **Time Stop state is identifiable in current native source.**
   [EEex.cpp:1614-1617](https://github.com/Bubb13/InfinityLoader/blob/f14a00527286f819b8034411bc92ca719cfe3c86/EEex-Core-Shared-Files/source/EEex.cpp#L1614-L1617)
   uses actual `m_nTimeStop`, caster ID `m_nTimeStopCaster`, and the target's
   `m_bImmuneToTimeStop`. This supplies a credible semantic predicate. Its Lua
   field exposure and precise counter/transition semantics on the pinned build
   were not runtime-verified. Never infer the end from the SPL's nominal length.

4. **Native clocks really differ.** [IESDP opcode 231](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op231)
   documents frequency effects suspending for frozen creatures while durations
   continue, including area-projectile lifetime. The caster is exempt.
   [Opcode 272](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op272)
   documents cadence changing with Haste/Slow/Disease. Native poison,
   regeneration, repeated EFF, and Lua-owned repeaters must be traced separately.

5. **No supported general pre-expiry/periodic-clock policy callback was found**
   in the bounded inspection of GameObject, GameState, Sprite, Opcode and Fix Lua
   modules/patches, or the native EEex source/header. This is a bounded negative
   result, not proof that no engine hook can be constructed. The next technical
   gate is to locate/version-verify those engine decision points. EEex's existing
   hook machinery makes this plausible, but a new native-level hook or EEex
   extension may be necessary. No full disassembly was attempted.

## Smallest useful next experiment, when authorized

Build one isolated development proof of concept, not a collection-wide patch:

1. Instrument before/after expiry and one native periodic effect, initially on
   the collection's supported Windows x64 game/EEex combination. Log raw game
   time, actual Time Stop state/caster, immunity, effect identity, deadline,
   pulse schedule, and source attribution. Do not infer elapsed time from calls.
2. Give a nonimmune actor one short timed buff plus native poison/regeneration.
   Preserve remaining duration throughout one Time Stop, then resume it without
   an expiry gap, duplicate application, or burst of deferred ticks. Preserve
   normal expiry outside Time Stop. Time Stop's own timer must still end.
3. Normalize one damage/healing pulse family against Haste while keeping attacks
   and movement accelerated. Preserve pulse phase when Haste starts/ends, not
   just total damage over an unusually convenient test duration.
4. If that works at the correct engine boundary, expand to the other pulse
   families, immune/caster policy and effect application/removal during the stop.
   Treat projectiles/area hazards as a separate phase with separate acceptance.

Prefer preserving native effect identities and changing their clock decisions
over deleting/recreating buffs or reimplementing damage. That protects charge
counters, stacking, dispels, source/kill attribution and mod-specific metadata.
Do not temporarily clear a creature's Haste globally or rewind world time.

**Go** only if a callback/hook demonstrably precedes both expiry and pulse
decisions, also covers frozen actors, and stays stable across repeated calls.
**Stop/re-scope** if the only available route is post-expiry repair, blanket
effect recreation, or guessed native addresses. In that case the concrete
deliverable is a small proposed upstream EEex capability, not a pretend Lua fix.

## Compatibility and acceptance boundaries

- Use two separately selectable behaviors if practical: Haste-independent
  periodic timing, and Time Stop duration/pulse behavior; shared provider once.
- Christopher must choose caster/Time-Stop-immune behavior. Freezing those as
  well is not equivalent to fixing frozen actors' duration loss. Decide whether
  Slow/Disease should also stop changing cadence. These choices need not block
  the first nonimmune proof of concept.
- Never freeze Time Stop's own control timer, encounter scripts, or every game
  timer. Permanent/equipped effects, delayed effects and summon lifetimes are
  distinct categories, not ordinary buffs to increment blindly.
- Do not blanket-rewrite opcode 272. BG Rebalance's
  `chriz-bg-rebalance/lib/tempus_holy_power.tpa:1253` uses it for Strength upkeep;
  lines 2270-2279 deliberately overlap child lifetimes to survive Slow.
  Housekeeping effects are not necessarily periodic damage/healing.
- SR and non-SR Improved Haste differ; resolve actual effect shape rather than
  displayed name or a shared SCS spell-state marker. Keep source-specific
  fixes in their owner repos when no shared engine correction is needed.
- Later tests: normal/Haste/Improved Haste/Slow; frozen actor/caster/immune;
  nested or consecutive stops; dispel/refresh/death; pause, rest and transition;
  item equip/unequip; save/reload before/during/after; party and enemy symmetry.
- Persisted absolute deadlines are especially risky. Prefer no new save format;
  if clock compensation needs persistent metadata, prove serialization/load
  normalization and component removal first. Do not keep raw sprite/effect
  pointers across load. Saving during Time Stop has historical engine caveats;
  confirm current behavior rather than trusting an old forum workaround.
- No current-run hotpatch promise. Removal should be tested from a save without
  an active Time Stop first; preserve backups, and never edit users' saves as a
  substitute for correct runtime semantics.

## Evidence limit

This establishes a technically credible research direction and the exact next
stop/go question. It does not prove native coverage, performance, PnP fidelity,
cross-platform support, save safety, or live compatibility. No implementation
time estimate or release commitment is justified yet.

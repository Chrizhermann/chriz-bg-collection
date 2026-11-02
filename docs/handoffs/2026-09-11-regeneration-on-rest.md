# Regeneration spells in automatic rest healing

Status: user-requested QoL improvement; feasibility researched, **not implemented
or live accepted**. Keep separate from any redesign of druids' healing spell lists.
Proposed owner: `chriz-bg-rebalance`, with an SR compatibility/rest QoL component.
The owning repository assigns its component ID. CEBG only pins/exposes its release.

## Findings

- First-hand engine investigations by kjeron and Bubb describe automatic rest
  healing as extracting direct HP-healing effects (opcode 17), not casting the
  whole spell. Other effects and area projectiles are skipped; even Mass Cure
  follows a single-recipient path.
  [kjeron's explanation](https://forums.beamdog.com/discussion/67382/general-mod-questions-thread/p41),
  [Bubb's investigation](https://forums.beamdog.com/discussion/75849/berserker-rage-heals-on-rest).
- Read-only inspection of the reference installation found regeneration (opcode
  98), but no opcode 17, in all seven SR regeneration spells. Cure Light Wounds
  contains 17. This supports the reported exclusion; no native rest test was done.
- Resolve SR's dynamically allocated spell resources through `SPELL.IDS`, not
  hardcoded SPPR numbers. The family uses `CLERIC_REGENERATE_*`,
  `CLERIC_MASS_REGENERATE`, and `CLERIC_REGENERATION_DRUID_VERSION` symbols.
  SR adds them in `spell_rev/components/main_component.tpa`; its existing
  non-stacking refresh code also covers the family.
- No simple eligibility flag/table fix was established. Adding ordinary healing
  effects to regeneration spells would also change normal casts. Converting them
  to delayed healing ticks could change stacking, haste and duration behavior.
- EEex is a possible route, not a demonstrated implementation. Its existing
  `QuickListCountsResetListener` runs after spell-use counts reset, too late to
  discover which copies were unused before resting. A script-action-only listener
  also does not establish coverage of both inn and wilderness UI rest paths.

## Bounded next implementation task

Use a worktree in the owning repository and a disposable game. Start with
Regenerate Light Wounds and establish a pre-rest availability snapshot plus a
reliable successful-rest boundary. Respect the game's healing-on-rest setting.
Do not alter the reference/stream installation or change ordinary casting.

Before extending to the full spell family, prove: unused versus spent copies;
successful, interrupted, denied and cancelled rests; inn and wilderness entry;
repeated rests; save/reload; mixed cleric healing; and no duplicate healing or
unearned spell uses. Capture actual HP and spell-use state, not only callback logs.

Proposed behavior for discussion, not approved balance changes:

- Credit the spell's normal finite healing total, bounded by missing HP; do not
  multiply it by eight hours of rest or by engine time-skip quirks.
- Decide whether Mass Regenerate should keep its ordinary party-wide behavior or
  mimic the engine's limited single-recipient rest-healing behavior.
- Exclude Wish and scripted/story rests until explicitly intended and tested.

If a small reliable hook cannot be demonstrated, record the missing engine hook
and stop the prototype. Do not add a misleading toggle, grant instant healing on
every ordinary cast, or delay the collection release for an open-ended engine dive.

# SRCB_RR_COMPAT — components

Separate compatibility installer from the pinned **v4.21-chriz.5** release.
This records Christopher's existing approval in the
[September 16 candidate](../../alpha16-candidate-2026-09-16.md); it does not add a new choice.

Component `0` is automatic when Spell Revisions core `0` and either Rogue Rebalancing
encounter `11` or `12` are selected. It repairs the affected encounter spellbooks,
Chaos script references and spell-protection checks, after SR and RR but before SCS.
Turning SR off, or omitting both encounters, omits this compatibility run.
The parent is default-checked with those dependencies; its sole component is mandatory
when the parent is included. No historical installation mark is asserted here.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Correct Rogue Rebalancing's spell choices for Spell Revisions | Spell compatibility |  |  | mandatory |

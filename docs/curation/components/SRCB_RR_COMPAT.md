# SRCB_RR_COMPAT — components

Separate compatibility installer from the pinned **v4.21-chriz.7** release (tag commit
`897f3b2`, pinned 2026-10-26).
This records Christopher's existing approval in the
[September 16 candidate](../../alpha16-candidate-2026-09-16.md); it does not add a new choice.

v4.21-chriz.6 (2026-10-23) corrected component `0` after a 2.7.3.0 lab check. Chaos stays
SR's level-7 spell, and Rogue Rebalancing v4.92's prepared level-5 Chaos copies become
level-5 spells where they stand: Hold Monster for Aran Linvail, Feeblemind for the
Zhentarim Mercenary and Domination for Selina. v4.21-chriz.4 and .5 moved those copies to
level 7, where Aran and the Mercenary could never cast them. The three RR scripts cast the
replacements with the target checks they need. v4.21-chriz.7 (2026-10-24) changes only
notes and the version label. The level-5 replacements have not been checked in game yet.
A game that already has an older version keeps it: the release notes allow an in-place
update only while the component is the last WeiDU.log entry, so the new version applies
to new installations.

Component `0` is automatic when Spell Revisions core `0` and either Rogue Rebalancing
encounter `11` or `12` are selected. It repairs the affected encounter spellbooks,
Chaos script references and spell-protection checks, after SR and RR but before SCS.
Turning SR off, or omitting both encounters, omits this compatibility run.
The parent is default-checked with those dependencies; its sole component is mandatory
when the parent is included. No historical installation mark is asserted here.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Correct Rogue Rebalancing's spell choices for Spell Revisions | Spell compatibility |  |  | mandatory |

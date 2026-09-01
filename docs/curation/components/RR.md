# RR — components

Listed at installed version **v4.92**.
14 entries, 7 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Components `9` and `10` form an optional, mutually exclusive `Revised Thievery`
  group; choosing neither is valid.
- Component `1` rewrites Assassin, Bounty Hunter, and Swashbuckler together. Make it
  unavailable with Artisan `7004` or `7006`: otherwise the result is a hybrid in which
  the later Artisan component supersedes only part of RR's revision.
- Component `2` is not generally incompatible with Artisan. Artisan `7004` composes with
  the then-current thief HLA table, while `7006` later supersedes only the Swashbuckler
  part. Keep it available with a partial-supersession note.
- Component `4` is unavailable with Bardic Wonders `1001`, `1002`, or `1003`, which
  replace the Blade, Jester, or Skald tables separately and would leave a hybrid RR True
  Bard. Component `5` is unavailable with Bardic Wonders `2007`, whose bard HLA suite
  overlaps RR's replacement.
- Component `6` is unavailable with Tweaks Anthology `2270` or `2271`; all three rewrite
  Bard spell progression. The reference preset selects `2270`, so RR `6` is disabled
  there.
- Component `999` is cosmetic and useful only when at least one content-bearing RR
  component is selected.

## Integration/follow-up note

RR components are independently installable. If a future preset intentionally wants one
of the hybrid combinations above, review and patch that exact combination rather than
using a generic RR/Artisan compatibility toggle. Install RR before aTweaks if aTweaks is
added later.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Proper dual-wielding implementation for Thieves and Bards |  |  | ✓ | default |
| 1 | Thief kit revisions |  |  |  | optional |
| 2 | Thief High Level Ability revisions |  |  |  | optional |
| 3 | Proper racial adjustments for thieving skills |  |  | ✓ | default |
| 4 | Bard kit revisions |  |  |  | optional |
| 5 | Bard High Level Ability revisions |  |  |  | optional |
| 6 | Proper spell progression for Bards |  |  |  | optional |
| 7 | Additional equipment for Thieves and Bards |  |  | ✓ | default |
| 8 | Upgradeable Equipment |  |  | ✓ | default |
| 9 | Use PnP thievery potions and prevent their effects from stacking |  | Revised Thievery |  | optional |
| 10 | Retain default thievery potions and prevent their effects from stacking |  | Revised Thievery |  | optional |
| 11 | Chosen of Cyric encounter |  |  | ✓ | default |
| 12 | Shadow Thief Improvements |  |  | ✓ | default |
| 999 | BG2-style icons for RR content |  |  | ✓ | default |

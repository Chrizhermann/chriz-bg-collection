# EEFIXPACK — components

The reference install used **Alpha 3**. The collection targets **Beta 2**, the latest
published package reviewed on 2026-09-01.
3 entries, 2 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Core Fixes |  |  | ✓ | mandatory |
| 1 | Beta Core Fixes |  |  |  | |
| 2 | Game Text Update |  |  | ✓ | mandatory |

## Collection behavior

- Reuse one pinned Beta 2 artifact for **two pre-EET runs**: first on the staged BGEE
  source after DLC Merger, then on the staged BG2EE target before EET core. Install
  components `0` and `2` in both runs. EE Fixpack refuses an already-merged EET game.
- Component `1` is reserved for future experimental fixes and is unavailable in Beta 2:
  the package ships no game-specific `*_beta.tph`, so its own prerequisite is false.
  Keep it hidden and excluded.
- Component `2` is mandatory for this English-only collection. It is only available when
  the game language matches the mod language and matching GTU data exists.

## Release and acceptance follow-up

- The installer manifest/engine must represent two logical runs of one artifact; a
  one-phase-per-mod model is insufficient.
- Recheck for a newer packaged release at manifest freeze. Beta 2 predates game 2.7,
  while unreleased `master` contains later compatibility work; do not silently track it.
- Validate components `0` and `2` on both clean 2.7 staged games, then validate the EET
  merge. These source checks are not runtime acceptance.

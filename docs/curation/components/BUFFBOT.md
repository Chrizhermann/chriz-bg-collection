# BUFFBOT — components

Target release: **v1.10.1**, tag commit `c1f2eb6` (pinned 2026-10-23). BuffBot is a
default-on parent mod for the alpha experience. Its two implementation components are
mandatory together and are not exposed as separate checkboxes.

2 entries, 2 current-alpha selections. ✓ = selected for the isolated 2026-09-03 release
candidate.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1 | BuffBot: EEex LuaJIT Support (auto-detected) | Buff automation |  | ✓ | mandatory |
| 0 | BuffBot: In-Game Buff Automation | Buff automation |  | ✓ | mandatory |

## UI and order

- Show one default-on parent checkbox named **BuffBot — in-game buff automation**.
- Selecting it installs both components in source declaration order: `1`, then `0`.
- Install BuffBot after every other mod, the player voicepacks included; only Modpack's
  scroll policy `430` runs after it, because it works on the final scrolls and spells.
  Component `1` verifies or establishes the exact LuaJIT loader state before component `0`
  installs the UI/runtime.
- Require a clean supported EEex runtime and launch the game through `InfinityLoader.exe`.

## Alpha evidence and follow-up

- The release archive is hash-pinned for the private candidate and its payload was compared
  with the tag before installation.
- Automated release evidence covers 457 tests and a targeted live fix test. The collection
  still needs its own final-loader boot and BG1 start/save/reload smoke on the completed EET
  stack; do not infer that from the upstream release checks.
- v1.10.1 (released 2026-10-20) fixes the combat-stop notice
  ([bg-eeex-buffbot#76](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/76)). Against
  v1.10.0 its TP2 changes only the version, so components `1` and `0` are unchanged.


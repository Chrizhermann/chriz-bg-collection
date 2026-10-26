# BUFFBOT — components

Target release: **v1.11.0**, tag commit `66f6840` (pinned 2026-10-26). BuffBot is a
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
- Automated release evidence covers 457 tests and a targeted live fix test. The
  collection's own final-loader boot and BG1 start/save/reload smoke on the completed EET
  stack passed on 2026-10-18 with the then-pinned v1.10.0 (see
  [FOLLOW_UPS](FOLLOW_UPS.md)); v1.11.0 has not been run in a collection lab yet.
- v1.10.1 (released 2026-10-20) fixes the combat-stop notice
  ([bg-eeex-buffbot#76](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/76)). Against
  v1.10.0 its TP2 changes only the version, so components `1` and `0` are unchanged.
- v1.11.0 (released 2026-10-24) takes selected improvements from Zach Hopkins' BuffBot
  Redux: an optional 2x/4x simulation speed for a buff run (an in-game choice; Normal
  stays the default), buff-name search, a named preset picker on right-clicking the HUD
  gear, automatic discovery of item buff powers delivered through child spells, and drawn
  enable/lock cells. It fixes configured spell variants that skipped casting time and
  aura, wearable items refused to characters without a kit, and the restore of
  temporarily equipped items, which could overwrite the first backpack slot on 2.7.3.0,
  corrupt the character name and crash the game. Against v1.10.1 the TP2 changes its
  version and copies
  two more inputs, `BfBotFst.lua` and the `buffbot/visual` folder (control BAMs with their
  PVRZ pages `MOS88809`–`MOS88993`); components `1` and `0` and their order are unchanged.
- The control artwork derives from Pecca's Infinity UI++ v1.15 and is used with
  permission; that permission does not travel with BuffBot's MIT license (see BuffBot's
  `ATTRIBUTION.md`). The collection only fetches the release and redistributes none of it.
- Owner validation for v1.11.0: 1,166 automated tests, and 2.7.3.0 / EEex v1.3.1 labs for
  Normal, 2x and 4x. Its EET/SR/SCS/Modpack matrix ran with Modpack's scroll policy `430`
  uninstalled; BuffBot's notes say the configuration with `430`, which the recommended
  preset installs, is not accepted yet. Not checked in game: the column headers and the
  control artwork.


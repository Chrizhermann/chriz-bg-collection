# BUFFBOT — components

Target release: **v1.12.1**, annotated tag on commit `a644013` (pinned 2026-10-28).
BuffBot is a default-on parent mod for the alpha experience. Its two implementation
components are mandatory together and are not exposed as separate checkboxes.

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
  [FOLLOW_UPS](FOLLOW_UPS.md)); neither v1.11.0 nor v1.12.1 has run in a collection lab
  yet.
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
  v1.12.0 is the first release that draws it.
- Owner validation for v1.11.0: 1,166 automated tests, and 2.7.3.0 / EEex v1.3.1 labs for
  Normal, 2x and 4x. Its EET/SR/SCS/Modpack matrix ran with Modpack's scroll policy `430`
  uninstalled; BuffBot's notes say the configuration with `430`, which the recommended
  preset installs, is not accepted yet. Not checked in game: the column headers and the
  control artwork.
- v1.12.0 (released 2026-10-27) adds Icewind Dale: Enhanced Edition (IWD:EE) support,
  which this recipe does not use. The panel's buttons, tabs, scrollbars and frame now draw
  the control artwork that v1.11.0 shipped without using, and the theme colours reach the
  screen. New in-game choices, each off or unchanged until the player picks it: a cast
  order per preset (*Order: Manual*, or *Order: Duration* for the longest-lasting buffs
  first), "Refresh buffs that are nearly over" in the EEex options (off, or under 1
  minute, 5 minutes or 1 hour left), and a panel toggle *Active: Skip*, *Refresh Low* or
  *Force All* that returns to Skip when the game restarts. The F12 preset innate abilities
  are gone: right-clicking the HUD gear picks a preset instead, and a save from an earlier
  version loses the old abilities about two seconds after loading.
- v1.12.1 (released 2026-10-27) fixes three panel layout defects of v1.12.0: captions
  that kept their size while their buttons shrank, so a click could land on the
  neighbouring button; spare tab slots placed outside the panel; and quick-preset menu
  captions left behind when the menu moved. None of them depends on the EEex version.
- Against v1.11.0 the TP2 changes its version, and both components also accept IWD:EE.
  The main component no longer appends the eight F12 innate names to the TLK, no longer
  writes `override/bfbot_strrefs.txt` and no longer copies `BFBOTIB.BAM`, and it copies
  the new `BfBotPln.lua`. The EEex and LuaJIT checks, the language list, the `buffbot/visual`
  art with its PVRZ pages `MOS88809`–`MOS88993`, and components `1` and `0` with their
  order are unchanged. Against v1.12.0 only the version changes. Fresh-download evidence:
  `manifest/evidence/release-intake-20261028-buffbot.toml`.
- Owner validation for v1.12.1: 1,520 automated tests, and panel shots in a BG2:EE 2.7.3.0
  lab with EEex v1.3.1 at 1075×621 and at the 550×350 minimum. v1.12.0's in-game suite
  passed all 28 phases in a BG2:EE 2.7.3.0 lab with EEex v1.3.1, and IWD:EE labs covered
  its installer, the LuaJIT helper, the panel, a buff run and the F12 cleanup. Not checked
  in game: BG:EE, EET, the six dialogs, and six long character names on a narrow panel.
  BuffBot's README still calls the configuration with `430` not accepted.
- Christopher, 2026-10-28: v1.12.1 goes into alpha.22 once it has been tested again; he
  tests it himself.


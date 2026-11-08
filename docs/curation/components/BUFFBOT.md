# BUFFBOT — components

Target release: **v1.16.1**, tag on commit `377bb1d` (pinned 2026-11-08 in both Chriz Easy
installers). BuffBot is a default-on parent mod for the alpha experience. Its two implementation
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
  installs the UI/runtime. Since v1.12.7 component `0` also installs without that state and
  names what is then missing; the recipe still installs both.
- Require a clean supported EEex runtime and launch the game through `InfinityLoader.exe`.

## Alpha evidence and follow-up

- The release archive is hash-pinned for the private candidate and its payload was compared
  with the tag before installation.
- Automated release evidence covers 457 tests and a targeted live fix test. The
  collection's own final-loader boot and BG1 start/save/reload smoke on the completed EET
  stack passed on 2026-10-18 with the then-pinned v1.10.0 (see
  [FOLLOW_UPS](FOLLOW_UPS.md)); no collection lab result is recorded for v1.11.0, v1.12.1,
  v1.12.6 or v1.16.0. For v1.16.1, the Chriz Easy IWD 0.1.9 lab installed a candidate whose
  code differs only in version strings, and BuffBot loaded in its start test (see the
  [IWD 0.1.9 acceptance](../../package-acceptance-iwd-0.1.9-2026-11-08.md)).
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
- v1.12.1 (released 2026-10-27, annotated tag on commit `a644013`; pinned from 2026-10-28
  to 2026-11-01) fixes three panel layout defects of v1.12.0: captions that kept their
  size while their buttons shrank, so a click could land on the neighbouring button; spare
  tab slots placed outside the panel; and quick-preset menu captions left behind when the
  menu moved. None of them depends on the EEex version.
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
  tests it himself. Christopher, 2026-10-29: it goes into the alpha.22 release candidate,
  and he tests it there instead of separately.
- v1.12.2 (released 2026-10-28) fixes the Summons view: its *Party* toggle sat under the
  summon row's own controls, so with a party of one to five the panel could not switch
  back, and the character tabs then looked dead. Tab areas are also laid out again when
  the party or the preset list changes size while the panel is open; before, a companion
  who joined got a tab stacked under the previous last one.
- v1.12.3 (released 2026-10-29) lists only creatures the party conjured in the Summons
  view: a clone link or the engine's summoner link has to lead to a party member
  ([bg-eeex-buffbot#101](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/101)). A
  friendly quest NPC such as Vithal, or a charmed or dominated enemy, no longer appears
  there with his spellbook. A creature whose link cannot be read stays listed.
- v1.12.4 (released 2026-10-29) gives the three colour schemes the right game's names
  ([bg-eeex-buffbot#102](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/102)): the
  warm parchment everyone gets by default is now called *Baldur's Gate* (was
  *Baldur's Gate II*) and looks as before, and a scheme a player picked earlier now shows
  the art of the game it names. On a wide panel the footer buttons spread out in their
  groups instead of sitting in the middle.
- v1.12.5 (released 2026-10-29) draws a pressed button with the loud selected frame
  ([bg-eeex-buffbot#103](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/103)) and
  fixes v1.12.4's default, which opened an install that had never picked a scheme on the
  blue-teal one. Open upstream: a button might keep looking pressed if the engine's hold
  counter goes stale, as on Zach Hopkins' Redux; not seen here
  ([bg-eeex-buffbot#105](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/105)).
- v1.12.6 (released 2026-11-01, tag on commit `171e616`; pinned from 2026-11-01, shipped by
  CEBG 0.1.0 and 0.1.1) lets the fast-forward guard read the game's own menu list instead of
  eight menu names
  ([bg-eeex-buffbot#104](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/104)).
  With every EEex the 2x/4x speed now stops while the options, the character record, a
  level-up, the journal, a shop or a mod's own dialog is open, where it used to run on.
  EEex v1.2.0 lacks the engine's popup check, so earlier releases relied there on the
  eight names alone; the menu list exists in both EEex versions.
- Against v1.12.1 the TP2 changes only its version and two comments. Nine files under
  `buffbot/` change and none is added or removed; the language files, the README and
  components `1` and `0` with their order are unchanged. No release from v1.12.2 to
  v1.12.6 changes BuffBot's save data (`BfBotPer.lua` is unchanged; v1.12.4 to v1.12.6
  also say so in their notes) or an installed file outside BuffBot's own. All 223 files
  under the published `buffbot/` folder are byte-identical to the tag commit's tree.
  Fresh-download evidence: `manifest/evidence/release-intake-20261101-buffbot-chriz-easy-bg.toml`.
- Owner validation for v1.12.6: 1,676 automated tests, and the in-game suite (all 28
  phases) in a headless BG2:EE 2.7.3.0 lab with EEex v1.3.1, where the new guard also ran
  against the real engine; EEex v1.2.0 was simulated there. Not checked in game: BG:EE,
  SoD, EET and IWD:EE (whose interface files name their screens differently), a real EEex
  v1.2.0 install, a real mouse press, a click on the *Party* toggle, and a save and reload
  with a summon in the area. BuffBot's README still calls the configuration with `430`
  not accepted.
- Christopher, 2026-11-01: v1.12.6 goes into alpha.22 and into Chriz Easy IWD; it
  replaces the v1.12.1 pin and the IWD pin of v1.12.1 in PR #51.
- v1.12.7 (released 2026-11-01) ends the bright look a pressed button kept after it was let
  go ([bg-eeex-buffbot#105](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/105)).
  The main component no longer refuses to install without the LuaJIT loader
  ([bg-eeex-buffbot#106](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/106)): it
  installs and names what is missing, which is preset export and import, the log files, the
  file-based in-game tests and every language but English. Everything that casts works.
- v1.12.8 (released 2026-11-02) adds a second reading of the plan: a *By caster* / *By
  recipient* switch beside the character tabs, with one row per buff and everybody receiving
  it as a chip. By caster stays the default. *Uses* now shows how many there are, not how
  many are left over; the BG1 parchment scheme is readable again; Reset no longer sits on
  the frame's corner.
- v1.12.9 (released 2026-11-02) offers five text sizes instead of three (*Tiny* and *Huge*
  are new; the three old steps keep their stored values, so no panel resizes itself), moves
  *Delete Preset* next to *Rename* and *New*, and closes the frame at its corners.
- v1.13.0 (released 2026-11-02) makes the party table where buffs are chosen: every buff
  anybody can cast has a row, and every recipient a cell to tick, with its state in the
  glyph. Ticking writes into the casting character's preset, and unticking undoes exactly
  that. A filter cycles through all, planned, not chosen and problems, and a character tab
  narrows the table to that recipient. The text-size option now says that *Large* and *Huge*
  move only the title and the button captions
  ([bg-eeex-buffbot#112](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/112), open).
- v1.14.0 (released 2026-11-03) lets a Project Image or a Simulacrum be set up before it is
  conjured
  ([bg-eeex-buffbot#107](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/107)): it
  gets a tab in the Summons view as soon as a preset creates it, showing the book it will
  have, by the engine's own copy rules. What a spell creates is read from the installed
  data. A planned Project Image shares its owner's casts; a Simulacrum has its own.
- v1.15.0 (released 2026-11-03) gives the party table a second order, *List: A-Z* (the
  default) or *List: cast order*, where Up, Down and Sort work
  ([bg-eeex-buffbot#115](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/115)). A
  *Clones: off / image / simulacrum / both* switch in the footer plans casts from a clone
  ([bg-eeex-buffbot#116](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/116)); off is
  the default. The plan now says which casts a planned Project Image takes from its owner,
  and a Project Image's own list arrives switched on only for what comes after its creation.
- v1.16.0 (released 2026-11-03 21:02 UTC, lightweight tag on commit `512a1fa`) lets a summon
  or clone receive buffs
  ([bg-eeex-buffbot#117](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/117)). The
  target picker is one scrolling list without the six-row limit. A cast aimed at a summon
  cannot be interrupted and does not wait for the aura, and an item or a 5E-converted caster
  refuses a summon as target and says so in the log. Repeat and Variant reach the party
  table for every owner of a row, which completes #115.
- Against v1.12.6 the TP2 changes its version, adds a comment, turns the main component's
  `FAIL @112` without LuaJIT into `PRINT @114` and copies the new `BfBotSum.lua`. Every
  `BEGIN`, `DESIGNATED`, `LABEL`, `GAME_IS` and the EEex checks are unchanged, and so are
  the names of components `1` and `0` (`@100`, `@111`) in all languages and their order. One
  file is added (`buffbot/BfBotSum.lua`) and 27 under `buffbot/` change: the Lua code, the
  menu, the nine `setup.tra` files, the TP2 and the frame art `BFBOTFR*.PVRZ`. None is
  removed, and all 229 published files but `setup-buffbot.exe` are byte-identical to the tag
  commit's tree. Nothing outside BuffBot's own installed files changes.
- BuffBot's save data stays at schema 10. Presets gain only the optional `cln` field for the
  *Clones* switch: absent means off, so nothing migrates. No installer choice is new or
  changed. In game the panel opens as before, By caster, with the A-Z list, *Clones* off and
  the same text size, and CEBG writes no BuffBot settings. Fresh-download evidence:
  `manifest/evidence/release-intake-20261104-buffbot-chriz-easy-bg.toml`.
- Owner validation for v1.16.0: 2,122 automated tests and the in-game suite (909 checks, 0
  failures, 31 phases). The notes of v1.15.0 and v1.16.0 end with `Requires EEex v1.3.1 or
  newer`, while its README, TP2 and code still accept older EEex; new installations get EEex
  v1.3.1. The only in-game setup BuffBot's notes name for v1.12.7 to v1.16.0 is BG2:EE 2.7.3.0.
  Not recorded as checked in game: the pressed look under a real mouse, a real game on EEex's
  own Lua without the LuaJIT loader, and BG:EE, SoD, EET and IWD:EE for the new views. BuffBot's
  README still calls the configuration with `430` not accepted.
- Christopher, 2026-11-04: BuffBot waits for 0.1.2, first v1.15.0, then v1.16.0, in both
  installers; 0.1.1 ships the tested v1.12.6. Since 2026-11-08 both installers pin v1.16.1
  and no hold remains: Chriz Easy IWD 0.1.9 ships it, and Chriz Easy BG gets it with 0.1.2.
- v1.16.1 (released 2026-11-08, tag on commit `377bb1d`) keeps scanned 5E Spellcasting rows
  visible when a hand-edited wrapper cannot name a usable cast target or an overlay fails. It
  chooses the table's usable target where it can and records the exceptional route once in the
  diagnostic log instead of printing it in the game. It adds no component, changes no default
  or save schema, and leaves an unchanged 5E install's casting unchanged. Fresh-download
  evidence: `manifest/evidence/release-intake-20261108-buffbot-chriz-easy-bg-chriz-easy-iwd.toml`.
  Its owner records 2,123 offline tests and a BG2:EE 2.7.3.0 / EEex 1.3.1 headless suite with
  909 passing checks; the hand-edited wrapper was not tried in a live game.


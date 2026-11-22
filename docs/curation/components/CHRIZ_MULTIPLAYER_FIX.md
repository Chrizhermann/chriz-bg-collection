# CHRIZ_MULTIPLAYER_FIX — components

Listed at **v0.1.0** (tag commit `6efc210`, reviewed 2026-11-22). Not installed in the August 2026
reference snapshot; added with both components on by default in both Chriz Easy BG and Chriz Easy
IWD (Christopher, 2026-11-22).
2 entries, 0 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Multiplayer lobby crash fix (EEex) | Fixes |  |  | default |
| 1 | Copy saves between single player and multiplayer on the Load screen (EEex) | Interface |  |  | default |

## UI and dependency behavior

- Component `0` stops the random crash right after leaving Beamdog's multiplayer lobby (for
  example Multiplayer > Local network game > Load): two threads tore down the same lobby
  connection. Two EEex hooks keep every lobby task on the lobby's own network thread. It copies
  `override/M_CZMP.lua` and its patterns `EEex_scripts/extra_pattern_dbs/CZMP.db`, but only when
  every pattern occurs exactly once in the game executable InfinityLoader starts: a pattern that
  finds nothing stops the game before its window opens. Otherwise it removes both files and
  installs as a logged no-op, and the game plays as before.
- Component `1` adds a button to the Load screen that copies the save into the other mode's save
  folder (`save` and `mpsave`) as a new save; the original stays. On BG:EE, SoD, BG2:EE and EET
  it is one button at the bottom that acts on the selected save; on IWD:EE it is a small "Copy"
  button on each save row. It copies `override/M_CZMS.lua` and `override/CZMS.menu`; the button
  appears once EEex's LuaJIT component is installed.
- The two components are independent of each other. Both need EEex (`override/M___EEex.lua`,
  component `1`) at install time; without it a component installs as a logged no-op. Neither
  patches a game resource or a save. In Chriz Easy BG they run in `post-eet-end` directly after
  Chriz Overhead Text; in Chriz Easy IWD directly after Chriz Overhead Text in `main`.
- In Chriz Easy BG each component is a default of its own. In Chriz Easy IWD the lobby crash fix
  always comes with the mod, and the save copy button is a default that can be turned off.
  Existing Chriz Easy IWD installations get both through the hotpatch
  `chriz-multiplayer-fix-0.1.0`.

## Release and acceptance follow-up

- Collection intake on 2026-11-22 downloaded the published release afresh and checked its length,
  SHA-256, layout, artifact contracts and native component order in every recipe
  (`manifest/evidence/release-intake-20261122-chriz-multiplayer-fix-chriz-easy-bg-chriz-easy-iwd.toml`).
- Combined full-collection EET and IWD:EE installs and multiplayer sessions with these components
  remain tracked under collection acceptance.

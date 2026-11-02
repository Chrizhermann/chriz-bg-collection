# EEEXREMOTE — components

Listed at installed version **v0.2.0**; the collection pins **v0.2.1** (tag commit
`3d98990`, reviewed 2026-10-23). Component `0` is the same in both.
1 entry, 1 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | EEex Remote Console |  |  | ✓ | optional |

## UI and dependency behavior

- This is an advanced modding/testing tool. Show it unchecked by default and note that
  ordinary players do not need it.
- It requires EEex components `1` (EEex) and `8` (LuaJIT), v1.2 or later (the collection
  pins v1.3.1), and installs after EEex. Its capability checks remain valid:
  `override/M___EEex.lua` is present and LuaJIT selects
  `LuaPatchMode=REPLACE_INTERNAL_WITH_EXTERNAL`.

## Release and acceptance follow-up

- v0.2.1 carries the EEex v1.2 and v1.3 documentation and installer guidance
  ([eeex-remote-console#3](https://github.com/Chrizhermann/eeex-remote-console/pull/3)) and the `START_SP` fix
  ([eeex-remote-console#4](https://github.com/Chrizhermann/eeex-remote-console/pull/4)). Its capability checks are unchanged.
- Live validation: BG2EE 2.6.6 with EEex v1.2.0 (22/22), and on 2026-10-18 EET over
  2.7.3.0 with EEex v1.3.1 in a disposable lab: README checklist steps 1-6, that is the
  ready handshake, `pong` and the 22-case smoke suite with the watchdog on the main menu
  and on the world screen, with the v0.2.0 loader and with the `START_SP` fix. The lab
  ran hidden, so nothing was checked on screen.

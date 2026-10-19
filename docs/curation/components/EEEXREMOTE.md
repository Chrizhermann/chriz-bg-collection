# EEEXREMOTE — components

Listed at installed and target version **v0.2.0**.
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

- The v0.2.0 user documentation still describes EEex 1.0.x. The refresh for EEex v1.2
  and v1.3 is [eeex-remote-console#3](https://github.com/Chrizhermann/eeex-remote-console/pull/3); re-pin
  after it merges (the module code is unchanged).
- Live validation: BG2EE 2.6.6 with EEex v1.2.0 (22/22), and on 2026-10-18 EET over
  2.7.3.0 with EEex v1.3.1 in a disposable lab: ready handshake, `pong` and the 22-case
  smoke suite with the watchdog on the main menu (22/22 with the suite fix from that pull request).
  World-screen polling on the target build is still open before marking this
  combination accepted.

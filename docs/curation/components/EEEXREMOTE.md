# EEEXREMOTE — components

Listed at installed and target version **v0.2.0**.
1 entry, 1 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | EEex Remote Console |  |  | ✓ | optional |

## UI and dependency behavior

- This is an advanced modding/testing tool. Show it unchecked by default and note that
  ordinary players do not need it.
- It requires EEex v1.2 components `1` (EEex) and `8` (LuaJIT), and installs after EEex.
  Its v1.2 capability checks remain valid: `override/M___EEex.lua` is present and LuaJIT
  selects `LuaPatchMode=REPLACE_INTERNAL_WITH_EXTERNAL`.

## Release and acceptance follow-up

- The v0.2.0 user documentation still describes EEex 1.0.x. Refresh that documentation
  in its own repository before the collection release.
- Historical live validation covered BG2EE 2.6.6 with EEex 1.2. Re-run the ready
  handshake, `pong`, menu/world screen, and watchdog smoke tests on the target EET 2.7
  build before marking this combination accepted.

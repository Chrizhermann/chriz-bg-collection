# CHRIZ_OVERHEAD_TEXT — components

Listed at **v0.1.0** (tag commit `2bbd76b`, reviewed 2026-11-16). Not installed in the August 2026
reference snapshot; added as a default quality-of-life option in both Chriz Easy BG and Chriz Easy
IWD (Christopher, 2026-11-16).
1 entry, 0 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Keep overhead text visible with Reveal Details and overhead HP (EEex) | Interface |  |  | default |

## UI and dependency behavior

- Keeps spoken overhead lines (`DisplayStringHead*`, `VerbalConstant`) visible while Tab / Reveal
  Details or `HP Over Head` is active, stacking the spoken box cleanly above the nameplate or HP
  label instead of letting them overwrite `CGameSprite.m_overHeadText` every frame.
- Requires EEex (`override/M___EEex.lua`, component `1`) and installs after EEex on BG:EE (incl.
  SoD), BG2:EE, EET, and IWD:EE (`2.7.3.0`). Copies only `override/M_CZOH.lua` without modifying
  game resources or saves.

## Release and acceptance follow-up

- Owner verification on `v0.1.0` (`Chrizhermann/chriz-overhead-text`): 173 offline checks (WeiDU
  249 install/uninstall/reinstall across synthetic `2.7.3.0` BG:EE/SoD/BG2:EE/EET/IWD:EE layouts,
  LuaJIT hook/layout tests, Keystone x86-64 assembly check) plus a disposable quiet-lab run on
  BG2:EE `2.7.3.0` with EEex `v1.3.1` verifying live in-game screenshots with Reveal Details,
  `HP Over Head`, and natural fallback. Combined full-collection EET and IWD:EE runtime checks
  remain tracked under collection acceptance.

# EEEX — components

The reference install used **v0.11.0-alpha**. This catalog is refreshed to the current
collection target, **v1.3.1** (reviewed 2026-10-18). v1.2's new core plus the eight old
capabilities produce nine components that are all included whenever EEex is included.
v1.3.0 added component `9`, the Attack Info module; its Decision is open, so it is not
installed yet.

10 entries, 8 reference equivalents installed. ✓ = capability present in the reference.
Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Quick Menu Core |  |  |  | mandatory |
| 1 | EEex |  |  | ✓ | mandatory |
| 2 | Enable effect menu module - LShift-on-hover to view spells affecting creature |  |  | ✓ | mandatory |
| 3 | Enable empty container module - Highlight empty containers in gray instead of cyan |  |  | ✓ | mandatory |
| 4 | Enable hotkey module - Edit override/B3Hotkey.lua to create advanced spell hotkeys |  |  | ✓ | mandatory |
| 5 | Enable scale module - Customizable UI scaling factor |  |  | ✓ | mandatory |
| 6 | Enable time step module - Advance 1 game tick on keypress |  |  | ✓ | mandatory |
| 7 | Enable timer module - Visual indicators for modal actions, contingencies, and spell/item cooldowns |  |  | ✓ | mandatory |
| 8 | Experimental - Use LuaJIT (can help stuttering) |  |  | ✓ | mandatory |
| 9 | Enable attack info module - Show to-hit chances when hovering attack cursor over target |  |  |  |  |

## Collection behavior

- This deliberately selects upstream's **Experimental** tier: component `1` requires
  `0`, and components `2` through `9` require `1`. Upstream's Full and Experimental
  tiers include `9` since v1.3.0.
- Component `8` retains upstream's experimental/crash warning, but it is also required
  by EEex Remote Console. Explain that tradeoff in the UI rather than silently omitting it.
- Component `9` only sets `Enable=1` under `[Attack Info Module]` in `EEex.ini`. The
  module's code ships with component `1`, defaults to off, and players can switch it in
  EEex's in-game options (takes effect after a restart). The TP2 declares `9` directly
  after `1`, so a run that adds it must keep that native order (`0, 1, 9, 2…8`).
- Start the game through `InfinityLoader.exe`/`EEex.exe`; do not launch the vanilla game
  executable after EEex is installed.

## Upgrade and acceptance follow-up

- Build v1.2 from a clean staged source. Do not overlay it on the old v0.11 files: v1.2
  moved internal Lua into root `EEex_scripts`, so an overlay can retain stale loader,
  LuaBindings, and `override/EEex_*.lua` files. Third-party `M_*.lua` bootstrap files
  correctly remain in `override`.
- After installation, verify `[EEex] Uncap FPS Limit Enabled`; v1.2 defaults it to `0`
  independently of the numeric FPS limit.
- v1.3.1 against v1.2.0: the TP2 differs only by component `9`. The Lua API removes four
  definitions (`B3EffectMenu_Private_Init`, `B3EffectMenu_Private_Menu_KeybindActive`,
  `EEex_Options_Private_IdToDisplayEntry`, `EEex_Sprite_Hook_CheckBlockWeaponHit`) and
  adds 90, among them `EEex_Keybinds_Reset`, `EEex_Keybinds_IsSatisfied`, the
  `EEex_GameState_*InputMode` functions and new options widgets; the loader pattern
  databases add 31 labels and remove none. `EEex.dll` is rebuilt; Modpack `650` checks
  its PE identity, and since Modpack alpha.11 its main profiles are v1.3.1's DLLs for
  2.6.6.0 and 2.7.3.0, with v1.2.0 accepted only on 2.7.3.0. Evidence:
  `manifest/evidence/release-intake-20261018.toml`.
- Bardic Wonders, Artisan's Kitpack and Bubb's Spell Menu still test
  `MOD_IS_INSTALLED` for EEex component `0`; that is the Quick Menu Core since v1.2, and
  the checks pass because the collection installs it with the rest.
- Smoke-test launch, every enabled module, LuaJIT, and EEex Remote Console on the target
  EET 2.7 build before calling the all-components preset accepted.

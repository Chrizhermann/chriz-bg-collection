# EEEX — components

The reference install used **v0.11.0-alpha**. This catalog is refreshed to the current
collection target, **v1.2.0**. Its new core plus the eight old capabilities produce nine
target components; all nine are included whenever EEex is included.

9 entries, 8 reference equivalents installed. ✓ = capability present in the reference.
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

## Collection behavior

- This deliberately selects upstream's **Experimental** tier: component `1` requires
  `0`, and components `2` through `8` require `1`.
- Component `8` retains upstream's experimental/crash warning, but it is also required
  by EEex Remote Console. Explain that tradeoff in the UI rather than silently omitting it.
- Start the game through `InfinityLoader.exe`/`EEex.exe`; do not launch the vanilla game
  executable after EEex is installed.

## Upgrade and acceptance follow-up

- Build v1.2 from a clean staged source. Do not overlay it on the old v0.11 files: v1.2
  moved internal Lua into root `EEex_scripts`, so an overlay can retain stale loader,
  LuaBindings, and `override/EEex_*.lua` files. Third-party `M_*.lua` bootstrap files
  correctly remain in `override`.
- After installation, verify `[EEex] Uncap FPS Limit Enabled`; v1.2 defaults it to `0`
  independently of the numeric FPS limit.
- Smoke-test launch, every enabled module, LuaJIT, and EEex Remote Console on the target
  EET 2.7 build before calling the all-components preset accepted.

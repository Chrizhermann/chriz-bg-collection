# KLATU — components

Listed at released version **1.7.4**.
1 entry, 1 selected. ✓ = selected for the recommended recipe.

## UI/dependency notes

- Component `2150` is the approved default-checked, optional no-penalty armor-thieving
  QoL. It removes armor-equipped opcode 144 restrictions for stealth and ordinary
  thieving without changing class/kit equipment permissions, spellcasting restrictions,
  skill values, or unrelated kit abilities.
- The immutable `Version-1.7.4` tag resolves to upstream commit
  `18ad2ab183e7dc71bbf481ec6331804ffd111e2f`. The released component passed the
  focused synthetic WeiDU 249 fixture on 2026-09-08,
  including plate, mod-added armor, Artisan-patched armor, and byte-exact uninstall.
  This is not live-game acceptance.
- The component deliberately does not remove distinct Find Traps button-14 restrictions
  or indirect spell/effect restrictions. Install it late, after armor-adding mods and
  Artisan modifications.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 2150 | Allow Thievery in Armor | Convenience Tweaks/Cheats |  | ✓ | default |

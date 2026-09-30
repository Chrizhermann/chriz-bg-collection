# Game interface choices

September 29 source integration for new installations. The recommended interface
and its selected EEex, Bubb's Spell Menu, HGO and BuffBot components stay unchanged.
Every replacement is optional in the exclusive `game-interface` group. Returning
to the default restores requested add-ons only when their requirements still hold.
No third-party UI code, assets or archive is bundled in CEBG.

| Choice | Pinned distribution | Core / English | Current readiness |
| --- | --- | --- | --- |
| Infinity UI++ | [v1.28](https://github.com/Renegade0/InfinityUI/releases/tag/v1.28), commit `f755721ef88f2370f35ee8d342abacf25b48e200` | 0 / 0 | Experimental, community testing |
| EET: SoD style | Existing EET commit `74e91d72bca5d073fa11c1d088b90d7ff0c7105d` | 0 / 0 | Experimental, community testing |
| Dragonspear UI++ | [anongit 22.01.3](https://github.com/anongit/DragonspearUI/tree/22.01.3), commit `d86bdddd0d74ebadc9ebcd40351021d6d1dadbc0` | 1 / 0 | Experimental, with released Modpack 640 helper |
| Lefreut: BG2 / BG1 / SoD | Three distinct [4.9.1 packages](https://github.com/r-e-d/LeUI/releases/tag/v4.9.1) | 0 / 0 | Experimental, with the same helper |

Artifact files in `manifest/artifacts` contain the measured hashes, exact lengths,
immutable source and publication allowlists. Mod files identify the actual TP2,
English index and core component. Optional tooltip, TLK rewrite, portrait backup
and quicksave-count components are not added by choosing an interface.

## Install order and add-ons

The authored order is EET core, EEex, the chosen full interface, its conditional
Modpack 640 compatibility helper, Bubb's Spell Menu,
later patchers including SCS, EET finalization, HGO, and the existing BuffBot tail.
No provider modifies the historical captured install order.

- Infinity's [installer](https://github.com/Renegade0/InfinityUI/blob/f755721ef88f2370f35ee8d342abacf25b48e200/infinity_ui/infinity_ui.tp2)
  reads EEex component 0 at install time; the pinned EEex run includes it.
  Infinity skins are chosen in-game. SCS 4115 is omitted with an explicit reason
  because its five-point thief-skill screen has a reported Infinity startup error.
  No replacement implementation or general SCS incompatibility is claimed.
- Bubb 5.2 explicitly detects
  [Infinity, both Dragonspear forms and LeUI](https://github.com/Bubb13/Bubbs-Spell-Menu-Extended/blob/5512e35b4e3ca474526ae20ea8f5bcc73ed40053/bubb_spell_menu_extended/bubb_spell_menu_extended.tp2).
  Its patch must see the final UI. EEex/BuffBot selection requirements are retained.
- EET GUI component 0 requires EET core, which generates `EET/SoD_GUI` from the
  imported game. The existing EET artifact now publishes `EET_gui` and its TP2.
  Component 1 requires IWD-in-EET and is not offered. The
  [pinned GUI installer](https://github.com/Gibberlings3/EET/blob/74e91d72bca5d073fa11c1d088b90d7ff0c7105d/EET_gui/EET_gui.tp2)
  shares EET's language files.

HGO 5.2 restrictions are component-specific:

| Provider | Omitted currently exposed HGO components | Kept eligible |
| --- | --- | --- |
| Infinity | 29, 30, 300, 301, 302, 304 | Other settings, key bindings, cheat menu 200 |
| LeUI variants | 28, 29, 30, 300, 301, 302, 304 | Other settings, key bindings, cheat menu 200 |
| Dragonspear 22.01.3 | 300, 301, 302, 304 | 28, 29, 30, other settings, key bindings, cheat menu 200 |
| EET SoD | None added | Existing HGO choices |

HGO has dedicated Dragonspear sliders for frame rate and action feedback; do not
inherit Infinity's 29/30 exclusions. Its
[TP2](https://github.com/Argent77/A7-HiddenGameplayOptions/blob/v5.2/HiddenGameplayOptions/HiddenGameplayOptions.tp2)
also excludes 303/306 with full replacements, but those components are not
offered in this batch. 306 is not automatically introduced by making EET GUI
available.

## Released classic-UI compatibility adapter

Game 2.7 renamed the old `t` translation helper to `getUiString`. The
[HGO 5.2 workaround](https://github.com/Argent77/A7-HiddenGameplayOptions/blob/v5.2/HiddenGameplayOptions/HiddenGameplayOptions.tp2#L41-L44)
patches existing `UTIL.LUA` when it defines `getUiString` and lacks the alias.
HGO is optional, so it cannot provide an undeclared UI requirement.

LeUI 4.9.1 copies `UI.menu` with `t(...)` calls; none of its three packages
provides `UTIL.LUA`, a base `t` implementation or `getUiString`. Dragonspear's
[M_dui.lua](https://github.com/anongit/DragonspearUI/blob/d86bdddd0d74ebadc9ebcd40351021d6d1dadbc0/dragonspear_ui%2B%2B/content-common/all/M_dui.lua#L16-L42)
captures the existing `t` in `__t` and wraps/calls it, also without a base helper.
This is a source-proven missing dependency, not an untested gameplay combination.
The independently verified Modpack alpha.8 release now supplies this dependency;
all four choices are experimental/selectable, not silently dependent on HGO.

Released owner solution: Modpack component 640 supplies the missing
legacy helper from the game's existing `getUiString`, preserves a valid existing
`t`, leaves the 2.6 implementation alone and fails clearly if neither exists.
It patches `UTIL.LUA` so the alias exists before `M_dui.lua` loads. The dedicated
run follows the selected full UI and precedes Bubb.
The dedicated `chriz-bg-modpack-ui-compat-bg2` run is a mandatory conditional
collection-root feature: it runs only for the four classic choices, even if HGO,
EEex or optional Modpack gameplay features are disabled. Default, Infinity and
EET SoD choices omit it. Third-party archives remain unmodified in this repository.

The old [Renegade0 v2.42](https://github.com/Renegade0/DragonspearUI/releases/tag/v2.42)
is not the selected Dragonspear distribution. It accepts only BGEE/BG2EE and does
not copy EET resources; WeiDU 249's
[GAME_IS implementation](https://github.com/WeiDUorg/weidu/blob/v249.00/src/tppe.ml#L437-L484)
explicitly excludes `eet.flag` games from both predicates. The anongit tag instead
supports EET and generates its menu using bundled Windows x86/amd64 Lua tools.
Its core is component 1 and it creates `M_BG.lua`, matching Bubb/HGO's older
Dragonspear detection branch. It is separate from Infinity's Dragonspear skin.

## Verification boundary and optional feedback

On September 29, production artifact verification freshly downloaded and verified
Infinity 1.28, all three LeUI packages, Dragonspear 22.01.3, the expanded EET
publication contract and Modpack alpha.8 with helper 640. Hash, extraction limits,
roots and TP2 paths passed. Source inspection confirmed the helper's `UTIL.LUA`
guard; no upstream UI payload was modified or redistributed.
The source-only recipe tests cover provider exclusivity, unchanged defaults,
specific exclusions, remembered preferences, order and automatic helper selection.
This is not a completed EET installation or live-game acceptance.

For community feedback, report the selected interface/version, game build and
any error, with optional checks of start/load, character creation and level-up,
inventory, spellbook, dialogue, options, Bubb's menu and BuffBot. No exhaustive
installation/playthrough matrix or personal test gate is required.

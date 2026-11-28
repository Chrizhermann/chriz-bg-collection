# Chriz Easy IWD 0.1.15

App **0.1.15** bundles collection **0.1.16**. Chris's Recommended includes **145 default
components across 34 install runs and 33 mods**, with **18 recruitable companions**.

## Highlights

- **Baldur's Gate spell rules and fixes.** Chriz IWD Encounters **v0.3.0** (component
  **120**, on by default) applies Baldur's Gate caster-level Dispel/Remove Magic, limits
  Minor Spell Deflection to spell levels 1–4 so Breach and Pierce Magic bypass it, keeps
  Spirit Armor / Mirror Image / Reflected Image from stacking, lets area-of-effect damage
  bypass Mirror Image, adds logical Web/Grease/Stinking Cloud/Spook/Horror immunities,
  and moves utility spells to the Universal school.
- **Main campaign overhaul (Chapters 2–6).** Component **420** (on by default) thins
  repetitive filler mobs, splits Kresselack's crypt into 3 paced waves, updates Terikan
  (level 1–3 spell immunity, immediate minion rally, vanilla respawn timers) plus
  Tarnsen and Ginafae script fixes, upgrades Severed Hand/Giant/Wyrm encounters, and adds
  five *Drasus*/*Iron Throne*-style elite squads.
- **Boss encounter updates.** Components **300**, **310**, and **360** replace floor
  dispel triggers with Belhifet's level-26 Dispel Magic and Yxunomei's level-18 Dispel
  Magic (plus fixing Yxunomei's missing telekinesis delay), and tune Poquelin to 310 HP.
- **Optional expansion overhaul.** Component **500** (optional, off by default under
  **Customize → Chriz IWD Encounters**) curates Heart of Winter and Trials of the
  Luremaster encounters, thins hordes, and repairs `AR9602` Xactile while keeping full
  SCS caster spellbooks.

## Testing and limits

- All five offline test suites (`test_templates`, `test_banter_guard`, and twice-installed
  `test_overwrite` scenarios `base`, `edited`, `embedded`, and `stack`) passed after the
  final edit in Chriz IWD Encounters v0.3.0.
- An IWD:EE 2.7.3.0 quiet lab verified live BG spell/dispel rules, Terikan, `AR3503`
  wave wake timing, and the Chapter 2–6 elite squads (`check_install.py` passed). A full
  app-driven 34-run installation and a full campaign playthrough remain pending.

## Installing and updating

- On app **0.1.7–0.1.14**, choose **Updates → Update Chriz Easy IWD app**.
- On **0.1.1–0.1.6** or for a first-time setup, download and run
  [Chriz.Easy.IWD_0.1.15_x64-setup.exe](https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/v0.1.15/Chriz.Easy.IWD_0.1.15_x64-setup.exe)
  from the [v0.1.15 release page](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.15).
- Updating the app leaves existing games unchanged; choose **New installation** for the
  collection **0.1.16** setup.

Tested with Icewind Dale: Enhanced Edition 2.7.3.0 on 64-bit Windows 10 or 11 (about 8 GB
of free space); other usable Windows copies install as Less tested.

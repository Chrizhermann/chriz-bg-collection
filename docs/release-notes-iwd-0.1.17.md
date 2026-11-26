# Chriz Easy IWD 0.1.17

App **0.1.17** bundles collection **0.1.22**. Chris's Recommended includes **148 default
components across 36 install runs and 35 mods**, with **18 recruitable companions**.

## Highlights

- **Dragon's Eye, Severed Hand & Dorn's Deep quest kill counters fixed (`Chriz IWD Encounters v0.3.4`).**
  Preserves quest Death Variables (`Talonite` in `AR4002` and `ELVEN_SOLDIER` in `AR5403`) on
  the custom squads and aligns thinned-area quest kill thresholds (`> 5` / `< 6` for Mother
  Egenia and the Dragon's Eye Level 2 captives, `> 11` for Kaylessa in the Severed Hand, and
  `> 8` / `> 4` for the Lower Dorn's Deep salamander quests) so clearing an area always completes
  the objective.
  - **Hotpatch for existing `0.1.15` & `0.1.16` games:** Update to app **0.1.17**, open
    **Updates**, and apply **Dragon's Eye, Severed Hand, and Dorn's Deep quest fix** (component
    **425**) — works even on saves already in progress.
- **Encounter pacing, elite stat calibration, loot economy & apex guardians (`Chriz IWD Encounters v0.3.4` & `v0.3.2`).**
  Fixes triple-stacked weapon-mastery AC/THAC0/APR bonuses on custom `CIE4xx`/`CIE5xx` elites,
  marks bespoke elite equipment and AI buff potions undroppable (at most 1 healing potion drop),
  recalibrates the `AR4002` Talonite Coven (`Priestess + Champion + Venom-Troll`), stretches
  Kresselack's antechamber (`AR3503`) wave timers to `48 s` / `+54 s` with sentry isolation and
  callout warnings, eliminates every remaining Bombardier Beetle (`AR4001`, `AR4002`, `AR9801`),
  adds six apex encounters in Chapters 2–5, and trims additional late-chapter and Watchknight
  hordes, on top of `v0.3.2`'s six mixed elite squads, Kaylessa overhaul, Poquelin finale dispel
  (`300`), and Heart of Winter / Trials of the Luremaster (`500`).
- **Multiplayer lobby crash fix & single/multiplayer save copy (`Chriz Multiplayer Fix v0.1.0`).**
  With EEex, eliminates the random crash after leaving Beamdog's multiplayer lobby (`0`) and adds a
  Load-screen **Copy** button to copy saves between single player and multiplayer (`1`, optional
  under **Customize**; both also available via hotpatch for earlier installs).
- **Spell-scroll learning follows Rules as Written (`Chriz BG Modpack v0.2.0-alpha.21`, component `470`).**
  With EEex, stops kitted bards (e.g. Blades), Wild Mages, kitted dual-class mages, and non-mages
  from suffering the `-15%` specialist penalty when scribing scrolls (package pinned cleanly at
  `v0.2.0-alpha.21`; BG-only/Challenge Mode `695`–`698` stay omitted from IWD).

## Testing and limits

- All offline test suites in Chriz IWD Encounters v0.3.4 (`87/87` across `test_templates`,
  `test_banter_guard`, and twice-installed `test_overwrite` on `base`, `edited`, `embedded`, and
  `stack` fixtures) passed, and Christopher's local stream installation was synced and verified on
  `v0.3.4` (`420, 425, 500`) with `check_install.py` (`all checks passed`).
- A full combined app-driven 36-run WeiDU installation and full IWD campaign playthrough remain
  pending for collection 0.1.22.

## Installing and updating

- On app **0.1.7–0.1.16**, choose **Updates → Update Chriz Easy IWD app**. Existing games stay
  unchanged; to apply the quest-counter fix or Multiplayer Fix to an existing **0.1.15–0.1.16**
  game, apply the hotpatch from **Updates** afterward.
- On **0.1.1–0.1.6** or for a first-time setup, download and run
  [Chriz.Easy.IWD_0.1.17_x64-setup.exe](https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/v0.1.17/Chriz.Easy.IWD_0.1.17_x64-setup.exe)
  from the [v0.1.17 release page](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.17).
- Choose **New installation** in app **0.1.17** for the full collection **0.1.22** stack.

Tested with Icewind Dale: Enhanced Edition 2.7.3.0 on 64-bit Windows 10 or 11 (about 8 GB of
free space); other usable Windows copies install as Less tested.

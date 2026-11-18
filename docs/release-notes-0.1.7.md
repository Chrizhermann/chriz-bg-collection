# Chriz Easy BG 0.1.7

App **0.1.7** bundles collection **0.1.4**. Chris's Recommended includes **468 default
components across 70 install runs and 56 mods**. See the
[package acceptance record](package-acceptance-0.1.7-2026-11-18.md) for release verification.

## Highlights

- **Save and rest with passive abilities and songs active.** The Artisan's Kitpack
  **v4.81a-dev.85928f9-chriz.7** fixes unintended blocks from passive abilities such as
  Awareness, Combat Casting and Tracking. Bardic Wonders **v2.9c-balance.9** fixes the same
  problem for bard songs, Inspirations and kit auras. Visible enemies and active combat
  still prevent saving and resting.
- **Round Clock counts time spent travelling and resting.** BG Rebalance **v0.8.12**
  (component **361**) counts travel, rest and scripted advances of the game clock toward
  the wait after casting a spell, using an item, swinging a weapon or trying to hide,
  and toward the next Contingency check. You can click Stealth while paused once Hide
  in Shadows is ready. Round Clock is on by default with EEex under **Spells** in
  **Customize**, in the **Spell balance** group.
- **BuffBot v1.16.2 updates the Italian translation.** Get it for an existing game from
  **Updates → BuffBot**.

## Known issues

- The mod authors' automated checks for the save and rest fixes passed, including
  154 offline and WeiDU tests for Artisan's Kitpack and 211 automated tests plus
  WeiDU checks for Bardic Wonders. Live gameplay testing of those fixes is pending.
- A BG2:EE 2.7.3.0 quiet-lab test covered Round Clock during world-map travel,
  completed and interrupted rests, and a Stealth click while paused. A full combined
  EET installation for this release is still pending.
- The setup carries a Tauri updater signature. Windows may show an unknown-publisher
  warning because the setup has no Windows Authenticode signature.

## Installing and updating

- For a first-time setup, download
  [Chriz.Easy.BG_0.1.7_x64-setup.exe](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/v0.1.7/Chriz.Easy.BG_0.1.7_x64-setup.exe)
  from the [v0.1.7 release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.7).
  Run the setup and choose **New installation**.
- To update the app, choose **Updates → Update CEBG app**. For the save and rest fixes
  and Round Clock, choose **Create updated installation** on the Updates page or
  **New installation**. Both create a separate installation for your next playthrough.
- For help, see the [troubleshooting and backup guide](troubleshooting.md).

# Chriz Easy BG 0.1.7

**Prepared for release** — app 0.1.7 / collection 0.1.4. Chris's Recommended includes
**468 default components across 70 runs and 56 mods**.

## Highlights

- **Round Clock** counts time spent travelling, resting or advancing the clock in a
  script toward the wait after casting a spell, using an item, finishing a weapon swing
  or trying to hide. It also counts toward the next contingency check. When Hide in
  Shadows is ready, you can click **Stealth** while the game is still paused.
  BG Rebalance v0.8.12 component 361 is on by default with EEex.
- **Saving and resting with passive kit abilities and bard songs.** On a new installation,
  you can save or rest while Awareness, Combat Casting or Tracking is active, or while a
  bard song, Inspiration or kit aura is running. Visible enemies and active combat can
  still prevent either action. These fixes come from The Artisan's Kitpack chriz.7 and
  Bardic Wonders balance.9.
- **BuffBot v1.16.2** updates 72 of the 252 entries in its Italian catalog. Spell names
  and option labels now use consistent terminology throughout the panel.

## Known issues

- A full multi-hour EET installation with all 468 recommended components and these updates
  has not yet been run. The Round Clock single-player BG2:EE 2.7.3.0 lab covered
  world-map travel, completed and interrupted rest, and using Stealth while paused.
  A standalone IWD:EE lab also verified paused Stealth. Travel interrupted by an ambush
  and multiplayer still await an in-game check.
- The Artisan's Kitpack save and rest fixes passed synthetic BG2:EE and EET installer
  tests. Bardic Wonders passed fixtures for BG2:EE, EET and IWD:EE. Both fixes still
  await in-game playtesting. BuffBot's Italian catalog was checked in a running
  BG2:EE 2.7.3.0 game with EEex v1.3.1.
- Windows may show an unknown-publisher warning for the installer. Use the official
  release download and keep Defender enabled.

## Installing and updating

- For a first installation, run `Chriz.Easy.BG_0.1.7_x64-setup.exe` and choose
  **New installation**. [Official release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.7)
- If you already use CEBG, open **Updates** and choose **Update CEBG app** to update
  the installer and launcher.
- For Round Clock and the save and rest fixes, choose **Create updated installation**
  on the Updates page or start a **New installation** for your next playthrough.
  For the Italian catalog in a game you are already playing, close the game, open
  **Updates**, select your installation, choose **Check for a BuffBot update**, and then choose **Update BuffBot**.
- For help with an installation or a backup, see the [troubleshooting and backup guide](troubleshooting.md).

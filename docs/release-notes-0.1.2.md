# Chriz Easy BG 0.1.2

**Released:** app 0.1.2 / collection 0.1.2, on 2026-11-10. The exact package and its test
results are in the [package verification record](package-acceptance-0.1.2-2026-11-10.md).

0.1.2 asks you what to do when a single part of a mod fails to install, brings BuffBot's
newest release to new installations and to games in progress, and lets our own mods write
their result whatever the game files hold. Chris's Recommended rolls hit points again, and
a new installation without Spell Revisions no longer stops at BG Rebalance. These notes
cover everything since 0.1.1.

Updating CEBG leaves your current games and saves unchanged, and the installer changes
help every new installation right away. The collection changes come with a new
installation; BuffBot can also be updated in a game you are playing. See
[Installing and updating](#installing-and-updating).

## Highlights

- **The installer asks when one part fails.** When the installer of a single component
  stops with an error, the installation pauses and names the mod and the part. Choose
  **Leave it out and continue**, **Try again** or **Stop the installation**. Only that
  part leaves, together with what needs it; the mod's other parts stay, and the end of the
  installation names everything left out. **Try again** is offered when nothing was
  installed after the failed part. A part whose loss would leave only half of a mod's core
  in the game, such as EEex's runtime, can only be tried again or stopped.
- **BuffBot 1.16.1** (was 1.12.6). Its panel can show the whole party as a table with one
  row per buff, where you choose who gets which buff, set the cast order and see what each
  character receives. You can set up a Project Image or Simulacrum before you conjure it,
  and summons can receive buffs, such as Stoneskin on a Planetar.
- **Update BuffBot in a game you are playing.** **Updates** has a **BuffBot** section:
  check for a newer release, update with a backup of the game folder, and **Undo** puts the
  game files back exactly. Your saves, BuffBot's settings, exported presets and mods you
  added stay as they are.
- **Our mods write their decided result.** BG Rebalance, Chriz BG Modpack, SoD Remix and
  the voicepacks no longer stop, warn or skip a component because another mod or a hand
  edit changed the game files before them. They write the result we decided, and their
  components, order and Chris's Recommended selection stay the same.
- **Rolled hit points in Chris's Recommended.** Hit points on level-up are rolled again,
  and creatures no longer all get maximum hit points. **Higher HP on Level Up (Maximum)**
  and **Maximum HP Creatures** are unticked options in **Customize**; **Max HP at Level
  One** stays on.

## Important fixes

- **No stop at BG Rebalance without Spell Revisions.** This was the known issue of 0.1.1.
  Flail of the Ages balance now recognises the Haste and Slow spells SCS uses without Spell
  Revisions, so you no longer need to keep Spell Revisions ticked or untick the flail. A
  target that makes its save against the flail's Slow keeps its Haste; a hasted target
  that fails it loses its Haste instead of being slowed.
- **Changed and unavailable mod downloads no longer stop the installation.** A newer file
  from a mod's official link is installed, and a mod whose download cannot be used is left
  out together with the mods that need it. The end of the installation names both. A
  changed installer tool still asks first.
- **A finished mod is no longer failed afterwards** because a game file looks different
  from what CEBG expected.
- **Microsoft Store copies are read instead of turned away.** CEBG tries the folder itself
  and names Windows' protection only when it really cannot read it.
- **App updates replace the bundled collection cleanly.** The 0.1.2 setup first removes
  the collection files of the version it replaces, so an old mod record can no longer stay
  behind next to its newer one and stop the app.

## Everything else

### Collection

- **The Flail of the Ages +5** lets Potion of Haste and Haste rings work, not only the
  Haste and Improved Haste spells. It still gives Free Action and immunity to Slow, Hold,
  Web and Entangle. (BG Rebalance v0.8.10)
- **BuffBot** also brings five text sizes, a readable parchment colour scheme again, a
  Clones switch in the table, and keeps 5E spells in the table when a hand-edited spell
  has no usable target.
- **Chriz BG Modpack's XP table** and BG Rebalance's bard spell-level tables are now found
  by their exact file names, also by case-sensitive installers.
- **Voicepacks** write their player soundset registrations whatever the game's soundset
  list holds; the audio is unchanged.

### Mod versions

- BuffBot 1.16.1 (was 1.12.6)
- BG Rebalance 0.8.11 (was 0.8.9)
- Chriz BG Modpack 0.2.0-alpha.16 (was alpha.15)
- SoD Remix 0.6.17 (was 0.6.16)
- ChrizFader's voicepacks 1.2.0 (was 1.1.0)

## Known issues

- The changed mod content, including BuffBot's new party table, clones and summons, has
  not been played in a game yet. A full test installation of Chris's Recommended passed,
  and the game started with BuffBot 1.16.1 loaded.
- No GOG, Beamdog or Microsoft Store copy has been tested in a full installation yet.
- The Updates page does not list the collection's changes yet; these notes describe them.
- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.2_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.2), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- **BuffBot in a game you are playing:** after the app update, choose that installation on
  the **Updates** page, close the game and use **Check for a BuffBot update**.
- **The collection changes** come with a new installation: **Create updated installation**
  on the Updates page, or **New installation**. Your current installation keeps its mods.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

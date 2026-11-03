# Chriz Easy BG 0.1.1

**Published:** app 0.1.1 / collection 0.1.1, on 2026-11-03.
[Download and release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.1).
Exact package and test results: [0.1.1 package verification](package-acceptance-0.1.1-2026-11-03.md).

0.1.1 makes the installer ask instead of stopping, opens CEBG to GOG, Beamdog and other
Windows copies of the games, and adds a button for your saves and portraits. The
recommended mod setup changes in two places: BG1 NPC Project is optional, and Marek
carries the Boomerang Dagger +2. These notes cover everything since 0.1.0.

Updating CEBG leaves your current games and saves unchanged, and the installer changes
help every new installation right away. The collection changes come with a new
installation. See [Installing and updating](#installing-and-updating).

## Highlights

- **GOG, Beamdog and other Windows copies work.** CEBG now finds GOG copies (GOG Galaxy
  and the offline installer) and Beamdog copies, and you can pick any other folder that
  holds the games. Steam's version 2.7.3.0 in English stays the tested setup; every other
  usable copy installs as **Less tested**, and its details say how it differs.
- **The installer asks instead of stopping.** When a mod's download has changed since we
  reviewed it, or cannot be used at all, CEBG asks what to do: install the newer file,
  leave the mod out together with the mods that need it, or stop. If a mod's installer
  fails and the collection can do without it, CEBG leaves out what did not install and
  carries on. A game that looks modded already gets one question before anything is
  copied.
- **Saves & portraits folder.** Each installation keeps its saves and portraits in its own
  folder in your Documents. **My installs** now has a button that opens it, and creates
  the `portraits` folder the game never creates itself. The folder's path is shown under
  **Installation details**, and a tip on first play says where custom portraits go.
- **Marek carries the Boomerang Dagger +2.** The Iron Throne assassin of chapter 5 has the
  dagger in his backpack and drops it when he dies, at his first meeting with you in the
  north-east of Baldur's Gate or later at the Blushing Mermaid. Without BG1 NPC Project
  it was missing from BG1 and Siege of Dragonspear.
- **BG1 NPC Project is optional.** It is no longer installed by default; tick it in
  **Customize** to get the same banters, quests, interjections and conveniences as
  before. With it ticked, Kivan's quest gives the Boomerang Dagger and Marek carries none.

## Important fixes

These installer changes work as soon as you update CEBG.

- **Leftover files no longer block your game.** A `WeiDU.log`, `setup-` files or mod
  folders in the game folder used to mark the game as modified. CEBG now leaves them out
  of its copy and names them; the game stays the tested setup.
- **Other game versions and languages install with a warning.** A version other than
  2.7.3.0, two games of different versions, or a game set to another language than English
  used to stop the installation or go unchecked; CEBG now says what differs and continues.
- **One unreadable game record no longer hides both games.** Each game copy is checked on
  its own; a folder CEBG cannot read is listed with the reason.
- **Copying the games waits for virus scanners.** A scanner that briefly held a game file
  used to stop the copy; CEBG now tries again for a few seconds, as it already did for
  downloads.
- **Installation names are cleaned instead of refused.** Characters Windows does not allow
  in folder names become `-`, a blank name becomes the default, a long name is shortened,
  and a name that is taken gets a number. The install screen shows the name it will save.
- **A folder that already has files can be the destination.** CEBG installs into a new
  `Chriz Easy BG` folder inside it instead of refusing it.
- **Disk space stops the installation only when it is certainly too small.** Between the
  space the games and downloads certainly need and the generous estimate, CEBG warns in
  the log and continues.
- **A newer manual download is accepted.** For mods you download yourself, such as
  Evandra, a newer file than the reviewed one is installed as you chose it and noted in
  the log.
- **Radar works in every folder.** Installations under `C:\Games` no longer lose Radar.

## Everything else

### Installer and app

- **Download questions:** a question names the mod, says what changed and lists the mods
  that depend on it. **Decide later** leaves the installation waiting, and its card offers
  **Answer the question**. Parts the collection cannot do without, such as EET and DLC
  Merger, and the installer's own tools are never left out; if their download cannot be
  used, the installation stops and says so.
- **Changed downloads:** a newer file must still be the kind of archive the collection
  expects and pass the same safety checks, and a page served instead of a file is asked
  for again. What was installed is recorded under its own checksum, and the end summary
  names every changed or left-out download.
- **Failed installers:** when a mod's installer stops with an error but provably changed
  nothing, the parts it did not install are left out, together with what needs them; the
  parts installed before the error stay. Failures whose effect is unclear still stop.
- **A game that looks modded already** (changed core files or files in `override`) gets
  one question: **No** stops before anything is copied, **Yes** continues and is recorded.
- **Microsoft Store copies** are named plainly: Windows protects their folder, so use a
  Steam or GOG copy.
- **Beamdog copies** already contain Siege of Dragonspear; CEBG leaves out DLC Merger's
  merge for them.
- **Game updates** and the BuffBot update are not offered to installations that left a
  mod out.

### Collection

- **BG1 NPC Project** is optional and off in Chris's Recommended. Ticking it selects the
  same components as before; the Kivan quest guard that protects its dialogue from SCS
  comes with it.
- **Marek's Boomerang Dagger** (Chriz BG Modpack `670`) is on in Chris's Recommended. Marek
  appears twice but can die only once, so one dagger drops. It is unidentified and not
  equipped; Marek keeps fighting with his usual weapons. When BG1 NPC Project's Kivan
  quest already gives the dagger, nothing changes.

### Mod versions

- Chriz BG Modpack 0.2.0-alpha.15 (was alpha.14)

## Known issues

- **Without Spell Revisions, a new installation stops at BG Rebalance.** Its Flail of the
  Ages component does not recognize the Slow and Haste spells SCS writes when Spell
  Revisions is missing. Keep Spell Revisions ticked in **Customize** until 0.1.2, which
  fixes this, or untick **Flail of the Ages balance** there. The recommended setup and every
  installation with Spell Revisions are not affected.
- No GOG, Beamdog or Microsoft Store copy has been tested in a full installation yet; the
  checks follow the stores' documented layouts.
- Marek's two fights and the dagger drop have not been played in a game yet; the
  component passed automated tests and installs on copies of BG:EE and EET game files.
- The Updates page does not list the collection's changes yet; these notes describe them.
- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.1_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.1), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- **The collection changes** come with a new installation: **Create updated installation**
  on the Updates page, or **New installation**. Your current installation keeps its mods,
  BG1 NPC Project included.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

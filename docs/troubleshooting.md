# Chriz Easy BG: quick help

CEBG is still in alpha. If something fails, you usually do not need to delete
your games or downloads to find out why.

## Which update do I need?

| Action | What changes |
| --- | --- |
| Update CEBG | The installer and launcher, not your modded game. |
| Apply a supported game fix or BuffBot update | Only the named fix/mod in the selected installation, after confirmation. Availability depends on that installation. |
| Create an updated installation | A new game folder using the newer collection. Your existing copy stays unchanged. |

The supported game-update features are in the alpha.19 candidate, not promised
in every older public installer. They are not an **Update all mods** button.
Check the installer version and release notes if an action is missing.

## CEBG disappears and Windows Security reports a threat

Check Windows Security's **Protection history** for the detection name and
affected file. Report those alongside the CEBG version and what you were doing.
Don't disable protection, add broad exclusions, or repeatedly restore/reinstall
the flagged build. We need to investigate the exact detection first.

On September 30, Defender quarantined the alpha.19 candidate as
`Behavior:Win32/DefenseEvasion.A!ml`. The cause is under investigation; it has
not been confirmed as a false positive. That incident removed the launcher, but
the separately installed game and Radar files remained present. Do not delete
your game or saves just because the launcher is missing.

## The game starts without Radar

Radar is stored in **BG Radar Overlay** inside the installation's `game` folder,
not directly beside `InfinityLoader.exe`.

The next corrected build adds **Start Radar with the game** beside Play. It is
off initially and remembers your choice for each installation. Windows may ask
for administrator approval for Radar; cancelling starts the game without it.
If Radar cannot start, the corrected build shows a warning and optional details.
This does not require running the whole CEBG installer as administrator.

Older alpha.19 builds can silently fail to start Radar because they do not
request its required Windows approval. The source correction is not yet a
published or live-tested fix.

## The installer cannot find or create a folder

- Check the exact path in the error. The clean game sources and the destination
  for the modded copy are different folders; the destination must not be your
  Steam game folder or contain your existing game.
- CEBG and your source games do not have to be on the same drive. Make sure every
  selected drive is connected and accessible.
- If the destination is missing or cannot be created, use the folder/location
  controls to choose a new, empty destination that your Windows account can write
  to. For example, a new folder inside your own `Games` folder.
- If Documents has moved or is redirected, confirm its actual location in File
  Explorer. Do not create a guessed replacement or move your saves just because
  an error mentions Documents. Send the exact error if it persists.
- For an access-denied error, check Windows Security's protection history. Do not
  disable antivirus or SmartScreen, or run everything as administrator, as a
  general workaround. A confirmed blocked file needs a specific check.

Before installation starts, an alternate empty destination is fine. After a build
has started, do not move its folder to try to repair it.

## An installation stops

Open the technical log or error details and use **Export diagnostics**. Keep the
failed installation unchanged until you know whether it can be recovered.

- If CEBG offers **Retry failed step**, resolve the reported problem, then retry.
- If it says the copy cannot safely resume, there is no general Continue button.
  Export diagnostics before deciding on a supervised repair or a new installation.
- If an update needs recovery, keep the game closed and use the offered **Restore**
  action. If Restore fails or is unavailable, keep its backup and diagnostics and
  ask for help; do not manually merge files from several backups.

Deleting `WeiDU.log`, the receipt or the cache does not repair a failed mod step
and can remove useful evidence. A component missing from the log tells us where
to look, not why it failed.

CEBG checks every reused download and unpacked mod copy before using it. If one
is damaged, for example after a manual deletion that Windows could not finish,
CEBG sets it aside and downloads or unpacks it again, then records that in the
installation's evidence. You never need to delete cache folders by hand. If a
file is only unreadable for the moment, for example while another program scans
or holds it, CEBG deletes nothing and stops the step instead; retry once that
program is done.

## An installation appears missing or incomplete

First check that you selected the right installation and that its drive is
available. Look at its folder under installation details.

If the folder exists and previously worked, export diagnostics. The app can have
a registration problem even when the game itself is intact; don't rebuild it
solely because the label says incomplete. Include a screenshot of the selected
installation and its status.

## An update seems busy for a long time

Keep CEBG open and the game closed while a game update or recovery runs. A full
game backup may take much longer than the mod update itself, especially on a slow
drive. Older builds provide little feedback during that stage.

Check the available activity details. If there is an error or you suspect it has
stalled, report the last visible stage and how long it has been there. Silence by
itself is not proof that a mod has failed; don't start a second installer against
the same folder.

## Backups and Undo

Supported game fixes and BuffBot updates always back up their affected files.
The recommended full-game backup is an additional copy of the playable game
folder. **Saved games are a separate optional backup** because their profile may
live outside that folder. Review the space estimate and backup location before
applying.

**Undo** returns the affected game files to the previous state. It does not roll
back your saved progress, restore an earlier save or update every mod. For BuffBot,
**Restored BuffBot 1.8.3-alpha** means the 1.8.4 update was reversed; offering 1.8.4
again afterwards is expected. Keep backups until you are happy with the result.

## Removing a test or failed installation

Use **Delete / remove installation** and check the exact folder in the confirmation.
Deleting an installation is permanent, including anything you manually put inside
that folder. Original source games and shared downloads stay untouched; saves
outside the deleted folder are kept. Export diagnostics first if you need its logs.

If the folder is already missing, **Forget installation** removes only its entry
from the list. That is not a deletion of files somewhere else on the disk.

If the folder still exists but CEBG can no longer confirm it is that installation,
for example because parts of it were deleted or renamed by hand, CEBG never
deletes anything in it. **Remove from list** then removes only the entry and shows
why. Delete the folder yourself afterwards if you are sure you no longer need it.

## The game crashes or freezes

The next build adds a **Troubleshooting** section at the end of each installation
on the home screen. It stays closed and changes nothing until you open it. There,
**Write a game log for crash reports** makes the game write `InfinityLoader.log`
in the installation's `game` folder from its next start. It is off initially,
works for existing installations without reinstalling, and turns off the same way.
Close the game before changing it; CEBG changes only the `LogFile` setting in the
`game` folder's `InfinityLoader.ini`.

The game starts a new log every time it starts. When you press **Play**, CEBG
first keeps the log of the previous session as `InfinityLoader-previous.log`, so
it survives one restart after a crash or freeze. Starting `InfinityLoader.exe`
directly skips that copy.

When the game crashes, it saves a report in the `InfinityLoader_Crash` folder
inside the `game` folder: two dumps, `_small.dmp` and `_big.dmp`, and a `.log`
file, numbered per crash. With the game log on, that `.log` file is a copy of the
crashed session's log, even after a start outside CEBG; without it, there is at
most a short Lua traceback. The Troubleshooting section shows where these files
are and how many crash reports there are.

## What to send when asking for help

- Your CEBG version and the step/action that failed.
- The exact error or a screenshot, plus the selected installation name.
- The diagnostics export, if available.
- Whether the destination exists and which drives hold the clean games and the
  modded copy. Mention manual changes to the modded installation.
- For a game crash: that crash's `.log` and `_small.dmp` files from
  `InfinityLoader_Crash`. For a freeze, send `InfinityLoader.log`, or
  `InfinityLoader-previous.log` if you have started the game again since. The
  `_big.dmp` files take gigabytes; send one only when asked.

Exports stay local until you share them. Review the ZIP first; do not include
passwords, a whole game folder or saved games unless specifically needed.
Use the support link on the [collection page](https://bg.chrizfader.org/collection).

# BuffBot updater for existing games (Chriz Easy IWD first, then Chriz Easy BG)

Status: design, 2026-11-04. Christopher asked for it on 2026-11-04: update BuffBot in an existing
game to the newest release from either installer, without breaking later hotpatches.

## Why the existing paths do not fit

- The IWD hotpatch only appends new components of one pinned release. `WeiDU.log` may only grow
  by its rows, and it refuses any installation with a recorded whole-mod update.
- The BG route (`engine/src/mod_upgrades/`) knows exactly one transition, 1.8.3-alpha → 1.8.4-alpha,
  English only, BuffBot last.
- Updating by hand changes the mod list, so the IWD hotpatch rule refuses the game afterwards.

## Flow

Updates → BuffBot: Check shows the installed version (from `WeiDU.log`) and the update. Apply,
Undo and Restore work as for hotpatches.

1. **Check (read only).** Completed installation, verified receipt, BuffBot rows present in
   `WeiDU.log`, no unfinished transaction. The release is downloaded into the cache only.
2. **Prepare.** The durable record is written before any game write: the whole game folder
   (names, lengths, SHA-256), copies of the root files WeiDU writes, of the old `buffbot/` folder,
   of WeiDU's BuffBot backup folder, and of every `override/` file BuffBot's uninstall lists
   name. Optional full-folder backup (on by default) and saves backup (off).
3. **Apply.** The new release replaces `buffbot/` (files the new release no longer has are
   removed). WeiDU re-installs BuffBot's installed components in their log order
   (`--force-install-list`), in the installed language, mapped by its folder name in the new
   installer, not by number. The run has no `--safe-exit`: with it, WeiDU 249 skips a component
   that is already installed (bg-modding `weidu-install-runs.md`, lab 2026-10-20).
4. **Verify.** Exit code 0 (3 = with warnings), the same `WeiDU.log` rows in the same order,
   BuffBot's rows now carry the new version, every file that was a plain copy of the old release
   is now the new release's file (WeiDU can report success and leave the old files), no new
   "Recently Uninstalled" line, and nothing outside the update's area changed. Otherwise the game
   files are put back automatically.

The engine is `engine/src/hotpatch/update.rs`; the catalog lists the mods it updates
(`[[updates]]` in `hotpatches/iwd/catalog.toml`). For labs:
`chriz-bg-install hotpatch update-check|update-apply --mod buffbot <managed root>`, and
`hotpatch undo|restore` as for hotpatches.

## Records, Undo, Restore

BuffBot updates are transactions in the same history as hotpatches
(`.chriz/hotpatches/v1/<id>/`, new field `update`), so there is one chain:

- **Undo** always takes back the latest change, hotpatch or BuffBot update, byte for byte, and
  can go back one change at a time.
- **Restore** after a crash, as for hotpatches; the game cannot be started from the app first.
- Fixed on the way: Undo of an earlier transaction after a later one was undone broke the
  history's chain, so the history became unreadable and the app refused to start the game. It
  was unreachable while there was at most one hotpatch, and reachable as soon as an update and a
  hotpatch (or two updates) are recorded. A transaction now chains to the applied record of the
  one before it when that one was undone after it, and the expected game folder is what the
  latest applied transaction left (or, when all were undone, what the first one started from).
- Saves, the profile (BuffBot's `[BuffBot]` settings in `Baldur.lua`), the per-character
  BuffBot setup stored in saves, exported presets in `override/bfbot_presets/`, and the
  original installation record are never written.

## Hotpatches and updates together

- The hotpatch rule takes the game folder as the last transaction left it, so a recorded
  BuffBot update is a known state and later hotpatches stay possible.
- Hotpatch lists ("applied", "next") count hotpatch transactions only.
- A BuffBot update on a folder that was changed by hand is allowed, but it is marked; hotpatches
  stay refused for that installation, because the record must not adopt hand changes.

## Mods after BuffBot

WeiDU takes every component after BuffBot off and puts it back. That is allowed only for rows
the installer or a hotpatch put there whose components install without questions (for example
the EEex Remote Console, hotpatch components). Their re-installation must give the same files;
otherwise everything is put back. Mods added by hand after BuffBot are refused, with their names:
WeiDU would have to re-run installers the app does not know.

## Variation handled

- Any installed BuffBot version, components 0 and 1 in either order, or only one of them.
- Any BuffBot language and game language (`weidu.conf`); a female `dialogF.tlk`.
- BuffBot not installed: nothing is offered ("This update does not add BuffBot").
- Already newest: "BuffBot is up to date".

## Open decision (Christopher)

Where the update comes from: the newest GitHub release at update time (like Radar's Latest,
checked against the size and SHA-256 GitHub publishes), or the newest BuffBot the installer's
recipe pins (tested, but a new installer release per BuffBot version).

## Chriz Easy BG

Same engine module. Chriz Easy BG installations have no finished-folder record, so the record
starts at the first update; hotpatches do not exist there. The 1.8.3 → 1.8.4 route is replaced.
In Chriz Easy BG, BuffBot is followed by the Modpack's scroll casting run, which WeiDU takes off
and puts back. Needs the hotpatch history and `game_files.rs` on the Chriz Easy BG line first.
Comes after Chriz Easy IWD, for CEBG 0.1.2.

## Lab check (2026-11-04)

A fresh installation of Chriz Easy IWD 0.1.5 (the recommended setup plus the EEex Remote Console,
so BuffBot 1.12.6 is followed by one more mod), from the Steam IWD:EE 2.7.3.0, in its own
application data folder; driven through `chriz-bg-install hotpatch`. The game folder (42,820
files, 4.6 GB) was compared by a separate script after every step. A file in
`override/bfbot_presets/` stood in for a player's exported preset. The game was not started.

- Freshly installed, the hotpatch check offered nothing and refused nothing ("Allowed difference
  … EEex Remote Console added"). The update check offered 1.12.6 → 1.16.0 (GitHub's Latest), with
  the Remote Console to be taken off and put back.
- The first apply (with the whole-folder backup) was refused by the result check, which put the
  game back byte for byte: WeiDU had worked, but it notes the old installation of every
  component it installs again as `// Recently Uninstalled:` lines, which the first version of the
  check took for a component left uninstalled. Such lines are now accepted for components that
  are active again.
- The second apply took 30 seconds. Changed: BuffBot's folder, 19 of its `override/` files (one new),
  its uninstall list and `WeiDU.log`, nothing else. The Remote Console came back byte for byte.
  The downloaded ZIP has the SHA-256 GitHub publishes; all 214 BuffBot files in `override/` equal
  the release's. `WeiDU.log` has the same rows in the same order, BuffBot's now naming v1.16.0.
  The exported preset and the profile stayed unchanged.
- After the update the hotpatch check still refused nothing, and offered Undo of "BuffBot 1.16.0".
- Undo took 18 seconds and gave the installed folder back byte for byte; the update was offered
  again. Applying it again gave exactly the first result.
- Not covered: starting the game with the updated BuffBot, a real hotpatch after an update (the
  synthetic tests cover the records), another BuffBot language, and Chriz Easy BG.

# BuffBot updater for existing games (Chriz Easy IWD first, then Chriz Easy BG)

Status: built for Chriz Easy IWD, 2026-11-04, lab check 2026-11-05. Christopher asked for it on
2026-11-04: update BuffBot in an existing game to the newest release from either installer, without
breaking later hotpatches.

## Why the existing paths do not fit

- The IWD hotpatch only appends new components of one pinned release; it cannot replace an
  installed mod.
- The BG route (`engine/src/mod_upgrades/`) knows exactly one transition, 1.8.3-alpha → 1.8.4-alpha,
  English only, BuffBot last.
- Updating by hand leaves no record: no Undo, no Restore, and the app does not know the version.

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
   removed). For the run `WeiDU.log` ends with BuffBot (see "Mods after BuffBot"). WeiDU
   re-installs BuffBot's installed components in their log order (`--force-install-list`), in the
   installed language, mapped by its folder name in the new installer, not by number. The run has
   no `--safe-exit`: with it, WeiDU 249 skips a component that is already installed (bg-modding
   `weidu-install-runs.md`, lab 2026-10-20).
4. **Verify.** Exit code 0 (3 = with warnings); the rows WeiDU saw are all there, in order, with
   the new version on BuffBot's; `// Recently Uninstalled:` lines only for components that are
   active again. Then the whole mod list comes back: the list from before the run with BuffBot's
   rows replaced by the ones WeiDU wrote, every other line byte for byte. Every file that was a
   plain copy of the old release must now be the new release's file (WeiDU can report success and
   leave the old files); in `override/` nothing but BuffBot's own files may have changed, and
   nothing outside the update's area. Otherwise the game files are put back automatically.

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
- What a player changed by hand before an update stays through it and its Undo, and does not
  stop either: as for hotpatches since 2026-11-04, the record holds the folder as it was right
  before. Files changed after an update stop its Undo, as for hotpatches, because the folder could
  no longer be put back byte for byte. The message names them.

## Mods after BuffBot

Left to itself, WeiDU takes every component after BuffBot off and puts it back. In Christopher's
stream game that would be 88 rows: voice packs, Rondak, The Hound and the twelve boss fights. The
first version of this update allowed that for rows the app knows and refused mods added by hand,
the strictness Christopher rejected for hotpatches the same day ("viel zu streng").

So for the run `WeiDU.log` ends with BuffBot: the rows after its first one that are not its own
are left out, and WeiDU sees BuffBot as the last mod. It takes off only the old version, by
BuffBot's own uninstall lists, and installs the new one. No other installer runs, so nothing is
refused and no other mod's files change. Afterwards the rows come back byte for byte, in their
own encoding. The record holds the whole list, so an interrupted run is put back with it.

This keeps WeiDU's own records of BuffBot true, unlike replacing the files directly as the
one-off script for the stream game did (2026-11-04). It relies on one thing: taking BuffBot off
must not undo another mod's work. No mod we know of builds on BuffBot's files.

## Variation handled

- Any installed BuffBot version, components 0 and 1 in either order, or only one of them.
- Any BuffBot language and game language (`weidu.conf`); a female `dialogF.tlk`.
- Any mods after BuffBot, from the installer, a hotpatch or the player.
- BuffBot not installed: nothing is offered ("This update does not add it").
- Already newest: "That is the newest release".
- Installations with mods added at an installation pause keep the rule the app promises for them:
  hotpatches and mod update checks are off, so the card is not shown. The engine does not need
  this rule for the update.

## Where the update comes from

Christopher, 2026-11-04: the newest GitHub release when the player checks, like Radar's Latest,
checked against the length and SHA-256 GitHub publishes. Fresh installations keep their exact pin
(`docs/own-mod-pins.md`), so a release is not tested with the installer before it reaches a game;
Undo is the way back. `artifact = …` in the catalog would pin one release instead.

## Chriz Easy BG

Same engine module. Chriz Easy BG installations have no finished-folder record, so the record
starts at the first update; hotpatches do not exist there. The 1.8.3 → 1.8.4 route is replaced.
Needs the hotpatch history and `game_files.rs` on `main` first. Comes after Chriz Easy IWD, for
CEBG 0.1.2.

## Lab check (2026-11-05)

The final version, on the lab installation of 2026-11-04 (Chriz Easy IWD 0.1.5 from the Steam
IWD:EE 2.7.3.0, in its own application data folder), put back to its installed state. Behind
BuffBot 1.12.6 stood the EEex Remote Console and a small WeiDU mod added by hand, its name holding
a Latin-1 byte. Driven through `chriz-bg-install hotpatch`. A separate script compared the game
folder (42,840 files, 4.6 GB) after every step; a file in `override/bfbot_presets/` stood in for a
player's exported preset.

- **Before the update:** the update check offered 1.12.6 → 1.16.0 (GitHub's Latest), the hotpatch
  check the twelve boss fights. Neither refused anything.
- **Update (40 seconds):** WeiDU took off and installed only BuffBot. Changed: BuffBot's folder, 19
  of its `override/` files (one new), its uninstall list and `WeiDU.log`, nothing else. In
  `WeiDU.log` only BuffBot's two rows changed; every other line stayed byte for byte, the Latin-1
  byte too.
- **Real hotpatch after the update (37 seconds):** the twelve boss fights went in after everything.
  Undo of the fights (24 seconds), then of the update (22 seconds), each gave the folder back byte
  for byte.
- **The other order:** the fights first, then the update with them behind BuffBot (29 seconds). The
  folder equals the first order's, byte for byte.
- **Game start with both** (quiet lab runner, two minutes, main menu, no save): the EEex Remote
  Console answered `BfBot.VERSION=1.16.0` with BuffBot's UI, Persist, Exec and Innate modules
  loaded. Afterwards only the Remote Console's handshake file differed, neither check refused
  anything and Undo of the update was offered. Undo of the update (20 seconds) and of the fights
  (36 seconds) then gave back the folder from before, apart from that file.
- **Another language:** BuffBot installed again by hand in German. The update ran in German, its
  rows stayed `#3`, and `override/bfbot_l10n.tra` is the new release's German file. Undo gave the
  folder back byte for byte. The update left the profile folder unchanged.
- **Stopped hard:** the update and WeiDU were killed while WeiDU ran, with the mod list ending at
  BuffBot. Both checks then said a game update did not finish and offered Restore. Restore gave the
  folder back byte for byte, the whole mod list included.
- Not covered: playing a save with BuffBot updated through the app (Christopher's stream game
  plays 1.16.0, put in by the one-off script), and Chriz Easy BG.

## Lab check of the first version (2026-11-04)

The first version let WeiDU take the mods after BuffBot off and put them back, on the same
installation without a mod added by hand. Its update took 30 seconds, its Undo 18. One lesson stayed
in the code: WeiDU notes the old installation of every component it installs again as
`// Recently Uninstalled:` lines, which the first check took for a component left uninstalled; the
first apply was refused and the game put back byte for byte. Such lines are now accepted for
components that are active again.

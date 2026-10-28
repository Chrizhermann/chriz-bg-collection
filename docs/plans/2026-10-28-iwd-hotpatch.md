# Chriz Easy IWD: Hotpatch

Status: merged into `claude/iwdee-app` (PR #52) and shipped with Chriz Easy IWD 0.1.1,
2026-10-29. Christopher confirmed the three decisions at the end on 2026-10-29.

## What it is

A hotpatch brings new WeiDU components of one pinned mod release into a **finished**
Chriz Easy IWD installation, installed after everything else. It patches the whole
installation, so the game a player is in the middle of gets it too. The first real use is
Christopher's own fight mod, Chriz IWD Encounters (Belhifet first).

The player finds it under **Updates → Hotpatch**: a short line that it patches the running
game too, a folded changelog ("What changes") with what saves already in progress get, and
Check, Apply, Undo and Restore.

## Why it was set aside before, and what is different now

Hotpatches for Chriz Easy BG were stopped because too many individual installations
differed (handover, `docs/hotpatch-pilot-acceptance-2026-09-20.md`). The BG fix that shipped
(Artisan description links) only works for one exact file set. This design keeps that
strictness but moves it into one rule that is checked for every installation, and covers the
general case (new components of one mod) instead of one hand-made adapter.

## The rule (who gets a hotpatch)

A hotpatch is offered only when **all** of these hold. Every reason found is shown together.

1. The installation is the collection **as published**: its saved recipe has exactly the
   content of a release listed in the catalog (digest of collection, artifacts, mods and
   presets). A recipe changed before installing is refused, but the hotpatch it would have got
   is still named.
2. No manual mods were added at an installation pause.
3. No earlier game fix or whole-mod update is recorded (those are BG features; they cannot be
   combined).
4. The mod list (`WeiDU.log`) is exactly what the installer and earlier hotpatches left.
5. **No file in the game folder changed by hand.** The installer now records every game file
   (length and SHA-256) when it finishes; a hotpatch compares the folder with that record (or
   with what the last hotpatch left). Files the game, its loader, BuffBot and Radar write while
   playing are listed in the catalog and ignored.
6. The choices are close to the recommended setup:
   - Sword Coast Stratagems must be installed.
   - At most **2** other mods may be added, left out or set differently.
   - Choices inside Tweaks Anthology, the interface (Infinity UI, LeUI) and EEex never count.
7. The hotpatch's own needs are met (an earlier hotpatch, a required component).

Checking only reads. It never writes the game.

## What a hotpatch may change

- Its own mod folder, `override/`, the game language's `dialog.tlk`/`dialogF.tlk`, and
  WeiDU's own files (`WeiDU.log`, `weidu.conf`, `weidu_external/` for this mod).
- `WeiDU.log` must grow by exactly the hotpatch's rows, after all others, in order.
- WeiDU's result is judged as in a fresh installation: every component needs its result line,
  and "installed with warnings" counts as installed there too.
- Anything else (for example `chitin.key` or new BIF files) makes the hotpatch fail; the game
  files are then put back automatically where the copies allow it.
- Saves, the profile folder and the original installation record are never written.

## How Undo works

Before WeiDU runs, the app writes a durable record: the whole folder as it was (names,
lengths, digests), copies of the root files WeiDU writes, and the package. Only then does any
game file change. After WeiDU, every file that was replaced also gets an exact copy in the
record (taken from WeiDU's own backup of it). So:

- **Undo** of the latest hotpatch puts the game folder back byte for byte and checks it.
  It is offered only while nothing but the game's own runtime files changed since.
- **Interrupted** (crash, power loss): the game cannot be started from the app until
  **Restore game files** has run. Restoring needs no extra backup.
- Optional extra backups (whole game folder, recommended and on by default; saves, off by
  default) are copied before anything changes.

## What running saves get

The game reads changed creatures, scripts and spells from `override/` when they are loaded.
What a running game gets therefore depends on the change, and every changelog entry says it:

- "Your current game gets this." (for example a script change)
- "…if you have not been to that area yet." (creatures already placed in a visited area are
  stored in the save)
- "…if that event has not happened yet." / "For your next playthrough." / "Only for new games."

For Belhifet: a save that has not reached the final fight gets it. Undo puts the game files
back; progress saved with the hotpatch stays in those saves.

## Package format (confirmed by Christopher, 2026-10-29)

Hotpatches are **WeiDU components appended at the end**, not copied override files:

- The mod's own installer decides what changes, exactly as in a fresh installation, so the
  same release works for fresh installs (as a recipe mod) and hotpatches.
- WeiDU keeps its uninstall copies; the app additionally keeps its own exact copies, so Undo
  never depends on WeiDU's uninstaller.
- A component must install without questions.

The catalog (`hotpatches/iwd/catalog.toml`, bundled with the app) lists, per hotpatch: id,
title, summary, which releases it fits, the pinned release ZIP (length and SHA-256, in the
recipe's own artifact format), the WeiDU components, optional requirements, and the
changelog. A release goes out with the app; there is no separate download channel.

Requirements for Chriz IWD Encounters (to settle with its session): new fights ship as new
components of the same `setup-chriz-iwd-encounters.tp2`; each must be correct when installed
at the very end, after all other mods; it may write only `override/`, its folder and the TLK.

## Releasing

- Each release adds a `[[bases]]` entry with its recipe digest. A test keeps the current
  release's digest equal to `recipes/iwd-current`, so a recipe change before the release
  fails with the new digest in the message.
- A new hotpatch adds `artifacts/<id>.toml`, `mods/<id>.toml` and a `[[hotpatches]]` entry.
- Lab check before shipping: apply on a recommended lab installation, play-check, Undo,
  apply again; refused on a deviating one.

## Engine and app pieces

- `engine/src/game_files.rs`: the finished-folder record (written by final verification).
- `engine/src/hotpatch/`: catalog, rule (`gate.rs`), transactions (`transaction.rs`), tests.
- `chriz-bg-install hotpatch check|apply|undo|restore --catalog … --cache …` for labs.
- App: `hotpatches` flavor switch (IWD only), bridge commands, Updates section, launch guard.

## Lab check (2026-10-28)

One fresh installation of the recommended setup, with Chriz IWD Encounters 0.1.0 (its two
components) as a stand-in hotpatch, driven through `chriz-bg-install hotpatch`. The game folder
(42,679 files, 4.5 GB) was compared byte for byte by a separate script after every step; the
game was not started.

- Right after installing, the bundled catalog said "no hotpatch yet" and named no reason, so a
  finished installation passes the rule. The lab catalog offered the stand-in.
- Applying (with the whole-folder backup) ran WeiDU unattended: 150 files added, 118 changed,
  9 folders added, the two rows at the end of `WeiDU.log`, nothing outside the allowed area.
- Undo gave the original folder back byte for byte; applying again gave exactly the first
  result.
- A file edited by hand after the hotpatch made Undo refuse without writing anything; with the
  file put back, Undo worked. A file or `WeiDU.log` edited by hand before any hotpatch made
  the check refuse and name it; once put back, the hotpatch was offered again.
- Not covered: a deviating installation (the rule is pure comparison; a test leaves SCS out of
  the real recipe), starting the game, and a hotpatch that writes the TLK.

## Release lab check (2026-10-29)

A fresh installation of Chriz Easy IWD 0.1.1 (the recommended setup plus the EEex Remote
Console), driven through `chriz-bg-install hotpatch` and played through the quiet lab runner.
The stand-in hotpatch was a lab-only Chriz IWD Encounters 0.1.1-lab with one extra component
that adds a string to `dialog.tlk` and a file to `override/`.

- Freshly installed, the folder matched its record (42,847 files); the bundled catalog said "no
  hotpatch yet" with no reason, the lab catalog offered the stand-in.
- The first game start wrote files the list did not know: `InfinityLoader.db` and
  `EEex_scripts/extra_pattern_dbs/v2.7.3.0.db` (the loader caches the addresses it found in
  them) and the empty folder `EEex_scripts/override`. Christopher's own played installation
  shows the same three. Every played installation would have been refused as edited by hand;
  they are runtime files now, together with the EEex Remote Console's files. After that a
  played and saved installation passed with no reason.
- Applying with the whole-folder backup worked (13 minutes, mostly the 4.6 GB copy while a
  second lab installed); `WeiDU.log` gained exactly its row. A save made before the hotpatch,
  loaded afterwards, read the new file and the new string.
- Undo after playing gave the installed folder back byte for byte (only the game's runtime
  files differed, as before); applying again gave the same result.
- A deviating installation (no SCS, three more mods left out) was refused with both reasons.

## Limits and open points

- Installations made before the file record existed are refused ("made before game updates
  could check them"). The first public IWD release has the record.
- The runtime file list comes from the loader, BuffBot 1.12, the remote console and Radar, and
  from what played installations wrote. A mod that writes other files while playing would make
  played installations look hand-edited; the list ships in the app's catalog, so a later app
  can extend it.
- A hotpatch cannot add an EEex pattern database: those files are runtime files, and the
  folder is outside what a hotpatch may change.
- Changes outside the allowed area after a failed hotpatch are left for a closer look; the
  optional whole-folder backup covers that case.

### Decisions (Christopher, 2026-10-29: as proposed)

1. Package format: WeiDU components at the end, not exact override files.
2. Threshold: at most 2 other mods; free: Tweaks Anthology, interface, EEex; SCS required.
3. Full-folder backup on by default.

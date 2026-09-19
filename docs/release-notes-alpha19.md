# Chriz Easy BG 0.1.0-alpha.19

Draft: the Windows package is **not built, signed or published** yet.

Collection **0.1.0-alpha.16** is unchanged; the recommended selection remains
**448 components / 50 mod-install runs**. This is an application-only update:
your installed mods and choices are not affected by upgrading CEBG.

## Fixes for your current game

- Updates now offers **targeted fixes for a game you already installed with CEBG**,
  instead of only a fresh installation. Pick the registered installation, check it
  explicitly, and CEBG reports whether a fix applies.
- The first fix restores the campaign class-table description links for the selected
  Artisan's Kitpack kits (English, recorded Artisan `chriz-v1.3.0` components only).
  It does not add kit mechanics, rewrite dialog.tlk, change saved characters or
  upgrade the mod.
- Backups of the affected files are always made. A full backup of the playable game
  folder is recommended and selected by default; a profile/save snapshot is optional.
  Sizes, free space and the backup location are shown before anything is written.
- **Apply**, **Undo** and, after an interruption, **Restore** are separate actions,
  each requiring the game to be closed. An interrupted fix blocks Play until the
  installation is restored, with recovery guidance in My installs.
- Your diagnostic export now includes the fix and recovery evidence.

## Installer

- The Updates badge counts applicable fixes and recovery states alongside the
  existing application, collection and Radar notices, without a startup scan.

## Notes

- CEBG updates itself; game fixes are separate and always ask first.
- Updater signing authenticates the setup to CEBG's own updater. It is not a Windows
  Authenticode publisher signature, which this alpha still does not have.

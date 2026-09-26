# Chriz Easy BG 0.1.0-alpha.19

Draft, not published. The combined normal installer built locally on September 23
(KST), with package/signature tests passing. It has not been installed or launched;
Christopher's finishing UI review is pending. BuffBot's update, existing-save
gameplay check and Undo passed in the earlier September 22 diagnostic package.
The September 26 source follow-up also fixes hidden patch/Radar feedback on the
initially selected installation; that correction is not in the September 23 binary.
See [package evidence](package-acceptance-alpha19-2026-09-22.md)
and [release readiness](plans/2026-09-22-release-readiness.md).

Collection **0.1.0-alpha.17** corrects component information and update records,
and includes the in-game interval-save controls by default. The recommended
selection is now **449 components / 50 mod-install runs**; mod versions and order
are unchanged. Updating the CEBG app does not change your existing game or choices.

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

- Fixes interval saves being installed but left inactive because EET wrote their
  settings to a different game profile. CEBG now adds missing settings to the
  installation's own profile and checks them before reporting success. Existing
  preferences, including explicitly turning interval saves off, are preserved.
- New recommended installations include the interval-save settings menu. It follows
  your chosen save interval and stays inactive if you exclude interval saves.
- The Updates badge counts applicable fixes and recovery states alongside the
  existing application, collection and Radar notices, without a startup scan.
- Both bundled recipe profiles pass the release checks. Already-approved components
  have complete catalog entries; older update records no longer repeat the same
  changes or omit changed download identities.
- Removes the incorrect Red Wizard / Spell Revisions exclusion note and unused
  archive records. Selected mods and their installation order do not change.

## Update BuffBot without reinstalling your game

- Compare the mods in an existing installation without resetting your choices.
- Eligible installations can update **BuffBot 1.8.3-alpha to 1.8.4-alpha in the
  same folder**. Unidentified items no longer reveal their names in BuffBot.
- Your saves, BuffBot settings and other installed mods stay in place. Review the
  update first; backups, Undo and recovery after an interruption are included.
- This exact update was applied, checked in-game with an existing save, and undone
  successfully. It does not upgrade the rest of the collection. See the
  [supported scope](existing-install-upgrades-2026-09-21.md).
- Fixed the package check that incorrectly rejected the original BuffBot downloads.
  If only a check fails, CEBG makes clear that it did not apply an update.

## Installer improvements in the local candidate

These changes are in the newly built local candidate. Source checks and four
package tests passed; user review is pending. They are not in the currently
installed diagnostic app, and the new candidate has not been published.

- A smaller installation selector at the top of Updates, shared by all game-specific
  actions; longer paths and technical explanations stay available under Details.
- Backups show verified bytes copied. Installation/update stages use a progress
  bar when a total is known and a spinner otherwise, without guessing time left.
- Obvious, dismissible completion, Undo and failure messages. App-update success
  is confirmed after restarting into the requested version.
- Unique names for new installations, with a display-only Rename option in My
  installs. Existing folders and saves are not renamed. Old duplicate names are
  distinguished in the selector.
- Folder errors explain unavailable folders, permissions, locks or conflicting
  files and preserve the exact operation/path in details.
- Customize brings common changes forward and supports the recipe's typed choices.
  The alternative-interface controls are integrated but stay hidden until their
  verified providers are included in the recipe; no new UI mod is activated here.

The confusing history entry for an undone BuffBot update now reads
**Restored BuffBot 1.8.3-alpha** in this candidate. The earlier diagnostic package
used for the accepted Apply/Undo test did not have this clearer wording.

## Notes

- CEBG updates itself; game fixes are separate and always ask first.
- General multi-mod upgrades are not implemented yet. The first BuffBot-only
  route above and targeted description fix are not a whole-collection upgrade.
- Broader mod updates, collection-wide compatibility categories and additional
  hotpatches are deferred; they are not requirements for this release.
- Updater signing authenticates the setup to CEBG's own updater. It is not a Windows
  Authenticode publisher signature, which this alpha still does not have.
- Need help? See the [short troubleshooting and backup guide](troubleshooting.md).

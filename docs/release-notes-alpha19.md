# Chriz Easy BG 0.1.0-alpha.19

**October 1 release hold:** Defender quarantined an earlier installed candidate.
Investigation and Microsoft analysis are pending; a successful package test is
not evidence that the runtime alert is resolved. See [current handover](handover.md).
The Radar launch correction is now in the verified local
[October 1 Radar Fix candidate](package-acceptance-alpha19-radar-2026-10-01.md).
That package has not been launched or cleared by Microsoft.

Draft, not published. The fresh Test19 installation completed on September 30,
and Christopher launched the game. The earlier failed attempt was superseded;
do not direct him back to its recovery steps or require another full install.

The [Dragon and Order Fix package](package-acceptance-alpha19-dragon-order-2026-09-30.md)
contains the dragon, component-order and resume corrections. The later
[Selection Fix package](package-acceptance-alpha19-dragon-order-2026-09-30.md#selection-fix-addendum--latest-local-setup)
adds the Customize focus fix without changing the recipe. Defender's logs identify
the affected installed app as Dragon and Order Fix, not Selection Fix. Neither
package includes the newer Radar consent correction. The October 1 candidate adds
that correction and has passed final automated packaging checks. The
remaining [short supervised checks](next-release-smoke-check.md) are still pending.
The accepted BuffBot Apply/existing-save/Undo test does not need repeating.

Current source uses collection **0.1.0-alpha.18**. Updating the CEBG app does not
change your existing game or choices. The new collection changes apply to fresh
installations unless an individual supported update is explicitly offered.

## Short Discord draft

**Not for posting yet:** finish the release decision and agreed manual check first.
The copy below describes the intended release, not the older test installers.

Hi friends! This CEBG update brings a bunch of fixes and some more choices:

- More interfaces: Infinity UI++, Dragonspear UI++, LeUI and EET's SoD style.
  Pick one for a new install; the current default stays available.
- Optional install breakpoints for people who want to add their own mods.
  This is an advanced option, not something you need for a normal install.
- Evandra's Sorcerer conversion and portrait are now included by default when
  you take her. BG1/SoD utility XP stays at BG1 values until you reach BG2;
  the smooth XP curve is still an option.
- Emotion/Courage/Hope no longer stack. Flail of the Ages +5 allows Haste, its
  Slow gets a saving throw, and dragons use the longer Wing Buffet cooldown by default.
- Optional generic unidentified-item visuals and talking while shapeshifted.
- Updated Artisan, Bardic Wonders, SoD Remix and BuffBot, plus clearer progress,
  update messages and installation names. You can choose whether Play starts Radar.

Updating CEBG does not replace your current modded game. The new setup is for
fresh installs; existing games only get the specific supported fixes offered
under Updates. Adding your own mods at a breakpoint disables game hotpatches
and mod upgrades for that copy, but not CEBG app updates.

Thanks for all the reports and patience! The extra interfaces and custom mod
combinations are open for community testing, so please report anything weird.

## Website summary draft

**Hold with the download:** publish only after the final release is approved,
and link to that exact package and the full changelog. Do not call the Defender
issue resolved until we have evidence for that claim.

Chriz Easy BG alpha.19 brings more interface choices, advanced pauses for adding
your own mods, clearer installation and update progress, and an optional Radar
launch checkbox. Collection alpha.18 refreshes the mod packages and includes
Evandra's Sorcerer conversion and portrait by default when she is selected, story-based utility XP,
Emotion and Flail of the Ages balance changes, and a longer dragon Wing Buffet
cooldown. Generic unidentified-item visuals and talking while shapeshifted are
optional. The collection changes are for new installations; updating the app
leaves your existing game unchanged. Supported fixes for existing games are
offered separately in Updates.

## New-install collection changes

- **Longer dragon Wing Buffet cooldown by default:** 18 seconds instead of 6.
  Requires SCS Smarter Dragons and can still be switched off in Customize.
- **Emotion, Courage and Hope no longer stack with each other.** Casting one
  replaces the others; recasting refreshes it. This is on by default. The separate
  bard progression alternatives remain optional and unselected.
- **Flail of the Ages:** the +5 version now allows Haste while retaining its Free
  Action protection. Slow allows a breath save at -2 for +3, -4 for either +4
  version, and -6 for +5. Its damage, proc chance and Slow duration stay unchanged.
- Updated Artisan's Kitpack, Bardic Wonders, SoD Remix and BuffBot packages. Fresh
  installs use BuffBot 1.9.0; this does not expand the older-game update route below.
- Optional **talking while shapeshifted** for Artisan's Shapeshifter/Hivemaster
  forms. Other form restrictions stay in place. Off by default.
- **Story-based utility XP** becomes the default: BG1-sized rewards throughout
  BG1 and SoD, BG2-sized rewards from SoA onward. Uncapped character XP will not
  unlock BG2 rewards early. The existing smooth XP-based curve remains an
  optional, mutually exclusive alternative.
- Evandra's **Sorcerer conversion and custom portrait** are included by default
  when Evandra is selected. Each can be turned off independently. This does not
  resolve the separately reported companion XP catch-up issue.
- Optional **generic visuals for unidentified items**, using ordinary game art
  by item category. Identifying an item reveals its usual appearance. This is
  separate from item randomization and conflicts with its appearance-changing
  option, not the ordinary randomizer.
- Artisan's fork version now names both the upstream base and our revision:
  **4.81a-dev.85928f9-chriz.1**. Older published releases have not been renamed.

Modpack v0.2.0-alpha.8 is published, verified and integrated. The refreshed
September 29 installer passed its combined checks/build and contains these
additions; the older September 23 binary does not.

## Choose your interface

New installations can choose Infinity UI++, EET's SoD-style interface, LeUI's
BG1/BG2/SoD variants or Dragonspear UI++. The recommended interface stays the
default. The installer excludes known conflicting UI options and adds the
required classic-UI text compatibility fix automatically. Existing installations
are not switched to another interface. Broader gameplay testing of the optional
interfaces is left to the community, rather than requiring a separate full test
install for every skin.

### Installation fixes

- Fixed an error when changing Customize options, including the stronger SoD
  bridge mages. The selected choice is retained and keyboard focus returns to
  the changed control without displaying an internal selector error.
- Apex Dragons supports both SCS difficulty systems. Valid alternative-interface
  setups no longer fail its INI-only compatibility check. Dragon abilities and
  difficulty thresholds are unchanged.
- Fixed the expected install order for Flail of the Ages within BG Rebalance.
  The installer now checks that recipe order against the actual mod source.
  These fixes do not automatically recover an already-stopped installation.
- When a mod partly installs, diagnostics distinguish missing components from
  an unexpected component order without weakening the install-history checks.
- Resuming or retrying starts only once, even if the action is pressed again
  while it is starting. Resumed installs show their current phase, and cached
  downloads no longer make an unrelated mod installation look 100% complete.
  This follow-up is included in the September 30 Dragon and Order Fix setup;
  leave any installation that is still running normally alone.
- Alternative interfaces use the supported mod-launch method. Incorrect launch
  settings are now caught before a new installation begins. Recovery of an
  older failed installation still depends on its recorded failure and history;
  installing the new app is not a blanket guarantee that Resume is safe.
- Reusing the same download with a different required folder layout no longer
  causes the EET cache mismatch. Download and extracted-file checks stay enabled.
- Installer configuration failures explain what to do; internal details stay in
  the expandable error section.

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
  archive records. Those record corrections preserve the prior selections and order;
  alpha.18 separately adds the approved Emotion default described above.

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

## Installer improvements

- Choose whether **Play** also starts Radar. The checkbox starts off and remembers
  your choice for each installation. Radar can request its own Windows approval;
  declining it or a launch failure leaves the game running and shows a clear
  message. The whole installer does not need administrator access for this.
  Automated frontend and Windows-native checks pass. This correction still
  needs a packaged, user-supervised launch check; it does not resolve the
  separate Defender investigation.

The improvements below predate the Radar correction and were included in the
September 29 test installer. Public release and outstanding manual checks remain
separate from package verification.

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
- **Advanced manual-mod pauses:** choose one or more completed-component boundaries,
  install your own additions in the displayed managed folder, then review and
  accept them before continuing. Normal installations have no extra pauses.
  Adding manual mods disables game fixes and mod upgrades for that copy; app
  updates remain available. See the [manual-mod guide](manual-mods.md).
- Installation removal shows an estimated folder size before deletion; the
  existing target checks and confirmation still apply.
- Fixed hidden patch/Radar feedback for the initially selected installation.

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

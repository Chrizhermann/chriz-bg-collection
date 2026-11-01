# Chriz Easy BG 0.1.0

**Release candidate, not published:** app 0.1.0 / collection 0.1.0.
Exact package and test results: [0.1.0 package verification](package-acceptance-0.1.0-2026-11-01.md).

This update brings a new recommended mod setup with new choices, updated mods and many
fixes, and makes the installer sturdier. These notes cover everything since app alpha.21
and collection alpha.20. 0.1.0 is the first CEBG release without the alpha label.

The new mod setup is for **fresh installations**. Updating CEBG leaves your current games
and saves unchanged. The installer fixes also help with installations you already have.

## Installer and launcher

- **Installations no longer stop when a virus scanner briefly holds a file.** Windows
  Defender and other scanners often look at a file right after it is written. If CEBG
  moved or replaced that file at the same moment, Windows refused, and the step could
  stop with "Access is denied". CEBG now waits a moment and tries again, for up to about
  five seconds, before it reports a problem.
- **CEBG carries the collection's change record.** The app now includes a record of the
  32 changes in collection 0.1.0, which follows collection alpha.20, the collection that
  app alpha.21 installed. The record marks every change for your next playthrough,
  because each comes with a new installation. The Updates page does not list these
  changes yet; these notes describe them.
- **The smaller setup installs Artisan's Kitpack and the Randomiser in the right order.**
  The second setup you can pick when you install, now called **Smaller downloadable
  setup** (was "Smaller downloadable alpha"), asked for some components of both mods in
  another order than the mods install them, so an installation with it could stop there.
  The default full setup was not affected.
- **Uninstalling with "Delete the application data" removes it completely and safely.**
  The uninstaller used to leave files with very long paths behind, such as unpacked mods,
  and could follow a folder link to delete files outside CEBG. CEBG now removes its own
  application data itself. If something is left, the uninstaller names the folders.
- **Interrupted installations stay resumable after the download folder was deleted.**
  They used to show "folder moved or changed". CEBG now recreates the folder and
  downloads what is missing.
- **Stop works while another program holds a download.** A download or unpack that
  waited for another CEBG process used to ignore Stop.
- **Removing an installation finishes** when another program holds its removal record
  open. The installation used to stay listed as if the removal had failed.
- **Game fixes, the BuffBot update and removing an installation can no longer be
  redirected by a folder link.** CEBG now writes and deletes the game's files for these,
  including their Undo and backup restore, only through the game folder it holds open.
  While an installation is being removed, another program can no longer move a file out
  of it and have CEBG delete it there.
- **A step whose WeiDU copy could not be prepared can be retried.** It used to require a
  fresh copy of the games.
- **Challenge Mode can pick an option of a choice.** Switching it on now also picks the
  strict potion rule instead of the recommended one, and switching it off restores the
  recommended rule.
- **EET's desktop shortcut and EET's 64-bit tools work in every environment.** If Windows
  had `NoDefaultCurrentDirectoryInExePath` set, EET's shortcut failed and EET quietly used
  its 32-bit tools. Mod installers no longer inherit that setting.
- **Large downloads on slower connections no longer fail after 30 seconds.** EET or
  Infinity UI could not be downloaded below roughly 60 Mbit/s. A download is now retried
  only when no data has arrived for a minute.
- **The game can write its own log for crash reports.** Open **Troubleshooting** at the
  end of an installation on the home screen and check **Write a game log for crash
  reports**. It works for existing installations without reinstalling and takes effect at
  the next game start; close the game before changing it. **Play** keeps the previous
  session's log as `InfinityLoader-previous.log`. The section shows where the logs and the
  game's crash reports are, and which files to send.
- **Unticking an optional mod no longer adds setup notes for its components.** Unticking
  Hidden Gameplay Options, for example, used to add one note per default component.
- **Removing an installation also removes its saved-game profile if the game never used
  it.** The confirmation names the folder. A profile with settings or saves in it is kept.

## New-install collection

### New choices

All are selected in Chris's Recommended unless marked optional.

- **True Time Stop – fair Haste & Slow.** Time Stop stops everything but the caster:
  frozen creatures' spells and potions no longer run out while they cannot act, and their
  poison and regeneration wait too. Haste and Slow change only actions and movement, not
  poison, regeneration or other repeated effects. Install it before a playthrough; adding
  or removing it during one is not supported.
- **Druid weapons.** Druids and every druid kit can use axes and short bows and put one
  point into them; fighter/druids can also use long bows, with up to two points in all
  three.
- **Alaghor of Clangeddin weapons.** Yeslick's kit trains and uses the cleric weapons plus
  axes, Clangeddin's weapon, with up to two points. The thief weapons drop out; points
  already spent stay.
- **Sirene in BG1 and Siege of Dragonspear.** Sirene, a paladin, can join the party in BG1
  and in Siege of Dragonspear, and carries on into Shadows of Amn with Sirene NPC for BG2.
  She is a True Paladin with the Sirick portrait; Cavalier, Inquisitor and Undead Hunter
  are optional. Pick the same class for her in both games.
- **Chris Revisions of Aid, Luck, Bless, Chaotic Commands, Prayer and Recitation**, with
  or without Spell Revisions. Aid grants only hit points (1d8 +1 per caster level, at most
  +10) and replaces itself when cast again. Luck lasts 90 seconds. Bless lasts 30 seconds
  plus one round per caster level. Druids can cast Bless and Chaotic Commands again, and
  rangers Bless. Prayer and Recitation no longer stack: Recitation's bonus or penalty
  replaces Prayer's while it lasts.
- **Potions: one dose of each type at a time.** Drinking a potion type that is already
  active restarts its duration instead of stacking its bonus; potions that raise maximum
  hit points keep the first dose until it ends. Potions of Genius still stack. The strict
  version, where they don't, is optional and part of Challenge Mode.
- **Heroism effects don't combine.** The first Heroism, Greater Heroism or Potion of
  Heroism active on a creature wins. Song of Heroism, Aid and the Potion of Power stay
  separate.
- **Fine weapons instead of +1 weapons, and they never break.** Most standard +1 weapons
  become nonmagical "fine" weapons: still +1 to hit and damage, but no initiative bonus,
  no harm to creatures immune to normal weapons, and about a third of the price. They
  never break in BG1's iron crisis. Fine ammunition is optional.
- **Companion stats**, each separately: Rasaad Dexterity 17 and Constitution 16; Dorn
  Constitution 16; Eldoth Strength 17, Constitution 16, Wisdom 9 and Intelligence 14;
  Garrick Dexterity 17 and Intelligence 15; Quayle Constitution 15. Only the base scores
  change.
- **Join XP for every joinable NPC.** NPCs who lacked EET's join-XP catch-up, mostly from
  mods, now get it: Sarah, for example, used to join at 89,000 XP. It never lowers XP and
  doesn't repeat when an NPC rejoins.
- **Choose how Imoen continues into Shadows of Amn.** By default she becomes the BG2
  thief/mage Imoen plus the tome boosts she read in BG1 and SoD. Optional: she continues
  as you took her along, or (with EEex) she dual-classes to mage. Her SoD gear is handed
  over like the party's, and she keeps only her BG2 belt, correctly equipped. Raise a
  dead Imoen before ending SoD.
- **Dorn's BG1 ambush no longer crashes the game.** The cutscene no longer locks the
  camera on Senjak; a completed conversation still ends in the fight.
- **Defensive Stance follows "Use special abilities".** With the option off in the AI
  settings, your main character and others on the default party AI no longer enter
  Defensive Stance on their own in BG1.
- **Kit weapon rules for weapons from later mods.** Artisan's Kitpack's weapon rules now
  also cover weapons that later mods add: the Priest of Tempus and the Eldritch Knight can
  use IWDification's two-handed axes, for example.
- **18 player voicepacks** from ChrizFader's voicepacks, made from the characters'
  original game audio, among them Neera, Safana, Jaheira, Edwin, Imoen, Viconia and
  Sarevok. Choose them as your character's voice in character creation; companions keep
  their own voices.
- **Summoned creatures clear the fog of war**, **Ribald's special stock in his regular
  store** and **sensible entrance points**, now also on EET (Tweaks Anthology).
- **EEex attack info** (optional): shows your chance to hit when the attack cursor is
  over a target. You can also switch it on or off later in EEex's options; the change
  takes effect after a restart.
- **No stealing from fences** (optional, part of Challenge Mode): fences can't be stolen
  from, and reputation no longer changes their prices.
- **Chapter-by-chapter XP caps** (optional, experimental, needs the campaign XP caps):
  usable XP is capped per chapter, and XP above the cap is banked until a later chapter.
- **Quest mods and more** (optional, experimental, offered as they are): Northern Tales of
  the Sword Coast and Ascalon's Questpack in a new "Quest mods" category, Thalantyr Item
  Upgrade, the Icewind Dale Voice Pack and Portraits Portraits Everywhere (about 0.4 GB).
  **Black Hearts** is offered for installations without the SoD Remix.

### Changed defaults

- **Unfinished Business** for BG1 and for BG2 is now **optional**. Every component stays
  available, unchecked in Chris's Recommended. With BG1 Unfinished Business and Northern
  Tales both ticked, BG1 Unfinished Business keeps the Nim Furlwing and Svlast encounters.
- **Druids level like clerics** is now **optional**. Druids keep their own XP table again:
  level 15 at 3,000,000 XP instead of 1,575,000.

### Changed behavior and fixes

- **Sorcerers can equip daggers, quarterstaves, darts and slings again** without
  proficiency points, the Dragon Disciple included. **Eldritch Knight and Arcane Trickster
  sorcerers keep their kit weapons** when they level up or join the party. **Monks can
  equip quarterstaves** with Monk Revisions, and shamans without a kit, Sacred Fists,
  Enlightened Fists and Ninjas are held to their weapons from the start.
- **The Invisible Blade and Bardic Wonders' Dancer can use katanas** and the rogue and bard
  items that kept them out by mistake, also those that later mods add, such as SCS's Drow
  Katana. The Shapeshifter and the Elementalist still cannot use them.
- **BuffBot v1.12.6** (was v1.10.0):
  - The "Combat detected - casting stopped" and "casting timed out" notices now appear
    over the party leader's head; before, they were always lost.
  - New: an optional 2x or 4x speed during a buff run, presets that cast their
    longest-lasting buffs first, casting buffs that are nearly over again (off by default),
    and a search by buff name.
  - The panel wears the theme's artwork and colours, so it is readable on dark themes, and
    a button you press looks pressed. The default colour scheme looks as before and is now
    called "Baldur's Gate".
  - The F12 preset abilities are gone: right-click the HUD gear to pick a preset. BuffBot
    no longer adds text to the game's `dialog.tlk`.
  - Fixed: restoring temporarily equipped items no longer risks overwriting the first
    backpack slot, which could corrupt the character's name and crash the game. A buff
    run no longer speeds the game up while the options, the character record, a level-up,
    the journal, a shop or another mod's dialog is open. The Summons view lists only
    creatures your party summoned or cloned, and the panel can switch from it back to the
    party again. On a small panel, captions no longer spill onto neighbouring buttons.
- **SoD Remix v0.6.16**: its victory ending installs alongside mods that add their own
  victory remark after the battle of Avernus, such as Sirene, and keeps the pause after
  each remark short. Its Boareskyr Bridge finale installs alongside "Summoned creatures
  clear the fog of war". Other parts no longer stop when a mod renames one of the game's
  timer lengths.
- **Scrolls at the reader's caster level** stay on for the whole session. Before, they
  switched themselves off after a load, a new game, an NPC joining or opening a
  container, showed an EEex message box, and scrolls then cast at their normal level until
  the game was restarted.
- **Campaign XP caps with banked XP** (optional, experimental) now work. With collection
  alpha.20 they refused every permanent XP award. Script variables also survive saving
  and loading with them.
- **Grandmastery:** High Mastery now gives +5 damage instead of +4, matching Grand
  Mastery. With Spell Revisions, the Black Blade of Disaster also gets +3/2 attacks per
  round and +8 to hit / +10 damage.
- **Move NPCs to convenient locations** now also moves Ajantis to the Friendly Arm Inn.
- **Party movement speed outside combat** (optional) now uses the collection's 125%
  instead of the Tweaks Anthology's 200%.
- **Fade's and Evandra's join XP** no longer adds a second catch-up under EET.
- **Bardic Wonders:** the Troubadour and Deathsinger spell copies and descriptions follow
  Spell Revisions, the Gallant gets the new bard spells, and Symphony of the Dark Children
  ends with its round of Invisibility again.
- **EEex v1.3.1** (was v1.2.0): the EEex Options menu pauses the game and no longer opens
  during dialogs or cutscenes, and InfinityLoader no longer fails to start the game on
  some systems.
- **BG Rebalance:** Bracers of Blinding Strike and Spell Revisions' Haste or Improved
  Haste no longer combine when the spell is cast first; a short full immunity such as
  SCS's Divine Protection stays at 100% under the party resistance cap while it lasts; and
  the cloud forms, elemental arrows, Cleric of Tempus and Emotion spells keep other mods'
  changes instead of overwriting them.
- **Spell Revisions:** with Rogue Rebalancing's encounters, Aran Linvail, the Zhentarim
  Mercenary and Selina prepare Hold Monster, Feeblemind and Domination instead of Chaos
  copies that had moved to level 7, and the Lightning Bolt choice keeps damage that
  another mod adds instead of stopping the installation.
- **Tweaks Anthology:** the SoD Death Cam no longer hangs in two cutscenes, mages can
  decline the Sphere stronghold, and familiars no longer get the hit points meant for
  party members.
- **EEex Remote Console** (optional, experimental): the single-player menu no longer
  stays behind the game's screens after you return to the main menu.
- **The Martyr paladin kit** now comes from Artisan's Kitpack alone and stays available at
  character creation, unchanged.

### Package versions

- Chriz BG Modpack 0.2.0-alpha.14 (was alpha.11)
- BG Rebalance 0.8.8 (was 0.8.2)
- SoD Remix 0.6.16 (was 0.6.13)
- BuffBot 1.12.6 (was 1.10.0)
- EEex 1.3.1 (was 1.2.0)
- Bardic Wonders 2.9c-balance.8 (was balance.7)
- Spell Revisions fork 4.21-chriz.7 (was chriz.5)
- Artisan's Kitpack fork 4.81a-dev.85928f9-chriz.5 (was chriz.1)
- EEex Remote Console 0.2.1 (was 0.2.0)
- The Tweaks Anthology: development version `7649ced`, 134 changes after v18 (was v18)
- New: ChrizFader's voicepacks 1.1.0, Sirene NPC for BG:EE (development version
  `1c86000`), Northern Tales of the Sword Coast (development version `f0ef953`),
  Ascalon's Questpack 7.0, Thalantyr Item Upgrade 4.5.1, Icewind Dale Voice Pack 1.2.0,
  Portraits Portraits Everywhere (DaftHunk's fork `417226e`) and Black Hearts 2.05

## Important notes

- Collection 0.1.0 needs app 0.1.0. Older apps cannot install it.
- Existing installations keep EEex v1.2.0 and their other mod versions; the new versions
  come with a new installation.
- Several new parts passed automated checks or their own lab tests but have not been
  played through in a full game yet, among them True Time Stop, Chris Revisions and the
  weapon changes. Saving, dispelling, curing or dying during a Time Stop are not covered
  yet.
- The quest mods, Portraits Portraits Everywhere, Thalantyr Item Upgrade, the Icewind
  Dale Voice Pack and Black Hearts are third-party mods, offered as they are and not yet
  tested in a combined installation. The campaign XP caps stay experimental.
- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

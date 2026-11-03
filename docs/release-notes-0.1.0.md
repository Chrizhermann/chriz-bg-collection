# Chriz Easy BG 0.1.0

**Published:** app 0.1.0 / collection 0.1.0, on 2026-11-02.
[Download and release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0).
Exact package and test results: [0.1.0 package verification](package-acceptance-0.1.0-2026-11-02.md).

0.1.0 is the first CEBG release without the alpha label. It brings a new recommended mod
setup with True Time Stop, Sirene in BG1 and Siege of Dragonspear, 18 player voicepacks
and more weapon choices, many fixes, and a sturdier installer. These notes cover
everything since app alpha.21 and collection alpha.20.

The new mod setup is for **fresh installations**. Updating CEBG leaves your current games
and saves unchanged, and the installer fixes also help installations you already have.
See [Installing and updating](#installing-and-updating).

## Highlights

- **True Time Stop – fair Haste & Slow.** Time Stop now stops everything but the caster:
  frozen creatures' spells and potions don't run out, and their poison and regeneration
  wait too. Haste and Slow change only actions and movement.
- **Sirene joins in BG1 and Siege of Dragonspear.** The paladin, already a companion in
  Shadows of Amn, can now travel with you from BG1 on and carries on into BG2.
- **18 player voicepacks.** Give your own character the voice of Neera, Jaheira, Edwin,
  Imoen, Viconia, Sarevok and more, made from their original game audio.
- **More weapon choices.** Druids can use axes and short bows, Yeslick's Alaghor of
  Clangeddin trains axes, and the Invisible Blade and Bardic Wonders' Dancer can use
  katanas.
- **BuffBot 1.12.6.** Buff runs at 2x or 4x speed, presets on a right-click of the HUD gear,
  longest-lasting buffs first, and a themed panel that is readable on dark themes.
- **Rebalanced buffs, potions and +1 weapons.** Chris Revisions rework Aid, Bless, Luck,
  Prayer and Recitation, each potion type works one dose at a time, Heroism effects no
  longer combine, and most standard +1 weapons become nonmagical fine weapons.
- **Companions keep up.** Imoen continues into Shadows of Amn as BG2's thief/mage, keeping
  the tome boosts she read. Every joinable NPC, mod NPCs included, now catches up in XP
  when joining in BG2, and Rasaad, Dorn, Eldoth, Garrick and Quayle get improved base
  stats.
- **Optional quest mods.** Northern Tales of the Sword Coast and Ascalon's Questpack in a
  new Quest mods category, and Black Hearts for installations without the SoD Remix:
  third-party, unticked and experimental.

## Important fixes

Game fixes come with a new installation; installer fixes work as soon as you update CEBG.

- **White werewolves no longer crash the game.** The Shapeshifter's white and greater
  white werewolves could crash it with an overhand swing next to a wall.
- **Dorn's BG1 ambush no longer crashes the game** when you leave the area after a stalled
  conversation.
- **Sorcerers can equip daggers, quarterstaves, darts and slings again**, the Dragon
  Disciple included, and Eldritch Knight and Arcane Trickster sorcerers keep their kit
  weapons.
- **BuffBot no longer risks corrupting a character's name** and crashing the game when it
  puts back temporarily equipped items.
- **Scrolls stay at the reader's caster level** for the whole session; the rule used to
  switch itself off, for example after loading a game.
- **Pierce Magic and Spellstrike describe what they do.** Their descriptions now match
  their effects. The spells themselves are unchanged.
- **Installations no longer stop with "Access is denied"** when a virus scanner briefly
  holds a file: CEBG waits a moment and tries again.
- **Large downloads work on slower connections.** EET and Infinity UI could not be
  downloaded below roughly 60 Mbit/s.

## Everything else

Grouped by area. Where a highlight or fix above has more to it, its details are here too.
Everything new is selected in Chris's Recommended unless marked optional.

### Installer and app

- **Virus scanners:** Windows Defender and other scanners often look at a file right
  after it is written. If CEBG moved or replaced that file at the same moment, Windows
  refused, and the step could stop with "Access is denied". CEBG now waits a moment and
  tries again, for up to about five seconds, before it reports a problem.
- **Slower connections:** every download attempt used to stop after 30 seconds, even
  while data was still arriving. A download is now retried only when no data has arrived
  for a minute.
- **CEBG carries the collection's change record.** The app now includes a record of the
  34 changes in collection 0.1.0, which follows collection alpha.20, the collection that
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
- **Removing an installation also removes its saved-game profile if the game never used
  it.** The confirmation names the folder. A profile with settings or saves in it is kept.
- **Game fixes, the BuffBot update and removing an installation can no longer be
  redirected by a folder link.** CEBG now writes and deletes the game's files for these,
  including their Undo and backup restore, only through the game folder it holds open.
  While an installation is being removed, another program can no longer move a file out
  of it and have CEBG delete it there.
- **A step whose WeiDU copy could not be prepared can be retried.** It used to require a
  fresh copy of the games.
- **The game can write its own log for crash reports.** Open **Troubleshooting** at the
  end of an installation on the home screen and check **Write a game log for crash
  reports**. It works for existing installations without reinstalling and takes effect at
  the next game start; close the game before changing it. **Play** keeps the previous
  session's log as `InfinityLoader-previous.log`. The section shows where the logs and the
  game's crash reports are, and which files to send.
- **Challenge Mode can pick an option of a choice.** Switching it on now also picks the
  strict potion rule instead of the recommended one, and switching it off restores the
  recommended rule.
- **Unticking an optional mod no longer adds setup notes for its components.** Unticking
  Hidden Gameplay Options, for example, used to add one note per default component.
- **EET's desktop shortcut and EET's 64-bit tools work in every environment.** If Windows
  had `NoDefaultCurrentDirectoryInExePath` set, EET's shortcut failed and EET quietly used
  its 32-bit tools. Mod installers no longer inherit that setting.

### Companions

- **Sirene in BG1 and Siege of Dragonspear:** as in BG2, she is a True Paladin with the
  Sirick portrait; Cavalier, Inquisitor and Undead Hunter are optional. Pick the same class
  for her in both games. Sirene NPC for BG2 continues her story in Shadows of Amn.
- **Imoen into Shadows of Amn:** by default she becomes the BG2 thief/mage Imoen plus the
  tome boosts she read in BG1 and SoD. Optional instead: she continues as you took her
  along, or (with EEex) she dual-classes to mage at her end-of-SoD thief level and Shadows
  of Amn's catch-up XP goes into mage. With every choice her SoD gear is handed over like
  the party's, and she keeps only her BG2 belt, correctly equipped; she no longer arrives
  with all her gear and her belt in the magic-weapon slot. Raise a dead Imoen before ending
  SoD.
- **Join XP for every joinable NPC:** NPCs who lacked EET's join-XP catch-up, mostly from
  mods, now get it. Joining in Shadows of Amn raises them to the highest XP threshold your
  main character has reached, from 125,000 up to 1,250,000, and joining in Throne of Bhaal
  to 2,500,000. Sarah, for example, used to join at 89,000 XP. It never lowers XP and
  doesn't repeat when an NPC rejoins.
- **Fade's and Evandra's join XP** no longer adds a second catch-up under EET.
- **Companion stats**, each a separate choice: Rasaad Dexterity 17 and Constitution 16;
  Dorn Constitution 16; Eldoth Strength 17, Constitution 16, Wisdom 9 and Intelligence 14;
  Garrick Dexterity 17 and Intelligence 15; Quayle Constitution 15. Only the base scores
  change; class, kit, equipment and current hit points stay as they are.
- **Dorn's BG1 ambush:** the cutscene no longer locks the camera on Senjak; a completed
  conversation still ends in the fight.
- **Move NPCs to convenient locations** now also moves Ajantis to the Friendly Arm Inn.
- **Make Faldorn an Avenger** and **Teach Dynaheir Haste** now also install when only some
  of the files they change are present, and name the ones they skipped.

### Classes and kits

- **More weapon choices:**
  - Druids and every druid kit can use axes and short bows and put one point into them;
    fighter/druids can also use long bows, with up to two points in all three.
  - Yeslick's Alaghor of Clangeddin kit trains and uses the cleric weapons plus axes,
    Clangeddin's weapon, with up to two points. The thief weapons drop out; points already
    spent stay. This needs EEex and Artisan's Kitpack's weapon rules.
  - The Invisible Blade and Bardic Wonders' Dancer can use katanas and the rogue and bard
    items that kept them out by mistake, also those that later mods add, such as SCS's
    Drow Katana. The Shapeshifter and the Elementalist still cannot use them.
- **Kit weapon fixes:** sorcerers, the Dragon Disciple included, can equip daggers,
  quarterstaves, darts and slings again without proficiency points, like mages, and
  Eldritch Knight and Arcane Trickster sorcerers keep their kit weapons when they level up
  or join the party. Monks can equip quarterstaves with Monk
  Revisions, and shamans without a kit, Sacred Fists, Enlightened Fists and Ninjas are held
  to their weapons from the start. Existing games keep the old weapon behavior.
- **Kit weapon rules for weapons from later mods.** Artisan's Kitpack's weapon rules now
  also cover weapons that later mods add: the Priest of Tempus and the Eldritch Knight can
  use IWDification's two-handed axes, for example, and the Arcane Trickster sorcerer the
  SCS Drow Katana. Needs EEex.
- **White werewolves:** the Shapeshifter's white and greater white werewolves, the forms
  with the white fur, had no overhand-swing animation in five of the eight directions, and
  drawing that swing next to a wall could crash the game. In those directions the swing now
  shows the werewolf's other attack.
- **Druids level like clerics** is now **optional**. Druids keep their own XP table again:
  level 15 at 3,000,000 XP instead of 1,575,000.
- **Grandmastery:** High Mastery now gives +5 damage instead of +4, matching Grand
  Mastery. With Spell Revisions, the Black Blade of Disaster also gets +3/2 attacks per
  round and +8 to hit / +10 damage.
- **Bardic Wonders:** the Troubadour and Deathsinger spell copies and descriptions follow
  Spell Revisions, and recasting such a copy refreshes it instead of stacking. The Gallant
  gets the new bard spells, and Symphony of the Dark Children ends with its round of
  Invisibility again.
- **The Martyr paladin kit** now comes from Artisan's Kitpack alone and stays available at
  character creation, unchanged.

### Spells

- **True Time Stop – fair Haste & Slow:** in a Time Stop, the frozen creatures' spells and
  potions no longer run out while they cannot act, and their poison and regeneration wait
  too; everything continues afterwards with the time it had left. Haste and Slow no longer
  speed up or slow down poison, regeneration or other repeated effects; attacks, actions
  and movement stay hasted or slowed. Needs EEex. Install it before a playthrough; adding
  or removing it during one is not supported.
- **Chris Revisions of Aid, Luck, Bless, Chaotic Commands, Prayer and Recitation**, with
  or without Spell Revisions. Aid grants only hit points (1d8 +1 per caster level, at most
  +10) and replaces itself when cast again. Luck lasts 90 seconds. Bless lasts 30 seconds
  plus one round per caster level. Druids can cast Bless and Chaotic Commands again, and
  rangers Bless. Prayer and Recitation no longer stack: Recitation's bonus or penalty
  replaces Prayer's while it lasts. Exact copies, such as Bardic Wonders' Troubadour Aid,
  change with them.
- **Heroism effects don't combine.** The first Heroism, Greater Heroism or Potion of
  Heroism active on a creature wins. Song of Heroism, Aid and the Potion of Power stay
  separate.
- **Pierce Magic and Spellstrike texts:** their descriptions now follow Spell Revisions'
  layout. Pierce Magic halves the target's magic resistance for 5 rounds, by at least 10
  and at most 40 points, and removes one spell protection; Spellstrike removes all spell
  protections and gives a 15% chance of spell failure for 2 rounds. Their scrolls, the
  Wand of Spell Striking and the monk's Spellstrike show the same texts.
- **Scrolls at the reader's caster level:** they used to switch themselves off after a
  load, a new game, an NPC joining or opening a container, showed an EEex message box, and
  scrolls then cast at their normal level until the game was restarted. Notices now go to
  the game's loader log; only in one rare case, when the rule can neither start cleanly nor
  undo its start, does a single box ask you to restart the game.
- **Full immunities under the party resistance cap:** a short full immunity, such as SCS's
  Divine Protection, stays at 100% while it lasts; other party physical resistance stays
  capped at 90%.

### Items

- **Fine weapons instead of +1 weapons:** still +1 to hit and damage, but no initiative
  bonus, no harm to creatures immune to normal weapons, and about a third of the price.
  They never break in BG1's iron crisis. Fine ammunition, the same for +1 arrows, bolts,
  darts and sling bullets, is optional and not part of Challenge Mode; Protection from
  Normal Missiles stops it.
- **One potion dose of each type at a time:** drinking a potion type that is already
  active restarts its duration instead of stacking its bonus; potions that raise maximum
  hit points keep the first dose until it ends. Potions of Genius still stack, and
  different potion types still combine. The strict version, where Potions of Genius don't
  stack either, is optional and part of Challenge Mode.
- **No stealing from fences** (optional, part of Challenge Mode): fences can't be stolen
  from, and reputation no longer changes their prices. At fences this also replaces the
  store-price option you chose.
- **Bracers of Blinding Strike** and Spell Revisions' Haste or Improved Haste no longer
  combine when the spell is cast first.

### Quests and content

- **Optional quest mods** in the new Quest mods category: Northern Tales of the Sword
  Coast (new quests, areas and dungeons north of Baldur's Gate and along the Sword Coast)
  and Ascalon's Questpack (nine small BG1 quests and one for Shadows of Amn). **Black
  Hearts**, a BG1 and Siege of Dragonspear quest in which Illasera hires the party, is
  offered only with the SoD Remix unticked, because its Siege of Dragonspear part runs
  through areas the remix restructures.
- **More optional mods**, also third-party and experimental: Thalantyr Item Upgrade
  (Thalantyr in High Hedge improves BG1 items), the Icewind Dale Voice Pack (Icewind
  Dale's player voices) and Portraits Portraits Everywhere (dialogue portraits for speaking
  NPCs, about 0.4 GB).
- **18 player voicepacks** from ChrizFader's voicepacks, each a separate choice: Neera,
  Safana (BG1 and SoD), Jaheira (BG1 and BG2), Yeslick, Edwin (BG1 and BG2), Aerie, Imoen
  (BG1 and BG2), Sarevok, Amelyssan, Gromnir Il-Khan, Ajantis, Viconia (BG1 and BG2), and
  Lord Garithos from Warcraft III (IAIWE's soundset). Choose them as your character's voice
  in character creation; companions keep their own voices.
- **Unfinished Business** for BG1 and for BG2 is now **optional**. Every component stays
  available, unchecked in Chris's Recommended. With BG1 Unfinished Business and Northern
  Tales both ticked, BG1 Unfinished Business keeps the Nim Furlwing and Svlast encounters.
- **Campaign XP caps with banked XP** (optional, experimental) now work. With collection
  alpha.20 they refused every permanent XP award. Script variables also survive saving
  and loading with them.
- **Chapter-by-chapter XP caps** (optional, experimental, instead of the Shadows of Amn
  cap; needs the campaign XP caps): usable XP is capped per chapter, from 16,000 in BG1
  Chapters 1–2 up to 8,900,000 in Throne of Bhaal's last chapter, and XP above the cap is
  banked until a later chapter. With them, Baeloth's Larswood scene waits for Chapter 5.

### Interface and quality of life

- **BuffBot 1.12.6** (was 1.10.0):
  - The "Combat detected - casting stopped" and "casting timed out" notices now appear
    over the party leader's head; before, they were always lost.
  - Normal speed and your own buff order stay the defaults; "Order: Duration" casts a
    preset's longest-lasting buffs first. Buffs that are nearly over can be cast again,
    through the EEex options (off by default) or the panel's Active toggle. Also new: a
    search by buff name, and item powers that work through a spell found automatically.
  - The panel wears its theme's artwork and colours, and a button you press looks pressed.
    The default colour scheme looks as before and is now called "Baldur's Gate".
  - The F12 preset abilities are gone: right-click the HUD gear to pick a preset. BuffBot
    no longer adds text to the game's `dialog.tlk`.
  - Fixed: restoring temporarily equipped items no longer risks overwriting the first
    backpack slot. A buff run no longer speeds the game up while the options, the
    character record, a level-up, the journal, a shop or another mod's dialog is open. The
    Summons view lists only creatures your party summoned or cloned, and with a party of
    one to five the panel can switch from it back to the party again. A companion who
    joins while the panel is open gets a tab of their own. A configured spell variant no
    longer skips its casting time and aura, and wearable items are no longer refused to
    characters without a kit. On a small panel, captions no longer spill onto neighbouring
    buttons.
- **Summoned creatures clear the fog of war** (needs EEex), **Ribald's special stock is in
  his regular store**, one store instead of two, and **sensible entrance points** now work
  on EET too: travelling to the Bridge District from areas further south arrives at a new
  southern entrance (Tweaks Anthology).
- **EEex attack info** (optional): shows your chance to hit when the attack cursor is
  over a target. You can also switch it on or off later in EEex's options; the change
  takes effect after a restart.
- **Party movement speed outside combat** (optional) now uses the collection's 125%
  instead of the Tweaks Anthology's 200%.
- **Defensive Stance follows "Use special abilities".** With the option off in the AI
  settings, your main character and others on the default party AI no longer enter
  Defensive Stance on their own in BG1.
- **EEex Remote Console** (optional, experimental): the single-player menu no longer
  stays behind the game's screens after you return to the main menu.

### Compatibility and mod updates

- **EEex v1.3.1** (was v1.2.0): the EEex Options menu pauses the game and no longer opens
  during dialogs or cutscenes, and InfinityLoader no longer fails to start the game on
  some systems.
- **SoD Remix v0.6.16:** its victory ending installs alongside mods that add their own
  victory remark after the battle of Avernus, such as Sirene, and keeps the pause after
  each remark short. Its Boareskyr Bridge finale installs alongside "Summoned creatures
  clear the fog of war". Other parts no longer stop when a mod renames one of the game's
  timer lengths.
- **BG Rebalance:** the cloud forms, elemental arrows, Cleric of Tempus and Emotion spells
  keep other mods' changes instead of overwriting them, and a component that cannot apply
  a change correctly now stops the installation with the reason instead of guessing.
- **Spell Revisions:** with Rogue Rebalancing's encounters, Aran Linvail, the Zhentarim
  Mercenary and Selina prepare Hold Monster, Feeblemind and Domination instead of Chaos
  copies that had moved to level 7, and the Lightning Bolt choice keeps damage that
  another mod adds instead of stopping the installation.
- **Tweaks Anthology:** the SoD Death Cam no longer hangs in two cutscenes, mages can
  decline the Sphere stronghold, and familiars no longer get the hit points meant for
  party members.

### Mod versions

- Chriz BG Modpack 0.2.0-alpha.14 (was alpha.11)
- BG Rebalance 0.8.9 (was 0.8.2)
- SoD Remix 0.6.16 (was 0.6.13)
- BuffBot 1.12.6 (was 1.10.0)
- EEex 1.3.1 (was 1.2.0)
- Bardic Wonders 2.9c-balance.8 (was balance.7)
- Spell Revisions fork 4.21-chriz.7 (was chriz.5)
- Artisan's Kitpack fork 4.81a-dev.85928f9-chriz.6 (was chriz.1)
- EEex Remote Console 0.2.1 (was 0.2.0)
- The Tweaks Anthology: development version `7649ced`, 134 changes after v18 (was v18)
- New: ChrizFader's voicepacks 1.1.0, Sirene NPC for BG:EE (development version
  `1c86000`), Northern Tales of the Sword Coast (development version `f0ef953`),
  Ascalon's Questpack 7.0, Thalantyr Item Upgrade 4.5.1, Icewind Dale Voice Pack 1.2.0,
  Portraits Portraits Everywhere (DaftHunk's fork `417226e`) and Black Hearts 2.05

## Known issues

- Several new parts passed automated checks or their own lab tests but have not been
  played through in a full game yet, among them True Time Stop, Chris Revisions and the
  weapon changes. Saving, dispelling, curing or dying during a Time Stop are not covered
  yet.
- The quest mods, Portraits Portraits Everywhere, Thalantyr Item Upgrade, the Icewind
  Dale Voice Pack and Black Hearts are third-party mods, offered as they are and not yet
  tested in a combined installation. The campaign XP caps stay experimental; the chapter
  caps have not been played through the story yet.
- The Updates page does not list the collection's changes yet; these notes describe them.
- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.0_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- **The new mod setup** comes with a new installation: **Create updated installation** on
  the Updates page, or **New installation**. It is a separate game for your next
  playthrough; your current installation stays as it is.
- Unfinished Business (BG1 and BG2) and the Tweaks Anthology option that makes druids
  level like clerics are now unticked in Chris's Recommended; tick them in **Customize** if
  you want them.
- Collection 0.1.0 needs app 0.1.0. Older apps cannot install it.
- Existing installations keep EEex v1.2.0 and their other mod versions; the new versions
  come with a new installation.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

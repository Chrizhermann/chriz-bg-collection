# Next release — changelog drafts

14 October 2026. Drafts for the release after app alpha.21 / collection alpha.20,
on branch `claude/cebg-installer-fixes-20261014`. Nothing here is released.
Labels: **Mandatory** = always installed with its mod; **Default** = selected in
Chris's Recommended; **Optional** = offered, unchecked; **Pending** = waiting for
Christopher's choice. Version the next release properly; never republish over
alpha.21. The [alpha.20 drafts](2026-10-13-next-release-changelogs.md) are already
published.

## Installer changelog

- **Uninstalling with "Delete the application data" removes it completely and
  safely.** The uninstaller used to delete CEBG's folders in AppData in a way
  that left files with very long paths behind (such as unpacked mods) and could
  follow a folder link to delete files outside CEBG. CEBG now removes its own
  application data itself. If something is left, the uninstaller names the
  folders instead of guessing.
- **Interrupted installations stay resumable after the download folder was
  deleted.** They used to show "folder moved or changed". CEBG now recreates the
  folder and downloads what is missing.
- **Stop works while another program holds a download.** A download or unpack
  that waited for another CEBG process used to ignore Stop.
- **Removing an installation finishes** when its removal record cannot be
  cleaned up because another program holds it open. The installation used to
  stay listed as if removal had failed.
- **Game fixes, the BuffBot update and removing an installation can no longer be
  redirected by a folder link.** For these, including their Undo and backup
  restore, CEBG now writes and deletes the game's files only through the game
  folder it holds open, so a link swapped in meanwhile cannot lead it to files
  outside the installation. While an installation is being removed, another
  program can also no longer move a file out of it and have CEBG delete it there.
- **A step whose WeiDU copy could not be prepared can be retried.** It used to
  require a fresh copy of the games.
- **Challenge Mode can pick an option of a choice.** Switching it on now also
  picks the strict potion rule instead of the recommended one, and switching it
  off restores the recommended rule. The Challenge Mode card lists it with the
  other changes, and the choice under its mod shows the Challenge Mode mark.
- **EET's desktop shortcut and EET's 64-bit tools work in every environment.**
  If Windows had `NoDefaultCurrentDirectoryInExePath` set, EET's shortcut
  component failed and EET's core quietly used its 32-bit tools. Mod installers
  no longer inherit that setting.
- **Large downloads on slower connections no longer fail after 30 seconds.**
  Every download attempt used to stop 30 seconds after it started, even while
  data was still arriving, so EET or Infinity UI could not be downloaded below
  roughly 60 Mbit/s. A download is now retried only when it actually stalls: when
  no data has arrived for a minute.
- **The game can write its own log for crash reports.** Open **Troubleshooting**
  at the end of an installation on the home screen and check **Write a game log
  for crash reports**. It works for existing installations without reinstalling,
  takes effect at the next game start and turns off the same way; close the game
  before changing it. The game starts a new log each time, so **Play** first keeps
  the last session's log as `InfinityLoader-previous.log`. The section shows where
  the log, that copy and the game's crash reports are, and which files to send.
- **Unticking an optional mod no longer adds setup notes for its components.**
  Unticking Hidden Gameplay Options, for example, added one setup note per default
  component, each saying "Available when Hidden Gameplay Options is selected." A
  default that is unavailable while its mod is ticked still gets its note.

## Collection changelog

The new mod setup is for fresh installations. Existing games and saves are
unchanged.

### New choices

- **Companion stats** — **Default**, each separately: Rasaad Dexterity 17 and
  Constitution 16; Dorn Constitution 16; Eldoth Strength 17, Constitution 16,
  Wisdom 9 and Intelligence 14; Garrick Dexterity 17 and Intelligence 15; Quayle
  Constitution 15. Only the base scores change; class, kit, equipment and current
  hit points stay as they are. (Modpack alpha.12)
- **Join XP for every joinable NPC** — **Default.** NPCs whose scripts lack EET's
  join-XP catch-up, mostly from mods, get EET's own: on joining in SoA they rise
  to the highest protagonist XP threshold reached, from 125,000 up to 1,250,000,
  and in ToB to 2,500,000. Sarah, for example, used to join at 89,000 XP. Never
  lowers XP and doesn't repeat when an NPC rejoins. (Modpack alpha.13)
- **Fine weapons instead of +1 weapons; they never break** — **Default.** Most
  standard +1 weapons become nonmagical "fine" weapons: still +1 to hit and
  damage, but no initiative bonus, no harm to creatures immune to normal weapons,
  and about a third of the price. They never break in BG1's iron crisis. (SCS)
- **Fine ammunition instead of +1 ammunition** — **Optional**, not part of
  Challenge Mode. +1 arrows, bolts, darts and sling bullets become "fine"
  ammunition: +1 to hit and damage, but no harm to creatures immune to normal
  weapons, and Protection from Normal Missiles stops them. (SCS)
- **Potions: one dose of each type at a time** — **Default** with Potions of
  Genius still stacking. Drinking a potion type that is already active restarts
  its duration instead of stacking its bonus; Heroism, Power and other potions
  that raise maximum hit points keep the first dose until it ends. Different
  potion types still combine. The strict version, where Genius does not stack
  either, is **Optional** and part of **Challenge Mode**: switching Challenge
  Mode on picks it instead, and switching it off restores the default. (Rebalance
  v0.8.3)
- **Heroism effects don't combine** — **Default.** Heroism, Greater Heroism
  (Bardic Wonders), their copies and the Potion of Heroism: the first one active
  on a creature wins. Song of Heroism, Aid and the Potion of Power stay
  separate. (Rebalance v0.8.3)
- **Chris Revisions: Aid, Luck, Bless, Chaotic Commands, Prayer and Recitation** —
  **Default.** Christopher's own spell balance changes, with or without Spell
  Revisions. Aid grants only hit points, 1d8 +1 per caster level (at most +10), and
  casting it again replaces the bonus. Luck lasts 90 seconds. Bless lasts 30
  seconds plus 1 round per caster level (36 seconds at level 1, 150 from level 20).
  Druids can cast Bless and Chaotic Commands again, and rangers Bless. Prayer and
  Recitation no longer stack: Recitation's bonus or penalty replaces Prayer's while
  it lasts. Exact copies, such as Bardic Wonders' Troubadour Aid, change with them.
  (Rebalance v0.8.5)
- **Time Stop stops everything but the caster; Haste and Slow only change actions and
  movement** — **Default.** In a Time Stop, the frozen creatures' spells and potions no
  longer run out while they cannot act, and their poison and regeneration wait too;
  everything continues afterwards with the time it had left. Haste and Slow no longer
  speed up or slow down poison, regeneration and other repeated effects; attacks,
  actions and movement stay hasted or slowed. It applies to new installations only:
  install it before a playthrough, because adding or removing it during one is not
  supported. Saving, dispelling, curing or dying during a Time Stop are not covered yet.
  (Rebalance v0.8.6)
- **EEex attack info** — **Optional.** Shows your chance to hit when the attack
  cursor is over a target. You can also switch it on or off later in EEex's
  options; the change takes effect after a restart. (EEex v1.3.1)
- **Summoned creatures clear the fog of war** — **Default.** (Tweaks Anthology)
- **Ribald's special stock in his regular store** — **Default.** One store
  instead of two. (Tweaks Anthology)
- **Sensible entrance points** — **Default.** Now available on EET. Travelling
  to the Bridge District from areas further south now arrives at a new southern
  entrance. (Tweaks Anthology)
- **No stealing from fences** — **Optional**, part of **Challenge Mode**: you
  can no longer steal from fences, and reputation no longer changes their
  prices. At fences this also replaces the chosen store-price option. (Tweaks
  Anthology)
- **Voicepacks** — **Default**, each separately: 18 player voicepacks from
  ChrizFader's voicepacks v1.1.0, made from the characters' original game audio:
  Neera, Safana (BG1 and SoD), Jaheira (BG1 and BG2), Yeslick, Edwin (BG1 and
  BG2), Aerie, Imoen (BG1 and BG2), Sarevok, Amelyssan, Gromnir Il-Khan, Ajantis,
  Viconia (BG1 and BG2), and Lord Garithos from Warcraft III (IAIWE's soundset).
  Choose them as your character's voice in character creation. They add voices
  only; companions keep their own.
- **Imoen in Shadows of Amn** — choose one (EET). **Default:** she becomes the
  BG2 thief/mage dual-class Imoen plus the tome boosts she read in BG1 and SoD.
  **Optional:** she continues as taken along, or (with EEex) she dual-classes to
  mage at her end-of-SoD thief level and Shadows of Amn's catch-up XP goes into
  mage. With every choice her SoD gear is handed over like the party's, and she
  keeps only her BG2 belt, correctly equipped; she no longer reaches Shadows of
  Amn with all her gear and her belt in the magic-weapon slot. Raise a dead
  Imoen before ending SoD. (SoD Remix v0.6.14)
- **Quest mods** — **Optional**, experimental, each separately, in the new "Quest
  mods" category: Northern Tales of the Sword Coast (new quests, areas and dungeons
  north of Baldur's Gate and along the Sword Coast) and Ascalon's Questpack (nine
  small BG1 quests and one for Shadows of Amn). Also new, optional and experimental,
  in the existing categories: Thalantyr Item Upgrade (Thalantyr in High Hedge
  improves BG1 items) in "Items", the Icewind Dale Voice Pack (Icewind Dale's player
  voices) in "Audio" and Portraits Portraits Everywhere (dialogue portraits for
  speaking NPCs, about 0.4 GB) in "Visuals". They are third-party mods, offered as
  they are and not yet tested in a combined installation.
  - **Pending:** which of their components start selected when you tick one.
    Christopher decides.
- **Black Hearts** — **Optional**, experimental, third-party: a BG1 and Siege of
  Dragonspear quest in which Illasera hires the party. It can't be combined with the
  SoD Remix, because its Siege of Dragonspear part runs through areas the remix
  restructures, so the installer offers it only with the SoD Remix unticked. (Black
  Hearts v2.05)
- **Dorn's BG1 ambush no longer crashes the game** — **Default.** When Dorn catches
  up with Senjak and Dorotea, the cutscene no longer locks the camera on Senjak. If
  their conversation stalled, leaving the area crashed the game; a completed
  conversation still ends in the fight. (Modpack alpha.13)
- **Defensive Stance follows "Use special abilities"** — **Default.** With the option
  off in the AI settings, your main character and others on the default party AI no
  longer enter Defensive Stance on their own in BG1. (Modpack alpha.13)
- **Chapter-by-chapter XP caps** — **Optional**, experimental, instead of the Shadows of
  Amn cap; needs "Campaign XP caps with banked XP". Usable XP is capped per chapter, from
  16,000 in BG1 Chapters 1-2 up to 8,900,000 in ToB's last chapter, and XP above the cap
  is banked until a later chapter raises it. Baeloth's Larswood scene waits for Chapter 5,
  so his starting XP fits under the cap. (Modpack alpha.13)

### Changed behavior

- **Unfinished Business (BG2)** — now **Optional** (was **Default**). All 22
  offered components remain available in the installer, unchecked by default in
  Chris's Recommended.
- **BG1 Unfinished Business** — now **Optional** (was **Default**), like Unfinished
  Business for BG2. All 16 offered components remain available in the installer,
  unchecked by default in Chris's Recommended. Its Nim Furlwing and Svlast encounters
  and the versions in Northern Tales of the Sword Coast are either-or; with both mods
  ticked, BG1 Unfinished Business keeps them.
- **Druids level like clerics** (Alter Druid Spell and Level Progression Tables) —
  now **Optional** (was **Default**), unchecked in Chris's Recommended. Druids keep
  their own XP table: level 15 at 3,000,000 XP instead of 1,575,000. (Tweaks Anthology)
- **Grandmastery** (Default, unchanged choice): High Mastery now gives +5
  damage instead of +4, matching Grand Mastery. To-hit and the other levels are
  unchanged. (Tweaks Anthology)
  - **Pending:** with Spell Revisions, the Black Blade of Disaster now also gets
    +3/2 attacks per round and +8 to hit / +10 damage (it was +5/+5).
    Christopher decides whether that stays. (Upstream's fix for Black Blade
    attacks stacking with grandmastery changes nothing here: v18 never had that
    bug.)
- **Move NPCs to convenient locations** (Default) now also moves **Ajantis**
  to the Friendly Arm Inn, along with Alora, Eldoth, Quayle, Shar-Teel, Tiax and
  Viconia. (Tweaks Anthology)
- **Tweaks Anthology fixes** in the new version, for components the collection
  installs:
  - The SoD Death Cam no longer hangs in two cutscenes.
  - Mages can decline the Sphere stronghold (Multiple Strongholds).
  - Familiars no longer get the boosted hit points meant for party members.
  - The kit ability table change that SCS reads is labelled correctly.
- **Modpack** (since alpha.12) also installs companion fixes `196`/`197` when only
  some of the files they patch are present, and says which ones it skipped.
- **Campaign XP caps with banked XP** (Optional, experimental, unchanged choice)
  now work, on EEex v1.3.1 too. With Modpack alpha.11 and alpha.12 no character
  got a bank account, so with the caps installed every permanent XP award was
  refused. This includes the optional SoA cap. (Modpack alpha.13)
  - Script variables survive saving and loading with the bank again: up to alpha.13,
    an XP change the bank could not take held up that character's other effects, and
    variables set behind it were lost at the next save. If the bank cannot start, XP
    stays uncapped and unbanked; the reason goes to the game's loader log, not into the
    game. (Modpack alpha.14)
- **Scrolls at the reader's caster level** (Default, unchanged choice) stay on for the
  whole session. Modpack alpha.11 to alpha.13 turned them off after a load, a new game,
  an NPC joining or opening a container, showed an EEex message box, and scrolls then
  cast at their normal level until the game was restarted. Technical notices now go to
  the loader log; only in one rare case, when the scroll rule cannot start cleanly and
  cannot undo its start, does one box ask you to restart the game as it starts.
  (Modpack alpha.14)
- **Fade and Evandra's SoA join XP** (Default) recognise a catch-up an NPC
  already has under EET, so they no longer add a second one. (Modpack alpha.13)
- **SoD Remix's Boareskyr Bridge finale** (Mandatory) installs alongside
  "Summoned creatures clear the fog of war" (Default). Chris's Recommended used to
  stop there; the hostile bridge elementals now leave that ally-only effect out.
  (SoD Remix v0.6.15)
- **Bardic Wonders fixes** (Default components; Bardic Wonders v2.9c-balance.8):
  - The Troubadour, Deathsinger and New Bard Spells copies of priest spells are
    now found by the spell's name, so they follow Spell Revisions. Recasting one
    of these copies refreshes it instead of stacking with the earlier cast.
  - The Troubadour and Deathsinger descriptions name the spells the kits
    actually get, as Spell Revisions and the game's language call them.
  - The Gallant now gets the new bard spells. In the collection's install order
    it got none of them.
  - Symphony of the Dark Children (Abettor of Mask): the party turns invisible
    again when the song ends. Before, nothing happened at the end, and a
    disabled singer gave Invisibility every round.
- **EEex v1.3.1** (Mandatory; was v1.2.0): the EEex Options menu pauses the game
  while it is open and no longer opens during dialogs or cutscenes, and
  InfinityLoader no longer fails to start the game on some systems. EEex's new
  attack info display (to-hit chances when the attack cursor is over a target) is
  offered as an optional checkbox (see New choices) and can also be turned on in
  EEex's options.
- **BuffBot** (Default): the "Combat detected - casting stopped" and "casting timed
  out" notices now appear over the party leader's head. Before, both notices were
  always lost; stopping the buff run itself worked. (BuffBot v1.10.1)
  - New: an optional 2x or 4x speed while a buff run lasts (Normal stays the
    default), a search by buff name, a named preset picker when you right-click
    the HUD gear, item powers that work through a spell found automatically, and
    the enable and lock cells drawn as artwork. (BuffBot v1.11.0)
  - Fixed: a configured spell variant no longer skips its casting time and aura;
    wearable items are no longer refused to characters without a kit; and
    restoring temporarily equipped items no longer risks overwriting the first
    backpack slot on BG2:EE 2.7.3.0, which could corrupt the character's name and
    crash the game. (BuffBot v1.11.0)
- **BG Rebalance fixes** for components the collection installs (Rebalance v0.8.4):
  - Bracers of Blinding Strike and Spell Revisions' Haste or Improved Haste no
    longer combine when the spell is cast first.
  - Party physical resistance: a short full immunity, such as SCS's Divine
    Protection, stays at 100% for as long as it lasts. Everything else stays
    capped at 90%.
  - Combat cloud forms: only immunities of 100% or more become 75% resistance;
    lower values from other mods stay as they are.
  - Elemental arrows: only the arrow's bow-fired hit changes; other mods' effects
    on the arrows stay.
  - The Cleric of Tempus rework finds what it changes by what it does, so it works
    with Artisan's Kitpack and other layouts, and keeps other mods' visuals, sounds
    and markers.
  - The optional Emotion spells accept other mods' casting times, ranges, schools
    and levels, and their descriptions state the spell's own values.
- **Spell Revisions fixes** (Spell Revisions fork v4.21-chriz.6):
  - With Rogue Rebalancing's encounters, Aran Linvail, the Zhentarim Mercenary and
    Selina prepare Hold Monster, Feeblemind and Domination in place of Rogue
    Rebalancing's level-5 Chaos. Before, the Chaos copies were moved to level 7,
    where Aran and the Mercenary could never cast them.
  - The Lightning Bolt choice keeps damage another mod adds to the bolt instead
    of stopping the installation.
- **Sorcerers can equip daggers, quarterstaves, darts and slings again** without
  proficiency points, like mages. This affects every sorcerer kit, the Dragon
  Disciple included. (Artisan's Kitpack fork chriz.2)
- **Eldritch Knight and Arcane Trickster sorcerers no longer lose their kit weapons
  when they level up or join the party.** (Artisan's Kitpack fork chriz.3)
- **Monks can equip quarterstaves with Monk Revisions** (Default), and **shamans without
  a kit, Sacred Fists, Enlightened Fists and Ninjas are held to their weapons from the
  start.** Before, a monk without quarterstaff proficiency could not equip a quarterstaff,
  and these characters could equip any weapon after joining the party until the game
  next loaded or entered an area; their level-ups did not apply the weapon limits either.
  (Artisan's Kitpack fork chriz.4)
  - Existing games keep the old sorcerer, Eldritch Knight, Arcane Trickster, monk and
    shaman weapon behavior until you install again.
- **EEex Remote Console** (Optional, experimental): after you return to the main menu
  from character creation, the load screen or the options, the single-player menu no
  longer stays behind the game's screens for the rest of the session. Its installer
  messages now name the EEex v1.2/v1.3 components it needs. (Remote Console v0.2.1)

### Mod versions

- Chriz BG Modpack v0.2.0-alpha.14 (was alpha.11)
- BG Rebalance v0.8.6 (was v0.8.2)
- Bardic Wonders v2.9c-balance.8 (was balance.7)
- EEex v1.3.1 (was v1.2.0)
- The Tweaks Anthology: development version `7649ced`, 134 changes after v18
  (was v18). There is no newer release yet; the collection pins this exact
  version, as it does for EET.
- ChrizFader's voicepacks v1.1.0 (new)
- SoD Remix v0.6.15 (was v0.6.13)
- BuffBot v1.11.0 (was v1.10.0)
- EEex Remote Console v0.2.1 (was v0.2.0)
- Spell Revisions fork v4.21-chriz.7 (was v4.21-chriz.5)
- The Artisan's Kitpack fork v4.81a-dev.85928f9-chriz.4 (was chriz.1)
- Northern Tales of the Sword Coast: development version `f0ef953`, 35 changes after
  v5.0.0 (new). The collection pins this exact version because v5.0.0 lacks a fix
  that SCS needs and the Island of the Sirines on the worldmap.
- Ascalon's Questpack v7.0 (new)
- Thalantyr Item Upgrade v4.5.1 (new)
- Icewind Dale Voice Pack 1.2.0 (new)
- Portraits Portraits Everywhere: DaftHunk's fork at `417226e` (its readme says
  v1.03) (new)
- Black Hearts v2.05 (new)

## Release-note entry (collection alpha.21)

**Collection alpha.21** updates Chriz BG Modpack to v0.2.0-alpha.14, BG
Rebalance to v0.8.6, Bardic Wonders to v2.9c-balance.8, SoD Remix to v0.6.15,
EEex to v1.3.1, BuffBot to v1.11.0, EEex Remote Console to v0.2.1, the Spell
Revisions fork to v4.21-chriz.7, the Artisan's Kitpack fork to
v4.81a-dev.85928f9-chriz.4 and The Tweaks Anthology to its current development
version. New in Chris's Recommended: Christopher's revisions of Aid, Luck, Bless,
Chaotic Commands, Prayer and Recitation; Time Stop that stops everything but the
caster, and Haste and Slow that no longer change poison and regeneration; companion
stat adjustments for Rasaad,
Dorn, Eldoth, Garrick and Quayle; EET's join XP for every joinable NPC that
lacked it; a fix for the crash after Dorn's BG1 ambush; party AI that uses Defensive
Stance only with "Use special abilities" on; SCS fine weapons that never break; one potion
dose of each type at a time (Potions of Genius still stack); Heroism effects
that don't combine; summoned creatures that clear the fog of war; Ribald's
special stock in his regular store; sensible entrance points; 18 player
voicepacks from ChrizFader's voicepacks; and a choice of how Imoen continues
into Shadows of Amn, by default as the BG2 thief/mage Imoen. Challenge
Mode adds that fences can't be stolen from and ignore reputation, and picks the
strict potion rule where Potions of Genius don't stack either. The optional,
experimental campaign XP caps with banked XP now work and keep script variables across
saving and loading, with chapter-by-chapter caps as a new option, and scrolls at the
reader's caster level now stay on for the whole session. New and optional, all
experimental: two third-party quest mods in a new "Quest mods" category (Northern
Tales of the Sword Coast and Ascalon's Questpack), Thalantyr Item Upgrade, the
Icewind Dale Voice Pack and Portraits Portraits Everywhere, and Black Hearts for
installations without the SoD Remix. Unfinished Business for BG1 and BG2 is now
optional, and druids level on their own XP table again instead of the clerics';
High Mastery now gives +5 damage, and the NPC move now includes Ajantis. In Bardic Wonders, the
Troubadour and Deathsinger spell copies and descriptions follow Spell Revisions,
the Gallant gets the new bard spells, and Symphony of the Dark Children ends
with its round of Invisibility. BuffBot's combat-stop notices now appear over the
party leader, BuffBot can speed up a buff run to 2x or 4x and no longer risks a crash
when it puts equipped items back, and the optional EEex Remote Console no longer leaves the
single-player menu behind the game's screens. Sorcerers, the Dragon Disciple
included, can equip daggers, quarterstaves, darts and slings again, and Eldritch
Knight and Arcane Trickster sorcerers keep their kit weapons when they level up or
join the party. Monks can equip quarterstaves with Monk Revisions, and shamans without a
kit, Sacred Fists, Enlightened Fists and Ninjas are held to their weapons from the start.
EEex's attack info display is offered as an option. These changes
apply to new installations; existing games and saves are unchanged.

## Notes for the release integrator

- **This recipe needs the new app.** Challenge Mode's potion choice uses the new
  feature-group `variants` field, and older apps reject unknown recipe fields.
  The release that ships this recipe must set `minimum_app_version` to the app
  version that adds variants.
- **Bardic Wonders v2.9c-balance.8** has not been played in a game yet (its
  release notes). Its standalone Abettor tail 1.2.0 is for existing
  playthroughs; the collection installs component 2004 fresh and doesn't use it.
- **Modpack alpha.14** fixes `430` (scrolls at the reader's caster level, Default) on
  EEex v1.3.1, which alpha.11 to alpha.13 turned off for the session, and the script
  variables `650` lost ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19)).
  Its owner labs ran on disposable 2.7.3.0 installs with EEex v1.3.1, the last one with the
  release candidate's package on a copy of the Recommended install; no ongoing
  playthrough has run with `430` yet, so the Recommended lab run checks the loader log,
  together with BuffBot v1.11.0 (see `FOLLOW_UPS.md`).
  `135` and `460` were checked in game for alpha.13 on a copy of the Recommended install.
  Chapter caps `652` stay experimental: their lab set chapters and gave XP from the
  console, not through the story (the Modpack's banking guide).
- **SoD Remix v0.6.15** fixes the bridge finale `256` with Tweaks Anthology `3500`
  on; the owner lab reinstalled the remix tail on a copy of the Recommended stack.
  This release still needs one combined lab install of the whole Recommended
  selection with `3500` on (see `FOLLOW_UPS.md`). The Imoen choice (v0.6.14) passed
  the owner lab for all three options; thief kits other than `C0TRICKS` and games
  without Spell Revisions were not covered there.
- **Voicepacks** (18 defaults, release v1.1.0) passed per-pack fixture installs
  with the pinned WeiDU only; this release needs one combined lab install with the
  full Recommended selection first (see `FOLLOW_UPS.md`).
- **EEex v1.3.1** reaches new installations only. EEex installs early, so moving an
  existing installation means reinstalling everything after it; how existing
  games move is Christopher's call (see `FOLLOW_UPS.md`).
- **BuffBot v1.10.1** changes only BuffBot's Lua code; its components and TP2 are
  unchanged apart from the version. Its lab check on EET 2.7.3.0 with EEex v1.3.1
  made the engine attach the notice over the party leader; a real combat stop with
  the notice on screen, EEex v1.2.0 and BG:EE or BG2:EE outside EET were not checked
  (its release notes).
- **BuffBot v1.11.0** changes BuffBot's Lua code and adds control artwork (the
  `buffbot/visual` BAM files with PVRZ pages `MOS88809`–`MOS88993`); its components
  and their order are unchanged. Its owner labs on 2.7.3.0 with EEex v1.3.1 passed
  Normal, 2x and 4x, but its EET/SR/SCS/Modpack matrix ran with Modpack `430`
  uninstalled: its notes call the configuration with `430`, which Chris's
  Recommended installs, not accepted yet. The column headers and the artwork were
  not checked in game, and nobody checked the new PVRZ page numbers against the
  interface mods' pages.
- **Rebalance `340`** (Chris Revisions, Default) was installed on scratch copies of
  game resources only and has not run in a game yet (its release notes). Its "ready"
  flag, like EEex `9`'s, stays as it is until the Recommended lab run decides
  (Christopher, 2026-10-26). Since v0.8.4,
  Rebalance components that cannot apply a change correctly stop the installation
  with the reason instead of guessing.
- **Rebalance `360`** (Time Stop and Haste effect clocks, Default; Christopher,
  2026-10-27) needs EEex installed before it, `data/PATCH27.BIF` and the 64-bit
  BG2:EE 2.7.3.0 executable (PE timestamp `0x6a18d73d`). CEBG's only accepted
  `Baldur.exe` has that timestamp, and EEex runs before Rebalance in both recipes.
  CEBG's game check does not fingerprint `data/PATCH27.BIF`. Its owner labs passed on
  BG2:EE 2.7.3.0 and in the full CEBG stack (EET, Recommended, Spell Revisions), both
  with EEex's LuaJIT option; it has not been accepted in real gameplay yet (its release
  notes). Saving, dispel, cure and death during a stop are not covered. It is marked
  "ready" like `340`, without a recorded readiness decision of its own.
- **Spell Revisions fork v4.21-chriz.7**: the Rogue Rebalancing level-5 replacements
  (v4.21-chriz.6) have not been checked in game yet (its release notes).
- **Artisan's Kitpack fork chriz.4** (with chriz.2's sorcerer fix and chriz.3's kit
  weapon fix) reaches new installations only: existing installations get the weapon
  fixes only through a new installation (Christopher, 2026-10-26 and 2026-10-27), as the
  collection changelog says. The fork's `AKCB_SORCERER_WEAPONS` tail is not part of the
  collection, and the kit, monk and shaman weapon fixes have no tail; in existing games
  the helper creature gives the weapons back as before. chriz.3's lab ran on BG2:EE
  2.7.3.0, not EET, with a fresh install in the collection's Kitpack order. chriz.4's labs
  ran on three disposable BG2:EE 2.7.3.0 games with EEex 1.3.1: chriz.3 and chriz.4 in
  CEBG's order, and chriz.4 in another order without Spell Revisions. Not covered there:
  EET, kits other than the Favored Soul and the Ninja, and characters created before the
  installation (its release notes; see `FOLLOW_UPS.md`).
- **EEex Remote Console v0.2.1** is a tag without GitHub release assets, like
  v0.2.0, so the pin is GitHub's commit archive of the tag commit. Its `START_SP` fix
  passed the README checklist on EET 2.7.3.0 with EEex v1.3.1 in the October 18 lab.
- **The game-log switch** is tested against synthetic game folders, against
  Windows' own reading of the loader settings, and against copies of two real
  loader settings files, as EEex v1.3.1 ships it and from a played game; no game
  was started with it. Check it once in a lab before the release (see
  `FOLLOW_UPS.md`).
- **The six new third-party mods** (the quest mods, Thalantyr Item Upgrade, the
  Icewind Dale Voice Pack and PPE; optional and experimental) and the now optional
  BG1 Unfinished Business have static recipe checks only; nothing was installed.
  They stay experimental until one combined lab install with the full Recommended
  selection and these mods ticked (see `FOLLOW_UPS.md`). NTotSC and
  PPE are GitHub commit archives without a release; PPE is a 387 MB download.
- **Default components under an unticked optional mod** are no longer reported as
  omitted defaults, neither in the installer's setup notes nor in release
  validation (see the installer changelog). The new mods' main components rely
  on this: without it, Chris's Recommended would show five setup notes and need
  omission records for them. A default that drops out while its optional parent is
  ticked is still reported.

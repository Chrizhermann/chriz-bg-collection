# Chriz Easy IWD

**Early Windows release. Expect bugs.** A few-click installer for ChrizFader's curated
Icewind Dale: Enhanced Edition setup, the one played on stream. Player guide:
[bg.chrizfader.org/iwd](https://bg.chrizfader.org/iwd).

Chriz Easy IWD copies your own clean Icewind Dale: Enhanced Edition into a new folder and
installs the recommended mods there: 28 mods with 126 components, each downloaded from its
official source and checked against the exact tested version. Your original game stays
unchanged. Installer packages and their SHA-256 checksums are published under
[Releases](https://github.com/Chrizhermann/chriz-easy-iwd/releases).

## What you need

- Windows 10 or 11, 64-bit, and an internet connection during installation.
- A clean, unmodded Icewind Dale: Enhanced Edition 2.7.3.0. Tested with the English Steam
  version; mods are installed in English.
- About 8 GB of free space: the modded copy takes about 4–5 GB, downloaded mods about 2 GB.

## Quick install guide

1. Download and open Chriz Easy IWD. Close the game and any save editors first.
2. Check the detected game folder, or use **Change source**. Pick an install name and a new,
   empty install location, separate from your game, your Steam library and the download cache.
3. Keep the recommended setup, or choose **Customize** to pick an interface, add quests and
   review optional components. Manual-mod pauses are for experienced modders only.
4. Choose **Install Chriz Easy IWD** and keep the computer awake. Need a break? Choose
   **Pause after current mod**; continue later from **My installs**.
5. When it is done, choose **Play Chriz Easy IWD**. The modded copy has its own game profile,
   so its saves stay apart from your original game's.

## What's different from vanilla

This describes the recommended setup. Optional choices are listed separately below.

- **Smarter fights.** Sword Coast Stratagems gives enemies better tactics: general AI and calls
  for help, mages, priests, dragons, beholders, fiends and celestials, and revised IWD undead.
  Enemies drink potions, elementals and summoning are revised, and 9 arcane and 6 divine spells
  are added. Chriz IWD Encounters makes sure the IWD:EE casters SCS would otherwise miss fight as
  intended, and enemy priests prepare when they first see your party (Core Rules difficulty and
  up). All creatures have maximum hit points, and spells from high-level casters are harder to
  save against.
- **17 companions** by The Artisan, Arvendor and Lava, with banter between them from the
  Icewind Dale Crossmod Banter Pack (at most one every 30 minutes). Companions react more
  often, answer your main character instead of whoever talks to them, and may grumble about
  reputation but never leave over it.
- **Restored content.** Unfinished Business adds Marketh's Ring, real shadows in the Vale of
  Shadows and the Wardstone for Lower Dorn's Deep.
- **Characters.** Maximum hit points at level 1 and on every level-up; un-nerfed spell
  progression for mages, sorcerers, bards and clerics; true grandmastery; kits without racial
  restrictions; mages may use bucklers and thieves small shields; dual- and multi-class thieves
  backstab with more weapons; armor plus one protection item; scrolls cast at the reader's
  level; no experience cap. Thieves assign skill points in steps of five; shapeshifting is
  improved; spell sequencers and contingencies become innate abilities. Druids keep their own
  experience table and use 3rd edition alignment rules.
- **Spells and magic items.** All of SCS's spell changes, including Baldur's Gate-style Insect
  Plague and Creeping Doom and Mordenkainen's Sword in both versions. Wizard scrolls are easier to
  find; potions need identifying, and shopkeepers identify only some items.
- **Quality of life.** Bubb's Spell Menu Extended for quick spell and ability selection. BuffBot
  (F11) casts the buffs you choose for your party. EEex adds hold-Left-Shift spell overviews on
  creatures, grey empty containers, cooldown and contingency timers and UI scaling. Bags of
  Holding are bottomless; ammo, gems, jewelry, potions and scrolls stack without limit; stores
  sell bigger stacks. A rotating autosave every 15 minutes, save names with your character's
  name, party AI off in new games, summons that clear the fog of war, and dialogue that always
  pauses the game. Hidden Gameplay Options adds about two dozen hidden engine settings to the
  in-game options, such as area-of-effect range, enhanced path search, frame rate and
  learnable-spell markers.
- **Looks.** The Icewind Dale Graphical Overhaul's higher-quality area art, improved and unique
  item icons, weapon animation tweaks and separate fire and cold resistance icons.

### Optional, in Customize

- **Interface:** Lefreut's Enhanced UI for IWD:EE or Infinity UI++, or the game's own. Next to
  either, a few Hidden Gameplay Options they do not support are left out automatically.
- **Quests by Lava:** Below & Below Inn, Night of the Blinking Dead, Snowytoes Hamlet, Tale of
  Our Lady Dreamless, Terror of the Skineater, The Rediscovery of Kuldahar, The War of Bloom and
  Rot, and Will of the Wisps (new shaman kits and items, optional new enemies).
- **More choices** from the Tweaks Anthology, SCS, the Crossmod Banter Pack and Hidden Gameplay
  Options. Alternatives are exclusive: picking one replaces the other.
- **EEex Remote Console**, a developer tool.

## The app

- **A separate copy.** Chriz Easy IWD never changes your original game. Removing an
  installation leaves the original and the shared downloads untouched.
- **Exact versions.** Mods are not bundled: the app downloads each one from its official source
  and checks it against the tested version before installing it.
- **Pause and resume.** An interrupted installation can be continued from **My installs**.
- **Hotpatches.** Fight updates from Chriz IWD Encounters can be applied to an existing
  installation from **Updates → Hotpatch**, so you can keep your current run. Each hotpatch has
  a short changelog that says what it changes and what reaches a game already in progress.
  Hotpatches are offered only to installations that stay close to the recommended setup.
- **Manual mods (advanced).** Pauses let you add WeiDU mods by hand after a chosen mod. Chriz Easy
  IWD records them but cannot guarantee them, and an installation with manual mods gets no
  hotpatches.
- **Radar (optional).** The BG Radar Overlay shows nearby enemies with their resistances and
  buffs. **Start with Radar** is off at first; Windows may ask you to approve it, and you can
  always play without it.

## Known limitations

- This is an early release: not every combination of optional choices has been playtested.
- Only the English Steam version of IWD:EE 2.7.3.0 has been tested.
- Chriz Easy IWD does not update itself yet. New versions, and the hotpatches they bring, are a
  new download from Releases. Installing a newer version does not change existing
  installations; a hotpatch is applied only when you choose it.
- The installer is not code-signed, so Windows may warn about an unknown publisher. Download it
  only from the official page or this repository, and compare the published SHA-256 checksum.
- Mods are installed as their authors released them. Please report problems with your Chriz
  Easy IWD version and its logs; check the logs for personal paths or other information you do
  not want to share first.

## Help and links

- Player guide, troubleshooting and download: [bg.chrizfader.org/iwd](https://bg.chrizfader.org/iwd)
- Questions and bug reports: [ChrizFader's Discord](https://discord.gg/RtXjRzy68k) or the
  [issues](https://github.com/Chrizhermann/chriz-easy-iwd/issues) of this repository. In the
  app, **Export diagnostics** collects the logs of a failed installation.
- Source code: [chriz-bg-collection](https://github.com/Chrizhermann/chriz-bg-collection), the
  same source as Chriz Easy BG.

## Mods and credits

Mod credit belongs to the original authors. Chriz Easy IWD does not supply the game or
redistribute mods.

| Mod | Author | Version | Recommended setup |
| --- | --- | --- | --- |
| Enhanced Edition Fixpack | Gibberlings3 | Beta 2 | yes |
| EEex | Bubb | 1.3.1 | yes |
| Bubb's Spell Menu Extended | Bubb | 5.2 | yes |
| Icewind Dale Graphical Overhaul | Weigo (Spellhold Studios) | 1.1 + fixes (fda64de) | yes |
| Unfinished Business for Icewind Dale | Gibberlings3 | 13 | yes |
| Minerva | The Artisan | 1.1 | yes |
| Karihi | The Artisan | 1.02 | yes |
| Athanasios | The Artisan | 1.0 (035784e) | yes |
| Guldruk | The Artisan | 1522ddb | yes |
| Nirik | The Artisan | 2a1e9a9 | yes |
| Cassia | Arvendor | 1.05 beta (8f9af4c) | yes |
| Bethany | Lava | 1.03 | yes |
| Dendjelion | Lava | 2.1 | yes |
| Dusky | Lava | 2.5 | yes |
| Hommet | Lava | 1.2 | yes |
| Ina | Lava | 2.5 | yes |
| L'anna | Lava | 1.8 | yes |
| Oak-Maw | Lava | 3.3 | yes |
| Orra | Lava | 2.7 | yes |
| T'viy | Lava | 2.4 | yes |
| Urchin | Lava | 2.5 | yes |
| Vito | Lava | 1.1 | yes |
| Icewind Dale Crossmod Banter Pack | The Artisan, Arvendor, Kulyok, Lava | 5 | yes |
| The Tweaks Anthology | Gibberlings3 (CamDawg et al.) | 18 + fixes (7649ced) | yes |
| Sword Coast Stratagems | DavidW | 35.21 | yes |
| Chriz IWD Encounters | ChrizFader | 0.1.0 | yes |
| Hidden Gameplay Options | argent77 | 5.2 | yes |
| BuffBot | ChrizFader | 1.12.0 | yes |
| Lefreut's Enhanced UI for IWD:EE | lefreut; IWD:EE port by r-e-d | 2.0 | optional |
| Infinity UI++ | Pecca | 1.31 | optional |
| Will of the Wisps | Lava | 3.6 | optional |
| Below & Below Inn | Lava | 3.1 | optional |
| Night of the Blinking Dead | Lava | 3.1 | optional |
| Snowytoes Hamlet | Lava | 1.6 | optional |
| Tale of Our Lady Dreamless | Lava | 2.7 | optional |
| Terror of the Skineater | Lava | 2.6 | optional |
| The Rediscovery of Kuldahar | Lava | 2.6.1 | optional |
| The War of Bloom and Rot | Lava | 1.3 | optional |
| EEex Remote Console | ChrizFader | 0.2.1 | optional |

The BG Radar Overlay is by tapahob. Fonts and other third-party notices ship with the app.

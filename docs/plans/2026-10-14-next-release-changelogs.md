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
- **Pending:** Modpack alpha.13 also adds a fix for Dorn's BG1 ambush, whose
  camera lock on Senjak could crash the game; party AI that no longer enters
  Defensive Stance with "Use special abilities" off; and chapter-by-chapter XP caps
  (experimental). Christopher decides whether the collection offers them.

### Changed behavior

- **Unfinished Business (BG2)** — now **Optional** (was **Default**). All 22
  offered components remain available in the installer, unchecked by default in
  Chris's Recommended.
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
  refused. If the bank cannot start, XP stays uncapped and unbanked, and the
  message log says so once per loaded game. This includes the optional SoA cap.
  (Modpack alpha.13)
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
  attack info display (to-hit chances when the attack cursor is over a target)
  comes switched off and can be turned on in EEex's options.
  - **Pending:** whether the collection switches the attack info display on
    (EEex component 9). Christopher decides.
- **BuffBot** (Default): the "Combat detected - casting stopped" and "casting timed
  out" notices now appear over the party leader's head. Before, both notices were
  always lost; stopping the buff run itself worked. (BuffBot v1.10.1)
- **EEex Remote Console** (Optional, experimental): after you return to the main menu
  from character creation, the load screen or the options, the single-player menu no
  longer stays behind the game's screens for the rest of the session. Its installer
  messages now name the EEex v1.2/v1.3 components it needs. (Remote Console v0.2.1)

### Mod versions

- Chriz BG Modpack v0.2.0-alpha.13 (was alpha.11)
- BG Rebalance v0.8.3 (was v0.8.2)
- Bardic Wonders v2.9c-balance.8 (was balance.7)
- EEex v1.3.1 (was v1.2.0)
- The Tweaks Anthology: development version `7649ced`, 134 changes after v18
  (was v18). There is no newer release yet; the collection pins this exact
  version, as it does for EET.
- ChrizFader's voicepacks v1.1.0 (new)
- SoD Remix v0.6.15 (was v0.6.13)
- BuffBot v1.10.1 (was v1.10.0)
- EEex Remote Console v0.2.1 (was v0.2.0)

## Release-note entry (collection alpha.21)

**Collection alpha.21** updates Chriz BG Modpack to v0.2.0-alpha.13, BG
Rebalance to v0.8.3, Bardic Wonders to v2.9c-balance.8, SoD Remix to v0.6.15,
EEex to v1.3.1, BuffBot to v1.10.1, EEex Remote Console to v0.2.1 and The Tweaks
Anthology to its current development version. New in Chris's Recommended: companion stat adjustments for Rasaad,
Dorn, Eldoth, Garrick and Quayle; EET's join XP for every joinable NPC that
lacked it; SCS fine weapons that never break; one potion
dose of each type at a time (Potions of Genius still stack); Heroism effects
that don't combine; summoned creatures that clear the fog of war; Ribald's
special stock in his regular store; sensible entrance points; 18 player
voicepacks from ChrizFader's voicepacks; and a choice of how Imoen continues
into Shadows of Amn, by default as the BG2 thief/mage Imoen. Challenge
Mode adds that fences can't be stolen from and ignore reputation, and picks the
strict potion rule where Potions of Genius don't stack either. The optional,
experimental campaign XP caps with banked XP now work. Unfinished
Business (BG2) is now optional; High Mastery now gives +5
damage, and the NPC move now includes Ajantis. In Bardic Wonders, the
Troubadour and Deathsinger spell copies and descriptions follow Spell Revisions,
the Gallant gets the new bard spells, and Symphony of the Dark Children ends
with its round of Invisibility. BuffBot's combat-stop notices now appear over the
party leader, and the optional EEex Remote Console no longer leaves the
single-player menu behind the game's screens. These changes apply to new installations;
existing games and saves are unchanged.

## Notes for the release integrator

- **This recipe needs the new app.** Challenge Mode's potion choice uses the new
  feature-group `variants` field, and older apps reject unknown recipe fields.
  The release that ships this recipe must set `minimum_app_version` to the app
  version that adds variants.
- **Bardic Wonders v2.9c-balance.8** has not been played in a game yet (its
  release notes). Its standalone Abettor tail 1.2.0 is for existing
  playthroughs; the collection installs component 2004 fresh and doesn't use it.
- **Modpack `430`** (scrolls at the reader's caster level, Default) fails on EEex
  v1.3.1 in alpha.12 and alpha.13: it shows its EEex error box and disables itself on
  the first load, a new game or an NPC join. A fix is in progress; this release needs
  it or Christopher's decision about the default (see `FOLLOW_UPS.md`).
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
- **EEex Remote Console v0.2.1** is a tag without GitHub release assets, like
  v0.2.0, so the pin is GitHub's commit archive of the tag commit. Its `START_SP` fix
  passed the README checklist on EET 2.7.3.0 with EEex v1.3.1 in the October 18 lab.

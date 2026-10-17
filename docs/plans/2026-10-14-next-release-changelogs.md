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

## Collection changelog

The new mod setup is for fresh installations. Existing games and saves are
unchanged.

### New choices

- **Companion stats** — **Default**, each separately: Rasaad Dexterity 17 and
  Constitution 16; Dorn Constitution 16; Eldoth Strength 17, Constitution 16,
  Wisdom 9 and Intelligence 14; Garrick Dexterity 17 and Intelligence 15; Quayle
  Constitution 15. Only the base scores change; class, kit, equipment and current
  hit points stay as they are. (Modpack alpha.12)
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
- **Modpack alpha.12** also installs companion fixes `196`/`197` when only some
  of the files they patch are present, and says which ones it skipped.
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

### Mod versions

- Chriz BG Modpack v0.2.0-alpha.12 (was alpha.11)
- BG Rebalance v0.8.3 (was v0.8.2)
- Bardic Wonders v2.9c-balance.8 (was balance.7)
- The Tweaks Anthology: development version `7649ced`, 134 changes after v18
  (was v18). There is no newer release yet; the collection pins this exact
  version, as it does for EET.

## Release-note entry (collection alpha.21)

**Collection alpha.21** updates Chriz BG Modpack to v0.2.0-alpha.12, BG
Rebalance to v0.8.3, Bardic Wonders to v2.9c-balance.8 and The Tweaks Anthology
to its current development version. New in Chris's Recommended: companion stat adjustments for Rasaad,
Dorn, Eldoth, Garrick and Quayle; SCS fine weapons that never break; one potion
dose of each type at a time (Potions of Genius still stack); Heroism effects
that don't combine; summoned creatures that clear the fog of war; Ribald's
special stock in his regular store; and sensible entrance points. Challenge
Mode adds that fences can't be stolen from and ignore reputation, and picks the
strict potion rule where Potions of Genius don't stack either. Unfinished
Business (BG2) is now optional; High Mastery now gives +5
damage, and the NPC move now includes Ajantis. In Bardic Wonders, the
Troubadour and Deathsinger spell copies and descriptions follow Spell Revisions,
the Gallant gets the new bard spells, and Symphony of the Dark Children ends
with its round of Invisibility. These changes apply to new installations;
existing games and saves are unchanged.

## Notes for the release integrator

- **This recipe needs the new app.** Challenge Mode's potion choice uses the new
  feature-group `variants` field, and older apps reject unknown recipe fields.
  The release that ships this recipe must set `minimum_app_version` to the app
  version that adds variants.
- **Bardic Wonders v2.9c-balance.8** has not been played in a game yet (its
  release notes). Its standalone Abettor tail 1.2.0 is for existing
  playthroughs; the collection installs component 2004 fresh and doesn't use it.

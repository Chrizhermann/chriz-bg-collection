# Chriz Easy BG 0.1.0-alpha.20

**Published:** app alpha.20 / collection alpha.20.
[Download and complete release notes](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.20).

This update adds Challenge Mode, new recommended choices and updated mods, and fixes
Lefreut's interfaces and resumed installations. These notes cover everything since
app and collection alpha.19.

The new mod setup is for **fresh installations**. Updating CEBG leaves your current
game and saves unchanged; only the Lefreut interface fix below also reaches existing
installations.

## Installer and launcher

- **Challenge Mode** choice at the top of the installer. It switches every Challenge
  Mode component on or off at once; each one can still be changed on its own and is
  marked as part of Challenge Mode. Off by default. Members today: Emotion, Courage
  and Hope don't stack; Baeloth's normal Sorcerer spells; lethal dragon melee; SoD's
  Insane bridge sequencers.
- **Lefreut's interfaces offer the trilogy campaigns.** With any of the three Lefreut
  interfaces, the start screen showed only Shadows of Amn, Throne of Bhaal and the
  Black Pits. Lefreut's interface learns that the game is EET from a setting EET's
  installer writes into its own profile, which an installer-made game doesn't use.
  The installer now writes it into the game's own profile. Existing installations get
  it the next time you press Play in the installer; launching from another shortcut
  first leaves it unchanged.
- **Resuming an installation** no longer stops when a mod is downloaded again and its
  link now redirects to a different address, as long as the file is identical.

## New-install collection

### Changed defaults

- **Emotion: Courage and Hope do not stack** is now **optional** (alpha.19: default).
  It is part of Challenge Mode; its effect is unchanged.
- **EET: Create desktop shortcut** is now **optional** (alpha.19: default). EET's
  shortcut starts the game without EEex and replaces any existing `EET.lnk` on the
  desktop; the installer's own shortcut starts the game correctly.

### New choices

- **Bard spell-level ceilings** (default). Blades, Skalds and Dancers stop at
  6th-level spells. Other bards, Jesters included, reach 7th at bard level 21;
  Kapellmeisters and Darkbloom reach 8th at bard level 29. Bardic Wonders kits are
  covered when Bardic Wonders is installed. Requires EEex. Existing characters gain
  the capacity at their next level-up.
- **Fade and Evandra catch-up XP** (default). On first recruitment in Shadows of Amn,
  each is raised to the highest protagonist XP threshold reached (125,000 to
  1,250,000). Never lowers XP, pays once, cannot be farmed.
- **Campaign XP banking** (experimental, optional). BG1 stops usable XP at 161,000
  and SoD at 500,000. Further XP is banked and released in a later campaign. An
  optional addition caps Shadows of Amn at 2,950,000. The character record's Class
  tab shows the bank. A dual class without banked XP follows the unmodded rule. If
  banking cannot start on a game version, it stays off and Imoen's and Fade/Evandra's
  XP work as without it.
- **Degrodel: keep the Helm of Balduran and fight him** (default). You can hand the
  helm over as before, or keep it: Degrodel then stays and fights with the same
  servants, and the quest closes with a short journal note. No extra difficulty is
  added.
- **SoD: early BG1 companions** (part of SoD Remix). Skie can join before the first
  rest. Eleven BG1 companions can be recruited around the palace, the Elfsong, Tiax's
  cell and the Coast Way camp. Imported companions keep their build; known deaths are
  respected.
- **SoD: report Korlasz to Liia later** (part of SoD Remix). Liia accepts the report
  after the first night for the same once-only reward.
- **SoD: Avernus portal and a fairer final battle** (part of SoD Remix). A one-way
  portal skips the lift and its waves. Belhifet and Hephernaan get bounded fight
  changes and a capped number of devils. The confrontation and victory are unchanged.

### Changed behavior

- **Scrolls use the reader's caster level** (default; alpha.19: mandatory). Replaces
  "Use Any Item scrolls get a fair caster level." Casters use their active caster
  level of the scroll's type. Other legitimate readers use half their highest active
  class level, minimum 1. Requires EEex.
- **Bardic Wonders Blade** (default, with the Blade Overhaul): two weapon slots in
  the listed swords, daggers, axes, spears and halberds; Two-Weapon Style at most
  three.
- **Battleluck** (default, with New Bard Spells): five rounds, the caster and allies
  within 30 feet; recasting refreshes instead of stacking.
- **Flail of the Ages** (default): its Slow save and Haste-compatible +5 now also work
  with modded versions of the flail. Same balance as before.
- **SoD relocated loot** (part of SoD Remix): jailbreak and camp-chest rewards keep
  their original identified or unidentified state.

### Package versions

- **Modpack 0.2.0-alpha.11**: Fade/Evandra catch-up, reader-level scrolls, Degrodel
  and experimental campaign XP banking.
- **SoD Remix 0.6.13**, up from 0.6.12: early companions, Korlasz report, Avernus
  portal and final battle, relocated-loot fix.
- **BG Rebalance 0.8.2**, up from 0.8.1: Flail of the Ages compatibility.
- **Bardic Wonders 2.9c-balance.7**, up from balance.6: Blade and Battleluck.

## Important notes

- Campaign XP banking is experimental and not yet played through in game. Removing
  the component or EEex does not pay out banked XP. Use it only if you accept these
  limits.
- The new Modpack and SoD changes pass their automated checks; in-game playthrough
  acceptance is still pending.
- CEBG's updater signature is not a Windows Authenticode publisher signature.
  Windows may still display an unknown-publisher warning. Download only from the
  official release; do not disable Defender or add blanket exclusions.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

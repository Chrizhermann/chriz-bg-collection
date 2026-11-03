# Chriz Easy BG 0.1.0-alpha.19

**Published:** app alpha.19 / collection alpha.19.
[Download and complete release notes](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.19).

This update adds more UI choices, advanced install breakpoints, clearer progress,
optional Radar launch, new balance options and updated mods. These notes cover
everything since public app alpha.18 / collection alpha.16.

The new mod setup is for **fresh installations**. Updating CEBG leaves your current
game and saves unchanged. The specific existing-game fixes below are separate.

## Installer and launcher

### More choices

- Choose **Infinity UI++**, **Dragonspear UI++**, **LeUI** in BG1/BG2/SoD styles,
  or **EET's SoD-style interface** for a new installation. The recommended
  interface stays available. Known conflicting UI options are excluded, and the
  needed LeUI/Dragonspear text compatibility fix is included automatically.
- **Advanced install breakpoints:** experienced modders can choose several points
  to pause after a component finishes, add their own mods, then review the changed
  mod list before continuing. Normal installations have no extra pauses.
  Manual changes disable CEBG game fixes and mod upgrades for that copy, but not
  app updates. See the [manual-mod guide](manual-mods.md).
- Mutually exclusive choices replace one another instead of allowing incompatible
  selections, including the conflicting stronghold options.
- Common customization choices are easier to find.
- New installations need unique names. Existing entries can be renamed without
  moving their game or save folders; old duplicate names are distinguished.

### Clearer progress and results

- Updates has one compact installation selector at the top, shared by its
  game-specific actions. Longer paths and technical explanations stay under Details.
- Installation, update and backup stages show a progress bar when progress can be
  measured, and a spinner otherwise. Backup progress uses bytes actually copied.
- Completion, failure and Undo messages are more obvious and can be dismissed.
  An app update is confirmed after restarting into the requested version.
- Folder errors explain missing folders, permissions, locks or conflicting files.
  Technical details remain expandable.
- Installation removal shows the estimated folder size before confirmation.
- Fixed update and Radar feedback being hidden for the initially selected game.
- The Updates badge includes applicable fixes and recovery states alongside app,
  collection and Radar notices, without automatically scanning games at startup.

### Radar is your choice

- Play has a **Start with Radar** checkbox, initially off and remembered separately
  for each installation.
- Radar can ask for its own Windows administrator approval without running all of
  CEBG as administrator. Declining that request or a launch failure leaves the
  game running and displays a message.

### Installation fixes

- Fixed interval saves being installed but inactive because their settings went
  into the wrong game profile. Missing settings now go into the installation's own
  profile and are checked before completion. Existing preferences, including
  deliberately disabling interval saves, are preserved.
- Recommended installations include the interval-save settings menu. It follows
  the chosen interval and stays inactive when interval saves are excluded.
- Fixed the cache-layout error when the same download is needed in two different
  folder layouts. Download and extracted-file checks remain enabled.
- Alternative UIs use the supported mod-launch method. Invalid launch settings
  are caught before installation starts.
- Fixed the Customize error when changing choices such as the stronger SoD bridge
  mages. The choice and keyboard focus are preserved.
- Fixed the expected component order for Flail of the Ages in BG Rebalance.
  Recipe order is checked against the mod source.
- Apex Dragons accepts both supported SCS difficulty systems, including the one
  used with alternative UIs. Its abilities and difficulty thresholds are unchanged.
- Retry/Resume cannot start twice while an installation is already starting.
  Resumed installations show the current phase; cached downloads no longer make
  an unrelated install step appear finished.
- Partial installations give more useful diagnostics for missing components and
  unexpected order. Install-history checks stay enabled; these fixes do not
  guarantee automatic recovery of every previously failed installation.
- Corrected recipe validation, component catalog entries and old update records,
  including the stale Red Wizard/Spell Revisions exclusion note and unused archive
  records. Diagnostic exports include update and recovery details.

## Supported updates to existing games

- Check installed mods without resetting your choices. CEBG offers only the
  specific updates or repairs supported for that installation.
- Eligible older games can update **BuffBot 1.8.3-alpha to 1.8.4-alpha** in place,
  fixing unidentified items revealing their names in BuffBot. Saves, BuffBot
  settings and other mods stay in place. This route does not jump to BuffBot
  1.10.0 or upgrade the whole collection.
- A separate repair restores campaign class-description links for eligible older
  Artisan's Kitpack installations: English, recorded `chriz-v1.3.0` components.
  It changes descriptions, not kit mechanics, dialog.tlk or saved characters.
- Affected files are always backed up. A full game-folder backup is recommended
  and initially selected; a profile/save copy is optional. Sizes, free space and
  the backup destination are shown first.
- **Apply**, **Undo** and interrupted-update recovery are separate actions. The
  game must be closed; an interrupted repair blocks Play until recovery completes.
- Fixed the BuffBot package check incorrectly rejecting the reviewed archive.
  An undone update now says **Restored BuffBot 1.8.3-alpha**.

These are specific supported operations, not a general update-all-mods feature.
See the [supported update scope](existing-install-upgrades-2026-09-21.md).

## New-install collection

### Spells, resistance and items

The following changes are selected by default when their required mods are selected.

- **Emotion: Courage and Hope do not stack.** Casting one replaces the other;
  recasting refreshes it.
- **Hardiness:** 40% physical resistance, without SR's extra damage resistances,
  for both normal and Wish Hardiness. Customize offers a 30% alternative and a
  separate optional 20% elemental/magic-damage resistance addition. That addition
  is not Magic Resistance.
- **Party physical resistance:** gains above 80% are halved, up to 90%.
  90% before the adjustment becomes 85%; 100% becomes 90%. This applies separately to
  physical damage types, not enemies, summons, elemental damage or Magic
  Resistance. The proposed 95% alternative is not included.
- **Pierce Magic:** halves current Magic Resistance for five rounds. The reduction
  is rounded up and limited to 10–40 percentage points, without going below zero.
  Recasting refreshes the original reduction rather than stacking. Lower
  Resistance can still combine with it; protection removal is unchanged.
- **Spellstrike:** 15% arcane and divine spell failure for two rounds, refreshing
  rather than stacking with itself. Protection removal is unchanged; other
  spell-failure effects can still combine with it.
- **Flail of the Ages:** Slow allows a save vs. Breath: −2 for +3, −4 for either
  +4 version, and −6 for +5. The +5 weapon keeps Free Action without blocking
  Haste. Damage, chance to trigger Slow and Slow duration are unchanged.
- **Bracers of Blinding Strike:** with SR, grant SR Improved Haste's +1 attack
  per round instead of vanilla doubled APR. They retain one daily use and their
  20-second duration.
- **Elemental arrows:** BG1/EET elemental arrows use the BG2-style balance.
- **Combat clouds:** covered genie and vampire cloud forms replace damage immunity
  with 75% resistance. Other resistance values and open
  damage channels, poison immunity, regeneration and duration remain. Normal
  forms, quest protections and scripted death escapes are unchanged.
- **Dragon Wing Buffet:** the longer 18-second cooldown is selected by default
  instead of six seconds. It requires SCS Smarter Dragons and can be switched off.
  The separate lethal-melee challenge remains optional and unchecked.

### Archer overhaul

- Manyshot's extra projectiles are replaced by **Aimed Shot** with EEex. From
  level seven, one arrow or bolt gets 50% extra physical damage, then recharges
  for six seconds. A miss spends the bonus. Mod-added bows and crossbows work too.
- Farsighted, Sniper and Greater Called Shot are removed. Rapid Shot and
  Hardiness remain.
- **Called Shot** lasts ten seconds. Hits apply a saveable 25% movement slow for
  two rounds, refreshing rather than stacking. Dexterity halving and APR
  reduction are gone. From level 16, the activation also adds +2 missile damage.

### Companions and progression

- **Evandra's Sorcerer conversion and custom portrait** are independent defaults
  when Evandra is selected. Starting spells match SR or vanilla, while her
  stored level and XP are preserved for recruitment level-ups.
- **Edwin with Artisan's Red Wizard:** his necklace can be removed and grants no
  extra spell slots; the kit supplies the intended +2. The previously missed EET
  Edwin variant receives the selected kit on new installations. Explicitly
  different kits and saved actors are not overwritten.
- **Optional Baeloth spellbook balance:** trims known spells to a legal Sorcerer
  spread, with SR-aware or vanilla spell choices. It is unchecked by default;
  this does not add the complete planned Challenge Mode.
- **Story-based scroll, lock and trap XP** is the default: BG1 rewards throughout
  BG1 and SoD, BG2 rewards from SoA onward. Removing the XP cap no longer unlocks
  BG2 rewards early. This also corrects the BG1/SoD reward for level-16+ locks.
- The existing smooth, protagonist-XP-based reward curve remains an optional,
  mutually exclusive alternative. It is not party-average XP; neither choice
  adds XP banking.

### SoD Remix

- The Boareskyr Bridge battle now has a collapse deadline: **10/8/7/6/5 unpaused
  minutes** on Easy/Normal/Core/Hard/Insane, with five on Legacy of Bhaal.
  Story Mode has no timer.
- Warnings, rockfall sounds and short shakes mark the approaching deadline.
  Defeat both mages and all four elementals to stop demolition; remaining guards
  do not keep it going. Save/reload and retreat preserve the deadline.
  Running out of time ends the game.
- Extra Insane mage sequencers remain a separate, unchecked option.

### Optional convenience changes

- **Generic unidentified-item visuals** reuse ordinary game art by item category,
  including cursed items. Identification restores the normal appearance. This
  works independently of loot randomization, but cannot be combined with
  Randomiser's cursed-appearance option.
- **Talking while shapeshifted** for Artisan's Shapeshifter and Hivemaster forms.
  Other restrictions and genuine Silence are unchanged. Off by default.
- Legacy text compatibility is added automatically for LeUI and Dragonspear UI++.

### BuffBot

- Fresh installations use **BuffBot 1.10.0**, up from 1.8.4-alpha.
- Identified carried items can expose their buff powers separately, including
  secondary weapon powers and backpack equipment.
- Its inventory-use preference can use supported powers directly or
  equip/use/restore eligible gear. Interrupted equipment swaps have recovery
  checks. This does not add automatic wand/scroll use or access to container
  contents; equipment eligibility restrictions remain.
- Includes the intervening spell-list refresh and compatibility fixes. The
  supported older-game update route remains the separate 1.8.3-to-1.8.4 update.

### Package versions

- **BG Rebalance 0.8.1** includes the finished balance changes above.
- **Modpack 0.2.0-alpha.10** includes Evandra, story XP, unidentified visuals,
  legacy UI text, optional Baeloth balance and the Edwin correction.
- **Artisan 4.81a-dev.85928f9-chriz.1** includes the Archer overhaul and optional
  shifted talking. Fork versions now name the upstream base and patch revision;
  older releases have not been renamed.
- **Bardic Wonders 2.9c-balance.6** preserves the selected kit balance. Its separate
  kit-specific spell-progression option is not selected by this update.
- **SoD Remix 0.6.12**, up from 0.6.11, includes the bridge deadline above.
- **BuffBot 1.10.0**, as described above.

## Important notes

- Further general mod updating, XP banking, a 95% physical-resistance option,
  Entropy Shield/Dispelling Screen redesigns, instant trap detection, reduced NPC
  chatter and new Fighter/Mystic Fire rebalancing remain later work.
- The reported Fade/Evandra XP catch-up issues are not claimed fixed. Companion
  changes for fresh installations do not automatically rewrite saved characters.
- Extra interfaces and custom combinations are open for community testing.
  Not every option or full campaign route has been playtested.
- CEBG's updater signature is not a Windows Authenticode publisher signature.
  Windows may still display an unknown-publisher warning. Download only from the
  official release; do not disable Defender or add blanket exclusions.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

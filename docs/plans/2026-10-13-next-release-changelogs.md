# Next release — changelog drafts

13 October 2026. These are drafts for the release after app/collection alpha.19.
Labels: **Default** = selected in Chris's Recommended; **Optional** = offered,
unchecked; **Pending** = waiting for Christopher's choice. Items only enter the
final notes once their owner release is published and pinned. Version the next
release properly; never republish over alpha.19.

## Installer changelog

No installer or launcher behavior has changed since alpha.19. The changes so
far are collection recipe data and their tests.

Planned (approved 13 October, not built yet):

- **Challenge Mode** choice at the top of the installer. It switches every
  Challenge Mode component on or off at once; each one can still be changed on
  its own and is marked as part of Challenge Mode. Off by default
  ([design](2026-10-13-challenge-mode-group.md)).

## Collection changelog

The new mod setup is for fresh installations. Existing games and saves are
unchanged.

### Changed default

- **Emotion: Courage and Hope do not stack** is now **Optional** (alpha.19:
  Default). It is a planned Challenge Mode member; its effect is unchanged.

### New choices

- **Bard spell-level ceilings** — **Default.** Blades, Skalds and
  Dancers stop at 6th-level spells. Other bards, Jesters included, reach 7th at
  bard level 21; Kapellmeisters and Darkbloom reach 8th at bard level 29.
  Bardic Wonders kits are covered when Bardic Wonders is installed. Requires
  EEex. Existing characters gain the capacity at their next level-up.
- **Fade and Evandra catch-up XP** — **Default.** On first recruitment in
  Shadows of Amn, each is raised to the highest protagonist XP threshold
  reached (125,000 to 1,250,000). Never lowers XP, pays once, cannot be farmed.
- **Campaign XP banking** — **Experimental, Optional.**
  BG1 stops usable XP at 161,000 and SoD at 500,000. Further XP is banked and
  released in a later campaign. An optional addition caps SoA at 2,950,000.
  Known limits: a no-bank dual class between level thresholds is blocked for
  now, the bank shows only in notices and a console command (a character-record
  display is in progress), and removing the component does not pay out banked
  XP.
- **SoD: early BG1 companions** — **Default.** Skie can join before the first
  rest. Eleven BG1 companions can be recruited around the palace, the
  Elfsong, Tiax's cell and the Coast Way camp. Imported companions keep
  their build; known deaths are respected.
- **SoD: report Korlasz to Liia later** — **Default.** Liia accepts the report
  after the first night for the same once-only reward.
- **SoD: Avernus portal and a fairer final battle** — **Default.** A one-way
  portal skips the lift and its waves. Belhifet and Hephernaan get bounded
  fight changes and a capped number of devils. The confrontation and victory
  are unchanged.

### Changed behavior

- **Scrolls use the reader's caster level** — replaces "Use Any Item scrolls
  get a fair caster level." Casters use their active caster level of the
  scroll's type. Other legitimate readers use half their highest active class
  level, minimum 1. Requires EEex. **Default** (alpha.19: mandatory).
- **Bardic Wonders Blade** (Default, with the Blade Overhaul): two weapon pips
  in the listed swords, daggers, axes, spears and halberds; Two-Weapon Style
  at most three.
- **Battleluck** (Default, with New Bard Spells): five rounds, the caster and
  allies within 30 feet; recasting refreshes instead of stacking.
- **Flail of the Ages** (Default): its Slow save and Haste-compatible +5 now
  also work with modded versions of the flail. Same balance as before.
- **SoD relocated loot** (part of SoD Remix): jailbreak and camp-chest rewards
  keep their original identified/unidentified state.

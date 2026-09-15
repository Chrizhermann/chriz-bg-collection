# Chriz Easy BG 0.1.0-alpha.16 — draft

Not published. Candidate bundle: app alpha.16 / collection alpha.14.
Recommended selection: **448 components / 50 runs**.

**September 16 candidate:** contains the completed owner releases and the existing
approved curation. Christopher will run the installation and playtest himself.
The public download remains unchanged until publication; hotpatching is not included.

## Changes

- A visible loading screen now appears immediately while CEBG finds your games.
  Startup failures show their details and a Try again button.
- **Use thief skills in armor** is checked by default and can be turned off in
  Customize. It permits ordinary thieving and stealth without adding skill
  penalties. Equipment permissions and spellcasting restrictions stay unchanged.
- Updates Artisan's Kitpack to `chriz-v1.5.0`: accepted Assassin, Magekiller,
  Hivemaster and Paladin changes, fighter modal adjustments and kit descriptions.
- Updates Bardic Wonders to `v2.9c-balance.5`, including its bard song, kit and
  shared high-level ability corrections. Existing component choices and the
  Darkbloom / Spell Revisions exclusion are preserved.
- **Edwin's Red Wizard kit** is default-checked again, including with Spell
  Revisions. It remains optional. An unsupported collection exclusion was removed;
  this does not install the kit into existing games or alter saved Edwin characters.
- **Choose kits regardless of race** is default-checked and optional. Complements
  the existing class unlock so elves can choose the missing mage specialists too.
  Internal/NPC-only kits and the separate gnome mage rules remain unchanged.
- **The Murder of Acton Balthis** (BG2 Unfinished Business) is now unchecked by
  default. It remains available as an optional quest in Customize.
- Three new **unchecked SCS options**: provisions for wilderness/dungeon rests,
  more expensive inns with rest bonuses, and revised death/petrification/imprisonment
  rules. All three are visibly **Not recommended / experimental** because of
  community reports of problems, not yet independently reproduced in CEBG.
  Death effects explains and enforces SCS's own incompatibility with Spell
  Revisions. Provisions are not required in SoD.
- **21 further SCS and 44 Tweaks Anthology choices**, all unchecked, with clearer
  descriptions and enforced alternative groups. Includes reputation/price options,
  rest encounters, cosmetic tweaks and identification alternatives. Riskier class,
  spell, proficiency and story changes remain excluded pending targeted review.
  This batch has source-level compatibility screening, not gameplay acceptance of
  every combination.

The racial kit unlock is now default; Acton Balthis becomes opt-in. The unchecked
customization batch changes no other selections. The 448/50 count includes the
expanded defaults below, not the two opt-in dragon components.

## Expanded collection

- SoD Remix v0.6.10 adds Khalid's bounded Bridgefort continuity fix; its accepted
  bridge, filler, loot, XP and companion changes are retained. Extra bridge
  sequencers stay optional; repair-only components are not fresh-install defaults.
- Spell Revisions v4.21-chriz.5 supplies classic bouncing Lightning Bolt by
  default, with a mutually exclusive non-bouncing option. Includes SR/RR encounter
  compatibility in its required pre-SCS position. Turning SR off removes both
  dependent changes automatically.
- Official Safana in Amn v05 is included with one-time arrival inventory cleanup.
  Unfinished Bard/Abettor conversion is not included.
- Modpack v0.2.0-alpha.6 includes BG1 Yeslick's Alaghor conversion when selected,
  Xan/Yeslick continuity through EET transitions and Spellhold Imoen XP catch-up.
  Vanilla companion choices remain available; these do not migrate existing saves.
- BG Rebalance v0.4.0 exposes two optional dragon challenge components: lethal
  melee at high difficulty and longer wing-buffet spacing. Both stay unchecked.
- BuffBot v1.8.4-alpha respects item identification and remains the final mod run.
  BG Radar Overlay retains its independent latest-release download/update flow.

No source-version change is needed for the existing Tweaks Anthology v18 component.
Unreleased mod-repository experiments are not part of this public recipe.

## Existing installations

Updating CEBG updates the installer and launcher. The new collection applies to
new installations; this update does not modify your current game or saves.
Completed games do not need an urgent rebuild. Targeted existing-game patches are
being assessed separately and are not an automatic feature of this release.

## Release status

Source intake, the Windows package and updater signature are verified. Publication
is pending; the working local candidate is documented in
[the package handoff](alpha16-candidate-2026-09-16.md).
The full installation and native acceptance belong to Christopher; no additional
agent-run installation or playtest is scheduled. Windows publisher signing remains
separate from CEBG's authenticated updater.

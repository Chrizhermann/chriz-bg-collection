# Chriz Easy BG 0.1.0-alpha.16 — draft

Not published. Proposed bundle: app alpha.16 / collection alpha.14.

## Changes

- A visible loading screen now appears immediately while CEBG finds your games.
  Startup failures show their details and a Try again button.
- **Use thief skills in armor** is checked by default and can be turned off in
  Customize. It permits ordinary thieving and stealth without adding skill
  penalties. Equipment permissions and spellcasting restrictions stay unchanged.
- Updates the existing Bardic Wonders balance fork to `v2.9c-balance.4`, including
  its Skald, Dancer, Jester and shared high-level ability corrections. Existing
  component choices and the Darkbloom / Spell Revisions exclusion are preserved.
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

The recommended recipe has 436 components across 44 runs. The racial kit unlock
adds one default; Acton Balthis becomes opt-in. The unchecked customization batch
changes no other selections.
No source-version change is needed for the existing Tweaks Anthology v18 component.
Unreleased mod-repository experiments are not part of this public recipe.

## Existing installations

Updating CEBG updates the installer and launcher. The new collection applies to
new installations; this update does not modify your current game or saves.
Completed games do not need an urgent rebuild. Targeted existing-game patches are
being assessed separately and are not an automatic feature of this release.

## Release status

Source preparation and focused checks are tracked in the handover. Packaging,
updater download/signature verification and publication are still pending.
No alpha.16 download or update notification is available yet. Windows publisher
signing remains separate from CEBG's authenticated updater.

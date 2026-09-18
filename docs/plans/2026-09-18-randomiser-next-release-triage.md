# Randomiser: next-release triage

Requested by Christopher on 2026-09-18 from community feedback. **Record only:
no investigation, implementation, recipe change or agent dispatch now.** These
items belong on the next-release discussion list, not a promise to ship every
feature in that release. Implementation belongs in the Randomiser repository;
CEBG owns later pins, compatibility metadata and player-facing options.

## Compatibility report — first priority

A player reports errors with Wisp's Item Randomiser, custom RR item additions,
Mode 2 installed last, and the original-BG-style weapon proficiency component
from Tweaks Anthology. They report a reinstall implicated the proficiency tweak.
Another player uses Randomiser before CDTweaks and suggested an order conflict.
The exact versions, component, error and causal explanation are **unconfirmed**.
Do not assume this affects CEBG's Mode 1 setup or reorder the curated recipe now.

When investigated, establish the actual component/error and whether ordering or
proficiency remapping needs a compatibility fix. Link this to the existing
[optional proficiency work](2026-09-07-optional-proficiencies-and-companion-customization.md).
User-supplied reference (not reviewed in this task):
<https://www.gibberlings3.net/mods/items/item_rand/>.

## Feature ideas to scope for the next release

- **Editable item pool:** add/remove entries, including Rogue Rebalancing and
  additional EE items. Adding an item must account for its original placement so
  it is relocated deliberately rather than accidentally duplicated.
- **Expanded cursed items:** variety across equipment slots, varying appearances,
  and a configurable amount/frequency (potentially a slider).
- **New-game randomisation:** explore EEex-based, Mode-2-like randomisation per new
  game rather than requiring a mod reinstall. Preserve the appeal of varied BG1
  starting shops and mage scrolls. Feasibility and save isolation are unverified;
  the conversation's suggestion is not an implemented capability.
- **Optional challenge placement:** favor worthwhile combat/boss rewards over
  easily looted caves or hidden spots bypassable with invisibility. Allow multiple
  rewards on selected difficult enemies. Example from the report: move a cave's
  valuable reward onto the associated named sirine; exact resources need identifying
  later. Keep ordinary exploration-friendly placement available, not a forced
  global replacement.

Discuss scope and defaults with Christopher before committing the larger features
to the release. Keep compatibility repair separate from new gameplay features.

# Randomiser: next-release triage

Originally requested by Christopher on 2026-09-18 from community feedback, with
a **record-only** boundary at that time. The September 27 appearance clarification
below authorizes a bounded source check and handoff to the active Randomiser task;
it does not authorize all of the broader ideas. These
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

## September 27: unidentified appearances — confirmed owner split

Christopher requested two alternative appearance policies and clarified that the
general policy must not depend on Randomiser:

- **A — Generic unidentified items:** belongs in `chriz-bg-modpack`. Show a generic
  appearance appropriate to the item's slot/type while it is unidentified; retain
  the real appearance after identification. It must work without loot randomisation
  and cover ordinary/mod-added items where supported, not just cursed items. No
  shipped Modpack component ID or implementation was verified in that first check;
  the source implementation below now reserves component 630.
- **B — Random cursed appearances each run:** belongs in Randomiser. Existing cursed
  items are in scope, not only future custom curses. Choose appearances for a new
  campaign, keep them stable through save/load, and isolate different campaigns
  when switching saves. Preserve item identity, curse and gameplay statistics.
- **CEBG must make A and B mutually exclusive.** This is an appearance conflict,
  not a reason to prohibit using generic appearances with ordinary loot
  randomisation. Include the existing install-time cursed-appearance option in
  conflict handling. Use stable component/marker identities and standalone
  conflict guards where practical. Modpack A now has the confirmed identity below;
  do not invent Randomiser B's component number.

### What the check established

The currently pinned Randomiser **v8.1.1 component 570** is already enabled in
CEBG's recommended preset. `randomiser.tp2` sets `flrc570.mrk`;
`lib/fixes.tpa`, starting at line 322 in that tag, uses WeiDU `RANDOM` and
`COPY_EXISTING` to change item appearance resources **during installation**.
This does not supply a new appearance mapping for each campaign.

The active task **Fix randomizer proficiency conflict**
(`01a0bb24-3c8e-77b2-849a-b3d3b0300eb0`) already described per-campaign cursed
appearances as planned, not implemented. Christopher's requirement and the owner
split were sent to that task on September 27, without interrupting its current
creature-source work or authorizing additional live-game tests.

For A, distinguish presentation from a permanent icon replacement. The
[ITM format](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/itm_v1.htm)
has separate identified/unidentified text but a shared inventory-icon field.
Replacing that field alone would not provide the requested before/after behavior.
A runtime presentation hook (EEex and/or UI integration) needs a bounded feasibility
check; do not promise every UI combination merely because EEex is installed.
Check inventory, stores, containers and item inspection for identity leaks, and
verify identification restores the intended presentation without changing mechanics.

**Initial check boundary:** requirements and coordination recorded, not implemented or tested.
No recipe/default change, component creation, release or game/save write in this
check. Broader curse designs, frequency controls and release timing remain separate.

### September 27: generic artwork implementation (unreleased)

Christopher subsequently approved implementation, prioritizing feature completeness,
code quality and the default setup. Work is isolated in Modpack branch
`codex/unidentified-item-appearance`, checkout
`C:\Users\chris\.codex\worktrees\modpack-compatible-presets\chriz-bg-modpack`.

- Component **630**, label **cbm_unidentified_appearance**, marker **CBMID.MRK**.
- Presentation-only EEex hooks cover native item icons/dragging, ground art and
  engine-to-Lua icons/description pictures. Identification retains the real art;
  no ITM, TLK, save or UI.menu rewrite. Generic art is selected by item type, with
  neutral fallback for unknown types. Ordinary loot randomization is independent.
- The standalone installer rejects Randomiser **570 / flrc570.mrk**. These
  identities were sent to the existing Randomiser task for the reverse check and
  its forthcoming per-campaign appearance option. CEBG must also exclude both
  directions once a released Modpack version is pinned.
- Source uses signatures audited against default BG2EE 2.7.3.0 / EEex 1.2.0;
  validates all sites at install/startup and preserves original code on a failed
  hook setup. Default UI is the acceptance target, not every third-party UI.
- Focused Lua/x64 emulation and real-WeiDU disposable-fixture tests pass. Live
  acceptance is still pending; follow Modpack `docs/unidentified-items.md` for
  inventory, shop, bag, drag, ground, Identify, save/reload and uninstall checks.

**Release boundary:** not published, not pinned in CEBG and not applied to a live
game. The current recommended recipe still selects 570. Do not label 630 ready,
silently replace that default, or overlay it onto an active stream installation.
No new-run requirement is introduced by stored campaign data, but existing-save
behavior remains part of the manual acceptance rather than a hotpatch promise.

**Test scheduling, September 27:** Christopher explicitly chose to include 630
in the [next combined release-install playtest](2026-09-22-release-readiness.md#next-release-install-playtest--additions-agreed-september-27),
with 570 off. Do not create a separate test install now. This does not mark the
component live-tested or change the public default selection.

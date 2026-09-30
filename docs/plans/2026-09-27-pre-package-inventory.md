# September 27 pre-package inventory

Fresh source/release check requested by Christopher before assembling the next
installer. This is an integration queue, not a release or live-test claim. Do not
change the stream installation, run another full test install, or silently enable
unapproved optional mechanics.

## Published updates and recipe integration

September 29: all four published updates below are now pinned and verified in
recipe source: Artisan `chriz-v1.6.0`, Bardic `v2.9c-balance.6`, SoD `v0.6.12`
and BuffBot `v1.9.0`. Existing selections are retained. The previous pins in the
historical inventory below show the original gap, not current source. This does
not extend the accepted existing-game BuffBot updater. See the
[current checklist](2026-09-22-release-readiness.md).

The recipe source is alpha.18; the September 23 app alpha.19 binary still carries
alpha.17. Compare against the source pins, not that older package. The owning
repositories' GitHub Latest releases were checked September 27 using the personal
account; local worktrees were inspected read-only for additional unshipped work.

| Owner | September 27 recipe pin | Release now integrated in source | Relevant change |
|---|---|---|---|
| SoD Remix | v0.6.11 | [v0.6.12](https://github.com/Chrizhermann/chriz-sod-rebalance/releases/tag/v0.6.12) | Bridge collapse timer and warnings. Fresh component 256 includes it; 258 is the adapter for older 256 installs, not an extra effect to stack on fresh installs. 257 remains the sequencer option. |
| Artisan Kitpack | chriz-v1.5.0 | [chriz-v1.6.0](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/tag/chriz-v1.6.0) | Released Archer rebalance: Aimed Shot and revised Called Shot; removes legacy Manyshot/Farsighted/Sniper/Greater Called Shot behavior. The separate shifted-dialogue addition below is NOT in this release. |
| Bardic Wonders | v2.9c-balance.5 | [v2.9c-balance.6](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.6) | Optional 3010 per-kit spell-progression adapter for BG Rebalance 420. Refreshing the source pin does not authorize enabling 420/421 or this new adapter. |
| BuffBot | v1.8.4-alpha | [v1.9.0](https://github.com/Chrizhermann/bg-eeex-buffbot/releases/tag/v1.9.0) | Stable release, including experimental 5E compatibility. A fresh-install pin update is separate from extending the accepted 1.8.3-to-1.8.4 updater. |

BG Rebalance v0.6.0, Modpack v0.2.0-alpha.7, SR patch v4.21-chriz.5 and
Randomiser v8.1.1 match the checked public releases. Rebalance 301 is already
default in recipe alpha.18; 420/421 remain unselected.

SoD release evidence: `bridge-collapse-timer` worktree,
`docs/releases/v0.6.12.md`, merge b2af832. Timer uses unpaused game time and
preserves its deadline across save/reload; Story disables it. Ordinary difficulty
budgets are 10/8/7/6/5 minutes for Easy/Normal/Core/Hard/Insane. Automated coverage
is recorded by the owner; native presentation/balance is deferred to the combined
install, not a separate release gate. Ashatiel group-battle ideas are still design,
not part of this release.

## Implemented now, not packaged or published

- **Approximate size in delete confirmation:** advisory metadata-only scan in
  `app/src-tauri/src/removal_size.rs`. Returns unknown for incomplete/error/link
  scans, capped at five seconds between filesystem calls and 500,000 entries.
  Does not change removal targets, confirmation identity or safeguards. No size
  scan when merely forgetting a missing entry. Typecheck, 42 focused frontend
  tests and six focused native checks passed. No real installation was deleted.
- **Optional shifted dialogue:** owned by the Artisan patch repository, branch
  `codex/shifted-dialogue-qol` in the existing `archer-rebalance` worktree.
  `ArtisansKitpack_tweak` component 51010 / `AKCB_SHIFTED_DIALOGUE`, off by default.
  Patches only the installed Shapeshifter/Hivemaster form items' equipped
  self-targeted Talk restriction. Other restrictions, Silence, combat fields,
  attack abilities and text remain untouched. Five real-WeiDU synthetic tests
  passed, including reinstall and byte-exact uninstall. Missing-form warning
  correctly returns WeiDU exit 3 in its explicitly expected-warning test.
  Christopher waived a separate manual playtest gate on September 27. Native
  dialogue is not claimed tested. Needs owner release and optional recipe wiring.

## Other owner work: do not lose it or mistake it for a release

- **Modpack 630, generic unidentified art:** committed as 6eefd2c on
  `codex/unidentified-item-appearance` in `modpack-compatible-presets`; unreleased.
  Already approved for the next combined install, with conflicting Randomiser
  appearance component 570 off. Ordinary loot randomization can stay. Needs owner
  release, verified pin, mutual exclusion and the existing combined checklist.
- **Companion changes:** `companion-spellbooks` Modpack checkout has uncommitted
  stat choices 230-234 and tolerant missing-variant work for Faldorn/Dynaheir.
  It is based on older alpha.6. Preserve that owner's work; reconcile with alpha.7
  before any integration. Baeloth's legal-spell-count work remains a proposal,
  not a verified released fix. Do not mix these into 630's release automatically.
- **Randomiser:** `mode1-postinstall-profiles` checkout has active, dirty
  per-new-campaign configuration/editor work. Some focused native examples exist;
  that is not complete collection/store/script compatibility. Latest published
  version remains 8.1.1. Do not pin a dirty checkout or absorb its code here.
- **SR Death Ward, Time Stop/Haste, larger Challenge Mode:** designs/research,
  not released additions. Keep out of a fast packaging batch.

## Viewer request: simplified weapon proficiencies

CDTweaks **2161** is the best first candidate: eight BG-style weapon groups while
keeping fighting styles. 2162 removes styles; 2163/2164 use IWD groupings. 2160
only relocates a few weapon types. All remain excluded in current curation.
[Upstream documentation](https://gibberlings3.github.io/Documentation/readmes/readme-cdtweaks.html#proficiencies).

This is not a harmless checkbox yet:

- Current Randomiser `lib/fl#gear.tpa` recognizes 2161/2162, but explicitly leaves
  IWD grouping 2163/2164 unsupported in its proficiency adjustment code.
- Artisan's per-weapon EEex proficiency/usability machinery uses original weapon
  categories; no complete grouped-category adapter was found in this bounded pass.
- Final Modpack companion presets also assign original weapon categories. Check
  final allocations, item usability and summoned/mod-added weapons after remapping.

Next work is a bounded 2161 compatibility adapter/review with those owners. Do not
offer all four variants or describe universal compatibility from EET support alone.

## Viewer request: choose all companion weapon pips

SCS **4100**, Improved NPC customization and management, is the existing EET
candidate. It is broader than a pip picker: level-zero recruitment, XP catch-up,
mage/bard spellbook handling, bonus-pip removal and special dual-class rules.
Class/kit changes use a separate optional Customize ability. Some NPCs are excluded;
legacy 4099 is not an EET substitute.
[Pinned installer](https://github.com/Gibberlings3/SwordCoastStratagems/blob/v35.21/stratagems/setup-stratagems.tp2),
[readme](https://gibberlings3.github.io/Documentation/readmes/readme-stratagems.html).

Current order is SCS/Randomiser, then the main Modpack NPC conversions, then
EET_end. The September 7 plan's description of those core conversions as
post-EET_end was stale. SCS leveller metadata can predate the final class/kit/pip
changes, so check or adapt that interaction before exposing 4100. Preserve the
chosen companion builds rather than silently undoing them.

4100 also includes the five-point thief-skill UI and excludes separate 4115.
The prepared Infinity UI exclusions currently address 4115, not this additional
route. Selecting 4100 would need the same UI compatibility accounted for. No new
default, selection or general companion respec was added in this pass.

## Alternative UI delivery status

**The frontend selector exists; the production recipe still has no provider
group, so users cannot choose these UIs in the current package.** Do not call
preview recipes shipped integration.

Owning checkout: `C:\Users\chris\.codex\worktrees\cebg-ui-options\chriz-bg-collection`.
Sources: `tools/ui_providers/catalog.toml`, `tools/prepare_ui_preview_recipe.py`,
`docs/plans/2026-09-22-parallel-ui-and-clarity.md`.

- **Infinity UI++ 1.27/component 0:** source/archive metadata prepared; intended
  optional experimental choice for new installs. Excludes SCS 4115 and HGO
  29/30/300/301/302/304. User accepts leaving general playtesting to users; known
  conflicts must still be reflected honestly in the recipe.
- **EET SoD-style UI/component 0:** prepared, but requires publishing the EET_gui
  folder from the existing EET archive. No extra HGO exclusions currently needed.
- **LeUI 4.9.1, BG2/BG1/SoD styles:** catalog prepared but blocked on the game 2.7
  `t()` / `getUiString` compatibility helper. Do not rely on an optional HGO choice
  to provide it. A small owned adapter or upstream fix is needed first.
- **September 29 update: Dragonspear UI++ is now approved for the next release**
  as another optional provider. Its own package pin, order and exclusions still
  need integration; Praetor's manual-install report is not compatibility proof.
  All interface choices are for **new installations only**, not UI swapping in
  an existing campaign.

## Next integration batch

1. Bring the released owner versions into the recipe with verified archives and
   synchronized evidence/curation/change records. Preserve existing choices;
   refreshing Bardic does not enable its new progression adapter.
2. Obtain releases for approved Modpack 630 and Artisan 51010, then add their
   optional integration and exclusions. Never publish a recipe pointing to
   unpublished component numbers in older archives.
3. Finish prepared Infinity/EET provider integration, add the September 29 approved
   Dragonspear UI++ provider, and resolve the concrete LeUI helper before exposing
   it. Keep full respec and grouped proficiencies separate
   if they require more than a small compatible change.
4. September 29 scope addition: implement the agreed advanced manual breakpoints,
   including accepted manual WeiDU entries surviving continuation/finalization.
   This is included in the next release, not left as an unspecified later feature.
5. Run proportionate combined source and basic package/safety checks, build a new
   package, and let Christopher and the community cover broader compatibility and
   gameplay. Do not require him to personally test every option or require an
   exhaustive agent-led full-install matrix. Existing BuffBot acceptance
   is not repeated and does not automatically qualify a new update adapter.

No build, release, recipe pin/default changes or live game actions occurred during
this inventory and the two bounded source implementations.

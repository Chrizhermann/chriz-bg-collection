# Evandra defaults: source repair and remaining delivery

Christopher authorized fixing the missing default Sorcerer conversion and custom
portrait. This is an omission from the public recipe, not an incorrect user
selection or a failed installed conversion. Work below is local source work;
no game, save, release or installer executable was changed for this repair.

## Owner source

Checkout: `C:\Users\chris\.codex\worktrees\modpack-compatible-presets\chriz-bg-modpack`
(existing `codex/unidentified-item-appearance` branch; preserve committed 630).

- **224:** trueclass Sorcerer for `rh#eva` and `rh#ev25`, not the vampire template.
  Preserve stored level/XP, unrelated identity, dialogue, scripts, equipment and
  personal grants. Replace only the owned class/kit and wizard book, using
  semantic spell symbols and installed known-spell/cast tables. Level 10 retains
  its starting book so subsequent level-ups remain player choices. SR/non-SR
  spell differences are handled explicitly. This does not migrate saved actors.
- **225:** independent custom portrait option, same `rh#evaL` resource used by
  the upstream epilogue and ordinary saved portrait references. Set both portrait
  fields on the two recruitment templates and vampire variant if present.
  Christopher created the image with ChatGPT and approved inclusion September 29.
  The approved BMP hash is
  `b4bde98aec195d47db521d3cc4d72b636ae5a0f391cc7469ea5d48688b112aad`.
- Common TP2/translation, public component tests, exact package allowlist, README,
  changelog, provenance notice and `docs/evandra.md` are updated. Number 191 stays
  reserved for the unrelated private Sarah portrait. Version markers stay alpha.7
  until a separately prepared owner release; these new components are unreleased.

## CEBG integration

`tools/evandra_defaults.py` integrates 224/225 only when the pinned Modpack
catalogue actually offers both and is not the old alpha.1–7 artifact. This hook
runs before feature presentation and recommended-preset generation. It does not
invent a new release URL, checksum or catalogue entry.

Both choices default on independently under Modpack, depend on Evandra's actual
core component 0, and run after it in the main pre-continuity companion slot
immediately after 223. Disabling Evandra omits both; disabling crossmod does not.
Neither is a mandatory override of the user's preference.

Normal Tauri packaging and `tools/package-recipe.ps1` now run the default checker
before bundling/signing. The current curated recipe intentionally fails this
readiness check because its real alpha.7 pin cannot deliver either component.
Ordinary development builds remain available. Historical manifests were not
falsely updated with nonexistent components.

## Verification on September 29

- **130 Modpack checks passed:** 34 Evandra/public-surface checks (including 25
  real-WeiDU Sorcerer and five portrait cases) plus 96 related defaults,
  companion-class, release-builder and spell-tail regression checks. WeiDU 249
  ran against disposable synthetic games, not the user's installation.
- **36 CEBG checks passed:** default integration/package guard, curated generator
  and combined-playtest generator suites. Profiles that do not offer Evandra do
  not acquire a Modpack dependency just because the packaging check exists.
- The readiness command against the current alpha.7-pinned curated recipe exited
  **1 as expected**, naming the missing 224/225 delivery. This is the remaining
  release prerequisite, not a failed implementation test. Future-catalogue
  fixtures pass; those fixtures are not a published artifact or engine acceptance.
- Read-only installed SR metadata inspection confirmed all 24 authored choices
  have the intended wizard type/level and are not hidden. This is not proof of a
  completed fresh-stack install or in-game recruitment/level-up acceptance.
- Independent source review found no actionable issue. Extracted future-release
  installation of 224/225 remains part of owner release verification.

## Before the next package

1. Review and publish the owner release with its exact runtime asset allowlist.
   Coordinate this with pending Modpack 630/other owner work; do not absorb it
   into this repository or silently drop it from its existing branch.
2. Verify the published archive, update the Modpack artifact and component
   catalogue, then regenerate the curated recipe. Add the new public curation
   rows and release-change evidence; synchronize recipe totals/digests normally.
3. Run `python -m tools.evandra_defaults --recipe recipes/curated-full-current`
   and normal engine recipe/default-selection validation. The default checker
   must then pass; do not bypass it to package the old incomplete recipe.
4. Package only after the remaining release checklist is satisfied. The combined
   install/playtest still provides live acceptance; fixture tests are not that.

The separate **Fade/Evandra XP catch-up** investigation remains high-priority and
open. No claim that these two source components fix recruitment XP. Christopher
owns the current EE Keeper repair; do not alter the running stream game or saves.

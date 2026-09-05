# Common customization controls

Christopher requested categorized include/exclude-all and prominent original-companion,
Randomiser, Artisan Kitpack, Bardic Wonders and Spell Revisions switches, including the
dependent consequences. SoD Remix is the sixth common control. Default curation is unchanged.

## Design and behavior

Retain CEBG's existing iron #151819 / charcoal #23282a / vellum #f1e7d2 /
brass #c59a51 / teal #75bab4 palette, Georgia headings and Segoe UI body.
Use a compact left-aligned two-column list, not six large cards or a new visual identity:

```text
Customize                              Back   Done
Original companion classes & kits      Item Randomiser
Artisan's Kitpack                      Artisan's Bardic Wonders
Spell Revisions                        SoD Remix
Reset to Chriz's setup
> Also adjusted: related choices (when applicable)
> Advanced options by category
```

- Six simple common switches; narrower screens stack them in one column.
- Advanced search/categories are collapsed when the common controls are available.
- Each category has **Include all compatible** and **Exclude all optional**. Optional
  includes default-checked choices, but not mandatory components. Unavailable options are
  not forced on; existing mutually exclusive choices win. These controls apply to the
  named category, not just the current text-search matches.
- A bundle change is one selection request/evaluation, not a chain of racing checkboxes.
  The engine still owns dependencies, readiness, conflicts and final component selection.
- The projection now supplies authored `requires` and `conflicts` IDs. Frontend bulk
  selection honors both directions of asymmetric conflicts without inventing engine rules.
- Requested child preferences survive parent off/on. Original-class and legacy Bardic
  bundles remember their previous individual choices during customization. Reset clears
  all feature/input overrides and restores the curated preset.
- A collapsed **Also adjusted** list names effective collateral inclusions/omissions after
  any individual or bundle change. Keyboard focus returns to the changed control.

## Bundle boundaries

- Artisan switches all three main/NPC/tweak parents; dependent Mazzy/Xan corrections follow.
- Bardic authoring now supplies a default-on parent over all 20 existing children. The
  frontend also covers the older ungrouped catalog, including its late compatibility patch.
- Darkbloom is authored as an SR conflict instead of a blanket blocked feature. Default SR
  still excludes it; SR-off makes it eligible according to the retained curated intent.
- Original companions excludes Artisan NPC assignments, the relevant modpack conversions,
  Xan class alternatives and offered CDTweaks class/kit conversions. Yeslick switches from
  Fighter-Alaghor to regular Fighter-Cleric only if a Yeslick route was wanted already.
  It does not silently add a previously excluded Yeslick NPC.
- "Original" means game defaults for original companions and the mod author's defaults
  for added companions. Sirene's authored True Paladin route stays. Portraits/audio,
  Dynaheir's Haste knowledge, stat-consistency choices and unrelated fixes stay.
- Randomiser/SR/SoD use their existing parents. SoD Remix off does not skip the SoD story.
- These are **new-install choices**, never savegame repairs or live-mod uninstallation.

### Owning-repo follow-up: combined dispel fix 410

Original-class Yeslick route 0 currently disables modpack 410 because its collection
requirement names route 1. The component patches both Yeslick SPIN112 and Keldorn SPCL231,
and hard-fails on missing/noncanonical resources. Thus its Keldorn half is also omitted;
the UI lists that related change. Do not broaden the requirement without proving the
regular Yeslick topology. The modpack owner should either validate route 0 with a fixture
or separate the two repairs. This is a known customization limitation, not a claim that
every independent fix is preserved under every combination.

## Verification and delivery limits

- 88 frontend tests pass; TypeScript check and production web build pass.
- 16 Rust recipe-view tests pass, including authored asymmetric conflict/prerequisite
  projection. Cargo was run from PowerShell.
- Four focused authoring tests pass, generating small temporary recipes only. They cover
  all Artisan parents off, Bardic off and Garrick collateral, SR-off dependencies/unlocks,
  and original classes while 130/430/440/450 and Dynaheir 197 stay selected.
- Headless Edge rendered a display fixture projected from the **493-control real catalog**,
  clicked all six common switches twice, and checked search/category expansion. No console
  errors; no horizontal overflow at 1920, 1366, 1160, 768, 375 or 320 pixels. The common view
  has no vertical scroll at 1920x1080, 1366x768, 1160x720 or 768x1024; mobile scrolls normally.
- Visual inspection found and fixed an invisible reset-button label and overly bold
  descriptions. Screenshots: `target/customize-1366.png`, `customize-320.png`, and
  `customize-advanced.png`; reproducible ignored harness `target/customize-visual.mjs`.
  This was a focused Customize check, not an exhaustive whole-app UX audit. The visual
  fixture is not native installation evidence; actual batch wiring is separately tested.

Source and web build only: **not yet in the installed/signed alpha.8**. Do not regenerate
`recipes/curated-full-current` or replace r5's frozen recipe during its run. Apply the
authoring changes when freezing the next distinctly versioned candidate. No game files,
Steam sources, stream saves or running installation were changed. No new game install
was started. The isolated headless browser and its Vite test server were closed.

Further useful common choices, after the existing priorities: additional companion packs,
portraits/voices, convenience tweaks and difficulty/encounter choices. Group these only
after their boundaries are mapped; do not add more switches just to fill the screen.

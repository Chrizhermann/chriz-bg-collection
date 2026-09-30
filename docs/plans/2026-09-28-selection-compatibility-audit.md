# Selection compatibility — September 28

Christopher confirmed that the reported custom stronghold install selected all
the offered stronghold options. Scope: fix proven native selection conflicts and
conduct a bounded review for similar gaps, not rebalance kits or test a live game.

## Strongholds: fixed in source, awaiting packaging

Tweaks Anthology v18 `cdtweaks/setup-cdtweaks.tp2:1088–1195` explicitly rejects
each individual all-class stronghold 1340–1347 when either Multiple Strongholds
1160 or 1161 is installed (also checked via variables in native batch mode).
The recipe previously modeled only the exclusion between 1160 and 1161.

The generator now authors all 16 cross-family pairs in both directions. This
feeds existing Customize availability, explanations and Include All filtering,
and the engine's final plan. The eight individual options are NOT exclusive to
each other. The recommended 1161 choice, all defaults and install order remain
unchanged. To choose individual strongholds, first select None for Multiple
Strongholds. No WeiDU verification or recovery rule was relaxed.

The generated curated recipe and the unreleased alpha.18 change ledger carry
the correction. The fallback `manifest/collection.toml` offers no CDTweaks run
and needs no invented feature entries. Historical released ledgers are untouched.

Regression coverage:

- Generator test verifies the exact symmetric exclusions, unchanged default and
  all eight legal individual choices together. Its missing-conflicts failure was
  reproduced before the fix.
- Production-recipe engine test checks both Multiple variants, all 16 illegal
  pairs (including stale/imported requests), the legal individual route and the
  unchanged rest of the plan. UI availability exposes the conflict reason.
- Customize test exercises Include All with either family already selected;
  it preserves that family and never requests the opposite one.

## Two additional dependency gaps: fixed in source

- **BG1 NPC Project v32:** its optional components require component 0. The
  generator-added 36 choices lacked this requirement even though the original
  base choices had it. They now require `feature:bg1npc:component-0`. Native
  example: `bg1npc/bg1npc.tp2:90–98`; the same prerequisite appears on the other
  affected components. Their requested preferences are preserved when the core
  is off, but they cannot enter an invalid install plan.
- **Our bundled SCS selection:** checking only the Ascension parent allowed its
  required finale component 0 to be switched off. SCS 35.21 components 6840,
  6850 and 8085 require `fingrom.cre` (`setup-stratagems.tp2:1317,1331,1687`).
  Ascension 2.1.0 component 0 creates it through `ascension_main.tpa` and
  `ascension_the_five.tpa` (Gromnir mapping at line 25; CRE copy at line 202).
  The standalone Tougher Gromnir component is not its provider. Both profiles
  now require `feature:ascension:component-0` for our SCS bundle. This is not a
  claim that every upstream SCS component requires Ascension.

These changes do not add components, change defaults, or reorder installation.
The unreleased alpha.18 ledger records all 37 affected dependency addresses.

## Completed bounded source review

The requested Opus 5.5 job **did not run the audit**: exit 1, unsupported model
in Claude Code 2.1.272 (requires 2.1.280+), no findings or model usage. Its saved
status and output were inspected in `target/work/sep28-opus-exclusion-audit/`.
No retry, client update, model substitution or billing change was made.
The lead and a native SCS review subagent completed the source checks instead;
the lead independently verified the consequential findings.

- CDTweaks v18: all 175 offered components matched native component blocks.
  All 26 ordinary multi-option groups already had exclusions. Beyond the
  stronghold family, no further missing exclusion was confirmed in the offered
  set. Existing IWDification casting-graphics exclusions are correct; other
  native conflicts target options/mods not currently offered.
- SCS 35.21: checked 106 mapped components, including six offered multi-option
  groups. Existing SR exclusions for 4130/4240 and bundled internal prerequisites
  are covered. Native checks that forbid installing spell components after the
  AI core are ordering requirements, not mutually exclusive choices.
- SCS 4100 cannot coexist with 4115 (thief skills in multiples of five).
  Component 4100 is not offered yet; retain the already documented replacement
  constraint in the deferred companion-customization plan. No option enabled.
- BG1NPC v32 core requirements were checked; Ascension 2.1.0 provider source and
  offered IWDification v11 headers were checked where relevant.

Scope is native component-level prerequisites and choice groups, not exhaustive
runtime compatibility or every third-party mod combination.

## Verification and delivery

Focused checks passed: **46 Python** generator/curation tests; **31 Rust** tests
across `curated_clarity`, `updates`, `production_recipe_tail` and
`production_recipe_coverage`; **13 Customize frontend** tests from the
stronghold-fix pass. No frontend implementation changed afterward.

Actual-recipe tests exercise all 16 illegal stronghold pairs, legal individual
strongholds, missing BG1NPC core and missing Ascension finale in both profiles.
Restoring prerequisites restores the unchanged default plan: **50 runs / 450
components**. Semantic diff confirms only exclusions, prerequisite references
and the SCS explanation changed in the features; decisions, readiness, component
lists and parents did not. Ledger and golden expectations were updated for those
exact changes, with identity and coverage assertions retained. Formatting and
diff checks pass; no unresolved test failure in this focused scope.

No full game install, save edit, package build or publication in this task.
An existing partial install is not automatically repaired by these source changes.

## Separate balance decisions

- Keep current kit defaults and broad variety. Balance specific outliers by a
  deliberate later decision, not merely because a viewer calls them strong.
- Mystic Fire is deferred to the Artisan owner, recorded in that repository's
  `docs/plans/2026-09-28-mystic-fire-balance.md`. Review Shatter Spell separately
  from scroll/wand component 3202; no nerf selected or implemented now.
- Warlock investigation and balancing remain deferred. No change to its current
  integration state is part of this task.

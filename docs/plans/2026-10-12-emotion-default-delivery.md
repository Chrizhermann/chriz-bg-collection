# Emotion default change — October 12 delivery

Status: **implemented locally and checked; next-release WIP, not published**.
Owner: Codex `emotion_default` worker; implementation is complete and no worker
is retaining an edit lock. Coordination/release integration remains with the lead.

Worktree: `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.

## Result

- Emotion no-stacking (`feature:chriz-bg-rebalance:component-301`) is optional
  and explicitly off in the recommended preset, in both recipe profiles.
- It remains ready, selectable and in its original component/run position.
- Baeloth235 remains optional/off. Flail302, elemental arrows130,
  physical-resistance cap310 and every other existing default are unchanged.
- This implements only the settled default policy, not the broader Challenge
  Mode group, membership or UI.

The source and generated current recipe are development candidates. Version
markers still say alpha.19; no shipped snapshot, release ledger or acceptance
record was changed. A future release must version/document this selection
change together with its other changes. Do not publish it over alpha.19.

## Checks

- Focused Python selection/generation/credits/release-metadata suites: **42 passed**.
- Focused Rust recipe suites: **25 passed** (11 curated clarity, 7 Rebalance/SoD,
  7 Modpack). Includes Emotion opt-in, Baeloth off, preserved default run content,
  parent/dependency behavior and original component order.
- Existing engine CLI: both `manifest` and `recipes/curated-full-current`
  validate under `public-alpha` with **zero findings**.
- Curation audit passes: 1,224 catalog rows; the only changed decision is Emotion.
- Parsed comparison against pre-change Git HEAD: only Emotion's decision and
  preset selection differ in either profile; all other feature data, preset
  values, runs and order are identical. Effective selections differ only by301.
- Rust formatting check and scoped `git diff --check` pass.
- Protected-file diff is empty for `manifest/install-order.tsv`, both release
  record trees and app version metadata.

Commands used:

```text
python tools/curated_full_recipe.py --output recipes/curated-full-current --commit fd0806d3db056239b833e424411807bb11dd1aef
python -m unittest tools.tests.test_expanded_release_recipe tools.tests.test_curated_full_recipe tools.tests.test_public_component_credits tools.tests.test_release_metadata
python tools/curation_audit.py docs/curation/components manifest/curation-map.toml
cargo test -p chriz-bg-engine --test curated_clarity --test production_recipe_chriz --test production_recipe_modpack
target/debug/chriz-bg-install.exe validate manifest --profile public-alpha --json
target/debug/chriz-bg-install.exe validate recipes/curated-full-current --profile public-alpha --json
```

The generator commit argument preserves the existing historical evidence
references; it does not claim the new edits were already committed or accepted.
No full app build, packaged UI test, game installation, game/save modification,
commit, push or publication was performed.

## Changed files

- `manifest/collection.toml`
- `manifest/presets/chris-recommended.toml`
- `docs/curation/components/CHRIZ-BG-REBALANCE.md`
- `recipes/curated-full-current/collection.toml`
- `recipes/curated-full-current/presets/chris-recommended.toml`
- `recipes/curated-full-current/RECONCILIATION.md`
- `tools/tests/test_expanded_release_recipe.py`
- `tools/tests/test_curated_full_recipe.py`
- `tools/tests/test_public_component_credits.py`
- `engine/tests/curated_clarity.rs`
- `engine/tests/production_recipe_chriz.rs`
- `engine/tests/production_recipe_modpack.rs`
- This delivery note.

Pre-existing handover/planning changes were preserved and not edited by this
worker. Old September release descriptions in the component catalog are labelled
historical; the new policy is explicitly stated above them.

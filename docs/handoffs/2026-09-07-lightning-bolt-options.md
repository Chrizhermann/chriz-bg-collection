# Lightning Bolt choices for the collection

Christopher requested three Lightning Bolt choices on 2026-09-07 and clarified
the scope of the third: the wizard spell and Wand of Lightning should stop
bouncing, while traps retain their existing bouncing behavior. This is an
implemented development integration, not an available recipe update.

This isolated worktree starts at collection commit
`bdb040e0d0ab7c63eac260497f3b828116fcb6aa` on branch
`codex/lightning-bolt-options`. The original `codex/installer-v0-real-alpha`
checkout has concurrent uncommitted UI work; this worktree does not copy or
modify it. The dirty `C:\src\private\chriz-bg-collection` curation checkout is
also separate.

## Authorized behavior and ownership

| Player choice | SR component selected beyond core | Intended behavior |
|---|---|---|
| Spell Revisions | Neither `80` nor `81` | Keep the existing SR wizard spell. This is the unchanged default. |
| Original | `80` | Restore the classic wizard bolt, including bouncing, hits along its path, and friendly fire. |
| Non-bouncing | `81` | Restore the line-hitting wizard bolt without bouncing; also stop Wand of Lightning bolts from bouncing. Preserve path hits and friendly fire, and leave traps unchanged. |

Components `80` and `81` are mutually exclusive. The maintained
`chriz-spell-revisions-patch` repository owns their WeiDU implementation and
uninstall behavior. The collection references that mod; it must not absorb or
redistribute its implementation.

The component-81 implementation in development derives a dedicated projectile
from `LIGHTB` using the non-bounce flag approach demonstrated by Tweaks Anthology
component `2530`. It redirects the intended spell/wand resources rather than
changing the shared trap projectile. The owning mod's 96 tests pass, including
39 Lightning Bolt checks. A disposable rehearsal using copies of the current
EET resources verifies wizard/wand routing, unchanged dedicated traps and original
projectile, wand statistics and descriptions, reflection protection, preservation
of SCS casting hooks and later save penalties, and byte-exact resource uninstall.
No live casting acceptance is claimed. Separate spell copies created by SCS or
another mod are not rewritten; confirm their intended treatment before accepting
the final collection order. A modded trap casting the ordinary wizard spell follows
that spell's selected behavior; dedicated trap spells keep their original projectile.

**Do not select CDTweaks `2530` to implement this choice.** That component is the
prior art for non-bouncing lightning, but its shared-projectile scope does not
preserve the requested trap behavior. Keep `2530` excluded from this collection
choice. Do not offer it as a second independent switch alongside `80`/`81`.

## Current release boundary

The development manifest now exposes the two alternatives as optional,
componentless **blocked** features. The catalog/map records their development
status, and the full-recipe generator preserves the group and authored unchanged
label. No new executable run or old-artifact component declaration was added.
The UI and engine transport/validate `choice_unchanged_label`; this group renders
Spell Revisions as its unchanged entry. Both new alternatives are disabled with
the reason that the pinned release lacks components `80` and `81`.

At the starting commit, both the base manifest and generated current recipes pin
`spell-rev-4.21-chriz.3`. Its immutable release menu contains only
`0, 10, 20, 30, 55, 60, 65, 70`; it cannot install `80` or `81`.

The authoritative current records are:

- `manifest/mods/spell-rev.toml`
- `manifest/artifacts/spell-rev-4.21-chriz.3.toml`
- `manifest/mod-sources.tsv`
- `manifest/evidence/artifact-verification.toml`
- `engine/tests/production_recipe_spell_rev.rs`

Do not invent a successor version, URL, asset size, hash, component menu, or
release-verification evidence. Do not mark the choices `ready` or author an
executable `80`/`81` run against the old artifact. A successor release must exist
and its exact archive/menu must be verified before enabling execution.

Executable run/component order, presets, generated current-release recipes,
artifact pins and installations remain unchanged. The authored development
manifest has the two blocked controls. No live-game acceptance is claimed.
Distributing a recipe with the new optional metadata also requires the updated
app/engine: old strict parsers will reject an unknown field. Do not publish the
new recipe separately to an app that cannot consume it.

## Semantic recipe contract

Use these stable identities when implementation proceeds:

| Identity | Value |
|---|---|
| Parent feature | `mod:spell-rev` |
| Prerequisite | `feature:spell-rev:mandatory-components` |
| Choice group | `choice:spell-rev:lightning-bolt` |
| Group label | `Lightning Bolt` |
| Original feature | `feature:spell-rev:component-80` |
| Non-bouncing feature | `feature:spell-rev:component-81` |
| Dedicated installer run | `spell-rev-lightning-bolt-bg2` |
| Installer | `spell-rev` |

Both component-backed features are `optional`, share the parent/category/group,
and have symmetric conflicts with each other. The group defaults to `none`:
neither feature selected means **Spell Revisions**, with no extra installer
component. Disabling the parent removes both options from the effective plan.
Do not change the recommended default merely to expose the alternatives.

When the successor is executable, its run owns `[80, 81]` as available menu
entries; selection must resolve to either no run, `[80]`, or `[81]`, never both.
The post-EET_end location is after the existing late
`cdtweaks-spell-save-penalties-bg2` run and before
`spell-rev-npc-spellbooks-bg2`, then BuffBot. The base manifest lacks the
CDTweaks late scan; the full-recipe generator inserts it. Update both authoring
and generation so the final order is explicit:

```text
... final spell/NPC/class changes
cdtweaks-spell-save-penalties-bg2 (2312, if selected)
spell-rev-lightning-bolt-bg2 (80 or 81, if selected)
spell-rev-npc-spellbooks-bg2 (60, if selected)
buffbot-bg2
```

This placement requires the owning mod's patch to remove only SR's own save
penalty while preserving the already applied CDTweaks penalties and protection
checks. Verify that behavior at representative levels and with the real
upstream spell shapes. Confirm SCS copy coverage before declaring this placement
accepted; do not assume changing only the base spell also changes copied spells.

Never hand-edit `manifest/install-order.tsv`: it is captured historical
WeiDU.log evidence, not the executable recipe's authored order.

## Exactly three labels in the app

The UI renders an optional mutually exclusive feature group as two component
options plus an empty choice. The implemented optional
`choice_unchanged_label` field names that empty choice; for this group it is
**Spell Revisions**. The engine validates that the label is nonblank, belongs to
a group with at least two non-mandatory members, and is identical across every
member. It propagates through the recipe view and native wire mapping. Existing
groups retain **None / unchanged**. Shared-scope, symmetric-conflict and
availability checks remain in force; presentation metadata does not authorize
installation. A separate third SR feature would introduce an unwanted fourth
empty entry, so none was added.

The completed selector should display exactly **Spell Revisions**, **Original**,
and **Non-bouncing**. Its description explains that the last choice affects the
spell and wand while traps keep bouncing. During development, both new choices
must remain visibly unavailable with an authored unreleased-component reason.

## Remaining integration work

The development feature/catalog/map and generic metadata work is implemented.
Both options have explicit blocked reasons and no executable component
references. Focused checks cover unchanged defaults, exact three-entry rendering,
synthetic ready-option switching/reset, blocked requests, parent disable,
metadata transport/validation and generator preservation of the old menu.

1. After the owning mod releases and verifies the successor, update artifact
   metadata/menu/pins, attach the exact run components, and implement generator
   ordering and selection tests. Update source and verification evidence with
   actual release facts. Switch readiness only when its stated checks pass.
2. Regenerate the derived recipe through `tools/curated_full_recipe.py` and the
   currently authorized recipe packaging flow. Keep default selections unchanged.
   The older creator-recipe tool deliberately refuses legacy WeiDU replay;
   do not revive that path merely to regenerate its historical snapshot.
3. Release the compatible app/engine with the recipe, then validate the final
   source/order integration, including save penalties and all three resolutions.

Primary code/data touchpoints are `engine/src/manifest.rs`,
`engine/src/recipe_view.rs`, frontend contracts/backend mapping/customization/
setup rendering, `docs/curation/components/SPELL_REV.md`,
`manifest/curation-map.toml`, `manifest/collection.toml`,
`manifest/mods/spell-rev.toml`, and `tools/curated_full_recipe.py`.

Focused validation when those files are actually implemented:

```powershell
python tools/curation_audit.py docs/curation/components manifest/curation-map.toml
python -m unittest discover -s tools/tests
cargo test -p chriz-bg-engine --test production_recipe_spell_rev --test production_recipe_coverage --test recipe_view
```

Completed checks: 110 focused Rust tests across `manifest_parse`,
`manifest_validate`, `recipe_view`, `resolve`, `production_recipe_spell_rev`, and
`production_recipe_coverage`; all 36 Python tests; 15 frontend customization and
wire-contract tests; frontend typecheck and production build. Curation audit
reports 1,191 rows, 62 choice groups and 14 optional-none groups. Rust formatting
was applied only to touched Rust files. No release or live install was performed.
Mod fixture evidence and later live casting acceptance remain separate from
these collection resolver/UI checks.

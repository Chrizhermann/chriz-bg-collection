# Recipe intake — 2026-09-06

## Follow-up for recipe alpha.11

Christopher accepted the SoD skip and supplied the official Windows Evandra file.
Alpha.11 retains the selections below, adds SoD Remix910 inside the default bundle,
and changes Evandra acquisition only. The SoD pin is now public `v0.6.7`, commit
`a6826ca1452cbe500c5e111f117b56a983a55d0d`: 1,509,999 bytes, SHA-256
`d82213b29e1d24cbd14562bbd57c9dab8d165e4ca80cb4d2f6f25bd590fb9155`.
Fresh extraction verified121 files and both TP2 paths; selected native order ends
290,900,910. Component910 requires EET/EET_end0 and remix110/140/150/160, not290/EEex.
Repair291 and alternative901 remain unselected. Owner reports82 Windows/Linux tests
and Christopher's six-person Yes-route acceptance; remaining No/reload/Imoen variants
remain in owning-repo issue19, not silently claimed tested here.

Evandra now uses exact `evandra-v2.2.exe` instead of the private aggregate. Both core
and crossmod follow Skip Evandra. All183 official payload files match the reference;
see the [acquisition record](handoffs/2026-09-06-evandra-public-acquisition.md).
Focused Python curation/generator23 and Rust SoD production4 tests passed.

The alpha.10-specific counts below are historical. Do not substitute curation-row
counts for the actual resolved WeiDU entries of alpha.11.

## Earlier alpha.10 intake

This note records the bounded intake from the released Chriz mods into recipe
`0.1.0-alpha.10`. It changes future recipe resolution only. The recovered r5 game remains
on `chriz-bg-modpack` alpha.5 plus Bardic balance 2 and is not mutated or relabelled.

## Included in the recommended preset

| Source | Component | Selection | Contract |
|---|---:|---|---|
| Chriz BG Modpack v0.2.0-alpha.5 | 220 — Yoshimo: Swashbuckler | default | Follows the vanilla-companion conversion policy. |
| Chriz BG Modpack v0.2.0-alpha.5 | 221 — Hexxat: Shadowdancer | default | Replaces the previous Artisan 7104 default. |
| Chriz BG Modpack v0.2.0-alpha.5 | 610 — Progressive utility XP | default | Requires EEex and the post-`EET_end` modpack run. The released alpha.5 XP curve and defaults are unchanged. |
| Chriz SoD Remix v0.6.6 | 290 — corrected victory ending | mandatory SoD bundle member | Runs after its 120, 130, and 185 prerequisites and before component 900. |

Relative to the matching alpha.9 recipe, these four additions and the Artisan 7104 removal
produce an exact net change of **+3**. Alpha.10 resolves to **344 engine plan entries** and
the generated reconciliation records **431 selected curation rows**. The recovered r5
WeiDU log's 430-component count is a different evidence surface and is not used to predict
the new recipe's install count.

## Alternatives and exclusions

- Chriz BG Modpack 222 (Hexxat Fighter/Thief), 223 (Hexxat Assassin), and Artisan
  Kitpack NPC 7104 (Hexxat Invisible Blade) remain visible, unchecked alternatives.
  Components 221, 222, 223, and 7104 are pairwise exclusive. Turning off the Artisan
  bundle does not disable modpack component 221.
- Chriz SoD Remix 291 is repair-only and is not selectable in a fresh-install recipe.
- Modpack 620 (Spellhold Imoen) and the dragon changes remain work in progress and are not
  authored into this recipe.
- The full-SoD skip remains a separate intake decision.

## Release evidence

- Chriz BG Modpack remains pinned to `v0.2.0-alpha.5`: 1,335,026 bytes,
  SHA-256 `2278c839f60e019bedba355cb794176a052d24b68db2851248af926580840b33`.
- Chriz SoD Remix is pinned to published release `v0.6.6`: 1,493,555 bytes,
  SHA-256 `73498501cf526e3f0f3d76b83b118cf16ffa8bcb4915736ad81ba3bd7de6c228`.
  Fresh archive verification found the expected direct-root publish layout, 116 entries,
  maximum depth 4, and 1,961,428 total uncompressed bytes.

## Focused verification

- Python curation audit and generated-recipe tests: 21 passed.
- Rust production recipe/modpack/SoD/Artisan tests: 19 passed, with one network/cache
  test intentionally ignored by the normal test suite; the SoD artifact was separately
  verified for this intake.
- Rust production coverage, release validation and manifest validation: 68 passed.
- Regeneration produced an identical generated tree; `git diff --check` passed.
- Exclusivity checks cover all four Hexxat alternatives, switching from Artisan 7104
  to modpack 221, and keeping 221 selected when the Artisan bundle is disabled.

No full installation, game launch or native UI test was performed for this intake.

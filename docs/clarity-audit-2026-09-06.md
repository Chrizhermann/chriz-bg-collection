# Recipe clarity and known-dependency audit — 2026-09-06

Scope: lightweight static audit of all 499 generated customization features, using the
existing component catalogs, `manifest/curation-map.toml`, and authored recipe
`parent`/`requires`/`conflicts` fields. No game, installer, live mod files, or exhaustive
compatibility research was used. Mechanics and selection policy were not inferred from
component names.

## Outcome

- Every feature now carries a non-empty source label and group label: 39 displayed
  sources and 127 displayed groups across 26 categories.
- The 33 explicit curation groups that contain at least two retained alternatives are
  projected to 130 options. All 33 pass the frontend's same-parent/category/group and
  full pairwise-conflict rule. The audit repaired the legacy split representation for
  BG1 NPC banter timing, Kivan proficiency, and Sarevok diary adjustment using those
  explicit curation groups as authority. The 28 one-option policy records are
  deliberately not shown as mutually exclusive choices.
- Previously, 195 features repeated their title as their description. Nineteen ambiguous
  Tweaks options now have concrete explanations; 176 straightforward labels remain
  repeated and are suppressed by the UI instead of receiving boilerplate prose.
- Component 1142 is titled “Potions require identification” and explains that gems are
  unchanged. Stronghold alternatives explain class restrictions, and the 16 stacking
  alternatives state the item type and per-inventory-slot limit. Other short choices
  retain their authored subgroup in `group_label`.
- Known dependency data remains explicit: 248 features have parents, 62 have `requires`,
  and 139 have `conflicts`. Choice metadata does not replace dependency rules.
- Runs, feature decisions/readiness/requirements/components, and recommended selections
  are unchanged. The raw semantic digest is now
  `6bb3e74be4e4bd50265329d25c99eeeb3636203a7c69a1557ee9f853e6a39517`; a regression
  snapshot that normalizes only the six repaired parents and three repaired conflict
  lists still matches the pre-audit digest
  `4320bd5a5a766e1acc1667d8979e1c33816baa0f0e110a1dabdc4a53d5cc8415`.
- This is an unshipped source patch. App alpha.10's strict TOML schema does not accept the
  new fields, so it requires the next app release when this recipe is packaged. No
  published recipe version or updater feed was changed by this audit.

## Category coverage

“Choice options” counts only options in a multi-option authored curation group.

| Category | Features | Parent | Requires | Conflicts | Choice options |
|---|---:|---:|---:|---:|---:|
| audio | 2 | 1 | 0 | 0 | 0 |
| banters | 12 | 0 | 0 | 6 | 6 |
| bg1-content | 10 | 9 | 0 | 0 | 0 |
| bg1-fixes | 7 | 7 | 0 | 0 | 0 |
| bg1-npcs | 16 | 15 | 8 | 9 | 9 |
| collection | 5 | 1 | 3 | 0 | 0 |
| compatibility | 1 | 1 | 1 | 0 | 0 |
| content | 23 | 0 | 0 | 5 | 5 |
| convenience | 60 | 2 | 2 | 47 | 47 |
| core | 9 | 5 | 0 | 0 | 0 |
| cosmetics | 21 | 0 | 0 | 9 | 9 |
| difficulty | 5 | 4 | 2 | 0 | 0 |
| endgame | 9 | 8 | 0 | 0 | 0 |
| engine | 3 | 1 | 1 | 0 | 0 |
| fixes | 11 | 10 | 10 | 1 | 0 |
| gameplay | 11 | 11 | 0 | 5 | 4 |
| interface | 31 | 29 | 3 | 3 | 3 |
| items | 4 | 0 | 0 | 0 | 0 |
| kits | 71 | 66 | 15 | 2 | 0 |
| npcs | 91 | 43 | 17 | 49 | 45 |
| quests | 16 | 0 | 0 | 0 | 0 |
| randomisation | 12 | 11 | 0 | 1 | 0 |
| rules | 36 | 3 | 0 | 2 | 2 |
| spells | 9 | 8 | 0 | 0 | 0 |
| tweaks | 13 | 7 | 0 | 0 | 0 |
| visuals | 11 | 6 | 0 | 0 | 0 |

Category review: audio, items, quests, spells, tweaks, and visuals contain no authored
dependency edges needing clarification. Banters, content, convenience, cosmetics, rules,
and part of NPC/gameplay are dominated by explicit same-purpose option conflicts. BG1
content/fixes, core, endgame, and most randomisation entries are parent-scoped. BG1 NPCs,
collection, compatibility, difficulty, engine, fixes, interface, kits, and NPCs retain
their authored `requires` constraints. The remaining cross-source conflicts are confined
to fixes, gameplay, kits, NPCs, and randomisation; they were inspected and left explicit
rather than recast as choice groups.

## Bounded findings

- Flat features without a recipe parent are not treated as dependency defects. Their
  source/group metadata supplies the missing presentation context without changing
  resolution behavior.
- Existing cross-source requirements and incompatibilities remain authoritative. The
  audit found no safe basis for adding further dependency edges from catalog prose alone.
- The three legacy-split BG1 NPC choice sets had clear authority in the explicit curation
  map. Their alternatives now share the existing default's `mod:bg1npc` parent and
  `bg1-npcs` category, and every option carries the same symmetric group conflicts.
- Some retained labels necessarily use mod vocabulary (kit names, WeiDU component names,
  spell-system names, and NPC/content proper nouns). They were not broadly rewritten
  without authoritative explanatory copy.
- Grouping is presentation-only. Install order, components, decisions, readiness,
  recommended defaults, and runplan inputs remain unchanged.

## Verification and remaining acceptance

- Frontend typecheck, all 109 tests, and production web build passed. Tests cover
  atomic dropdown switching, None with recommended defaults, disabled alternatives,
  source search, legacy fallback, diagnostics and log-based BuffBot guidance.
- 23 focused Python authoring/curation tests passed. Rust recipe/manifest/resolver
  checks passed (19/52/9); actual curated-recipe checks retain 43 runs and 434 default
  components, BuffBot last, and isolate stacking/BG1 NPC choice changes.
- Native command contracts: 39 passed; consistency checks: 3 passed. No game or full
  installation was run. No browser/native-window visual acceptance was performed.
- The broad native integration-test command encounters the existing updater harness
  `tauri::test` feature requirement; explicitly scoped command-contract/lib checks ran
  successfully. Native updater apply/restart acceptance remains separate.
- BuffBot community report remains open pending Christopher's own installation test.
  A successful default-path test will not establish what happened on the tester's PC.

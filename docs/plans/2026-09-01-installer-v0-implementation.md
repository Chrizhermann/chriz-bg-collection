# Installer v0 implementation plan

**Goal:** prove the curated-selection and update UX end to end without mutating a game or
pretending the incomplete engine can install one.

**Architecture:** a framework-neutral static prototype establishes the interaction and
visual vocabulary. Rust then gains an immutable recipe-ledger parser and update
classification. After the engine's Task 6–8 work and manifest UI-rule design are stable,
a Tauri shell calls engine-owned projections for the same screens. Production installation
stays disabled until the Phase 1 orchestrator exists.

All Rust tasks use TDD. Cargo runs from PowerShell. Nothing in this plan reads or writes
`C:\Games`.

## Task 1: Static interaction prototype

**Files:**

- Create `docs/prototypes/installer-v0/index.html`
- Create `docs/prototypes/installer-v0/README.md`

Implement three responsive views with plain HTML, CSS, and minimal JavaScript: returning
home, Setup/Review, and Update Center. Demonstrate all four curation decisions, a radio
choice group, SCS `4240` disabled by Spell Revisions with an explanation, the campaign
ledger rail, and an installed copy whose update is marked “next playthrough.”

The prototype must work from `file://`, require no network assets, preserve keyboard
operation, expose a visible focus state, and honor `prefers-reduced-motion`. Add a small
self-check script that asserts unique element ids and that every interactive control has a
label. Commit: `docs: prototype installer v0 flow`.

## Task 2: Recipe release-ledger types

**Files:**

- Create `engine/src/updates.rs`
- Modify `engine/src/lib.rs`
- Create `engine/tests/updates.rs`
- Create `engine/tests/fixtures/updates/2026.09.0.toml`

Add serde types for `RecipeRelease`, `RecipeChange`, `SaveApplicability`, and `Urgency`.
Use `deny_unknown_fields`, path-bearing parse errors, and exhaustive enums. Load one
versioned TOML ledger and verify the declared manifest digest is a 64-character hexadecimal
value.

Tests first:

- parses a complete ledger fixture;
- rejects unknown fields and enum values;
- reports the ledger path on parse failure;
- rejects an invalid manifest digest;
- rejects duplicate change ids.

Commit: `engine: parse recipe update ledgers`.

## Task 3: Update classification

**Files:**

- Modify `engine/src/updates.rs`
- Extend `engine/tests/updates.rs`

Add a pure classifier over an installed recipe version and one or more later immutable
ledgers. It returns the accumulated changes and one presentation disposition:
`UpToDate`, `DeferredForNextPlaythrough`, `MayAffectCurrentPlaythrough`,
`UnknownApplicability`, or `AppUpdateRequired`.

Tests first:

- all `next_playthrough`/`new_game_only` changes classify as deferred;
- any `current_save` or conditional-before-event change prevents the deferred claim;
- `unknown` remains explicit;
- urgency is preserved and never inferred from applicability;
- skipped releases accumulate in order without duplicate change ids;
- a minimum app version newer than the running app wins as a prerequisite;
- a broken `supersedes` chain is rejected.

Do not inspect saves. Commit: `engine: classify recipe update impact`.

## Task 4: Define the engine-owned recipe projection

**Files:**

- Create `docs/plans/installer-recipe-view-contract.md`
- Modify `engine/src/manifest.rs`
- Modify `engine/src/resolve.rs`
- Add focused tests in `engine/tests/manifest_parse.rs` and `engine/tests/resolve.rs`

Before code, specify `RecipeView`, `SelectionEvaluation`, authored descriptions, grouping,
and conditional availability. Keep old fixture manifests valid through optional fields.
Express compatibility such as “unavailable with Spell Revisions” as data evaluated in
Rust. Do not encode mod ids or component numbers in generic engine branches.

Tests first must cover optional metadata defaults, one authored incompatibility, normalized
selection after a control becomes unavailable, and the returned human-readable reason.
Commit only after independent review of the contract. Commit: `engine: expose curated recipe view`.

## Task 5: Scaffold the Tauri Recipe Preview

**Prerequisites:** installer-v0 plan Tasks 2–4 reviewed; engine Phase 1 Tasks 6–8 completed
and reviewed; the recipe-view contract approved; a frontend framework selected with a short
recorded rationale.

**Files:** create `app/` with the selected Tauri 2 web frontend and Rust command layer.

Expose only `load_recipe`, `evaluate_selection`, `resolve_preview`, and fixture-backed
`check_updates`. Port the prototype's tokens and components; do not connect a functional
Build button. The Update Center reads a bundled fixture through the Rust parser.

Tests:

- Rust command tests for success and serialized path-bearing errors;
- frontend unit tests for decision mapping and unavailable reasons;
- keyboard traversal and accessible-name checks;
- update presentation tests for deferred, unknown, and app-required cases;
- viewport checks at 320 CSS pixels and 200% zoom.

Commit: `app: add non-destructive recipe preview`.

## Task 6: Connect later engine capabilities without redesign

This task is intentionally deferred. As Phase 1 lands, map session, events, orchestrator,
and report contracts to Build/Complete screens. Each connection gets its own TDD task.
Never replace the fixture Build screen with a real action until preflight, acquire, stage,
runner, per-run verification, resume, and diagnostics are all present and reviewed.

## Verification before calling v0 complete

From PowerShell:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Also run the prototype self-check and the chosen frontend's unit/accessibility checks. A
green Recipe Preview is not evidence of a real game install; the handoff must say exactly
which screens remain fixture-driven.

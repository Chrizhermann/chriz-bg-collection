# Installer v0 — campaign builder and update center

**Date:** 2026-09-01
**Status:** Working v0 design, authorized for implementation.
**Builds on:** [the approved installer-app design](2026-08-19-installer-app-design.md).

This document makes the first UI slice concrete without committing the project to a
product name, frontend framework, or finished visual identity. Those decisions can wait.
The v0 must prove that the curated recipe is understandable and that update messaging is
honest before the installer is allowed to copy games or run WeiDU.

## Product shape

Use a guided **campaign build** wizard, not a general mod-manager cockpit. The happy path
remains three decisions: find the games, accept the recommended setup, and review the
build. A returning-user home screen lists installed copies from their install reports;
an `Updates` action is always visible.

Alternatives considered:

1. A single scrolling build sheet is quicker to prototype, but becomes intimidating once
   path validation, compatibility explanations, and manual downloads appear.
2. A full component-tree cockpit is powerful, but contradicts the curated-installer scope.
3. The guided wizard takes a few more small screens and best matches the target audience.

The guided wizard is the v0 choice.

## Information architecture

The global shell shows the working collection name and recipe version, `Updates`, and
help/diagnostics. First-time users enter the wizard directly. Returning users first see
one card per successful `install-report.json`.

| Screen | Purpose | Primary action |
|---|---|---|
| Welcome | Explain EET requirements, disk cost, and copy-not-mutate behavior. | Find my games |
| Games | Show BG1+SoD and BG2 paths, detected builds, and pass/fail reasons. | Continue |
| Destination | Choose an isolated copy and explain its distinct save directory. | Continue |
| Setup | Start from Chris's recommended collection; optionally expose curated controls. | Review setup |
| Review | Show target, phases, selections, warnings, and manual-download needs. | Build collection |
| Build | Stream the append-only campaign ledger and stop on any failure. | Retry or export diagnostics |
| Complete | Launch through InfinityLoader and link the report/save location. | Launch |

The production `Build collection` action remains unavailable until session, acquire,
stage, runner, verification, and orchestrator tasks exist. A fixture-driven Build screen
is allowed only when visibly labeled as a developer preview.

## Curation semantics in the UI

The Rust engine owns selection and compatibility. The frontend renders its projection;
it does not reproduce dependency logic.

| Catalog decision | UI behavior |
|---|---|
| blank | Not exposed and not selected. |
| `optional` | Visible checkbox, off by default. |
| `default` | Visible checkbox, on by default. |
| `mandatory` | Added automatically when its parent is active; no standalone checkbox. |

Mutually exclusive choices use radio controls. An unavailable option may remain visible
when the reason helps the user understand the setup, but it is disabled and carries an
authored explanation. For example, SCS `4240` remains visible but unavailable while Spell
Revisions is selected. The UI never exposes raw component numbers as the main label.

## Installer state model

```text
booting
  +-- recipe-invalid
  +-- ready
       +-- configuring
       +-- review-ready
       +-- running
            +-- waiting-for-manual-download
            +-- failed-resumable
            +-- complete
```

Errors are persistent cards with a recovery action, not transient toasts. A failed WeiDU
run offers retry and diagnostics export, never skip.

The Build screen's signature element is a **campaign ledger rail**:

```text
BG1 preparation -> EET merge -> Main campaign -> Post-EET
```

It represents the actual append-only phases and can later show per-run progress without
turning the primary UI into a console.

## Update model

The UI must keep three independent properties separate:

1. **Target:** app, recipe, or an installed copy.
2. **Save applicability:** `app_only`, `current_save`, `before_npc_join`,
   `before_area_visit`, `before_event`, `next_playthrough`, `new_game_only`, or `unknown`.
3. **Urgency:** `critical`, `recommended`, `optional`, or `informational`.

Applicability is authored for every recipe change. The app must not inspect a save or
guess whether an NPC has joined or an area has been visited. Urgency remains independent:
a new-game-only crash fix can still be important, while an ordinary NPC-class change can
remain informational.

If every recipe change since an installed copy is `next_playthrough` or `new_game_only`,
show the copy as deferred:

> Available for your next playthrough. Your current save will not benefit.

The action is `Build updated copy`, never `Update installation`. A recipe release can be
selected for the next build without touching a working copy. App updates use the signed
Tauri updater and a separate `Restart to update` action. A recipe that requires a newer
app makes that app update a prerequisite.

Offline checks are nonfatal and show the last successful check time. A digest mismatch is
an error; the app never silently substitutes a changed recipe.

### Installation receipt

The future `install-report.json` is the canonical receipt. The app stores only an index
of report paths. It must record at least:

```text
install_id, target_path, engine_name, created_at
app_version, engine_version, game_build
recipe { id, version, digest }
selection, resolved_plan_digest
source versions and sha256 values
final outcome
```

### Versioned recipe ledger

Each recipe release carries an immutable ledger, rather than appending to one mutable
changelog. This lets an installed copy safely aggregate changes across skipped releases.

```toml
schema = 1
recipe_id = "chriz-eet"
version = "2026.09.0"
manifest_sha256 = "..."
published_at = "2026-09-01T00:00:00Z"
minimum_app_version = "0.1.0"
supersedes = "2026.08.0"

[[changes]]
id = "kivan-archer"
title = "Kivan now starts as an Archer"
summary = "The recommended companion setup now applies the Archer kit semantically."
save_applicability = "before_npc_join"
urgency = "informational"
condition_note = "Applies only before Kivan is first instantiated."
```

Release CI must reject changed recipe content without a corresponding, classified ledger
entry. A manifest diff is a coverage guard, not the source of save-applicability claims.

## Engine boundary

The existing engine already owns `Manifest`, validation, `Selection`, resolution, and
serialized `EngineEvent`s. Tauri should expose UI projections instead of leaking all
internal manifest structures:

```text
load_recipe(path) -> RecipeView
evaluate_selection(selection) -> SelectionEvaluation
resolve_preview(selection) -> PlanSummary
check_updates(receipt?) -> UpdateStatus

start_install(request) -> RunId       # only after the orchestrator exists
resume_install(target) -> RunId       # only after session persistence exists
export_diagnostics(target) -> path    # only after reports exist
```

`SelectionEvaluation` returns a normalized selection, each control's enabled state and
authored reason, findings, and the plan summary. The current schema does not yet express
all UI descriptions and conditional availability; that is engine-owned follow-up, not
frontend hardcoding.

## Visual and accessibility direction

The interface should resemble a careful campaign-build ledger, not copied game UI and not
faux parchment. Use blackened iron (`#151819`), charcoal (`#23282A`), slate (`#343B3E`),
vellum (`#F1E7D2`), tarnished brass (`#C59A51`), arcane teal (`#67AAA4`), and ember
(`#D36B58`). Headings may use a restrained book serif; body copy must remain highly
legible; paths and logs use monospace. Bundle fonts only after license review.

Baseline requirements: WCAG AA contrast, keyboard-complete controls, visible focus,
44-pixel primary targets, text plus icon plus color for status, 200% zoom without workflow
horizontal scrolling, reduced motion, and polite announcements for phase changes only.
Console auto-scroll can be paused and console text can be copied.

## V0 stopping point

The first real app slice is a non-destructive **Recipe Preview**:

1. Load and validate a local fixture recipe through Rust.
2. Render default and optional toggles plus choice groups.
3. Re-evaluate changes in Rust and show unavailable reasons.
4. Show the resolved ordered plan on Review.
5. Show an Update Center from a bundled ledger fixture.
6. Exercise the visual system with a fixture-only campaign ledger.

No game discovery, copying, downloads, network update checks, or real Build action belong
in this slice. A framework-neutral static prototype can precede the Tauri scaffold, so the
visual and interaction decisions do not prematurely select Svelte, React, or another web
framework.

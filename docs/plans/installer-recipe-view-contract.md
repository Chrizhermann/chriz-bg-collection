# Installer recipe-view contract

Status: approved real-alpha engine contract.

This contract separates curated player choices from WeiDU component numbers. The engine is the
only layer that evaluates compatibility, normalizes selection state, and expands features into an
exact install plan. The frontend renders `RecipeView`, submits semantic feature and input IDs, and
never implements compatibility rules or selects numeric components.

## Projection and order

`RecipeView` contains visible `FeatureControl` values in manifest authoring order. Features marked
`excluded` are absent. Category IDs appear in the order in which their first visible feature
appears; features are not regrouped or alphabetized by the engine.

Each control carries its semantic ID, authored text, category, decision, readiness, parent, current
effective state, interactivity, unavailable reason, and typed inputs. Component references remain
engine-private and do not appear in a control.

## Decisions and readiness

- `excluded`: absent from the view, normalized selection, and plan.
- `optional`: visible, interactive when available, and unchecked by default.
- `default`: visible, interactive when available, and checked by default.
- `mandatory`: follows its parent when the parent is effective and has no independent control.

`ready` and `experimental` features may resolve. A `blocked` feature remains visible and retains its
semantic desired state and authored unavailable reason, but never contributes components. If a
blocked default would otherwise be selected, evaluation emits a nonfatal omission finding.

Parents and requirements gate effective selection. Conflicts are directional: when the conflict
target is effectively selected, the owning feature is unavailable and uses the exact authored
reason. In particular, SCS 4240 uses `Unavailable while Spell Revisions is selected.` and
Randomiser 10300 conflicts with SCS 8040.

## Semantic selection and presets

Feature state is keyed by the stable feature ID. Typed input state is keyed by
`feature-id/input-id`. A `NormalizedSelection` stores typed values by those IDs, so identity
survives view reordering and reevaluation. Presets contain the same semantic keys and values; they
do not contain component numbers.

Inputs are one of:

- Boolean, with an authored Boolean default.
- Choice, with ordered semantic options and an authored option-ID default.
- Integer, with inclusive minimum and maximum bounds and an in-range default.

Optional-only choice groups explicitly include and default to a semantic `none` option. The current
curation has seven such groups.

## Exact additive expansion

Resolution starts with zero components. Only components owned by effective ready or experimental
features are added. Run order and component order always follow the manifest; a run with no selected
components is omitted.

The approved atomic expansions include:

- SoD Remix: `100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 197, 187, 200, 210, 215, 220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 900`.
- Tempus: `400, 401, 404, 405, 407, 408`.

SoD Remix component 290 is not part of the approved expansion.

## Prompt scripts

A component may own ordered `PromptStep` values. Each step names expected installer output and an
answer that is either a typed literal or a reference to a validated feature input. Evaluation
renders the selected component's prompt script one step at a time, including a terminating newline
in each answer. It neither concatenates an eager stdin stream nor starts or writes to a process.

## Static validation

Before evaluation, the engine rejects:

- missing or duplicate semantic IDs, missing parents/requirements/conflict targets, mandatory
  features without parents, and component references outside the named run;
- cycles through parent and requirement edges;
- directly conflicting features that are both defaults;
- duplicate ownership of one exact `(run_id, component)`;
- duplicate or malformed input IDs, invalid choice defaults/options, and invalid integer bounds or
  defaults;
- prompt references to missing feature inputs; and
- visible blocked features or compatibility rules without authored unavailable reasons.

Selection evaluation rejects unknown semantic keys, invalid typed values, and unsupported
platforms. These failures occur before an install process is started.

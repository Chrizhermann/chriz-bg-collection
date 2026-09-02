# Installer v0.1 Real Alpha Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:executing-plans to implement this plan task-by-task.

**Goal:** Build, package, and rehearse a Windows x64 installer that creates a verified,
resumable EET 2.7.3 installation from clean user-owned games and a signed curated recipe.

**Architecture:** Evolve the existing synchronous Rust engine into the sole owner of recipe
resolution, game discovery, acquisition, staging, WeiDU execution, recovery, receipts, and
updates. A small Tauri 2 + Vite + vanilla TypeScript application renders engine projections
and events. Curation/source freezing proceeds alongside engine and UI work, then all three
converge in Christopher's cold-cache release-candidate installation.

**Tech Stack:** Rust 1.97.1 stable-msvc, serde/toml/serde_json, clap, ureq 3.4 with rustls
platform verification and Windows system-proxy discovery, sha2, minisign-verify 0.2.5,
zip, walkdir, fs4, windows-sys, pelite, Tauri 2, Vite, vanilla TypeScript, Vitest,
Testing Library, axe-core, and WebdriverIO for one packaged-app smoke flow.

---

## Execution rules

- Work only in isolated git worktrees. Never modify either protected reference directory
  under `C:\Games`.
- Run every Cargo command from PowerShell. Preserve unrelated and user-authored dirty work.
- Use TDD for every behavior: focused RED, minimal implementation, focused green, broader
  green, then one logical commit.
- Tasks 1, 17, 18, and 25 are tracked as integration checkpoints, but their explicitly
  named slices are separate RED/GREEN commits. Never batch an entire checkpoint into one
  unreviewable change.
- Load the `bg-modding` skill for Tasks 4–7, 13, 16, 18, and 26. Load
  `frontend-design` for Tasks 21–24, `ai-ux-testing` for Task 25, and
  `verification-before-completion` before every milestone/release claim.
- Public recipes contain no all-zero hashes, moving branch URLs, FAIL placeholders,
  bundled third-party archives, or silently omitted promised defaults.
- Real installs never use `--quick-log`, never skip a failed run, and never mutate a
  source/store game.
- No push, tag, release, or public updater change occurs until the corresponding final task
  is explicitly authorized and `gh auth status` shows `Chrizhermann`.

## Parallel delivery waves

```text
Wave A:  0 curation snapshot   1 schema v2   21 app scaffold
Wave B:  2 recipe view   4 invocation -> 5 log verify   8 discovery   11 download/cache
Wave C:  3 ledger   6 runner -> 7 real WeiDU   9 stage -> 10 locks
         12 extraction/materialize   22 wizard
Wave D:  13 orchestrator -> 14 receipts -> 15 CLI
Wave E:  16 BG1 capture + 17 curation coverage -> 18 production freeze -> 19 release gate
Wave F:  23 bridge   20 signed recipe -> 24 updates
Wave G:  25 packaged CI/rehearsal -> 26 creator RC and public alpha
```

Tasks whose dependencies are satisfied may run in parallel, but each agent uses its own
worktree and reports a commit for review before integration. The co-located `.tasks.json`
file is authoritative for exact dependency edges and resume status.

### Task 0: Preserve and integrate the authoritative curation snapshot

**Files:**

- Import from the dirty `main` checkout: `docs/curation/components/*.md`
- Import: `docs/next-session.md`
- Import: `docs/pin-list-2.7.md`
- Import: `manifest/mod-sources.tsv`
- Reconcile, do not overwrite: `docs/handover.md`
- Reconcile, preserving Task 6 completion: `docs/plans/2026-08-20-engine-phase1-implementation.md.tasks.json`

**Step 1: Record the exact source state**

Run in `C:\src\private\chriz-bg-collection`:

```powershell
git status --short --branch
git diff --name-status
git ls-files --others --exclude-standard
```

Expected: the curated component catalogs plus `FOLLOW_UPS.md`,
`COLLECTION_TAIL_FIXES.md`, and `next-session.md`; `.claude/` is present but out of scope.

**Step 2: Create a preservation worktree**

Create `codex/curation-snapshot` from `main` in a new external worktree. Reproduce the
tracked diff and three intended untracked docs with `apply_patch`. Do not copy or stage
`.claude/`, nested worktrees, generated state, or unrelated files.

**Step 3: Verify semantic coverage before committing**

Run:

```powershell
rg -n "^(#|##)|\|.*\b(blank|optional|default|mandatory)\b" docs/curation/components
git diff --check
git status --short
```

Expected: all 44 edited catalogs and the two new curation indexes are present; no `.claude`
path is staged.

**Step 4: Commit the preservation snapshot**

```powershell
git add -- docs manifest/mod-sources.tsv
git commit -m "docs: preserve completed component curation"
```

**Step 5: Integrate into this implementation branch**

Cherry-pick or merge the preservation commit. Resolve `handover.md` by retaining both the
current engine status and the curation queues. Keep the engine branch's completed Task 6
record. Run `git diff --check` and `cargo test --workspace` before committing the
reconciliation as `docs: integrate curation and engine work`.

### Task 1: Replace manifest v1 with the executable recipe-v2 contract

**Files:**

- Modify: `engine/src/manifest.rs`
- Modify: `engine/src/loader.rs`
- Modify: `engine/src/validate.rs`
- Modify: `engine/src/resolve.rs`
- Modify: `engine/src/error.rs`
- Modify: `engine/tests/manifest_parse.rs`
- Modify: `engine/tests/manifest_load.rs`
- Modify: `engine/tests/manifest_validate.rs`
- Modify: `engine/tests/resolve.rs`
- Replace fixtures under: `engine/tests/fixtures/manifest/`
- Create fixtures under: `engine/tests/fixtures/manifest/artifacts/`
- Create fixtures under: `engine/tests/fixtures/manifest/presets/`

No recipe has shipped, so bump directly to schema 2; do not maintain a v1 migration layer.

**Step 1: Write failing parse tests for artifacts, installers, and runs**

Add a fixture in which one EE Fixpack artifact supplies one installer and two explicit
runs. Assert the shape:

```rust
assert_eq!(manifest.artifacts.len(), 2); // mod + WeiDU tool
assert_eq!(manifest.collection.runs[0].run_id, "eefixpack-bg1");
assert_eq!(manifest.collection.runs[0].phase, Phase::Bg1Preparation);
assert_eq!(manifest.collection.runs[1].run_id, "eefixpack-bg2");
assert_eq!(manifest.collection.runs[1].phase, Phase::Bg2Preparation);
assert_eq!(manifest.collection.runs[0].components, vec![0, 2]);
```

Also assert that EET and EET_END can reference one shared artifact and that Artisan main,
NPC, and tweak installers can share one artifact while keeping distinct TP2 paths.

**Step 2: Run the focused tests to prove RED**

```powershell
cargo test -p chriz-bg-engine --test manifest_parse --test manifest_load
```

Expected: FAIL because schema 2 and the new directories/types are unknown.

**Step 3: Implement the minimal schema**

Use these core types (with `Serialize`, `Deserialize`, `deny_unknown_fields`, and doc
comments on every public item):

```rust
pub enum GameRoot { Bg1, Bg2 }

pub enum Phase {
    Bg1Preparation,
    Bg2Preparation,
    EetInitialization,
    Main,
    EetFinalization,
    PostEetEnd,
}

pub enum RunArg {
    Literal(String),
    StagedRoot(GameRoot),
}

pub enum InvocationMode { SetupName, ExplicitTp2 }

pub struct Artifact {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: Source,
    pub archive: ArchiveSpec,
    pub acquisition: AcquisitionPolicy,
    pub provenance: Provenance,
}

pub struct ModFile {
    pub id: String,
    pub artifact_id: String,
    pub name: String,
    pub tp2: String,
    pub language: u32,
    pub weidu_artifact_id: String,
    pub invocation_mode: InvocationMode,
    pub components: Vec<Component>,
}

pub struct Run {
    pub run_id: String,
    pub mod_id: String,
    pub phase: Phase,
    pub components: Vec<u32>,
    pub args: Vec<RunArg>,
}

pub struct ComponentRef {
    pub run_id: String,
    pub component: u32,
}
```

`Phase::game_root()` derives BG1 only for `Bg1Preparation`; all other phases target BG2.
Load `collection.toml`, `artifacts/*.toml`, `mods/*.toml`, and `presets/*.toml` in sorted
case-insensitive `.toml` order. Preserve path-bearing errors and id/file-stem checks.

**Step 4: Make the parser slice green and commit it**

```powershell
cargo test -p chriz-bg-engine --test manifest_parse --test manifest_load
git add engine
git commit -m "engine: parse executable recipe v2"
```

**Step 5: Write and run failing validation tests**

Cover globally unique `run_id`; all run components explicit/nonempty; referenced
artifact/tool/installer/component existence; relative traversal-free TP2/archive paths;
duplicate component forbidden on one game root but permitted across BG1/BG2; monotonic
phase order; EET initialization first after preparation; EET finalization last before only
explicit post-EET tail runs; no silent platform filtering.

```powershell
cargo test -p chriz-bg-engine --test manifest_validate
```

Expected: each new case fails before its rule exists.

**Step 6: Implement validation and update resolution identity**

`PlannedRun` carries `run_id`, `mod_id`, derived `target`, phase, exact ordered components,
typed args, artifact id, and WeiDU artifact id. Hashing/selection comes in Tasks 2–3.

**Step 7: Run the complete engine suite**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all migrated existing tests and new schema-v2 tests pass.

**Step 8: Commit the validation/resolution slice**

```powershell
git add engine
git commit -m "engine: validate executable recipe v2"
```

### Task 2: Add additive curation, presets, typed inputs, and UI projections

**Files:**

- Create: `docs/plans/installer-recipe-view-contract.md`
- Create: `engine/src/recipe_view.rs`
- Modify: `engine/src/manifest.rs`
- Modify: `engine/src/resolve.rs`
- Modify: `engine/src/lib.rs`
- Create: `engine/tests/recipe_view.rs`
- Extend: `engine/tests/manifest_validate.rs`
- Extend: `engine/tests/resolve.rs`

**Step 1: Specify the engine-owned contract**

Document `RecipeView`, `SelectionEvaluation`, category/order rules, normalized semantic
selection, authored disabled reasons, and exact component expansion. The frontend never
reimplements compatibility rules or selects numeric components.

**Step 2: Write the failing semantic matrix**

Use fixtures for all settled meanings:

```rust
assert!(!view.control("blank-component").is_present());
assert!(!view.control("optional-feature").unwrap().selected);
assert!(view.control("default-feature").unwrap().selected);
assert!(!view.control("mandatory-child").unwrap().interactive);
assert_eq!(plan.components_for("sod-remix"), EXPECTED_THIRTY_COMPONENTS);
assert_eq!(plan.components_for("tempus-rework"), [400, 401, 404, 405, 407, 408]);
assert_eq!(view.control("scs-4240").unwrap().unavailable_reason.as_deref(),
           Some("Unavailable while Spell Revisions is selected."));
```

Also cover Randomiser 10300 versus SCS 8040, a blocked default with an omission finding,
selection identity surviving reevaluation, and the seven optional-only choice groups using
an explicit `none` default.

**Step 3: Run to prove RED**

```powershell
cargo test -p chriz-bg-engine --test recipe_view --test resolve
```

Expected: FAIL because controls/readiness/inputs do not exist.

**Step 4: Implement additive feature resolution**

```rust
pub enum Decision { Excluded, Optional, Default, Mandatory }
pub enum Readiness { Ready, Experimental, Blocked }
pub struct Feature {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub decision: Decision,
    pub readiness: Readiness,
    pub parent: Option<String>,
    pub components: Vec<ComponentRef>,
    pub requires: Vec<String>,
    pub conflicts: Vec<Conflict>,
    pub inputs: Vec<InputSpec>,
}
pub enum InputSpec {
    Boolean { id: String, default: bool },
    Choice { id: String, default: String, options: Vec<InputOption> },
    Integer { id: String, default: i64, min: i64, max: i64 },
}
```

Start from no components, then add only effective ready/experimental features. Mandatory
children follow their parent and have no control. Blocked features never resolve, retain
their semantic identity, and return their authored reason. Validate cycles, missing
parents, conflicting defaults, duplicate component ownership on one run, and input bounds.

**Step 5: Add prompt rendering without eager stdin**

`Component` owns ordered `PromptStep { expected_output, answer }` values. Answers are
typed literals or references to validated feature inputs. Rendering returns a prompt
script; it does not concatenate bytes or write to a process.

**Step 6: Run targeted and full verification**

```powershell
cargo test -p chriz-bg-engine --test recipe_view --test resolve --test manifest_validate
cargo test --workspace
```

Expected: all mappings and normalization cases pass.

**Step 7: Commit**

```powershell
git add docs/plans/installer-recipe-view-contract.md engine
git commit -m "engine: expose curated recipe selection"
```

### Task 3: Replace snapshot resume with a create-once campaign ledger

**Files:**

- Create: `engine/src/digest.rs`
- Rewrite: `engine/src/session.rs`
- Modify: `engine/src/error.rs`
- Modify: `engine/src/lib.rs`
- Rewrite: `engine/tests/session.rs`
- Create fixtures under: `engine/tests/fixtures/session/`

**Step 1: Write failing journal tests**

Cover create, append, replay, sequence gaps, changed previous-record hash, truncated temp
record, duplicate sequence, changed recipe/selection/plan/source identity, and a crash with
the last event `StepStarted`.

```rust
let store = SessionStore::create(&managed_root, created_event())?;
store.append(SessionEvent::StepStarted { step_id: "install:eet".into(), attempt: 1 })?;
let replay = SessionStore::open(&managed_root)?.replay()?;
assert_eq!(replay.unresolved_step(), Some("install:eet"));
```

The final assertion deliberately differs from current behavior: loading does not convert
Running to Failed before evidence reconciliation.

**Step 2: Run to prove RED**

```powershell
cargo test -p chriz-bg-engine --test session
```

Expected: FAIL because the current implementation replaces one `session.json` snapshot.

**Step 3: Implement immutable record files**

Use this layout:

```text
<managed-root>/.chriz/
  recipe/payload.zip
  recipe/envelope.json
  ledger/0000000000.json
  ledger/0000000001.json
  attempts/<attempt-id>/
```

Each `LedgerRecord { sequence, previous_sha256, recorded_at, event }` is serialized to a
new `.tmp`, flushed and synced, renamed to its never-before-used final sequence path, then
the ledger directory is synced where supported. Replay verifies the complete hash chain.
Ignore abandoned `.tmp` files; never overwrite a final record.

**Step 4: Freeze every consequential identity**

The `Created` event records install id, managed root, canonical cache root, exact signed
recipe bytes/digest, normalized selection, plan digest, source-game fingerprints,
tool/artifact identities, and both staged paths. Resume reopens and validates that exact
cache root; it never silently falls back to a default cache. Add pure canonical JSON digest
helpers for selection and plan.

**Step 5: Run tests and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test session
cargo test --workspace
git add engine
git commit -m "engine: persist append-only campaign ledger"
```

### Task 4: Build production-safe WeiDU invocations

**Files:**

- Create: `engine/src/weidu/mod.rs`
- Create: `engine/src/weidu/invocation.rs`
- Modify: `engine/src/lib.rs`
- Add unit tests in: `engine/src/weidu/invocation.rs`

**Step 1: Write exact failing argument tests**

Assert the normal argument vector includes:

```text
--language <authored mod language>
--use-lang en_US
--force-install-list <exact remaining component ids>
--no-exit-pause
--skip-at-view
--safe-exit
--noautoupdate
--log <attempt log path>
```

Assert it does not contain `--quick-log`, `--yes`, `--reinstall`, or any uninstall flag.
Test BG1/BG2 cwd selection and that changing any component, argument, tool hash, root, or
log destination changes the invocation digest.

**Step 2: Add EET's typed dynamic-path test**

For its initialization run, assert the trailing group is exactly:

```text
--args-list sp <canonical staged BG1 root>
```

This is the documented EET auto-install path: `p` supplies the source and `s` suppresses
the unwanted desktop shortcut.

**Step 3: Prove RED**

```powershell
cargo test -p chriz-bg-engine weidu::invocation
```

Expected: FAIL because the module does not exist.

**Step 4: Implement setup-name emulation**

```rust
pub struct Invocation {
    pub program: PathBuf,
    pub cwd: PathBuf,
    pub args: Vec<OsString>,
    pub prompts: Vec<ResolvedPrompt>,
    pub debug_path: PathBuf,
    pub identity_digest: String,
}
```

By default, remove one case-insensitive leading `setup-` from the TP2 stem, derive
`Setup-<normalized-stem>.exe`, materialize the verified pinned WeiDU binary at the target
root under that name, and let WeiDU auto-select its TP2. Thus both `testmod.tp2` and
`setup-testmod.tp2` resolve to `Setup-testmod.exe`; declaring both is an ambiguity and must
fail validation. This preserves
`%MOD_FOLDER%`; neutral `weidu.exe path/to/mod.tp2` can incorrectly resolve it as
`weidu_external`. Permit explicit-relative-TP2 mode only when the signed recipe marks it
as tested. Reject absolute, UNC/device, ADS, or traversing recipe-supplied TP2 paths.
The log path is not recipe data: the engine generates an absolute canonical path and must
prove it is confined beneath the current create-once attempt directory before spawning.

**Step 5: Run and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine weidu::invocation
git add engine
git commit -m "engine: build safe WeiDU invocations"
```

### Task 5: Parse and reconcile ordered WeiDU.log evidence

**Files:**

- Create: `engine/src/weidu/log.rs`
- Create: `engine/src/weidu/verify.rs`
- Modify: `engine/src/weidu/mod.rs`
- Create: `engine/tests/weidu_verify.rs`
- Create fixtures under: `engine/tests/fixtures/weidu/`

**Step 1: Bake representative before/after/debug fixtures**

Include descriptive annotations, blank/header lines, old `Recently Uninstalled` comments,
warning success, a partial multi-component run, an unexpected addition, and a removal. Do
not read a live game during tests.

**Step 2: Write failing parser and reconciliation tests**

```rust
assert_eq!(parse_active_entries(AFTER)?.last().unwrap().annotation.as_deref(),
           Some("Expected component name"));
assert_eq!(reconcile(BEFORE, COMPLETE, DEBUG_OK, &run), Reconciliation::ProvenDone);
assert_eq!(reconcile(BEFORE, PARTIAL, DEBUG_PARTIAL, &run),
           Reconciliation::PartialPrefix { remaining: vec![10] });
assert_eq!(reconcile(BEFORE, DISTURBED, DEBUG_OK, &run), Reconciliation::StackDisturbed);
```

Also prove exit code zero cannot convert missing log evidence into success.

**Step 3: Run to prove RED**

```powershell
cargo test -p chriz-bg-engine --test weidu_verify
```

**Step 4: Implement ordered-prefix verification**

```rust
pub enum Reconciliation {
    ProvenDone,
    PartialPrefix { remaining: Vec<u32> },
    UnchangedRetryable,
    StackDisturbed,
    Ambiguous,
}
```

The active before sequence must remain an exact prefix. The appended tail must match
expected TP2 key, language, and components in order. Pure installs reject newly introduced
uninstall comments. Normalize TP2 identity case-insensitively while preserving path and
line evidence. `ProvenDone` requires both the exact log tail and complete terminal debug
markers. Warning installs are success only with exact additions and `INSTALLED WITH
WARNINGS`; incomplete debug evidence is `Ambiguous`.

**Step 5: Run and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test weidu_verify
git add engine
git commit -m "engine: verify ordered WeiDU log changes"
```

### Task 6: Stream and supervise WeiDU without guessing at prompts

**Files:**

- Create: `engine/src/weidu/runner.rs`
- Create: `engine/src/bin/mock-child.rs`
- Modify: `engine/src/events.rs`
- Modify: `engine/src/weidu/mod.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/runner.rs`

**Step 1: Add the missing attention/control contract**

```rust
pub enum RunnerControl { ContinueWaiting, Cancel }
pub enum RunOutcome { Exited { code: i32 }, Cancelled, SpawnFailed }
```

Add `EngineEvent::AttentionRequired { step_id, reason, last_output }`. It is persistent UI
state, not a failure and not an instruction to kill the process.

**Step 2: Write failing process tests**

The mock child must support fragmented prompts, arbitrary bytes, simultaneous stdout and
stderr, silent delay, child-process spawning, and stdin echo. Test that:

- an answer is sent only after its authored expected output appears, including a prompt
  split across reads without a newline;
- an unmatched/silent prompt alerts once and remains alive until Continue or Cancel;
- a legitimately quiet child can continue and exit normally;
- no-prompt invocations receive closed stdin;
- invalid UTF-8 is display-decoded lossily while raw bytes remain in the attempt log;
- large simultaneous streams cannot deadlock;
- Cancel terminates the complete process tree.

**Step 3: Run to prove RED**

```powershell
cargo test -p chriz-bg-engine --test runner
```

**Step 4: Implement the supervisor**

Read byte chunks on independent threads, write raw logs directly, and send bounded display
chunks through a channel. Match only the next expected prompt against a rolling buffer.
After all prompts are answered, close stdin. A silence threshold emits attention and keeps
waiting for a control message or process exit; it never auto-kills.

On Windows, place the process in a Job Object configured with
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, so an application crash cannot orphan WeiDU or its
children. Only explicit cancellation terminates the job.
Enable only `Win32_Foundation`, `Win32_System_JobObjects`, and
`Win32_System_Threading` on `windows-sys` in `engine/Cargo.toml`.

**Step 5: Run and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test runner
cargo test --workspace
git add engine
git commit -m "engine: supervise WeiDU processes safely"
```

### Task 7: Prove the production invocation with real WeiDU

**Files:**

- Create: `engine/tests/fixtures/testmod/setup-testmod.tp2`
- Create payloads under: `engine/tests/fixtures/testmod/testmod/`
- Create: `engine/tests/weidu_integration.rs`
- Modify: `engine/Cargo.toml`
- Reuse: `engine/tests/support/fakegame.rs`

**Step 1: Write ignored end-to-end tests**

Build a temporary fake game, materialize a verified WeiDU 249 binary as
`Setup-testmod.exe`, run the production invocation and supervisor, then reconcile the real
logs. Fixture components prove `COPY_EXISTING`, multiple-component success, one matched
`ACTION_READLN`, a controlled failure after a successful prefix, and an unmatched prompt.
Use `pelite` to assert the pinned Windows tool is x64 before invoking it.

The fixture records `%MOD_FOLDER%`; assert setup-name mode resolves it to `testmod`, while
explicit-relative-TP2 mode is separately covered and never becomes the default.

**Step 2: Run the ignored test to establish its initial failure**

```powershell
$env:CHRIZ_WEIDU_EXE = 'C:\src\private\chriz-bg-rebalance\weidu.exe'
cargo test -p chriz-bg-engine --test weidu_integration -- --ignored --nocapture
```

Expected: FAIL before the fixture/integration wiring is complete. Do not point this at a
real game.

**Step 3: Implement only the harness glue**

Hash KEY, BIF, both TLKs, and the initial override tree. Assert success preserves
descriptive WeiDU.log annotations and changes only the explicit publication allowlist.
Assert a failing component rolls back its own writes, the successful prefix reconciles,
and an unexpected prompt requires attention until cancellation.

**Step 4: Run all engine gates**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p chriz-bg-engine --test weidu_integration -- --ignored --nocapture
```

Expected: the ordinary suite and all ignored real-WeiDU tests pass; no `C:\Games` path is
opened for writing.

**Step 5: Commit**

```powershell
git add engine/tests
git commit -m "engine: prove installer against real WeiDU"
```

### Task 8: Discover Steam/GOG games and fail closed on freshness

**Files:**

- Create: `engine/src/games/mod.rs`
- Create: `engine/src/games/discovery.rs`
- Create: `engine/src/games/profile.rs`
- Create: `engine/src/games/probe.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/games.rs`
- Create fixtures under: `engine/tests/fixtures/games/`

**Step 1: Define fixture-backed profiles and failing tests**

```rust
pub enum GameRole { BgeeSod, Bg2ee }
pub enum Storefront { Steam, Gog }
pub struct GameCandidate {
    pub role: GameRole,
    pub storefront: Storefront,
    pub root: PathBuf,
    pub build: Option<String>,
    pub eligibility: Eligibility,
    pub findings: Vec<GameFinding>,
    pub fingerprint: Option<String>,
}
```

Test Steam library VDF discovery, GOG registry discovery, Browse inspection, multiple
libraries, 2.7.3 with a modified core file, missing SoD, stale DLC Merger residue, unknown
fingerprint, and two simultaneous findings. Assert GOG 2.7.3 is `Experimental`, not
silently promoted to verified.

**Step 2: Prove RED**

```powershell
cargo test -p chriz-bg-engine --test games
```

**Step 3: Implement discovery behind injectable providers**

Use provider traits for registry and filesystem access so tests never depend on the host.
Steam candidates come from the Steam registry root plus `libraryfolders.vdf` app manifests;
GOG candidates come from installed-game registry records. Profiles—not generic branches—
own executable names, product version, role/storefront/locale, required fingerprinted
files, allowed clean variants, and forbidden residue patterns. Freshness also compares a
profiled inventory of mod-sensitive surfaces: `chitin.key`, relevant TLKs, `engine.lua`,
root setup/WeiDU files, override contents, and known mod directories. Any unexpected entry
or digest on those surfaces is `Modified` or `Unknown`, never Fresh.

Use `pelite` for ProductVersion on Windows. English (`en_US`) is the sole verified alpha
locale. There is no production `--assume-build` escape hatch.

**Step 4: Add one ignored read-only local probe**

The probe may inspect configured candidate paths but makes no writes. It should reproduce
the known evidence: clean Steam BG2EE 2.7.3 and modified Steam BGEE+SoD with stale DLC
Merger 1.7 residue.

**Step 5: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test games
git add engine
git commit -m "engine: discover and inspect source games"
```

### Task 9: Stage independent game copies and isolate save identity

**Files:**

- Create: `engine/src/stage.rs`
- Modify: `engine/src/error.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/stage.rs`

**Step 1: Write failing containment and filesystem-isolation tests**

Use temporary trees to reject source=target, target inside source, source inside target,
textual-prefix tricks, case variants, symlinks, junctions/reparse points, and hardlink
reuse. Test a successful managed layout:

```text
<managed-root>/
  bg1/
  game/
  .chriz/
```

Each destination regular file must have the source bytes but an independent file identity.

**Step 2: Prove RED**

```powershell
cargo test -p chriz-bg-engine --test stage
```

**Step 3: Implement resumable regular-file copying**

Canonicalize existing ancestors; never trust string prefixes. Inspect entries with
`symlink_metadata` and Windows reparse attributes before opening them. Use ordinary copy,
never hardlinks. For resume, retain a destination file only after size and SHA-256 match
the still-identical source; otherwise copy to a temporary sibling, sync, and publish.
Compare source identity before and after staging so a concurrent store update discards the
stage.

**Step 4: Add engine-name/save-root tests**

Resolve Documents with the Windows known-folder API. Generate a normalized unique name,
derive the exact save root, and reject collision with an unmanaged/different installation.
Patch BG1 immediately to prevent accidental shared saves. Patch BG2 again only after EET
finalization/tail runs, because EET may replace `engine.lua`; read it back before launch.

**Step 5: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test stage
git add engine
git commit -m "engine: stage isolated game copies"
```

### Task 10: Enforce disk, process, TLK, target, and cache locks

**Files:**

- Create: `engine/src/lock.rs`
- Create: `engine/src/preflight.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/lock.rs`
- Create: `engine/tests/preflight.rs`

**Step 1: Write failing exclusive-lock tests**

Acquire one lock for a managed target and one per cache digest. A second process/handle
must fail while the first is alive and succeed after it closes, including after the child
holding it exits unexpectedly. Use OS locks (`fs4`), not a stale sentinel file.

**Step 2: Write failing preflight tests**

Cover insufficient space, protected exact creator paths, non-writable destination, active
target executable, locked root TLK, locked `lang/en_US/dialog.tlk`, mismatched frozen
review token, and a clean pass. Process matching uses canonical executable paths under the
target, never names alone.

**Step 3: Prove RED**

```powershell
cargo test -p chriz-bg-engine --test lock --test preflight
```

**Step 4: Implement preflight and per-run recheck**

Use `fs4` for locks/free-space queries and a Windows process enumerator for executable
paths. On Windows, prove both TLKs can be opened without truncation and with share mode 0.
Initial preflight verifies sources, destination, total space, recipe/selection/plan
digests, required artifacts, tool pins, and relevant processes. Immediately before every
mutating WeiDU run, repeat target-process and TLK checks. App/recipe update installation
must refuse while the target lock is held.

The creator-only denylist contains the two exact canonical protected reference paths; it
does not reject arbitrary public `C:\Games` directories.

**Step 5: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test lock --test preflight
git add engine
git commit -m "engine: enforce install preflight and locks"
```

### Task 11: Download immutable artifacts into a content-addressed cache

**Files:**

- Create: `engine/src/acquire/mod.rs`
- Create: `engine/src/acquire/http.rs`
- Create: `engine/src/acquire/cache.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/acquire_http.rs`
- Create: `engine/tests/acquire_cache.rs`

**Step 1: Correct the HTTP dependency decision**

Enable both existing ureq 3.4 features and configure the client explicitly:

```toml
ureq = { version = "3.4", default-features = false,
         features = ["rustls", "platform-verifier", "win-system-proxy"] }
```

Configure rustls root certificates as `RootCerts::PlatformVerifier`. SHA-256 remains the
artifact identity; platform trust is transport authentication, not a substitute.

**Step 2: Write failing local-server tests**

Cover a complete download; verified cache hit; interrupted response; correct 206 resume
with strong ETag and Content-Range; changed ETag; 200 response to a range; invalid range;
HTTPS-to-HTTP redirect rejection; retry exhaustion; hash mismatch; and two concurrent
requests for one digest.

**Step 3: Prove RED**

```powershell
cargo test -p chriz-bg-engine --test acquire_http --test acquire_cache
```

**Step 4: Implement the cache protocol**

```text
<cache>/sha256/ab/<full-digest>.archive
<cache>/sha256/ab/<full-digest>.json
<cache>/partial/<request-id>.part
```

Stream with bounded buffers and progress events. Resume only when URL, expected length,
and strong validator match and the server returns the exact requested range. Otherwise
restart at byte zero. Hash and length-check the complete partial, sync it, acquire the
digest lock, then publish it without replacing a valid entry. Record original/final HTTPS
URLs, validators, length, hash, and timestamp. A failed partial never replaces a verified
cache object.

**Step 5: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test acquire_http --test acquire_cache
git add engine Cargo.lock
git commit -m "engine: acquire hash-pinned artifacts"
```

### Task 12: Extract, validate, and materialize mod payloads safely

**Files:**

- Create: `engine/src/acquire/archive.rs`
- Create: `engine/src/acquire/manual.rs`
- Create: `engine/src/acquire/materialize.rs`
- Modify: `engine/src/acquire/mod.rs`
- Modify: `engine/src/lib.rs`
- Create: `engine/tests/acquire_archive.rs`
- Create: `engine/tests/acquire_manual.rs`
- Create: `engine/tests/materialize.rs`
- Add fixtures under: `engine/tests/fixtures/archives/`

**Step 1: Write malicious-archive RED tests**

Generate ZIPs in tests that contain absolute/drive/UNC paths, `..`, alternate data
streams, reserved device names, trailing dots/spaces, symlinks, case-insensitive duplicate
paths, file/directory prefix collisions, excessive depth/count/size/compression ratio,
encrypted/unsupported entries, missing expected TP2, and two possible TP2 roots. Every
case must fail before publishing any extraction directory.

**Step 2: Write valid extraction/manual tests**

Cover `.zip` and `.iemod`, one GitHub wrapper directory, exact expected roots/TP2s, manual
archive selection with matching hash, and manual mismatch. Manual acquisition is an
explicit `provide_manual_archive(path)` call; never poll an arbitrary Downloads folder.

**Step 3: Run to prove RED**

```powershell
cargo test -p chriz-bg-engine --test acquire_archive --test acquire_manual --test materialize
```

**Step 4: Implement bounded extraction**

Validate the complete central directory and case-folded destination map before writing.
Extract regular files only into a new temporary directory, enforce recipe limits, verify
exact expected TP2 paths, sync, then publish under the artifact digest. Public mode rejects
all-zero hashes and SFX executables.

**Step 5: Implement declared materialization**

Publish only recipe-declared roots and TP2 files into the selected staged game. Skip
archive-supplied setup executables because Task 4 supplies the verified pinned binary.
Reject undeclared overwrites; permit an existing path only when bytes are identical or an
explicit signed collision rule names both owners and expected input/output hashes. Record
every published path. Reuse one extraction for split artifacts and allow the same artifact
to be materialized independently to BG1 and BG2.

For each destination file, write and sync a uniquely named temporary sibling, then publish
without replacing an unverified file. Persist a deterministic publication manifest with
relative path, owner, length, and SHA-256. After a crash, remove only engine-owned temporary
siblings and reconcile the already-published prefix by hash before continuing; never treat
a truncated or unknown destination as a harmless overwrite.

**Step 6: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test acquire_archive --test acquire_manual --test materialize
cargo test --workspace
git add engine
git commit -m "engine: materialize verified mod payloads"
```

### Task 13: Orchestrate the complete two-root build and reconcile crashes

**Files:**

- Create: `engine/src/orchestrator.rs`
- Modify: `engine/src/lib.rs`
- Extend: `engine/src/events.rs`
- Create: `engine/tests/orchestrator.rs`

**Step 1: Define injected stage interfaces**

Use traits for acquisition, staging, preflight, materialization, invocation, running, log
verification, receipt writing, and the clock. Task 13 supplies only a fake receipt sink and
the orchestration seam; Task 14 implements the immutable receipt writer. State-machine
tests use deterministic fakes; module integration remains in the focused suites.

**Step 2: Write failing happy/failure/resume tests**

Expected stable step order:

```text
lock -> freeze -> preflight -> acquire:<artifact>* -> stage:bg1 -> stage:bg2
-> materialize:<artifact_id>:<target>* -> install:<run_id>*
-> identity:final -> verify:final -> receipt
```

Assert artifacts are acquired once, EE Fixpack materializes/runs on both roots, EET receives
the staged BG1 argument, runs are strictly serialized, and phase events are friendly.
Failure N leaves N unresolved and later steps pending; no skip path exists.

**Step 3: Write crash-window tests**

For an unresolved `StepStarted`, test:

- materialization reconciles its frozen publication manifest, keeps only an exact hashed
  prefix, and resumes without accepting unknown or truncated files;
- unchanged log plus proven rollback -> retry the same run;
- strict successful prefix -> invoke only the remaining suffix;
- exact expected tail plus complete debug evidence -> append success without rerunning;
- removal, extra, reorder, or incomplete/ambiguous debug -> stop and require a fresh copy.

**Step 4: Prove RED**

```powershell
cargo test -p chriz-bg-engine --test orchestrator
```

**Step 5: Implement the transaction order**

Immediately before each mutation: recheck process/TLK safety, append and sync intent,
snapshot and sync the before log, record invocation digest, then spawn. After exit: sync raw
output/debug/after snapshots, reconcile, append verified outcome, and only then advance.
Never adopt a newer recipe on resume. Final identity patches/reads back the unique BG2
`engine_name` after EET_end/tail runs.

**Step 6: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test orchestrator
cargo test --workspace
git add engine
git commit -m "engine: orchestrate resumable EET builds"
```

### Task 14: Produce immutable receipts, diagnostics, and managed-install records

**Files:**

- Create: `engine/src/receipt.rs`
- Create: `engine/src/diagnostics.rs`
- Create: `engine/src/registry.rs`
- Modify: `engine/src/orchestrator.rs`
- Modify: `engine/src/lib.rs`
- Create: `engine/tests/receipt.rs`
- Create: `engine/tests/diagnostics.rs`
- Create: `engine/tests/registry.rs`

**Step 1: Write failing success and failure receipt tests**

A receipt records install/attempt ids and timestamps; app/engine/schema/recipe versions;
both source fingerprints; recipe/selection/plan digests and exact plan; artifact original
and final URLs, versions, lengths, hashes and cache outcomes; WeiDU binary versions/hashes;
per-run target/components/prompts/timing/exit/warnings/log diff/snapshot hashes; final logs;
engine names; managed save root; and launch path.

Receipts under `.chriz/attempts/<id>/` are create-once. A successful
`install-receipt.json` is also create-once and cannot be replaced by a retry.

**Step 2: Write diagnostics/privacy tests**

The ZIP must contain the receipt, ledger, frozen recipe, and sanitized logs on both success
and failure. It must exclude archives, game content, credentials, private-key material,
raw home paths where redaction is possible, and files outside an explicit allowlist.

**Step 3: Write registry tests**

Store one immutable record per install id in app data instead of one overwrite-prone JSON
index. Missing/moved targets become stale UI cards, not data loss. Duplicate ids with
different roots are rejected.

**Step 4: Run all three focused suites to prove RED**

```powershell
cargo test -p chriz-bg-engine --test receipt --test diagnostics --test registry
```

Expected: FAIL because the three writers do not exist.

**Step 5: Implement and green one boundary at a time**

Implement receipt publication, run `--test receipt`; implement the diagnostic allowlist,
run `--test diagnostics`; then implement the create-once registry and run `--test
registry`. Commit each green boundary separately before wiring the real receipt sink into
the Task 13 seam.

**Step 6: Run the broad gates**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

**Step 7: Commit the orchestration integration**

```powershell
git add engine
git commit -m "engine: record installs and diagnostics"
```

### Task 15: Expose the safe engine as a CLI vertical slice

**Files:**

- Rewrite: `engine/src/bin/chriz-bg-install.rs`
- Create: `engine/tests/cli.rs`

**Step 1: Write failing command tests**

Commands:

```text
validate <recipe> [--profile authoring|public-alpha]
plan <recipe> --preset <id> [selection/input overrides]
games discover <recipe>
games inspect <recipe> <bgee-sod|bg2ee> <path>
install <recipe> --preset <id> --bg1 <path> --bg2 <path> --managed-root <path> --cache <path>
resume <managed-root>
report <managed-root>
diagnostics <managed-root> --output <path>
```

Test deterministic human and `--json` output, validation exit codes, path-bearing errors,
modified-game refusal, lock contention, unknown selection/input, and resume using the
frozen recipe rather than the newest one. Do not add `--assume-build` or a skip flag.

**Step 2: Prove RED**

```powershell
cargo test -p chriz-bg-engine --test cli
```

**Step 3: Implement thin command adapters**

Clap parses only; library APIs own behavior. `install` prints events and persistent manual
download/attention instructions. Ctrl+C sends explicit cancellation through the runner
control channel, waits for process-tree termination, and preserves diagnostics.

**Step 4: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git add engine
git commit -m "engine: expose safe install CLI"
```

### Task 16: Capture and re-curate the missing BG1 pre-merge recipe

**Files:**

- Create: `engine/src/bin/chriz-bg-author.rs`
- Create: `engine/tests/recipe_author.rs`
- Create: `engine/tests/bg1_recipe.rs`
- Create: `manifest/reference/bg1-premerge.tsv`
- Create: `manifest/reference/bg1-premerge-approved.toml`
- Create: `docs/curation/components/DLCMERGER.md`
- Create: `docs/curation/components/BG1UB.md`
- Create: `docs/curation/components/BG1NPC.md`
- Modify: `docs/curation/components/README.md`

**Step 1: Write a failing read-only capture test**

Feed a fixture WeiDU.log to `chriz-bg-author capture-log --source-label ...` and assert
stable TSV output plus provenance fields. Comments remain evidence but only active entries
become rows. The command writes only to stdout; it never changes the source game or the
historical `manifest/install-order.tsv`.

**Step 2: Implement and run against the reference log read-only**

```powershell
cargo test -p chriz-bg-engine --test recipe_author
cargo run -p chriz-bg-engine --bin chriz-bg-author -- capture-log `
  --source-label "BG1 premerge reference, captured 2026-09-02" `
  --log "C:\Games\Baldur's Gate Enhanced Edition modded\WeiDU.log"
```

Expected evidence: 28 active entries—DLC Merger 1.7 component 1; EE Fixpack 0/2;
BG1UB 17.1 components `0,11,12,13,14,16,17,18,19,21,22,29,30,32,33,34`; BG1NPC 32
components `0,10,90,111,120,130,240,160,200`. Add the reviewed output with
`apply_patch`; do not redirect into the protected tree.

**Step 3: Re-list current pinned menus**

Acquire DLC Merger 2.1 and current immutable BG1UB/BG1NPC candidates into the ordinary
cache, extract them safely, and use WeiDU 249 menu listing against a synthetic/read-only
fixture. Create neutral catalogs showing old selection, current number/name, dependencies,
and undecided status. The stale historical selection is evidence, not automatic approval.

**Step 4: Obtain Christopher's narrow curation decision**

Ask only about changed/new BG1UB and BG1NPC choices and the resulting default set. Record
the answer in the three catalogs. Do not reopen already settled BG2 curation.

**Step 5: Write the recipe-order RED test**

Assert the approved reference fragment begins:

```text
DLC Merger on BG1
EE Fixpack 0+2 on BG1
approved BG1-only runs
EE Fixpack 0+2 on BG2
EET initialization on BG2 with staged BG1
```

The two EE Fixpack runs must reference one artifact identity. Task 18 consumes this reviewed
reference fragment when it authors the complete production recipe; Task 16 never edits or
depends on Task 18 outputs.

**Step 6: Verify and commit**

```powershell
cargo test -p chriz-bg-engine --test recipe_author --test bg1_recipe
git diff --check
git add engine manifest/reference docs/curation/components
git commit -m "recipe: capture and curate BG1 premerge"
```

### Task 17: Audit every catalog decision into an explicit recipe outcome

**Files:**

- Create: `tools/curation_audit.py`
- Create: `tools/tests/test_curation_audit.py`
- Create: `manifest/curation-map.toml`
- Modify catalogs only to normalize malformed rows; never change Chris's decisions by
  inference.

**Step 1: Write failing parser fixtures**

Using Python stdlib only, parse the component Markdown tables mechanically and report mod,
component id, subgroup, installed marker, and Decision. Reject unknown Decision values,
duplicate component rows, malformed numeric ids, and raw notes inside a table.

```powershell
py -B -m unittest tools.tests.test_curation_audit -v
```

Expected: FAIL before the parser exists.

**Step 2: Implement the audit, not an autonomous curator**

The tool reads table cells and emits a coverage report; prose dependencies and semantic
grouping remain authored data. Freeze the preservation snapshot's expected totals as a
regression oracle: 1,081 rows—568 blank, 120 optional, 264 default, 129 mandatory—then
update that oracle only with a reviewed curation commit.

Run the focused Python suite and commit this parser slice as
`tools: parse component curation tables` before mapping any decisions.

**Step 3: Map every row**

`manifest/curation-map.toml` maps each nonblank row to one feature/component expansion or
an explicit blocked alpha omission. Every blank row maps to excluded. Every declared
mapping names either an authored feature id or an approved omission id; whether production
recipe components resolve from those features is checked only after Task 18 creates them.

Generate only a mechanical skeleton, review it catalog by catalog, run the audit after each
catalog batch, and commit each reviewed batch as `recipe: map <catalog batch> curation`.

**Step 4: Add catalog/map coverage tests**

Extend the Python fixtures to test that all 60 exposed choice subgroups are represented and
the seven optional-only groups declare an explicit `none` default; every mandatory row has
a parent or collection root; no blank row maps to a selectable outcome; and every mapping
target is uniquely defined. These tests do not load a production recipe.

**Step 5: Verify and commit the coverage gate**

```powershell
py -B -m unittest tools.tests.test_curation_audit -v
git diff --check
git add tools manifest/curation-map.toml docs/curation/components
git commit -m "recipe: enforce complete curation coverage"
```

### Task 18: Freeze immutable sources and author the production alpha recipe

**Files:**

- Create: `manifest/collection.toml`
- Create files under: `manifest/artifacts/`
- Create files under: `manifest/mods/`
- Create: `manifest/presets/chris-recommended.toml`
- Create profiles under: `manifest/game-builds/`
- Modify: `engine/src/bin/chriz-bg-author.rs`
- Create: `engine/tests/artifact_manifest.rs`
- Create: `engine/tests/production_recipe.rs`
- Create: `engine/tests/production_recipe_coverage.rs`
- Modify: `docs/pin-list-2.7.md`
- Modify: `manifest/mod-sources.tsv` as provenance only

**Step 1: Add authoring inspection commands with failing tests**

`artifact inspect <url>` downloads into a quarantined authoring cache, reports final URL,
length, SHA-256, archive layout, expected TP2 matches, declared component menu, and VERSION
evidence. It never edits the recipe. `artifact verify <artifact.toml>` uses the production
acquisition path and fails on any drift.

**Step 2: Define the immutable artifact contract**

Every artifact records exact version/ref, immutable official URL or official page, expected
filename/length when stable, SHA-256, archive kind/root rules/limits, declared publish
roots and TP2s, provenance URL, review date, and one acquisition policy:
`fetch-only`, `manual-user-supplied`, `bundle-permitted`, or `blocked`.
Executable tool artifacts also record their expected PE machine architecture; public alpha
validation requires the pinned WeiDU executable to be x64.

Public validation rejects missing/all-zero hashes, moving branch archives, HTTP downgrade,
unreviewed redirects, ambiguous expected TP2 roots, and shared-artifact disagreement.
Evandra remains explicit manual acquisition with an exact hash.

**Step 3: Complete the human source/license gate**

For every selected public artifact, record evidence that the chosen fetch/distribution
route is allowed. A public GitHub fork is not itself permission. Resolve ambiguous
fork/asset cases—especially Spell Revisions and custom portraits—through permission,
upstream merge, official upstream plus a separately permitted patch, manual user supply,
or a blocked omission. Never bundle third-party content merely because it is narrow.

**Step 4: Author the recipe in four reviewable slices**

1. DLC Merger, both EE Fixpack runs, BG1 content, EET/EET_END, and tools.
2. Release-ready official third-party mods.
3. Release-ready Chriz-owned/fork layers.
4. Blocked controls and explicitly accepted alpha omissions with user-facing reasons.

Use the historical TSV files only as provenance. Generate fresh targeted run order from
reviewed dependencies; do not replay the obsolete 90-row tail after historical EET_END.

For DLC Merger 2.1, the alpha pins a verified x64 WeiDU tool and installs component 1. Its
main component applies the 2.7 SoD language-selection fix automatically. Catalog component
10 as conditional, assert it is inapplicable to this x64 route, and add a post-run probe
that fails closed if the expected language fix is absent; do not silently omit the check.
Reference: [DLC Merger's official component documentation](https://github.com/Argent77/A7-DlcMerger#components).

**Step 5: Add deterministic recipe matrix tests**

Load/validate the public profile; resolve the recommended preset; flip each toggle once;
select every choice once; test each numeric/bool input boundary; and snapshot exact run,
target, component, prompt, artifact, and WeiDU-tool identities. Repeated resolution must
produce the same plan digest.

Add the reverse curation gate now that the recipe exists: every production component maps
to one catalog row or approved BG1/collection row; no blank/blocked row resolves; no FAIL
stub or historical local-fix installer appears; and no one-root component is duplicated.

**Step 6: Capture verified Steam profiles**

This is the first human release prerequisite: Christopher repairs/reacquires BGEE+SoD
before this step, without deleting residue as a shortcut or modifying either protected
reference copy. Only after a clean English 2.7.3 source is independently rescanned Fresh,
capture the
minimal authoritative Steam BGEE+SoD and BG2EE fingerprint profiles. GOG profiles remain
experimental and cannot satisfy the verified-release matrix yet.

**Step 7: Verify and commit each slice**

```powershell
cargo run -p chriz-bg-engine --bin chriz-bg-install -- validate manifest --profile public-alpha
cargo run -p chriz-bg-engine --bin chriz-bg-install -- plan manifest --preset chris-recommended
cargo test -p chriz-bg-engine --test artifact_manifest --test production_recipe --test production_recipe_coverage
cargo test --workspace
```

Commit each green slice as `recipe: add <slice> alpha manifest`, then one final
`recipe: freeze v0.1 alpha sources` commit after all hashes/evidence pass.

### Task 19: Make alpha omissions and acceptance evidence release-enforced

**Files:**

- Create: `engine/src/release_validate.rs`
- Modify: `engine/src/lib.rs`
- Create: `engine/tests/release_validate.rs`
- Create: `manifest/releases/v0.1.0-alpha.1/known-limitations.toml`
- Create: `manifest/releases/v0.1.0-alpha.1/acceptance.toml`
- Modify: `docs/curation/components/FOLLOW_UPS.md`

**Step 1: Write failing release-profile tests**

`validate --profile public-alpha` fails when a selected default/mandatory outcome has no
artifact, points at a FAIL stub, lacks required static evidence, appears in an unapproved
tail, or is blocked without an omission record. It also fails when an omitted default has
no Christopher-approved reason and user-facing limitation.

**Step 2: Run the focused suite to prove RED**

```powershell
cargo test -p chriz-bg-engine --test release_validate
```

Expected: FAIL because the public release validator does not exist.

**Step 3: Implement evidence references, not magic booleans**

An acceptance record names the feature/run, evidence kind, repository/commit or test
artifact, date, and status. Static CI may require the record, but only the real test can
create runtime acceptance. `unknown`, `pending`, and `failed` never become accepted through
resolution logic.

**Step 4: Normalize the release candidate**

Every unresolved default is either completed before freeze or listed as an explicit,
visible alpha omission. Experimental selectable content is labeled. Unresolved optional
content stays unavailable with an authored note where useful. Blank content remains hidden.

**Step 5: Verify and commit**

```powershell
cargo test -p chriz-bg-engine --test release_validate --test production_recipe
cargo run -p chriz-bg-engine --bin chriz-bg-install -- validate manifest --profile public-alpha
git add engine manifest/releases docs/curation/components/FOLLOW_UPS.md
git commit -m "recipe: enforce public alpha release gates"
```

### Task 20: Sign immutable recipe releases and classify updates

**Files:**

- Create: `engine/src/updates.rs`
- Create: `engine/src/recipe_envelope.rs`
- Modify: `engine/src/bin/chriz-bg-author.rs`
- Modify: `engine/src/lib.rs`
- Modify: `engine/Cargo.toml`
- Create: `engine/tests/updates.rs`
- Create: `engine/tests/recipe_envelope.rs`
- Create fixtures under: `engine/tests/fixtures/updates/`
- Create: `tools/package-recipe.ps1`
- Create release files under: `manifest/releases/v0.1.0-alpha.1/`

**Step 1: Write failing ledger tests**

Use exhaustive `SaveApplicability` and `Urgency` enums, minimum app version, supersedes,
and unique change ids. Test deferred-only releases, current/conditional guidance, unknown,
independent urgency, skipped-release accumulation, broken chain, duplicate id, and minimum
app prerequisite. These are authored generic claims; no test or API inspects a save.

**Step 2: Write failing signature/trust tests**

Sign the exact emitted envelope bytes; the envelope binds the SHA-256 of the exact packaged
recipe bytes, never reserialized TOML. Reject payload/ledger tamper,
unknown key, malformed signature, replay/downgrade, broken version monotonicity, minimum-app
violation, and key rotation without a statement signed by the currently trusted key.

**Step 3: Run the focused suites to prove RED**

```powershell
cargo test -p chriz-bg-engine --test updates --test recipe_envelope
```

Expected: FAIL because update classification and signature verification do not exist.

**Step 4: Implement the envelope**

```rust
pub struct RecipeEnvelope {
    pub recipe_id: String,
    pub version: String,
    pub payload_sha256: String,
    pub key_id: String,
    pub minimum_app_version: String,
    pub published_at: String,
}
```

Verify a detached Minisign-compatible signature with an embedded public trust root. The
private recipe key exists only in local/CI secret storage and is distinct from Tauri's app
updater key. Store the highest trusted version and exact recipe snapshot; a running or
resumable build keeps its frozen snapshot when a newer release arrives.

Pin `minisign-verify = "0.2.5"` for the application-side verifier. Keep signing in the
authoring/release path so no private-key parsing or signing capability ships in the app.

**Step 5: Add the change-coverage gate**

Diff two recipe packages by feature, artifact, run, input, and game profile. Every semantic
change requires a ledger entry with authored applicability and urgency. Cosmetic text-only
changes are still identified but may use informational/unknown classifications.

**Step 6: Implement and test deterministic packaging**

`tools/package-recipe.ps1` accepts `-RecipeRoot`, `-Version`, `-OutputDirectory`, and a
`-SigningKeyPath` that must be outside the repository. It writes deterministic
`payload.zip`, `envelope.json`, and the detached `envelope.json.minisig`, then immediately
verifies the signature and payload digest through `chriz-bg-author verify-recipe`. Tests
create an ephemeral untrusted key under a temporary directory and inject its public key
only into the authoring verifier; production key contents are never test fixtures or
command-line values.

**Step 7: Verify and commit**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p chriz-bg-engine --test updates --test recipe_envelope
cargo test --workspace
git add engine tools manifest/releases
git commit -m "engine: verify signed recipe updates"
```

### Task 21: Scaffold the smallest real Tauri application

**Files:**

- Modify: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Modify: `.gitignore`
- Create: `app/package.json`
- Create: `app/package-lock.json`
- Create: `app/tsconfig.json`
- Create: `app/vite.config.ts`
- Create: `app/index.html`
- Create: `app/src/main.ts`
- Create: `app/src/app.ts`
- Create: `app/src/backend.ts`
- Create: `app/src/contracts.ts`
- Create: `app/src/state.ts`
- Create: `app/src/styles.css`
- Create: `app/tests/state.test.ts`
- Create: `app/src-tauri/Cargo.toml`
- Create: `app/src-tauri/build.rs`
- Create: `app/src-tauri/tauri.conf.json`
- Create: `app/src-tauri/capabilities/default.json`
- Create: `app/src-tauri/src/main.rs`
- Create: `app/src-tauri/src/lib.rs`

Use Tauri 2's officially recommended Vite + vanilla TypeScript starting point. Do not add
React, Svelte, a router, a state library, Tailwind, a component kit, SQLite, a tray, or a
background service. Commit exact npm and Cargo lockfiles instead of depending on floating
versions. Reference: [Tauri project setup](https://v2.tauri.app/start/create-project/).

**Step 1: Write the failing frontend state test**

```ts
it("starts at Welcome and cannot reach Build without a frozen review", () => {
  const state = initialState();
  expect(state.route).toBe("welcome");
  expect(reduce(state, { type: "continue" }).route).toBe("welcome");
});
```

**Step 2: Scaffold and prove the test is initially RED**

Create npm scripts `typecheck`, `test`, `build`, `tauri`, and `check`. Add only Vite,
TypeScript, Vitest, Testing Library DOM/user-event, axe-core, and Tauri API/CLI packages.

```powershell
Push-Location app
npm ci
npm test -- --run
Pop-Location
```

Expected: the state module/test fails before the minimal reducer exists.

**Step 3: Implement the shell and fake backend boundary**

`Backend` defines the eventual command surface while `FixtureBackend` returns deterministic
fixtures. The app state is one discriminated union; no install truth is written to
`localStorage`. Render a Welcome heading and developer-only backend status.

**Step 4: Configure a locked-down Tauri shell**

Add `app/src-tauri` to the root workspace and depend on `bg_engine` by path. Use a thin
generated `main.rs`; `lib.rs` owns the builder. Name the Cargo package exactly
`chriz-bg-app`, matching later focused test commands. Set `withGlobalTauri: false`, a strict CSP,
current-user install scope, and no shell/filesystem/HTTP plugin permission exposed to
JavaScript. Use system fonts for alpha; do not fetch Atkinson Hyperlegible implicitly.

**Step 5: Verify and commit**

```powershell
Push-Location app
npm run typecheck
npm test -- --run
npm run build
Pop-Location
cargo fmt --all -- --check
cargo test --workspace
git add Cargo.toml Cargo.lock rust-toolchain.toml .gitignore app
git commit -m "app: scaffold Tauri installer shell"
```

### Task 22: Implement all screens against an engine-shaped fixture backend

**Files:**

- Create: `app/src/components/app-shell.ts`
- Create: `app/src/components/status-card.ts`
- Create: `app/src/components/campaign-ledger.ts`
- Create: `app/src/components/technical-log.ts`
- Create: `app/src/screens/home.ts`
- Create: `app/src/screens/updates.ts`
- Create: `app/src/screens/welcome.ts`
- Create: `app/src/screens/games.ts`
- Create: `app/src/screens/destination.ts`
- Create: `app/src/screens/setup.ts`
- Create: `app/src/screens/review.ts`
- Create: `app/src/screens/build.ts`
- Create: `app/src/screens/complete.ts`
- Modify: `app/src/app.ts`
- Modify: `app/src/state.ts`
- Modify: `app/src/styles.css`
- Create: `app/tests/wizard.test.ts`
- Create: `app/tests/backend-contract.test.ts`
- Create: `app/tests/accessibility.test.ts`

**Step 1: Load `frontend-design` and preserve only the useful prototype language**

Port the blackened-iron/vellum/brass/teal/ember tokens, card hierarchy, campaign ledger,
focus style, and responsive intent from `docs/prototypes/installer-v0/index.html`. Do not
port its hardcoded Spell Revisions/SCS logic or fixture counts.

**Step 2: Write failing workflow tests**

Cover all seven wizard screens plus Home/Updates: Back behavior, separate BG1/BG2
dropdowns, all freshness findings, safe destination, default/optional/mandatory/blocked
controls, frozen Review, manual-download and attention states, retry/diagnostics, Complete,
and returning managed installations. Late asynchronous evaluation results must not replace
newer selections.

**Step 3: Write failing security/accessibility tests**

Malicious-looking recipe names, paths, reasons, and console output must render as text, not
HTML. Run axe on every screen. Verify native labels/fieldsets/headings, focus movement,
focusable unavailable explanations, keyboard-only navigation, phase-only polite live
announcements, reduced motion, 44px primary targets, and no workflow horizontal overflow at
320 CSS pixels/200% zoom.

**Step 4: Prove RED**

```powershell
Push-Location app
npm test -- --run
Pop-Location
```

**Step 5: Implement the fixture flow**

Use DOM APIs and `textContent`, never recipe-driven `innerHTML`. The technical console is
focusable/selectable, retains only a bounded in-memory tail, and offers Pause auto-scroll;
the full log remains backend-owned. Unavailable controls use `aria-disabled` plus a
keyboard-discoverable reason rather than disappearing from focus.

Green and commit three bounded route slices in order: Home/Welcome/Games/Destination,
Setup/Review, then Build/Complete/Updates. Run the focused workflow tests after each slice;
finish with the cross-screen accessibility/security suite.

**Step 6: Verify and commit**

```powershell
Push-Location app
npm run typecheck
npm test -- --run
npm run build
Pop-Location
git add app/src app/tests
git commit -m "app: build the guided collection wizard"
```

### Task 23: Connect Tauri commands and live build events to the engine

**Files:**

- Create: `app/src-tauri/src/commands.rs`
- Create: `app/src-tauri/src/bridge.rs`
- Create: `app/src-tauri/src/error.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src/backend.ts`
- Modify: `app/src/contracts.ts`
- Create: `app/src-tauri/tests/command_contract.rs`
- Extend: `app/tests/backend-contract.test.ts`

**Step 1: Write failing serialized command-contract tests**

Expose narrow operations:

```text
bootstrap
discover_games
choose_game_folder
inspect_game_path
choose_destination_folder
evaluate_build
start_build
resume_build
get_run_snapshot
supply_manual_archive
open_manual_source
continue_waiting
cancel_run
export_diagnostics
launch_install
open_install_folder
```

All failures serialize as `CommandError { code, message, recovery_action,
technical_detail }`. Commands for managed installations accept install ids, not arbitrary
paths.

The two picker commands return a user-selected path from a native dialog and immediately
pass it through the matching engine validator. `open_manual_source` accepts an artifact id
and opens only the HTTPS provenance URL from the frozen trusted recipe; it never accepts a
caller-supplied URL.

**Step 2: Add frozen review-token tests**

`evaluate_build` returns a short-lived token bound to source fingerprints, destination,
normalized selection, recipe digest, and plan digest. `start_build` accepts that token,
reloads its server-side snapshot, reacquires the target lock, and repeats preflight. A
changed path/file/selection or expired token is rejected.

**Step 3: Prove RED**

```powershell
cargo test -p chriz-bg-app --test command_contract
Push-Location app
npm test -- --run tests/backend-contract.test.ts
Pop-Location
```

**Step 4: Implement the bridge**

Run synchronous engine work off the Tauri main thread. Stream
`RunEventEnvelope { run_id, sequence_as_string, event }` through a typed Tauri Channel;
the channel is presentation-only and a dropped renderer never stops the build. On reopen,
`get_run_snapshot` replays the authoritative ledger.

Folder browsing, opening the official manual-download page, opening the managed folder,
and launching InfinityLoader occur in Rust commands with validated engine-owned paths.
Do not expose generic shell, filesystem, opener, updater, or HTTP capabilities to frontend
code.

**Step 5: Connect screens in safety order**

Connect discovery/destination/setup/review first. Connect Build only after Tasks 3–15 all
pass. Then connect Complete, launch, diagnostics, and Home cards. Preserve FixtureBackend
for deterministic UI tests.

**Step 6: Verify and commit**

```powershell
Push-Location app
npm run check
npm run tauri build -- --debug --no-bundle
Pop-Location
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git add app Cargo.lock
git commit -m "app: connect installer engine commands"
```

### Task 24: Add managed installations and signed update UX

**Files:**

- Modify: `app/src/screens/home.ts`
- Modify: `app/src/screens/updates.ts`
- Modify: `app/src/state.ts`
- Modify: `app/src/backend.ts`
- Modify: `app/src/contracts.ts`
- Modify: `app/src-tauri/src/commands.rs`
- Create: `app/src-tauri/src/updates.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src-tauri/tauri.conf.json`
- Modify: `app/src-tauri/capabilities/default.json`
- Extend: `app/tests/wizard.test.ts`
- Extend: `app/src-tauri/tests/command_contract.rs`

**Step 1: Write failing three-target update tests**

Distinguish app update, recipe update, and each managed copy's status. Test deferred
next-playthrough messaging, conditional authored guidance without a save-inspection claim,
unknown applicability, app prerequisite, offline last-checked state, invalid signature,
replayed recipe, and stale/moved managed target.

Every recipe update action is `Build updated copy`; no command named `patch`, `update
installation`, or `uninstall mod` exists. Hot-patch eligibility may be displayed as future
metadata, but its executor is absent.

**Step 2: Add active-build exclusion tests**

App update installation and trusted-recipe replacement fail with a clear deferred status
while any managed target build lock is held. A download may be staged, but the running
session retains its exact recipe snapshot.

**Step 3: Run frontend and Rust suites to prove RED**

```powershell
cargo test -p chriz-bg-app --test command_contract
Push-Location app
npm test -- --run tests/wizard.test.ts
Pop-Location
```

Expected: both focused boundaries fail before update commands and states exist.

**Step 4: Implement engine-owned update commands**

Rust fetches and verifies the fixed HTTPS alpha channel, detached recipe signature,
version monotonicity, and Tauri app updater result. JavaScript receives only display DTOs
and calls a narrow `install_app_update` command. Tauri updater signatures are mandatory;
private signing material is never present in the repository or app.

Add and pin `tauri-plugin-updater`, register its builder in Rust, and grant only the updater
permissions required by the narrow Rust-owned command. The frontend receives no generic
updater permission.

**Step 5: Verify and commit**

```powershell
Push-Location app
npm run check
Pop-Location
cargo test --workspace
git add app
git commit -m "app: add signed update center"
```

### Task 25: Package and rehearse the Windows alpha pipeline

**Files:**

- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release-recipe.yml`
- Create: `.github/workflows/release-app.yml`
- Create: `app/wdio.conf.ts`
- Create: `app/tests/e2e/alpha-smoke.test.ts`
- Modify: `app/package.json`
- Modify: `app/package-lock.json`
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/tauri.conf.json`
- Create: `docs/release/v0.1-alpha-rehearsal.md`
- Create: `docs/release/v0.1-alpha-known-limitations.md`
- Create: `docs/release/v0.1-alpha-getting-started.md`

**Step 1: Add CI before release automation**

On Windows, run Rust format/clippy/tests, the gated real-WeiDU fixture with a pinned and
hash-verified official WeiDU 249 binary, npm clean install/typecheck/unit/accessibility
tests/build, recipe public-profile validation, signature fixture verification, and secret
scanning. CI never downloads or embeds third-party mod archives as build artifacts.

**Step 2: Add one packaged WebdriverIO smoke flow**

Against a synthetic game and local artifact server, walk Welcome through Complete, force
one resumable failure, reopen the renderer, verify ledger recovery, export diagnostics,
and confirm the launch command targets the fixture InfinityLoader. Keep exhaustive state
coverage in fast unit tests; this is one boundary test.

Use the current recommended `@wdio/tauri-service` embedded provider, so the test does not
depend on an external `tauri-driver` or matching EdgeDriver. Add the two WebdriverIO Tauri
plugins behind an `e2e` Cargo feature, build a debug test binary with that feature, and
prove the production release binary is built without it. Pin the npm and Cargo versions in
their lockfiles; CI never installs a floating driver.

Reference: [Tauri WebDriver testing](https://v2.tauri.app/develop/tests/webdriver/).

Commit the green packaged smoke slice as `test: exercise packaged alpha flow`.

**Step 3: Configure one Windows x64 NSIS artifact**

Use current-user install mode, default WebView2 bootstrapper, strict CSP,
`createUpdaterArtifacts: true`, and one stable repository-derived application identifier.
Create no MSI matrix for alpha. App uninstall must leave managed game copies and save roots
untouched.

**Step 4: Configure separate signed channels**

Recipe and Tauri updater keys are separate CI secrets. Configure workflows that can
generate immutable prerelease assets and a fixed `alpha/latest.json` HTTPS channel; do not
rely on GitHub's generic “latest release,” which excludes prereleases. Task 25 may exercise
only local/private staging and must not mutate the public channel. Rehearse
`0.1.0-alpha.0 -> 0.1.0-alpha.1` for both app and recipe.

If no Authenticode certificate is available, publish SHA-256 values and document the
expected SmartScreen warning. Tauri updater signing remains mandatory and must not be
misdescribed as Authenticode.

Reference: [Tauri updater signatures](https://v2.tauri.app/plugin/updater/) and
[Windows installers](https://v2.tauri.app/distribute/windows-installer/).

Commit the NSIS and channel configuration as `release: configure signed alpha channels`
before adding the workflows that publish them.

**Step 5: Run the complete local release check**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
$env:CHRIZ_RECIPE_SIGNING_KEY_PATH = '<local alpha recipe signing key path>'
$recipeOutput = Join-Path $env:TEMP 'chriz-bg-recipe-alpha-test'
& .\tools\package-recipe.ps1 -RecipeRoot .\manifest -Version '0.1.0-alpha.1' `
  -OutputDirectory $recipeOutput -SigningKeyPath $env:CHRIZ_RECIPE_SIGNING_KEY_PATH
cargo run -p chriz-bg-engine --bin chriz-bg-author -- verify-recipe $recipeOutput
Push-Location app
npm ci
npm run check
npm run tauri build -- --debug --no-bundle --features e2e
npm run test:e2e
$env:TAURI_SIGNING_PRIVATE_KEY = '<local test key path or content>'
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '<local test password>'
npm run tauri build -- --target x86_64-pc-windows-msvc --bundles nsis
Pop-Location
```

Expected: signed updater artifact plus `.sig`, a verified recipe package/signature, passing
synthetic packaged flow, and no private key in status/diff/log output.

**Step 6: Commit automation, not secrets**

```powershell
git add .github app docs/release
git commit -m "release: package Windows alpha safely"
```

### Task 26: Build Christopher's release candidate and publish the public alpha

**Files/evidence:**

- Update: `manifest/releases/v0.1.0-alpha.1/acceptance.toml`
- Update: `docs/release/v0.1-alpha-rehearsal.md`
- Update: `docs/release/v0.1-alpha-known-limitations.md`
- Create: `docs/release/v0.1-alpha-publication.md`
- Update: `docs/handover.md`
- Create no game files in the repository.

This is an acceptance/release task, not a place to patch code during the run. Any defect
returns to its owning TDD task and produces a new candidate.

**Step 1: Reverify the clean-source gate from Task 18**

Confirm the repaired/reacquired Steam BGEE+SoD 2.7.3 source still rescans Fresh and both
base games launch as needed. Do not use the currently modified BG1 directory, delete
residue as a shortcut, or write to either protected `C:\Games` reference.

**Step 2: Freeze the exact candidate**

Review the normalized recommended plan, every explicit alpha omission, manual Evandra
flow, source/license evidence, app/recipe versions, signatures, hashes, and known
limitations. Run the release-profile validator from a clean checkout and empty cache.

**Step 3: Perform Christopher's real installation with the packaged app**

Use a normal user account and a new writable managed root outside store/protected paths.
The app must discover both paths, show freshness/build/storefront, download every public
artifact, perform any documented manual archive step, execute the entire recommended
recipe, verify both WeiDU logs, write the immutable receipt, and leave both store sources
byte-for-byte unchanged.

**Step 4: Exercise recovery deliberately**

On separate disposable candidates, interrupt download, staging copy, and a synthetic or
safe early WeiDU boundary. Resume each. Also prove hash mismatch, unavailable source,
unexpected prompt, insufficient disk, locked TLK/active game, and target-lock contention
fail closed with diagnostics.

**Step 5: Run gameplay acceptance**

Launch only through InfinityLoader, reach the main menu, start the BG1 campaign, save,
exit, relaunch, and reload. Perform the targeted checks recorded in `FOLLOW_UPS.md`: EET
merge/finalization, EEex 1.2/LuaJIT and loader settings, SCS 35.21 under WeiDU 249, selected
Chriz/rebalance/SoD layers, and any selected migrated tail/portrait checks. Record exactly
what passed and what remains untested; static evidence is not live acceptance.

**Step 6: Rehearse signed updates**

Publish to a private/staging alpha channel first. Confirm app and recipe updates verify,
offline checks remain nonfatal, invalid signatures preserve the trusted version, and a
recipe update offers Build updated copy without touching the working install or saves.

**Step 7: Commit the accepted release candidate before tagging**

```powershell
git add manifest/releases/v0.1.0-alpha.1 docs/release docs/handover.md
git commit -m "release: record v0.1 alpha acceptance"
```

The candidate tag and binaries must derive from this exact reviewed commit.

**Step 8: Publish only after explicit final authorization**

Verify `gh auth status` is `Chrizhermann`, push reviewed commits, create the normal GitHub
prerelease `v0.1.0-alpha.1`, upload installer/signature/recipe/signature/hash/known
limitations/getting-started assets, update the fixed alpha channel, and download-verify the
published assets from an empty directory. The public page must say Steam 2.7.3 English is
verified, GOG is experimental, the collection is alpha, and third-party mods are fetched
from their own sources.

**Step 9: Record and push post-publication verification**

```powershell
git add docs/release/v0.1-alpha-publication.md docs/handover.md
git commit -m "docs: verify published v0.1 alpha artifacts"
```

Record downloaded hashes, updater checks, release/tag/commit identity, and whether the
published installer reproduced the cold-cache flow. Push this evidence commit only under
the same explicit release authorization.

Do not call the milestone complete until the published installer itself reproduces the
cold-cache flow and its receipt/log/launch evidence has been inspected.

## Final verification matrix

Before the release decision, all rows must be green or explicitly nonblocking and visible
in known limitations:

| Layer | Required evidence |
|---|---|
| Schema/selection | Schema-v2 parser/validator/resolver, catalog coverage, deterministic plan digest |
| WeiDU | Production invocation, real-WeiDU fixture, ordered log reconciliation, prompt supervision |
| Filesystem | Source freshness, regular-copy isolation, reparse rejection, unique save root, target lock |
| Acquisition | Cold cache, hashes, strong resume validator, bounded extraction, manual archive path |
| Recovery | Append-only ledger and download/copy/WeiDU crash-window rehearsals |
| Recipe | Immutable pins, license/provenance records, signed envelope, classified change ledger |
| UI | Keyboard/axe/zoom tests, persistent recovery states, no hardcoded compatibility logic |
| Packaging | Normal-user NSIS install, updater signature, recipe signature, uninstall preserves games |
| Runtime | InfinityLoader boot, BG1 start, save/reload, targeted EET/EEex/SCS/Chriz checks |
| Publication | Empty-cache download of published assets and honest alpha limitations |

## Superseded plan assumptions

This plan supersedes the earlier Recipe Preview stopping point and these unsafe Phase-1
details:

- `--quick-log` is forbidden on real installs.
- WeiDU silence requests attention; it never causes an automatic kill.
- Prompt answers are matched to expected output, not eagerly concatenated into stdin.
- Setup-name emulation is the default; explicit TP2 invocation requires compatibility
  evidence because `%MOD_FOLDER%` can otherwise become `weidu_external`.
- Session truth is an append-only hash-chained ledger, not a repeatedly replaced JSON file.
- Phase/run identity is per targeted run, so one artifact can safely serve both game roots.
- Production hashes are mandatory, including manual sources; authoring placeholders cannot
  pass a public profile.
- Resume reconciles actual WeiDU evidence before rerunning anything.

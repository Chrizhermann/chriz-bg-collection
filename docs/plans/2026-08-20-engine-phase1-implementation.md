# Engine Phase 1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task-by-task. Also load the `bg-modding` skill (WeiDU facts) before Tasks 7-11.

**Goal:** The headless Rust install engine + CLI (`chriz-bg-install`) that can take a
manifest, acquire mods, copy game dirs, drive WeiDU in manifest order, verify per
component, and resume after failure — proven against a synthetic fake game with real
WeiDU, without needing the curated manifest content (Chris is curating in parallel).

**Architecture:** One Rust crate `engine/` (lib `bg_engine` + bin `chriz-bg-install`),
zero Tauri deps. Synchronous core: `ureq` downloads, `std::process` WeiDU runner with
reader threads, `crossbeam-channel` events behind an `EventSink` trait (the Tauri app
later subscribes; the CLI prints). Persisted session state in the target dir makes every
stage resumable. Append-only WeiDU invariant throughout — no uninstall paths in Phase 1.

**Tech Stack:** Rust stable, serde/toml/serde_json, thiserror, clap (derive), ureq
(rustls), sha2+hex, zip, walkdir, crossbeam-channel, tempfile+tiny_http (dev), pelite
(Windows exe version). TDD with `cargo test`; real-WeiDU integration tests are `#[ignore]`
and gated on env `CHRIZ_WEIDU_EXE` (path to a WeiDU ≥24900 binary).

**Ground rules (from design doc + verified WeiDU facts):**
- Install strictly in manifest order; never trigger the WeiDU stack cascade.
- Exit code 0 proves nothing — WeiDU.log snapshot diff is the per-component truth.
- Invocation template: `<weidu> <tp2> --language N --use-lang en_US
  --force-install-list <ids> --no-exit-pause --skip-at-view --safe-exit --quick-log
  --log <logs>/<mod>.debug`, cwd = game root, tp2 path relative.
- `C:\Games\...modded\` and the mod archive are READ-ONLY. Tests never touch them.
- Commits go to `main` (repo practice), one commit per green step, prefix `engine:`.

---

## Task 0: Cargo workspace + crate scaffold

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `engine/Cargo.toml`, `engine/src/lib.rs`, `engine/src/bin/chriz-bg-install.rs`
- Modify: `.gitignore` (add `/target`)

**Step 1: Write the workspace + crate**

`Cargo.toml` (repo root):
```toml
[workspace]
resolver = "2"
members = ["engine"]
```

`engine/Cargo.toml`:
```toml
[package]
name = "chriz-bg-engine"
version = "0.1.0"
edition = "2021"

[lib]
name = "bg_engine"
path = "src/lib.rs"

[[bin]]
name = "chriz-bg-install"
path = "src/bin/chriz-bg-install.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "2"
clap = { version = "4", features = ["derive"] }
ureq = { version = "2", features = ["tls"] }
sha2 = "0.10"
hex = "0.4"
zip = "2"
walkdir = "2"
crossbeam-channel = "0.5"

[target.'cfg(windows)'.dependencies]
pelite = "0.10"

[dev-dependencies]
tempfile = "3"
tiny_http = "0.12"
```

`engine/src/lib.rs`:
```rust
pub mod manifest;
pub mod error;
```
(create `engine/src/manifest.rs` and `engine/src/error.rs` as empty `//! stub` files;
they gain content in Task 1)

`engine/src/bin/chriz-bg-install.rs`:
```rust
fn main() {
    println!("chriz-bg-install {}", env!("CARGO_PKG_VERSION"));
}
```

**Step 2: Verify it builds and the bin runs**

Run: `cargo build && cargo run -p chriz-bg-engine --bin chriz-bg-install`
Expected: compiles; prints `chriz-bg-install 0.1.0`

**Step 3: Baseline hygiene**

Run: `cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Expected: all pass (no tests yet is OK)

**Step 4: Commit**

```bash
git add Cargo.toml engine .gitignore
git commit -m "engine: cargo workspace + crate scaffold"
```

---

## Task 1: Manifest schema types + TOML parsing

**Files:**
- Create: `engine/src/manifest.rs` (replace stub)
- Create: `engine/src/error.rs` (replace stub)
- Create: `engine/tests/fixtures/manifest/collection.toml`
- Create: `engine/tests/fixtures/manifest/mods/testmod.toml`, `.../eet.toml`
- Test: `engine/tests/manifest_parse.rs`

**Step 1: Write the failing test**

`engine/tests/manifest_parse.rs`:
```rust
use bg_engine::manifest::{ModFile, SourceKind};

#[test]
fn parses_mod_file() {
    let text = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/manifest/mods/testmod.toml"),
    )
    .unwrap();
    let m: ModFile = toml::from_str(&text).unwrap();
    assert_eq!(m.id, "testmod");
    assert_eq!(m.weidu, "249.00");
    assert_eq!(m.phase, bg_engine::manifest::Phase::Main);
    assert_eq!(m.source.kind, SourceKind::GithubRelease);
    assert_eq!(m.components.len(), 2);
    assert_eq!(m.components[1].stdin.as_deref(), Some("1\n"));
    assert_eq!(m.platforms, vec!["windows", "macos", "linux"]);
}
```

Fixture `engine/tests/fixtures/manifest/mods/testmod.toml`:
```toml
id = "testmod"
name = "Test Mod"
version = "1.0"
tp2 = "testmod/setup-testmod.tp2"
language = 0
weidu = "249.00"
platforms = ["windows", "macos", "linux"]
phase = "main"

[source]
kind = "github-release"
url = "https://example.invalid/testmod-1.0.zip"
sha256 = "0000000000000000000000000000000000000000000000000000000000000000"

[[components]]
id = 0
name = "Core"

[[components]]
id = 10
name = "Optional thing"
stdin = "1\n"
```
(`eet.toml`: same shape, `phase = "bg1-pre-merge"` is NOT valid for EET — use
`phase = "main"`; give it `kind = "github-commit-zip"` to cover the second enum value.)

**Step 2: Run test to verify it fails**

Run: `cargo test -p chriz-bg-engine --test manifest_parse`
Expected: FAIL — `manifest` module has no types

**Step 3: Implement the types**

`engine/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error in {path}: {msg}")]
    ManifestParse { path: String, msg: String },
    #[error("manifest validation: {0}")]
    Validation(String),
}
pub type Result<T> = std::result::Result<T, EngineError>;
```

`engine/src/manifest.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Bg1PreMerge,
    Main,
    PostEetEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    GithubRelease,
    GithubTagArchive,
    GithubCommitZip,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub kind: SourceKind,
    pub url: String,
    pub sha256: String,
    /// For kind = manual: page the GUI/CLI opens for the user.
    #[serde(default)]
    pub manual_page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: u32,
    #[serde(default)]
    pub name: String,
    /// Scripted stdin for READLN prompts, verbatim (include trailing newline).
    #[serde(default)]
    pub stdin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub id: String,
    pub name: String,
    pub version: String,
    pub tp2: String,
    #[serde(default)]
    pub language: u32,
    pub weidu: String,
    pub platforms: Vec<String>,
    pub phase: Phase,
    pub source: Source,
    #[serde(default)]
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEntry {
    /// mod id; granularity finer than a whole mod uses `components`
    pub id: String,
    /// None = all of the mod's components at this position
    #[serde(default)]
    pub components: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toggle {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub default_on: bool,
    /// mod ids fully removed when off
    #[serde(default)]
    pub removes_mods: Vec<String>,
    /// (mod id, component ids) additionally removed when off — knock-on exclusions
    #[serde(default)]
    pub removes_components: Vec<ComponentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRef {
    pub mod_id: String,
    pub component: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceGroup {
    pub id: String,
    pub name: String,
    pub default: String,
    pub options: Vec<ChoiceOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub adds_components: Vec<ComponentRef>,
    #[serde(default)]
    pub removes_components: Vec<ComponentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub game_build: String,
    #[serde(default)]
    pub order: Vec<OrderEntry>,
    #[serde(default)]
    pub toggles: Vec<Toggle>,
    #[serde(default)]
    pub choice_groups: Vec<ChoiceGroup>,
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p chriz-bg-engine --test manifest_parse`
Expected: PASS

**Step 5: Commit**

```bash
git add engine
git commit -m "engine: manifest schema v1 types + parse test"
```

---

## Task 2: Manifest loader (collection.toml + mods/*.toml directory)

**Files:**
- Create: `engine/src/loader.rs`; register `pub mod loader;` in `lib.rs`
- Create: `engine/tests/fixtures/manifest/collection.toml` (if not yet)
- Test: `engine/tests/manifest_load.rs`

**Step 1: Failing test** — `Manifest::load(dir)` returns a `Manifest { collection,
mods: BTreeMap<String, ModFile> }`; asserts both fixture mods load, keyed by id, and a
missing dir errors cleanly. Fixture `collection.toml`:
```toml
game_build = "2.7.3.0"

[[order]]
id = "eet"

[[order]]
id = "testmod"

[[toggles]]
id = "testmod-optional"
name = "Test optional bits"
default_on = true
removes_components = [{ mod_id = "testmod", component = 10 }]
```

**Step 2:** Run: `cargo test -p chriz-bg-engine --test manifest_load` → FAIL

**Step 3: Implement** `loader.rs`: read `collection.toml`, walk `mods/*.toml`
(walkdir, sorted), parse each with the file path in `ManifestParse` errors,
reject duplicate ids.

**Step 4:** Run again → PASS

**Step 5: Commit** `engine: manifest loader`

---

## Task 3: Manifest validators

**Files:**
- Create: `engine/src/validate.rs`; register in `lib.rs`
- Test: `engine/tests/manifest_validate.rs`

Validation rules (each = one table-driven test case with a broken fixture built in-memory
by mutating the parsed good fixture — no extra fixture files):
1. Every `order[].id` references an existing mod; every mod appears in `order` exactly once.
2. `order[].components` (when present) and all `ComponentRef`s reference declared components.
3. Component ids unique per mod.
4. Toggle/choice ids unique; choice `default` is one of its options.
5. `platforms` values ⊆ {windows, macos, linux}; non-manual sources have `https://` url +
   64-hex sha256 (all-zero sha256 allowed only behind `--allow-unpinned`, a later
   CLI escape hatch for pre-hash authoring; validator returns Warning not Error for it).
6. Mods with phase `bg1-pre-merge` must precede `eet` in order; nothing may follow
   `eet_end` except mods with phase `post-eet-end` (encode as: order must be grouped
   bg1-pre-merge < main < post-eet-end, with `eet`/`eet_end` as main-phase anchors).

**API:** `pub fn validate(m: &Manifest) -> Vec<Finding>` where
`Finding { severity: Error|Warning, rule: &'static str, message: String }`.
Steps: failing tests (one per rule, good fixture passes with zero errors) → run → implement
→ run → commit `engine: manifest validators`.

---

## Task 4: Resolve stage — preset + toggles → InstallPlan

**Files:**
- Create: `engine/src/resolve.rs`; register in `lib.rs`
- Test: `engine/tests/resolve.rs`

**Step 1: Failing tests**
```rust
use bg_engine::resolve::{resolve, Selection};

#[test]
fn default_selection_includes_everything_on_by_default() { /* toggle on → comp 10 present */ }

#[test]
fn toggle_off_removes_mod_and_knock_ons() { /* removes_mods + removes_components applied */ }

#[test]
fn choice_group_swaps_components() { /* non-default option adds/removes */ }

#[test]
fn plan_preserves_manifest_order_and_groups_by_mod_run() {
    // order entries with explicit component splits become separate runs of the same mod
}

#[test]
fn platform_filter_drops_incompatible_mods() { /* windows-only mod dropped on linux target */ }
```

**Step 3: Implement** `resolve.rs`:
```rust
pub struct Selection {
    pub toggles_off: Vec<String>,
    pub choices: std::collections::BTreeMap<String, String>, // group id -> option id
    pub platform: &'static str, // "windows" for Phase 1
}

pub struct PlannedRun {
    pub mod_id: String,
    pub phase: Phase,
    pub components: Vec<Component>, // resolved, in declared order
}

pub struct InstallPlan { pub runs: Vec<PlannedRun> }

pub fn resolve(m: &Manifest, sel: &Selection) -> Result<InstallPlan>
```
Semantics: start from `order`; apply platform filter; apply toggles (off → remove); apply
choice groups (default unless overridden); drop runs that end up with zero components;
error if a removed mod is still referenced by a remaining `ComponentRef`.

**Step 5: Commit** `engine: resolve stage (selection -> install plan)`

---

## Task 5: Event model

**Files:**
- Create: `engine/src/events.rs`; register in `lib.rs`
- Test: unit tests inside `events.rs`

`EngineEvent` (serde-serializable, for the future Tauri bridge):
`PhaseStarted{name}`, `StepStarted{id, label}`, `StepProgress{id, done, total}`,
`ConsoleLine{step_id, stream: Stdout|Stderr, line}`, `StepFinished{id, outcome}`,
`ManualDownloadNeeded{mod_id, page, expected_sha256, drop_dir}`, `Error{step_id, message}`.

`pub trait EventSink: Send { fn emit(&self, e: EngineEvent); }` + two impls:
`ConsoleSink` (human lines to stderr) and `ChannelSink(crossbeam_channel::Sender<EngineEvent>)`.
Test: ChannelSink delivers; ConsoleSink doesn't panic. Commit `engine: event model`.

---

## Task 6: Session persistence + resume

**Files:**
- Create: `engine/src/session.rs`; register in `lib.rs`
- Test: `engine/tests/session.rs`

Session = `session.json` in the target install dir:
```rust
pub struct Session {
    pub manifest_fingerprint: String, // sha256 over canonicalized manifest content
    pub selection: Selection,
    pub steps: Vec<StepRecord>,       // one per pipeline step, in order
}
pub struct StepRecord { pub id: String, pub status: StepStatus, pub detail: Option<String> }
pub enum StepStatus { Pending, Running, Done, Failed }
```
Tests: round-trip save/load; `next_pending()` skips Done; a `Running` step on load is
downgraded to `Failed` (crash detection); loading with a different manifest fingerprint
errors (`resume with changed manifest is forbidden — rebuild instead`); atomic write
(write tmp + rename). Commit `engine: session state + resume semantics`.

---

## Task 7: WeiDU invocation builder (pure)

**Files:**
- Create: `engine/src/weidu/mod.rs`, `engine/src/weidu/invocation.rs`
- Test: unit tests in `invocation.rs`

**Step 1: Failing tests**
```rust
#[test]
fn builds_standard_install_invocation() {
    let inv = Invocation::install(&run, &paths); // run: PlannedRun, paths: WeiduPaths
    assert_eq!(inv.program, paths.weidu_for("249.00"));
    assert_eq!(inv.cwd, paths.game_root);
    assert_eq!(
        inv.args,
        [
            "testmod/setup-testmod.tp2",
            "--language", "0",
            "--use-lang", "en_US",
            "--force-install-list", "0", "10",
            "--no-exit-pause",
            "--skip-at-view",
            "--safe-exit",
            "--quick-log",
            "--noautoupdate",
            "--log", "../logs/testmod.debug",
        ]
    );
}

#[test]
fn concatenates_component_stdin_answers_in_order() {
    // components [#0 (no stdin), #10 (stdin "1\n")] -> feed exactly "1\n"
}

#[test]
fn tp2_path_stays_relative_and_forward_slashed() { /* cwd-relative for log fidelity */ }
```

**Step 3: Implement.** Facts encoded (verified, bg-modding skill):
- `--use-lang en_US` always (omitting it blocks on stdin).
- `--no-exit-pause` always (failures pause by default otherwise).
- `--safe-exit` + `--quick-log` on pure installs (Phase 1 is install-only).
- `--noautoupdate` always (we pin binaries; setup-exe soup must not upgrade anything).
- `--log` target OUTSIDE the game dir (logs dir sibling), forward slashes.
- stdin = concatenation of selected components' `stdin` fields in component order; `None`
  when no component has one (do not open a pipe that feeds nothing — see Task 9).

**Step 5: Commit** `engine: weidu invocation builder`

---

## Task 8: WeiDU.log parsing + per-run verification (pure)

**Files:**
- Create: `engine/src/weidu/log.rs`, `engine/src/weidu/verify.rs`
- Create: `engine/tests/fixtures/weidu/WeiDU-before.log`, `WeiDU-after.log`,
  `setup-testmod.debug` (assembled from real reference-install lines — copy a handful of
  lines from `manifest/install-order.tsv` provenance; do NOT read the game dir at test
  time, bake the fixture in)
- Test: `engine/tests/weidu_verify.rs`

**Step 1: Failing tests**
- `parses_entries_and_skips_comment_lines` — `// Recently Uninstalled` lines ignored.
- `diff_detects_added_components` — before/after diff yields exactly the expected
  `(tp2_normalized, lang, comp)` set; tp2 normalization: uppercase, strip leading path,
  strip `SETUP-` prefix, strip `.TP2` — matching WeiDU's own MOD_IS_INSTALLED semantics.
- `diff_detects_missing_component_as_failure` — expected comp absent → `Verdict::Missing`.
- `diff_detects_unexpected_removal` — entry vanished → `Verdict::StackDisturbed` (hard abort).
- `debug_grep_classifies_statuses` — finds `SUCCESSFULLY INSTALLED`, `INSTALLED WITH
  WARNINGS`, `NOT INSTALLED DUE TO ERRORS`, `SKIPPING`, ignoring lines about OTHER mods
  (the METADATA parse-noise landmine).
- `exit_code_zero_with_missing_component_is_still_a_failure` — belt-and-suspenders rule.

**Step 3: Implement** with the entry regex
`^~(?P<tp2>[^~]+)~\s+#(?P<lang>\d+)\s+#(?P<comp>\d+)(?:\s+//.*)?$` (hand-rolled parser is
fine; avoid the regex crate dependency if a split-based parser stays readable).
`pub fn verify_run(before: &str, after: &str, debug: &str, run: &PlannedRun) -> RunVerdict`.

**Step 5: Commit** `engine: weidu log diff verification`

---

## Task 9: Process runner with streaming, scripted stdin, watchdog

**Files:**
- Create: `engine/src/weidu/runner.rs`
- Create: `engine/src/bin/mock-child.rs` (test helper bin: prints lines/sleeps/reads stdin
  per its args — cross-platform fake WeiDU)
- Test: `engine/tests/runner.rs` (drives `env!("CARGO_BIN_EXE_mock-child")`)

**Step 1: Failing tests**
- `streams_stdout_lines_as_events_in_order`
- `feeds_scripted_stdin_then_closes_pipe` — mock-child echoes what it read; runner must
  write all bytes, then close stdin (WeiDU READLN gets EOF → deterministic failure instead
  of hang if answers run out).
- `captures_exit_code`
- `watchdog_fires_on_output_stall` — mock-child sleeps 5s silently; runner with
  `stall_timeout=1s` emits `StallDetected` event and (config) kills → outcome
  `Outcome::Stalled`. Timer resets on every output line (long CDTweaks/SCS component
  installs print continuously; silence is what indicates a prompt).
- `no_stdin_pipe_when_no_answers` — child sees closed stdin immediately.

**Step 3: Implement** `run(inv: &Invocation, sink: &dyn EventSink, cfg: &RunnerCfg) ->
RunResult`: spawn with piped stdio, two reader threads sending lines over a channel,
stdin writer thread, select loop with `recv_timeout` for the watchdog.

**Step 5: Commit** `engine: process runner (streaming, stdin script, stall watchdog)`

---

## Task 10: Fake-game builder (test support)

**Files:**
- Create: `engine/tests/support/mod.rs`, `engine/tests/support/fakegame.rs`
- Test: `engine/tests/fakegame.rs` (self-test of the builder)

Port of the verified Python fixture writer (bg-modding `weidu-testing.md`, layouts
verified 2026-07-16 against WeiDU 24900):
- `chitin.key` KEY V1 (header 0x18: sig/ver, BIF count, res count, BIF-table off,
  res-table off; BIF entry 12+name; res entry 14: resref8, type u16, locator u32).
- One uncompressed BIFF V1 (header 0x14; var entry 0x10) holding stub resources.
- **Include `OH6000.ARE` (type 1010) so `GAME_IS ~bg2ee~` is true.**
- Minimal `dialog.tlk` at root AND `lang/en_US/dialog.tlk`: `TLK V1  ` sig, 1 empty
  string, string-offset 0x2C, one 26-byte entry.
- Empty `override/`, plus stub `SPELL.IDS`/`STATS.IDS` in the BIF (sibling-predicate
  IDS-load noise prevention).
- `engine.lua` with `engine_name = 'FakeGame'` (Task 12's patch target).

Builder self-test: structural asserts (offsets, counts, round-trip read of the res table).
Commit `engine: synthetic fake-game test builder`.

---

## Task 11: Real-WeiDU integration test (gated)

**Files:**
- Create: `engine/tests/fixtures/testmod/setup-testmod.tp2` + `testmod/` payload
  (two trivial components: #0 `COPY_EXISTING` a BIF resource to override; #10 an
  `ACTION_READLN`-free component with a SUBCOMPONENT pair to prove numeric selection)
- Test: `engine/tests/weidu_integration.rs`, all `#[ignore]`, skip unless
  env `CHRIZ_WEIDU_EXE` is set

**Flow per test:** build fake game in a tempdir → copy fixture mod in → build Invocation
→ runner → verify_run. Asserts: exit 0, WeiDU.log gained exactly (testmod, 0, {0, 10}),
debug contains 2× `SUCCESSFULLY INSTALLED`, override contains the copied file,
KEY/BIF/TLK bytes unchanged (hash before/after).
Second test: force a failure (component with `FAIL` guarded behind a variable) → verdict
`Missing`, tree untouched.

Run: `set CHRIZ_WEIDU_EXE=<path> && cargo test -p chriz-bg-engine --test weidu_integration -- --ignored`
(document: grab WeiDU 249 from WeiDUorg releases into `tools/weidu/` — gitignored).
Commit `engine: real-weidu integration harness (gated)`.

---

## Task 12: Stage — game detection, copy, engine.lua patch

**Files:**
- Create: `engine/src/stage.rs`; register in `lib.rs`
- Test: `engine/tests/stage.rs` (tempdir fixtures); one `#[ignore]`d read-only probe test
  against the real reference install (existence + version read ONLY — never writes)

Pieces (each its own TDD cycle):
1. `detect_game(dir) -> GameInfo` — requires `chitin.key`; reads build from the game exe
   VERSIONINFO ProductVersion via `pelite` (Windows), `Err(Unsupported)` elsewhere for now.
   **NEEDS-VERIFICATION note:** confirm ProductVersion == `2.7.3.0` form on the real
   Steam/GOG exes via the ignored probe test before trusting the equality check; if the
   resource is absent/odd, fall back to a `--assume-build` CLI flag.
2. `copy_game(src, dst, sink)` — walkdir copy with `StepProgress` events (count files
   first), skip nothing, preserve case, refuse dst inside src and src inside dst.
3. `patch_engine_name(dst, name)` — rewrite/insert `engine_name = '<name>'` in
   `engine.lua` (create the file if the game shipped without one); test round-trip.
4. `preflight(plan, games, target) -> Vec<Finding>` — disk space (2× src sizes + 2 GiB
   slack), target-not-inside-game-dir, both games present for EET.

Commit per piece, final `engine: stage (detect/copy/patch/preflight)`.

---

## Task 13: Acquire — cache, downloads, extraction, manual queue

**Files:**
- Create: `engine/src/acquire.rs`; register in `lib.rs`
- Test: `engine/tests/acquire.rs` (tiny_http local server; zip assembled with the zip crate)

Pieces:
1. Cache layout `cache/<mod_id>/<version>/<filename>` + `cache.json` index keyed by
   sha256; `is_cached()` hit short-circuits.
2. `download(url, dest, sink)` via ureq: stream to `dest.part`, progress events, resume
   with `Range` when `.part` exists, 3 retries with backoff, real User-Agent
   `chriz-bg-collection/<ver>`; then sha256 verify (mismatch → delete + error), rename.
3. `extract(archive, staging_dir)` — zip + iemod (iemod IS a zip); find the mod folder
   containing the tp2 at any depth ≤2 (GitHub tag/commit zips wrap in `Repo-<sha>/`);
   reject archives without the expected tp2.
4. Manual queue: for `SourceKind::Manual` emit `ManualDownloadNeeded` and poll `drop_dir`
   until a file's sha256 matches (test: drop the file from the test after 2 polls).
   All-zero sha256 in manifest + manual → accept any file but record its hash in the
   session (authoring mode).

Tests: happy path, resume-after-truncated-part, sha mismatch, wrapped zip, manual drop.
Commit `engine: acquire stage (cache/download/extract/manual queue)`.

---

## Task 14: Install orchestrator

**Files:**
- Create: `engine/src/orchestrator.rs`; register in `lib.rs`
- Test: `engine/tests/orchestrator.rs` (mock-child as fake weidu via `WeiduPaths` override;
  fixture manifest; tempdir target)

Pipeline steps generated from an `InstallPlan`, recorded in the Session in order:
`preflight` → `acquire:<mod>`* → `stage:copy-bg1` → `stage:copy-bg2` → `stage:patch` →
`install:<run>`* (bg1-pre-merge runs against the BG1 copy, then main/post against BG2
copy — EET merge is just the `eet` run in order) → `verify:final` → `report`.

Behaviors under test:
- happy path drives all steps Done, WeiDU.log snapshot taken before/after each install run
  (copy the log file next to the session per run: `logs/<n>-<mod>.WeiDU.log`).
- failure in run N → session Failed at N, later steps Pending, process exits nonzero.
- resume picks up at N (acquire/copy steps Done are skipped), reruns N.
- `StackDisturbed` verdict → abort with a distinct error (never continue).

Commit `engine: install orchestrator with resume`.

---

## Task 15: CLI

**Files:**
- Modify: `engine/src/bin/chriz-bg-install.rs`
- Test: `engine/tests/cli.rs` (assert_cmd-style via `std::process::Command` on
  `CARGO_BIN_EXE_chriz-bg-install`; no extra dep needed)

Subcommands (clap derive):
- `validate <manifest-dir>` — load + validate, print findings, exit 1 on Error findings.
- `plan <manifest-dir> [--toggle-off id]* [--choice group=option]* [--platform windows]`
  — print the resolved run list (mod, phase, component ids).
- `install <manifest-dir> --bg1 <dir> --bg2 <dir> --target <dir> [--cache <dir>]
  [selection flags] [--weidu-dir tools/weidu] [--assume-build X]` — full pipeline.
- `resume <target-dir>` — reload session, continue.
- `report <target-dir>` — print the JSON report path/summary.

Tests: `validate` on good fixture exits 0; on broken fixture exits 1 with rule name in
stderr; `plan` prints deterministic order. Commit `engine: CLI`.

---

## Task 16: Report + diagnostics bundle

**Files:**
- Create: `engine/src/report.rs`; register in `lib.rs`
- Test: `engine/tests/report.rs`

`install-report.json` in target: engine version, manifest fingerprint, selection, per-step
status+duration, per-run verdicts, tool versions (weidu `--version` output captured at run
time). `diagnostics-bundle.zip`: report + session.json + logs dir. Tests: report written
on both success and failure paths; bundle round-trips (unzip lists expected entries).
Commit `engine: report + diagnostics bundle`.

---

## Milestone check (manual, after Task 16)

With a 2–3-mod real manifest snippet (e.g. testmod fixture + a small real mod zip placed
in cache by hand): `chriz-bg-install install` against a fake game — full pipeline green,
resumable mid-way (kill the process during a run, `resume` completes). The full
451-component EET validation waits for the curated manifest (Phase 0.4) + a real BG1+BG2
copy — that is the design doc's Phase 1 milestone gate, run when curation lands.

## Deliberately NOT in Phase 1 (YAGNI)

Uninstall/mid-stack repair (append-only only), GitHub API update checks, PI metadata
export, macOS/Linux game detection, SFX-exe extraction (manifest pins .zip/.iemod assets
instead), Tauri bridge (Phase 2 wraps `orchestrator` + `ChannelSink`).

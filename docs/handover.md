# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Current priority — curation-derived recipe, not historical replay (2026-09-05)

The user has now requested the complete corrected installation/test/cleanup flow. Continue
from [the current acceptance run](curated-full-acceptance-2026-09-05.md), not the invalid
overnight replay. Recipe reconstruction and source/order verification run in parallel.

Start with [the approved reconciliation path](plans/2026-09-05-curation-reconciliation.md).
The overnight `creator-full-current` recipe bypassed recorded curation; it is invalid as
the curated collection. **Do not repair Bristlelick or resume that selection.** Curation
files are intact. Restore them as the authority, preserve deferred mod work, check changed
ordering against mod documentation/source. The corrected alpha.6 recipe validates, but
replacement r3 is now stopped at SoD Remix 120 after SCS/Randomiser/EET_END passed.
Its source assumes nonexistent BDSCRY state 4; see the [owning-repo handoff](handoffs/2026-09-05-sod-remix-component-120.md).
The alpha.4 native run stopped on component order; r2 stopped on a missing scripted
Randomiser compatibility answer. All three native-order errors and the conditional `y`
answer are fixed without changing component selections (43 runs / 430 components). The
user explicitly retained the full SoD bundle and authorized the source fix. The owning
SoD task has received the handoff and is active; Christopher can do focused live testing.
The acceptance follow-up remains paused awaiting that fix/test candidate, with component
290 still separate and deferred. No fourth blind rebuild was started. Existing alpha.3
packages still contain the old profile; source guards
do not retroactively fix those binaries. Installer acceptance remains the main priority.

## Historical overnight continuation — CEBG app alpha.3 / recipe alpha.2 (2026-09-05)

Start with [`overnight-2026-09-05.md`](overnight-2026-09-05.md): current code changes,
real test paths, cleanup ownership, release boundary and remaining acceptance. The active
worktree is `installer-v0-real-alpha`, **not** `4d39`. Both Steam source games are now
verified clean. The dotted-artifact freeze/evidence failures are fixed. A compact UI,
full-creator recipe profile, Radar add-on, human receipt versions, lazy mod-list consistency
and signed-app updater are implemented; real full-install acceptance is still in progress.
The first overnight follow-up fixed EET's Windows staged-path argument; EET completed.
The second follow-up stopped on malformed Bristlelick source; the later curation audit
supersedes the proposed repair because Bristlelick was excluded. See the current priority
above. Signed app alpha.3 is built locally but its historical full profile must not be used.
No public release/channel has been published. The old status below is historical.

## Earlier Chriz Easy BG 0.1 alpha status (2026-09-05)

Branch `codex/installer-v0-real-alpha` is pushed. The installer engine and UI are
functional, and the public-alpha recipe validates and resolves **35 runs**. All **30
selected artifacts** have passed cold-cache acquisition, extraction, and payload
verification. Notable ready pins include public CHRIZ-BG-MODPACK `0.2.0-alpha.1`,
CHRIZ-SOD-REMIX `0.6.4`, Artisan's Kitpack `chriz-v1.3.1`, Spell Revisions
`v4.21-chriz.3`, and public CHRIZ-BG-REBALANCE `v0.3.1` with its ten-component curated
fresh selection.

Tasks 19, 20, and 23 are implemented: public-alpha omissions and evidence are
release-enforced; immutable recipe envelopes, update classification, and packaging are in
place; and the Tauri command surface includes restart-safe discovery and resume through the
immutable campaign index. Task 24 has a tested three-track update-center foundation, but
the production updater channel is deliberately still unconfigured.

The player-facing app is now **Chriz Easy BG (CEBG)**. With no registered installation it
opens directly on one compact install screen with detected sources, editable install name
and location, recommended choices, a clear readiness state, and one primary Install action.
With an existing installation it opens as a launcher with Play, Open game folder, install
switching, collapsed technical paths, and recovery for resumable or moved installs. The
optional CEBG desktop shortcut is checked by default and points back to the registry-verifying
launcher rather than directly to the game. Startup remains registry-first, so a completed
game can be launched even when its original source installs are unavailable.

Fresh verification after this UX slice passes 54 frontend tests, 37 native app tests, full
workspace tests and Clippy, and public-alpha validation with zero findings. Responsive checks
pass at 1920x1080, 1366x768, 768x1024, and 375x812: desktop keeps the primary action visible
without a scrollbar, and constrained screens use one column with normal internal scrolling.
The final local NSIS lifecycle smoke passed for the **4,413,788 byte**
`Chriz Easy BG_0.1.0-alpha.1_x64-setup.exe` with SHA-256
`7BF879133A0E49E67A811F85BCAFF98FC61AFA34B32207314F51375D4F42F227`: its branding and
version were correct, all 78 bundled manifest files hash-matched, the installed app stayed
alive for five seconds, and silent uninstall removed the isolated smoke directory.

There is **no full game-install acceptance and no public release yet**. The immediate E2E
blocker is a genuinely clean Steam BG:EE+SoD 2.7.3 source; the detected Steam BG2 source is
clean, while the available BG1 source is rejected for mod residue. These checks were
read-only and no game directory was modified. Production Minisign and Tauri updater keys
also remain to be provisioned, and Task 24's signed updater UI remains pending.

Immediate next actions only:

1. Obtain or restore a genuinely clean Steam BG:EE+SoD 2.7.3 source.
2. Run the first full installer-driven clean EET build and focused smoke before the public
   installer release.
3. Provision the production signing keys/channel and publish the first explicitly alpha
   build only after those gates pass.

Other blocked items remain later work and are not expanded here. Older status sections
below are retained as historical implementation context.

## Status (2026-09-02 — curation snapshot integrated; real-alpha implementation active)

The authoritative component-catalog snapshot from the dirty `main` checkout is preserved
in commit `1eedd8e` and integrated here without modifying that checkout. The normalized
decision semantics and per-component notes live under `docs/curation/components/`. Start
with [`docs/next-session.md`](next-session.md), use
[`FOLLOW_UPS.md`](curation/components/FOLLOW_UPS.md) for the complete evidence-backed queue,
and use [`COLLECTION_TAIL_FIXES.md`](curation/components/COLLECTION_TAIL_FIXES.md) for the
22-fix migration inventory. Explicit choices and source/release/acceptance gates remain;
do not treat every `default` row as currently installable.

The active implementation branch is `codex/installer-v0-real-alpha`. The approved current
milestone is defined by `docs/plans/2026-09-02-installer-v0-real-alpha-design.md` (`ae5855d`)
and `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md` plus its separate
27-task ledger (`fd73921`). This supersedes the fixture-only Recipe Preview as the first
release milestone. The new plan explicitly selects `ureq`'s `rustls`,
`platform-verifier`, and `win-system-proxy` features.

**Current RC status (2026-09-03):** `C:\BG-EET-RC-20260902` has been played for an
extended session without a crash, and BuffBot works. It is not suitable as the public
alpha: Item Randomiser v8.1's physical CRE-item removal left invalid inventory offsets
and references in 95 post-Randomiser creature resources, which can duplicate or omit loot
and equipment. There is no evidence of a crash or save-file corruption. Released v8.1.1
fixes the source defect; a clean rebuilt RC, new game, and Tarnesh loot smoke remain open.

## Engine Phase 1 baseline

The branch includes the `feat/engine-phase1` lineage through Task 6. Historical plan =
`docs/plans/2026-08-20-engine-phase1-implementation.md` plus its status ledger. Crate
`engine/` provides lib `bg_engine` and bin `chriz-bg-install`.

| Task | State | Notes |
|---|---|---|
| 0 scaffold, 1 schema types (`manifest.rs`) | done, reviewed | |
| 2 loader (`loader.rs`) | done, independently reviewed | original `a4cdb6a`; fixes `05fe9f7` + `05b8058`; 16 loader tests |
| 3 validators (`validate.rs`, 9 rules) | done, independently reviewed | original `5feb0a8` (merged `65e3bb8`); fix `a0c812b`; coverage `71f0387`; 41 tests |
| 4 resolve (`resolve.rs`) | done, independently reviewed | original `13df297` (merged `98c5515`); fix `4c61a8c`; 16 tests |
| 5 events (`events.rs`) | done, reviewed | |
| 6 snapshot session persistence (`session.rs`) | done | `39a6692`; nine focused tests |
| 10 fake-game builder (`tests/support/fakegame.rs`) | done, reviewed | KEY/BIF/TLK writer |
| 7 WeiDU invocation, 8 log-diff verify, 9 runner, 11–16 | not started | historical Phase-1 ledger only |

The real-alpha plan is authoritative for next-work order. Its Task 3 intentionally replaces
the Phase-1 Task 6 snapshot model with a create-once, append-only hash-chained campaign
ledger; do not confuse those two separately numbered tasks.

**Task 2 fix-up completed locally (2026-09-01):** schema is probed before strict parsing;
stem mismatch wins over duplicate-id defence; `.toml` matching is case-insensitive; file
symlinks are followed and broken ones return pathful I/O errors; roots are canonicalized;
non-UTF-8 stems have an explicit error; `Manifest::conventional_mod_path` clearly documents
that it is not an actual loaded-file lookup; fixture mods are copied as a directory. Follow-up
tests cover a real duplicate id on case-sensitive filesystems and case-variant extensions;
Windows symlink tests skip only unsupported/permission-denied link creation. Two independent
reviews found no blocking issues; their bounded follow-up requests are now covered. The
focused RED run failed in all five intended cases before implementation.

**Task 3 review completed locally (2026-09-01):** manual sources no longer inherit the
non-manual HTTPS/hash rule; hexadecimal hashes accept either case; one explicit order slot
cannot repeat a component; split `eet_end` entries must form the final contiguous main-phase
block; and aggregation is covered with two independent errors. The four intended regressions
failed before implementation and pass after `a0c812b`.

**Task 4 review completed locally (2026-09-01):** a selected choice may add a component to
the only explicit slot of an unsplit mod even when that slot did not list it originally.
Split mods still require unambiguous explicit placement. The regression failed before the
fix and passes after `4c61a8c`.

**Historical installer v0 preview:** `docs/plans/2026-09-01-installer-v0-design.md` and its
implementation plan established the guided wizard and engine-owned UI boundaries. The
static prototype at `docs/prototypes/installer-v0/index.html` remains a no-op Recipe Preview;
it does not download, copy, or install anything and is not the current release milestone.

**How to build (Windows):** Rust 1.97 stable-msvc via rustup; run cargo from **PowerShell**
with `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` — Git Bash's coreutils `link.exe`
shadows the MSVC linker. Work in an isolated worktree; never develop in the dirty primary
checkout or either protected game/archive directory. Repo files are CRLF.

**Conventions:** TDD per task (failing test → implement → green → commit "engine: …");
`#[serde(deny_unknown_fields)]` on all manifest structs; errors carry `PathBuf`s; no
`unwrap`/`expect` in library code; doc comments on public items; never write under
`C:\Games\…` (read-only reference); never hand-edit `manifest/install-order.tsv`;
curation content is Chris-only — the engine is curation-independent.

## Background (2026-08-19 — installer app designed)

- **Installer-app foundation:** `docs/plans/2026-08-19-installer-app-design.md` established
  the public EET-only, copy-then-install architecture with Tauri 2 and a headless Rust
  engine. The 2026-09-02 real-alpha design linked above is the current extension.
- `manifest/install-order.tsv` — recaptured by `fbd914b` from the reference WeiDU.log:
  **451 entries / 91 mods**. This resolved the earlier 414-vs-364 discrepancy; the 364
  figure was wrong. Never regenerate it except from the live reference install.
- `manifest/mod-sources.tsv` — 89 source rows are classified. The integrated curation
  snapshot points historical local fixes at the migration inventory, but several immutable
  pins and release artifacts still need refreshing before recipe freeze.
- `presets/` and `app/` are not started. The headless `engine/` contains the Phase-1
  baseline; the current real-alpha plan owns the remaining engine, UI, and release work.
- Parked: [#1 EET XP scaling fix](https://github.com/Chrizhermann/chriz-bg-collection/issues/1)
  (future chriz-layer component; home repo TBD).

## Hard guardrails (user directives)

1. **The game folder (`C:\Games\Baldur's Gate II Enhanced Edition modded\`) is a READ-ONLY
   reference.** Read WeiDU.log / EET_MODDING_GUIDE.md / mod folders freely; never write,
   install, or test there.
2. **Never redistribute third-party mods.** Private archiving of hard-to-find zips is
   acceptable in a private repo, but the public-facing design is links + versions. If the
   repo ever goes public, archived third-party content must be dropped/licensing-reviewed.
3. `gh` CLI auth is shared across concurrent agent sessions — `gh auth status` before any
   gh op; this repo needs `Chrizhermann`.

## Work queue

1. Follow the dependency graph and delivery waves in
   `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md.tasks.json`; do not use
   the older Phase-0 list or Recipe Preview as the schedule.
2. Resolve only Chris's remaining content choices in `docs/next-session.md`, then normalize
   those catalog rows. Its curation decisions remain authoritative, but its older engine/UI
   scheduling paragraphs are superseded by the real-alpha plan.
3. Refresh and freeze every selected immutable source, hash, license/provenance record, and
   split BG1/BG2 run. Keep blocked or unimplemented choices unavailable.
4. Implement engine and UI slices with TDD in isolated worktrees, integrating reviewed
   commits only after their focused and workspace checks pass.
5. Treat Christopher's cold-cache install, recovery rehearsal, InfinityLoader boot,
   BG1 start, and save/reload as separate live acceptance evidence—not as implied by static
   tests or catalog review.

## Known wrinkles for the driver (from the reference install's history)

- EET requires a BG1EE+SoD source install to import from; EET_end must be the last "core"
  entry, but ~50 additive components legitimately sit after it (see install-order.tsv tail).
- WeiDU v24900 template pattern: copy `Setup-Branwen.exe` as `Setup-<modname>.exe`.
- Some mods prompt interactively despite force-install flags — capture required extra args
  per mod in mod-sources.tsv `notes` as they're discovered.
- Test installs must set a distinct `engine_name` in `engine.lua` so they don't share the
  Documents user dir (saves/baldur.lua) with the live install.

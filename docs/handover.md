# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Engine Phase 1 — status for the next agent (updated 2026-09-01, branch `feat/engine-phase1`)

Written for a ChatGPT/Codex pickup (Claude budget exhausted this week). Project files are
the source of truth; this section is the entry point for the engine work.

**Where:** branch `feat/engine-phase1`; 24 commits ahead of `main` after the local Task 2
review follow-up commit (not yet pushed). Plan =
`docs/plans/2026-08-20-engine-phase1-implementation.md` (17 TDD tasks) + status file
`…implementation.md.tasks.json`. Crate `engine/` (lib `bg_engine`, bin `chriz-bg-install`).
86 tests green, `cargo fmt --check` + `cargo clippy --workspace --all-targets -D warnings` clean.

| Task | State | Notes |
|---|---|---|
| 0 scaffold, 1 schema types (`manifest.rs`) | done, reviewed | |
| 2 loader (`loader.rs`) | done, independently reviewed | original `a4cdb6a`; fixes `05fe9f7` + review follow-up; 16 loader tests |
| 3 validators (`validate.rs`, 9 rules) | merged `65e3bb8`, **unreviewed** | 34 tests |
| 4 resolve (`resolve.rs`) | merged `98c5515`, **unreviewed** | 15 tests |
| 5 events (`events.rs`) | done, reviewed | |
| 10 fake-game builder (`tests/support/fakegame.rs`) | done, reviewed | KEY/BIF/TLK writer |
| 6 session, 7 WeiDU invocation, 8 log-diff verify, 9 runner, 11–16 | not started | 6/7/8 unblocked now |

**Next, in order:**
1. Independently review Tasks 3 and 4 against the plan text (rules 1–9; resolver semantics
   incl. "components in any option's `adds_components` are excluded from the baseline").
2. Tasks 6, 7, 8 (plan sections; all depend only on merged work), then 9 → 11 (real-WeiDU
   test gated on `CHRIZ_WEIDU_EXE`), 12–16.
3. Open decision for Task 13: `ureq` is built with only `rustls` (static WebPKI roots, env-var
   proxy only) — confirm with Chris or add `platform-verifier` before writing the downloader.

**Task 2 fix-up completed locally (2026-09-01):** schema is probed before strict parsing;
stem mismatch wins over duplicate-id defence; `.toml` matching is case-insensitive; file
symlinks are followed and broken ones return pathful I/O errors; roots are canonicalized;
non-UTF-8 stems have an explicit error; `Manifest::conventional_mod_path` clearly documents
that it is not an actual loaded-file lookup; fixture mods are copied as a directory. Follow-up
tests cover a real duplicate id on case-sensitive filesystems and case-variant extensions;
Windows symlink tests skip only unsupported/permission-denied link creation. Two independent
reviews found no blocking issues; their bounded follow-up requests are now covered. The
focused RED run failed in all five intended cases before implementation.

**How to build (Windows):** Rust 1.97 stable-msvc via rustup; run cargo from **PowerShell**
with `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` — Git Bash's coreutils `link.exe`
shadows the MSVC linker. Work in a git worktree (e.g. `.claude/worktrees/engine-phase1`),
never in the main checkout while Chris curates on `main`. Repo files are CRLF.

**Conventions:** TDD per task (failing test → implement → green → commit "engine: …");
`#[serde(deny_unknown_fields)]` on all manifest structs; errors carry `PathBuf`s; no
`unwrap`/`expect` in library code; doc comments on public items; never write under
`C:\Games\…` (read-only reference); never hand-edit `manifest/install-order.tsv`;
curation content is Chris-only — the engine is curation-independent.

## Status (2026-08-20 — Phase 0 data done, WAITING ON USER CURATION)

**Current state: paused at the curation gate.** Phases 0.1–0.3 are done (manifest
re-captured, all sources resolved + liveness-checked, 2.7 pin-list drafted). The user is
now curating independently: he will produce his own list from `docs/curation-worksheet.md`
(+ answer the 5 review questions in `docs/pin-list-2.7.md`) and then discuss it. Until
then: no curation suggestions, no manifest authoring. Independent work that may proceed
if asked: Phase 1 engine scaffolding (`engine/` crate — schema parser, WeiDU runner,
session model are curation-independent).

## Background (2026-08-19 — installer app designed)

- **Installer-app design APPROVED**: `docs/plans/2026-08-19-installer-app-design.md` is the
  canonical plan (public curated-compilation installer, EET-only, 2.7.3.0, copy-then-install,
  Tauri 2.x with a headless Rust engine crate living in this repo). Research base with all
  version pins and hosting facts: `docs/research/2026-08-19-installer-app/`.
- `manifest/install-order.tsv` — captured 2026-07-03 from the reference WeiDU.log
  (414 rows) — ⚠ the live WeiDU.log now shows **364** entries; re-capture is Phase 0.1.
- `manifest/mod-sources.tsv` — SKELETON, all TODO (the reference EET_MODDING_GUIDE
  documents zero URLs; sources must come from research — see mod-hosting brief).
- `presets/` — empty; `engine/`, `app/` — not started (Phase 1/2).
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

## Work queue (in order — from the approved installer-app design, Phase 0 first)

1. **Re-capture manifest** from the live reference WeiDU.log (resolves 414-vs-364 row
   discrepancy); reconcile with the 18-phase order in the game dir's EET_MODDING_GUIDE.md.
2. **Fill `manifest/mod-sources.tsv`** — DONE 2026-08-19 except sha256: all 89 mods
   classified (37 auto-fetchable via GitHub, 2 manual [Bristlelick weaselmods, Evandra G3
   page-gated], 1 private [BASTIL], 49 local/chriz-layer); all 30 pinned URLs
   liveness-checked 200. Remaining: sha256 column — compute when first building the
   download cache (downloads several GB; deferred). Note: BRISTLELICK v2.4 is no longer
   publicly downloadable (site has 2.5.1 only) — local archive copy preserves the pin.
3. **2.7 pin-list** — EET master SHA ≥ 2026-08-06, EEex ≥ v1.1.5, SCS 35.21 + WeiDU 249
   (per-mod WeiDU pins!), SR 4.21, cdtweaks v18, EEFP Beta 2; flag every divergence from
   the 2.6-era reference for user review (`game-version-landscape.md`).
4. **Curation pass — HARD HUMAN GATE** (user directive 2026-08-19): worksheet at
   `docs/curation-worksheet.md` (mod inventory by source class, blank decision columns,
   user's open questions). The user curates
   content (drop/add mods, toggles, choice groups, presets) exclusively himself. Agents
   deliver neutral inventories/option lists with factual compat notes only; suggestions
   only when explicitly asked. Output → `manifest/collection.toml` + `manifest/mods/*.toml`
   (schema v1 in the design doc), authored from the user's filled worksheet.
5. **Phase 1: engine crate + CLI** — milestone: full unattended EET test install from
   scratch on 2.7.3.0 (distinct `engine_name`, never in the reference game dir).
6. **Phase 2: Tauri app**; **Phase 3: signing/updater/manifest release channel** — see
   design doc. Config layer (ini tweaks as per-preset diffs) folds into the manifest work.

## Known wrinkles for the driver (from the reference install's history)

- EET requires a BG1EE+SoD source install to import from; EET_end must be the last "core"
  entry, but ~50 additive components legitimately sit after it (see install-order.tsv tail).
- WeiDU v24900 template pattern: copy `Setup-Branwen.exe` as `Setup-<modname>.exe`.
- Some mods prompt interactively despite force-install flags — capture required extra args
  per mod in mod-sources.tsv `notes` as they're discovered.
- Test installs must set a distinct `engine_name` in `engine.lua` so they don't share the
  Documents user dir (saves/baldur.lua) with the live install.

# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Status (2026-09-01 — curation reviewed; consolidation and engine work active)

Chris has completed the component-catalog pass. The normalized decision semantics,
catalog notes, 22-fix migration inventory, and durable unresolved queue live under
`docs/curation/components/`. Start the next session with
[`docs/next-session.md`](next-session.md); use
[`FOLLOW_UPS.md`](curation/components/FOLLOW_UPS.md) for the complete evidence-backed
queue. A small set of explicit choices and multiple source/release/acceptance gates remain,
so do not treat every `default` row as currently installable.

Engine Phase 1 is developed in the isolated `feat/engine-phase1` worktree/branch. Loader
Task 2 is now independently reviewed and closed by local commits `05fe9f7` + `05b8058`;
validator Task 3 is closed by `a0c812b` plus test coverage `71f0387`, and resolver Task 4
by `4c61a8c`. All three reviews are complete and 94 tests are green. Tasks 6, 7, and 8 are
next. Commit `f398127` adds the bounded installer-v0 design, implementation plan, and
no-op static Recipe Preview. The branch's top handover section is authoritative for
commit-level state. Do not merge it over this dirty curation checkout.

## Background (2026-08-19 — installer app designed)

- **Installer-app design APPROVED**: `docs/plans/2026-08-19-installer-app-design.md` is the
  canonical plan (public curated-compilation installer, EET-only, 2.7.3.0, copy-then-install,
  Tauri 2.x with a headless Rust engine crate living in this repo). Research base with all
  version pins and hosting facts: `docs/research/2026-08-19-installer-app/`.
- `manifest/install-order.tsv` — captured 2026-07-03 from the reference WeiDU.log
  (414 rows) — ⚠ the live WeiDU.log now shows **364** entries; re-capture is Phase 0.1.
- `manifest/mod-sources.tsv` — source inventory exists, but completed curation has made
  several pins and all legacy local-fix notes stale; refresh it from the catalogs before
  manifest authoring.
- `presets/` and `app/` are not started on `main`. Engine work exists on
  `feat/engine-phase1`; the app has an approved product design and a bounded v0 design on
  that branch.
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
4. **Curation pass:** component review is substantially complete. Resolve the remaining
   choices in `docs/next-session.md`, then author only release-ready rows into
   `manifest/collection.toml` and `manifest/mods/*.toml`. Blocked components remain
   unavailable until their owning repositories and acceptance evidence are ready.
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

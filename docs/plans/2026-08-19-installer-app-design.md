# Installer app — design

**Date:** 2026-08-19 · **Status:** APPROVED — user validated every section in-session.
Supersedes the "install driver v0 script" concept in the original work queue; the driver
becomes the engine's CLI face. Research base: `docs/research/2026-08-19-installer-app/`
(7 sourced briefs + gap check, captured 2026-08-18).

## What we are building

A **curated-compilation installer**: a signed, cross-platform-capable GUI app that installs
Chris's modded EET stack onto a user's own BG1:EE(+SoD) + BG2:EE installs — one click for
the recommended setup, small curated customization on top. Not a general mod manager
(Project Infinity / BIO already exist); the app installs *this* compilation only.

## Decisions (all user-approved)

The signing-provider assumptions in this historical design are superseded by the
[2026-09-07 signing audit](../audits/2026-09-07-signing-dependency-review.md).
MIT source publication and SignPath Foundation acceptance are separate decisions;
the current linked UnRAR dependency prevents assuming OSI-only eligibility.

| Decision | Choice | Key rationale |
|---|---|---|
| Audience | Public community release | Sets quality bar; downloads only from official author sources (never rehost — BWS/Roxanne blacklisting precedent) |
| Product shape | Curated compilation installer | Simple-persona UX; full component tree at most a later "Advanced" door |
| Game targets | EET only (v1) | Requires BG1:EE+SoD and BG2:EE; standalone variants would triple the matrix |
| Game build | 2.7.3.0 only | Steam/GOG users cannot get 2.6.6; matches intent to modernize off the 2.6-era reference |
| Install model | Copy-then-install | Originals stay pristine; updates = rebuild from clean; append-only WeiDU stack |
| Platforms | Windows-first, cross-platform foundation | Mac/Linux later; EEex is Windows-only → per-mod `platforms` metadata, degraded no-EEex variant deferred as open option |
| Customization | Whole-mod toggles + curated choice groups | Defaults = Chris's recommended install; UI selection is a choice group |
| Stack | Tauri 2.x (Rust core + web UI) | ~5 MB installer, Rust ideal for process orchestration/downloads/multi-GB copies, signed built-in updater, active IE-tool ecosystem is Rust (BIO, mod_installer, modda) |
| Engine | Own Rust crate, not a mod_installer wrapper | Manifest far richer than a replayed WeiDU.log; WeiDU surface fully mapped; crib mod_installer's prompt-detection patterns |
| Repo layout | App lives in THIS repo | Manifest schema and engine co-evolve; two release channels via tag prefixes (`app-v*` vs `manifest-*`); split later only if it hurts |
| License / signing | MIT source; signed release binaries | SignPath Foundation (free OSS signing) first, Azure Trusted Signing (~$10/mo, EU self-employed eligible) fallback; unsigned = SmartScreen kills trust (PI's #1 complaint is AV false positives) |

## Load-bearing research findings

- **Append-only is the core invariant.** WeiDU.log is a stack; installing strictly in
  manifest order never triggers the uninstall/reinstall cascade. No mid-stack surgery, ever.
  Failed component → halt + resume (or rebuild), never skip-and-continue.
- **Exit code 0 proves nothing** (already-installed is a silent 0 no-op). Ground truth =
  WeiDU.log snapshot diff per mod run + debug-log grep (`NOT INSTALLED DUE TO ERRORS` /
  `INSTALLED WITH WARNINGS`). Exit map: 0 ok, 2 install failure, 3 warnings, 4 parse, 6 arg-warn.
- **Invocation template:** `<pinned-weidu> <tp2> --language N --use-lang en_US
  --force-install-list <ids> --no-exit-pause --skip-at-view --safe-exit --log <out>`.
  `--use-lang` is mandatory even for English (otherwise stdin-blocking prompt). READLN has
  no bypass flag → scripted stdin per component (3 known in stack: bardic wonders #1008
  "1", c0warlock #0 "2", randomiser #1100 "y"). Watchdog for stalled/prompting processes.
- **WeiDU is a compatibility axis:** SCS 35.21 breaks under WeiDU 251 (`finbalth.bcs`
  PARSE ERROR) → **per-mod WeiDU pins** (249 for SCS), never blanket-upgrade.
- **2.7 pin-list (2026-08):** EET = master commit SHA ≥ 2026-08-06 (v14.1 tag is NOT
  2.7-clean; repo now Gibberlings3/EET), EEex ≥ v1.1.5 (v1.2.0 current; left alpha;
  Windows-only; patch-version-locked), SCS 35.21 + WeiDU 249, Spell Revisions v4.21,
  cdtweaks v18, EE Fixpack Beta 2 (still beta). 2.7 wipes mods on update → copy model +
  build check protect users.
- **Hosting:** ~85–90 % of the stack has stable GitHub `releases/download/` URLs (G3, SHS,
  PPG orgs all migrated; release-asset downloads don't consume API rate limits — embed
  pinned URLs, no API at install time). Holdouts: weaselmods (manual-download queue:
  open page, watch drop folder, verify hash), Artisan's Kitpack (branch zip only → pin
  commit-SHA archive URL + content hash). Optional PAT only for update checks.
- **Reference-install intel** (EET_MODDING_GUIDE): 18-phase canonical order, per-mod
  gotchas/prohibitions (Divine Remix, Wheels of Prophecy, Sandrah), ajantisbg2 uses
  `--language 1`, pre-merge BG1 phase exists (DLC Merger, BG1 NPC Project, BG1 UB), the
  guide documents **zero** download URLs, and the live WeiDU.log counts **364** entries vs
  the 414 captured in `install-order.tsv` → re-capture required.
- **Tool landscape 2026:** BIO = Born2BSalty's Infinity Orchestrator (Rust/egui alpha GUI
  wrapping dark0dave's mod_installer); PI semi-dormant but its metadata ini is the
  community standard (stay interop-friendly; PI-compatible export remains a later option).

## Architecture

### Repo layout

```
chriz-bg-collection/
├── manifest/
│   ├── collection.toml     # game-build pin, phases, flat install order, toggles,
│   │                       #   choice groups, presets, validation rules
│   ├── mods/<id>.toml      # per-mod: version, source+sha256, weidu pin, language,
│   │                       #   platforms, phase, components (+stdin answers)
│   └── (install-order.tsv, mod-sources.tsv stay as captured reference data)
├── presets/                # named toggle/choice sets; "chris-recommended" = default
├── engine/                 # Rust crate, zero Tauri deps; also builds the CLI binary
├── app/                    # Tauri 2.x shell (web UI wizard)
└── docs/
```

Dependency direction: `app → engine → manifest`. Manifest is **pure data** — every gotcha
is a field, no mod-specific code in the engine; all learned constraints (ordering rules,
prohibitions, knock-on exclusions) become CI-checked validation rules.

### Manifest schema v1 (sketch)

Per-mod file: `id`, `name`, `version`, `tp2`, `language`, `weidu` (pin), `platforms`,
`phase` (`bg1-pre-merge | main | post-eet-end`), `[source]` (`kind` =
`github-release | github-tag-archive | github-commit-zip | manual`, `url`, `sha256`),
`[[components]]` (`id` = DESIGNATED number — subcomponent choices are just numbers —
optional `stdin`). `collection.toml`: `game_build`, stack-level pins (EET master SHA),
flat `order` (mod/component granularity where needed, e.g. Artisan Kitpack Tweaks
pre/post-cdtweaks split), toggles (with knock-on exclusions, e.g. dropping Spell Revisions
also drops SCS #2000 spell-tweak layer + SR #60), choice groups (UI, SCS difficulty, XP
options — mutually exclusive component swaps), presets.

### Engine pipeline (persisted session, resumable at every step)

1. **Preflight** — detect Steam/GOG installs (manual browse fallback), verify build ==
   2.7.3.0 (friendly hard fail otherwise), disk-space check, target-not-inside-game-dir check.
2. **Resolve** — preset + toggles → concrete ordered component list; run all validators;
   plan recorded in session.
3. **Acquire** — everything downloaded to a content-addressed sha256-verified cache
   *before* any install step; resumable; polite backoff; manual-download queue for
   `kind = "manual"`.
4. **Stage** — copy both game dirs; set distinct `engine_name` in `engine.lua` (own
   Documents/saves dir, no collision with existing installs).
5. **Install** — BG1 pre-merge phase → EET merge → main → post-EET_end; per mod: extract
   from cache → pinned WeiDU with the invocation template above → stream stdout live;
   watchdog flags stalls.
6. **Verify** — per mod run: WeiDU.log diff + debug grep; never trust exit codes alone.
   Failure → halt, preserve, offer retry/abort; never skip.
7. **Report** — machine-readable JSON report + one-click diagnostics bundle (session,
   debug logs, versions).

### GUI/UX (wizard; happy path = 3 clicks)

Welcome (requirements, cost) → Games (auto-detect, inline build check) → Location →
Customize (*skippable*; toggles + choice groups only, recommendation marked; no component
tree in v1) → Review & download (sizes; manual queue if needed) → Install (per-phase
progress, ETA, collapsed live console; failure card with retry + diagnostics export) →
Done (Launch via InfinityLoader, saves location, "changes = rebuild" honestly explained).

### Distribution & updates

- MIT source; release .exe signed (SignPath → Azure Trusted Signing fallback).
- App updates: Tauri built-in signed updater off GitHub Releases (`app-v*` tags).
- Recipe updates: app fetches latest `manifest-*` release at startup — recipe evolves
  without shipping binaries; manifest releases gated by CI (schema, rules, URL/hash liveness).
- Trust posture in README: downloads only from official author sources, never rehosted.

## Roadmap

**Phase 0 — recipe groundwork** (valuable regardless of app):
1. Re-capture manifest from live reference WeiDU.log (resolve 414 vs 364) + reconcile
   with the 18-phase guide order.
2. Fill `mod-sources.tsv`: URL + version + sha256 for all ~80 mods (scripted verification).
3. Build the 2.7 pin-list; flag every divergence from the 2.6-era reference for review.
4. Curation pass with Chris: drop/add mods (NPC review deferred from kickoff), define
   toggle list + choice groups + shipped UI options. Output = `collection.toml` + `mods/*.toml`.

**Phase 1 — engine (CLI)**: parser/validators → acquire → stage → WeiDU runner +
verification → session/resume. **Milestone: full unattended EET test install from scratch
on 2.7.3.0 via CLI** (validates recipe + engine before any UI).

**Phase 2 — app**: Tauri shell, wizard, live console, manual-download queue, failure UX.
**Milestone: same install, three clicks.**

**Phase 3 — release**: signing, updater, manifest release channel + CI, docs, community
beta. IWD:EE sibling app is out of scope until after this.

## Open items

- App name (needed before Phase 2/3; repo name stays chriz-bg-collection).
- Curated toggle/choice-group content list (Phase 0.4, with Chris).
- Shipped UI options + default (Chris's current UI = default).
- Mac/Linux no-EEex variant: deliberately undecided; `platforms` metadata keeps it open.
- [#1 EET XP scaling fix](https://github.com/Chrizhermann/chriz-bg-collection/issues/1) —
  future chriz-layer component, joins the manifest once built.
- 364-entry WeiDU.log vs 414-row TSV discrepancy — resolved by Phase 0.1 re-capture.

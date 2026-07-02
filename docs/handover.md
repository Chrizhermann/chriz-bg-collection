# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Status (2026-07-03 bootstrap)

- `manifest/install-order.tsv` — CAPTURED from the reference install's WeiDU.log
  (414 components, 84 mod folders). This is ground truth for ordering.
- `manifest/mod-sources.tsv` — SKELETON, all TODO. Filling it is the first real task.
- `presets/`, `tools/` — empty.

## Hard guardrails (user directives)

1. **The game folder (`C:\Games\Baldur's Gate II Enhanced Edition modded\`) is a READ-ONLY
   reference.** Read WeiDU.log / EET_MODDING_GUIDE.md / mod folders freely; never write,
   install, or test there.
2. **Never redistribute third-party mods.** Private archiving of hard-to-find zips is
   acceptable in a private repo, but the public-facing design is links + versions. If the
   repo ever goes public, archived third-party content must be dropped/licensing-reviewed.
3. `gh` CLI auth is shared across concurrent agent sessions — `gh auth status` before any
   gh op; this repo needs `Chrizhermann`.

## Work queue (in order)

1. **Fill `manifest/mod-sources.tsv`** — for each of the 84 mod folders: installed version
   (from install-order.tsv component strings + SETUP-*.DEBUG in the game dir), download URL
   (EET_MODDING_GUIDE.md in the game dir documents most; else G3/SHS/GitHub search), and
   archive path in the local mod archive (`C:\Games\Baldurs Gate 1 and 2 mods\`).
2. **`presets/chris-full.tsv`** — formalize the reference selection (initially = every row
   of install-order.tsv; becomes the "recommended" preset).
3. **Install driver v0** (`tools/`) — script that, given a preset: verifies mod presence,
   copies each mod into a target game dir, runs
   `Setup-<mod>.exe --force-install-list <comps> --language 0 --no-exit-pause` in manifest
   order, halts on failure with resume support. First validation target: building the
   planned EET test install from scratch.
4. **Config layer** — capture ini-level tweaks (e.g. `stratagems.ini`) as documented diffs
   per preset.
5. Project-Infinity-compatible metadata export (later; optional).

## Known wrinkles for the driver (from the reference install's history)

- EET requires a BG1EE+SoD source install to import from; EET_end must be the last "core"
  entry, but ~50 additive components legitimately sit after it (see install-order.tsv tail).
- WeiDU v24900 template pattern: copy `Setup-Branwen.exe` as `Setup-<modname>.exe`.
- Some mods prompt interactively despite force-install flags — capture required extra args
  per mod in mod-sources.tsv `notes` as they're discovered.
- Test installs must set a distinct `engine_name` in `engine.lua` so they don't share the
  Documents user dir (saves/baldur.lua) with the live install.

# chriz-bg-collection

Umbrella/orchestrator repo: manifest + install order + presets + install driver for a
heavily-modded EET / IWD:EE stack. **Bundle the recipe, not the mods** — this is a public
source repository. Do not add third-party mod/game archives or private diagnostic data.

Read `docs/handover.md` first — it is the live entry point (status, guardrails, work queue).

`AGENTS.md` is the canonical cross-agent guidance for this repository (used natively by
Antigravity and Codex; `CLAUDE.md` imports `@AGENTS.md`).

## Rules

- Always load and follow both `bg-modding` (`~/.agents/skills/bg-modding/SKILL.md`) and
  `bg-modding-learn` (`~/.agents/skills/bg-modding-learn/SKILL.md`) on ALL Baldur's Gate
  (BG:EE, SoD, BG2:EE, EET) and Icewind Dale (IWD:EE) modding, installer, WeiDU, EEex, and
  `.menu` work — including `Chriz Easy BG` and `Chriz Easy IWD`.
- Follow the current bg-modding `references/compatibility-policy.md` ("We Overwrite"): write
  the decided result regardless of old game content. Do not add version, content or layout
  gates. Preserve PC safety, CEBG archives, install history, concurrency, backups and
  recovery. Failed components keep the player's Try again / Skip / Stop decision.
- Record newly verified Infinity Engine / WeiDU / EEex / platform findings and shipped
  reusable fixes via `bg-modding-learn` (and run its skill backup script).

- Chriz Easy BG and Chriz Easy IWD work and releases come from `main` (Christopher,
  2026-11-08). One code base builds two flavors (`tauri.conf.json` / `tauri.iwd.conf.json`),
  with separate releases, updater feeds, and CI checks on both `windows-2022` and
  `ubuntu-24.04`. See `docs/BUILDING.md` and each product's release runbook.
- Target runtime on Linux / Steam Deck: native Linux x86_64 app + CLI (`chriz-bg-app`,
  `chriz-easy-iwd`, `chriz-bg-install`) driving native Linux WeiDU 251 (`v251.00`, no
  `tolower` or case-insensitive FUSE needed) against a Windows build of the game launched via
  Steam Proton / Wine (required for EEex / `InfinityLoader.exe`). Note: `InfinityLoader.exe`
  detects Wine/Proton automatically at runtime (`wine_get_version`) — never write
  `ProtonCompatibility` into `InfinityLoader.ini`.
- Game dir `C:\Games\Baldur's Gate II Enhanced Edition modded\` = READ-ONLY reference.
  Never write/install/test there. Mod archive `C:\Games\Baldurs Gate 1 and 2 mods\` is also
  read-only reference.
- `manifest/install-order.tsv` is ground truth captured from the reference WeiDU.log —
  regenerate only from the live install, never hand-edit order.
- The chriz-layer repos (modpack, bg-rebalance, sod-rebalance, *-Balance-Patch) are
  independent; this repo only references/pins them, never absorbs their content.
- Our own mods (`manifest/own-mods.toml`): pin a new release in every installer (Chriz
  Easy BG and Chriz Easy IWD) in the same step, or record Christopher's hold. Follow
  `docs/own-mod-pins.md` on every pin; `tools/own_mod_pins.py` checks it in CI.
- `gh auth status` before any gh operation (shared CLI state across concurrent sessions);
  this repo lives on the `Chrizhermann` account (pass `GH_TOKEN = (gh auth token --user Chrizhermann)`
  per command instead of running `gh auth switch`).
- A session deletes its worktree's `target/` (`cargo clean`) once its work is merged;
  Rust build output is 15–35 GB per worktree.

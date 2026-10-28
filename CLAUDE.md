# chriz-bg-collection

Umbrella/orchestrator repo: manifest + install order + presets + install driver for a
heavily-modded EET stack. **Bundle the recipe, not the mods** — this is a public
source repository. Do not add third-party mod/game archives or private diagnostic data.

Read `docs/handover.md` first — it is the live entry point (status, guardrails, work queue).

## Rules

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
  this repo lives on the `Chrizhermann` account.
- Domain knowledge (WeiDU, EET quirks, install gotchas) lives in the `bg-modding` skill.
- A session deletes its worktree's `target/` (`cargo clean`) once its work is merged;
  Rust build output is 15–35 GB per worktree.

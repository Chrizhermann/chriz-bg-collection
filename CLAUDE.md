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
- `gh auth status` before any gh operation (shared CLI state across concurrent sessions);
  this repo lives on the `Chrizhermann` account.
- Domain knowledge (WeiDU, EET quirks, install gotchas) lives in the `bg-modding` skill.

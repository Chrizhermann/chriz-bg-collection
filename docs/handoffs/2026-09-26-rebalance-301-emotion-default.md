# Handoff: make BG Rebalance 301 (Emotion, Courage / Hope exclusivity) a default

**For the collection release line (`codex/installer-v0-real-alpha`).** First finish and
commit the in-progress worktree state there (recipe alpha.17 / app alpha.19 work is
still uncommitted). Then add this change to the next recipe.

## Decision

Christopher, 2026-09-26: BG Rebalance component **301** becomes a recommended default,
accepted on the basis of earlier in-game testing. Fix any issue in a follow-up release
rather than gating on further testing.

## Published source (verified)

- [BG Rebalance v0.6.0](https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/v0.6.0),
  tag on merge commit `4aa6110` (PR #3).
- Asset `chriz-bg-rebalance-v0.6.0.zip`: 1,724,047 bytes, SHA-256
  `e5f39cb60ecf90a715715d95848df4582168e55cdcff4fc5455bc768f3063224`. GitHub's
  server-side digest matches; anonymous download redirects to
  `release-assets.githubusercontent.com`.
- 49 entries, 2,355,002 bytes uncompressed, largest entry 1,364,992 bytes (the setup
  exe). Same layout as v0.5.0: `chriz-bg-rebalance/` and `setup-chriz-bg-rebalance.tp2`
  at the ZIP root.
- v0.4.0 → v0.6.0 only adds 420/421 (v0.5.0 bard progression) and 301. Every currently
  selected component is unchanged; the only removed lines are one comment and `VERSION`.
- 369 tests pass (26 capture-gated dragon tests skip); 44 of them cover 301.

## Recipe changes

1. Replace artifact `chriz-bg-rebalance-0.4.0` with `chriz-bg-rebalance-0.6.0` in the
   manifest and recipe (plus evidence and mod-sources records, as for earlier pins).
2. `mods/chriz-bg-rebalance.toml`: new `artifact_id`; add component 301,
   "Emotion, Courage / Emotion, Hope — mutually exclusive beneficial emotions".
   420/421 exist in the artifact but stay unselected; bard progression is its own decision.
3. Run `chriz-bg-rebalance-bg2` in TP2 declaration order:
   `[100, 101, 110, 111, 120, 121, 301, 400, 401, 404, 405, 407, 408]`.
4. Feature `feature:chriz-bg-rebalance:component-301`: decision `default`, readiness
   `ready`, parent `mod:chriz-bg-rebalance`, no extra requirements. Suggested copy —
   title "Emotion, Courage and Hope don't stack"; description "Casting either Emotion
   removes the other from each target; the latest wins and recasting refreshes."
5. `docs/curation/components/CHRIZ-BG-REBALANCE.md`: add row 301 (group Cross-cutting
   audits, decision default) and update counts and curation-map totals. Regenerate the
   recipe, update golden digests and component counts, and add a new recipe version and
   ledger entry (applies to new installations / next playthrough).

## Order and compatibility (already checked)

- In the collection, 301 rebuilds IWDification's Courage and Hope in place. It must run
  after IWDification 30/40, SCS 5900 and Spell Revisions; the post-EET-end Rebalance run
  does.
- CDTweaks 2312 runs later and applies the same save penalty to every saved effect in each
  header (`cdtweaks/lib/saves_macro.tpa`), so 301's save-coupled Fear/Symbol removers stay
  aligned. Courage and Hope have no saved effects and are untouched.
- Modpack 430 only rewrites scroll casts at caster level 0/instant. The IWDification and
  301 scrolls cast at fixed level 10 and are untouched.
- Earlier live tests used standalone mode; the collection's provider mode has installer
  and transformer fixture coverage only. A quick Hope/Courage check in the next install
  is worthwhile but not a release gate.

## Background

301 was implemented on 2026-08-30 but never committed; its Codex worktree was cleaned up
on 2026-09-05 and only an automatic snapshot kept it. It was recovered, ported onto
v0.5.0 and released as v0.6.0 on 2026-09-26. See the Rebalance handover and
`research/11-emotion-hope-courage.md` for the full design and evidence.

# Manifest re-capture — 2026-08-19 (Phase 0.1)

Regenerated `manifest/install-order.tsv` from the live reference WeiDU.log.

## Result

- **451 entries / 91 unique mod folders** (previous capture 2026-07-03: 414 / 84).
- **Nothing was removed** since the July capture; relative order of surviving entries
  unchanged. The design doc's "364 entries" figure (from the local-modding-guide research
  brief) was simply wrong — treat 451 as current truth.
- **37 entries added**, almost all chriz-layer work landed since July:
  - `CHRIZ-SOD-REMIX` v0.5.0 — 26 components (the SoD remix, now real)
  - `SETUP-CHRIZ-BG-REBALANCE` v0.1.0 — 6 Tempus components (+ 1 older dir-form entry, see below)
  - `SETUP-CHRIZ-BG-MODPACK` — #440, #450, #600
  - `AKCB_BERSERKER` #0, `AKCB_SHAPESHIFTER` #0 (Artisan kitpack balance patches)
  - `SETUP-ABETTORHLAREBALANCE` #0 (1.0.0), `SETUP-EEEXREMOTE` #0 (v0.2.0)
  - `CD_SAREVOK` #0 (Sarevok Soundset), `CDTWEAKS` #3347 (party movement speed [argent77])

## Capture notes

1. **Junk entries kept faithfully, excluded at authoring time**: lines 409–410 of the live
   log are `~__EXTRACT.TP2~ #0 #0` and `~__IDS.TP2~ #0 #0` with garbage names — artifacts
   of some past tool run, not mods. They stay in the TSV (ground-truth capture, never
   hand-edit) but must NOT enter `collection.toml`.
2. **Broken display name**: `~CHRIZ-BG-REBALANCE/SETUP-CHRIZ-BG-REBALANCE.TP2~ #0 #101 //
   ?????? -> : ???` is a real install whose tra didn't resolve at install time. Actual
   component: `@101 DESIGNATED 101` = "SCS: Adventurer's Mart — restore the five Freedom
   scrolls (spell tweak orphaned in v35)". Note it uses the **dir-form tp2 path** while the
   later Tempus entries use top-level `SETUP-CHRIZ-BG-REBALANCE.TP2` — same mod, two
   invocation styles across install sessions.
3. **Top-level tp2 handling**: mod folders for path-less entries (`SETUP-<mod>.TP2`) are
   now derived from the basename (strip `SETUP-`/`.TP2`).
4. Non-ASCII (em-dashes etc.) in component names round-trips correctly (UTF-8).
5. `BG2EE-EET-FIXPACK` #101 ("Branwen BG2: Kit Fix", line 385) was already in the July
   capture — noting it here because it's also chriz-layer (own fixpack), relevant to
   source classification in Phase 0.2.

## Chriz-layer folders in the manifest (source = own repos, not web research)

`CHRIZ-BG-MODPACK`, `CHRIZ-BG-REBALANCE`, `CHRIZ-SOD-REMIX`, `AKCB_BERSERKER`,
`AKCB_SHAPESHIFTER`, `ABETTORHLAREBALANCE`, `EEEXREMOTE`, `BG2EE-EET-FIXPACK`.

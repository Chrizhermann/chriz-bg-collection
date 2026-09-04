# Next session — 2026-09-02

This is the short working list for tomorrow. The evidence-backed master queue remains
[the curation follow-up list](curation/components/FOLLOW_UPS.md), and per-component choices
remain authoritative in their individual catalogs. Checking an item here must not erase
the detailed acceptance requirement in those files.

## Start with Chris's remaining choices

These are the only content decisions still needed before the corresponding recipe rows can
be considered final:

- [ ] Curate Artisan main `1010`, `2003`, `5300`, `5200`, `7000`, `7007`, `7008` and
  tweaks `20021`, `1020000`, `2000001`.
- [ ] Curate Bardic Wonders `2011`, `2012`, and `2013`.
- [ ] Decide BGGO `4` (Alternate Graphics): excluded, optional, or default.
- [ ] Choose Yes or No for Warlock's hidden Contingency spell-learning prompt.
- [ ] Decide whether the separate BG1 Sirene v3.1 package joins the already curated
  BG2/EET Sirene route.
- [ ] Choose exactly one restored-Bhaalspawn-powers provider: Ascension `40`
  (technical recommendation) or UB `19`.
- [ ] Finish the remaining NPC edge cases: Sarah remains an Archer; Dynaheir gets Haste
  known-only or known plus one memorized copy; and Kivan's Archer change is kit-only with
  no loadout/proficiency rewrite. Shar-Teel is settled as a default Wizard Slayer.

For update notices, the v0 design assumes save applicability is authored per recipe
change. It does not inspect saves or guess whether an NPC has joined. Revisit this only if
Chris explicitly wants the much larger save-analysis project.

## Do not reopen settled curation

- blank = excluded, `optional` = visible/off, `default` = visible/on, and `mandatory` =
  automatic whenever its parent feature is active.
- SCS `3017` and `6020` are accepted. SCS `4240` is visible but unavailable with Spell
  Revisions and has an explanatory UI note.
- Randomiser uses Mode 1, so Mode 2's install-order concern does not drive this recipe.
- Hidden Gameplay Options remain optional; the options used in the reference setup are
  default-on, with mutual exclusions encoded rather than hidden.
- CHRIZ-SOD-REMIX pins the public v0.6.4 release. Its implemented selection
  is one default-on parent checkbox; component `290` remains unimplemented and excluded.
- The monolithic NPC component `chriz-bg-modpack 500` is retired. Native assignments stay
  in their native mods; new collection assignments become semantic per-NPC components.

## Recommended work order

1. Preserve the dirty curation checkout; do not merge over it or commit unrelated
   `.claude/` state.
2. Answer the short decision list above and normalize those rows.
3. Fix the stale pin list and `manifest/mod-sources.tsv` notes from the completed catalog
   reviews, then freeze immutable sources for the first manifest slice.
4. Turn release-ready choices into manifest data. Keep blocked/unimplemented choices
   unavailable rather than allowing placeholders to leak into the UI.
5. Continue the engine on `feat/engine-phase1`: Tasks 2–4 are independently reviewed and
   closed; implement Tasks 6, 7, and 8 with TDD.
6. Use the approved installer design and the v0 Recipe Preview design/prototype on
   `feat/engine-phase1` for UI work; do not
   connect a real Build action before the engine can safely acquire, stage, run, verify,
   persist, and resume.

## Work that belongs in its owning repository

| Repository/layer | Required work before the affected feature is release-ready |
|---|---|
| `chriz-bg-modpack` | Replace `500` with semantic Kagain, Skie, Faldorn, Dynaheir, Kivan, Viconia, and Shar-Teel components; merge Sarah's separately reviewed components when ready. Implement conditional Fade `110`, Kivan quest `130`, Mazzy `140`, Safana `150`, Skie `160`, Branwen `400`, and Yeslick/Keldorn `410`. Retire `120`/`500`; verify retirement of `200`/`210`/`300`. Keep Safana unavailable until every class/spell/snare/late-spell/SoD-inventory piece is semantic and tested. |
| Artisan fork | `chriz-v1.3.1` is pinned with the Shapeshifter footprint and Xan EET-assignment repairs. Keep `30001/300010` unavailable; the broader priest-delivery and EEex dependency redesign remains separate. |
| Bardic Wonders fork | Integrate and release the finite Abettor-HLA work; add SR fixtures for `1008/1009/2008`; correct legacy EEex detection. |
| Aura fork | Reconcile the seven missing upstream commits, compare the old spell-patch snapshot, verify all six legacy item changes, fix component `6` EEex detection, then release. |
| Spell Revisions fork | Publish immutable `v4.21-chriz.1` and absorb `SR_SUBSPELL_FIX`; do not rebuild it as modpack `180`. |
| BG2EE/EET fix layer | Migrate Branwen `101` to a narrow maintained home; redesign stat transfer `200`; audit Edwin `400`; retire duplicate `300`. Do not expose the historical parent package. |
| `chriz-bg-rebalance` | Publish the narrow component-`401` Artisan-CLAB compatibility fix discovered and test-proven by the isolated RC, then pin that successor to v0.3.0. Components `120/121` are accepted and selected. The Tempus APR gameplay check on EEex 1.2 remains. |
| `chriz-sod-rebalance` | v0.6.4 is published and pinned; keep `290` deferred and complete the natural five-item/save-reload acceptance for `225`. |
| EEex Remote Console | Refresh the repository's stale EEex-compatibility documentation and rerun the target-stack handshake/watchdog smoke. |
| Collection tail | Build the curated EET Tweaks progressive-XP replacement and the IWDification Arcane Trickster Evasion patch. Identify the eight ownerless CIMC/CDMC animation INIs. |

## Collection/engine blockers before a real build

- Encode split runs from one artifact, especially EE Fixpack `0+2` in BG1 and BG2.
- Encode UI availability predicates, authored disabled reasons, choice groups, numeric and
  scripted prompt inputs, manual archives/assets, and parent semantics in engine-owned data.
- Update immutable pins for EET/EET_END, EE Fixpack, EEex, HGO, Randomiser, Artisan,
  Bardic, Aura, Warlock, Spell Revisions, CHRIZ-SOD-REMIX, and CHRIZ-BG-REBALANCE.
- Record an immutable install receipt and versioned update ledger. Keep app-update urgency,
  recipe-update availability, and save applicability as separate facts.
- Before Task 13, decide whether the Windows-first downloader must use the platform trust
  store/system proxy path; the current `ureq`/rustls-only proposal does not provide it.

## Live acceptance later

These checks are release evidence, not tomorrow's first task: a clean staged 2.7 EET build
with both EE Fixpack runs; EEex 1.2 modules/LuaJIT and Remote Console; SCS 35.21 under
WeiDU 249; Tempus APR; SoD Remix `225`; every selected migrated tail fix; and Sarah/Evandra
portrait verification. Modpack `600` is not part of the fresh preset while UB `3` remains
excluded; test it only in a dedicated legacy-UB fixture.

## Deliberately nonblocking

Do not let these consume the next session unless their parent feature is selected:
Randomiser's expanded Mode-1 pool, SoD Remix `290`, the full Safana
composite, priest-delivery redesign, cross-platform no-EEex support, app branding, or
automated save inspection.

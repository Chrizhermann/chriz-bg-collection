# DLCMERGER — components

The collection targets **v2.1**. Its menu was re-listed with WeiDU 249 on 2026-09-03;
the isolated BG1 release candidate installed component `1` successfully. This is current
alpha curation, not a claim that every future DLC layout should use the same route.

4 entries, 1 release-candidate-installed. ✓ = installed in the verified BG1 candidate.
Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1 | Merge DLC into game -> Merge "Siege of Dragonspear" DLC | BG1 preparation | DLC merge mode | ✓ | mandatory |
| 2 | Merge user-defined DLC | BG1 preparation | DLC merge mode |  | optional |
| 3 | Merge all available DLCs | BG1 preparation | DLC merge mode |  | optional |
| 10 | "Siege of Dragonspear" language selection fix | BG1 preparation |  |  | |

## Collection behavior

- The v0.1 alpha supports the known BGEE+SoD route, so component `1` is fixed and runs
  before EE Fixpack. Components `2` and `3` remain visible as alternative modes but are
  unavailable while the curated mode is mandatory.
- Component `3` is not available to 32-bit WeiDU. Production uses the separately verified
  64-bit WeiDU 249 executable.
- Component `10` is excluded: v2.1 component `1` performs the relevant game-2.7 language
  fix during its merge. Do not install the standalone correction a second time.

## Evidence and follow-up

- Active-run provenance is frozen in
  [`manifest/reference/bg1-premerge.tsv`](../../../manifest/reference/bg1-premerge.tsv).
- The candidate was an isolated writable copy; the source installation remained read-only.
- Re-list this menu and re-evaluate the language fix if the DLC Merger pin changes.

# EET — components

The reference install reports **v14.0**. The collection currently targets official
`master` commit `74e91d72bca5d073fa11c1d088b90d7ff0c7105d`, which contains the planned
2.7 compatibility work but whose TP2 still reports v14.0. The latest packaged release
reviewed on 2026-09-01 is v14.1; it does not contain all of that work.

2 entries, 2 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | EET core (resource importation) |  |  | ✓ | mandatory |
| 100 | Create desktop shortcut |  |  | ✓ | optional |

## Collection behavior

- Core `0` requires a BG2EE target plus a BGEE+SoD source. BP-BGT Worldmap must not be
  installed before it.
- Shortcut `100` requires core `0` and is unavailable on macOS. It is an unchecked
  option (Christopher, 2026-10-13): EET's shortcut starts `Baldur.exe` directly, so
  it skips EEex, and every install that selects it replaces the Desktop `EET.lnk`.
  The CEBG app offers its own launcher shortcut instead.
- EET and EET_END must always resolve from the same archive and exact commit.

## Release and acceptance follow-up

- Recheck the official releases immediately before manifest freeze. Prefer a v14.2-or-
  newer package only if it includes the required 2.7 changes; otherwise re-resolve and
  pin the exact official `master` SHA.
- Re-list both EET component menus if the resolved SHA changes.
- The recipe artifact is the pin; do not infer the downloadable artifact from the
  version string printed by the current TP2.
- Validate the merge from the two staged 2.7 games, including the separately required
  EE Fixpack runs, before treating the pin as accepted.

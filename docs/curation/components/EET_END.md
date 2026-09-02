# EET_END — components

Use the same official EET archive and exact commit as EET core (currently planned:
`74e91d72bca5d073fa11c1d088b90d7ff0c7105d`).
2 entries, 1 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Standard installation |  | EET end (last mod in install order) | ✓ | mandatory |
| 1 | Also update saves (no backups, check the readme file) |  | EET end (last mod in install order) |  | |

## Collection behavior

- Component `0` requires EET core and closes the normal WeiDU installation phase. Only
  explicitly inventoried and validated collection patch layers may run after it.
- Component `1` is an alternative save-migration utility, not a fresh-install option.
  It requires pre-generated `saves.tra` and `saves.txt`, cannot be uninstalled, and
  requires manual save backups. Keep it unavailable in the collection UI.
- Re-list this menu whenever the EET pin changes, and reject a manifest in which EET and
  EET_END resolve to different archives or hashes.

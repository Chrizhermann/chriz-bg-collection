# CROSSMODBG2 — components

Listed at installed and target version **v30**.
3 entries, 3 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Crossmod Banter Pack for Shadows of Amn Content |  |  | ✓ | default |
| 1 | Crossmod Banter Pack for Throne of Bhaal Content |  |  | ✓ | default |
| 2 | Crossmod Romance Conflicts |  |  | ✓ | default |

## UI, dependencies, and order

- Component `1` requires `0`; the EET target satisfies its Throne of Bhaal requirement.
- Component `2` is checked by default, but make it unavailable when the user selects a
  multi-romance route. Explain that conflict in the UI.
- Install Crossmod after every supported NPC/quest mod whose content it should detect,
  and before CDTweaks. Do not rely only on its Project Infinity `After=` metadata: v30
  supports Yeslick but omits Yeslick from that list.
- The collection's later `SAFANA2` entry is Roxanne's Safana, not the Smiling Imp
  `B7XSAFA` mod supported by Crossmod; it is therefore not evidence of a Safana ordering
  defect.

Recheck the final resolved install order against Crossmod's actual detection blocks when
the manifest is generated.

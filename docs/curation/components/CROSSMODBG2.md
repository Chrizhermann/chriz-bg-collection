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

## Sirene detection (audited 2026-10-18)

Crossmod v30 detects Sirene by `bc0siren.dlg`, but Sirene NPC for BG2:EE (pinned commit
`00beda90`, still the upstream head) ships its banter files as `BC0SIRE2` and `BC0SIR25`.
Detection therefore fails; the Crossmod logs of the reference install and of a collection
install both report "Mod not detected" for Sirene.

Crossmod's only Sirene content is one Shadows of Amn banter pair with Lava's Will NPC
(`files/will_sirene/soa_will_sirene.d`), which also addresses `BC0SIREN`. The collection
does not include Will, so it loses nothing. The fix belongs upstream in Crossmod: detect
`bc0sire2.dlg` and address `BC0SIRE2`. Revisit if Will joins the collection.

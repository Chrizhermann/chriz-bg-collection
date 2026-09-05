# ARTISANSKITPACK_NPC — components

Refreshed for the selected Artisan fork release **chriz-v1.3.1**
(`ac718614991e34b4f720807bec5edc96266c6c5e`).
18 entries, 6 reference-installed. ✓ = installed in the reference. Subgroup = choose one.

## UI/dependency notes

- `7101/7102` are mutually exclusive Imoen choices.
- `1102/1103/1104` are one Emily subgroup and remain unavailable because neither Emily
  nor SkitiaNPCs is currently part of the collection.
- Every assignment is available only when its kit and NPC are present. In particular:
  `1101→1002/1007`, `1102→1000`, `1103→1001`, `1104→2003`,
  `3101/3102→3001`, `5101→5001`, `7101→7001`, `7102→7003`,
  `7104→7005`, `21001→20001`, `9101→SoD+9001`, `10004→10003`,
  `20002→20000`, `200010→8004` plus tweak `8204`, and `99001→Bardic Wonders 1008`.
- `5102` and `10004` are unavailable with Spell Revisions.
- Garrick uses `99001` exactly once. Script the embedded Bardic Wonders `1008`
  Garrick prompt as **No** so the same assignment is not also performed there.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 1101 | Artisan's Kitpack: Give Khalid the Vanguard Kit |  |  |  | default |
| 1102 | Arcane Archer |  | Artisan's Kitpack: Make Skitia's Emily an |  | |
| 1103 | Arcane Archer / Mage |  | Artisan's Kitpack: Make Skitia's Emily an |  | |
| 1104 | Arcane Archer (Ranger) |  | Artisan's Kitpack: Make Skitia's Emily an |  | |
| 2001 | Artisan's Kitpack: Rashemi Berserker Ranger Kit for Minsc |  |  | ✓ | default |
| 3101 | Artisan's Kitpack: Give Ajantis the Divine Champion Kit |  |  | ✓ | default |
| 3102 | Artisan's Kitpack: Make Mazzy a Divine Champion |  |  |  | default |
| 5101 | Artisan's Kitpack: Give Cernd the Elementalist Kit |  |  |  | optional |
| 5102 | Artisan's Kitpack: Red Wizard Mage Kit for Edwin |  |  | ✓ | default |
| 7101 | Rogue Archer |  | Artisan's Kitpack: Make Imoen a |  | optional |
| 7102 | Trickster |  | Artisan's Kitpack: Make Imoen a |  | default |
| 7104 | Artisan's Kitpack: Give Hexxat the Invisible Blade Kit |  |  | ✓ | optional |
| 21001 | Artisan's Kitpack: Give Jan the Arcane Trickster Kit |  |  | ✓ | default |
| 9101 | Artisan's Kitpack: Give M'khiin the Warhorn Shaman Kit |  |  |  | default |
| 10004 | Artisan's Kitpack: Make Rasaad a Sacred Fist |  |  |  | default |
| 20002 | Artisan's Kitpack: Make Xan an Eldritch Knight |  |  | ✓ | default |
| 200010 | Artisan's Kitpack: Make Nalia an Arcane Trickster (Sorcerer) |  |  |  | optional |
| 99001 | Bardic Wonders: Give Garrick the Troubadour Kit |  |  |  | default |

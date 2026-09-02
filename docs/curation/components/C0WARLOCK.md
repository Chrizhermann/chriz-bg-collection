# C0WARLOCK — components

Listed at installed version **3.0**; target is exact untagged upstream commit
`a821992228c87a4bef75be4bc2fc833f8db46819` (post-v4.0 source).
1 entries, 1 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- The target source renamed its folder/TP2 to `Artisans_Warlock/Artisans_Warlock.TP2`
  and carries no `VERSION`; the manifest must pin the exact SHA and new target path.
- Component `0` asks whether to install the Contingency UI-based spell-learning system.
  That answer is absent from WeiDU.log and must be explicitly curated and scripted for
  unattended installation.
- Bubb's Spell Menu must precede Warlock for its integration path.
- The selected Artisan, Bardic, and Warlock sources carry the same ClassSpellTool helper,
  but this does not prove runtime compatibility. Audit the pit fiend's hardcoded `SPWI`
  loadout under Spell Revisions and its copied demon scripts under SCS; then validate EEex v1.2.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Warlock Kit |  |  | ✓ | default |

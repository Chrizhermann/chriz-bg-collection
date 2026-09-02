# BG2EE-EET-FIXPACK — components

Listed at installed version **1.0**.
5 entries, 5 installed. ✓ = installed. Subgroup = choose one.

## Consolidation/readiness notes

Do not expose this historical package as a selectable parent. Preserve only audited
survivors in maintained, narrowly scoped homes:

- `100` depends on Ajantis BG2 and remains unavailable while that mod is excluded.
- `101` is the desired conditional Branwen kit-transfer fix. Its blank decision means the
  historical parent remains excluded. After migration, apply the maintained replacement
  automatically only when Branwen is selected.
- `200` is incomplete for the collection's NPC set and requires a redesign before use.
- `300` duplicates Tweaks Anthology `3121`; use the Tweaks component only.
- `400` assumes Artisan's Red Wizard path but can remove Edwin's baked slots even when
  the replacement cannot be installed. Keep it unavailable pending an effective-resource
  audit and consolidation into the appropriate balance layer.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 100 | Ajantis BG2: Kit Fix (applies BG1 kit to BG2 CREs) |  |  | ✓ | |
| 101 | Branwen BG2: Kit Fix (applies BG1 kit to BG2 CREs) |  |  | ✓ | |
| 200 | EEex NPC Stat Transfer: BG1 tome bonuses carry to BG2 companions [requires EEex] |  |  | ✓ | |
| 300 | Evil NPCs: complain but never leave due to high reputation (HAPPY.2DA cap) |  |  | ✓ | |
| 400 | Edwin Spell Slot Rebalance: strip vanilla CRE-baked +2/level, kit delivers +1/level via new CLAB (AK Red Wizard) |  |  | ✓ | |

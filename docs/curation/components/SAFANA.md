# SAFANA — components

Listed at installed version **v0.5**.
1 entries, 1 installed. ✓ = installed. Subgroup = choose one.

## UI/readiness notes

- Component `0` is the sole core route and is included automatically when Safana is
  selected. The upstream installer supports EET only; keep the parent unavailable on
  other game types.
- Safana is not fresh-stack ready yet. The mod moves the live SoD Safana into SoA, which
  also carries her SoD-only inventory forward. A semantic inventory cleanup must be
  implemented and installed automatically with this mod.

## Related Safana-Bard preset

The reference stack also converts Safana to Bard/Abettor of Mask, removes her obsolete
Thief snare, and supplies Bard spells. Preserve that as a planned default preset, but keep
the whole preset unavailable for now:

- It requires Bardic Wonders `1004` and a semantic, per-NPC replacement for the
  monolithic `NPC_KIT_CHANGES` conversion.
- `SAFANA_SNARE_FIX` is valid only after the class conversion and should then be
  automatic.
- `BARD_SPELL_FIX` and `SAFANA_LATE_SPELLS` are not safe with the selected Spell
  Revisions build. They hardcode vanilla `SPWI` resource numbers that now identify
  different spells. Rewrite them against effective semantic spell identities before
  exposing or installing the preset.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Safana in Amn |  |  | ✓ | mandatory |

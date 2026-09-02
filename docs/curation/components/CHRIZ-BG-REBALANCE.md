# CHRIZ-BG-REBALANCE — components

The 14-component target menu below is release **v0.3.0**, tag commit
`94ecd323b9177f469d2c187322e4ef5c9a57bd69`. It includes the accepted component `120`
weapon-protection repair and component `121` EEex/SCS ambient-readiness bridge. The first
clean curated-stack installation also exposed a narrow component `401` compatibility gap:
v0.3.0 rejects Artisan's legitimate `AP_C0PR#CL` cells when they are packed into
`ABILITY1`. The private release candidate uses the test-first staged correction, but the
public collection must pin a follow-up release containing it rather than silently patching
v0.3.0.

14 entries, 10 current-alpha selections. ✓ = selected and receipt-verified in the isolated
2026-09-03 release candidate. Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 100 | SCS: Telekinetic Storm — restore save vs. spell for half damage (+ bypass Mirror Image) | SCS adjustments |  | ✓ | mandatory |
| 101 | SCS: Adventurer's Mart — restore the five Freedom scrolls (spell tweak orphaned in v35) | SCS adjustments |  | ✓ | default |
| 120 | SCS + Spell Revisions: repair false weapon-protection semantics | SCS adjustments |  | ✓ | mandatory |
| 121 | EEex + SCS: ambient caster readiness and one honest first-contact defense | SCS adjustments |  | ✓ | default |
| 400 | Cleric of Tempus: weapon training — axe, longsword, crossbow, and two-pip mastery | Class and kit revisions |  | ✓ | default |
| 401 | Cleric of Tempus: Holy Power — automatic semantic detection (recommended) | Class and kit revisions | Cleric of Tempus: Holy Power compatibility | ✓ | default |
| 402 | Cleric of Tempus: Holy Power — force true-doubling compatibility | Class and kit revisions | Cleric of Tempus: Holy Power compatibility |  | |
| 403 | Cleric of Tempus: Holy Power — force additive compatibility | Class and kit revisions | Cleric of Tempus: Holy Power compatibility |  | |
| 404 | Cleric of Tempus: Chaos of Battle — one announced battle tide per cast | Class and kit revisions |  | ✓ | default |
| 405 | Cleric of Tempus: Divination toll — Tempus grants no future sight | Class and kit revisions |  | ✓ | default |
| 406 | Engine-table variant: full warrior attack progression (CLSWPBON, stronger than advertised) | Class and kit revisions | Cleric of Tempus: weapon specialization grants extra attacks |  | |
| 407 | EEex variant: exactly +1/2 attack while wielding a specialized weapon (recommended) | Class and kit revisions | Cleric of Tempus: weapon specialization grants extra attacks | ✓ | default |
| 409 | Cleric of Tempus: EEex specialization APR — re-ship the listener (migration repair for the v0.1.0 runaway-APR build) | Class and kit revisions |  |  | |
| 408 | Cleric of Tempus: updated kit and ability descriptions | Class and kit revisions |  | ✓ | default |

## Atomic Tempus UI behavior

- Show one default-on checkbox named **Cleric of Tempus rework**.
- On selects exactly `400`, `401`, `404`, `405`, `407`, and `408`, in source order. Off
  selects none of them. Do not expose the six rows as independent toggles.
- Keep `402`, `403`, and `406` unavailable. Component `409` is a migration-only repair
  for an existing v0.1.0 installation and is never selected on a fresh installation.
- The rows retain `default` rather than `mandatory` because excluding the atomic Tempus
  checkbox must remain possible while using the independent SCS adjustments.

## Dependencies and order

- Component `100` requires SCS `2500` (`extra_arcane_spells`); the curated SCS preset
  includes it. Component `101` requires SCS `2000` or `5900`. Both run after SCS.
- Components `120` and `121` inspect the effective SCS/SR/EEex stack and therefore run as
  late compatibility components. On the researched stack, run `120` before `121`.
- Run the Tempus bundle after SCS, final Spell Revisions processing, Artisan's Kitpack
  proficiency infrastructure, and EEex.
- Component `400` requires Artisan's `C0PR#C4`, its permission spells, and its custom
  proficiency stat. Component `407` requires EEex.
- Component `408` consumes artifacts from `400`, `404`, `405`, and `407` and assumes the
  `401` family. The atomic bundle guarantees that chain. Component `401` supports either
  SR or non-SR spell mappings, but must inspect the final installed resources.

## Release blockers and follow-up

- Publish and pin the narrow component `401` Artisan-CLAB compatibility correction proven
  by the isolated release candidate; do not use unmodified v0.3.0 for the curated Tempus
  bundle.
- Update `manifest/mod-sources.tsv` to the resulting immutable release and its hash.
- Stage the fixed APR implementation against EEex v1.2 and verify 1.5 APR with a two-pip
  weapon, 2.5 under Holy Power, no cycling, and prompt return to normal with a zero-pip
  weapon. The earlier installed test used EEex v0.11, so it is not target acceptance.
- Components `120` and `121` are now curated and selected. Current-version component `121`
  passed ambient delivery/accounting and the neutral-to-hostile urgent path; its older-EEex
  fallback remains a nonblocking compatibility check.
- Revisit planned `110`, `200`, and `301` only if they enter the real menu.
- Recheck whether the final SCS release supersedes `100` or `101`, and refresh the source
  repository's stale component documentation before publishing.

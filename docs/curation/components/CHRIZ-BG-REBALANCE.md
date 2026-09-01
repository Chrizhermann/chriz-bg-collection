# CHRIZ-BG-REBALANCE — components

The 12-component target menu below is local `main` commit
`f9badd5fd5987ea660d74acdbf248e9391092e1c`, which declares **v0.2.0**. It is not yet
fetchable from GitHub: `origin/main` still exposes the older
11-component **v0.1.0** menu. Do not let the collection resolve that remote version while
the Tempus bundle is enabled; its component `407` is the documented runaway-APR build.

12 entries, 7 reference-installed. ✓ = installed in the captured reference WeiDU log.
Subgroup = choose one.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 100 | SCS: Telekinetic Storm — restore save vs. spell for half damage (+ bypass Mirror Image) | SCS adjustments |  |  | mandatory |
| 101 | SCS: Adventurer's Mart — restore the five Freedom scrolls (spell tweak orphaned in v35) | SCS adjustments |  | ✓ | default |
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
- Run the Tempus bundle after SCS, final Spell Revisions processing, Artisan's Kitpack
  proficiency infrastructure, and EEex.
- Component `400` requires Artisan's `C0PR#C4`, its permission spells, and its custom
  proficiency stat. Component `407` requires EEex.
- Component `408` consumes artifacts from `400`, `404`, `405`, and `407` and assumes the
  `401` family. The atomic bundle guarantees that chain. Component `401` supports either
  SR or non-SR spell mappings, but must inspect the final installed resources.

## Release blockers and follow-up

- Choose whether the release contains this 12-component `main` menu or also the unaccepted
  `120`/`121` work on `codex/ambient-readiness-121`; both currently call themselves
  v0.2.0, so the version string alone is ambiguous.
- Push the chosen fixed source, give it an unambiguous exact tag/release or commit pin,
  update `manifest/mod-sources.tsv`, and re-list the component menu. Never fetch current
  remote v0.1.0 with `407` enabled.
- Stage the fixed APR implementation against EEex v1.2 and verify 1.5 APR with a two-pip
  weapon, 2.5 under Holy Power, no cycling, and prompt return to normal with a zero-pip
  weapon. The earlier installed test used EEex v0.11, so it is not target acceptance.
- Do not expose `120` or `121` without fresh curation decisions and staged v1.2 runtime
  acceptance. Revisit planned `110`, `200`, and `301` only if they enter the real menu.
- Recheck whether the final SCS release supersedes `100` or `101`, and refresh the source
  repository's stale component documentation before publishing.

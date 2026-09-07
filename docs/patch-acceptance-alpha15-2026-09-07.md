# Alpha.15 / collection alpha.13 — no-SR Tempus fix

Christopher approved publishing BG Rebalance v0.3.2, updating the collection pin,
and recovering the existing no-SR test without a full restart. The default recipe
still selects 434 components; other mod pins and defaults are unchanged.

## Focused verification

- Mod owner: 281 passing tests retained from the unchanged runtime fix; packaged
  v0.3.2 also passes captured SCS/no-SR and base-game 401 installation checks.
  The public archive was independently downloaded and its size/SHA256 verified.
- Engine: 7 rollback-planner and 8 recovery-receipt tests passed; 21 production
  recipe, Chriz, Spell Revisions and coverage checks passed. The network/cache
  artifact test remains opt-in; the changed artifact was verified separately.
- Native packaging: 3 normal package contract tests passed. Recovery acceptance
  example compiles; PowerShell parse, rustfmt and diff checks passed.
- Frontend: typecheck, all 139 tests and production Vite build passed.
- Python: all 42 tooling tests passed, including recipe generation, public credits
  and updater metadata. Notices regenerated only their two lockfile digest headers.
- No new dependencies, signing keys, broad UI redesign or automatic recovery
  bypass. The incident-specific operator uses existing supervised recovery APIs,
  exact identity checks, a scoped backup, protected history, process/lock guards,
  verified rollback and exit-0 operation evidence. It never rewrites the failure.

## Actual installation acceptance

The original no-SR test was completed in place, with 429 exact rows and all five
supervised WeiDU operations returning 0. Read-only Rust acceptance passed before
publishing the separate completion receipt. Earlier mods and the prepared BG1
copy were preserved. The original failed 401 rollback was only WeiDU-reported;
the supervised top-tail rollback was independently byte-verified. These are
different claims. See [incident evidence](issues/no-sr-tempus401-2026-09-07.md).

Gameplay acceptance is not claimed. The successful game is retained for the user;
there is no new disposable full-game installation to delete. Scoped repair evidence
is retained intentionally. Stream installations and reference sources are untouched.

## Publication

App alpha.15 packaging/publication and signed updater verification are pending.
The versioned setup must be verified before changing the public update feed or
handing the verified release tuple to the website task. Existing versioned releases
remain immutable; no Discord/email announcement is sent on Christopher's behalf.

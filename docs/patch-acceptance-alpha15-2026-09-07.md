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

Radar Overlay 2.5.0.0 was installed through the normal verified add-on helper using
the cached official release. Its executable matches the add-on receipt, and the
two final WeiDU.log hashes remained unchanged after installation.

## Publication and signing

App alpha.15 was built from source `255be75aa232f3af5ad1e1969ccba313fffcccce`.
The NSIS setup is 5,499,035 bytes with SHA256
`fefb6ae39c2a1811cdf22ca5c7f9a5ed11ff26e9e9a30e4c86fed808cd9103e6`.
Its Tauri signature passed against the bundled public key. The freshly compiled
real-Tauri updater harness offered alpha.15 over alpha.14, downloaded and verified
the exact setup, and rejected tampered bytes. This is not GUI apply/restart testing.
Defender scanned the setup with remediation disabled and reported no threats;
no Authenticode publisher signature or broader security guarantee is claimed.

[The versioned public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.15)
contains setup, signature, updater metadata/checksums, public component inventory,
license and dependency notices. An anonymous download matched the exact size/hash
and passed signature verification again. Only then was the mutable alpha updater
feed replaced and fetched from its normal URL: app alpha.15, collection alpha.13,
and the expected immutable setup URL. Existing versioned assets were not changed.

A verified setup copy is at
`C:\Users\chris\Downloads\Chriz Easy BG_0.1.0-alpha.15_x64-setup.exe`.
The installed desktop app was not replaced or restarted by this task. The website
owner received the verified release tuple and deployment instructions. No Discord,
forum or email announcement was sent on Christopher's behalf.

Source build commit is pushed to public main. Hosted Windows CI
[34112232804](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/34112232804)
is still running; the last inspected state had passed setup, frontend and formatting
and was running native tests, with no failures reported. Duplicate branch CI at
the identical commit was canceled deliberately to avoid repeating the same build.

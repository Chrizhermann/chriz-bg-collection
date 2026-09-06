# App alpha.13 — EET Documents paths and quiet-install notices

## Diagnosis and scope

Two supplied community exports show the same retryable EET core failure while
creating its user settings/save directory and copying `Baldur.lua`. The EET runs
exited 2 after roughly 172 and 145 seconds; this was not a silence timeout. Their
before/after WeiDU logs retain the same two BG2 EEfixpack components.

The pinned official EET `GET_USER_DIRECTORY` helper reads the Windows Personal
registry value with `tokens=3,*` but retains only the first token, losing the rest
of a path containing spaces. A synthetic Documents path reproduces the truncation.
Both exports describe the same frozen installation, with an original app alpha.10 /
recipe alpha.11 identity. The second export does not establish that the currently
executing app was old: frozen version fields survive an app update.

Christopher chose a **new attempt using the corrected app/current recipe** instead
of manual repair of that old installation. Its frozen SoD Remix 0.6.7 pin would
otherwise retain the separate component 900 problem. No remote installation was
edited or restarted by this task.

## EET correction

The app performs a narrow Windows compatibility correction immediately before EET
core invocation. It matches the exact artifact ID/archive hash, BG2 target, TP2,
component 0 and complete recognized `EET/lib/macros.tph` hash. Unknown edits are
not overwritten. Uniform LF/CRLF variants are recognized, preserving line endings.

- Official commit: `74e91d72bca5d073fa11c1d088b90d7ff0c7105d`.
- Original LF file: `f7c6fc721f05d7bb38149f8f935153f80f6f4b0040df5ea637a5d7b955287168`.
- Corrected LF file: `1bd191c84ff6c890a5df5d8b556349ec54a996812d0d5f50998f9b86d524ef89`.

The batch retains the complete registry remainder, quotes values and output,
disables delayed expansion and preserves expanded environment variables. It does
not suppress EET errors or omit its required settings. The original download and
extracted cache remain untouched; only the direct staged file is atomically
replaced. Ancestors/leaf are checked against links/reparse points. Subsequent
preparation recognizes the already-corrected file and performs no further write.

Each affected attempt stores `eet-compatibility.json` with observed before/after
hashes and correction identity. The diagnostics allowlist includes this JSON, not
the mod payload. This also reaches an older retryable EET attempt, without claiming
to migrate that attempt's frozen recipe.

## Quiet installation

The production warning threshold is five minutes, previously 30 seconds. It remains
a notice, not a process timeout: no automatic kill, pause or stdin response occurs.
Actual output resets the runner warning and timer. The UI clears stale attention on
output, next step or successful completion without reviving paused/failed states.
The button is **Keep waiting**, and the notice explicitly says installation is
still running.

## Verification

- EET: seven focused tests plus one explicit opt-in real WeiDU 249 `--nogame` test
  passed. The real test verified cached upstream source/WeiDU hashes, applied the
  production correction twice, then exercised the actual first EET macro with only
  the registry query replaced by synthetic input. It preserved a spaced path with
  ampersand/exclamation and environment-variable expansion. All temporary fixture
  files were cleaned automatically; real Documents/registry/game paths were not written.
- New EET and warning regressions failed before their fixes, then passed.
- Frontend: TypeScript, all 111 tests, production Vite build passed.
- Python packaging/curation tooling: all 35 tests passed. The public inventory's
  expected application version was updated; recipe/default selection stays unchanged.
- Engine integration: CLI 26, diagnostics 11, orchestrator 27, runner 15 and recovery
  seven tests passed. New diagnostics regression failed before adding the evidence
  allowlist entry, then passed.
- Native command contracts 39 and normal package contracts three passed. Package
  signature acceptance is performed separately against the built signed setup.
- Formatting-only normalization of three previously unformatted files was included;
  no behavior changed there. Workspace formatting/diff checks pass.

No new dependencies, public APIs, recipe schema, curation changes, original-game
writes, full mod installation, save changes or desktop/browser control. Runtime
gameplay and native updater apply/restart are not claimed by these focused tests.
Project knowledge is recorded here, not in generated memories or global skills.
Previously tracked updater mock-harness issue 2 and broader targeted-recovery issue 3
remain separate; no repeated clean compilation or speculative repair was attempted.

## Package/publication

App version is `0.1.0-alpha.13`; bundled recipe remains `0.1.0-alpha.12`, with 434
recommended components, SoD Remix 0.6.8 and BuffBot last. The public component
inventory was compared against published alpha.12 and is identical except for the
application-version label.

Signed Windows x64 NSIS build completed. Setup size is **5,359,225 bytes**, SHA-256
`cd6002efcc0c4b1e38a946be75f0fce014a5f1f183c30cdec9f100776bab5d51`.
The rebuilt native command contracts (39) and package contracts including actual
setup signature verification (4) passed. The established headless real-Tauri
updater harness offered alpha.13 over alpha.12, downloaded/verified exact setup
bytes and rejected tampering. The source/embedded key of that existing harness
remains applicable; this is not a native updater apply/restart claim.

Windows Defender custom scan of the exact setup, with remediation disabled,
reported no threats. This is not a guarantee or an Authenticode signature.

## Published artifacts and follow-through

Source commit `9f89830` is pushed to the private collection branch. The public
[alpha.13 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.13)
is published as a Windows alpha/prerelease with setup, signature, feed, SHA256SUMS,
component inventory and notices. Old versioned releases were not modified.

An anonymous download through the versioned public feed matched the exact size/hash
above; its signature matched the feed and passed verification against the bundled
public key. The mutable `alpha/latest.json` channel was then updated and anonymously
checked: app alpha.13, recipe alpha.12, exact versioned setup URL/signature.
The previously tested updater harness also passed download/tamper acceptance on the
same setup bytes before publication. A hash-verified public copy is available at
`C:\Users\chris\Downloads\Chriz Easy BG_0.1.0-alpha.13_x64-setup.exe`.

The existing **Build interactive BG run page** task received the verified URL/hash,
versions, unchanged component inventory and precise fix/acceptance limits, with the
normal scoped `/collection` deployment request. Website deployment remains pending
its own confirmation. No Discord/forum/email was posted by this task.

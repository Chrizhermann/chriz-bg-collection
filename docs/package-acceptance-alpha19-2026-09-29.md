# September 29 combined-test installer: automated package verification

Historical candidate: superseded by the [September 30 UI-launch repair](package-acceptance-alpha19-2026-09-30.md).
Use that setup to resume Test 19; the checks below describe the older package only.

Status: **built and verified locally, ready for the user's combined install test**.
No setup, app or game was launched by this build/check. No existing installation,
save, public CEBG release, updater feed or website was changed. This is not manual
UI/gameplay acceptance and is not the older September 23 alpha.19 binary.

## Candidate identity

- App: **0.1.0-alpha.19**; bundled curated collection: **0.1.0-alpha.18**.
- Active source checkout: `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- User installer: `C:\Users\chris\Downloads\Chriz Easy BG alpha19 - Combined Test 2026-09-29-setup.exe`.
- Updater signature: the same filename with `.sig` appended.
- Setup size: **5,952,159 bytes**.
- Setup SHA-256: `4f987dd21bea492d3bea943f3522567b26530b3b440b88a3e1e715633f481d00`.
- Signature-file SHA-256: `d06f1cc087a480b223b8ceb138132c0e0a369c5d058874e31faf905e037f2816`.
- Extracted app SHA-256: `d51274ae4c2ef89d21926814ab4c75e48a5080882bfa4ae973281a2440c49080`.

The updater signature verifies against the bundled public key. It is not Windows
Authenticode publisher signing. Downloads copies were created without overwriting
another file and their hashes were rechecked after the job completed.

## Checks completed

Continuation job `a820c812-eac4-479e-bdad-68f1fb19645c` ran from
2026-09-29 12:34:07 UTC to 12:39:35 UTC and exited **0**. Saved `status.json`,
`stdout.log` and `stderr.log` were inspected by the lead and an independent
read-only audit. Evidence is in `target/work/sep29-next-release-package-final`;
the command sequence is `target/work/sep29-next-release-continue-build.ps1`.

- Rust workspace: **832 passed, 0 failed, 13 existing opt-in tests ignored**.
  Includes 42 native unit, 42 native command-contract and 745 engine tests;
  three package tests make up the remaining passed cases.
- Formatting and Clippy with warnings denied: pass.
- Frontend: typecheck, **353 tests across 30 files**, and production build pass.
- Normal Windows x64 NSIS build: pass; updater signature generated.
- Post-build package suite: **4 passed**, including the previously ignored
  signature case. Three cases repeat the workspace's package checks.
- Offline extraction: **221 bundled resources match source**, no extra recipe
  files, and the packaged executable matches the compiled executable after only
  the exact Tauri NSIS bundle-marker normalization performed in memory.
- Bundled curated `release.json` explicitly contains `0.1.0-alpha.18`.
- Evandra recipe-default contract passes again in the package build hook.
- Earlier unchanged Python suite: **129 total, 128 passed, one existing skip**,
  in the original job `630d3594-bbb5-4f5e-84ba-7d03240fe9f0`.

The original job stopped before packaging on stale exact UI test expectations.
Both affected test lists were corrected for the six approved providers and helper
640, retaining strict assertions. Their 11 tests and the full continuation pass.
No feature, safety rule or validation was disabled. Failure-looking event-renderer
output in stderr is synthetic fixture text, not a failed installation or test.

After the signature case ran, 12 distinct opt-in cases remain unexercised by this
combined job: real updater download, real BuffBot archives, network patch adapter,
real WeiDU patch transaction, EET real-source macro, configured local-game probe,
real-artifact verification, and five real-WeiDU integration cases. Separate owner
and artifact-intake checks remain their own evidence, not implied coverage here.

## Included scope and next step

The package includes approved owner refreshes (Modpack alpha.8, BG Rebalance 0.7.0,
Artisan 4.81a-dev.85928f9-chriz.1, Bardic 2.9c-balance.6, SoD Remix 0.6.12 and
BuffBot 1.9.0), new defaults, optional visuals/shifted dialogue, six alternative UI
choices and advanced manual breakpoints. See the [release notes](release-notes-alpha19.md).

Next: **one fresh installation and the [short manual check](next-release-smoke-check.md)**.
Do not launch the user's setup/game or perform desktop actions without permission.
Do not turn this into another kit/UI matrix or repeat already accepted BuffBot
Apply/Undo. Public CEBG publication remains a separate decision after user review.

The shared source worktree still contains uncommitted approved and pre-existing
work. This package record is not a claim that a clean release commit/tag exists;
review and capture the intended source before public publication.

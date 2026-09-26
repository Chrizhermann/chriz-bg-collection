# Alpha.19 local test package — September 22

App **0.1.0-alpha.19**, bundled collection **0.1.0-alpha.17**.
Built from the current dirty working tree on `codex/installer-v0-real-alpha`
(HEAD `32ab76df7ed52054c3e3534a7fee5c3bbff2daa1` plus uncommitted work).
This is a local acceptance candidate, not a published release or immutable
source-to-release record. It includes the existing-install comparison,
BuffBot update route and September 22 interval-save fix/default controls.

## September 23 feedback/naming review package — verified, ready for user review

September 26 source follow-up: the Updates feedback banner now resolves the same
default installation as the game selector. This fixes missing patch/Radar progress
and results before the user manually changes the selection. The September 23
binary below predates this correction; its hashes and acceptance remain historical
evidence, not verification of the corrected source.

Source now includes compact Updates, progress/completion/Undo feedback, unique
display names and Rename, free default destinations, clearer folder failures and
the completed Customize frontend. Alternative UI provider recipes remain outside
this package. No existing installation or save was changed for this source work.

Frontend: 323 tests/typecheck/web build passed. Python: 102 tests with one existing
opt-in skip. Native: 797 tests passed, 13 opt-in tests ignored; formatting/Clippy
passed. A six-test native default-folder follow-up landed after that full run.
The final job passed all **38 app-library tests** (including eight naming tests),
**40 command contracts**, Clippy/formatting, **323 frontend tests/typecheck/build**,
then the signed normal (not diagnostic) package and **four package/signature tests**:
`target/work/sep23-feedback-review-package-final`, job
`d75b1f80-08e3-4958-9b3d-bc28168d0a12`, exit **0** at **2026-09-22T15:13:36Z**.
Saved status and both logs were read; no job was rerun on notification.
The preceding attempt stopped before packaging on two mixed-separator Windows
test expectations. Those are corrected with native path joins; destination
selection behavior was already correct.

- Setup: **5,891,617 bytes**, SHA-256
  `9cbae97c0f51505adc66cd36d897d96933982287c7487377c3d67e88b9528e6a`.
- Signature SHA-256:
  `5f1b888daca98ba304b654285fc7baf7062826af769fc7df74227eb875d3ba4d`.
- Extracted app SHA-256:
  `b8b9addf15764d40823837bb4f63f8fce4af10baa5cb810b2bf25a96abadee5a`.
- Offline NSIS extraction matched all **191 declared resource files** to current
  source hashes; no unexpected manifest/recipe files were found.
- The extracted app differs from the raw build by exactly three bytes at offset
  18,910,154: Tauri's bundle marker changes `UNK` to `NSS`. This is documented in
  locked `tauri-utils-2.9.3/src/platform.rs` and logged by the packaging command.
  After accounting for that exact marker in memory, all other bytes match. No
  executable was edited or launched by verification.
- Evidence helper: `target/work/verify-feedback-package-sep23.ps1`; verified
  extraction remains in the final job's `extracted/` directory.
- New, non-overwriting copies of the setup and signature were hash-verified at
  `C:\Users\chris\Downloads\Chriz Easy BG alpha19 - Review 2026-09-23-setup.exe`
  and its adjacent `.sig`. Older Downloads packages were not replaced or removed.

**Ready for Christopher's finishing UI review.** The package was not installed,
launched or published. It retains app alpha.19 / collection alpha.17, so this is a
manual local review package, not a newly advertised automatic update.
The earlier BuffBot acceptance below does not substitute for this package's checks
or Christopher's finishing UI review. No automatic Apply/Undo is requested.

## Latest acceptance — user-applied update, gameplay and Undo passed

Christopher installed the later opt-in lookup diagnostic package himself
(`Chriz Easy BG alpha19 - Lookup Diagnostics-setup.exe`; SHA-256
`6a260c146873f7d9dd9c25edcd60d46eb28590b1be032a90d2fe681a232cd73b`).
See [current handover](handover.md) for its packaging checks and the proven Codex
AppData redirection that hid the repaired No-SR completion record. Christopher
restored the record with the narrowly scoped user-run helper, then confirmed the
normal app recognized it and offered the BuffBot update.

He applied BuffBot **1.8.3-alpha to 1.8.4-alpha** to
`C:\Users\chris\Games\ChrizEasyBG-No-SR-Test`, transaction
`01790071515151257500`, and reported **"its fine, it works"** after the requested
existing-save/gameplay check. This narrow update's Apply and user gameplay checks
are accepted; do not repeat gameplay testing merely to restate that result.

Read-only verification: 96 updated-file hashes, 96 affected-backup hashes and 59
unchanged guards matched; original receipt and dialog.tlk unchanged; all 400 other
active component rows unchanged. Full-game backup path/type/size inventory matched
(159,300 files, 11,898,950,716 bytes; not a full content rehash). All 12 profile files
matched their backup byte-for-byte. Actual file handles resolved to the intended
normal AppData/game paths, not the Codex package view.

Christopher then used Undo. At 20:12:39 +09:00 it restored **1.8.3-alpha**. Read-only
checks confirmed all 155 original guarded game-file hashes and 96 affected-backup
hashes, the entire original WeiDU.log, original receipt and dialog.tlk. The restored
marker matches the prepared journal hash. The exact Apply → user gameplay → Undo
flow is accepted. No agent clicked Undo or modified game files. Do not reapply
automatically; keep the recovery backups for now.

**Still pending:** updated UI packaging/acceptance and authorized
publication. The compact selector, simplified results, useful progress and
conspicuous completion feedback are now in source; see the current build above.
User drives the current
app and game; no unsolicited UI control or game modification. No broader mod-update
or release acceptance follows from this one successful update.

## Earlier corrected candidate — morning build

Job `f78462ce-7d67-4d7d-8fb3-20dc1d3aef79` exited **0** at
`2026-09-21T22:16:56Z` (September 22, 07:16 in Korea).
Verified its saved status and both logs; it was not rerun. Includes the corrected
BuffBot source fingerprints and distinct read-only preview error wording.

- 24 update-module tests, including extraction and validation of both exact
  published BuffBot archives, passed. No network or game execution in that test.
- Six native bridge tests, 211 frontend tests, typecheck/assets, native Clippy
  with warnings denied, formatting, and four package/signature tests passed.
- Setup: **5,782,529 bytes**, SHA-256
  `0cb108545b4b63564b41766578be0b4d7c70c52692f09e83c31b13cc8b266a83`.
- Signature SHA-256:
  `4b79f92d02fcfe2bd9339f575ff2c528efa664a34ce6591e2f34dc68844139f4`.
- Verified copies replace the earlier candidate in Downloads:
  `C:\Users\chris\Downloads\Chriz Easy BG_0.1.0-alpha.19_x64-setup.exe`
  and its adjacent `.sig`. Build output uses the same bundle path below.

At this build's completion it was **not installed or published**; the corrected
source's read-only eligibility check on No-SR passed. Later user-driven installation
and acceptance are recorded above. The stream installation remains untouched.

## Earlier candidate — superseded

- Build job `1b37d06d-079f-43b1-8044-b36227ffca79` exited **0** at
  `2026-09-21T20:52:45Z` (September 22 in Korea).
- Frontend typecheck, **211 tests / 21 files**, and production asset build passed.
- Locked Windows x64 release compilation and NSIS packaging succeeded.
- **Four package tests** passed, including cryptographic verification of this
  setup's Tauri updater signature against the bundled public key.
- Setup is **5,780,755 bytes**, SHA-256
  `5bded2f25edd25efc8cd8d9b7742b1bcfc7183c9524ae737e35341abff1f0e7a`.
- Signature SHA-256:
  `a4228c494273cc6848d56e7d1ec725e333aa5edc4cdd1decd9e5063ee0b50de9`.

Package:
`target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.19_x64-setup.exe`
(matching `.exe.sig` beside it). Job status and logs:
`target/work/alpha19-update-candidate-sep22/`.

## Still pending

**Overnight correction:** the first package above failed the real BuffBot preview
because both expected source-tree fingerprints were wrong (archive hashes and
installed files were correct). Source and regression tests are fixed; this first
package is not an accepted update candidate. The corrected candidate passed the
real-package fingerprint test before packaging. See the
[failure and correction](existing-install-upgrades-2026-09-21.md#september-22-overnight-authorization-and-preview-blocker).

The builds themselves did not launch setup/app/game. The former overnight UI
authorization is historical: Christopher drives the current acceptance himself.
Apply, gameplay and Undo now passed as recorded above. A new antivirus scan
and native app-updater apply/restart have not been verified. Tauri updater signing
is not Windows Authenticode publisher signing. Public alpha.18 assets and the live
update feed were not changed; no publication is authorized by these local builds.

The existing test target is
`C:\Users\chris\Games\ChrizEasyBG-No-SR-Test`. Its completed receipt matches the
registry. BuffBot 1.8.3 components 1 then 0 were the final entries before this test;
they were updated to 1.8.4 and are now restored to 1.8.3 after the successful Undo.
The [update acceptance flow](existing-install-upgrades-2026-09-21.md) has completed
for this exact adapter; this does not prove arbitrary mod updates safe.

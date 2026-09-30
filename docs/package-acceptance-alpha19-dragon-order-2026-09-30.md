# September 30 dragon/order repair — local package verification

**Later security incident:** on September 30 at 17:18 local, Defender quarantined
the user's installed CEBG executable under `Behavior:Win32/DefenseEvasion.A!ml`.
The successful package checks below do not resolve that runtime detection or
prove a false positive. Investigation remains open; see the live handover.
The completed game and its separately installed Radar files remain present.

Status: **signed, verified and copied to Downloads; not installed or publicly released**.
No app setup or game installation was launched. Public CEBG releases and feeds
stay unchanged.

## Exact deliverable

- Setup: `C:\Users\chris\Downloads\Chriz Easy BG alpha19 - Dragon and Order Fix 2026-09-30-setup.exe`.
- Signature: the same path with `.sig` appended.
- Size: **6,025,583 bytes**.
- Setup SHA-256: `3d94c594470486030e3792f94c22ef42bc7cc68e9b2894969fb3c653ace8c6f9`.
- Signature SHA-256: `204b6f9736df3bf815db3af100459cadfd0b85df6491188850f27cb93103815f`.
- Packaged app SHA-256: `0b0ea4e0217d3c202485a3fe738adb6c470a7e6f1470bd4de7518cb369ae063e`.

This is Tauri updater signing with the existing configured key, not Windows
Authenticode. The older Downloads setups were not overwritten.
An independent read-only spot-check remeasured both Downloads hashes and confirmed
both extracted profiles use Rebalance 0.7.1 and 302-last order. The curated release
is alpha.18; the base fallback's existing alpha.2 label is unchanged and matches
source. No claim is made that both internal release files say alpha.18.

## Included corrections

- BG Rebalance v0.7.1 supports valid INI and GLOBAL SCS difficulty scripts.
  The mod is published and freshly verified: 1,734,037 bytes, SHA-256
  `ba53f02e1b5542534412331dc2f33d78c4b3a19e2dd3dee60ef6a9755ef170ab`.
  Dragon tuning and the rest of SCS's combat AI are preserved.
- Both recipes put Flail 302 in the TP2's actual declaration order, after 408.
  The generator and archive-backed source-order check protect this choice.
- Partial-install diagnostics distinguish missing components from order mismatch
  when the old history is demonstrably unchanged. Reconciliation stays strict.
- Pending Resume/Retry/Continue actions cannot start twice; resumed phases and
  download/install progress are distinguished.

## Verified before packaging

- Owner mod suite: **395 tests, no skips**, exit 0; includes the 52 focused dragon
  checks and real captured Test19 inputs in disposable synthetic games.
- Collection job `6846363d-d5ff-4337-ab90-4b36a6aace25`: exit 0. Python 137 cases
  (one existing skip); Rust 850 passes (14 existing opt-in ignores); frontend 380
  tests in 32 files; TypeScript, Vite, formatting and warnings-denied Clippy pass.
- Both public-alpha profiles validate with zero findings.
- Freshly downloaded release passes both native-order checks. The separate
  post-publication metadata/checker suite passes all 11 cases.
- Ordinary workspace tests do not claim optional network/live integration tests
  passed. Packaging will separately enable signature verification for its new setup.

## Build job and boundary

Job `794e24e5-6a8d-42dc-8e30-3078d3bcaa26` saves results in
`target/work/sep30-dragon-order-package`. It exited **1 after a successful app
and NSIS build**, before signing: the Windows PowerShell helper could not resolve
`Get-FileHash`. That command is available in a fresh shell, so this does not
establish that it is missing from the machine. The completed job is not rerun.

`target/work/sep30-dragon-order-sign-existing.ps1` continues from the exact built
setup with PowerShell 7 and an explicitly loaded Utility module. It checks the
known setup hash before signing, then verifies the signature and bundled bytes.
The future build wrapper now preflights this dependency before compiling. No
validation is bypassed and no old signature is accepted for the new setup.

The continuation **exited 0**: all four package tests passed, including signature
verification against the configured updater public key. Offline extraction
matched **223 bundled resource files** with no extra recipe files. The packaged
executable matches the compiled app after only the reviewed NSIS marker
normalization in memory. Setup bytes are unchanged across signing, and both
Downloads copies match their built originals.

App remains **0.1.0-alpha.19**, bundled collection **0.1.0-alpha.18**. This new local
setup supersedes the earlier UI Resume Fix setup for the next fresh test. It was
not sent to the public update feed.

Test19 remains stopped and unchanged. Final read-only WeiDU.log SHA-256:
`ac62ca57362662983ad81ec13eb3ebec5fe1536b40824716dd1d7109ab8fe9d0`.
Neither this app update nor the mod publication makes ordinary Resume safe.
Recovery requires a separately verified order correction and rollback rehearsal;
the old Tempus recovery worker cannot simply be retargeted. No live game, save,
frozen recipe or failure history was modified.

## Selection Fix addendum — latest local setup

The user subsequently chose to delete Test19 and start fresh, not pursue recovery.
Changing Customize options exposed a frontend focus bug: IDs containing colons
were interpolated into a CSS selector after the selected choice had been saved.
The correction matches literal IDs within the app root without changing recipe
IDs, selection semantics, saved choices or installation safeguards.

The latest verified local installer supersedes the Dragon and Order Fix setup:

- Setup: `C:\Users\chris\Downloads\Chriz Easy BG alpha19 - Selection Fix 2026-09-30-setup.exe`.
- Signature: the same path with `.sig` appended.
- Size: **6,027,850 bytes**.
- Setup SHA-256: `614e71cfc4416dc0a71e2bb2eb4616b0f96a33ba889796ab8cc1f628f569620f`.
- Signature SHA-256: `68529539d413e4111d8422968c60ae98a861b6f2bbf0c6f5300857446be9731c`.
- Packaged app SHA-256: `3e50d7d3c6c155a5e505e48d19df832ada61b69538ca815693d2632e33581243`.

Focused regression: 24 passing tests. Full frontend check: **382 tests in 33
files**, TypeScript and Vite passed. Existing option-field tests now check
restored focus and absence of an error banner, rather than only saved values.
The new tests also cover the reported component ID, dots/colons, toggling both
ways, persistence and an unrelated duplicate ID outside the app root. The
original focus assertions failed before the fix. A punctuation stress fixture
outside the draft's valid identifier grammar was corrected to a valid ID; draft
validation was not loosened. A bounded read-only review found no concrete defect.

Packaging job `a08a616a-fe4c-4d4b-83a4-1899806f118b` completed with **exit 0**;
status and logs are in `target/work/sep30-selection-focus-package/`. It used
explicit PowerShell 7, rechecked native component order in both recipes and
passed the Evandra default contract before building. Four package tests passed,
including the new setup's signature against the configured public key. Offline
extraction matched all **223 resource files** and the executable after the exact
in-memory NSIS marker normalization. Both Downloads hashes were independently
remeasured after completion and matched the job output.

This follow-up changes only the frontend behavior. The earlier backend/mod suite
results above are retained as prior evidence, not claimed as rerun for this
package. App alpha.19 and curated collection alpha.18 remain unchanged. All older
Downloads setups were preserved. No app/setup/game launch, game install, desktop
interaction or public CEBG release/feed update was performed. User-side testing
of this replacement is pending.

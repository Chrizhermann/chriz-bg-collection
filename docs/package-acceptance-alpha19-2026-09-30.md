# September 30 UI-launch repair: verified local CEBG package

Status: **signed, verified and copied to Downloads; not installed or publicly released**.
This replaces the September 29 candidate for further testing. No CEBG app/setup
or live game was launched by this packaging work. Test 19 remains stopped for
Christopher to retry after updating the app. Its game files, saves and ledger
were not changed by this source/build repair.

## Exact candidate

- App: **0.1.0-alpha.19**; bundled curated collection: **0.1.0-alpha.18**.
- Source: `%USERPROFILE%\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- Setup: `%USERPROFILE%\Downloads\Chriz Easy BG alpha19 - UI Resume Fix 2026-09-30-setup.exe`.
- Signature: the same path with `.sig` appended.
- Setup size: **6,023,677 bytes**.
- Setup SHA-256: `b17421335809b1f5b9c26a4475c1194875bbb86e3f6669860fc2e9b85bb19738`.
- Signature-file SHA-256: `1b03ace10f28940504e3818309271c60cce0ea037fcc53dc103f713be26e0c04`.
- Packaged app SHA-256: `9a40dc45be27b019985fc475dcd1cc9820b68a26daa809c78dee99c5d7319e0b`.

The public updater key embedded in the app verifies this signature. This is Tauri
updater signing, **not Windows Authenticode**. Both Downloads hashes were checked
independently after the copy; the previous candidate was not overwritten.

## Verification and accurately recorded failures

1. Job `4e19b950-1727-4886-8d44-69fb261498a3`, results in
   `target/work/sep30-ui-launch-recovery-package`, exited **101** before packaging.
   Python passed (129 cases, one existing skip), as did the Evandra-default gate
   and formatting. The Rust workspace had 834 passes, 13 failures and 14 opt-in
   ignores. All failures came from production-command fixtures still copying
   the generic authoring fixture's intentional `explicit-tp2` mode. Only the two
   disposable fixture builders were corrected to supported setup-name mode;
   shared authoring fixtures, production validation and existing assertions stay
   unchanged. Complete affected suites then passed: **30 CLI + 42 native command**.
   Thus all 847 non-ignored workspace cases pass across the full run and focused
   corrections; this is not a claim that the original failed job succeeded.
2. Continuation `04767c1f-361f-4b20-b6fe-dff90cf32fc8`, results in
   `target/work/sep30-ui-launch-recovery-package-final`, passed formatting, Clippy
   with warnings denied, both public-alpha profile validations (**zero findings**),
   TypeScript, **358 frontend tests**, Vite, Rust release compilation and NSIS
   generation. It then **timed out at the signing password prompt**. Windows
   PowerShell removed the empty password environment variable. No game install
   ran, and the newly built setup was not offered with its stale old signature.
3. `target/work/sep30-ui-launch-sign-existing.ps1` signed that exact existing setup
   with the existing external key, preserving an explicitly empty password via
   Node argv. It exited **0**. No rebuild/key rotation was needed. The setup hash
   remained unchanged across signing. All **four package tests** pass, including
   verification of the new signature against the bundled public key.
4. Offline package extraction matched **all 221 bundled resources**, with no extra
   recipe files. The packaged executable matches the compiled executable after
   only the known Tauri NSIS marker normalization in memory. All six packaged
   alternate-UI descriptors were separately checked for `setup-name`; curated
   `release.json` is alpha.18. The verifier copied the setup and signature without
   overwriting an existing Downloads file.

The focused exact-failure recovery matrix, changed/shadowed TP2 rejection and
diagnostics evidence checks pass. A separately enabled real WeiDU 249 integration
test installed a synthetic component using Infinity's exact folder/setup naming
in a disposable fake game. It verifies discovery/launch mechanics, not the full
Infinity mod or gameplay. Other ignored live/network tests were not silently
treated as passed.

## Included repair and next action

All six new UI providers now use supported setup-name launch. Public validation
and new-install preflight reject unsupported launch settings before staging.
The exact old Infinity 1.28/WeiDU 249 frozen recipe has a narrow compatibility
correction; it does not enable arbitrary explicit-TP2 execution or rewrite the
recipe/ledger. Retry requires the exact original pre-launch failure, first
attempt, before.log-only evidence and an unchanged current WeiDU.log. Launch
additionally verifies the pinned TP2 and rejects shadowing discovery candidates.
The correction is recorded in new attempt evidence and diagnostic exports.

Also included: layout/policy-aware extraction-cache identity, leaving old caches
untouched; readable failure messages with collapsed technical details; the
approved Wing Buffet default that postdated the September 29 package.

Christopher should close CEBG, run the exact setup above, reopen **My installs →
Test 19**, and choose **Retry failed step**. Do not create a replacement game
installation or edit its history. Actual user-install resume and UI gameplay
remain unverified until he reports the result. The version label stays Alpha 19;
use the filename above to distinguish this corrected local build.

No public release, updater feed, website, signing-key contents or Git publication
was changed. The shared checkout still includes previously approved and unrelated
uncommitted work; this document does not imply a clean release commit or tag.

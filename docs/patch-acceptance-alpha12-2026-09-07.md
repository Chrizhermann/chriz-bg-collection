# App alpha.12 — automatic installation-folder creation

## Confirmed cause and scope

A community user could only begin installation after manually creating the default
user `Games` folder. A synthetic engine regression reproduced the exact cause:
`claim_new_root` required the immediate parent to exist and failed before preflight
with Windows error 2. Native destination inspection already allowed prospective
nested paths and did not create them, so inspection and startup disagreed.

Startup now creates missing ordinary ancestor directories from the nearest existing
ancestor, checks the directory chain, and exclusively creates the final managed root.
Inspection remains read-only. Existing occupied roots, reparse ancestors, source/cache
overlap, lock and other safety boundaries remain in force. This does not grant an
unsafe resume of a prior partially installed game.

The main screen and location screen explain: enter a new path or browse; missing
folders are created at installation time. The visible Change label is now Browse;
the accessible location label and edit controls remain available after a rejected
path. The new help is associated with the input alongside any error description.
No new layout system, API or dependency is introduced.

## Verification

- New frontend regression failed before the copy change; then typecheck and all
  110 frontend tests passed. It checks an initially rejected location can be edited
  into a nested new path, restoring install readiness, and checks accessible help.
- Python tooling tests: 35 passed. App markers are alpha.12, while the entire bundled
  recipe stays alpha.12. Public inventory changes only its application-version label;
  recommended component choices/order/source pins remain unchanged.
- Backend: 27 orchestrator tests passed, including real temporary multi-level folder
  creation, required Windows symlink rejection (not silently skipped), blocking-file
  rejection and existing nonempty-root preservation. All 19 preflight tests and all
  39 native command-contract tests passed; three normal package contracts passed.
- Signed Windows x64 NSIS build passed. Setup length is 5,373,568 bytes, SHA-256
  `9fc66528e0b0946fac975bebda3805f74401bed8267cb680c5cf6c03e847dc4b`.
  Fresh package signature verification passed. The existing working headless real-Tauri
  updater harness offered alpha.12 over alpha.11, verified the exact setup download,
  and rejected tampered bytes. Native apply/restart is not claimed.
- Windows Defender custom setup-file scan with remediation disabled reported no
  threats; this does not guarantee safety or provide Authenticode signing.

No game/source/save was changed, no full mod installation was started, and no native
window or browser was controlled. Filesystem tests use temporary fixtures only.
The Windows folder-picker visual interaction was not manually exercised; the editable
path and native validation/startup behavior have automated coverage.

Documentation/release notes are updated in the repo, not generated memory. Security
review focuses on directory input/creation, direct-ancestor checks and exclusive final
root creation. Existing updater harness issue 2 and targeted-recovery issue 3 remain
separate follow-ups. The website owner is holding one combined publication update
until the verified alpha.12 binary is available.

## Publication

Source commit `ce3c290` is pushed to the private collection branch. The public
[alpha.12 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.12)
is published (not draft, explicitly prerelease), including setup, matching signature,
feed, checksums, component inventory and notices. Anonymous public setup download
matches the length/hash above, its signature matches the versioned feed, and the
real updater download/tamper test passed again on those public bytes.

The mutable alpha feed now offers app alpha.12 / recipe alpha.12 with the correct
versioned URL/signature, confirmed anonymously. Immutable alpha.11 assets remain
untouched. A verified alpha.12 setup copy is in Christopher's Downloads folder.
The website owner has the verified URL/hash and is proceeding with the combined page
update; its final deployment evidence is a separate pending handoff.

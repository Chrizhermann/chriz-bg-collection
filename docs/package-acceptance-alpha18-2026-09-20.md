# Alpha.18 package acceptance — September 20

App **0.1.0-alpha.18**, bundled collection **0.1.0-alpha.16**. Product sources
are at `66b130d`; the subsequent packaging commit refreshes only the generated
dependency-notice lockfile digests and release documentation.

## Package and checks

- Locked Windows x64 release build and NSIS packaging succeeded from PowerShell.
  The package was rebuilt after regenerating the dependency-notice digests;
  dependency versions and license text did not change.
- Setup: `Chriz.Easy.BG_0.1.0-alpha.18_x64-setup.exe`, **5,577,328 bytes**.
- SHA-256: `ab3dd13ae9b7656dde4e996b6d3981e9036f2bb22a4156916ef58772765c55dc`.
- Matching Tauri signature SHA-256:
  `54c218c815798f76a4148d43d2853d6469f45dab4ee5d6f51b29a0b80c816a3f`.
- Feed SHA-256: `4d14bec81068fcedfdd36b99d608677ebfa053eb177dd12bf5cc077965a24645`.
- TypeScript checks and all **157 app tests / 18 files** passed.
- **13 packaging/expanded-recipe Python tests** passed, in addition to the
  preceding 26 focused recipe tests recorded in the candidate report.
- Three package-configuration checks and the exact setup's bundled-public-key
  signature check passed.
- The real Tauri updater loopback test passed: version discovery, downloading
  exact setup bytes, signature verification and rejection of tampered bytes.
  Existing compiled acceptance harnesses were used because their source/public-key
  contract is unchanged; this avoids the separately documented fresh-test-harness
  startup issue. This is not updater install/apply/restart acceptance.
- Generated dependency notices match the locked, checksum-verified package texts.
- Public component inventory: **448 default components, 50 runs, 39 manifest
  entries and 35 distinct project homepages**. These are project links, not an
  exhaustive count of individual mod authors.

No full game installation was repeated. Christopher's successful alpha.17 test
remains the preceding full-install evidence, not a playtest of Modpack alpha.7.
No native app/update execution, new managed-install deletion test, or live-game
modification was performed. The setup has a Tauri updater signature, not a Windows
Authenticode publisher certificate.

Public asset/feed verification and website deployment are recorded below when
completed. Versioned assets must remain immutable.

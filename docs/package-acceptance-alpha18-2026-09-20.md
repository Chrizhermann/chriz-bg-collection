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

## Publication

- [Public alpha.18 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.18)
  published as a prerelease, following the existing CEBG alpha convention.
- Source packaging commit `64cb6e7f3ef25a764cd1c7906bf8354687d7eab8` was pushed to
  `codex/installer-v0-real-alpha` before publication. The Windows source CI run
  was still in progress; no hosted-CI success is claimed here.
- All seven versioned public assets were anonymously downloaded and matched the
  local release hashes: setup, signature, feed, checksums, component inventory,
  application license and third-party notices.
- The downloaded setup passed the bundled-public-key signature test.
- Windows Defender's targeted setup scan completed and reported no threats.
  This is a scan result, not a guarantee or publisher signing certificate.
- The `alpha/latest.json` channel asset was replaced only after asset checks.
  GitHub's stored digest and a cache-busted anonymous download match the new
  feed's exact SHA-256. The first ordinary URL read returned cached alpha.15;
  a subsequent anonymous ordinary-URL download returned alpha.18/recipe alpha.16
  and matched the feed byte-for-byte. Notification visibility may briefly lag
  across other CDN caches.
- The verified alpha.18 setup was copied to the user's Downloads folder. No
  older setup or game folder was deleted or overwritten.
- The website owner task received the verified URLs, hashes and public inventory
  with authorization to deploy; its final deployment result is recorded separately.

Versioned assets must remain immutable. Native updater apply/restart remains a
user acceptance check, not something proven by the download/signature tests.

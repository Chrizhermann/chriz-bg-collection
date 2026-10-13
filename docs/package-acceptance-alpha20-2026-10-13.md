# Alpha.20 package verification — October 13

App **0.1.0-alpha.20**, bundled collection **0.1.0-alpha.20**. Build source:
`3f3a44bed054ea3eb75597b21477bcab9c0408c9`. Later publication documentation does
not alter the verified package.

## Exact package

- Setup: `Chriz.Easy.BG_0.1.0-alpha.20_x64-setup.exe`, **6,091,998 bytes**.
- Setup SHA256: `ad71cdb87e1ef0ac654201e0109222196a760cab71c4045fab2eb4caa089ba9f`.
- Signature SHA256: `8ffca74d15a45d43af4637ecf81c02d8ed9bb241a23ef8120445aafe3d89b737`.
- Feed SHA256: `1a33c9e42e480f9f031b3a204ac5ce683199289e8be3493b5d7153b1f985bb8d`.
- Extracted app SHA256: `1d0ab8394c09384676a00e4639e6c18c53e0e538451a9381e4b53ccf760124e5`.
- All **239 bundled resources** matched source bytes, with no unexpected recipe
  resource. The packaged app differed from the compiler output only by Tauri's
  expected NSIS bundle marker.
- A verified copy was saved to the owner's Downloads folder for review.

## Automated checks

The final packaging run completed with exit **0** on a clean checkout of the build
source. Its log and source snapshot are under `target/work`
(`oct13-release-candidate.log`, `oct13-release-candidate-final`).

- Frontend typecheck, **413 tests / 35 files**, and production build passed.
- Native workspace: **906 passed**, zero failures; **14 explicitly opt-in tests
  ignored** in the normal suite. No new ignores were added.
- Formatting, Clippy with warnings denied, dependency notices and **161 Python
  tests** passed (one existing optional test skipped).
- The four October 13 owner releases (Modpack 0.2.0-alpha.11, SoD Remix 0.6.13,
  BG Rebalance 0.8.2, Bardic Wonders 2.9c-balance.7) passed fresh public download,
  extraction and native TP2-order checks: Modpack, SoD and Rebalance in both
  recipes, Bardic Wonders in the curated recipe. See
  `manifest/evidence/release-intake-20261013.toml`.
- Package/signature checks passed, including the opt-in check against this exact
  setup and the bundled updater public key.
- The opt-in real Tauri updater download check passed from alpha.19 to alpha.20:
  version discovery, exact signed bytes accepted, deliberately altered bytes
  rejected. It did not install the update or test updater restart.
- Public component inventory: **469 default components, 53 runs, 39 mod entries**
  (alpha.19: 462 / 51 / 39). Added: bard ceilings (Rebalance 420/421, Bardic
  Wonders 3010), Fade/Evandra catch-up (Modpack 237/238), reader-level scrolls
  (Modpack 430, replacing the old 430), Degrodel (Modpack 660) and SoD Remix
  177/198/285. Now optional: Emotion exclusivity (Rebalance 301) and EET's desktop
  shortcut (EET 100).

The setup was signed with the existing updater key, passed to the signer by path
only. No key was created, copied or rotated.

## Defender and runtime boundaries

Both the exact final setup and its extracted contents returned **no threats**,
exit 0, with security intelligence **1.459.691.0**. Antivirus, real-time,
behavior-monitor and downloaded-file protection remained enabled. No exclusions
or security preferences were changed. Diagnostic `-DisableRemediation` custom
scans report findings without scan-triggered remediation; real-time protection
was not disabled.

No setup, app or game was launched by this packaging pass. The LeUI profile fix
and Challenge Mode are covered by automated tests; this exact binary was not
launched. Full new-install gameplay, campaign XP banking in play, and updater
apply/restart remain untested. No stream game or saved character was edited by
the release work.

These are bounded clean-scan results, not a Microsoft guarantee or a Windows
Authenticode publisher certificate. The owner authorized publication on October 13.

## Publication

[Alpha.20](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.20)
is published as a prerelease (13 October 2026, 13:31 UTC), following the existing
CEBG alpha convention. Source through `3f3a44b` was pushed to
`codex/installer-v0-real-alpha` first. All seven versioned public assets were
anonymously downloaded and matched local hashes: setup, signature, feed,
checksums, component inventory, license and notices. The license and notices are
the committed file bytes. The downloaded setup also passed the bundled-public-key
signature test.

The alpha channel was updated only after those checks; the previous alpha.19 feed
was saved locally for rollback. Both a cache-busted and an ordinary anonymous
download of the channel feed match the exact feed hash above and announce
app/recipe alpha.20.

After the owner approved it, the [collection website](https://bg.chrizfader.org/collection)
was updated and verified live. Website PR 28 merged as `ef0ad67` (content
`cfddf70`); its 115 site tests, static build, PR CI and merged-master CI (run
`38409529541`) passed. The live content file matches the merged commit byte for
byte and shows app/recipe alpha.20 with the setup hash above. The collection,
rules, voicepacks, logs and links routes return HTTP 200 with security headers;
the root redirects. The setup downloaded through the live website's link matches
6,091,998 bytes and the setup hash above. A local preview showed no console errors
and no horizontal overflow at 375 px width; a full desktop/mobile visual review was
not performed. Run rules and other non-collection content are unchanged.
Versioned assets are immutable.

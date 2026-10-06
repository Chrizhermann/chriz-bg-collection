# Alpha.19 package verification — October 7

App **0.1.0-alpha.19**, bundled collection **0.1.0-alpha.19**. Build source:
`f07e77a5f3650786bca0c866d41ec38f8508cc30`. Later publication documentation does
not alter the verified package. Existing unrelated handover/planning edits were
preserved and excluded from these source commits.

## Exact package

- Setup: `Chriz.Easy.BG_0.1.0-alpha.19_x64-setup.exe`, **6,017,614 bytes**.
- Setup SHA256: `ec89ebd064f331db77d1c2666731eca81e4224a30e0e48780f1db28e18a35965`.
- Signature SHA256: `86b3a6ddc3207bb0f8ce6f2da3874371bc0d73d2f7ae8ead376b5d4f7109d590`.
- Feed SHA256: `2d4af6b3111f8b0f404f3cf758e7888de773a73210e0fb61ba1de7d97eaeb622`.
- Extracted app SHA256: `a659cd5073cb54770ef8066ae26d16b459b4099cb1de03db17a40880fd6cd23a`.
- All **231 bundled resources** matched source bytes. The packaged app differed
  from the compiler output only by Tauri's expected NSIS bundle marker.
- Verified copy: `C:\Users\chris\Downloads\Chriz Easy BG alpha19 - Release 2026-10-07-setup.exe`.

## Automated checks

Final job `c3b69ae8-6237-48d0-9a58-4c8f571786d3` completed with exit **0**.
Logs and source snapshot are under `target/work/oct07-release-candidate-final`.

- Frontend typecheck, **396 tests / 34 files**, and production build passed.
- Native workspace: **863 passed**, zero failures; **14 explicitly opt-in tests
  ignored** in the normal suite. No new ignores were added.
- Formatting, Clippy with warnings denied, dependency notices and **144 Python
  tests** passed (one existing optional test skipped).
- Both public-alpha recipes validate with **zero findings**. Released Modpack,
  Rebalance and BuffBot archives passed fresh production acquisition/extraction
  checks and native TP2-order checks in both recipes; see
  `manifest/evidence/release-intake-20261007.toml`.
- The normal Evandra-default packaging guard passed before the NSIS build.
- Four package/signature checks passed, including the opt-in check against this
  exact setup and the bundled updater public key.
- The opt-in real Tauri updater download check passed: version discovery,
  exact signed bytes accepted, deliberately altered bytes rejected. It did not
  install the update or test updater restart.
- Public component inventory: **462 default components, 51 runs, 39 mod entries**.

The first package attempt stopped before building because three tests still
expected the old recipe. Those expectations were corrected to the approved exact
counts, pin and native order; all 13 affected tests and the final full suite
passed. No selection, archive or install-history safety check was weakened.

## Defender and runtime boundaries

Both the exact final setup and its extracted contents returned **no threats**,
exit 0, with security intelligence **1.459.576.0**. Antivirus, real-time,
behavior-monitor and downloaded-file protection remained enabled. No exclusions
or security preferences were changed. Diagnostic `-DisableRemediation` custom
scans report findings without scan-triggered remediation; real-time protection
was not disabled.

The preceding October 1 app candidate passed the authorized October 7
installation, Play, details/path-copy, Radar-off and user-approved Radar-on
checks without new Defender detection events. See
[the separate launch evidence](defender-smoke-2026-10-07.md). Those checks are not
a launch test of this rebuilt binary. Declined Radar consent, full new-install
gameplay and updater apply/restart remain untested in this final packaging pass.
No stream game or saved character was edited by the release work.

These are bounded clean-scan/reproduction results, not a Microsoft guarantee or
Windows Authenticode publisher certificate. The owner authorized publication
after this assessment; a generic Microsoft no-detection reply is not treated as
a guarantee for future builds.

## Publication

[Alpha.19](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.19)
is published as a prerelease, following the existing CEBG alpha convention.
Source through `a23b811` was pushed to `codex/installer-v0-real-alpha` first.
All seven versioned public assets were anonymously downloaded and matched local
hashes: setup, signature, feed, checksums, component inventory, license and
notices. The downloaded setup also passed the bundled-public-key signature test.

The alpha channel was updated only after those checks. Its cache-busted anonymous
download matches the exact feed hash above and announces app/recipe alpha.19.
The first ordinary-URL request returned cached alpha.18/recipe alpha.16. A later
ordinary-URL download now also returns app/recipe alpha.19 and matches the exact
feed hash; CDN convergence is verified without replacing the artifact again.
The [collection website](https://bg.chrizfader.org/collection) is published and
verified live. Website [PR27](https://github.com/Chrizhermann/twitch-setup-chriz/pull/27)
merged as `58f3b5668467642b42c5e3161514ccdd40367387` (content `2465e3a`).
All 115 site tests, build, PR CI and merged-master CI passed; master run
`37540148672`. Live content matches the reviewed source and displays app/recipe
alpha.19. All six public routes return HTTP 200 with security headers. The setup
downloaded through the live website's actual URL again matches 6,017,614 bytes
and the setup hash above. Existing rules and non-collection content are unchanged.
Desktop/mobile visual review was not performed; no style/runtime changes were
made. The preview server was stopped and the website worktree is clean.

Versioned assets are immutable. Hosted source CI is separate from the completed
local checks and was still running when publication began.

## Cleanup

After all local Cargo processes exited, the exact non-linked
`target/debug/incremental` directory was removed: **81,695 generated files,
48,611,704,999 logical bytes (45.27 GiB)**. Cargo can regenerate that cache.
The directory is absent; C: reported about 117.7 GiB free afterward. Logical
file totals are not a measurement of physical space reclaimed.

Release packages, signatures, input archives, download-verification copies,
job logs, diagnostics, all game installs, saves and backups were retained.
No other source worktree or unrelated temporary folder was deleted.

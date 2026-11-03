# Alpha.21 package verification — October 14

App **0.1.0-alpha.21**, bundled collection **0.1.0-alpha.20** (unchanged). Build
source: `c938a052a4df5a4ea79b37313675458ae7190a73`. Later publication documentation
does not alter the verified package.

## Why this release

A streamer's resumed alpha.20 installation stopped on every retry with "could not
inspect extraction marker … (os error 2)". CEBG's shared download and extraction
cache had been partly deleted by hand and by the uninstaller's "Delete the
application data" option, which cannot remove very long paths. Removing the
affected installations from My installs then always failed.

- **Cache repair** (`0835256`, `5dee0f1`): reused downloads and unpacked mods are
  checked before use. Only definite damage inside the entry (missing or corrupt
  content, unreadable records after a full read) sets it aside and rebuilds it,
  under a per-entry lock so concurrent requests repair it once. Files that are only
  locked, denied or busy stop the step without deleting anything.
- **Remove from list** (`05ca57f`, `36b0baa`, `c938a05`): a folder that CEBG can no
  longer prove to be the installation (records missing, damaged or made for another
  folder; renamed; replaced by a file or link) is only removed from My installs, and
  nothing in it is deleted. A deleted, replaced or disconnected download cache no
  longer prevents deleting a proven installation, and journal cleanup never follows
  links out of application data.

Three independent reviews (two Codex, one Claude) raised thirteen findings on the
first versions of these fixes. Ten were fixed with tests, each shown to fail when
its defect is reintroduced; one did not apply (no released CEBG repairs cache
entries). Two are deferred to the next version: a hung process holding a cache
lock can keep **Stop** waiting, and a journal-cleanup failure after a successful
removal is reported as an error. Also deferred: long-path-safe cleanup of CEBG's
application data, and incomplete-campaign availability when the download cache
is gone.

## Exact package

- Setup: `Chriz.Easy.BG_0.1.0-alpha.21_x64-setup.exe`, **6,101,746 bytes**.
- Setup SHA256: `c2a725a80409505c46d5724da50c27df7a7e58001fd74d0c2f999f86bb9527ed`.
- Signature SHA256: `e3bc96e7eb839773fedd17a8571fa1dc356ad9c8dd00483fa3a7ed6cf3dfd118`.
- Feed SHA256: `3b76538b665783ab9173e2716c5a59a6dc73f13742d22580580fdaf1fddbde18`.
- Extracted app SHA256: `ecf334db3112ef4eba7d1ec136be4b7659a804ea98de0f69c2c2c23a492c8776`.
- All **239 bundled resources** matched source bytes. The public component
  inventory (469 default components, 39 mod entries) equals alpha.20's except for
  the application version.
- A verified copy was saved to the owner's Downloads folder for review. An earlier
  candidate built from `36b0baa` was superseded before publication; its outputs
  were moved to `target/work/oct14-superseded-36b0baa`.

## Automated checks

The final packaging run completed with exit **0** on a clean checkout of the build
source. Its log and source snapshot are under `target/work`
(`oct14-release-candidate.log`, `oct14-release-candidate-final`).

- Frontend typecheck, **415 tests / 35 files**, and production build passed.
- Native workspace: **937 passed**, zero failures; **14 explicitly opt-in tests
  ignored** in the normal suite. No new ignores were added.
- Formatting, Clippy with warnings denied, dependency notices and **161 Python
  tests** passed (one existing optional test skipped).
- The October 13 pinned inputs passed the native TP2-order checks again: Modpack,
  SoD and Rebalance in both recipes, Bardic Wonders in the curated recipe.
- Package/signature checks passed, including the opt-in check against this exact
  setup and the bundled updater public key.
- The opt-in real Tauri updater download check passed from alpha.20 to alpha.21:
  version discovery, exact signed bytes accepted, deliberately altered bytes
  rejected. It did not install the update or test updater restart.

One engine CLI test (`manual_checkpoints_resume_two_boundaries_and_finalize_with_dragonspear_history_intact`)
failed once in a separate engine-only run between the two candidates and passed
alone, in four further full CLI runs and in the final run. It is the test named in
[issue 9](https://github.com/Chrizhermann/chriz-bg-collection/issues/9), which fails
whenever any `weidu.exe` runs on the machine; other agent sessions were active.
Its error text was not captured, so the cause is likely but unconfirmed.

The setup was signed with the existing updater key, passed to the signer by path
only. No key was created, copied or rotated.

## Defender and runtime boundaries

Both the exact final setup and its extracted contents returned **no threats**,
exit 0, with security intelligence **1.459.691.0**. Antivirus, real-time,
behavior-monitor and downloaded-file protection remained enabled. No exclusions
or security preferences were changed.

No setup, app or game was launched by this packaging pass. The cache repair and
Remove from list are covered by automated tests, including Windows locks, denied
listings, junctions, an unused drive letter and concurrent repair; this exact
binary was not launched. The uninstaller was not run. No stream game or saved
character was edited.

These are bounded clean-scan results, not a Microsoft guarantee or a Windows
Authenticode publisher certificate. The owner authorized publication on October 14.

## Publication

[Alpha.21](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.21)
is published as a prerelease (13 October 2026, 19:50 UTC), following the existing
CEBG alpha convention. Source through `c938a05` was pushed to
`codex/installer-v0-real-alpha` first. All seven versioned public assets were
anonymously downloaded and matched local hashes: setup, signature, feed,
checksums, component inventory, license and notices. The license and notices are
the committed file bytes. The downloaded setup also passed the bundled-public-key
signature test.

The alpha channel was updated only after those checks; the previous alpha.20 feed
was saved locally for rollback. Both a cache-busted and an ordinary anonymous
download of the channel feed match the exact feed hash above and announce app
alpha.21 with collection alpha.20. Versioned assets are immutable.

After the owner approved it, the [collection website](https://bg.chrizfader.org/collection)
was updated and verified live. Website PR 29 merged as `37064d7` (content
`a607a59`); its 115 site tests, static build, PR CI and merged-master CI passed.
The live content file matches the merged commit byte for byte and shows app
alpha.21 / collection alpha.20 with the setup hash above. The current run,
collection, rules, voicepacks, WeiDU log and links routes return HTTP 200 with
security headers; the root redirects. The setup downloaded through the live
website's link matches 6,101,746 bytes and the setup hash above. A local preview
showed no console errors and no horizontal overflow at 375 px width. Run rules and
other non-collection content are unchanged.

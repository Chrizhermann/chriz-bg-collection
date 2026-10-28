# CEBG Windows releases

The public distribution repository is `Chrizhermann/chriz-easy-bg`. It is
for the installer, release notes, signatures and update metadata. It should not
contain third-party mod archives. The public source lives separately in
`Chrizhermann/chriz-bg-collection`. CEBG downloads
mod sources from their own publishers; a packaged recipe is not a mod bundle.
Repository creation and publication are separate release actions.

Before a release, every one of our own mods must be on its newest release in every
installer that ships it, or held with Christopher's recorded reason. Run
`python -I tools/own_mod_pins.py check --release` and keep its output in the acceptance
record; see [our own mods](own-mod-pins.md).

After a successful signed NSIS build, prepare its update files from PowerShell:

```powershell
./tools/package-cebg-update.ps1 `
    -Version '0.1.0-alpha.2' `
    -SetupPath './target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.2_x64-setup.exe' `
    -SignaturePath './target/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.2_x64-setup.exe.sig' `
    -OutputDirectory './target/cebg-release/0.1.0-alpha.2' `
    -Notes "Navigation fixes, compact Updates, and full-setup support."
```

Use the actual `.exe` and its matching Tauri `.sig`; keep the signing key outside
the repository. The helper never reads a private key, builds, or uploads anything.
`-RecipeVersion` defaults to the app version and can be supplied separately when
the bundled collection keeps a different version.

The helper checks that the setup and base64 signature are nonempty, then writes
`latest.json` and `SHA256SUMS`. The checksum file covers the original setup, its
signature, and the generated feed. It does not cryptographically prove that the
signature matches the setup; Tauri verifies that pair against the bundled public
key before applying an update. Do not substitute a signature from another build.

For an approved release, the `v<version>` GitHub release receives the original
setup, its `.sig`, `latest.json`, and `SHA256SUMS`. The feed uses Tauri's
`platforms.windows-x86_64` shape and points to that version's setup asset, with
the portable dotted asset filename GitHub publishes. It also includes `recipe_version`, release
notes, and a UTC publication timestamp.

The configured alpha channel reads `releases/download/alpha/latest.json`. After
the versioned assets are published and checked, the same generated feed can be
published as `latest.json` on the separate `alpha` channel release. Update that
channel file for subsequent alphas; keep each versioned setup and signature
unchanged. The helper leaves both publishing steps to the release owner.

## Antivirus reports during testing

A clean archive scan or successful package test does not clear a later runtime
detection. The September 30 alpha.19 quarantine was assessed with the owner's
approved October 7 reproduction and final package scans. Those checks found no
new detection, and the owner authorized proceeding; see the
[launch evidence](defender-smoke-2026-10-07.md) and
[final package evidence](package-acceptance-alpha19-2026-10-07.md). The original
alert's precise cause is not established, and future builds are not guaranteed.

- Preserve the affected build and its hashes. Record the threat name, action,
  security-intelligence versions and launch sequence without publishing personal
  logs or extracting quarantine as a workaround.
- Use Microsoft's developer-analysis submission route with the owner's approval.
  Record the submission ID and actual result; don't label it a confirmed false
  positive before evidence supports that conclusion.
- Tauri updater signatures do not supply Windows Authenticode publisher identity.
  Consistent publisher signing is worth evaluating, but neither signing nor an
  earlier clean scan guarantees that future behavior detections cannot occur.
- If Microsoft needs reproduction, arrange a separate, approved disposable
  environment with protection enabled. Distinguish opening CEBG, Play with Radar
  off, and Play with Radar consent. Do not conduct those live tests on the user's
  playing installation without agreement.
- Do not publish instructions to disable antivirus, add broad exclusions, or
  change binaries merely to avoid detection. Keep the public release on hold
  until the incident has been assessed and the owner chooses how to proceed.

[Microsoft guidance for software developers](https://learn.microsoft.com/en-us/defender-xdr/developer-faq)
and [file-analysis submission](https://www.microsoft.com/en-us/wdsi/filesubmission).

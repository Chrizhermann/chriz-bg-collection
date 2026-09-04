# CEBG Windows releases

The proposed public distribution repository is `Chrizhermann/chriz-easy-bg`. It is
for the installer, release notes, signatures and update metadata. It should not
contain the private collection source or third-party mod archives. CEBG downloads
mod sources from their own publishers; a packaged recipe is not a mod bundle.
Repository creation and publication are separate release actions.

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
spaces in the filename URL-encoded. It also includes `recipe_version`, release
notes, and a UTC publication timestamp.

The configured alpha channel reads `releases/download/alpha/latest.json`. After
the versioned assets are published and checked, the same generated feed can be
published as `latest.json` on the separate `alpha` channel release. Update that
channel file for subsequent alphas; keep each versioned setup and signature
unchanged. The helper leaves both publishing steps to the release owner.

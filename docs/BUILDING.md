# Building CEBG on Windows

This guide describes the source audited at `bdb040e0d0ab7c63eac260497f3b828116fcb6aa`
and its documentation cleanup. The supported packaging target is Windows x64 MSVC with
NSIS. Building the app does not require installed games, mod archives, creator reference
directories, or production signing credentials. Running a game installation is a separate
acceptance activity requiring supported game sources and its own disposable destination.

## Prerequisites

- Use PowerShell on Windows x64. Install Microsoft C++ Build Tools with the Desktop
  development with C++ workload and a Windows SDK, plus the WebView2 runtime for native
  app use. See [Tauri's Windows prerequisites](https://v2.tauri.app/start/prerequisites/#windows).
- Install Rust through rustup with an MSVC host. The repository's `rust-toolchain.toml`
  selects **1.97.1**, `clippy`, and `rustfmt`; do not substitute an unrecorded toolchain
  for release verification. Run Cargo from PowerShell to avoid Git Bash's unrelated
  `link.exe` shadowing the MSVC linker.
- Install Node.js **22.12 or later in the 22.x line**, with npm. This satisfies the
  committed frontend dependencies' engine ranges. Use `npm ci` with the committed
  `app/package-lock.json`, not a dependency refresh.
- Python **3.11 or later** and PowerShell 7 (`pwsh`) are needed for the authoring/packaging
  tooling tests. They are not needed to compile the frontend and native app.

Work in a separate checkout/worktree. The commands below start at the repository root,
use its default `target/` directory, and assume the prerequisites are on `PATH`.
Dependencies and NSIS build tools may be downloaded on the first build. Do not share a
build output directory with another active build or replace a running installation.

## Compile and check without signing keys

Install the locked frontend dependencies and run its typecheck, tests, and production build:

```powershell
Set-Location app
npm.cmd ci
npm.cmd run check
```

Still in `app/`, compile the desktop app without creating an installer or launching it:

```powershell
npm.cmd run tauri -- build --debug --no-bundle -- --locked
```

`--no-bundle` bypasses packaging, so no updater private key is required. The resulting
debug executable is under the workspace's `target/debug/`. The flag and config-override
behavior are documented in the [Tauri CLI reference](https://v2.tauri.app/reference/cli/#build).

Return to the repository root for native and tooling checks:

```powershell
Set-Location ..
cargo fmt --all -- --check
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
python -m unittest discover -s tools/tests
```

Normal tests use fixtures and temporary directories. Tests marked ignored require explicit
inputs such as a real WeiDU executable, external download/cache, game-source probe, or a
signed setup. Do not run all ignored tests as part of a routine source build. A passing
normal suite does not establish full installation, gameplay, save/reload, or native updater
apply/restart acceptance.

## Optional local NSIS package without updater signing

For a local packaging check, use a generated override file. From `app/`:

```powershell
New-Item -ItemType Directory -Path ..\target -Force | Out-Null
$cebgLocalConfig = Join-Path (Resolve-Path ..\target).Path 'cebg-local-build.json'
'{"bundle":{"createUpdaterArtifacts":false}}' | Set-Content -LiteralPath $cebgLocalConfig -Encoding utf8
npm.cmd run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis --config $cebgLocalConfig -- --locked
```

This creates a local setup under
`target/x86_64-pc-windows-msvc/release/bundle/nsis/`. It does not create the Tauri `.sig`
required by the production updater and does not provide an Authenticode signature. The
override leaves the tracked production configuration and its public updater key/URL intact.
It is a packaging check, not an official release or an installer smoke test; do not upload
this setup to the live update channel. See Tauri's
[`createUpdaterArtifacts` setting](https://v2.tauri.app/reference/config/#createupdaterartifacts).

The production resource allowlist is in `app/src-tauri/tauri.conf.json`. It bundles CEBG's
license/notices, runtime manifest data, and `recipes/curated-full-current/`. It excludes
creator-only recipes and authoring reference/evidence inventories. Mods are acquired from
their declared sources during installation. Linked dependencies retain their own licenses;
see [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md), including the UnRAR restriction.

After Cargo and npm have populated their caches, check that the bundled notices match
the locked packages and pinned supplementary license texts:

```powershell
$cebgNpmCache = (npm.cmd config get cache).Trim()
python tools/generate-third-party-notices.py --check --npm-cache $cebgNpmCache
```

Run this command from the repository root. To regenerate after an intentional dependency
update, omit `--check`, then review the resulting notice changes and any missing upstream
texts. The generator verifies archive checksums, reads licenses in memory, and performs
no downloads or builds. `LICENSES/supplemental.json` records sources and checksums for
upstream packages that omit notices and for embedded components. Git preserves those
supplemental files' bytes so Windows checkout does not invalidate the hashes.

## Release identity and signing

The recorded alpha.13 release provenance is:

| Item | Recorded identity |
| --- | --- |
| App | `0.1.0-alpha.13` |
| Bundled current recipe | `0.1.0-alpha.12`, 434 recommended components |
| Released implementation commit | `9f89830be760338c74a2f0839a25e2cd1980faad` |
| Publication-evidence commit | `bdb040e0d0ab7c63eac260497f3b828116fcb6aa` (documentation only) |
| Setup SHA-256 | `cd6002efcc0c4b1e38a946be75f0fce014a5f1f183c30cdec9f100776bab5d51` |

The [acceptance record](patch-acceptance-alpha13-2026-09-07.md) ties that implementation to
the published package and records anonymous download/signature checks. A local rebuild is
not claimed to reproduce identical setup bytes: SDK, packaging-tool versions, environment,
and signing metadata need their own captured provenance.

The source repository is `Chrizhermann/chriz-bg-collection`. Binary distribution uses the
separate `Chrizhermann/chriz-easy-bg` repository.
Preserve these existing public locations:

- [Versioned alpha.13 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.13)
- [Versioned alpha.13 setup](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/v0.1.0-alpha.13/Chriz.Easy.BG_0.1.0-alpha.13_x64-setup.exe)
- [Alpha update feed](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/alpha/latest.json)

The production build enables Tauri updater artifacts. An authorized release builder supplies
`TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` through external secret
storage; no contributor needs those credentials to compile or run normal tests. The public
verification key in `tauri.conf.json` is deliberately public. It is not a private signing key.

Three signing concerns are separate:

- **Tauri updater signing** authenticates the final setup bytes to CEBG's updater. It does
  not give Windows a trusted publisher identity. Alpha.13 has this signature.
- **Recipe signing** uses `tools/package-recipe.ps1`, Minisign, and an external private key
  with its matching `.pub` file. This signs a recipe envelope and verifies it immediately;
  it is separate from app packaging and unnecessary for building the bundled recipe.
- **Windows Authenticode** identifies a Windows publisher. Alpha.13 has no Authenticode
  signature. Publishing source alone neither signs a binary nor establishes acceptance
  by a certificate provider or signing sponsorship program.

`tools/package-cebg-update.ps1` consumes an existing setup and Tauri `.sig` to generate local
`latest.json` and `SHA256SUMS`. It does not build, sign, cryptographically verify the signature,
or publish anything. Its release URLs intentionally target the existing distribution repo.

## Windows CI and remaining release automation

[Windows source checks](../.github/workflows/ci.yml) runs on pushes, pull requests, and manual
dispatch. It uses a Windows 2022 runner, Node 22, Python 3.11, and the repository's Rust
toolchain. Official GitHub actions are pinned to immutable commit IDs; the workflow grants
read-only repository permissions and does not receive production signing keys.

The workflow installs the locked frontend dependencies, runs the frontend check, Rust
format/tests/Clippy, Python tooling tests and locked-notice verification, then builds an unsigned NSIS package
using the local updater-artifact override. It does not launch the setup or the app, run a
game installation, publish a release, or edit the live updater feed. These are configured
checks; consult the actual [Actions run](https://github.com/Chrizhermann/chriz-bg-collection/actions/workflows/ci.yml)
for a passed or failed result.

Each run retains source-check evidence for 14 days: checked-out commit, app/recipe versions,
runner/Visual Studio/SDK/tool versions, source-input hashes, command logs, production Rust
dependency graph, and direct npm dependency versions. A successful run also retains the
setup with an explicit `UNSIGNED-CI` filename, provenance, and SHA-256 for seven days. For
pull requests, the checked-out source may be GitHub's test merge commit; the provenance
records that exact commit. Workflow artifacts are development outputs, not official
releases or signatures, and do not establish byte-for-byte reproduction of alpha.13.

Production release automation still needs an approved Authenticode integration and durable
source-to-release evidence tied to the final public artifact. Sign CEBG-owned executable(s),
package and Authenticode-sign the final setup, then create the Tauri updater signature and
checksums over those final unchanged setup bytes. Verify the exact published bytes while
preserving existing versioned release assets and updater URLs. Native updater apply/restart
and full game acceptance remain separate checks.

No release, signing enrollment, key rotation, or live feed change is performed by the build
instructions above.

# CEBG signing and dependency review — 2026-09-07

Audit base: `bdb040e0d0ab7c63eac260497f3b828116fcb6aa` on the separate
`codex/public-source-signing-audit-20260907` worktree. This review covers dependency
licenses, configured bundle resources and build/signing provenance. The companion
publication audit owns repository history, privacy and third-party fixture provenance.

## Verdict and classification

| Finding | Classification | Action |
|---|---|---|
| CEBG's own source is MIT; UnRAR is a fetched dependency with a separate license. | No source-publication blocker identified by this dependency review. | Preserve the dependency and describe its separate terms; the full repository/history audit still controls publication. |
| The shipped engine statically links UnRAR with a restriction on reconstructing RAR compression. | Signing-only blocker under the plain reading of SignPath's current all-components licensing rule. | Do not claim free-signing eligibility. Keep extraction working; obtain a service-specific determination only if later authorized. |
| The checked-in notices cover CEBG and UnRAR, but not the wider Rust/frontend dependency set. | Binary-release compliance cleanup, separate from exposing unvendored application source. | Assemble versioned dependency notices and applicable source links before the next binary release; inspect embedded sublicenses rather than trusting SPDX fields alone. |
| The EET compatibility marker reproduces three upstream batch lines. | Source attribution/provenance cleanup; whole-program license implications not established by this bounded audit. | Add the exact upstream file/commit and recorded GPL-3.0 provenance to notices and a scoped source comment; do not represent borrowed text as authored MIT code. |
| No checked-in CI workflow or Authenticode configuration establishes an approved source-to-signed-binary chain. | Signing/provenance work, not a reason to keep source private. | Add reviewed Windows build/signing automation after selecting a signing route. |
| Earlier research assumes SignPath eligibility and a paid fallback's price/availability. | Small documentation cleanup. | Mark those assumptions superseded; current release notes already distinguish updater signing from Authenticode. |

No dependency, application implementation, recipe/default/pin, signing key, installer,
release artifact, updater feed or running installation was changed by this review.

## Method and dependency scope

Read `CLAUDE.md`, the live `docs/handover.md`, the supplied audit handover, root and
crate manifests, both lockfiles, Tauri resources/build configuration, packaging tools,
and the cached manifests/license files referenced below. There were no builds, tests,
package installations, enrollment actions or GitHub operations.

The first inventory parsed `Cargo.lock` and existing Cargo-registry manifests. The lock
contains **534 package/version entries: two workspace packages and 532 third-party
packages**. All 532 cached package manifests were available and declare licenses.
A conservative closure from normal/build roots across all platforms contained 518
third-party entries; this is not the Windows shipping count.

The target-specific inventory then used this read-only, offline command in PowerShell:

```powershell
cargo tree --locked --offline --target x86_64-pc-windows-msvc `
  -p chriz-bg-app --edges normal,build --prefix none --format '{p}|{l}'
```

Deduplicating package/version pairs and excluding the two workspace packages yields
**321 third-party packages in the Windows normal/build dependency graph**, including
procedural macros and build tools. This command resolved metadata without compiling
or downloading. It is a dependency-graph inventory, not a claim that every listed
crate's machine code occurs in the final executable or a binary-derived SBOM.
No license field in this graph is empty. Neither lockfile was changed.

`app/package.json` has **one direct runtime dependency** and eight direct development
dependencies. `app/package-lock.json` contains **165 third-party package entries**;
exactly one lacks the development-only flag: `@tauri-apps/api` **2.11.1**, under
`Apache-2.0 OR MIT`, with no dependencies of its own. All 165 entries declare licenses.
The remaining 164 entries are build/test tooling and optional platform tool packages;
their presence in the lockfile does not mean they are shipped to players.

### Direct Rust dependencies at the audit base

Engine declarations: 24 normal dependencies, including three Windows-only entries.
The app declares ten external normal dependencies plus the local engine, and one
build dependency. Five external app dependencies overlap the engine. There are
therefore **29 distinct direct external normal dependencies plus `tauri-build`**.
Versions below are resolved from `Cargo.lock`, not just manifest requirements.

| Dependency | Version | Declared license | Direct owner |
|---|---|---|---|
| clap | 4.6.6 | MIT OR Apache-2.0 | engine |
| crossbeam-channel | 0.5.16 | MIT OR Apache-2.0 | engine |
| ctrlc | 3.5.2 | MIT/Apache-2.0 | engine |
| fs4 | 0.13.1 | MIT OR Apache-2.0 | engine |
| hex | 0.4.3 | MIT OR Apache-2.0 | engine |
| minisign-verify | 0.2.5 | MIT | engine |
| pelite | 0.10.0 | MIT | engine, Windows |
| same-file | 1.0.6 | Unlicense/MIT | engine |
| semver | 1.0.28 | MIT OR Apache-2.0 | engine + app |
| serde | 1.0.229 | MIT OR Apache-2.0 | engine + app |
| serde_json | 1.0.151 | MIT OR Apache-2.0 | engine + app |
| sevenz-rust2 | 0.22.2 | Apache-2.0 | engine |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | engine |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | engine |
| thiserror | 2.0.20 | MIT OR Apache-2.0 | engine |
| toml | 1.1.4+spec-1.1.0 | MIT OR Apache-2.0 | engine + app |
| unrar_sys | 0.5.8 | MIT OR Apache-2.0 **for the binding; embedded UnRAR terms also apply** | engine |
| ureq | 3.4.0 | MIT OR Apache-2.0 | engine |
| url | 2.5.8 | MIT OR Apache-2.0 | engine |
| walkdir | 2.5.0 | Unlicense/MIT | engine |
| widestring | 1.2.1 | MIT OR Apache-2.0 | engine |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | engine, Windows |
| winreg | 0.55.0 | MIT | engine, Windows |
| zip | 8.6.0 | MIT | engine + app |
| tauri | 2.11.5 | Apache-2.0 OR MIT | app |
| tauri-plugin-dialog | 2.7.3 | Apache-2.0 OR MIT | app |
| tauri-plugin-updater | 2.11.0 | Apache-2.0 OR MIT | app |
| time | 0.3.55 | MIT OR Apache-2.0 | app |
| windows | 0.61.3 | MIT OR Apache-2.0 | app, Windows |
| tauri-build | 2.6.3 | Apache-2.0 OR MIT | app, build |

The 321-entry Windows graph includes five MPL-2.0 packages: `option-ext 0.2.0`,
`cssparser 0.36.0`, `cssparser-macros 0.6.1`, `dtoa-short 0.3.5`, and
`selectors 0.36.1`. Other declarations include MIT/Apache alternatives, BSD, ISC,
Zlib, Unicode-3.0, and certificate data under CDLA-Permissive-2.0. These are license
declarations, not an assertion that every data license fits SignPath's code policy.
The complete target-specific name/version/license inventory can be reproduced with
the command above; it should drive a later notice generator rather than a manually
maintained second lockfile.

### Notice work needed for binary releases

[`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md) currently has only the
UnRAR section. [`LICENSE`](../../LICENSE) supplies CEBG's MIT text. The bundle includes
both, but there is no broader notice generation in the checked-in packaging tools.
The existing package-contract test checks those two resources exist; it does not
establish license completeness.

Generate the applicable copyright/license texts and source locations for the
resolved Windows graph and bundled frontend runtime, filtering build-only packages
where justified. Include upstream sublicenses and NOTICE files where applicable.
For example, `ring 0.17.14/LICENSE` expressly points to `LICENSE-BoringSSL`,
`LICENSE-other-bits`, and the once_cell subdirectory licenses; its single Cargo SPDX
expression is not the whole notice inventory. The UnRAR exception below likewise
demonstrates why the manifest field alone is insufficient.

Where MPL code occurs in the distributed executable, tell recipients where its
corresponding source is available. Mozilla describes this requirement for unchanged
MPL code compiled into larger works; it does not require changing CEBG's own MIT
license. [Mozilla MPL FAQ, Q8](https://www.mozilla.org/en-US/MPL/2.0/FAQ/)

This audit did not inspect a freshly built installer for all embedded license text,
resolve code elimination, or review every upstream source file. It identifies the
notice gap without claiming a complete legal clearance or changing existing releases.

### Small EET compatibility excerpt

`engine/src/weidu/eet_compat.rs:72-80` contains `ORIGINAL_BATCH` (three upstream
Windows batch lines) and CEBG's corrected replacement. The original marker matches
`EET/lib/macros.tph` at commit `74e91d72bca5d073fa11c1d088b90d7ff0c7105d`.
The corresponding curated artifact's `[provenance]` records GPL-3.0 for original EET
code, with separate licenses for bundled third-party parts. This is a real source
excerpt, even though the full mod is fetched separately; the recipe-only architecture
does not make all copied text automatically MIT.

Proposed notice/source-comment text, to be applied by the coordinating owner without
altering the batch strings or behavior:

> The original three-line Windows batch marker in `engine/src/weidu/eet_compat.rs`
> comes from the EET project's `EET/lib/macros.tph`, commit
> `74e91d72bca5d073fa11c1d088b90d7ff0c7105d`. CEBG's artifact provenance records
> original EET code under GPL-3.0. The marker identifies the exact upstream routine
> for a hash-checked Documents-path compatibility correction; the complete EET mod
> is obtained separately from upstream.

Link the notice to the
[exact upstream file](https://github.com/Gibberlings3/EET/blob/74e91d72bca5d073fa11c1d088b90d7ff0c7105d/EET/lib/macros.tph)
and retain the provenance record. Attribution is useful cleanup, not a claim that it
alone resolves every license question. This bounded review does not determine the
copyrightability of these short functional commands or conclude that CEBG must be
relicensed as a whole. Confirm the excerpt's applicable terms before making an
unqualified repository-wide MIT assertion. This finding does not itself justify a
history rewrite or an extraction redesign; either publication strategy would need
the same accurate attribution.

## UnRAR: source publication and SignPath are different questions

[`engine/Cargo.toml`](../../engine/Cargo.toml) pins `unrar_sys = "=0.5.8"`.
[`engine/src/acquire/archive.rs`](../../engine/src/acquire/archive.rs) invokes its
listing and extraction APIs, including byte callbacks; this is working extraction
support, not a dormant declaration. The cached crate's `build.rs` compiles
`vendor/unrar/*.cpp` into `libunrar.a`, so the non-OSI restriction is within the
application dependency, not merely an unrelated downloadable mod.

The exact dependency license was read at
`$CARGO_HOME/registry/src/<registry>/unrar_sys-0.5.8/vendor/unrar/license.txt`.
It allows use/distribution of the decoder, with a prohibition on developing a
RAR-compatible archiver/recreating the compression algorithm and a notice condition
for modified source. The Rust wrapper's MIT/Apache alternative does not remove that
condition. CEBG's notice reproduces the relevant paragraph; the source is fetched
through Cargo and is not vendored in this repository.

The official [RARLAB source-download page](https://www.rarlab.com/rar_add.htm) pointed
on the audit date to
[`unrarsrc-7.2.7.tar.gz`](https://www.rarlab.com/rar/unrarsrc-7.2.7.tar.gz).
Its `unrar/license.txt` was inspected in memory without extracting files. It matches
the cached dependency license exactly after CRLF normalization. Archive SHA-256:
`01d903a7dcf413cb2925696d7796e48e38d471f79bfe7ef3ad2aebf6c12dbefd`;
license-member SHA-256:
`6ecc1687808b7d66b24f874755abfed7464d9751ed0001cd4e8e5d9bf397ff8a`.
This checks the license, not whether the current RARLAB source version is the source
compiled by `unrar_sys 0.5.8`.

SignPath requires OSI-approved licensing for all components and excludes non-open
components except system libraries. Its Foundation certificate names the Foundation
as publisher; admission remains discretionary. Required controls include verifiable
build origin, manual signing approval, MFA, assigned roles, and a published signing
and privacy policy. Sign only project-owned binaries; do not sign downloaded mods
under CEBG's identity. [SignPath Foundation terms](https://signpath.org/terms.html)

The UnRAR use restriction conflicts with the OSI prohibition on restrictions on a
field of endeavor. **Our inference is that the current linked binary does not meet
SignPath's all-components requirement.** Source publication alone does not cure it.
[Open Source Definition, section 6](https://opensource.org/osd)

Preserve extraction support. Reasonable next options, if separately authorized:

1. Keep UnRAR and investigate a paid Authenticode route, such as Azure Artifact
   Signing or a suitable certificate provider, subject to identity/location and
   service eligibility. This avoids an extraction rewrite. Tauri supports custom
   signing commands. [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/)
2. Ask SignPath for a determination with the exact static-link/license facts before
   investing in its pipeline. Do not label the existing design accepted or sponsored.
3. If free signing is essential, separately evaluate a truly compatible OSI-licensed
   RAR/SFX decoder. It must preserve archive-type handling, safety limits and the
   verified Evandra payload behavior. No replacement has been selected or validated;
   a different wrapper around the same UnRAR code would not solve the issue.

No price, personal eligibility, acceptance, or guaranteed SmartScreen outcome is
claimed. Microsoft's current FAQ explains that signed files can still receive
SmartScreen prompts. [Artifact Signing FAQ](https://learn.microsoft.com/en-us/azure/artifact-signing/faq)

## Bundled resources and build provenance

[`app/src-tauri/tauri.conf.json`](../../app/src-tauri/tauri.conf.json) expands to
**160 tracked resource files**: LICENSE (1), notices (1), manifest collection/release
(2), artifacts (32), mods (33), presets (1), game-build profiles (2), release metadata
(4), and the current curated recipe (84). Their extensions are 155 TOML, two JSON,
two Markdown and one extensionless license. This resource map is text metadata; it
does not package the referenced third-party mod archives. Authoring reference TSVs,
evidence directories and `recipes/creator-full-current` are excluded from this map.
The wider repository/history payload review is separate.

The only tracked application image assets are the geometric
[`app-icon.svg`](../../app/src-tauri/app-icon.svg) and `icons/icon.ico`, introduced
by scaffold commit `e391fa20fd22a97bf45beae050d49ebb6505f9b9`. No font files or third-party
artwork are configured; CSS names installed system fonts. The ICO was not regenerated
in this audit, so its exact derivation is not independently proved here. No
`externalBin` is configured. WebView2 is a platform runtime: the pinned
`tauri-utils 2.9.3` default is `DownloadBootstrapper { silent: true }`, not a bundled
fixed runtime. Future signing review should explicitly account for platform/runtime
and NSIS build-tool handling rather than treating these as CEBG-owned binaries.

`rust-toolchain.toml` pins Rust **1.97.1**. Cargo and npm lockfiles are tracked;
[`app/src-tauri/build.rs`](../../app/src-tauri/build.rs) delegates to `tauri_build`;
the Tauri configuration runs the ordinary frontend build. No `.github` workflow is
tracked at this base, and no Authenticode `signCommand`, certificate thumbprint or
signing policy is configured. `tools/package-cebg-update.ps1` accepts an already
built setup and signature, then emits feed/checksum metadata. It does not build or
Authenticode-sign the application. `tools/package-recipe.ps1` is separate Minisign
recipe-envelope signing and requires the private key outside the repository.

The recorded alpha.13 implementation commit is `9f89830`; comparing it to this audit
base changes only `docs/handover.md` and
`docs/patch-acceptance-alpha13-2026-09-07.md`. Thus the implementation, dependency
locks and configured resources here match that recorded released source commit.
The acceptance document records the setup hash
`cd6002efcc0c4b1e38a946be75f0fce014a5f1f183c30cdec9f100776bab5d51`.
This audit verified the Git relationship, not a reproducible rebuild or the current
remote binary. Public source should retain an explicit source-commit/version/hash
mapping; a clean snapshot would also need its origin commit and any deliberate
omissions documented.

For a future release pipeline: build from a reviewed immutable source ref and locked
inputs; generate notices; Authenticode-sign CEBG executables; package and
Authenticode-sign the final setup; then produce the Tauri update signature and
checksums over those final unchanged bytes. Verify the exact published bytes against
both schemes. Tauri's updater requires its own signatures, independent of the Windows
publisher certificate. [Tauri updater](https://v2.tauri.app/plugin/updater/)

## Handback

Source publication can proceed independently of free-signing eligibility once the
companion tree/history audit and scoped documentation cleanup are satisfied. This
review gives no dependency-based reason to require a new source snapshot instead
of the existing repository. Preserve all live download/updater URLs and the active
installation. Track broader binary notices, signing-provider selection and CI
provenance as follow-up release work; do not remove RAR extraction to make a scanner
or eligibility check pass.

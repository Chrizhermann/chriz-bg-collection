# Supplemental license provenance

`THIRD_PARTY_NOTICES.md` is the complete generated notice artifact bundled with
CEBG. This directory supplies the small number of license texts omitted from the
upstream package archives, plus the Microsoft WebView2 SDK license and notices
for the statically linked loader. It contains licensing documents, not upstream
mod, game, library or SDK payloads.

Run from a checkout whose locked Cargo/npm packages have already been cached:

```powershell
python tools/generate-third-party-notices.py
python tools/generate-third-party-notices.py --check
```

The script needs Python 3.11+ and Cargo on PATH. It calls offline `cargo tree` for
the Windows x64 normal/build graph without compiling. It reads Cargo `.crate`
archives and npm's content cache and verifies their lockfile checksums before
using their text. Use `--cargo-home` or `--npm-cache` for nondefault cache locations.
It never fetches packages, installs dependencies, extracts payloads or invokes a
build script. Missing or changed inputs fail rather than generating incomplete
notices. A CI runner must populate the required caches separately before checking.

The generated inventory conservatively includes build dependencies and Vite's
notice because Vite emits a module-preload helper. It includes every package
license/NOTICE/COPYING/COPYRIGHT/AUTHORS/acknowledgment file found, including nested
files, and deduplicates identical text after normalizing line endings and trailing
line whitespace while retaining each origin. Original supplemental bytes stay
unchanged; `.gitattributes` disables checkout line-ending conversion for them.
It is not a binary-derived SBOM or a whole-application license adjudication.

`supplemental.json` pins every supplement to the applicable Cargo archive hash
and the license text's own SHA-256. Sources are immutable upstream revisions
reported by each crate's `.cargo_vcs_info.json`, except the standard MPL-2.0 text
for `selectors`, supplied by the locked `cssparser` package. `selectors` itself
declares MPL-2.0 in its manifest and source headers but omits a full license file.
No author copyright has been invented to fill a missing package license.

The WebView2 SDK source URL identifies the exact NuGet version selected by the
binding's pinned upstream update script. This was downloaded only into memory to
read `LICENSE.txt` and `NOTICE.txt`; the x64 `WebView2LoaderStatic.lib` bytes were
compared with the locked `webview2-com-sys` archive and matched. Only those two
licensing texts are checked in. The SDK's license is BSD-style three-clause text;
the Rust wrapper's separate MIT notice is also retained. Archive/library/text
hashes and the selection-script URL are recorded in the manifest.

The generator preserves the manually reviewed opening sections of
`THIRD_PARTY_NOTICES.md`, including UnRAR and EET attribution. A dependency update
must regenerate/review the notices and refresh any affected supplements; stale
supplement package entries fail the check. All third-party texts retain their
upstream rights and are not relicensed by CEBG's root MIT license.

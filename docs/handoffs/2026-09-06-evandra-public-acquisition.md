# Evandra public acquisition — approved Windows manual fallback

## Current checkpoint (2026-09-06)

Christopher approved an upfront manual download **or Skip Evandra** while automatic
official acquisition remains blocked. He downloaded the official Windows release
to `C:\Users\chris\Downloads\evandra-v2.2.exe`. Requesting the Linux ZIP was unnecessary;
the Windows self-extractor can be unpacked without executing it.

- File: 13,430,253 bytes; SHA-256
  `21724b6d4679d6df6dbcf95a0a4dbe6ee41d5bfdb3ae13907ec89f5d00014861`.
- Read-only inspection: RAR4 payload at offset 184320; 198 entries (184 files),
  14,965,787 uncompressed bytes, largest file 1,487,545 bytes, maximum path depth 4.
  Single volume, non-solid. Roots are `evandra` and `setup-evandra.exe`.
- Recipe `0.1.0-alpha.11` replaces the private aggregate with the exact public
  `evandra-2.2-windows` manual artifact. It publishes only `evandra`; CEBG supplies
  its own verified WeiDU, and never executes the archive self-extractor.
- The inline start screen offers official-page download, explicit file selection,
  or skip. File size and SHA-256 are verified into CEBG's cache; cancelled/wrong
  files do not start work. Both freeze and worker start recheck cache readiness.
- Skip turns off `mod:evandra`, core component 0 and crossmod component 1 together.
  Other default component choices remain unchanged. No automatic Downloads scan
  or watcher, and no surprise auto-start after choosing/skipping.
- Focused recipe and UI/backend tests pass. Engine RAR extraction passed against
  the real package: 183 published files / 13,990,427 bytes, every file hash-identical
  to the read-only accepted Evandra reference. TP2 SHA-256 is
  `d35217e68ae3c91871af9b19fdb9dc6af64a652d0468bd938006e9d0f01a85d7`.
  Retained second-pass evidence is under `target/evandra-v2.2-rar-acceptance-v2/`.
- Narrow security review caught and resolved archive identity across reopening and
  output quotas: all passes use a synced engine-owned snapshot; RAR callbacks
  bound actual bytes before writing. Explicit MZ, path/link/collision/encryption/
  volume validation remains fail-closed. ZIP/IEMOD header restrictions are retained.
- Full frontend suite: 101 passed. Backend command contracts: 39 passed. RAR tests
  and focused validation/materialization regressions pass. Final signed package
  integration is pending. No game install or UI takeover was started.

No mod archive is bundled, mirrored, repacked, or published. No G3 message was sent.
The automatic route still needs an author/host-supported endpoint. The older
app alpha.9 / recipe alpha.10 package does **not** contain this fallback.

## Earlier automatic-only investigation (superseded where noted above)

Christopher approved replacing the private aggregate with automatic acquisition
from the official host on 2026-09-06. No approved component choices may change.

## Actual evidence

- The official [Evandra 2.2 download chooser](https://www.gibberlings3.net/files/file/1008-evandra/?do=download)
  lists `evandra-v2.2.exe`, `lin-evandra-v2.2.zip`, and `osx-evandra-v2.2.zip`.
- A normal unattended PowerShell `Invoke-WebRequest` to that chooser on
  2026-09-06 returned a Cloudflare managed challenge requiring JavaScript/cookies,
  not the archive. Do not store challenge tokens or copy browser cookies into CEBG.
- Earlier web-tool archive retrieval also failed at its download redirect. No
  official standalone archive had been downloaded or hash/layout verified at that
  point. Page readability is not automated acquisition acceptance.
- The engine's `ArchiveKind` then accepted ZIP/IEMOD only. The Windows
  self-extracting EXE cannot simply be relabelled as ZIP. A ZIP candidate must
  preserve the Windows-required mod payload, including its audio tooling.
- A bounded official-page/source search did not identify a verified supported
  alternative endpoint. `MattyGroove/Evandra` exists, but author endorsement and
  correspondence to the official release are unverified. It was not substituted.
- The [official readme](https://gibberlings3.github.io/Documentation/readmes/readme-evandra.html)
  asks users not to host or redistribute the mod without author permission.

## Scope left unchanged

No recipe URL, artifact digest, selections, package, cache, game or save was changed
by this investigation. No source package was mirrored, repacked or published, and
no author/forum message was sent. The alpha.9 app / alpha.10 recipe still uses the
private Evandra acquisition contract; that is not a public-distribution fix.

## Next decision

For the approved automatic route, request a stable, supported official ZIP URL
or official GitHub release from G3/Rhaella. Do not bypass the host's challenge,
freeze expiring signed URLs, or adopt an unverified third-party mirror.

Christopher subsequently approved the interim manual official download and supplied
the Windows package; the current checkpoint above supersedes this decision gate.
Do not ask public users for the private aggregate and do not drop Evandra silently.

Suggested message, not sent:

> Hi! I'm building Chriz Easy BG, an alpha installer that downloads mods from
> their official hosts rather than bundling them. We'd like to include Evandra
> 2.2, but G3's download page returns a Cloudflare browser challenge to the
> installer's HTTP client. Is there a supported stable ZIP download URL or an
> official GitHub release we should use? We'll credit/link the mod, verify the
> package checksum, and won't rehost anything without permission. Thanks!

Once an authorized route and package exist: verify exact bytes and safe layout,
compare the Evandra payload with the accepted source, update the authoring map and
generator under a new recipe version, then run one empty-cache acquisition and
extraction check. No full game installation is required for acquisition proof.

## Bounded code-seam handoff

Existing acquisition already limits HTTPS redirects to 10, checks cross-host
redirects against authored `redirect_hosts`, and verifies length/SHA-256 before
publication. A normal stable HTTPS source needs no new transport implementation.
Add a truthful `DirectHttps` source kind if using a non-GitHub endpoint, rather
than mislabelling it `GithubRelease`. Retain the existing extraction validations.

Generator changes belong in `tools/curated_full_recipe.py`: replace Evandra's
artifact ID in `MOD_SPECS` and remove the private aggregate copy/rewrite block in
`build_recipe`. Tests should require the official artifact and absence of the
private aggregate. Keep both Evandra runs and component selections unchanged.

Focused checks after receiving the verified source contract: Python
`tools.tests.test_curated_full_recipe`, Rust `artifact_manifest`,
`manifest_validate`, `acquire_http`, and the author CLI's `artifact verify` with
an isolated cache. The later Windows-package approval supersedes the earlier
ZIP-only scope; only the explicitly declared, verified RAR SFX format is added.

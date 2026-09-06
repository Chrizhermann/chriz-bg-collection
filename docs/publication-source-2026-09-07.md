# CEBG public source — 2026-09-07

Christopher authorized making the existing repository public and preparing it for
release, retaining MIT for CEBG's own source. This follows the
[bounded source audit](audits/2026-09-07-public-source-readiness.md).

## Publication scope

The existing source repository is now **public** at
[Chrizhermann/chriz-bg-collection](https://github.com/Chrizhermann/chriz-bg-collection).
GitHub recognizes its MIT license. A clean source snapshot was unnecessary: the
bounded audit found no secret or unlicensed game/mod payload requiring history removal.
The default branch was advanced normally, preserving the existing history.
The publication line integrates finalized app **0.1.0-alpha.14** source at
`5610783590ad49b24101d5d3a6a85b018abadb7e`, recipe **0.1.0-alpha.12** and the same
**434 recommended components**. The original audited alpha.13 implementation was
`9f89830be760338c74a2f0839a25e2cd1980faad`; `bdb040e` added release evidence.
The owning installer task completed alpha.14 while source publication was being
prepared. Only its finalized committed source was imported, without altering its
production code, recipe, lockfiles or generated notices. Publication-specific work
adds documentation, attribution, dependency notices and hosted build verification.

The old default branch's curation documents precede the integrated approved source
snapshot. Publication preserves current curation and both histories, bringing forward
the older research note that exists only on `main`. The installer task's uncommitted
work and runtime installation remain outside this source-publication task.

## Licensing and build readiness

- [LICENSE](../LICENSE) remains MIT for CEBG-owned material.
- [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) retains dependency terms and
  source references. The generation inputs are pinned by the committed dependency
  locks and any supplementary license sources; regeneration must verify those inputs.
- The [EET attribution record](audits/2026-09-07-eet-attribution.md) identifies the
  excerpt's contributor, exact upstream commit and license statement. Added source
  comments and notices preserve those rights; the executable logic is unchanged.
- [BUILDING.md](BUILDING.md) documents Windows builds without private game directories,
  mod archives or signing credentials. The Windows CI workflow checks the standalone
  checkout and retains build evidence and a clearly labeled unsigned development setup.
- MIT source publication does not establish SignPath eligibility. The linked UnRAR
  restriction and any service-specific component assessment remain relevant when
  selecting an Authenticode provider. No enrollment or purchase was performed.

## Existing distribution

The installer task published the [alpha.14 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.14)
and its [setup](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/v0.1.0-alpha.14/Chriz.Easy.BG_0.1.0-alpha.14_x64-setup.exe)
from source `5610783590ad49b24101d5d3a6a85b018abadb7e`, including the expanded notices.
This task independently downloaded the public setup without running it: **5,447,242 bytes**,
SHA-256 `c36058d92165ff4e2e3d993c5eaedee95e05c1de95f388668f61b4d7571c1a3b`.
The [alpha updater feed](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/alpha/latest.json)
was separately advanced to alpha.14 by the installer task and verified here.

The earlier [alpha.13 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.13)
remains available. Its asset IDs, sizes, hashes and URLs were compared before and after
the source-link documentation edits and are unchanged. Its setup SHA-256 remains
`cd6002efcc0c4b1e38a946be75f0fce014a5f1f183c30cdec9f100776bab5d51`.
The distribution README now links this public source and its build instructions;
both release descriptions link their immutable build source.

No released setup, updater signature, asset or feed is replaced by this source work.
CI packages have no Tauri updater signature or Authenticode signature and are not
official releases. Subsequent official releases must retain complete notices and
follow the normal versioning, signing and exact-public-byte verification flow.

## Verification record

During this work Christopher separately authorized the installer task to prepare
alpha.14 and continue his failed test installation. That task received the independent
notice/attribution commit `90cf4eeb31681012924f414f71c0dc06488164c6` for cherry-pick
before packaging, with instructions to regenerate the notice header after its version
bump. Its application, recovery and feed changes are separate from this publication.

- Before visibility changed, the refreshed remote history passed Gitleaks 8.30.1
  with redacted output (240 nonempty commits inspected), as did the publication tree.
  The earlier audit records the complete ref/blob and fixture inventory; the refreshed
  tree has 692 tracked files and no additional binary payload beyond the existing icon.
- Anonymous reads of the published README, MIT license, Cargo lockfile and complete
  notices at `c452a8ab6a3df256bfdc1346a49107e27faef1cd` match the committed bytes.
  A fresh Windows checkout also verified the pinned raw supplemental license bytes.
- The [initial hosted run](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/34062197102)
  passed frontend checks and Rust formatting, then found that the ignored updater
  acceptance integration test could not compile without Tauri's `test` feature.
  This feature is enabled only through a development dependency; the release's
  runtime code, dependency locks and bundled resource inputs remain unchanged.
  The [follow-up run](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/34062790645)
  compiled successfully and passed the app library, command and package tests, then
  reproduced the previously tracked updater mock-harness startup failure
  ([issue 2](https://github.com/Chrizhermann/chriz-bg-collection/issues/2)). Tauri's
  [test build example](https://github.com/tauri-apps/tauri/blob/dev/examples/api/src-tauri/build.rs)
  supplies a Windows manifest to address this failure; CEBG's test binaries receive
  the equivalent common-controls manifest through test-only linker arguments.
  Current CI status is available in
  [Windows source checks](https://github.com/Chrizhermann/chriz-bg-collection/actions/workflows/ci.yml).

No local application build, full game installation, updater apply/restart or native
window interaction is performed. Hosted build/test evidence does not imply fresh
gameplay, save/reload or installation acceptance.

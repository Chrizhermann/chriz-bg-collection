# CEBG public source — 2026-09-07

Christopher authorized making the existing repository public and preparing it for
release, retaining MIT for CEBG's own source. This follows the
[bounded source audit](audits/2026-09-07-public-source-readiness.md).

## Publication scope

The source repository is [Chrizhermann/chriz-bg-collection](https://github.com/Chrizhermann/chriz-bg-collection).
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

The [public alpha.13 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.13)
and its [setup](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/v0.1.0-alpha.13/Chriz.Easy.BG_0.1.0-alpha.13_x64-setup.exe)
remain the official downloads. Setup SHA-256:
`cd6002efcc0c4b1e38a946be75f0fce014a5f1f183c30cdec9f100776bab5d51`.
The [alpha updater feed](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/alpha/latest.json)
offered alpha.13 at the start of publication, with SHA-256
`56e5ecc70d0c16464f72ef69dc1621f95ed73cd11a53515c9e4e501575f1e7ef`.

No released setup, updater signature, asset or feed is replaced by this source work.
CI packages have no Tauri updater signature or Authenticode signature and are not
official releases. The next official app release must carry the expanded notices
and follow the normal versioning, signing and exact-public-byte verification flow.

## Verification record

During this work Christopher separately authorized the installer task to prepare
alpha.14 and continue his failed test installation. That task received the independent
notice/attribution commit `90cf4eeb31681012924f414f71c0dc06488164c6` for cherry-pick
before packaging, with instructions to regenerate the notice header after its version
bump. Its application, recovery and feed changes are separate from this publication.

Publication and hosted CI results are recorded here after their completion. The
initial audit's tree/history checks are evidence for that captured baseline; the
publication turn also checks its new source, notices, workflow and remote refs.
No local application build, full game installation, updater apply/restart or native
window interaction is performed. Hosted build/test evidence does not imply fresh
gameplay, save/reload or installation acceptance.

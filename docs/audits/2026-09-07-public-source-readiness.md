# CEBG public-source readiness audit

Historical audit record: Christopher subsequently authorized source publication.
See [the publication record](../publication-source-2026-09-07.md) for current status
and completed attribution, notice and build preparation.

Date: 2026-09-07 KST. Source audited: `bdb040e0d0ab7c63eac260497f3b828116fcb6aa`,
the verified tip of `codex/installer-v0-real-alpha`. Audit/cleanup branch:
`codex/public-source-signing-audit-20260907`, in its own worktree.

## Recommendation

**Publish the existing repository after the scoped documentation cleanup and the
small attribution follow-up below. A clean public snapshot is not warranted by the
audited tree/history.** No real credential, private game/mod archive, tester save,
raw diagnostic bundle or received private correspondence was identified. This is
a bounded publication recommendation, not exhaustive security or legal clearance.

The existing repository is still private. Nothing was published, pushed, renamed,
re-signed, rotated or enrolled. No game installation/source/archive, existing
download/cache, receipt or save was accessed or modified. The reported alpha.13
installation's location, progress and completion were not checked.

**SignPath is a separate conclusion:** the current binary statically links UnRAR.
Its restriction conflicts with SignPath Foundation's published all-components
OSI-licensing requirement. Do not claim eligibility or sponsorship; retain working
extraction. See the [dependency/signing review](2026-09-07-signing-dependency-review.md).

## What was scanned

| Surface | Evidence and scope |
| --- | --- |
| Prospective tracked source | All **653 files** exported from the exact baseline commit, excluding every untracked file and other worktree's state. Content, signatures, sizes and concrete payload leads inspected. |
| Actual remote refs/history | Fresh isolated bare mirror, checked against `git ls-remote` again after scanning: **3 branches, 0 tags, 235 reachable commits, 1,631 distinct blob versions / 29,953,989 bytes**. No shallow-history boundary. |
| Additional local reachable history | Supplementary `--all` scan: **40 refs, 259 commits, 1,666 distinct blob versions / 30,269,304 bytes**, including local branches and Codex snapshot/checkpoint refs. These local refs are not automatically published by changing remote visibility. |
| Commit metadata | All 259 local reachable commit messages and author/committer headers, which include the complete remote set. Normal maintainer identity/email metadata is present; values are not reproduced here. |
| LFS/submodules | No LFS pointers in any scanned blob, no gitlinks in reachable trees, no `.gitmodules` or `.lfsconfig`. `.gitattributes` only marks synthetic fixture bytes as binary. No separate LFS/submodule payload was indicated. |
| Source-repository releases | GitHub API: **0 releases, therefore 0 repository release assets**. |
| Source-repository Actions | API: **0 runs and 0 artifacts**; no tracked workflow at the baseline. Thus there were no run logs/artifacts to download. |
| Source-repository conversations | All **3 issues**, **0 issue comments, 0 PR review comments, 0 commit comments**. Issue data received a redacted secret scan plus correspondence/contact/path-marker checks. No PR entries appeared in the all-state issues response. |
| Other source-repository surfaces | Wiki, Pages and Discussions disabled; Pages API also returned 404. GraphQL repository `packages.totalCount` is **0**. Detailed package REST/GraphQL fields lacked `read:packages`, but the repository-linked count was readable; no new scope was requested. |
| Public distribution repository | `Chrizhermann/chriz-easy-bg` is already public and its recursive `main` tree is exactly `README.md`. Alpha.13 release/asset metadata was verified; this audit did not unpack or re-download its existing setup. |

Remote branch heads at capture:

| Branch | Commit |
| --- | --- |
| `codex/installer-v0-real-alpha` | `bdb040e0d0ab7c63eac260497f3b828116fcb6aa` |
| `feat/engine-phase1` | `745f0a94a4362c6a421251562d73c1fcb922c3c5` |
| `main` (current default) | `395c9400910c79897e8a21a7dcb16fc9a5ecf9d1` |

The scanner was official **Gitleaks 8.30.1**, with its downloaded Windows x64 ZIP
verified against the publisher's checksum list (SHA-256
`d29144deff3a68aa93ced33dddf84b7fdc26070add4aa0f4513094c8332afc4e`).
Default rules, `--redact=100`, `--ignore-gitleaks-allow`, no custom suppression
configuration, and archive traversal depth 3 were used. Both history scans used
`--log-opts='--all --full-history'`. Independent directory scans covered the
prospective tree, every unique full blob version, commit metadata and API text.
Full blobs supplement diff scanning so unchanged/merge/deleted-file content is
included. [Gitleaks release and tooling](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1)

History, tree, full-blob and commit-metadata Gitleaks scans returned zero findings.
The API-response scan flagged GitHub's generated `temp_clone_token` field in the
private-repository metadata response. That value was never printed or committed;
it was removed from the temporary response copy, and the sanitized surface scan
returned zero findings. It was generated authentication context, not a token found
in repository content. Supplemental private-key/provider-token/credential-URL
patterns found only synthetic tests, examined below. No credential rotation is
indicated by the repository-exposure evidence.

Temporary scanner/mirror/raw inventories remain outside the checkout. Only this
redacted account and the companion reviews belong in the proposed docs patch.
Current project guidance was authoritative. Historical memory about fake-game
builders was only a search lead; the current fixture bytes and builders were
independently inspected.

## Classified findings and cleanup

| Classification | Location / evidence | Handling |
| --- | --- | --- |
| Publication blockers | None confirmed in the scanned content/history. | Keep the remaining attribution question explicit; refresh any new refs/content before a later visibility action. |
| Small source-attribution follow-up | `engine/src/weidu/eet_compat.rs:72` embeds a three-line original EET batch marker, with a short correction. Artifact provenance identifies GPL-3.0 original EET code at `74e91d72bca5d073fa11c1d088b90d7ff0c7105d`. | Record its exact upstream attribution and disposition before claiming complete license compliance. The companion review supplies proposed notice wording. This audit does not decide whether that limited functional fragment carries whole-program obligations. A snapshot retaining the same fragment would have the same question. No working patch or recipe was altered. |
| Signing-only blocker | `engine/Cargo.toml:35`; `unrar_sys 0.5.8` compiles embedded UnRAR. | The MIT/Apache binding license does not remove UnRAR's restriction. Keep extraction; select an appropriate signing route or seek a later authorized SignPath determination. |
| Binary-release notice cleanup | `THIRD_PARTY_NOTICES.md` covers UnRAR but not the wider linked Rust/frontend dependency notices and applicable source links. | Generate/review complete notices for the Windows shipping graph before the next binary release. This does not require vendoring mods or hiding CEBG source. |
| Build/signing provenance follow-up | No CI or Authenticode pipeline at the baseline; recorded local build evidence links the release to source. | Use [BUILDING.md](../BUILDING.md); later CI must bind source, locked inputs, logs and final signed bytes. No claim of reproducible builds or fresh binary inspection is made. |
| Small cleanup, prepared here | `README.md`, new `docs/BUILDING.md` | Replace stale unpublished alpha.1 claims with app alpha.13 / recipe alpha.12 / 434 recommended components; document source versus downloads and secret-free Windows build steps. |
| Small privacy cleanup, prepared here | `docs/handoffs/2026-09-06-evandra-public-acquisition.md:82`; introduced in `aa44060` | Replace the unsent outbound-message draft with factual acquisition status; update its handover label. This does not erase the historical draft. It contains no received private correspondence/contact data and does not justify history rewriting. |
| Small cleanup, prepared here | `docs/overnight-2026-09-05.md`; original design and GUI/local-modding research briefs | Generalize the external private-key location, add historical-status banners, and mark old SignPath eligibility assumptions superseded. Keep useful engineering evidence. |
| Acceptable public verification material | `app/src-tauri/tauri.conf.json` updater public key; recipe hashes/signatures; `engine/tests/diagnostics.rs:71`, `:523` | Public keys are intended for distribution. Key/token-shaped diagnostic tests use obvious marker/example data, not usable secrets. Historical versions of these matches have the same synthetic provenance. |
| Acceptable synthetic fixtures | 26 game-shaped files / 400 bytes; 11 WeiDU log fixtures; source-generated fake games/RAR cases | Full fixture/provenance findings are in the [payload review](2026-09-07-payload-provenance-review.md). No game executable or DLC ZIP bytes are hidden behind those filenames. |
| Acceptable archive/source metadata | `recipes/creator-full-current/`, manifest reference/evidence files | Private archive filenames, source URLs, component identities, sizes and hashes are not archive payloads. Keep the recipe-not-mods architecture and nonredistribution statements. |
| Acceptable history metadata | Maintainer local paths, Git identity/email, community technical outcomes, old engineering decisions | These become visible with history; the targeted review found no tester identity/raw save/diagnostic or received correspondence to remove. No history rewrite is recommended merely for this metadata. |

The only binary blob in the complete remote history is the **7,594-byte authored
application icon**, introduced with its geometric SVG at `e391fa2`. The additional
local-only binary is an **82,621-byte illustrative UI screenshot** in Codex snapshot
`8e1f498`, visually inspected as a mock update screen without personal data. No
remote blob exceeds 285,640 bytes; the largest is recipe text. No executable,
game/mod archive, save or diagnostic payload was found in historical binary data.

## Publication and release handoff

1. Integrate only this reviewed documentation patch into the active source line,
   preserving concurrent shortcut/preferences implementation and untracked research.
   Resolve the small EET attribution/license item in the same publication preparation.
2. Prefer **the existing `Chrizhermann/chriz-bg-collection` repository**. Its default
   `main` is old; before public launch, explicitly choose the audited current branch
   as the landing branch or integrate it into `main` through normal review. Neither
   operation was performed here. Do not present old `main` as alpha.13 source.
3. Immediately before a separately authorized visibility change, compare remote
   refs/surfaces to this inventory and scan intervening commits/assets. Changes to
   GitHub visibility expose Actions history/logs as well as source; the zero counts
   above are a dated observation. [GitHub visibility documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility)
4. Preserve the existing distribution repository, versioned downloads and alpha
   updater URL. The source implementation commit is
   `9f89830be760338c74a2f0839a25e2cd1980faad`; `bdb040e` only adds two documentation
   changes. GitHub's alpha.13 setup metadata matches the recorded SHA-256
   `cd6002efcc0c4b1e38a946be75f0fce014a5f1f183c30cdec9f100776bab5d51`.
   The distribution release tag targets its README-only repository, so retain this
   explicit source mapping. [Public alpha.13 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.13)
5. If a later scan finds restricted private history, a clean snapshot is the fallback:
   preserve private history, export only the reviewed source tree, document its
   originating commit/tree and every omission/change, carry licenses/lockfiles and
   build instructions, and choose its public host with Christopher. No new public
   repository name or snapshot has been assumed or created.

SignPath admission, full dependency notice generation, exact Windows build provenance,
future Authenticode integration and live acceptance remain separate work. A paid
signing route can retain UnRAR; switching only its wrapper cannot satisfy the
all-components license rule. Future signing order is CEBG executable(s), packaged
setup Authenticode, then Tauri updater signature/checksums over final unchanged setup
bytes. Do not sign third-party mod binaries as CEBG.

## Verification limits

No app build, test suite, installer launch, updater apply/restart, or game acceptance
was run for this documentation audit. The offline Cargo dependency-graph query did
not compile or download packages. Documentation whitespace/link and PowerShell
snippet parsing checks passed. No new runtime success is inferred.

Unreachable/deleted server objects not advertised by Git, deleted external uploads,
unlinked account-level packages, third-party hosted mod payloads, every upstream
source file's licensing, and future changes are outside this bounded audit. These
are not demonstrated publication defects. Existing binary payloads were not newly
unpacked or cryptographically reverified; the reported public hash came from GitHub
metadata and matches the existing acceptance record.

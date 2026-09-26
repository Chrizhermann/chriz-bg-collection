# CEBG public-source readiness and signing audit

## User direction

Christopher explicitly does not mind CEBG's installer source being public and wants
a focused review before publication. There is no technical requirement for a private
installer repository. Legacy project guidance allowed privately archived dead-link
mods; that permission is not permission to redistribute them publicly.

Audit and prepare a publication recommendation. Do not flip repository visibility,
publish a source snapshot, rewrite history, force-push, enroll in a signing service,
send applications/email, buy anything, or rotate credentials in this task without
the corresponding explicit authorization. Public-source readiness and eligibility
for free signing are separate conclusions.

## Exact starting point

- Source repository: `Chrizhermann/chriz-bg-collection`, currently PRIVATE, MIT license.
- Active source branch: `codex/installer-v0-real-alpha`; inspected HEAD `bdb040e`.
- Released app implementation commit: `9f89830`, app **0.1.0-alpha.13**.
- Current recipe: **0.1.0-alpha.12**, 434 recommended components; unchanged by app13.
- Public distribution repository: `Chrizhermann/chriz-easy-bg`. Its current source
  tree contains only README.md; public release downloads are not public source.
- Existing coordinating worktree:
  `C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
- This handover is initially a local file in that worktree; read it by absolute path
  if it has not yet been committed into your checkout.

Use a separate git worktree/`codex/` branch based on the verified active source branch
or its exact commit, NOT an assumed up-to-date `main`. Read applicable AGENTS.md,
CLAUDE.md and docs/handover.md. Use cross-agent-context for current guidance, not
old generated memory as authority. Check `gh auth status` before every gh operation;
this project uses Chrizhermann, not the work account.

## Preserve the active test and release workflow

Christopher reports a full alpha.13 UI installation is running. The suggested path
was `C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907`; its actual use/completion
has not been independently checked. Do not touch the running app, installer test,
game sources, archives, downloads/cache, receipts or saves. No desktop takeover,
install/reinstall, runtime replacement, updater-feed edit, or another full game run.
All `C:\Games` references and existing stream installs remain read-only. Run Cargo
only from PowerShell if necessary; avoid costly builds while the full install runs.

Current coordinator-owned work is shortcut-name collision/error reporting and setup
preference persistence. Do not overlap those implementation files or change mod
curation/defaults/pins as part of this audit. Current worktree also contains uncommitted
proficiency research docs and generated Python caches; preserve them, do not publish
untracked files merely because they are present.

## Bounded audit

1. **Secrets and privacy:** scan the prospective public tree AND all branches/tags/
   reachable history that a visibility change would expose. Include repository-owned
   release assets/LFS/submodules and other public surfaces if applicable. Redact scanner
   output: report categories and paths/commit IDs, never credential values or private
   correspondence. Distinguish real private signing keys/tokens from the intentionally
   public Tauri updater key, hashes, and ordinary test values. Report any needed rotation
   without doing it. Incidental local usernames/paths are review items, not automatically
   grounds for history rewriting.
2. **Redistribution/provenance:** verify that no actual game/mod payload, private archive,
   tester save or diagnostic bundle is being published without permission. Check fixtures
   are synthetic or otherwise redistributable; filenames alone prove neither. Existing
   mod URLs, component metadata and hashes are not equivalent to bundling the mods.
3. **Licenses and signing eligibility:** inventory production dependencies and bundled
   resources, license notices, build scripts and signer requirements. Preserve the
   recipe-not-mods architecture. Do not remove working extraction support to pass an
   eligibility check. Keep source-publication findings separate from SignPath findings.
4. **Public usability/build provenance:** fix stale release descriptions in a scoped docs
   patch in your worktree, or provide the exact proposed edits. Document supported build
   steps without requiring Christopher's private paths/assets/secrets. Explain how public
   source corresponds to released binaries, and what CI/signing integration remains.
   This is not an exhaustive application-security audit or a demand to review every line.
5. **Publication strategy:** prefer the existing repository if all exposed content/history
   is suitable. If not, recommend a clean, provenance-documented public source snapshot
   while preserving private history. Do not assume which public repo should host it or
   rename existing repos. Preserve all live release/download/updater URLs.

Start with automated redacted checks and concrete high-risk files. Classify findings
as publication blockers, signing-only blockers, small cleanup or acceptable metadata.
Stop once there is an actionable answer; do not grow this into a general refactor.

## Initial inventory: leads, not an audit result

The inspected current tracked tree has 653 files. A filenames-only inventory found:

- `engine/tests/fixtures/games/content/`: 26 files, including four `Baldur.exe`,
  four `chitin.key`, four `data/base.bif`, eight `dialog.tlk`, four `engine.lua`,
  and two `dlc/sod-dlc.zip`. Verify synthetic provenance before flagging these.
- `engine/tests/fixtures/weidu/`: 11 .log fixtures; inspect provenance and privacy.
- `recipes/creator-full-current/`: 122 files, including
  `artifacts/creator-full-private-extras-20260902.toml`, `reference/source-bg1.tsv`
  and `reference/source-bg2.tsv`. Check creator-only references/content.
- `manifest/install-order.tsv`, `manifest/mod-sources.tsv`, `manifest/reference/`,
  `manifest/evidence/`, `manifest/game-builds/`: captured metadata/provenance.
- Community incident/acceptance reports, `docs/overnight-2026-09-05.md`,
  `docs/research/2026-08-19-installer-app/local-modding-guide.md`, and handovers:
  check private context without deleting useful engineering history indiscriminately.
- Root README still describes alpha.1 as unpublished. Follow current handover/release
  evidence instead; `docs/releases/public-README.md` targets binary distribution.

No other tracked archive/executable payloads were identified by that filename scan.
No content secret scan or history audit has been performed by the coordinator.

## Signing-specific finding already known

`engine/Cargo.toml` pins `unrar_sys = "=0.5.8"`. THIRD_PARTY_NOTICES.md states the
installer links UnRAR extraction source and reproduces its prohibition on developing
a RAR-compatible compression algorithm. The binding's MIT/Apache license does not
erase the underlying UnRAR restriction. Review whether this prevents SignPath
Foundation eligibility; public release of CEBG's own MIT source does not automatically
require removing that dependency.

[SignPath terms](https://signpath.org/terms.html) require OSI-approved open-source
licensing, verifiable source-to-binary builds, project review and approval of signing
requests. The certificate uses SignPath Foundation as publisher. Admission is not
guaranteed; do not state that source publication alone makes us eligible, or claim
sponsorship before acceptance. Our Tauri update signature remains necessary and is
different from Windows Authenticode. Do not sign third-party mod binaries as CEBG.

For future pipeline design: sign CEBG-owned executable(s), package and Authenticode-sign
the final setup, then produce the Tauri update signature and checksums over the final
unchanged setup bytes. Verify exact public bytes. No existing released artifact or
updater key/feed should be modified by this audit.

## Deliverables

- Short verdict: safe to publish as-is / publish after listed cleanup / recommend a
  clean public snapshot. State exactly what was scanned and what remains unverified.
- Redacted findings with source locations and small remediation proposals.
- Separate SignPath verdict, including the UnRAR question and any minimum changes;
  offer alternatives if the free path is unsuitable rather than breaking features.
- A concise handback to the installer task. No public release or claim of exhaustive
  security approval. Any proposed source/docs changes belong only in your own worktree.

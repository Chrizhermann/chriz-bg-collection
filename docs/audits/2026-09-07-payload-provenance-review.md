# Current-tree payload and provenance review

Audit date: 2026-09-07. Baseline: `bdb040e0d0ab7c63eac260497f3b828116fcb6aa`.
This is the bounded current-tree part of the public-source audit, not clearance of
all Git history, remote repository surfaces, dependency licenses, or the application
as a whole. Those are separate parts of the coordinating review.

## Verdict

The inspected current tree provides no payload or privacy reason to require a clean
source snapshot. It contains installer code, recipe/provenance metadata, engineering
notes, and synthetic test fixtures. The apparent game executables and DLC archives
are tiny text markers. The private creator archive is described, not included.

Retain useful curation and captured-order evidence. A visibility change still depends
on the separate all-ref/history and remote-surface review. Small documentation cleanup
is appropriate before publication; no running installation or release needs changing.

## Scope and method

- Inventoried all 653 tracked paths at the baseline, including sizes and extensions;
  checked all file bytes for NULs and examined every game-shaped fixture's full bytes.
  This is a structural/payload inventory, not a malware or secret-detector guarantee.
- Read all 26 files in `engine/tests/fixtures/games/content/`, all 11 WeiDU log
  fixtures, both tracked TP2 fixtures and their text/TRA companions, the Steam discovery
  fixtures, and the fake-game/RAR fixture builders.
- Inspected the 122-file `recipes/creator-full-current/` inventory, its manual
  artifact contract, source-reference TSVs and supporting generator/documentation.
  Checked the reference/evidence/game-build metadata and packaged resource allowlist.
- Reviewed the community SoD/EET incident notes, their acceptance reports, the overnight
  handover, local modding/archive summary and Evandra acquisition handoff. A targeted
  user-path scan of tracked Markdown/TSV/TOML/JSON found 47 maintainer-user path
  occurrences and no other-user path occurrences. This scan is deliberately narrower
  than a general personal-data detector.
- Read the SVG; parsed all six embedded PNG directory entries in the ICO and visually
  inspected its largest image. Reviewed the introducing commits for fixture/icon
  provenance. No external game/archive/cache files, diagnostic exports, installer
  processes or GitHub APIs were accessed. No build or live test was run.

## Findings

| Classification | Evidence at baseline | Result and proposed handling |
| --- | --- | --- |
| Acceptable synthetic fixtures | `engine/tests/fixtures/games/content/`; introduced by `ee3610d` | All 26 files total **400 bytes**, individual sizes 12–22 bytes. The four `.exe` files contain storefront/game text markers, not PE/MZ headers. The two `.zip` files contain analogous DLC markers, not ZIP signatures. KEY/BIF/TLK names also contain short markers, and four `engine.lua` files contain one ordinary assignment. No proprietary game data is in this set. Retain; document the distinction for future reviews. |
| Acceptable synthetic logs | `engine/tests/fixtures/weidu/before.log:1`, `complete.log:1`, the nine other logs; introduced by `1ff6d6b` | Eleven short files model success/failure/removal/order outcomes. Full content inspection found synthetic component labels and mod-relative identities, with no tester path, save, chat or diagnostic bundle. Retain. |
| Acceptable authored test mods | `engine/tests/fixtures/archives/simple/mod/setup-mod.tp2:1`; `engine/tests/fixtures/testmod/setup-testmod.tp2:1`; companion `.txt`/`.tra` files | Explicit integration-test authorship and controlled copy/prompt/failure operations, not bundled third-party mods. The EET-style path case is a small synthetic compatibility model. `engine/tests/fixtures/session/truncated-record.tmp:1` is a single opening brace for malformed-input testing. Retain. |
| Acceptable generated fixtures | `engine/tests/support/fakegame.rs:1`, `:117`, `:376`; `engine/tests/rar_archive.rs:33` | The fake game constructs minimal format headers, a zero-filled area stub, two minimal IDS stubs and an empty TLK string. RAR tests construct stored archive records in memory with trivial text. These are source-generated fixture bytes, not captured game/mod assets. |
| Acceptable private-archive metadata | `recipes/creator-full-current/artifacts/creator-full-private-extras-20260902.toml:8`, `:30`; `docs/creator-full-install.md:28`, `:37` | The contract records a manually supplied **1,263,540,769-byte** archive's filename, size, hash, roots and explicit nonredistribution provenance. The archive is absent from the tracked tree. Publication of the contract does not transfer the archive bytes or grant permission to distribute them. Retain historical metadata; never add the referenced archive to source/releases. |
| Acceptable captured metadata | `recipes/creator-full-current/reference/source-bg1.tsv:1`, `source-bg2.tsv:1`; `manifest/install-order.tsv:1`; `manifest/mod-sources.tsv:1`; `manifest/reference/`; `manifest/evidence/`; `manifest/game-builds/` | These are order/component identities, labels, source URLs, filenames, versions and hashes. They are neither executable payloads nor a copy of game resources. The archived creator recipe remains explicitly historical/quarantined in `docs/creator-full-install.md:3` and `tools/creator_full_recipe.py:27`. Do not hand-edit captured order or strip curation to make publication look cleaner. |
| Acceptable packaged resources | `app/src-tauri/tauri.conf.json:35` | The declared resources include application notices, manifest metadata and the curated recipe. They exclude creator/reference source TSVs and the private creator recipe. This config inspection does not independently unpack or attest an existing setup executable. |
| Acceptable icon/system fonts, small provenance improvement | `app/src-tauri/app-icon.svg:1`; `app/src-tauri/icons/icon.ico`; introduced together by `e391fa2`; `app/src/styles.css:19` | The SVG is seven lines of geometric shapes, with no external image reference. The ICO is the only NUL-containing tracked file (7,594 bytes) and contains six PNG sizes depicting the same book/plus mark. No game art, bundled font, sound, video or portrait file was found. Root MIT covers project-authored material. A short icon-origin/regeneration note would improve reproducibility; the exact original conversion command is not recorded here. |
| Acceptable technical community evidence | `docs/issues/sod900-community-recovery-2026-09-06.md:5`, `:18`; `docs/patch-acceptance-alpha13-2026-09-07.md:5` | These notes preserve component counts, relevant resource identifiers, exit outcomes and diagnosis, while omitting raw diagnostic bundles and tester identities. The source text supports technical summaries, not a full installation or privacy attestation. Retain the useful engineering findings. |
| Small privacy/usability cleanup | `docs/handoffs/2026-09-06-evandra-public-acquisition.md:82`; introduced in `aa44060` | Lines 82–94 contain an explicitly unsent proposed outbound message: 815 characters, six quote lines, no email address. It is not evidence of received private correspondence or author permission. Remove the proposed wording from the publication tip and retain a factual note that an official stable acquisition endpoint remains desirable and no message was sent. Do not send the draft as part of this audit. |
| Acceptable local operational metadata | `docs/overnight-2026-09-05.md:14`, `:166`; `docs/research/2026-08-19-installer-app/local-modding-guide.md:3` | Maintainer paths and an outside-repository private-key location are present, but these locations are not credentials. The local-modding guide is a procedure/archive-name summary, not the archive. Path generalization is optional usability/privacy cleanup, not grounds by itself for history rewriting. Preserve precise incident evidence where useful. |
| Small stale-context cleanup | `docs/research/2026-08-19-installer-app/local-modding-guide.md:7`, `:10`, `:47` | Historical counts, ordering guidance and local-only acquisition assumptions are superseded by the current handover/curated recipe. Add a historical-research banner that points readers to current entry points; do not rewrite the captured source claims as though they were newly verified. |
| Attribution review item | `engine/src/weidu/eet_compat.rs:71`; `manifest/artifacts/eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d.toml:35` | Production code embeds a three-line upstream Windows batch match string and a short corrected replacement. The artifact provenance labels original EET code GPL-3.0. The full EET macro file/archive is not tracked. Preserve the exact upstream reference and have the license review explicitly account for this small functional excerpt; do not claim every byte of the installer tree was independently authored. This is distinct from bundling an EET mod payload. |

## Exact minimal documentation proposals

1. Replace `docs/handoffs/2026-09-06-evandra-public-acquisition.md:82–94` with:

   > The approved manual Windows download/skip route above remains available. A stable,
   > author/host-supported automatic endpoint is still a future improvement. No request
   > was sent and no hosting or redistribution permission is implied.

   Change the outdated "unsent G3 request" link label in `docs/handover.md:176` to
   "Evandra public acquisition checkpoint". Retain the target and diagnosis.

2. After the title of
   `docs/research/2026-08-19-installer-app/local-modding-guide.md`, add:

   > Historical research snapshot from 2026-08-19. Counts, ordering statements and
   > local-only acquisition assumptions below describe that source material and are
   > superseded where the current handover and curated recipe differ. Start with
   > the current handover (`docs/handover.md`) for supported installation/build work.

3. Record the fixture-byte and geometric-icon provenance above in the public audit/build
   documentation. Retain fixtures and the icon; no game/mod archive removal is needed
   from this baseline tree.

These proposals are restricted to documentation. No source/curation/pin, archive,
installer, updater, signing key or release was changed by this component of the audit.

## Limits

The evidence rules out the concrete apparent game/archive payload leads in this tree.
It does not prove that no secret or restricted material exists anywhere in all refs,
GitHub issues/assets/actions, LFS, submodules or abandoned history; use the coordinating
scan for those conclusions. The upstream license review, private-history suitability,
build reproducibility and signing-service eligibility remain separate conclusions.
No legal opinion, exhaustive application-security review, or new runtime acceptance
is claimed.

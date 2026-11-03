# Alpha.18 candidate — September 20

Source candidate: app **0.1.0-alpha.18**, collection **0.1.0-alpha.16**.
Christopher approved including the completed, reviewed Modpack fixes before the
next website release. This record supersedes alpha.17. The Windows package and
versioned release were published on September 20 (local time); see the
[package/publication record](package-acceptance-alpha18-2026-09-20.md).

## Included changes

- Pin the published **chriz-bg-modpack v0.2.0-alpha.7** release. This addresses the
  reported Mazzy 140, Xan 170 and Yeslick/Keldorn Dispel 410 installer failures,
  plus related Fade 110, Skie 160/195, Sarah 190 and Hexxat 222 compatibility fixes.
  Only already-selected/offered components participate: no new choices, defaults,
  dependency changes or order changes were introduced for this pin update.
- Retain the alpha.17 SoD v0.6.11 fix, download retries, usable technical-log and
  category disclosures, scroll sizing and confirmed managed-install removal.
- Include source commit `58fb956` in the next build: WeiDU installation and probe
  processes use Windows `CREATE_NO_WINDOW`, avoiding console focus interruptions.
- Do not include the newly proposed Baeloth rebalance, broader My installs
  redesign, Randomiser ideas, wiki or existing-game hotpatching.

## Exact source and verification

- Release: <https://github.com/Chrizhermann/chriz-bg-modpack/releases/tag/v0.2.0-alpha.7>.
- ZIP: `chriz-bg-modpack-v0.2.0-alpha.7.zip`, **1,367,203 bytes**.
- SHA-256: `f134085e8220a4222190981f173034653efe7aa92d47caa8430682a1352d1c19`.
- The release is published, non-draft and GitHub Latest. Its 45-entry ZIP carries
  matching TP2/VERSION markers and standalone WeiDU 249 AMD64 setup EXE.
- CEBG's actual artifact verification downloaded the official ZIP, verified its
  digest and successfully extracted the configured roots under the unchanged
  archive limits. The standalone setup EXE is not executed by CEBG.
- The owning task reports the completed Opus-5 xhigh review had no blockers,
  PR #6 was merged, and installer/rollback/repeat-application/uninstall/package
  checks passed. Native gameplay and the reporter's exact installation were not
  tested by that release; do not upgrade those claims to live acceptance.
- **26 focused collection recipe tests pass**, including a regression requiring
  alpha.7 and preserving the **448-component / 50-run** recommended selection.
- Public-alpha validation has no findings. Actual engine planning retains the
  same five previously documented conditional/default omissions.
- Generated collection selections, preset, order and the historical alpha.15
  ledger are unchanged. Historical ledger bytes are regression-tested.
- Source integration commit: `5a5ed2b`; generated recipe evidence points to its
  full commit identity, not the earlier alpha.17 source.

## Existing installation and publication

The user's alpha.17 / collection alpha.15 receipt was checked on September 19:
it reports success and an exact frozen stack match across all 50 planned runs,
including BuffBot. Reuse that full-install evidence; do not start another full
installation merely for this reviewed modpack update. It is not evidence that
the new alpha.7 payload was installed or playtested there.

Existing games and saves remain untouched. The collection ledger labels this
update new-game-only; no automatic conversion of saved actors is implemented.

Published: [app alpha.18](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.18),
including collection alpha.16. All seven public assets were downloaded anonymously
and compared with the local release bytes; the public setup's updater signature
passes. The alpha channel now hosts the matching feed, with initial CDN caching
noted in the package record. The website task received the verified publication
handoff. The alpha.18 setup is also in Downloads; older setups must not be relabelled.

# Public-source/signing audit handback

Audited active source `bdb040e` in separate branch
`codex/public-source-signing-audit-20260907`. App alpha.13 source is `9f89830`;
recipe remains alpha.12 / 434 recommended components.

**Recommendation: use the existing source repository after scoped docs cleanup
and the small EET excerpt attribution/license disposition.** All 653 tracked files
and all three remote branches' history (235 commits / 1,631 blob versions) were
scanned. No credential, actual game/mod archive, tester save or raw diagnostic
bundle was found. The apparent game/ZIP fixtures are tiny synthetic markers.
Remote tags/releases/assets/Actions artifacts and linked Packages are absent.
Maintainer paths/email and useful engineering history do not warrant a clean snapshot.

Prepared: current README, portable Windows BUILDING guide, historical signing/status
banners, removal of the unsent request from the publication tip, and redacted reports.
Before publishing, use a current source landing branch: remote default `main` is old.
Refresh refs/surfaces and scan intervening content before the separately authorized
visibility action. Full details: [publication audit](../audits/2026-09-07-public-source-readiness.md).

**Free SignPath signing is not established:** static UnRAR conflicts with the
published OSI-only component rule. Retain extraction. A paid Authenticode route,
an authorized service determination, or a separately validated alternative decoder
are future options. Broader binary dependency notices and verifiable CI/signing
remain release work; [signing review](../audits/2026-09-07-signing-dependency-review.md)
records the inventory and exact EET attribution proposal. A clean snapshot would
not fix these source/dependency questions.

No visibility change, push, publication, history rewrite, signing enrollment,
credential rotation, external message, live updater/download change or game run
occurred. The running alpha.13 installation was not inspected or interrupted.
No application implementation/default/pin was changed. No app build/tests were run;
checks were limited to scanning, metadata/dependency reads and documentation checks.

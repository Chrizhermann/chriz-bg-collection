# First existing-install patch pilot — September 20

Result: the supervised Artisan description-link patch **applied, detected an
already-fixed repeat without launching WeiDU, and rolled back successfully** on
the explicitly selected old RC. It was then reapplied in a separate transaction,
and **Christopher accepted the native test**: descriptions worked and a loaded
existing save behaved as expected. This is acceptance of the six-link repair,
not newer Assassin mechanics, every mod, or a public hotpatch release.

## Scope and backup

- Target: `C:\BG-EET-RC-20260903\game`, BG2EE 2.7.3.0, English.
- Current stream installation and `C:\Games` were not touched.
- The first apply/rollback cycle made no game launch, profile selection change
  or save edit. The second cycle's separate isolated-profile harness and subsequent
  user-run native acceptance are recorded below; original saves were not edited.
- Full independent backup:
  `C:\Users\chris\Games\CEBG-Backups\RC-20260903-before-kit-links-20260920`.
  Its `game` child contains 166,410 files / 12,099,587,163 bytes (~11.27 GiB).
  Copy reported zero failures; the complete relative-path/size/mtime manifest
  matched the source before patching, and protected/affected files were hash-checked.
- The separate `profile` child contains all 786 source profile files (~175.69 MiB),
  individually hash-compared against the original. The backup is outside OneDrive;
  originals were read only. Keep the backup until Christopher chooses to remove it.

The legacy replay receipt recorded 386 rows. Per-run post-receipt reconstruction
matched the current first 386 rows; eight later soundset/progression/paladin rows
form an append-only tail. The receipt itself does not contain a full standalone
old log, so the reconstruction is not claimed as independent evidence for every
original row. The pilot imports the actually observed 394-row baseline instead.

## What was applied

Owner adapter: `AKCB_KIT_DESCRIPTIONS` component 0, version 1.0, committed with
Artisan release source `d16d35ba29c68aa18025a1c5323a8973986a97d9`.
CEBG stores only its identity/validation metadata; the adapter remains in its
own mod repository. The pilot permits installed Artisan `chriz-v1.3.0` only.

| Campaign tables | Row | Old reference | Existing local reference |
|---|---|---:|---:|
| BGCLATXT and SODCLTXT | ARCHER | 224298 | 326946 |
| BGCLATXT and SODCLTXT | BEAST_MASTER | 224300 | 25212 |
| BGCLATXT and SODCLTXT | ASSASSIN | 224301 | 25213 |

Exactly these six description cells changed. No kit ability, spell, creature or
saved-game update is implied. All other table tokens matched the precomputed
postconditions. CLASTEXT, KITLIST, engine identity, KEY and all 20 detected TLKs
remained byte-identical. All 394 prior component entries (including their comments)
were preserved, followed by the single patch row.

The complete game-tree metadata diff found no changes outside the two description
tables, WeiDU.log, normal weidu.conf rewrite and the staged adapter/backup folder.
The configuration's bytes remained unchanged. The external verified WeiDU executable
was used directly; no root setup executable was overwritten. No quick-log or
auto-update was used. The process ran without a console window.

## Transaction and checks

Supervised entry points:

- `tools/hotpatch_pilot.py`: read-only component/version, table shape, local string
  bounds, adapter identity and semantic postcondition checks.
- `tools/run_hotpatch_pilot.py`: exact-RC-only application, full-backup prerequisite,
  process/exclusive-file checks, state lock, immutable hash-chained events, scoped
  restoration and guarded latest-patch rollback. This is pilot tooling, not the
  public app's general executor. A new pilot cycle requires a deliberately prepared
  new transaction; completed rollback evidence is not overwritten.

Private state and diagnostics remain outside the game at:
`C:\BG-EET-RC-20260903\.chriz\patches\kit-links-pilot-v1`.
The observed sequence is `prepared → applied → rolled_back`. Failure records
would remain separate from success; undeclared changes block an automatic claim
of restoration. Source controls do not prevent users starting the game EXE directly.

Focused tests: **20 passed** (12 eligibility/semantic cases, seven transaction/path
cases, one real-WeiDU synthetic-game integration case). Coverage includes unrelated
cell preservation, missing/unknown component versions, invalid references, duplicate
rows, malformed inputs, already-fixed eligibility, journal-chain tampering, hardlinks,
simulated partial-write restoration, deleted declared outputs and undeclared writes.
The real-WeiDU fixture applied/uninstalled the owner adapter and compared table/TLK
bytes. Its synthetic historic comments are not a source of version provenance;
component identities/order are compared, and eligibility uses its recorded baseline.

On the real RC, repeat application returned `already_fixed` with `weidu_started=false`.
Rollback removed only newly staged pilot adapter files, restored the original
description tables/config/log, matched the complete pre-patch metadata manifest and
rechecked all recorded protected-file hashes. Original WeiDU.log SHA256 restored:
`37a828a8a628abe9acbc26cd6062f005e1d7317e772373e895179665067435d4`.
No new installation, download of the mod stack or cascade reinstall was performed.

## Subsequent production source integration — September 20

The separate Rust/Tauri/Updates flow is now **implemented in source**, including
the signed patch catalog, per-install eligibility, backup choices/space reporting,
progress, recovery and Apply/Undo. See the
[production-flow acceptance record](existing-install-patches-acceptance-2026-09-20.md)
for its exact supported scope and remaining delivery checks. The pilot tooling
and its historical transactions remain separate evidence.

Verification: 25 engine patch tests (24 ordinary plus one public-download/native
WeiDU flow), 15 native-app tests, and 176 frontend tests passed; frontend typecheck
and build also passed. The public-flow test downloaded the pinned LF adapter and
official WeiDU, created independent game/profile backups, applied using the hidden
production runner to a **synthetic game**, checked semantic/protected hashes,
recognized a repeat without another transaction, and undid back to originals.
The base receipt and save were unchanged. This is not a new real-game UI playtest:
the RC, stream installation and original saves were not written in this source
integration turn.

## Still pending — do not advertise these as delivered

1. Public packaging/release of a supported patch. Alpha.18 app/recipe channels are
   unchanged by this work. No public hotpatch button exists yet.
2. Native end-user interaction with the packaged Updates flow is not established
   by the synthetic game and frontend/native-app tests above.
3. Broader component coverage and a separately audited local-text adapter. No TLK
   writes, saved-actor migrations or automatic save editing belong to this pilot.

The [accepted design](plans/2026-09-20-existing-install-patch-pilot.md) records the
three user-facing categories, backup requirements and the completed Opus 5 xhigh
design review. That review was not a review of the subsequent implementation.

## Reapplied for Christopher's native test — September 20

At Christopher's request, a separate transaction at
`C:\BG-EET-RC-20260903\.chriz\patches\kit-links-native-test-20260920`
reapplied the same six cells successfully. The previous cycle's immutable rollback
record remains intact and is linked by hash. Existing guards rechecked the full
backup against the restored game, source/tool identities, closed-game condition,
exact table changes, component suffix and protected hashes including all 20 TLKs.
No full reinstall or new game backup was necessary.

After successful patch verification, a separate test-harness step changed only
the RC's `engine_name` to
`Baldur's Gate - Enhanced Edition Trilogy - CEBG Patch Test 20260920`.
The matching profile under the user's actual Windows Documents folder contains
786 files copied from the independent profile backup, all hash-verified. Original
profile/save files were not edited. The original engine file and before/after
hashes are retained in the transaction's `harness` directory.

Launch `C:\BG-EET-RC-20260903\game\InfinityLoader.exe` for this test. In BG1
character creation, inspect Archer (Rapid Shot), Beast Master (Beast Friend), and
Assassin (Cloak of Shadows) descriptions; compare BG2 if desired. Load a copied
save, inspect party/record screens, save to a new slot and reload. This tests
description routing and basic existing-save loading, not kit ability behavior or
every mod. This was the requested checklist, not evidence that every individual
step was performed; Christopher's actual report is recorded below.

The engine/profile switch is **not part of the patch itself**. Restore the backed-up
engine identity with the game closed before transaction audit/undo. Gameplay may
produce additional runtime files; assess those rather than bypassing rollback's
drift protection. Keep both the full backup and test saves pending cleanup approval.

## User live acceptance — September 20

Christopher reported testing intensely and confirmed that the applied repair
worked. After clarification that the older passive Cloak of Shadows text was
expected for the RC's installed kit version, he confirmed: “everything seems to
be working. Even the loaded save worked as expected.”

Record the description-link repair and existing-save loading/observed behavior
as **user accepted**. A separate new-slot save/reload was suggested but was not
explicitly reported; do not claim that specific sequence, exhaustive campaign
coverage, or the newer activated Assassin rework was tested by this pilot.
The patch reused existing installation-local description references; all 20 TLKs
were byte-identical in the pre/post application checks. User acceptance does not
extend that write scope to mechanics, new text or saved-character migration.

Christopher subsequently approved connecting this controlled process to Updates.
Other deferred work is indexed in
[the next-work index](plans/2026-09-20-next-work-index.md); it is not implicitly
part of the first public patch implementation.

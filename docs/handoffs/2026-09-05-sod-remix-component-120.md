# CHRIZ-SOD-REMIX 120 blocks full curated installation

## Local fix candidate received (2026-09-05)

The owning task reports a fix on `codex/fix-sod120-compat-v0.6.5`, commit
`3b21d6f0ed023551ceaa209479f31d8264b7ef3f`. It traced the former state 4 to the
Aura-expanded dev dialogue. Christopher clarified in that task that Aura is removed and
Aura compatibility is not required. This supersedes the original request below to support
both layouts: the candidate intentionally validates the observed no-Aura four-state graph.
Components 120/225 locate their three picker routes through semantic local flags and
reject an unexpected graph; the BDIMOEN 67.1 hook remains covered.

Owning-task evidence: WeiDU 249 parses TP2/all TPA files; 11 tests pass, including
BDIMOEN fixtures, portable WeiDU discovery, and the 120/220/225 contract. Its read-only
scrying verifier on `C:\BG-EET-RC-20260903\game` also reports zero failures. **That
read-only check is not evidence that the new candidate was installed or played.** The
collection requested explicit separation of fresh-fix testing from existing-copy baseline
checks, plus Christopher's exact remaining live-test steps. No live acceptance is claimed.

Executable-bearing local release candidate:
`C:\Users\chris\Documents\Codex\2026-09-05\sod-remix-v0.6.5-rc-3b21d6f-r2\chriz-sod-remix-v0.6.5.zip`.
SHA-256 `1113b9e6fd0f2929fce187a1a693b253b30b34da80c48850c9741333847d891e`
was independently checked by the collection task. The owning task reports 106 packaged
files with extraction/manifest/hash validation. Nothing is pushed/tagged/released; do not
change the collection pin to a nonexistent release. R3 remains frozen and non-resumable;
all approved component selections remain unchanged, with component 290 still separate.

## Confirmed failure

The collection's third corrected full acceptance run passed SCS, Randomiser (including
the approved compatibility answer), EET_END, and the late Bardic dialogue patch. It then
failed in CHRIZ-SOD-REMIX `v0.6.4` component **120**. Dependent component **225** was
skipped because 120 did not install. Do not drop either: the approved SoD Remix selection
is the whole 30-component bundle.

Error from the actual attempt:

```text
WARNING: internal label [4] not found in processed DLG [BDSCRY]
ERROR: processing .D actions [chriz-sod-remix/dlg/csrhood.d]: Invalid_argument("index out of bounds")
ERROR Installing [SoD remix: remove the hooded man from the mid-campaign], rolling back to previous state
```

`chriz-sod-remix/dlg/csrhood.d` unconditionally applies:

```text
ADD_TRANS_TRIGGER BDSCRY 0 ~False()~ DO 2
ADD_TRANS_TRIGGER BDSCRY 4 ~False()~ DO 2
ADD_TRANS_TRIGGER BDIMOEN 67 ~False()~ DO 1
```

The failed copy's restored `game/override/BDSCRY.dlg` is DLG V1.0, **943 bytes**, with
**four states** (header state count at offset 0x08). State 4 therefore does not exist.
The process returned code 3 despite the component failure and missing dependent component;
the collection correctly rejected the incomplete WeiDU suffix. Do not equate this exit
code alone with harmless warnings.

The initial failure evidence established the immediate source assumption mismatch, not
which transformation created it. The owning task's later Aura finding is recorded above;
do not attribute the difference to EET_END.

## Exact reproduction evidence (read-only)

- Managed root: `C:\Users\chris\Games\CEBG-Curated-20260905-r3`
- Install ID: `install-c117933c9f2de8edc020`
- Frozen recipe: alpha.6, SHA-256
  `e5ced746536e989e7482834b0f2bb40e18bc1fba595ac13a7e27112c96c2563a`
- Ledger terminal record: `.chriz/ledger/0000000216.json`, `fresh_copy_required`
- Attempt evidence:
  `.chriz/attempts/attempt-490fa4d27e68192a1f98/steps/0107-c656b8d09bdc1ad9/attempt-0001/`
  includes invocation, before/after logs, process result, full output and WeiDU debug.
- Separate sanitized diagnostics: collection worktree
  `target/curated-sod120-failure-20260905/` (553 files).
- Exact archive: `chriz-sod-remix-v0.6.4.zip`, 1,459,461 bytes, SHA-256
  `560168af4de06aaa17419213801447863be58f3f74454470c314b03edad79db3`.
- GitHub's public Latest API was checked 2026-09-05: `v0.6.4`, published
  `2026-09-04T10:58:45Z`. No newer released fix was available at that check.

The approved 30-component order was correct in this attempt. This is not the previously
fixed 210-before-197 declaration issue. Remaining collection tail runs were not executed.

## Owning repository and next scope

### User decision and dispatch (2026-09-05)

Christopher explicitly rejected deferring the bundle: keep the full approved SoD Remix
selection and fix component 120 now in its owning task. He can perform the focused live
tests. The handoff was sent to task `019f6539-74ef-7260-93c3-00b63cee296a`.
It requests a bounded fix, current/older-layout fixture coverage, checks of the 120/220/225
interaction, and a short exact live-test checklist. Unrelated component-290 ending work
stays separate. No collection component selection was changed.

The fix belongs in `Chrizhermann/chriz-sod-rebalance`, not in a patched copy embedded in
the collection. At initial handoff the checkout `C:\src\private\chriz-sod-rebalance`
contained the same unguarded state references and an unrelated untracked `AGENTS.md` to preserve.
Its existing Codex task ID is `019f6539-74ef-7260-93c3-00b63cee296a`.

Original bounded work request (Aura-layout scope superseded above):

1. Reproduce 120 against a fixture copied from the failed four-state dialogue; inspect
   the actual reply/action structure. Check the related BDIMOEN and 225 assumptions too.
2. Target the intended Hooded Man hooks robustly across supported layouts. Do not merely
   skip a missing numeric state if the intended dialogue survives elsewhere.
3. Test the current layout and the older supported layout, plus the dependent 225 path.
4. Prepare the normal source-repository release, including the conventional Windows WeiDU
   executable. Confirm applicable publication authority before pushing/tagging/releasing;
   return the tested release/pin details to the collection task for acceptance continuation.
5. Only then plan the next complete acceptance attempt. The frozen failed r3 cannot resume
   with a different recipe or silently edited source. Avoid another blind full rebuild.

At initial handoff creation, no changes had been made to the owning mod repository, the
failed game's evidence, Steam sources, protected C:\Games references, stream installation,
or saves. The owning task is now implementing the authorized fix; frozen game evidence
and protected installations remain read-only.

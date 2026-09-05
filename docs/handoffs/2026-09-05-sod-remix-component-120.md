# CHRIZ-SOD-REMIX 120/225 fix acceptance and release handoff

## Published release verified; collection continuation authorized

Christopher authorized standard mod finish-off and continuation here. The owning task
published [v0.6.5](https://github.com/Chrizhermann/chriz-sod-rebalance/releases/tag/v0.6.5),
annotated tag peeling to `a2762f5da0e77d38dfd5eb10c69d51beb25c8d93`.
The final [ZIP](https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.5/chriz-sod-remix-v0.6.5.zip)
is 1,460,498 bytes with SHA-256
`e964507612730d0c44c0ea155291a1935ee8a6355cc83566be6a14f069e9d601`.
The collection independently downloaded and verified it, then compared all 106 file
entries with the accepted RC: **only COMPONENTS.md differs; no mod/WeiDU bytes changed**.

The owning task's completed fix worktree may now be removed as its own repo cleanup.
Use the [tagged acceptance record](https://github.com/Chrizhermann/chriz-sod-rebalance/blob/v0.6.5/docs/playtest/2026-09-05-scrying-pool-v065.md)
instead of the historical worktree paths below. CI's later synthetic-fixture casing and
documentation count corrections did not change the mod payload (parse scope: TP2 plus
33 TPA libraries). Full 30-component bundle retained; 290 still deferred.

Collection recipe alpha.7 now pins that verified published artifact, and the complete
fresh r4 installation has started (43 runs / 430 components, including all 30 SoD
components). App alpha.8 is packaged. See the [current acceptance record](../curated-full-acceptance-2026-09-05.md)
for exact identities/progress. Never resume or mutate frozen r3. Retain the focused test
copy/profile until the final collection cleanup is coordinated. Publication-waiting notes
below are historical and superseded by this section.

## Focused runtime acceptance received (2026-09-05)

Christopher reports the test looks good. The owning task additionally confirms his
explicit save/reload acceptance and screenshots showing the 3,000 party-total scepter
reward, the tested character's 1,000-XP omen reward, approved Caelar text, and repeated
dormant responses without a second visible XP award. Screenshots do not separately prove
one-Essence refusal, inventory consumption, or every party-slot reward; those have the
passing installed-resource verifier as evidence. Natural acquisition remains a later
playthrough check. This accepts the focused fix, not the whole collection installation.

The owning task recorded acceptance in documentation-only commit `d1300e1`; tested
implementation remains `3b21d6f` and RC hash remains `1113b9e6fd0f2929fce187a1a693b253b30b34da80c48850c9741333847d891e`.
Its detailed record is
`C:\src\private\chriz-sod-rebalance\.worktrees\sod120-compat-v0.6.5\docs\playtest\2026-09-05-scrying-pool-v065.md`.

Publication still awaits Christopher's explicit decision in the SoD task. The final
archive will refresh its packaged feature-inventory documentation without a mod-code
change, so verify the eventual published artifact and its new hash rather than assuming
the RC ZIP is the final release. Collection pins are unchanged; frozen r3 must not resume.
No rebuild or test-copy/profile cleanup starts yet. Component 290 remains separate.

## Retained focused-test setup

After native acceptance, the owning SoD task configured only the retained disposable
clone's `engine.lua` to use the unique profile name `CEBG SoD120 v065 Test`, and created
fresh finalized-EET/SoD/console configuration at
`C:\Users\chris\OneDrive\Documents\CEBG SoD120 v065 Test\Baldur.lua`.
The collection verified those paths/configuration exist. At preparation no game had been
launched and no existing saves were imported or modified. This supersedes the earlier compatible-seed
discussion: use a **new disposable SoD character** in this separate profile.

Launch only `C:\Users\chris\Games\CEBG-SOD120-v065-test\game\InfinityLoader.exe`,
with all other Baldur/InfinityLoader games closed. The exact short staging and pass/fail
instructions are in
`C:\Users\chris\Documents\Codex\2026-09-05\chriz-sod-remix-v0.6.5-rc\LIVE-TEST.md`.
They exercise one-Essence refusal, two-Essence consumption/text/XP, no cinematic/travel,
and dormant state after save/reload. Component 290 and the unfinished collection tail
are outside this test. Retain both this test game directory and its unique profile until
Christopher finishes; then they are disposable cleanup targets, never the stream profile.
Publication is ready but still awaits Christopher's explicit decision in the SoD task.

## Focused native repair acceptance PASSED (2026-09-05)

The collection created only `C:\Users\chris\Games\CEBG-SOD120-v065-test\game`, a
disposable copy of frozen r3's game directory (not its managed `.chriz` state). Robocopy
copied 161,404 files / 11.326 GiB with zero failures or mismatches; code 1 means files
were copied successfully. No Baldur/InfinityLoader process was running.

The exact v0.6.5 RC below was extracted there and only the missing components 120/225
were installed using its packaged WeiDU. **Exit 0; both components successfully installed;
no original label-4/index-out-of-bounds failure or other ERROR/WARNING in the transcript.**
WeiDU identities increased from 344 to 346 with the original 344-row prefix unchanged and
only 120/225 appended; existing 220 remains. WeiDU regenerated the older SoD rows' version
comments as v0.6.5 from the current TP2. Those comments are not evidence of reinstalling
the other 28 components: their prior v0.6.4 operations were retained.

The source-repository semantic verifier then passed against these **newly patched bytes**:
`SUMMARY: 0 failure(s)`. It covers picker gating, Essence placement/consumption, once-only
1,000 XP per player slot, and absence of dialogue/cutscene/travel/spawn actions. Its first
invocation required `game/weidu.exe`; the packaged setup executable was copied to that
alias only in the disposable clone, after which verification passed.

Evidence in the collection worktree:

- `target/sod120-v065-copy.log`
- `target/sod120-v065-install.log`
- `target/sod120-v065-before-weidu.log`
- `target/sod120-v065-verifier-with-weidu.log`

Frozen r3's WeiDU.log and BDSCRY.dlg hashes were unchanged after the check:
`93ede256c8aca44c2af9781080e6548b10a47c7957fbf8dabc3a7a5ff98ae5a2` and
`807bdb045b3f67e0fc64ac72719e6bb97a9e1b8cbd7b4c60be125bbd6c55ac67`.

This is a successful focused repair test, **not a full fresh curated installation**.
No game was launched or save modified. The clone lacks the remaining collection tail and
must not be represented as the completed stream-ready install. Keep it temporarily for
Christopher's focused pool test, then remove only this disposable test when no longer
needed. Do not load an arbitrary stream save without checking its recipe/TLK compatibility;
prefer a fresh disposable test character if a compatible seed is unavailable. The owning
task has received the exact results and is handling live-test instructions/publication
coordination. No release was published or collection pin changed here.

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
checks, plus Christopher's exact remaining live-test steps. The focused new-code native
check is now recorded above; in-game acceptance remains unclaimed.

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

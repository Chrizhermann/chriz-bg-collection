# chriz-bg-collection — Handover

Live entry point for anyone (user, future agent) picking up work on this repo.

## Existing-install patch triage — 2026-09-14

[Patch candidates and remaining tests](plans/2026-09-14-existing-install-update-triage.md)
now complement the release plan. Separate component/resource compatibility from
saved-state applicability; never infer either from WeiDU.log alone. Dedicated
description, Lightning and Abettor tails exist, while full Artisan/Bardic saved-grant
migration does not. Modpack 199 remains new-campaign/pre-EET_end work; 189/620 and
SoD 115 have specific event boundaries. Local TLK appends are possible; copying
another install's TLK/compiled string numbers is not the proposed approach.
Source/planning checks only: no game changes or generic patch executor implemented.

## Expanded next-release plan — 2026-09-13

Christopher wants the next release within the next couple of days, including
implemented mod work whose live tests or owner releases are still pending. The
[next-release scope and acceptance plan](plans/2026-09-13-next-release-scope-and-acceptance.md)
is the current planning entry point. It inventories Artisan kits/modals/descriptions,
Bardic, both SoD work lines (bridge/filler plus Khalid 115), dragons, modpack
continuity/Imoen/Safana, the retained SR Lightning implementation, BuffBot 1.8.4 and
the current CEBG draft. Planned-only work is separated for discussion.

This expands the older narrow release proposal, **not the recipe yet**. Current
draft 436/44 and historical Combined 446/50 are not the final expanded count.
Reuse Combined for applicable native tests; consolidate sources and do one final
frozen packaged-candidate install, not a rebuild for every feature. The plan flags
SR/RR's earlier deferral, Safana's public selection and Lightning's default for
discussion. No game/source-pin changes or releases were made by this planning pass.

## Challenge Mode recorded for later — 2026-09-13

Christopher requested future Challenge Mode components: SoD changes, stronger
dragons, wand-to-scroll replacements, selected-fight area locks and practical
enforcement of his run rules. The complete 16-rule screenshot inventory, new
requests, SoD filler exceptions, owner routing and unresolved policy details are
in [the Challenge Mode outlook](plans/2026-09-13-challenge-mode-outlook.md).
Documentation/preparation only: do not start implementation, change defaults or
delay the next patch on its account. Subsequent source inventory identifies
implemented optional SoD **257**, the Insane bridge sequencers, as an existing
challenge piece. Its final split still needs native acceptance; it does not mean
the full Challenge Mode rules are implemented.

## Acton Balthis no longer default — 2026-09-13

At Christopher's request, BG2 Unfinished Business **25: The Murder of Acton
Balthis** is now optional and unchecked in the draft alpha.16 / collection
alpha.14. Catalog, base manifest/preset and generated recipe agree. No other
component depends on it; all other selections and source pins remain unchanged.
The recommended plan is now **436 components / 44 runs**, retaining CDTweaks 2380.
37 focused tests pass; public-alpha validation has no findings. Opting into this
quest adds exactly `ub-bg2/25`, with no collateral selection changes.
Existing games, historical installation order and published release evidence were
not changed. This remains an unpublished next-patch change.

## Racial kit unlock default approved — 2026-09-12

Draft alpha.16 / collection alpha.14 now includes **CDTweaks 2380**, default-checked
and optional, after the main Artisan/Bardic kit additions. It fills the elf mage
specialist availability gap; internal/NPC-only kits and gnome mage policy stay
unchanged. Recommended plan: **437 components / 44 runs**. Switching it off removes
only `cdtweaks-bg2/2380` and returns 436. 36 focused tests pass; public-alpha
validation has no findings. Native WeiDU 249 fixtures using copied stream and
Combined tables pass install/uninstall with exact restoration; all 66 newly
eligible kit/race choices pass the ability-minimum checks. Completed in-engine
character creation is not yet tested. The pre-existing dwarf/Gallant CHA mismatch
is tracked separately in the community follow-up plan below. No user game/save
changes, packaging or publication.

## Optional choices and community follow-ups — 2026-09-11

Draft alpha.16 / collection alpha.14 now offers **24 additional SCS and 44
CDTweaks choices**, all unchecked. Recommended selections remain exactly unchanged:
**436 components / 44 runs**. Alternative groups have real mutual conflicts;
CDTweaks 2310/2311 remain alongside default 2312 in the late spell-scan run.
SCS 4130/4135/4140 have visible **Not recommended / experimental** cautions after
community feedback. 4130 also explains and enforces SCS's native SR exclusion.
Do not infer that source screening proves every combination works in-game.

35 focused Python tests and 12 Customize UI tests passed; public-alpha validation
has no findings; eight CLI planning cases and pinned SCS/CDTweaks native order were
checked. No game was modified, no installer test was started, and nothing was
packaged/published. Historical release acceptance records were preserved.

Christopher deferred the unresolved **Kivan/Jozzi** viewer case until he can
reproduce it; the existing component 130 guard was already shipped, not a promised
new fix for that case. Druid instant-cure availability is a future SR/BG Rebalance
balance discussion. **Automatic regeneration on rest** is a separate requested
QoL feature, researched but not implemented; owning-mod prototype scope is in
`docs/handoffs/2026-09-11-regeneration-on-rest.md`.

Next-release scope, source-screening caveats, Kivan follow-up, and the explicitly
deferred **website differences-from-vanilla comparison** are tracked in
`docs/plans/2026-09-11-optional-tweaks-and-community-followups.md`. The website list
must be discussed with Christopher/Twitch task after release, not published as an
unreviewed exhaustive claim now.

Also recorded there: Christopher's **elf Mage character-creation screenshot** shows
only Mage, Diviner, Enchanter and Wild Mage. Read-only checks of stream/combined
`K_M_E.2DA` and `mgsrcreq.2da` confirm exactly that list. Selected Artisan 1 unlocks
base classes, not the missing specialist choices. **CDTweaks 2380** is the
kit-unlock counterpart; default inclusion was subsequently approved and implemented
as recorded above. Gnome policy is separate. No game changes; not the earlier
Red Wizard/SR issue.

## Red Wizard default restored after independent review — 2026-09-10

Christopher asked another agent to investigate the Red Wizard/SR exclusion and
restore the kit as default unless a substantiated blocker was found. The review
found no basis for a blanket ban. Artisan NPC **5102** is now default-checked with
or without Spell Revisions, still optional. Other exclusions and source pins are
unchanged. Draft alpha.16 / collection alpha.14 has **436 components / 44 runs**.
This correction is not yet packaged or published.
Verification: 27 focused tests pass, public-alpha recipe validation has no findings,
and CLI plans confirm 436 components with Red Wizard / 435 without, retaining SR.

Evidence, original rule history and actionable mod-repo follow-ups are in
`docs/handoffs/2026-09-10-red-wizard-sr-default.md`. There are narrower starting-spell
and custom-kit spellbook-cleanup imperfections, plus an EET Edwin variant gap;
do not misrepresent them as a demonstrated universal SR incompatibility. Do not
silently enable the old Edwin amulet/slot patches.

Neither the stream nor Combined-20260908 has been modified: both still lack 5102.
The 446-component combined acceptance below does not cover Red Wizard. Focused
native Edwin testing and any existing-save migration remain separate work.

## Kivan quest fix default reconfirmed; already installed — 2026-09-09

Christopher requested default inclusion through the owning task **Fix Kivan Sahaguin
quest** (`01a08666-0eff-77e0-a178-bda4a232ff78`). This is already provided by the
normal hash-pinned `chriz-bg-modpack v0.2.0-alpha.5` dependency, component **130**;
do not add a duplicate standalone installer or select a stale checkout's stub.
The recipe keeps it `mandatory` when BG1NPC quest component **10** and SCS general
AI **6000** are selected, after EET import and SCS (public recipe: post-EET_end).
It is independent of Kivan's Archer choice. The dependency was tightened from
mere BG1NPC mod presence to component 10 after a regression test demonstrated that
turning quests off previously left the guard selected. The default component set
is unchanged; this authored/generated recipe correction is not yet published.

Read-only proof: Combined-20260908 `game/WeiDU.log:408` records modpack 130 once;
no standalone fix entry. Both `override/X#SAHA01.BCS` and `X#SAHA02.BCS` are 225 bytes,
SHA256 `f63e741a2b3c39eab66e34f18b3fa8830022465e2bf54c762b73267c282a45e7`, matching
the handoff's standalone verification exactly. No completed-install files, receipts,
saves, frozen recipes or source games were changed. The stale migration/stub
documentation has been corrected in `COLLECTION_TAIL_FIXES.md`.

Reference-only handoff package:
`C:\Users\chris\Documents\Codex\2026-09-09\kivan-quest-fix\KIVAN_QUEST_FIX-v1.0-verified.zip`,
verified SHA256 `7fc3769c0c18e32305c1ceefb9727b27a5331edb1165f04c786bb1c0eb035d23`.
Its `verification.json` records four passing installer cases, not new native
playtest acceptance. Do not use this local ZIP as a public dependency.
Test the quest from before the encounter: dialogue should finish before lower SCS
combat AI takes over. This prevents the interruption; it does not resurrect dead
quest actors or repair an already failed saved encounter. No new release authorized.

## Combined playtest completed and retained — 2026-09-09

**Installation accepted: 446/446 components (27 BG1 + 419 BG2).** All 391
previously completed components were preserved; no full reinstall or sibling
uninstall/replay was needed. Corrected SoD 256 and all ten remaining frozen runs
completed, including dragons, Safana 189, Imoen 620, SR 60/80, Klatu 2150 and
BuffBot 1/0. BG Radar Overlay **2.5.2.0** is also installed from official Latest.

Keep `C:\Users\chris\CEBG-Tests\Combined-20260908` for Christopher's testing.
Launch `game\InfinityLoader.exe`; Radar is
`game\BG Radar Overlay\BG Radar Overlay.exe`. The isolated save folder is
`C:\Users\chris\OneDrive\Documents\CEBG Combined Playtest 2026-09-08 - c2a832a75741`.
**Native launch/gameplay acceptance is still pending.** This is a local experimental
snapshot, not a public recipe release or a stream-install update. The source and
stream games remain untouched. No installation worker remains active.

Fresh CLI `report <root> --json` returns `ok: true`. The separately published
`.chriz/install-receipt.json` records recovery `sod-256-append-20260909`, version
`local-40712587d45b (repaired)`, exact final component order, source provenance and
attempt evidence. Receipt SHA256:
`51d670ab9fab44ca4fd6d282c4530fb352b1ebe49a6f6e2975585426ce7edcbd`.
The original failed receipt/ledger/frozen recipe were not rewritten; the original
terminal receipt SHA256 remains
`710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938`.
Keep `.chriz/recoveries/sod-256-append-20260909/` immutable: it is hash-bound evidence.
Do not rerun either installation worker or repair mode. The read-only follow-up
`cebg-overnight-install-follow-up` is now **PAUSED** after verified completion.

Focused recovery tests and example/CLI builds passed, followed by actual retained
installation, acceptance and fresh report verification. The acceptance example
uses canonical forward-slash evidence paths on Windows. Current CLI understands
the additive append-recovery receipt; public GUI compatibility with this local
recovery metadata has not been tested. The SoD owner received the successful 256
installation evidence; native bridge playtest and separate release approval remain
outstanding. Next: use the disposable checkpoints in
`docs/plans/2026-09-08-combined-playtest-and-alpha16.md`.

## Combined playtest completion authorization and execution history — 2026-09-09

Christopher explicitly requested: **"then finish the installation."** Retain
`C:\Users\chris\CEBG-Tests\Combined-20260908` and all 391 completed components.
Apply only the verified SoD 256 source fix below, append that missing component,
then execute the ten untouched frozen runs through BuffBot (446 total). Do not
rollback/replay the 34 successful SoD siblings or start a new installation.

Recovery executed in `installer-v0-real-alpha`: an incident-specific worker
and a narrowly authorized append-order recovery verifier. The original failure
receipt, ledger and recipe remain immutable; successful completion must publish
a separate recovery receipt with replacement-source and physical-order evidence.
The follow-up was active and read-only during execution; it is now paused.

**04:31 +09:00:** corrected 256 installed successfully, preserving all 364 old BG2
rows byte-for-byte (392 total including BG1). Evidence/backup is under the retained
root's `.chriz/recoveries/sod-256-append-20260909/`. `tools/recover-combined-bridge.ps1`
continues the frozen tail. A helper-only HGO executable-naming no-op was detected
with unchanged before/after logs; its evidence is retained, and the real HGO
operation uses stem `hiddengameplayoptions-bg2-attempt2`. Do not replay 256 or HGO.
The remaining runs and final recovery acceptance subsequently completed (see above).

## SoD 256 fix ready; minimal recovery rehearsal passed — 2026-09-09

The owning SoD task delivered commit `29e123a79b9f03334ab88ce93e28c300b287a8e0`
on `codex/issue-14-bridge-finale` / draft PR22. It fixes the missing review of
Artisan's MO1 equipment-policy pair, preserving the pair after validating its
delivery shape and SPLPROT semantics. It does not disable unknown-effect rejection
or require a Kitpack source change. Only runtime source changed:
`chriz-sod-remix/lib/comp256_creatures.tpa` (worktree SHA256
`6740293f96cb2d3d1c510b29984fc23106609eb3bd2f108f6099ccd9346b6387`;
committed LF blob `b730bae065bc1d47fcadd117eb5eefe1438afb02831c94a6eac38a9a8340eeca`).
The owner reports all 63 focused tests and exact-commit CI green.

Final disposable `fixed-complete` rehearsal with WeiDU249 succeeded: it preserved
all 364 original BG2 canonical log rows in order and appended only 256 as row365;
21 new assets, six intended existing-resource edits, zero removals/uninstalls.
The later installed SoD siblings have no conflicting recorded resource writes.
All 472 protected files and 129 read resources from the retained source were
verified unchanged. The source report and handoff are under the bridge worktree's
`research/data/issue14-fullstack-20260909/`; report SHA256
`b13efb10c573df8d6234b8c26f65d3d4a8d8236d670dd21def130cfd52ff09fd` was checked here.

This supports preserving **all 391 completed components** with a component-only
repair. It is not yet an accepted recovery of the retained installation: the
collection still needs narrow missing-component/source/order reconciliation and
evidence before continuing the untouched tail. Do not use blind Resume or alter
the original failed receipt. Its hash remains
`710311d47ccab96da032787f3e0ace6281e7f0a30400f0a9d2084f45b5c86938` (rechecked here).
No retained game write, public release, new installation or native playtest was
performed by this handoff. The ready-fix notification has been delivered; the
check-only follow-up is paused pending the recovery decision, and must be
reactivated when a recovery worker starts.

## Combined playtest stopped; targeted recovery assessed — 2026-09-09

The 446-component local test stopped at **01:49 +09:00** in SoD Remix 256. No
installer process remains. Preserve `C:\Users\chris\CEBG-Tests\Combined-20260908`;
**do not delete it or start another full installation**. Current active rows:
27 BG1 + 364 BG2 = **391**. EET, SCS, modpack continuity and EET_end completed;
the ten later runs (including dragons, Safana/Imoen, Lightning, Klatu and BuffBot)
have not started. Public recipe and stream were not changed.

Cause: `BDOLONEI.CRE` contains Artisan MonkRevision-Mystic's opcode 326 delivery
`C0PR#MO1`; `comp256_creatures.tpa` omitted it from its reviewed donor-effect list.
The failure occurred during preflight and WeiDU reports zero component-256 files
to roll back. Other 34/35 SoD components completed. Root error is around line 10907
of the game's `SETUP-CHRIZ-SOD-REMIX.DEBUG`; retain that and the immutable terminal
receipt `.chriz/attempts/terminal-0000000234-19d855a4d1b269e8/receipt.json`.

Christopher asked whether the installation can be salvaged and to send the problem
to its owner.
The SoD task **Find next roadmap item** (`01a071d8-870a-7b02-bbde-e33d0f0b051f`)
received the exact logs, source commit, cause, focused regression request and
read-only-install boundary. It is working on the source fix; no public release or
game mutation was requested from that task.

Read-only recovery evidence: the SoD before-log matches the previous successful
run's after-hash. Its 330 BG2 active rows are retained exactly; the current log adds
only the 34 successful SoD siblings in relative authored order. The existing
supervised recovery mechanism can retain **all 27 BG1 + 330 pre-SoD BG2 components**
and redo only the SoD run plus the untouched tail. It requires reverse rollback of
those 34 SoD siblings, verification of the original prefix, a corrected 35-component
SoD run, complete tail evidence, and a separate recovery receipt. The existing
operator scripts are incident-specific, so this case still needs its own small
operator adapter; it is not a generic CLI recovery command. Ordinary Resume is
not valid, and no repair has been performed or accepted yet.

A smaller append-only 256 repair is potentially possible but not currently
supported by the recovery verifier. The owner must confirm compatibility after
the already-installed 260–910 siblings, not merely after 255; otherwise do not
invent an out-of-order success receipt or disable exact-order verification. The
owner was asked to clarify its 'never uninstall 255' note for both approaches.

Hourly check-only follow-up `cebg-overnight-install-follow-up` is now ACTIVE for
this fix/test (the old no-SR one-time follow-up had been paused). Stay quiet on this
already-reported failure; notify a ready fix, new failure, required decision or
verified completion. Pause when awaiting user direction or completed; reactivate
the same follow-up whenever an authorized recovery worker is launched.

## Combined playtest launched — 2026-09-08

Christopher confirmed modpack completion and asked us to continue. Bridge/filler
source is ready at `3b34eaee19dcb9043f3b72c77bf5b92adbed05ca`; clean modpack
integration is `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`. Preserve Sarah 190 and
existing companions; use Safana **189** (191 is reserved), continuity 199 and Imoen
620. The bridge hold and modpack source-integration blocker below are resolved.

The separate ignored `target/combined-playtest-20260908/recipe` is assembled from
seven frozen local sources. It preserves all 435 current recommended components
and adds exactly 11 experimental components, with no duplicates: **446 / 50 runs**.
Authoring validation has zero findings. Focused source/recipe tests pass; generated
archives are deterministic. Public recipe/feed and stream remain unchanged.

One hidden CLI installation started at **2026-09-08 22:54:18 +09:00**, PID **60364**,
managed root `C:\Users\chris\CEBG-Tests\Combined-20260908`, preset
`chris-recommended`, unique name/save profile `CEBG Combined Playtest 2026-09-08`.
It reuses `C:\CEBG-creator-full-cache`; seven local ZIPs and Evandra's existing
official EXE use the normal verified manual intake. CDTweaks/IWDification archives
were reused from the app cache. Initial source checks passed; the engine created
its managed ledger/frozen recipe and is acquiring/verifying the selected artifacts
at this checkpoint. Completion/native playtest are not yet claimed. **Do not start
another copy or edit this recipe during the run.**

Worker and full stdout/stderr are retained under ignored
`target/combined-playtest-20260908/run/`. Frozen worker SHA256:
`1e251c6af996bd64d039fa0a25ce65b02baae87353629e7756c9f70e7b937318`.
The first launch guard stopped before creating a game because the shared debug
binary had been rebuilt during recipe checks; the final copied binary was
revalidated and then launched. This was not a game-install restart.

Safana core v0.5 is added only as prerequisite for 189, addressing the inventory
gate documented in her curation. No Safana Bard/Abettor preset. Continuity 199
follows companion conversions immediately before EET_end; Safana/189/620 stay
late. SR/RR compatibility is after both core mods and before SCS; SR60's scans
target joinable NPCs, not those hostile RR actors. Lightning80 is after SR60 and
the other spell modifiers, then Klatu and BuffBot last. See the
[updated plan](plans/2026-09-08-combined-playtest-and-alpha16.md) for playtest saves.

## Combined test on hold for SoD bridge — 2026-09-08

Christopher wants to wait for the SoD bridge implementation (issue 14), then test
it together with the other SoD changes in one combined installation. Do not start
that installation until the bridge has a testable source snapshot. Refresh its
commit/component requirements before assembly; this is not a requirement to
publish the mod first. The prepared alpha.16 source remains saved, with no change
to public releases or the stream installation. See the
[combined playtest plan](plans/2026-09-08-combined-playtest-and-alpha16.md).

## Alpha.16 source prepared; combined playtest mapped — 2026-09-08

Proposed release is app alpha.16 / collection alpha.14: existing startup loading
feedback, default-optional Klatu armor QoL, and released Bardic `v2.9c-balance.4`.
435 recommended components / 44 runs; no other curation/pin changes. App version
markers and notices agree. The Bardic ZIP was fetched from its official release
and independently checked (5,177,696 bytes, SHA256
`ca7bb2b70ad50b5b6c0fa59a051e53b90c42d3cc9f98fd187a5c1ed40fc1efa7`).
Draft [release notes](release-notes-alpha16.md) are ready. Public remains app
alpha.15 / collection alpha.13: no new setup package, upload or feed update yet.

Checks: frontend typecheck + 146 tests + production build; 30 Python recipe/credits/
updater tests; 16 native curated/release tests, 8 update-ledger tests and 3 package
contract tests. Rust formatting passes. The package signature test remains unrun
until a new signed setup exists. No fresh game install or new live acceptance.
The draft ledger now covers the actual artifact/run/feature update addresses;
the focused regression reproduced the missing coverage before correction.

The [combined-playtest plan](plans/2026-09-08-combined-playtest-and-alpha16.md) shows
how most implemented gated work can share one fresh SR-on EET install with several
disposable saves. Do not bulldoze a stream clone. Important assembly fix: the old
modpack continuity snapshot reuses Sarah's published component 190 and lacks the
newer 192–198 conversions. Port it onto current modpack, preserve those IDs and
allocate a new continuity ID (199 proposed), integrating 620 deliberately. Split
its required pre-EET_end work from the existing late repairs. Keep SCS and SoD in
their current curated phases. SoD PR21 filler changes have code; bridge issue 14
does not. Mutually exclusive Lightning variants and no-SR Darkbloom are not all
simultaneously testable. No mod repo, stream files or saves changed here.

## Armor-thieving QoL — 2026-09-08 (integrated, not published)

Approved: default-checked optional thief skills in armor, **no added penalties**.
CDTweaks 2100 stays excluded. Klatu 2150 is integrated in draft collection alpha.14,
after armor/kit changes and immediately before BuffBot (still last): 435 recommended
components across 44 runs. No other Klatu component or unrelated mod update was added.
The real WeiDU synthetic install/uninstall fixture passed, as did 28 Python tests,
16 native curation/release tests and Rust formatting. No full reinstall was needed.
Exact scope and limits are [recorded here](plans/2026-09-08-armor-thieving-qol.md).
Published app remains alpha.15 / collection alpha.13; alpha.14 release metadata is
draft, including its provisional timestamp. No live game or save was modified.

The requested quick [mod-repo and patch-candidate audit](plans/2026-09-08-mod-repo-readiness.md)
is complete. Bardic balance.4 is now pinned in draft alpha.14; SR/RR's released tail
patch remains subject to the earlier collection deferral. Artisan's merged work is
not release-approved. Armor QoL and the separate kit-description repair are the
first existing-game patch candidates, not already applied patches.

## Visible startup while detecting games — 2026-09-08 (source only)

The blank window during slow first-launch checks is fixed in source: early HTML
fallback, staged loading text/spinner and retryable startup errors. The recommended
recipe and installed games are unchanged. Frontend checks and a delayed browser
preview pass; include this in the next app patch, not a collection recipe update.
Published app remains alpha.15 / collection alpha.13. See
[cause and acceptance](issues/startup-black-window-2026-09-08.md).

## Alpha.15 published; no-SR installation recovered — 2026-09-07

Christopher approved publishing BG Rebalance v0.3.2, updating the collection pin,
and recovering `C:\Users\chris\Games\ChrizEasyBG-No-SR-Test` without a full restart.
The original alpha.12 recipe failed only Rebalance 401; its seven successful
siblings form the exact top tail after 372 BG2 components. Use the existing
supervised recovery lineage, not a rewritten ledger or an unsafe Resume bypass.
BG Rebalance v0.3.2 is published and independently verified. The existing no-SR
copy now has all 429 components (BG1 27 / BG2 402), including 401 and BuffBot,
with no full restart or repetition of the earlier 372 BG2 components. Its separate
supervised recovery receipt is published; the historical failure evidence is
unchanged. Launch `game/InfinityLoader.exe`; gameplay acceptance remains pending.
Do not run the recovery stages again. BG Radar Overlay 2.5.0.0 is installed and
verified too; final WeiDU logs remain unchanged. Collection alpha.13/app alpha.15
are published, the public updater feed is verified, and the website task has the
release tuple for deployment. Public setup is also in Downloads. See
[scope and evidence](issues/no-sr-tempus401-2026-09-07.md) and
[package acceptance](patch-acceptance-alpha15-2026-09-07.md).

The earlier Alpha13 test completed and Christopher subsequently reported that its
quick gameplay smoke test looked good. Bardic Wonders' agent captured the needed
compatibility evidence separately. Its separate fix is not included in this patch.
The no-SR two-hour follow-up is paused because its failure was inspected already.

## Overnight installation succeeded and retained — 2026-09-07, 07:57 KST

`C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` completed successfully:
43 actual WeiDU runs, all 434 frozen components, BG1 27 / BG2 407 exact final log
rows and hashes verified. SoD Remix's 32 components and BuffBot 1/0 are installed.
No restart/deletion or repetition of the five completed pre-EET runs occurred.
Worker 40540 has exited; final ledger is 236. Do not resume/reinstall this completed
copy. It is retained for Christopher's in-game check; gameplay has not been tested.

The launcher is `game/InfinityLoader.exe`. BG Radar Overlay 2.5.0.0 was also installed
through the existing verified add-on helper and its executable receipt hash checked;
mod logs stayed unchanged. Overnight heartbeat is paused. Desktop app itself still
needs the alpha.14 update from Downloads because unattended setup was blocked.
Do not confuse the receipt's frozen original app alpha.13 label with its unchanged
current collection alpha.12 or the alpha.14 engine used to recover it.

Nonblocking installed-with-warning findings are preserved for later compatibility
work, notably Bardic Wonders 2004 skipping Symphony of the Dark Children because
its finite Abettor controller was unrecognized. Do not silently fix the completed
copy. Exact evidence and acceptance limits: [completion report](overnight-install-complete-2026-09-07.md).

## Public source and release preparation — 2026-09-07

Christopher authorized publishing the existing source repository under MIT for
CEBG-owned code. The publication line integrates the finalized app alpha.14 source
and retains recipe alpha.12 / 434 recommended components. Current source/build instructions
are in [BUILDING.md](BUILDING.md); the [publication record](publication-source-2026-09-07.md)
tracks the public ref, hosted Windows checks and preserved binary/update URLs.

Public source preparation is complete. [Windows CI run 34064618951](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/34064618951)
passed at `36b59971107fe6ef02f1ed312fb904fe4f82fc93`: 620 normal native tests,
139 frontend tests, 42 Python tests, formatting/Clippy/notices and an unsigned NSIS
build. The downloaded CI setup's checksum and unsigned status were verified.
Issue 2's fresh updater-harness startup failure is fixed by test-only manifest
linkage. Build-readiness fixes preserve the recipe/pins and released alpha.14 bytes;
they include test corrections and one equivalent launcher consistency lint cleanup.

The original source audit is historical. EET excerpt attribution is recorded in
[the attribution review](audits/2026-09-07-eet-attribution.md), with upstream rights
preserved. The dependency notices cover separately licensed dependencies; MIT is
not a relicensing of those components. No third-party game/mod archive belongs here.

The installer task's subsequent shortcut/preferences/recovery work is independent
of this publication line. Do not replace its dirty worktree or the running alpha.13
installation. Publishing source does not publish a new app version or alter the
existing `chriz-easy-bg` downloads/updater. Authenticode enrollment remains separate.

## Alpha.14 published; overnight test continued without restart — 2026-09-07

- App **0.1.0-alpha.14** is [published](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.14).
  Signed public setup/download verified; live alpha update feed points to it.
  Collection **alpha.12**, all 434 recommended components/pins, and user selections
  are unchanged. Build source commit: `5610783590ad49b24101d5d3a6a85b018abadb7e`.
- Christopher's exact `C:\Users\chris\Games\CEBG-Tests\Alpha13-20260907` installation
  resumed at EET attempt 2 with the new release engine. Ledger 156 proves EET
  completed; EET components 0/100 are recorded. Main mods are now installing.
  Five prior successful runs were preserved. Nothing was deleted or restarted.
- Background CLI worker started as PID 40540; check its current identity/status,
  newest ledger and `target/cebg-overnight/alpha14-resume-20260907/stderr.log` before
  doing anything. Do not start a duplicate worker. Same native registry ID:
  `install-5f63e5d7939d3f509139`. The CLI uses the native app's frozen-campaign engine,
  but native GUI process controls cannot attach to this background worker.
- The old failed alpha.13 app closed normally. Unattended NSIS execution was blocked
  by the tool environment before starting, so the installed app remains alpha.13;
  do not retry that blocked operation by another route. The verified alpha.14 setup
  is in Downloads for Christopher to apply. This does not affect the running patched
  game-install engine. Native updater apply/restart acceptance remains unclaimed.
- Reused heartbeat `cebg-overnight-install-follow-up` checks every 20 minutes,
  staying quiet while progress is normal. On completion verify the receipt, retain
  the successful copy and pause the heartbeat. On failure preserve evidence and use
  bounded diagnosis/safe recovery. Cleanup/restart of this exact test is authorized
  only as a fallback; stream games, source games, saves and shared cache are protected.
- Website alpha.14 deployment is verified live at `/collection`: PR 9, merge
  `a47e11d6a170ad09a6b95cf28cddc5ea64bc3025`, all 76 site tests and public
  version/hash/download checks passed. No Discord/email was posted.
  The separate source audit task reports the source repository is now public and
  `origin/main` includes build source `5610783` plus audited docs/CI. Do not override
  its work or force-push. Remaining cleanup UI is documented, deferred, not implemented.

See [patch acceptance](patch-acceptance-alpha14-2026-09-07.md),
[incident](issues/alpha13-eet-pre-spawn-2026-09-07.md), and
[cleanup roadmap](plans/2026-09-07-managed-install-cleanup.md).

## Launcher friction patch + current EET recovery incident — 2026-09-07

**Overnight update:** the pre-spawn recovery/process fixes pass focused tests and
Christopher's exact `CEBG-Tests/Alpha13-20260907` copy resumed at EET attempt 2,
ledger 155, at 06:37 KST. Five previous mod runs were retained. Background worker
PID at launch is 40540; logs: `target/cebg-overnight/alpha14-resume-20260907/`.
Do not launch another worker or restart the stack. App alpha.14 is now published;
recipe alpha.12 is unchanged. Full install completion is not yet established.
Christopher authorized releasing a fix and only using cleanup/restart if recovery
cannot safely work. Cleanup UI is deferred: [roadmap](plans/2026-09-07-managed-install-cleanup.md).

Source-only launcher patch: separate Play shortcut names with safe collision handling,
unfinished setup preferences restored with fresh checks, and visible native error
details. Windows shortcut tests 7, native command contracts 39, frontend tests 139,
TypeScript and Vite build pass. These changes are included in alpha.14 without
changing the recipe. See [scope/acceptance](plans/2026-09-07-launcher-friction-patch.md).

Christopher's live alpha.13 test stopped before invoking EET: the process check saw
`game/EET/bin/win32/x86_64/weidu.exe`. The five preceding mod runs succeeded. That
process was absent during inspection; EET has only a before-log, identical to current
WeiDU.log. Alpha.13 mishandles this never-spawned evidence shape, so he was
asked to preserve the folder and hold off on Retry in that old app. Process-liveness
and pre-spawn recovery fixes are complete in the alpha.14 source. Do not restart the entire
installation or claim other agents caused the process match without evidence.
See [incident/evidence](issues/alpha13-eet-pre-spawn-2026-09-07.md).

## EET Documents-path and quiet-install fixes — 2026-09-07 (app alpha.13)

The community EET failure is an upstream Windows path parser truncating a spaced
Documents registry value, not a timeout or missing first game launch. A narrowly
hash-checked staged-file correction runs before EET core, preserving original
downloads/cache and recording exported per-attempt compatibility evidence.
Focused tests and a real verified WeiDU249 `--nogame` macro test pass.

The separate quiet-install notice now waits five minutes, clears on resumed output/
next successful step, truthfully says the process is still running and offers
**Keep waiting**. Frontend 111, Python 35 and focused native/engine checks pass.
App alpha.13 is publicly published, with signed package/public-download verification
and the live alpha Updates feed checked. Recipe alpha.12 and all 434 recommended
choices/pins remain unchanged. The website owner has the verified download and is
updating `/collection`; website deployment is not yet confirmed here.
See [acceptance](patch-acceptance-alpha13-2026-09-07.md)
and [release notes](releases/0.1.0-alpha.13.md).

Christopher chose that the affected tester can **start new with the fixed version**.
Do not spend more time on manual recovery instructions: the old frozen recipe also
contains SoD0.6.7, whereas the current recipe includes corrected SoD0.6.8. No game
or tester folder was modified or restarted by this task; no full install is queued.

## Installation-folder hotfix — 2026-09-07 (app alpha.12)

A community user had to manually create the default user Games directory. The exact
engine bug was reproduced: startup required the immediate install parent to exist,
although read-only path inspection accepted a new nested path. Startup now creates
missing ordinary ancestors and exclusively claims the final root, retaining reparse,
occupied-folder and other safety checks. Both location views explain automatic folder
creation, and the main Browse control remains available after a rejected location.

App alpha.12 is published and signature/download/tamper-checked, with all 110 frontend,
27 orchestrator, 19 preflight, 39 native command-contract, 3 package and 35 Python
checks passing. The recipe is unchanged at alpha.12: the same 434 recommended
components/source pins. No game install or native-window control was used.
See [folder-hotfix acceptance](patch-acceptance-alpha12-2026-09-07.md) and
[release notes](releases/0.1.0-alpha.12.md). The [public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.12)
and mutable alpha updater feed passed anonymous URL/hash/signature checks. A verified
setup is in Downloads. The [website](https://bg.chrizfader.org/collection) update is
deployed: site PR 5, merge `2f13d9196a0efb47f5c84fe97a227c2f15adbd85`. Owner confirmed
75 passing site tests, builds/CI, live content and an independent public setup hash
match. Full website handoff evidence is in the acceptance note above.

## Community SoD 900 fix — 2026-09-06 (published alpha.11)

The supplied diagnostics confirm an overly strict SoD Remix v0.6.7 chest guard,
not a Steam-version defect or user error. Component 900 rejected three existing
items, rolled back zero files, and WeiDU then installed 910. Earlier 310 active
components remain unchanged; BuffBot was not reached. Preserve the failed folder.
See [recovery assessment](issues/sod900-community-recovery-2026-09-06.md) and
[patch acceptance](patch-acceptance-alpha11-2026-09-06.md).

SoD Remix v0.6.8 is publicly released and freshly verified; CEBG now pins it without
changing the approved 434-component recommended selection. App alpha.11 / recipe
alpha.12 signed package is published and passed signature/download/tamper checks,
including a fresh anonymous public download. The mutable alpha feed now offers it.
[Public release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.11)
and local Downloads copy are ready; the website owner is deploying the page update.
This patch includes the clarity/support work below and syntax-safe JSON diagnostic
redaction. It replaces misleading mandatory-restart wording but does not add an
unsafe Continue button: automatic targeted recovery is still private issue 3.
A supervised repair can target the SoD run and six remaining runs, without replaying
the historical 310 components, once the unchanged local evidence is available.
No game/save was modified and no full installation was restarted.

## Post-alpha clarity/support patch — 2026-09-06 (included in alpha.11)

Christopher approved a lightweight clarity/dependency pass, grouped alternatives,
installed-mod visibility, diagnostics from completed installs and first-play help.
Source implementation is in the `installer-v0-real-alpha` worktree; public app
alpha.10 / recipe alpha.11 is superseded by the published patch above; all game installations remain untouched.
See [the section-by-section audit](clarity-audit-2026-09-06.md).

- All 26 categories / 499 choices retain source/group context; ambiguous identification,
  stronghold and stacking options have concrete explanations. Repeated description text
  is suppressed, not expanded into boilerplate. Existing charcoal/brass styling stays.
- 33 explicit alternative groups use compact selectors with None/unchanged. Defaults
  remain optional, initially checked. Atomic switching only replaces group siblings;
  ordinary readiness, parent, dependency and external-conflict checks still apply.
  Three legacy BG1 NPC groups had inconsistent parents/conflicts; the authored curation
  establishes their exclusivity and those constraints are now consistent.
- My installs has a searchable receipt/current-WeiDU list, reported versions and
  missing/extra entries; it does not claim resource-byte or in-game activation checks.
  Diagnostics can be exported after restart for known installs with a terminal receipt.
  Exports stay local and warn about paths/logs before sharing. First-play tips collapse.
- Diagnostic ZIPs now start with a readable outcome/version/run-evidence summary.
  Empty BuffBot `attempts` means no finalized run record in that receipt, not proof
  that WeiDU never started or that the tester made a mistake. Raw evidence and later
  resume history must be checked. A crash before the first terminal receipt still
  prevents ZIP export; preserve the install folder and its `.chriz` evidence.
  See [diagnostic interpretation and verification](diagnostics-reading-2026-09-06.md).
- Christopher will run the install himself to check BuffBot. The community cause is
  established by its diagnostics above; do not run another full install or modify r5/
  the stream installation merely to reproduce this already confirmed failure.

Release integration: these additive presentation fields require the alpha.11 app binary;
old alpha.10 strictly rejects them. App/recipe versions are bumped together and the
new recipe ledger requires app alpha.11. Do not
publish this as a recipe-only update to alpha.10 or overwrite its immutable artifacts.
Native updater apply/restart and pause-close acceptance remain the previously recorded
follow-ups, not completed by the automated/package checks.

## Public alpha publication checkpoint — 2026-09-06

**Published and complete:** https://bg.chrizfader.org/collection now serves the
alpha download/guide/overview/roadmap/credits. Website owner verified production
merge `0cda80e65a0a9c0a5a2cb1b7a8b2b579ab8f92ca`, Cloudflare build
`7e3a258a-3af7-4593-a92f-a1ecaba12552`, 57 passing site tests, successful GitHub/
Workers builds, live page/content/header/route checks and an independent installer
hash match. Public setup and updater feed also passed this task's anonymous hash/
signature checks. See [publication evidence](publication-acceptance-2026-09-06.md).
No further full install or review loop is queued. Tomorrow's follow-ups are native
updater apply/restart and pause-close acceptance, public alpha reports, and the
already-approved post-alpha customization improvements. The fresh updater mock
harness's loader failure is private issue2, not a demonstrated app startup failure.

Christopher explicitly authorized publication after announcing the forthcoming
download on Discord. App `0.1.0-alpha.10` / recipe `0.1.0-alpha.11` is the candidate.
The separate public distribution repo `Chrizhermann/chriz-easy-bg` now exists;
**this collection source repository remains private**. Website task
`Build interactive BG run page` owns `/collection`, quick guide at top, overview,
roadmap, credits and deployment, gated on the exact verified public binary URL/hash.

Final source intake includes official Windows Evandra download-or-skip and SoD
Remix `v0.6.7`/component910. Christopher accepted the six-person skip Yes route;
the owner released commit `a6826ca1452cbe500c5e111f117b56a983a55d0d` with 82 Windows/
Linux tests. The first recipe's 31 SoD components now include 910 as the 32nd;
291/901 remain excluded. The early skip prompt is default inside the SoD toggle.
No full installation was restarted. R5 and all game/save sources remain untouched.

Frontend typecheck/all101 tests, 23 Python curation/generator tests, native command
contracts39, focused recipe/archive/materialization/validation tests passed.
RAR extraction safety review issues were fixed and rechecked; all183 Evandra files
match the accepted reference. Public bundle resources now exclude creator/live
reference TSVs and authoring inventories, and include application/UnRAR notices.
Signed packaging and public download/feed verification are next; do not claim
native updater apply/restart, close/pause interaction or another end-to-end install.
See `docs/releases/0.1.0-alpha.10.md` for the player-facing scope/limits.

## What this is

The umbrella/orchestrator for the whole modded-BG stack: manifest + install order + presets
+ (future) install driver. It composes the chriz-* WeiDU mod repos with 80-ish third-party
mods **without redistributing them**. Architecture + rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md` (user-approved 2026-07-03).

## Current priority — curation-derived recipe, not historical replay (2026-09-05)

**Latest approvals (2026-09-06):** [release intake and website coordination](plans/2026-09-06-release-intake-and-website-roadmap.md).
Christopher reports r5 gameplay without crashes/issues. Include released utility
XP610, Yoshimo/Hexxat choices220-223 and SoDv0.6.6 ending290 in the next recipe;
Hexxat's approved default is Shadowdancer221;222/223 remain alternatives. Imoen620 and dragon
work remain WIP. FullSoDskip is reportedly nearly ready: owning-task readiness
refresh requested before intake. Website task owns roadmap aggregation; later
UI-mod selection includes compatibility checks. Recipealpha10 source integration
and focused verification are complete; see [intake evidence](recipe-intake-2026-09-06.md).
Safe pause/app-close is implemented with focused automated checks;
native interaction and real WeiDU pause acceptance remain pending. These changes
were packaged and signed locally as app alpha.9 / recipe alpha.10; see
[package evidence](package-acceptance-2026-09-06.md). R5 is not being modified or rebuilt.
Christopher approved friendlier exclusive-choice dropdown/radio controls for a
post-alpha patch, not the initial release. Current conflict blocking/reasons remain.

**Evandra public acquisition:** automatic official download was approved, but the
ordinary HTTP probe received a Cloudflare browser challenge. Christopher then
approved an upfront download-or-skip exception and supplied official Windows
`evandra-v2.2.exe`. Recipe alpha.11 now pins its exact size/hash rather than the
private aggregate; explicit RAR SFX extraction never executes the downloaded EXE.
The early UI gate and native readiness checks are implemented and focused tests
pass; final payload comparison/package integration are underway. See the
[Evandra public acquisition checkpoint](handoffs/2026-09-06-evandra-public-acquisition.md).
Do not bypass the challenge, mirror the mod, or publish the older private contract.

**Latest instruction (2026-09-06 KST): [targeted r5 recovery](targeted-recovery-2026-09-06.md).**
Christopher approved repairing just the failed modpack rather than repeating the full
installation. **R5 has now completed all 430 components through supervised recovery**:
16/16 corrected modpack components and the three remaining commands passed, preserving
the prior 383 active BG2 entries. Final exact-order audit passed. Original managed
failure history remains intact. [Managed recovery completion](managed-recovery-acceptance-2026-09-06.md)
now passed too: separate receipt and registry, final isolated save identity, exact
versions, launcher consistency, CLI report and idempotent publication. Radar 2.5.0.0
is installed and its second current-release check reused the installation. Next is
current native package/launcher, game startup/save-reload and app-update apply/restart
acceptance, not another mod installation.
R6 was deliberately stopped before mod installation. Its cleanup was tool-rejected;
do not retry deletion by another route, resume it or start a full copy. The recovered
r5 retains earlier Bardic balance.2 and all approved selections. No game smoke yet.
The status below is the preceding r6
snapshot and is superseded by the targeted recovery record.

### Historical r6 context — superseded by completed r5 recovery above

The user has now requested the complete corrected installation/test/cleanup flow. Continue
from [the current acceptance run](curated-full-acceptance-2026-09-05.md), not the invalid
overnight replay. Recipe reconstruction and source/order verification run in parallel.

Start with [the approved reconciliation path](plans/2026-09-05-curation-reconciliation.md).
The overnight `creator-full-current` recipe bypassed recorded curation; it is invalid as
the curated collection. **Do not repair Bristlelick or resume that selection.** Curation
files are intact. Restore them as the authority, preserve deferred mod work, check changed
ordering against mod documentation/source. **Fresh r6 is running** at
`C:\Users\chris\CEBG-Tests\CEBG-Curated-20260906-r6` with recipe **alpha.9**.
Released modpack **alpha.5** fixes both r5 failures (170 Xan / 192 Viconia), with
public-component installation, repeat and uninstall checks on the actual captured r5
input resources. Bardic Wonders **balance.3** is also pinned. Both packages were
independently downloaded, hash/layout verified and extracted by the author verifier.
The resolved **43 runs / 430 components have identical selections, order and arguments**;
all 30 approved SoD components remain, Aura and 290 remain deferred. No new companion
or utility-XP defaults were invented. See the [current acceptance record](curated-full-acceptance-2026-09-05.md)
for the exact worker, startup retry and checkpoints. Full integration is not yet proven.

R5 remains read-only terminal evidence at `C:\Users\chris\Games\CEBG-Curated-20260905-r5`,
installation `install-e6325c7c98451ad4901e`. Ledger 222 is `fresh_copy_required`;
14 other modpack components succeeded and the previous 383 WeiDU rows were unchanged.
The [resolved owning-repo handoff](handoffs/2026-09-06-modpack-xan-viconia.md) retains
the original diagnosis. Do not resume r5 or change its recipe/seal. SoD and BG Rebalance passed.
SoD v0.6.5 is published, independently downloaded/hash-verified, and pinned. Its mod bytes
match the candidate Christopher accepted in focused gameplay/save-reload testing. See the
[owning-repo handoff](handoffs/2026-09-05-sod-remix-component-120.md).

The prior native-order and conditional Randomiser-answer failures are fixed. R4 then hit
a recipe TP2-alias mismatch after all 30 SoD components succeeded. The nested canonical
path is corrected with regression coverage; no component choices changed. Failed r3/r4
remain immutable and must not resume. App alpha.8 is built and installed locally and fixes
the misleading Retry option when a new copy is required. Radar Latest 2.5.0.0 download,
layout and seven engine tests passed; add it only to a successfully completed copy. The
[current acceptance record](curated-full-acceptance-2026-09-05.md) has the exact worker,
frozen hash, logs, remaining launch/update acceptance and cleanup boundaries. Full fresh
collection acceptance is **not complete**. Older packages/status below are historical;
there is still no public CEBG release/update channel. Installer acceptance is the priority.

Christopher's [short alpha-release path](plans/2026-09-05-alpha-release-short-path.md)
avoids another whole install merely to start it through a different entrypoint. Source
update notifications and the post-app-update/new-setup action are now visually tested
([evidence](updates-ui-acceptance-2026-09-05.md), 80 frontend tests); the installed alpha.8
predates this slice. Next signed candidate must include it. Public Evandra acquisition
needs the verified original standalone package, not the creator's private aggregate ZIP.
The explicit seven-copy cleanup request was tool-rejected too; see the acceptance record.
Christopher is now deleting those copies manually. Do not race his cleanup or assume each
folder still exists. No agent workaround deletion is allowed; the agent removed nothing.

Christopher requested a separate destination for future tests. Use
`C:\Users\chris\CEBG-Tests\<unique-run-name>` for NEW agent-owned disposable installations,
not the user's normal Games folder or any folder directly under C:. Christopher rejected
the earlier root-level proposal; it was not created or used. Keep frozen r5 where it is; no move,
rename or restart is implied. Public/player install defaults are unchanged. This separation
is for clear ownership and cleanup, not a workaround or guarantee about deletion policy.
Preserve small diagnostics and retire failed full copies when permitted; surface a blocked
cleanup backlog before starting additional large test copies.

Christopher also permits today's finished mod work in the first alpha. The
[cross-repo intake queue](plans/2026-09-05-first-alpha-mod-intake.md) records two
released modpack/Bardic updates now included in alpha.9, utility XP 610 awaiting
selection policy, unselected Yoshimo/Hexxat additions, and remaining WIP. New choices
belong in a later versioned candidate; do not mutate r6, invent defaults, or let WIP
become an installer blocker.

Latest queued requirements are captured in [recovery and casual customization](plans/2026-09-05-recovery-and-casual-customization.md).
Dual update notification is implemented in source. **Safe pause/app-close handling is
missing and is the next installer priority**: Cancel currently kills WeiDU and must not be
advertised as Pause. A read-only recovery matrix and bounded test plan are documented.
The [failure-detail slice](failure-ui-acceptance-2026-09-06.md) is now implemented
in source: retain the actual report reason, identify safely attributable missing
component log rows, keep fresh-copy recovery strict, and do not announce empty
progress as complete. 91 frontend / 9 focused CLI tests and responsive checks passed;
not in installed alpha.8. The fresh r6 CLI binary includes these error-detail improvements.
The requested [common Customize controls](customization-acceptance-2026-09-06.md) are now
implemented in source: six bundles, category bulk actions, preserved preferences and
visible collateral, with 88 frontend / 16 recipe-view / 4 authoring tests passing.
Headless real-catalog layout checks passed. Not in the installed alpha.8 yet; authoring
metadata is now in alpha.9 with no recommended-selection delta. Yeslick's original-class
route currently also excludes combined modpack dispel fix 410; owning-repo follow-up is
recorded. Typed advanced inputs and broader Customize QoL remain; no default curation changed.
Optional confirmed SoD skip with +250,000 protagonist-only XP was dispatched to
the owning SoD task; separate from component 290 and frozen r4, not a first-alpha blocker.
Its native ground-pile probe subsequently failed (owning commit 19e221b); that prototype
must not enter the recipe. Released SoD v0.6.5 is unaffected.

## Historical overnight continuation — CEBG app alpha.3 / recipe alpha.2 (2026-09-05)

Start with [`overnight-2026-09-05.md`](overnight-2026-09-05.md): current code changes,
real test paths, cleanup ownership, release boundary and remaining acceptance. The active
worktree is `installer-v0-real-alpha`, **not** `4d39`. Both Steam source games are now
verified clean. The dotted-artifact freeze/evidence failures are fixed. A compact UI,
full-creator recipe profile, Radar add-on, human receipt versions, lazy mod-list consistency
and signed-app updater are implemented; real full-install acceptance is still in progress.
The first overnight follow-up fixed EET's Windows staged-path argument; EET completed.
The second follow-up stopped on malformed Bristlelick source; the later curation audit
supersedes the proposed repair because Bristlelick was excluded. See the current priority
above. Signed app alpha.3 is built locally but its historical full profile must not be used.
No public release/channel has been published. The old status below is historical.

## Earlier Chriz Easy BG 0.1 alpha status (2026-09-05)

Branch `codex/installer-v0-real-alpha` is pushed. The installer engine and UI are
functional, and the public-alpha recipe validates and resolves **35 runs**. All **30
selected artifacts** have passed cold-cache acquisition, extraction, and payload
verification. Notable ready pins include public CHRIZ-BG-MODPACK `0.2.0-alpha.1`,
CHRIZ-SOD-REMIX `0.6.4`, Artisan's Kitpack `chriz-v1.3.1`, Spell Revisions
`v4.21-chriz.3`, and public CHRIZ-BG-REBALANCE `v0.3.1` with its ten-component curated
fresh selection.

Tasks 19, 20, and 23 are implemented: public-alpha omissions and evidence are
release-enforced; immutable recipe envelopes, update classification, and packaging are in
place; and the Tauri command surface includes restart-safe discovery and resume through the
immutable campaign index. Task 24 has a tested three-track update-center foundation, but
the production updater channel is deliberately still unconfigured.

The player-facing app is now **Chriz Easy BG (CEBG)**. With no registered installation it
opens directly on one compact install screen with detected sources, editable install name
and location, recommended choices, a clear readiness state, and one primary Install action.
With an existing installation it opens as a launcher with Play, Open game folder, install
switching, collapsed technical paths, and recovery for resumable or moved installs. The
optional CEBG desktop shortcut is checked by default and points back to the registry-verifying
launcher rather than directly to the game. Startup remains registry-first, so a completed
game can be launched even when its original source installs are unavailable.

Fresh verification after this UX slice passes 54 frontend tests, 37 native app tests, full
workspace tests and Clippy, and public-alpha validation with zero findings. Responsive checks
pass at 1920x1080, 1366x768, 768x1024, and 375x812: desktop keeps the primary action visible
without a scrollbar, and constrained screens use one column with normal internal scrolling.
The final local NSIS lifecycle smoke passed for the **4,413,788 byte**
`Chriz Easy BG_0.1.0-alpha.1_x64-setup.exe` with SHA-256
`7BF879133A0E49E67A811F85BCAFF98FC61AFA34B32207314F51375D4F42F227`: its branding and
version were correct, all 78 bundled manifest files hash-matched, the installed app stayed
alive for five seconds, and silent uninstall removed the isolated smoke directory.

There is **no full game-install acceptance and no public release yet**. The immediate E2E
blocker is a genuinely clean Steam BG:EE+SoD 2.7.3 source; the detected Steam BG2 source is
clean, while the available BG1 source is rejected for mod residue. These checks were
read-only and no game directory was modified. Production Minisign and Tauri updater keys
also remain to be provisioned, and Task 24's signed updater UI remains pending.

Immediate next actions only:

1. Obtain or restore a genuinely clean Steam BG:EE+SoD 2.7.3 source.
2. Run the first full installer-driven clean EET build and focused smoke before the public
   installer release.
3. Provision the production signing keys/channel and publish the first explicitly alpha
   build only after those gates pass.

Other blocked items remain later work and are not expanded here. Older status sections
below are retained as historical implementation context.

## Status (2026-09-02 — curation snapshot integrated; real-alpha implementation active)

The authoritative component-catalog snapshot from the dirty `main` checkout is preserved
in commit `1eedd8e` and integrated here without modifying that checkout. The normalized
decision semantics and per-component notes live under `docs/curation/components/`. Start
with [`docs/next-session.md`](next-session.md), use
[`FOLLOW_UPS.md`](curation/components/FOLLOW_UPS.md) for the complete evidence-backed queue,
and use [`COLLECTION_TAIL_FIXES.md`](curation/components/COLLECTION_TAIL_FIXES.md) for the
22-fix migration inventory. Explicit choices and source/release/acceptance gates remain;
do not treat every `default` row as currently installable.

The active implementation branch is `codex/installer-v0-real-alpha`. The approved current
milestone is defined by `docs/plans/2026-09-02-installer-v0-real-alpha-design.md` (`ae5855d`)
and `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md` plus its separate
27-task ledger (`fd73921`). This supersedes the fixture-only Recipe Preview as the first
release milestone. The new plan explicitly selects `ureq`'s `rustls`,
`platform-verifier`, and `win-system-proxy` features.

**Current RC status (2026-09-03):** `C:\BG-EET-RC-20260902` has been played for an
extended session without a crash, and BuffBot works. It is not suitable as the public
alpha: Item Randomiser v8.1's physical CRE-item removal left invalid inventory offsets
and references in 95 post-Randomiser creature resources, which can duplicate or omit loot
and equipment. There is no evidence of a crash or save-file corruption. Released v8.1.1
fixes the source defect; a clean rebuilt RC, new game, and Tarnesh loot smoke remain open.

## Engine Phase 1 baseline

The branch includes the `feat/engine-phase1` lineage through Task 6. Historical plan =
`docs/plans/2026-08-20-engine-phase1-implementation.md` plus its status ledger. Crate
`engine/` provides lib `bg_engine` and bin `chriz-bg-install`.

| Task | State | Notes |
|---|---|---|
| 0 scaffold, 1 schema types (`manifest.rs`) | done, reviewed | |
| 2 loader (`loader.rs`) | done, independently reviewed | original `a4cdb6a`; fixes `05fe9f7` + `05b8058`; 16 loader tests |
| 3 validators (`validate.rs`, 9 rules) | done, independently reviewed | original `5feb0a8` (merged `65e3bb8`); fix `a0c812b`; coverage `71f0387`; 41 tests |
| 4 resolve (`resolve.rs`) | done, independently reviewed | original `13df297` (merged `98c5515`); fix `4c61a8c`; 16 tests |
| 5 events (`events.rs`) | done, reviewed | |
| 6 snapshot session persistence (`session.rs`) | done | `39a6692`; nine focused tests |
| 10 fake-game builder (`tests/support/fakegame.rs`) | done, reviewed | KEY/BIF/TLK writer |
| 7 WeiDU invocation, 8 log-diff verify, 9 runner, 11–16 | not started | historical Phase-1 ledger only |

The real-alpha plan is authoritative for next-work order. Its Task 3 intentionally replaces
the Phase-1 Task 6 snapshot model with a create-once, append-only hash-chained campaign
ledger; do not confuse those two separately numbered tasks.

**Task 2 fix-up completed locally (2026-09-01):** schema is probed before strict parsing;
stem mismatch wins over duplicate-id defence; `.toml` matching is case-insensitive; file
symlinks are followed and broken ones return pathful I/O errors; roots are canonicalized;
non-UTF-8 stems have an explicit error; `Manifest::conventional_mod_path` clearly documents
that it is not an actual loaded-file lookup; fixture mods are copied as a directory. Follow-up
tests cover a real duplicate id on case-sensitive filesystems and case-variant extensions;
Windows symlink tests skip only unsupported/permission-denied link creation. Two independent
reviews found no blocking issues; their bounded follow-up requests are now covered. The
focused RED run failed in all five intended cases before implementation.

**Task 3 review completed locally (2026-09-01):** manual sources no longer inherit the
non-manual HTTPS/hash rule; hexadecimal hashes accept either case; one explicit order slot
cannot repeat a component; split `eet_end` entries must form the final contiguous main-phase
block; and aggregation is covered with two independent errors. The four intended regressions
failed before implementation and pass after `a0c812b`.

**Task 4 review completed locally (2026-09-01):** a selected choice may add a component to
the only explicit slot of an unsplit mod even when that slot did not list it originally.
Split mods still require unambiguous explicit placement. The regression failed before the
fix and passes after `4c61a8c`.

**Historical installer v0 preview:** `docs/plans/2026-09-01-installer-v0-design.md` and its
implementation plan established the guided wizard and engine-owned UI boundaries. The
static prototype at `docs/prototypes/installer-v0/index.html` remains a no-op Recipe Preview;
it does not download, copy, or install anything and is not the current release milestone.

**How to build (Windows):** Rust 1.97 stable-msvc via rustup; run cargo from **PowerShell**
with `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"` — Git Bash's coreutils `link.exe`
shadows the MSVC linker. Work in an isolated worktree; never develop in the dirty primary
checkout or either protected game/archive directory. Repo files are CRLF.

**Conventions:** TDD per task (failing test → implement → green → commit "engine: …");
`#[serde(deny_unknown_fields)]` on all manifest structs; errors carry `PathBuf`s; no
`unwrap`/`expect` in library code; doc comments on public items; never write under
`C:\Games\…` (read-only reference); never hand-edit `manifest/install-order.tsv`;
curation content is Chris-only — the engine is curation-independent.

## Background (2026-08-19 — installer app designed)

- **Installer-app foundation:** `docs/plans/2026-08-19-installer-app-design.md` established
  the public EET-only, copy-then-install architecture with Tauri 2 and a headless Rust
  engine. The 2026-09-02 real-alpha design linked above is the current extension.
- `manifest/install-order.tsv` — recaptured by `fbd914b` from the reference WeiDU.log:
  **451 entries / 91 mods**. This resolved the earlier 414-vs-364 discrepancy; the 364
  figure was wrong. Never regenerate it except from the live reference install.
- `manifest/mod-sources.tsv` — 89 source rows are classified. The integrated curation
  snapshot points historical local fixes at the migration inventory, but several immutable
  pins and release artifacts still need refreshing before recipe freeze.
- `presets/` and `app/` are not started. The headless `engine/` contains the Phase-1
  baseline; the current real-alpha plan owns the remaining engine, UI, and release work.
- Parked: [#1 EET XP scaling fix](https://github.com/Chrizhermann/chriz-bg-collection/issues/1)
  (future chriz-layer component; home repo TBD).

## Hard guardrails (user directives)

1. **The game folder (`C:\Games\Baldur's Gate II Enhanced Edition modded\`) is a READ-ONLY
   reference.** Read WeiDU.log / EET_MODDING_GUIDE.md / mod folders freely; never write,
   install, or test there.
2. **Never redistribute third-party mods.** Private archiving of hard-to-find zips is
   acceptable in a private repo, but the public-facing design is links + versions. If the
   repo ever goes public, archived third-party content must be dropped/licensing-reviewed.
3. `gh` CLI auth is shared across concurrent agent sessions — `gh auth status` before any
   gh op; this repo needs `Chrizhermann`.

## Work queue

1. Follow the dependency graph and delivery waves in
   `docs/plans/2026-09-02-installer-v0-real-alpha-implementation.md.tasks.json`; do not use
   the older Phase-0 list or Recipe Preview as the schedule.
2. Resolve only Chris's remaining content choices in `docs/next-session.md`, then normalize
   those catalog rows. Its curation decisions remain authoritative, but its older engine/UI
   scheduling paragraphs are superseded by the real-alpha plan.
3. Refresh and freeze every selected immutable source, hash, license/provenance record, and
   split BG1/BG2 run. Keep blocked or unimplemented choices unavailable.
4. Implement engine and UI slices with TDD in isolated worktrees, integrating reviewed
   commits only after their focused and workspace checks pass.
5. Treat Christopher's cold-cache install, recovery rehearsal, InfinityLoader boot,
   BG1 start, and save/reload as separate live acceptance evidence—not as implied by static
   tests or catalog review.

## Known wrinkles for the driver (from the reference install's history)

- EET requires a BG1EE+SoD source install to import from; EET_end must be the last "core"
  entry, but ~50 additive components legitimately sit after it (see install-order.tsv tail).
- WeiDU v24900 template pattern: copy `Setup-Branwen.exe` as `Setup-<modname>.exe`.
- Some mods prompt interactively despite force-install flags — capture required extra args
  per mod in mod-sources.tsv `notes` as they're discovered.
- Test installs must set a distinct `engine_name` in `engine.lua` so they don't share the
  Documents user dir (saves/baldur.lua) with the live install.

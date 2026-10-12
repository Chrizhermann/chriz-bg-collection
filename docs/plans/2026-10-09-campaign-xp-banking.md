# Campaign XP caps with banking — implementation brief

Christopher authorized design and assignment on October 9, aiming for an
implementation candidate to review tomorrow (October 10, Korea time). This
promotes banking from the deferred list. It is not a readiness guarantee, release
approval, or authorization to modify his game or saves.

**Latest scope correction, October 9:** the
[practical hobby-mod delivery direction](2026-10-09-next-patch-batch.md#practical-hobby-mod-scope--latest-october-9-direction)
takes precedence over treating every edge/compatibility/recovery case below as a
prerequisite. Target the default setup first, document unsupported operations,
keep essential XP/save correctness, and propose any gameplay simplification
instead of silently dropping the approved dual behavior. A short native trial,
when authorized, is preferable to indefinitely expanding offline infrastructure.

## Player-facing design

Optional Modpack component, off by default in CEBG:

- **Campaign XP caps — keep excess XP for later:** BG1 161,000; SoD 500,000.
- Separate dependent option: **Also cap Shadows of Amn at 2,950,000 XP**.
- Without the SoA option, this component stops imposing a cap upon entering SoA.
  With it, the last campaign restriction ends upon entering ToB.
- Outside selected campaigns, preserve the installation's other progression and
  total-cap choices. Do not silently add a ToB cap or extend level tables.
- The bank belongs to each character, not to the party or installation. Usable
  XP stops at the applicable cap; excess earned XP is retained. On a confirmed
  campaign transition, release only what the new cap allows, exactly once.
- Example: 600,000 total earned XP means 161,000 usable + 439,000 banked in BG1;
  500,000 usable + 100,000 banked in SoD; all 600,000 usable in SoA. No bonus XP is
  created by the transition. Normal manual leveling remains normal.
- Different party sizes, XP reward mods and class progression tables keep their
  existing behavior. This is not XP catch-up, reward scaling or automatic leveling.

Baseline cap values are documented in the
[EET Tweaks readme](https://github.com/K4thos/EET_Tweaks/blob/master/EET_Tweaks/readme-EET_Tweaks.html),
under XP adjustments. That mod's ordinary cap choices are not evidence of banking.

## Ownership and delivery

Gameplay owner: `chriz-bg-modpack`, near existing XP components 610/611/620 but
with a distinct identity. Do not invent a component number until the current
owner has checked ongoing allocations (recruitment 237/238 is being developed).
CEBG later owns option dependencies, truthful labels, pins and changelog.

Assigned October 9 to existing owner chat **Review Dynaheir and Baeloth spells**
(`01a0b92c-da6d-7701-8d51-b0df1a17fe08`). Delivery is pending, not implemented
or accepted in the collection merely because the assignment was sent.

Later October 9 checkpoint: the owner's interrupted turn has been resumed and
confirmed active. A tested accounting model and offline native-hook evidence
exist, but there is still no selectable banking component. The owner is
continuing runtime integration, persistence and the approved dual-class behavior
below. Current source of implementation status:
`C:\Users\chris\.codex\worktrees\campaign-xp-banking\chriz-bg-modpack\research\2026-10-09-campaign-xp-banking.md`.

Delegate a separate native worker/worktree within the existing Modpack owner chat
so XP banking can progress alongside companion XP and scroll work. One owner must
serialize TP2, TRA, release-manifest and public-surface integration. No new public
release, push/merge, live-game launch or current-game/save mutation in this task.
Use synthetic/copied-resource tests; prepare any short native acceptance script
for a separately approved disposable run. Do not create another full game copy
merely for source research. Deadline target is a reviewable candidate and honest
test report, not a requirement to bypass an unsolved correctness problem.

## Campaign detection

EET is the first supported continuous campaign, with EEex expected for runtime
accounting/persistence. Confirm actual API/version requirements. SR is not a
dependency; test both SR and non-SR. Additional platform support must not be
claimed without proving its transition and persistence path.

Start from Modpack's `story-utility-xp/cbmsxp.lua` and `cbmsxrt.lua`, which read the
current campaign with ENDOFBG1 and INTOB. Their existing two reward phases merge
BG1/SoD and SoA/ToB, so banking needs a separate four-campaign policy. Do not alter
611's reward amounts or make banking require its selection.

Use confirmed campaign progress, not protagonist XP, an area's name, or a raw
chapter number alone. ENDOFBG1=1 means Sarevok died, not necessarily that SoD has
started (Endless BG1 matters); ENDOFBG1=2 identifies progress into SoA, and INTOB
identifies ToB. The current campaign/transition source must establish BG1-to-SoD.
Support direct campaign starts and initialization delay. Returning to older
areas cannot lower the cap. Loading an older save restores that save's earlier
progress; no process-global high-water mark may override it. See
[EET's campaign documentation](https://github.com/Gibberlings3/EET/blob/master/EET/docs/Modder%27s%20Notes.html).

## Accounting contract and feasibility gate

Prove an engine path that captures each character's actual XP entitlement after
normal party distribution, but before a cap can discard it or permit an unwanted
level-up. Existing XPBONUS rewriting and ChangeStat XP SET code do not prove this.
Polling visible XP changes alone is not an acceptable lossless accounting proof.
Do not ship a list of individually rewritten quest rewards as complete coverage.

Cover kill XP, party-total and individual quest awards, scroll/lock/trap awards,
dialogue/script awards, and supported mod award paths. Never divide a party-total
reward again or award the full party amount to each character. For ordinary
positive gains: usable increment + bank increment must equal original entitlement.
Releasing a bank only transfers its balance; it is not another earned award and
must not trigger a multiplier or be intercepted recursively as new XP.
Debit only what was successfully credited. If another retained total XP ceiling
limits the release, keep the remainder banked and explain it; do not zero the
bank merely because the component's own campaign ceiling has ended.

Distinguish earning from absolute XP SET, recruitment/catch-up, respec, dual-class
reset, intentional XP loss and own bank release. A catch-up operation must compare
its intended target against the relevant existing entitlement, not repeatedly
grant a delta caused solely by hiding banked XP. Coordinate with 237/238 and 620;
do not change their chosen catch-up amounts as a side effect. Report unsupported
external writers explicitly instead of silently adding/removing bank balances.

Multiclasses share a character cap, not one full cap per class. Preserve native
class distribution and rounding. For duals, investigate and document native cap
accounting before implementing: active m_xp and retired-class XP are not the same
thing. Modpack 610's threshold-derived LifetimeXP is not an exact earned ledger.
Retain the bank through dual-classing without treating its reset as XP loss or
creating extra cap room. Do not turn retired-class XP into fresh active-class XP.

### Dual-class choice confirmed in the owner chat, October 9

Christopher approved immediate dual-classing with separate bank ownership:
pre-dual banked XP remains assigned to the old class; XP earned afterward belongs
to the new class. Do not use the discussed delayed-dual or reactivation-lock
alternatives, and do not transfer the old bank to the new class. Both partitions
remain subject to the character's campaign limit.

His hypothetical example: 500,000 usable plus 50,000 banked XP in SoD, then dual
immediately into a level-one new class. If another 200,000 is earned while the
cap has no room, it belongs to the new class's bank. When SoA permits release,
the old class receives its 50,000 and the new class its 200,000. The 550,000
figure is illustrative, not a claimed vanilla Fighter level threshold.

If the old-class bank makes a deferred level due, implement proper progression
with normal player choices and benefits; changing a level byte alone does not
fulfil the design. Do not silently auto-level on transition. The new class must
exceed the old class's resulting level for normal dual-class reactivation. The
owner has identified that vanilla freezes the old class and its level-up screen
only progresses the new one: extending that behavior is unfinished implementation,
not a missing user choice or something to omit from the promised component.

Keep native threshold-based retired-class cap accounting distinct from exact
bank ownership and pre-reset XP. Preserve the agreed bank without converting
ordinary discarded between-level progress into a new award. The owner evidence
and deferred-progression notes must guide the adapter and its regression tests;
do not infer these values from the earlier illustrative arithmetic alone.

### Other XP systems

Do not blindly conflict with CDTweaks 2090: an uncapped underlying storage path
may be needed for lossless banking. The player-visible campaign limit and native
storage ceiling are different. Establish ordering/compatibility with XPCAP,
XPLEVEL, recruitment scripts and EET Tweaks cap scripts so two independent systems
cannot truncate, release or bank the same award. Preserve unrelated class rows
and level progression. If the current hooks cannot prove complete capture,
deliver the exact missing hook and a bounded next step; do not advertise a working
component on the strength of a mocked callback alone.

## Persistence, existing saves and recovery

- Save actor-owned bank data with versioned, validated EEex marshal state (or
  another proven save-bound mechanism). No INI/sidecar shared balance. Loading an
  older save must not carry XP earned later; multiple saves/runs stay independent.
- Handle party reorder, dismissal/rejoin, death/resurrection, area unload,
  campaign handoff and engine object-ID reuse. Summons/clones/previews cannot
  duplicate a bank or become another source of XP. Exporters must be non-mutating.
- Publish XP and balance changes as one coherent operation across save boundaries;
  test reentrancy and repeated callbacks. Preserve unknown/newer schema payloads
  and report errors without zeroing balances or guessing recovery values.
- Existing-save support is a separate acceptance outcome, not automatically
  promised. Safe default design: never remove existing levels, abilities or XP.
  An already-over-cap character keeps that progress and banks future gains until
  the campaign ceiling provides room. Do not reconstruct XP lost before install.
  Report the exception clearly; it is not a retroactive challenge reset.
- Do not silently activate a stricter mode on an existing bank or let ordinary
  settings changes cash out/rebank repeatedly. Bind selected policy to the save;
  a supported policy change requires explicit migration semantics.
- Provide an explicit, tested release-and-disable/migration route before calling
  uninstall with a nonempty bank safe. WeiDU cannot casually edit every save.
  Raw removal of the component or EEex may make banks inaccessible; document the
  limitation and preserve data. If no lossless route is ready, say so prominently.

## Minimal presentation

Brief message when a cap is first reached and when a transition releases XP.
Provide an on-demand human-readable status: usable XP, banked XP, current cap.
No reward-by-reward spam. Do not require CEBG's custom UI: normal/Infinity/
Dragonspear/LeUI must not control mechanical correctness. A concise status action
is sufficient if a character-sheet integration would expand the task.

## Acceptance and handover

1. Boundary arithmetic, huge and consecutive awards, exact-cap awards, party sizes
   1/3/6, and conservation of usable+banked XP under every supported award path.
2. BG1->SoD->SoA->ToB, direct starts, skipped SoD, repeated notifications, travel
   to old areas, initialization delay and save-specific rollback.
3. Reload older/newer saves, two independent runs, save during award/release,
   dismiss/rejoin, dead NPC, actor recreation and copy/clone exclusion.
4. Single/multi/dual classes, odd distribution, level drain, dual reset, NPC
   catch-up, respec/SET and explicit XP losses; no fictional ledger reconstruction.
5. New and already-over-cap characters; native cap-remover and conflicting-cap
   configurations; 610/611, companion XP, SR/non-SR and alternate UI preservation.
6. Installation/reinstallation/uninstall resources and nonempty-bank recovery,
   malformed/unknown state and missing runtime capability. No silent data loss.

Owner handover must list component IDs, dependencies, install order, source/branch,
tests actually run, unproven award paths, existing-save/disable status and the
shortest remaining native check. Synthetic arithmetic passing is not native
award/save/transition acceptance. Prefer a focused engine check over a campaign
playthrough; prepare it, do not run Christopher's game to obtain evidence.

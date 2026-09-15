# Next CEBG release: implemented work, acceptance and scope decisions

Status: **release plan, not a published recipe**, 2026-09-13. Christopher wants
the next release within the next couple of days. Include implemented work as
candidates even when its live tests, merge or standalone release are outstanding.
Do not silently drop those candidates to preserve the older, smaller patch scope.
Discuss additions that still require implementation instead of starting them now.

**September 14 follow-up:** [existing-install update triage](2026-09-14-existing-install-update-triage.md)
records the smaller remaining playtests and per-change patch candidates. It does
not enable automatic hotpatching or relax the existing-save boundaries below.

**September 15 decisions:** Christopher waived the extra final bridge/Mislead
257/266 check as a release gate (not recorded as a test pass). The newly confirmed
[BG1 Yeslick kit gap](../handoffs/2026-09-15-yeslick-bg1-kit-and-dispel.md) is an
explicitly requested next-patch fix, including continuity integration. Retain
already-released Dispel 410; its absence from the old stream install is separate.

**September 16 installer follow-up:** [Documents-folder robustness](../handoffs/2026-09-16-documents-folder-robustness.md)
records the expected automatic handling of redirected/OneDrive/missing folders.
Bounded hardening candidate, not implemented or a confirmed viewer root cause;
obtain diagnostics before claiming a specific fix. No extra full-install test needed.

**September 16 release decisions:** this is now the complete implemented-candidate
release plan. Artisan and Bardic have the user's required acceptance and are
releasing through their owners now. Classic bouncing Lightning **80** is the
default; non-bouncing **81** is optional and mutually exclusive. Include SR/RR
compatibility, and include official Safana plus core 189; Christopher will test
Safana on the candidate. Include the implemented BG1 Yeslick conversion only when
its owner delivery is available. Do not represent unfinished Bard/Abettor conversion
as included. The only compulsory checks before the one full candidate install are
selection, order and source validity; feature playtests may follow on that candidate.

**Wrap-up clarification:** Christopher will run that installation and playtest
himself. Finish source intake and the packaged installer with focused checks only;
do not add unattended installs or further review rounds before handoff.

This supersedes the narrow public-release scope at the end of the
[September 8 combined plan](2026-09-08-combined-playtest-and-alpha16.md), not that
plan's historical installation evidence. No mod, game, source pin or selection was
changed while preparing this document. Owning repos retain all gameplay code.

## Release baseline and scope rule

- Public baseline observed today: **CEBG app alpha.15 / collection alpha.13**.
  Existing working draft names are **app alpha.16 / collection alpha.14**; retain
  these provisionally and synchronize all markers at freeze, not per feature.
- The current authored draft has **436 selected components / 44 runs**. The
  retained experimental install has **446 / 50**. Neither is the final count for
  this expanded release. Generate and review the final selection diff explicitly.
- Keep existing approved curation. A newer mod archive is not permission to enable
  every component it exposes, repair-only components, or an unfinished preset.
- Implemented but unmerged/dirty work needs a named integration source and release
  artifact, not another design project. Pending live tests are visible acceptance
  tasks, not automatic exclusion. If a real blocker appears, discuss a small fix
  versus a specific cut; do not silently remove a feature or restart the whole run.
- This release applies the updated collection to **new installations**. Updating
  the app does not patch existing games, saved actors or `dialog.tlk`.

## 1. Candidate inventory: include in preparation

Evidence below combines current source, owning-repo acceptance notes, release
metadata and the retained collection installation. Automated results are existing
reported evidence, not tests rerun during this planning turn. Release state is a
snapshot: recheck the exact owner tag/artifact once at intake.

| Owner / candidate | What is implemented | Remaining work for this release |
|---|---|---|
| **CEBG startup and curation** | Immediate detection/loading feedback; no-penalty armor thieving via Klatu 2150; Red Wizard 5102 default restored with SR; racial kit unlock CDTweaks 2380 default; Acton Balthis UB25 now unchecked; 24 additional SCS + 44 CDTweaks choices, unchecked with explanations and exclusive groups. | Retain all draft changes. Short native/UI smoke, regenerate final recipe after source intake. SCS 4130/4135/4140 remain Not recommended / experimental; 4130 is unavailable with SR. |
| **Artisan: Assassin 7004** | Cloak of Shadows, revised Death Attack and Expose Weakness; Preparation removed. Merged after public `chriz-v1.3.1`, at `136af542` including `98305dc`. | User acceptance is complete and the owner is releasing now. Preserve the 25 synthetic tests/WeiDU parsing as supporting—not substitute—evidence. |
| **Artisan: Magekiller 7002 and Hivemaster 5002** | Witchbane Strike HLA replacing Imprison Arcana; Insect Shroud's 25% spell-failure change. Same unreleased master candidate. | Focused native HLA/retaliation checks and include in the same consolidated Artisan release. |
| **Artisan: Fighter modals** | Normal/improved Power Attack and Expertise 2/4 tradeoffs. `codex/fighter-modal-balance`, `4602fd1`, ahead of the unreleased master. | Integrate that branch rather than pinning master and losing it. Check AC/THAC0/damage and Minsc fallback resources; automated/byte checks exist. |
| **Artisan: kit Class descriptions** | Production `lib/kit_strref.tpa` correction in dirty worktree `5f1b`; explicit production diff already captured for Combined. Copied-EET install/uninstall checks exist. | Consolidate production patch with the above release; check the actual Class screen. Do not also select its separate existing-game repair installer for fresh games. |
| **Bardic Wonders** | Released `v2.9c-balance.4`, `37d3e5a`: Skald, Dancer, Jester and shared HLA corrections. Already pinned in the current draft. | User acceptance is complete and this owner release is going out now. Darkbloom 1006 remains excluded with SR. |
| **SoD Remix: bridge/filler/companion corrections** | v0.6.9 source: bridge 256, optional challenge sequencers 257, finite Insane Shadow Aspect Mislead 266, assassin/filler/loot/XP corrections 135/175/230/265, Skie catch-up/dialogue/potions and Xan/Yeslick/Shar-Teel fixes. | Intake the released owner artifact that contains the intended consolidation. The extra 257/266 check is waived as a release gate, not passed. |
| **SoD Remix: Khalid at Bridgefort 115** | Released **v0.6.10**: carried Khalid remains controllable at Bridgefort, Adirran provides briefing/commands, the ordinary route remains for non-carried Khalid, and personal-quest/scene continuity is protected. Requires 110. | Entry, normal wardstone transport and briefing were accepted by the user; its 300 main tests, 18 continuity checks and installation rehearsal are owner evidence. This is not a claim of a complete fort/campaign playthrough. Intake the matching v0.6.10 artifact. |
| **BG Rebalance: dragons 110/111** | Lethal melee behavior at the intended high SCS difficulty and wing-buffet spacing 6 to 18 seconds. Concrete dirty implementation in worktree `a947`; already installed in Combined. | Keep both as **optional challenge** components, never forced defaults or a mandatory pre-install combat gate. SCS 6540 required; 110 also needs EEex. |
| **Modpack: continuity 199** | Xan/Yeslick progression transfer through EET, preserving developed stats/spells/proficiencies with BG2 presentation. Integrated source `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`, based on public alpha.5. | Native transition/rejoin/save-reload check, owner release. Keep Sarah 190 intact; never restore the old conflicting continuity component number. |
| **Modpack: BG1 Yeslick Alaghor coverage** | Implemented at `f76b7fbf33d86c4c9501215a41529bd833ca0900` in `chriz-bg-modpack`; 53 tests pass and one is skipped. It covers the BG1 Alaghor gap before 199/EET_end while preserving vanilla opt-out. | Owner integration of 188 before 199/EET_end is still pending. Include it when that delivery exists; no existing-save migration is established and no customized build may be reset. |
| **Modpack: Imoen XP 620** | Spellhold mage XP catches up to party average, capped at 3M; normal and player-dual variants. Same integrated source. | Test first recruitment and no repeated award on rejoin/reload. Install after EET_end and every `IMOEN2.BCS` replacer. This was previously WIP but now belongs in the implemented list. |
| **Modpack: Safana arrival cleanup 189** | Include official Safana and implemented core 189, which clears inherited arrival inventory once without removing later-earned gear. It installed with official Safana in Amn v05 in Combined. | Christopher will test first arrival and later gear retention on the candidate. Do not include unfinished Bard/Abettor conversion without a separate owner delivery. |
| **Spell Revisions: Lightning Bolt 80/81** | 80 restores bouncing with friendly fire; 81 makes spell/wands non-bouncing while retaining bouncing traps. Production helper, installer declarations and tests exist in retained snapshot `29538896e4d9f2836833f5d925b90f7fe181c69c`; Combined installed 80. | Make **80 the default** and **81 an optional mutually exclusive alternative**. Recover/reconcile/package the exact source; geometry, wand and trap checks can run on the full candidate. |
| **BuffBot** | New public `v1.8.4-alpha` excludes unidentified items from inventory/presets/Add picker and respects identification in mixed stacks. Current draft still pins 1.8.3. | Bump verified source/hash to 1.8.4, retain BuffBot as the final mod run. Quick identify-and-refresh/preset smoke; release has automated but no native acceptance claim. |
| **BG Radar Overlay** | Public 2.5.2.0 was still Latest at inventory; discovery/download/update support already exists. | Retain latest-release checking and verify install/launch location and recorded version in the final candidate. No separate redesign needed. |

### Already shipped: preserve, do not count as new implementation

- SoD full skip and victory ending 290, earlier accepted companion work and the
  established ambient/readiness changes stay in the curated setup.
- Modpack alpha.5 already has progressive utility XP 610, Yoshimo/Hexxat 220–223
  choices, and NPC class hardening. Preserve **Shadowdancer as Hexxat's default**
  and real exclusivity; do not select all three alternatives to “include all work.”
- The Shar-Teel canonical-kit fix at `e2db660` is textually already present in
  alpha.5, despite misleading branch ancestry. Do not duplicate it as a new fix.
- Yeslick/Keldorn Dispel 410 is already in public modpack alpha.5 and the default
  draft. Combined's Yeslick resource passes all 40 hostile-only 1.5x header checks;
  the older stream installation lacks 410. Retain it, not a second implementation.
- Artisan Shield Bash CLAB repair and Berserker v2 changes are already in 1.3.1.
  A regression smoke can accompany changed-kit tests; no second implementation.
- BG Rebalance v0.3.2 already contains the Tempus/Branwen compatibility correction.
  Randomiser v8.1.1 and the pinned EEex Remote Console commit have no identified
  newer release delta in this inventory. Preserve them and normal receipt checks.
- Existing app update notifications, diagnostics, receipts, destination handling,
  safe pause and close handling are implemented. The latter still needs native
  acceptance; stale planning text must not turn it into a rebuild task.

## 2. Assembly rules that prevent losing completed work

1. Owners consolidate **specific named deltas** into one release per mod. Do not
   archive entire dirty checkouts: Artisan master/modals/descriptions, SoD
   v0.6.9/115, BG Rebalance v0.3.2/dragons, modpack alpha.5/85cbc425, SR current
   release/Lightning snapshot each have different integration histories.
2. SR Lightning source is retained under the snapshot above and in the local
   `target/combined-playtest-20260908/sources/sources.json` lock/archive. Recover
   that evidence before any worktree/archive cleanup; absence from master is not
   evidence that the implementation never existed.
3. Respect install-time consumers: companion conversions before modpack 199,
   199 immediately before EET_end, Safana 189 after its final arrival-area writer,
   Imoen 620 after its last script writer; dragons after SCS 6540. Keep approved
   SoD placement and TP2 dependency order rather than sorting component numbers.
4. Keep SR60's late NPC scan. Lightning 80/81 follows relevant spell modifiers;
   keep CDTweaks 2530 off. Include SR/RR compatibility: its separate
   `SRCB_RR_COMPAT:0` package belongs **after SR + RR 11/12 and before SCS**, not
   at the generic tail.
5. Fresh SoD installs select the corrected normal components, **not repair-only
   176/235/291**. 257 is optional Extra Challenge, requires 256 and stays unchecked;
   266 is the owner's default Mislead correction. Preserve the full skip route.
6. Keep matching conventional Windows mod releases with setup EXE, licenses and
   source/version provenance. CEBG pins verified artifacts; it does not absorb mod
   implementations or redistribute unrelated third-party archives.
7. Do not infer a final count from the 41 SoD declarations or the 446-row experiment.
   Review defaults, optional choices, prerequisites and exclusions against the
   actual final generated recipe and its diff from the shipped baseline.

## 3. Selection decisions resolved

- Include SR/RR compatibility in its correct pre-SCS position.
- Include official Safana plus core 189 only; Bard/Abettor conversion remains out
  unless its owner delivers it as a separately identified completed change.
- Lightning 80 is the recommended default. Lightning 81 remains available as the
  one mutually exclusive optional alternative.

No need to discuss new creative designs for every completed fix. Keep the tested
regular SoD bridge as the default and 257 as the existing opt-in challenge choice.

## 4. Candidate-following playtests

Use the retained isolated game where its installed source actually matches the
feature. **Do not rebuild it just to test another character.** Launch path:
`C:\Users\chris\CEBG-Tests\Combined-20260908\game\InfinityLoader.exe`.
Keep its separate Documents profile; never run destructive experiments in the
stream game. Use fresh actors/appropriate pre-event saves for grants and area edits.

| Test block | Acceptance worth checking | Existing evidence / preparation |
|---|---|---|
| **Artisan kits** | Retain owner/user acceptance for the releasing work; any later regression sample is evidence, not a gate before integration. | The owner release artifact, not an old Combined snapshot, supplies the candidate. |
| **Fighter/bards/QoL** | Expertise AC by damage type and THAC0 tradeoff, Power Attack damage; representative changed bard song/HLA; armor thieving without added penalties; elf specialist creation and Red Wizard/SR starting spells. | Bardic/armor are in Combined. Racial kit unlock, restored Red Wizard and later modal changes are not covered by its original 446-row receipt. Klatu does not remove distinct Find Traps button-14 or indirect restrictions; report actual behavior, not “all restrictions removed.” |
| **SoD** | Carried Khalid's fort briefing/quest and any independently outstanding owner checks; no extra final bridge/Mislead recheck. | Bridge combat, Bence arrival, save/reload/crossing, Liia reward and selected loot fixes already have owner native evidence. Do not repeat the entire SoD playthrough. 115 is newer than the original Combined install and needs a valid pre-event save. Christopher waived final 257/266 rechecks on September 15; these are not claimed as tested. Broader Shadow Aspect redesign is not a gate. |
| **Companions/Spellhold** | Xan/Yeslick recognizable progression survives transitions/rejoin; Safana clears arrival gear only once; Imoen receives intended XP once, including player-dual route. | 199/189/620 are installed in Combined. Use real transition hooks and first-recruitment saves; teleporting to an already-cached area does not test capture/transfer. These are important progression/data checks, not cosmetic polish. |
| **Dragons** | Optional 110 high-difficulty lethal-melee gating and 111 buffet spacing, if a user chooses the challenge components. | Not required for recommended defaults or before the full candidate install; use fresh encounters and disposable saves if tested. |
| **Spells/utility** | Lightning direction/bounce/friendly fire, wands/trap distinction; BuffBot hides unidentified items and shows them after identification. RR fresh hostile casting only if promoted. | Lightning 80 is installed; 81 can use a separately restored checkpoint at its own tail. BuffBot 1.8.4 is newer. Do not replay EET/SCS solely to switch a final Lightning test. |

Before applying any test delta, record installed versus candidate source and choose
a supported append/isolated fixture. Changing files in a mod source folder does
not apply an installed component, and a tail patch is not automatically equivalent
to the fresh public order. Preserve original receipts and accepted recovery evidence.
If a safe short test route is unavailable, use the final candidate below instead of
building a complicated repair project. Record pass/fail/untested per changed feature.

## 5. Practical release sequence: a couple of days, not an open-ended audit

### A. Minimum pre-install intake

- Lock the already resolved selections, verify their dependency/order placement,
  and verify the exact owner artifacts, versions, hashes and source provenance.
  Do not add a compulsory separate kit, dragon or continuity test gate before the
  full candidate install.
- Freeze exact release sources/hashes and the new curated recipe. Preserve existing
  selections, show the complete additions/removals/default changes, and capture
  optional-source requirements such as Evandra's supported manual intake.
- Validate selection/order/source validity, including SR-off/kit-off/group alternatives
  where they affect resolution. A new failure is investigated, not hidden by disabling
  checks or silently skipping a component.

### B. One final packaged-candidate installation

- After source freeze, run **one public-style full installation** with the actual
  packaged app, verified downloads/cache, normal-user permissions and the recommended
  selections. Use a clearly named managed test destination under the user's test
  root, never C:\Games or the stream. Reuse verified downloads. This is the final
  integration run, not a restart for every pending mod test.
- Exercise download/cache handling, source/manual archive intake and failure
  messaging using small fixtures or a bounded interruption; no need to redownload
  the whole collection to prove one retry path.
- Use that run for native **Pause safely / close / reopen / resume** acceptance.
  Interrupt only at a controlled point; do not kill a real long-running WeiDU write
  to simulate every possible crash. Existing automated recovery checks cover the
  dangerous cases; clearly mark any untested case.
- If a component fails, preserve its logs and the successful prefix, investigate
  rollback/durable changes and try supported recovery. Never automatically redo
  hours of EET/SCS work or silently skip a mandatory/dependent mod.
- Verify final receipt versus WeiDU rows, actual selected versions, final utility
  runs including BuffBot, and Radar path/version. Launch, new game, save/reload and
  a brief gameplay smoke. Do not label an install-only pass as gameplay acceptance.
- Keep the successful install for Christopher. Clean only explicitly identified
  disposable fixtures/failed test copies after evidence is retained; never delete
  the Combined saves before their remaining feature tests are finished.

### C. Update and release acceptance

- Check both **app and collection update indications**, their combined notification,
  version details/changelog, and the “new installation; current save unchanged”
  wording. Radar remains independently versioned.
- Verify the new package's download and updater signature, then one real native
  **apply/restart** path from the supported previous app. Signature/download tests
  already exist but are not proof of GUI update application. Do not move the public
  update feed merely to stage a mock test.
- Finish version markers, release notes, checksums, notices and public artifacts;
  verify website/download/feed agree before calling it released. Authenticode
  publisher signing is separate from the already-authenticated updater, not a new
  signing project that blocks these mod changes.
- Give the website task a short verified changelog and download/version links after
  publication. The broader differences-from-vanilla page still needs user/Twitch
  triage; do not claim every planned feature is already included.

## 6. Planned-only work: recommendation is later, discuss before promoting

| Item | Why it should not silently enter this near-term release |
|---|---|
| Full Challenge Mode | Wand replacements, trap/Skull Trap rules, selected-fight exit locks and the other run restrictions need implementation/policy decisions. Existing SoD 257 and dragon work above can ship without claiming the full mode exists. See the [rule inventory](2026-09-13-challenge-mode-outlook.md). |
| Further SoD encounter work | Ashatiel's Chosen-of-Cyric-style design, bridge collapse/extra units, broader Shadow Aspect simplification and remaining Dorn/Neera audits are separate work; do not wait for them to call the implemented bridge ready. |
| Regeneration on rest / broader spell balance | Rest regeneration is researched but unimplemented; druid healing, Death Ward physical-vorpal coverage and other SR/BG Rebalance ideas need their own decisions/tests. |
| Simplified proficiencies / fully custom companion pips | Existing third-party components are not proof of compatibility with later conversions, Artisan kits and this spell stack. Keep the viewer-suggestion investigation as a separate decision; its local draft is `2026-09-07-optional-proficiencies-and-companion-customization.md`, not a release input. |
| UI-mod selector, automatic install cleanup, generic live patching | Useful installer work, but new capabilities rather than needed intake of completed mods. Do not promise safe hotpatching just because a change touches override files. |
| Script-engine project | Architecture/roadmap exists, not a releasable runtime to add wholesale to CEBG. |
| Aura / ambiguous local scratch work | Aura remains held. Extra untracked Artisan installers and unrelated dirty Hivemaster/Shapeshifter/Swashbuckler work have no established release boundary; ask the owner to identify any additional completed delta rather than bulk-packaging or deleting them. |
| Unreproduced Kivan/Jozzi case and cosmetic follow-ups | Preserve the existing fix and deferred reproduction task. No new fix has been demonstrated for that viewer case; do not invent a release claim. |

**Review checkpoint:** freeze the verified sources and resolved selections, then do
the one final installer run. Pending feature tests run on that candidate afterward;
they are not a pre-run mechanism for quietly shrinking implemented scope.

## Evidence pointers

- [Current draft patch notes](../release-notes-alpha16.md) — narrower implemented
  recipe only until this plan is integrated; not the final expanded changelog.
- [Combined source/install record](2026-09-08-combined-playtest-and-alpha16.md) and
  [handover](../handover.md) — 446/446 installation/recovery evidence, not universal
  native acceptance of later source revisions.
- SoD owner: `docs/releases/v0.6.10.md` records the released 115 artifact and
  bounded entry/briefing acceptance; `v0.6.9.md` and the feature inventory retain
  bridge/filler history.
- Modpack owner: integrated `85cbc425` and
  `docs/testing/2026-09-08-combined-playtest-source.md`.
- [BuffBot 1.8.4 release](https://github.com/Chrizhermann/bg-eeex-buffbot/releases/tag/v1.8.4-alpha),
  [Bardic .4 release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.4),
  [Radar 2.5.2.0 release](https://github.com/tapahob/BG2RadarOverlay/releases/tag/2.5.2.0).

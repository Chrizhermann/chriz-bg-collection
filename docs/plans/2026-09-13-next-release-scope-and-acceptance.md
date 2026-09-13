# Next CEBG release: implemented work, acceptance and scope decisions

Status: **release plan, not a published recipe**, 2026-09-13. Christopher wants
the next release within the next couple of days. Include implemented work as
candidates even when its live tests, merge or standalone release are outstanding.
Do not silently drop those candidates to preserve the older, smaller patch scope.
Discuss additions that still require implementation instead of starting them now.

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
| **Artisan: Assassin 7004** | Cloak of Shadows, revised Death Attack and Expose Weakness; Preparation removed. Merged after public `chriz-v1.3.1`, at `136af542` including `98305dc`. | User's live ability/grant checks, then owner release approval and packaging. Existing 25 synthetic tests/WeiDU parsing are not live acceptance. |
| **Artisan: Magekiller 7002 and Hivemaster 5002** | Witchbane Strike HLA replacing Imprison Arcana; Insect Shroud's 25% spell-failure change. Same unreleased master candidate. | Focused native HLA/retaliation checks and include in the same consolidated Artisan release. |
| **Artisan: Fighter modals** | Normal/improved Power Attack and Expertise 2/4 tradeoffs. `codex/fighter-modal-balance`, `4602fd1`, ahead of the unreleased master. | Integrate that branch rather than pinning master and losing it. Check AC/THAC0/damage and Minsc fallback resources; automated/byte checks exist. |
| **Artisan: kit Class descriptions** | Production `lib/kit_strref.tpa` correction in dirty worktree `5f1b`; explicit production diff already captured for Combined. Copied-EET install/uninstall checks exist. | Consolidate production patch with the above release; check the actual Class screen. Do not also select its separate existing-game repair installer for fresh games. |
| **Bardic Wonders** | Released `v2.9c-balance.4`, `37d3e5a`: Skald, Dancer, Jester and shared HLA corrections. Already pinned in the current draft. | Focused changed-song/HLA smoke, not an exhaustive repeat of every historical bard test. Darkbloom 1006 still remains excluded with SR. |
| **SoD Remix: bridge/filler/companion corrections** | v0.6.9 source: bridge 256, optional challenge sequencers 257, finite Insane Shadow Aspect Mislead 266, assassin/filler/loot/XP corrections 135/175/230/265, Skie catch-up/dialogue/potions and Xan/Yeslick/Shar-Teel fixes. Source branch `codex/issue-14-bridge-finale`: `bf89703`, followed by release documentation `f5d883d`. | Intake completed owner release, or its exact successor containing the consolidation below. Public Latest was still v0.6.8 when checked; a document saying v0.6.9 alone is not publication proof. Final 257/266 native checks remain. |
| **SoD Remix: Khalid at Bridgefort 115** | Separate implemented, uncommitted root-checkout change: Adirran briefs/commands when Khalid arrives with the party; retain Khalid's personal quest and protect carried Khalid/Jaheira scene participation. Requires 110. | Integrate with the newer bridge/filler release without overwriting either branch. Test before first fort briefing, including normal/no-carried-Khalid route and onward progression. This is not merely a wishlist item. |
| **BG Rebalance: dragons 110/111** | Lethal melee behavior at the intended high SCS difficulty and wing-buffet spacing 6 to 18 seconds. Concrete dirty implementation in worktree `a947`; already installed in Combined. | Port only this implemented delta onto current released v0.3.2, then native encounter test and owner release. SCS 6540 required; 110 also needs EEex. Test permanent-death behavior only in a disposable profile. |
| **Modpack: continuity 199** | Xan/Yeslick progression transfer through EET, preserving developed stats/spells/proficiencies with BG2 presentation. Integrated source `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`, based on public alpha.5. | Native transition/rejoin/save-reload check, owner release. Keep Sarah 190 intact; never restore the old conflicting continuity component number. |
| **Modpack: Imoen XP 620** | Spellhold mage XP catches up to party average, capped at 3M; normal and player-dual variants. Same integrated source. | Test first recruitment and no repeated award on rejoin/reload. Install after EET_end and every `IMOEN2.BCS` replacer. This was previously WIP but now belongs in the implemented list. |
| **Modpack: Safana arrival cleanup 189** | Clear the inherited arrival inventory once, without removing gear earned later. Same integrated source; installed successfully with official Safana in Amn v05 in Combined. | Test first arrival and later gear retention. Public core-mod selection needs the explicit decision in section 3; unfinished Bard/Abettor conversion is separate and must not be enabled incidentally. |
| **Spell Revisions: Lightning Bolt 80/81** | 80 restores bouncing with friendly fire; 81 makes spell/wands non-bouncing while retaining bouncing traps. Production helper, installer declarations and tests exist in retained snapshot `29538896e4d9f2836833f5d925b90f7fe181c69c`; not on current master. Combined installed 80. | Recover the exact snapshot/local source archive into the owner branch, reconcile with current SR releases and package. Test geometry/wands/traps. Offer real mutually exclusive choices; public default needs confirmation rather than assuming the experimental selection is approval. |
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
   keep CDTweaks 2530 off. If SR/RR is approved, its separate `SRCB_RR_COMPAT:0`
   package belongs **after SR + RR 11/12 and before SCS**, not at the generic tail.
5. Fresh SoD installs select the corrected normal components, **not repair-only
   176/235/291**. 257 is optional Extra Challenge, requires 256 and stays unchecked;
   266 is the owner's default Mislead correction. Preserve the full skip route.
6. Keep matching conventional Windows mod releases with setup EXE, licenses and
   source/version provenance. CEBG pins verified artifacts; it does not absorb mod
   implementations or redistribute unrelated third-party archives.
7. Do not infer a final count from the 41 SoD declarations or the 446-row experiment.
   Review defaults, optional choices, prerequisites and exclusions against the
   actual final generated recipe and its diff from the shipped baseline.

## 3. Small decisions before selection freeze

These are implemented/testable candidates with **older curation boundaries**, not
permission to reopen every settled design:

- **SR/RR compatibility:** released separately as `v4.21-chriz.4`, but Christopher
  explicitly deferred its public inclusion earlier. Confirm whether to lift that
  deferral now; merely raising the main SR version does not install this package.
- **Safana:** decide whether to promote the official core plus implemented 189
  from the combined experiment to public selections. This does not approve the
  unfinished Safana class/spell conversion.
- **Lightning default:** expose 80 and 81 as alternatives after acceptance; confirm
  whether classic bouncing 80 should be recommended or remain an unchecked option.

No need to discuss new creative designs for every completed fix. Keep the tested
regular SoD bridge as the default and 257 as the existing opt-in challenge choice.

## 4. Tomorrow's focused playtest list

Use the retained isolated game where its installed source actually matches the
feature. **Do not rebuild it just to test another character.** Launch path:
`C:\Users\chris\CEBG-Tests\Combined-20260908\game\InfinityLoader.exe`.
Keep its separate Documents profile; never run destructive experiments in the
stream game. Use fresh actors/appropriate pre-event saves for grants and area edits.

| Test block | Acceptance worth checking | Existing evidence / preparation |
|---|---|---|
| **Artisan kits** | Assassin grant/uses, Cloak/save-reload, Death Attack consumption, Expose Weakness expiry; Magekiller HLA; Hivemaster retaliation. One Class-description check. | Core candidate/descriptions already in Combined. Source tests exist; native checks still needed. Newly changed Fighter modals need a scoped prepared test delta, not an assumption that old Combined contains them. |
| **Fighter/bards/QoL** | Expertise AC by damage type and THAC0 tradeoff, Power Attack damage; representative changed bard song/HLA; armor thieving without added penalties; elf specialist creation and Red Wizard/SR starting spells. | Bardic/armor are in Combined. Racial kit unlock, restored Red Wizard and later modal changes are not covered by its original 446-row receipt. Klatu does not remove distinct Find Traps button-14 or indirect restrictions; report actual behavior, not “all restrictions removed.” |
| **SoD** | Carried Khalid's fort briefing/quest; final regular versus challenge bridge behavior; Shadow Aspect Mislead only once; remaining assassin/quest/travel checks. | Bridge combat, Bence arrival, save/reload/crossing, Liia reward and selected loot fixes already have owner native evidence. Do not repeat the entire SoD playthrough. 115 and final 257/266 are newer than the original Combined install and need preparation with valid pre-event saves. Broader Shadow Aspect redesign is explicitly not a gate. |
| **Companions/Spellhold** | Xan/Yeslick recognizable progression survives transitions/rejoin; Safana clears arrival gear only once; Imoen receives intended XP once, including player-dual route. | 199/189/620 are installed in Combined. Use real transition hooks and first-recruitment saves; teleporting to an already-cached area does not test capture/transfer. These are important progression/data checks, not cosmetic polish. |
| **Dragons** | Intended difficulty gating, lethal-melee protections/delivery and longer buffet spacing, ordinary difficulty unaffected. | 110/111 already installed in Combined. Fresh encounter, disposable saves only. |
| **Spells/utility** | Lightning direction/bounce/friendly fire, wands/trap distinction; BuffBot hides unidentified items and shows them after identification. RR fresh hostile casting only if promoted. | Lightning 80 is installed; 81 can use a separately restored checkpoint at its own tail. BuffBot 1.8.4 is newer. Do not replay EET/SCS solely to switch a final Lightning test. |

Before applying any test delta, record installed versus candidate source and choose
a supported append/isolated fixture. Changing files in a mod source folder does
not apply an installed component, and a tail patch is not automatically equivalent
to the fresh public order. Preserve original receipts and accepted recovery evidence.
If a safe short test route is unavailable, use the final candidate below instead of
building a complicated repair project. Record pass/fail/untested per changed feature.

## 5. Practical release sequence: a couple of days, not an open-ended audit

### A. Scope and source intake

- Resolve the three selection choices above; owners finish consolidated releases
  after the relevant user tests/approval. In particular, the prior Assassin
  “no release yet” remains an actual hold until its playtest result and release
  authorization, not a reason to omit it from this preparation plan.
- Freeze exact release sources/hashes and the new curated recipe. Preserve existing
  selections, show the complete additions/removals/default changes, and capture
  optional-source requirements such as Evandra's supported manual intake.
- Run focused owner regressions and collection plan/dependency tests once against
  the combined sources. Test relevant SR-off/kit-off/group-alternative plans without
  demanding a second complete install for every option. A new failure is investigated,
  not hidden by disabling checks or silently skipping the component.

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

**Review checkpoint:** agree this candidate list and the small selection decisions,
finish targeted tests/consolidation, then freeze and do the one final installer run.
If time is tight, discuss a named outstanding feature rather than letting planned
work expand the scope or quietly losing completed work.

## Evidence pointers

- [Current draft patch notes](../release-notes-alpha16.md) — narrower implemented
  recipe only until this plan is integrated; not the final expanded changelog.
- [Combined source/install record](2026-09-08-combined-playtest-and-alpha16.md) and
  [handover](../handover.md) — 446/446 installation/recovery evidence, not universal
  native acceptance of later source revisions.
- SoD owner: `docs/releases/v0.6.9.md`, `docs/00-feature-inventory.md` and
  `docs/playtest/2026-09-13-shadow-aspect-resume.md` on the bridge branch;
  root-checkout inventory separately records implemented 115.
- Modpack owner: integrated `85cbc425` and
  `docs/testing/2026-09-08-combined-playtest-source.md`.
- [BuffBot 1.8.4 release](https://github.com/Chrizhermann/bg-eeex-buffbot/releases/tag/v1.8.4-alpha),
  [Bardic .4 release](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/tag/v2.9c-balance.4),
  [Radar 2.5.2.0 release](https://github.com/tapahob/BG2RadarOverlay/releases/tag/2.5.2.0).

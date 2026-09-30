# Next work after the accepted patch pilot

Status checkpoint: September 20. This is a navigation index, not a new release
promise or authorization to implement every idea at once. Historical plans retain
their original snapshots; the latest release/acceptance evidence takes precedence.
Game-content work stays in its owning mod repository; CEBG owns integration and UX.

September 29 late request: [less mod-NPC chatter](2026-09-29-companion-chatter.md).
Preserve quest dialogue; review optional banter/interjections per companion,
starting with Fade. Blanket timer scaling risks missing Evandra story content.
Direction approved, but explicitly deferred to the release after the currently
prepared candidate. No change to the verified installer or its defaults.

September 29 follow-up: Emotion Courage/Hope 301 remains an included default,
not a missing addition. Instant trap detection is deferred beyond this release.
Owner discussions now cover utility-XP campaign-based pacing and Artisan Fighter
all-weapon pips; no balance changes selected yet. Fork upstream reviews were
dispatched without automatic merge authority. See the latest
[handover entry](../handover.md#september-29--release-follow-up-decisions-and-parallel-owner-reviews).

September 27 refresh: the [pre-package inventory](2026-09-27-pre-package-inventory.md)
records four newer public mod releases, current UI delivery gaps and the viewer
proficiency/customization compatibility findings. Folder-size preview and optional
Artisan shifted dialogue are now implemented in source, not packaged/released.
The dialogue change has no separate manual-test gate by Christopher's decision.
Historical packaged counts below predate these additions and recipe alpha.18.

## Current priority

**September 29 — high-priority follow-up:** Fade reportedly does not catch up in
experience; Evandra also fails to reach the expected **1,250,000 XP**, despite
the protagonist having substantially more. A later read-only source check found
Evandra's 1,250,000 XP rule, but its actual compiled inclusion remains unverified.
It also confirmed that her intended Sorcerer conversion and custom portrait were
never carried into the public setup. Both are now implemented in Modpack source
as independent components 224/225; Christopher confirmed ChatGPT artwork ownership
and approved inclusion. CEBG default integration and a packaging check are ready,
but owner publication and a verified recipe pin are still required. See the
[delivery handoff](../handoffs/2026-09-29-evandra-default-restoration.md).
Do not assume Fade shares the XP cause. No game/save mutation is authorized;
Christopher is doing his current repair himself. Confirm recruitment/rejoining
behavior and the responsible XP handling before implementing a fix. See the
[report and boundary](../handover.md#september-29--high-priority-for-later-fade-xp-catch-up).

September 28: [stronghold exclusion fix and selection audit](2026-09-28-selection-compatibility-audit.md).
The bounded source scan is complete: stronghold exclusions, BG1NPC core
requirements and our SCS bundle's Ascension-finale requirement are corrected and
tested, with no default change. Not packaged/released. The Opus job failed before
review because its client was outdated; lead/native review supplied the findings.
Mystic Fire balance is deferred in the Artisan repository; Warlock work is deferred
and kit defaults stay as-is.

**Latest user direction, September 22: lower the priority of broader mod updates,
collection-wide current-run/new-run/new-install classification, and additional
hotpatch adapters.** None is a gate for this release. Retain the accepted BuffBot
update/Undo and description patch, existing safeguards and honest scope; do not
disable implemented features. The general updater is not one small finish-line
task: each additional route needs its own compatibility and recovery work.

Christopher approved that proposed parallel batch. Source integration now includes
operation progress/completion feedback, unique installation naming, actionable
folder diagnostics, documentation and the completed Customize frontend.
Combined checks and the normal local package build passed. The package has not
been installed, launched or published; Christopher's finishing UI review remains.
See the
[September 22 release checklist](2026-09-22-release-readiness.md).
No game actions, deletion of real installs, publication or new curation choices
were performed. The September 22 UI-provider recipe/website work still has its
own remaining integration; its earlier parked label below is historical, not a
reason to redo the completed frontend.
September 29 supersedes the earlier deferral: manual checkpoints are in the next
release scope, though implementation is more than a UI-only change. Keep focused
automated/safety checks; the community may cover broader compatibility/playtesting.

See [the preserved update requirements](2026-09-20-existing-install-upgrades-priority.md).
Updating CEBG, a targeted fix, and creating a fresh copy remain distinct from a
general installed-set upgrade. Preserve existing choices and truthful reporting.

The [implementation preparation](2026-09-20-existing-install-upgrades-preparation.md)
records the Sol/Opus source checks, preserved-choice planner, multi-update history,
and first full-component upgrade candidate. The [September 21 implementation
record](../existing-install-comparison-2026-09-21.md) now covers the read-only
comparison and preserved choices. The [first whole-mod route](../existing-install-upgrades-2026-09-21.md)
now implements BuffBot 1.8.3 → 1.8.4 Apply/Undo/Restore in source. Christopher's
September 22 Apply, gameplay and Undo passed in the diagnostic package, with
read-only disk verification; see [the acceptance record](../package-acceptance-alpha19-2026-09-22.md).
The updated UI is now packaged locally; its user review remains pending. Other
whole-mod routes are unsupported.
Interactive testing waits for Christopher; no unattended app/game tests are authorized.

### Earlier narrow-patch delivery checkpoint

**Implemented in source, not publicly released:** the accepted, description-only
patch now has an Updates flow with per-install eligibility, backup/restore,
clear scope and patch history. Next is the bounded delivery/acceptance finish line,
not implementing that flow again.
The [accepted design](2026-09-20-existing-install-patch-pilot.md) defines the three
player-facing categories; the [pilot acceptance record](../hotpatch-pilot-acceptance-2026-09-20.md)
now includes Christopher's successful native descriptions and loaded-save report.
The [production-flow acceptance record](../existing-install-patches-acceptance-2026-09-20.md)
records 25 engine patch tests (including public download and real WeiDU on a
synthetic game), 15 native-app tests and 176 frontend tests, plus frontend
typecheck/build. These do not establish public packaging or a live packaged UI
playtest. No release/version change or RC/stream write occurred during integration.
New Assassin mechanics remain outside this patch; no automatic save editing.

## Already delivered: do not put these back on the implementation queue

[Alpha.18 release notes](../release-notes-alpha18.md) and
[package/publication evidence](../package-acceptance-alpha18-2026-09-20.md) supersede
older draft labels: app alpha.18 / collection alpha.16 is published, including
Modpack alpha.7, accepted SoD work, expanded optional SCS/CDTweaks choices,
background WeiDU, loading feedback, collapsible Customize sections and explicit
managed-install removal. The website points to that verified release.

These are not blanket gameplay claims. The release record still distinguishes
download/signature tests from native updater apply/restart; managed-delete live
acceptance and Modpack alpha.7 native gameplay were not claimed. The historical
full-install plan should not trigger another full installation by itself.

## Open and deferred work

The older bundled `manifest/` fallback's missing evidence/approval records and
release-ledger defects now have local fixes; see the current [handover](../handover.md).
Do not redispatch that resolved source blocker. The remaining release boundary is
final packaged-file inspection, user-supervised review and publication authorization.

| Area | Recorded next work and boundary | Existing record |
|---|---|---|
| Flail of the Ages balance | September 29: Rebalance 302 implemented for next-release default. Retain +5 Free Action but allow Haste/Improved Haste; Slow saves vs breath at -2 (+3), -4 (all +4 variants), -6 (+5). Thirteen disposable real-WeiDU tests pass; owner release/pin integration and live acceptance pending. Prominent website/item-description note required. | [Captured design and boundaries](2026-09-28-flail-free-action-and-dispelling-screen.md#1-flail-of-the-ages-rebalance) |
| Utility XP | September 29: Modpack 611 implemented for campaign-based default, with BG1-sized awards through SoD and BG2-sized awards from SoA/ToB. Existing 610 smooth/protagonist-XP mode stays unchanged as an exclusive optional alternative. Party-average redesign deferred. Owner reports 376 offline checks passing; Christopher approved EE Fixpack's 155-XP correction for high-level BG1/SoD locks. Branch reconciliation, release and collection integration remain. | [Current decision and owner routing](../handover.md#september-29--approved-fork-refresh-story-xp-and-flail-delivery) |
| General Free Action / Haste | Separate policy discussion: permit beneficial Haste while retaining harmful-impairment protection. Reuse an existing compatible tweak if suitable; optional/default and scope undecided. The cause of reported cleric Free Action behavior is unverified. | [Separate Free Action policy](2026-09-28-flail-free-action-and-dispelling-screen.md#2-wider-free-action--haste-policy--separate-discussion) |
| Dispelling Screen / dispel bursts | September 28: first investigate reported dispels getting through an active Screen; intended behavior may need a fix rather than redesign. Separate undecided alternatives: 0.5–1 second grace period, earlier full-round/six-second protection, or exclude Remove/Dispel Magic and SR equivalents from all sequencers/triggers/contingencies on both sides. Coordinate with AI work; no investigation or implementation tonight. | [Screen proposals and open questions](2026-09-28-flail-free-action-and-dispelling-screen.md#3-dispelling-screen--research-and-design-discussion) |
| Entropy Shield balance | Lower-priority discussion: retain the spell's enjoyable identity while reassessing its excessive combined benefits. No particular nerf/default/release selected; verify the provider and existing owner work before implementation. | [Captured concern](2026-09-28-flail-free-action-and-dispelling-screen.md#4-entropy-shield--lower-priority-balance-discussion) |
| Updates / selected game clarity | Packaged locally: compact top selector, distinct names, one target for comparison/fixes/Radar, and supported actions before collapsed comparison details. Frontend typecheck, 323 tests, web build and scoped desktop/narrow review (20 screenshots) passed, followed by native and package checks. User review/publication remain pending. Christopher's earlier BuffBot Apply, gameplay and Undo acceptance is preserved; it does not cover the new presentation. | [Current package and review boundary](2026-09-22-release-readiness.md#local-package-built-user-review-next) |
| Progress feedback | Packaged locally: compact shared operation feedback, actual stages and verified backup-byte progress; unknown work uses a spinner. No invented percentage or ETA, and copied bytes reaching the total do not themselves declare the operation successful. Frontend, native and package checks passed; user review/publication remain pending. | [Current package and review boundary](2026-09-22-release-readiness.md#local-package-built-user-review-next) |
| Clear completion feedback | Packaged locally: persistent, dismissible completion/failure feedback for supported operations, with truthful Undo/recovery wording. App-update success after restart requires the requested different version to be running; a partial failure is not success. Frontend, native and package checks passed; user review/publication remain pending. | [Current package and review boundary](2026-09-22-release-readiness.md#local-package-built-user-review-next) |
| Remove legacy app naming | September 22: obsolete local app copies/installers were recycled. Still migrate the live `%LOCALAPPDATA%\Chriz BG Collection` registry/lock state to a CEBG-named data location, with backups and app/engine agreement. Do not lose existing game registrations, patch recovery or updater identity; do not move live state while CEBG runs. | [Cleanup and current boundary](../handover.md#old-app-cleanup-and-user-driven-acceptance--2026-09-22) |
| More existing-install patches | Deferred, not a release gate. When resumed, triage narrow mechanics and installation-local text adapters individually; preserve existing TLK references and show partial coverage. New grants, cached NPC/quest state and saved-character migration are not proven by the description pilot. Recheck current owner releases before selecting another adapter. | [Component triage](2026-09-14-existing-install-update-triage.md), [accepted patch design](2026-09-20-existing-install-patch-pilot.md#text-and-dialogtlk) |
| My installs / unique names | Packaged locally: compact selection, persistent unique display names, numbered defaults, legacy duplicate disambiguation and rename controls. Labels do not move game/save folders or rewrite receipts. Frontend, native naming and package checks passed; user review/publication remain pending. The narrow concurrent same-name creation limitation is recorded in the handover. September 27: advisory folder-size preview is implemented and checked in source, not yet packaged. Owned-shortcut cleanup and automatic diagnostic export remain later refinements; managed deletion already exists. | [Current naming boundary](../handover.md#september-22-implementation-notes), [size preview](2026-09-27-pre-package-inventory.md#implemented-now-not-packaged-or-published), [cleanup follow-up](2026-09-07-managed-install-cleanup.md#follow-up-roadmap-my-installs-clarity-and-unique-names--september-16) |
| Casual customization | Completed Customize and typed-input frontend from the owning fork is included in the local package, including compact presentation and existing category/bundle actions. Frontend typecheck, 323 tests, web build and scoped desktop/narrow review passed, followed by native and package checks. User review/publication remain pending. Do not redispatch completed grouped-alternative or typed-input work. | [Current package and review boundary](2026-09-22-release-readiness.md#local-package-built-user-review-next), [preserved casual-player plan](2026-09-05-recovery-and-casual-customization.md#3-customize-for-a-casual-player-after-recovery) |
| Viewer weapon/companion suggestions | Investigate optional simplified weapon groups and companion pip allocation, separately from full respec. Existing third-party options have collateral effects; no recipe inclusion or universal compatibility is established. | [Bounded research plan](2026-09-07-optional-proficiencies-and-companion-customization.md) |
| Randomiser / unidentified appearances | September 27: generic unidentified artwork is implemented on Modpack branch `codex/unidentified-item-appearance` as 630 / CBMID.MRK; focused automated checks pass, live acceptance/release/CEBG integration pending. Per-campaign cursed appearances stay with the existing Randomiser task. The two policies and existing install-time Randomiser 570 must be mutually exclusive, without blocking ordinary loot randomization. Current CEBG default 570 is unchanged. Broader pool/curse/reward ideas retain separate scope. | [Implementation and release boundary](2026-09-18-randomiser-next-release-triage.md#september-27-generic-artwork-implementation-unreleased) |
| Challenge Mode | Future prominent main checkbox with individual component choices. September 29 addition: with Spell Revisions, Wish must grant SR Improved Haste rather than the vanilla effect. Broader rules include wand replacements, traps/Skull Trap and selected-fight exit locks. Some SoD/dragon components already ship; that is not completion of the wider mode. Defaults, encounter exceptions and bounded first scope still need discussion. Not part of the current release candidate. | [Rule inventory and owner routing](2026-09-13-challenge-mode-outlook.md), [released scope](../release-notes-alpha18.md) |
| Poison kill XP | Reported player-origin poison kills apparently award no XP. Unconfirmed and explicitly deferred: identify exact source/install, reproduce poison finishing tick versus direct damage, then route the cause to its owner. No generic poison fix is approved or implemented. | [Handover backlog entry](../handover.md#deferred-until-tomorrow-player-poison-kill-xp--reported-2026-09-16) |
| Time Stop / Haste effect timing | September 25 research complete: credible EEex route, no proven universal Lua-only fix. Proposed owner BG Rebalance. Next, separately authorized isolated prototype must intercept before expiry/pulses; otherwise scope an EEex extension. Caster/immune, Slow and area-spell policies remain open. No implementation, tabletop-fidelity or current-save claim; not a current-release gate. | [Feasibility and next proof](2026-09-25-timestop-haste-feasibility.md) |
| Instant trap detection | Requested September 24. Look for an existing component first; working scope removes the detection delay without bypassing skill, range or detectability, revealing all traps or auto-disarming. Passive detection and default selection are separate decisions. Preparation only. | [Reuse check and acceptance](2026-09-24-effect-timing-traps-and-shift-dialogue.md#2-instant-trap-detection) |
| Shapeshift dialogue QoL | September 27: implemented as Artisan tweak 51010, independent and **off by default**; five real-WeiDU synthetic tests passed. Removes only the intended form-item Talk restriction, preserving other restrictions and genuine Silence. Owner release/CEBG wiring pending; separate manual gate explicitly waived, native behavior not claimed tested. | [Current implementation and boundary](2026-09-27-pre-package-inventory.md#implemented-now-not-packaged-or-published) |
| Potion-effect stacking | Requested September 23: no potions should stack, including Heroism, Master Thievery and the rest. Working interpretation: repeated doses must not accumulate their effects, not a change to inventory stack sizes. Reuse an existing component if it fits before writing custom code. Quick lookup: RR 10 retains default thievery potions and prevents their effects stacking; it is already optional in the recipe, but does not establish all-potion coverage. CDTweaks 2640 (PnP Potions) adds random mixing consequences, including explosion/death, and is not the requested simple rule. Settle duration refresh/cross-potion overlap and check selected mod/SR variants before implementation. Any missing custom coverage would belong in BG Rebalance. Recorded only; no selection, game or current review-package changes. | [RR catalog](../curation/components/RR.md), [CDTweaks PnP Potions](https://gibberlings3.github.io/Documentation/readmes/readme-cdtweaks.html#contents_2640) |
| Companion changes | Christopher's ownership decision is to keep companion-specific changes, including rebalances, in `chriz-bg-modpack`, not create a parallel companion layer in BG Rebalance. Dynaheir's Haste already appears as component 197. Baeloth's legal spell-selection proposal must account for SR/non-SR and remain separate from alpha.18; see the documentation gap below. | [Existing Modpack catalog](../curation/components/CHRIZ-BG-MODPACK.md), [alpha.18 exclusions](../alpha18-candidate-2026-09-20.md) |
| The Vanishing of Skie Silvershield | Requested September 22 for later consideration; optional versus default remains undecided. Before proposing inclusion, check current EET integration/install order and overlaps with Eldoth/Skie relocation or companion changes. Not approved for the current recipe or release; no immediate investigation needed. | [Author thread](https://forums.beamdog.com/discussion/81292/mod-the-vanishing-of-skie-silvershield) |
| Regeneration on rest | Requested QoL, researched but unimplemented in this record. Prove a bounded pre-rest/successful-rest hook without changing normal casts before broadening; proposed owner is BG Rebalance. Mass-regeneration semantics and story/Wish rests are unresolved. Druid spell-list balance is a separate discussion. | [Owning-mod handoff](../handoffs/2026-09-11-regeneration-on-rest.md) |
| Website differences from vanilla | After release, create a gamer-readable comparison with Christopher and/or the Twitch website task. Distinguish recommended, optional and planned content. The alpha.18 page is published; the broader comparison is not thereby complete. | [Discussion requirement](2026-09-11-optional-tweaks-and-community-followups.md#website-differences-from-vanilla--after-the-next-release), [website coordination](2026-09-06-release-intake-and-website-roadmap.md#website-owner-and-content-handoff) |
| UI-mod selector | Frontend controls are integrated, but provider recipe integration is still separate. Controls stay hidden when the bundled recipe has no verified provider group; no alternative UI mod is available through this candidate yet. Infinity UI++ and EET SoD preparation remains in the owning fork; LeUI remains compatibility-blocked. September 29: Dragonspear UI++ is approved as an optional next-release provider. Recommended UI unchanged; new installations only. | [Current provider boundary](2026-09-22-release-readiness.md#local-package-built-user-review-next), [preserved design and decisions](2026-09-20-ui-choices-and-manual-checkpoints.md#1-choose-a-game-interface) |
| Manual install checkpoints | Explicitly in the next-release scope, reconfirmed September 29; implementation pending. Advanced/off by default, multiple durable component-boundary pauses with recorded manual additions. No managed-component removal/reinstall workflow; manual additions disable game hotpatches, not app updates. Current pauses are run-level only. Cover Praetor's EEex → manual Dragonspear UI++ → finalization rejection with a regression check; preserve accepted rows and retain checks on unapproved changes. Focused automated/safety checks here; broader compatibility/playtesting may be community-led. | [Preserved design and report](2026-09-20-ui-choices-and-manual-checkpoints.md#2-pause-to-install-your-own-mods-advanced) |
| Darkness / automatic blindness | September 27, for later: Christopher wants the Darkness effect/spell he encounters in SoD and EE content revised because its automatic blindness is unfun. Identify the exact spell/effect and its owner before choosing a change; the reported content scope is not yet source-verified. Replacement behavior, owning mod and release target are undecided. Capture only: no investigation, implementation or live-game action while he is streaming. | User request, September 27; no technical diagnosis yet. |
| Other reported issues | Kivan/Jozzi remains deferred without reproduction/save evidence; dwarf Gallant's CHA gate needs a narrow owner check. Documents-folder failure has a bounded handoff but an unproven root cause, not proof that separate drives are unsupported. | [Community follow-ups](2026-09-11-optional-tweaks-and-community-followups.md), [Documents handoff](../handoffs/2026-09-16-documents-folder-robustness.md) |

## Documentation gaps / refresh before resuming

- **Baeloth:** collection docs mark the new work deferred, but no detailed owning-mod
  handoff or current implementation/acceptance record was found here. The user's
  proposal is legal sorcerer spell counts with deliberate SR/non-SR choices; obtain
  the Modpack owner's current plan before creating duplicate work. The companion
  ownership decision above records the conversation, not verified owner delivery.
- **Poison XP:** the handover is the only focused record found; capture the exact
  poison source and repro when this is picked up rather than inventing a diagnosis.
- **Modding lookup wiki:** explicitly deferred; no separate scope document found.
  Decide what should be durable repo decisions versus reusable modding knowledge
  before creating another documentation system.
- **Recovery/customization records:** several contain historical “source only” or
  “unimplemented” sections. Consult later release/acceptance records and current
  code before treating those paragraphs as present-day missing features.
- **Independent mod repositories:** this index does not re-audit their current
  heads or live tests. Ask the owner for a short implemented/released/pending
  snapshot when selecting the next batch; do not dispatch all deferred ideas now.

The latest direction above supersedes the earlier open-ended next-batch choice:
finish packaged-file inspection and user review of the local candidate. Broader updates,
collection-wide eligibility classification and additional hotpatches remain
deferred; do not silently change curation or resume parked features.

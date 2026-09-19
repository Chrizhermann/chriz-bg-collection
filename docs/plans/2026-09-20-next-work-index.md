# Next work after the accepted patch pilot

Status checkpoint: September 20. This is a navigation index, not a new release
promise or authorization to implement every idea at once. Historical plans retain
their original snapshots; the latest release/acceptance evidence takes precedence.
Game-content work stays in its owning mod repository; CEBG owns integration and UX.

## Current priority

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

Before packaging the new patch UI, reconcile the older bundled `manifest/`
fallback's missing static-evidence/tail-approval records; four native regression
tests expose this existing gap. See the
[exact production acceptance note](../existing-install-patches-acceptance-2026-09-20.md#remaining-delivery-work).
Keep the curated selection and validation protections unchanged.

| Area | Recorded next work and boundary | Existing record |
|---|---|---|
| More existing-install patches | Triage narrow mechanics and installation-local text adapters individually. Preserve existing TLK references; show partial component coverage. New grants, cached NPC/quest state and saved-character migration are not proven by the description pilot. Recheck current owner releases before selecting the next adapter. | [Component triage](2026-09-14-existing-install-update-triage.md), [accepted patch design](2026-09-20-existing-install-patch-pilot.md#text-and-dialogtlk) |
| My installs / unique names | Approved roadmap: compact layout, persistent unique names, numbered defaults, legacy duplicate disambiguation and clear removal feedback. Still planned; the existing delete action is already delivered. Space estimates, owned-shortcut cleanup and automatic diagnostic export remain later refinements. | [Cleanup and naming follow-up](2026-09-07-managed-install-cleanup.md#follow-up-roadmap-my-installs-clarity-and-unique-names--september-16) |
| Casual customization | Common bundles/category actions already exist; continue clarity and compact presentation. Typed advanced inputs and actual remaining UX gaps need a current source check before implementation. Do not redo grouped alternatives: the September 11 record already documents automatic sibling clearing. | [Casual-player plan](2026-09-05-recovery-and-casual-customization.md#3-customize-for-a-casual-player-after-recovery), [delivered bundle evidence](../customization-acceptance-2026-09-06.md), [choice-group follow-up](2026-09-11-optional-tweaks-and-community-followups.md) |
| Viewer weapon/companion suggestions | Investigate optional simplified weapon groups and companion pip allocation, separately from full respec. Existing third-party options have collateral effects; no recipe inclusion or universal compatibility is established. | [Bounded research plan](2026-09-07-optional-proficiencies-and-companion-customization.md) |
| Randomiser | Unconfirmed Mode 2/proficiency report first; editable RR/EE pools, cursed-item variety, per-new-game randomisation and combat-focused rewards are ideas to scope with Christopher, not promised next-release features. Current Mode 1 curation stays unchanged. | [Recorded-only triage](2026-09-18-randomiser-next-release-triage.md) |
| Challenge Mode | Broader rule enforcement remains preparation only: wand replacements, traps/Skull Trap, selected-fight exit locks and the full run-rule inventory. Some SoD/dragon components already ship as options; that is not completion of the wider mode. Defaults, encounter exceptions and bounded first scope still need discussion. | [Rule inventory and owner routing](2026-09-13-challenge-mode-outlook.md), [released scope](../release-notes-alpha18.md) |
| Poison kill XP | Reported player-origin poison kills apparently award no XP. Unconfirmed and explicitly deferred: identify exact source/install, reproduce poison finishing tick versus direct damage, then route the cause to its owner. No generic poison fix is approved or implemented. | [Handover backlog entry](../handover.md#deferred-until-tomorrow-player-poison-kill-xp--reported-2026-09-16) |
| Companion changes | Christopher's ownership decision is to keep companion-specific changes, including rebalances, in `chriz-bg-modpack`, not create a parallel companion layer in BG Rebalance. Dynaheir's Haste already appears as component 197. Baeloth's legal spell-selection proposal must account for SR/non-SR and remain separate from alpha.18; see the documentation gap below. | [Existing Modpack catalog](../curation/components/CHRIZ-BG-MODPACK.md), [alpha.18 exclusions](../alpha18-candidate-2026-09-20.md) |
| Regeneration on rest | Requested QoL, researched but unimplemented in this record. Prove a bounded pre-rest/successful-rest hook without changing normal casts before broadening; proposed owner is BG Rebalance. Mass-regeneration semantics and story/Wish rests are unresolved. Druid spell-list balance is a separate discussion. | [Owning-mod handoff](../handoffs/2026-09-11-regeneration-on-rest.md) |
| Website differences from vanilla | After release, create a gamer-readable comparison with Christopher and/or the Twitch website task. Distinguish recommended, optional and planned content. The alpha.18 page is published; the broader comparison is not thereby complete. | [Discussion requirement](2026-09-11-optional-tweaks-and-community-followups.md#website-differences-from-vanilla--after-the-next-release), [website coordination](2026-09-06-release-intake-and-website-roadmap.md#website-owner-and-content-handoff) |
| UI-mod selector | Later optional choice with actual EET/EEex/BuffBot/other-UI compatibility and order checks, not a promise that every modern UI mod works. | [Recorded scope](2026-09-06-release-intake-and-website-roadmap.md#later-cebg-feature-choose-a-game-ui) |
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

After the first supported Updates patch flow, choose the next small batch with
Christopher. Outstanding ideas remain visible without delaying that delivery or
silently changing curation.

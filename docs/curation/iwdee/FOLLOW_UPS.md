# Chriz Easy IWD — follow-ups

This queue tracks outstanding work; it does not select components or replace the
user's decisions. Started 2026-09-10.

## Curation and sources

- [x] Record the name, same-repo architecture and existing user choices.
- [x] Set up the BG-style worksheet/component-review workflow and additions intake.
- [ ] Inventory shared candidate mods and exact source versions against the reviewed BG collection.
- [ ] Extract full candidate menus and separately verify IWDEE predicates. Keep pending
  inventories outside authoritative catalogs; preserve numeric IDs and semantic labels.
- [ ] Review SCS and CDTweaks choices in spoiler-free batches.
- [ ] Complete Artisan's/Bardic and UI/buff component curation.
- [ ] Finish NPC choices, per-mod expansion checks and crossmod component selections.
- [ ] Review any new quest/item candidates; Frostforged remains deferred.
- [ ] Create the explicit IWD curation map and reconcile every catalog row to an outcome.

## SCS enhancement research

- [x] Inspect SCS v35.21's IWDEE mage/priest routes, generated spellbooks/prebuffs and
  authored override data; see [the source audit](SCS-RESEARCH.md). This is source evidence.
- [ ] Verify release-payload parity and trace the unresolved override-array plumbing
  before concluding that all authored individual spell choices take effect.
- [ ] Build an isolated SCS baseline later and measure actual coverage anonymously:
  actor selection, generated spellbooks, prebuff routines, first-contact behavior.
- [ ] Distinguish absent coverage, delayed execution, weak choices and deliberately
  unprepared casters. Do not infer all enemies are affected from source support alone.
- [ ] Propose conservative caster-role recipes or targeted exceptions where justified,
  preserving identity/level, spell availability, resources and interruption behavior.
- [ ] Adapt existing Chriz readiness/prebuff fixes only after checking their dependencies.
- [ ] Validate with controlled cases and report aggregate results without encounter spoilers.

## Technical implementation and acceptance

- [ ] Confirm supported source builds and pin the matched EEex release.
- [ ] Complete BuffBot IWDEE enablement and focused UI/casting/persistence checks in its repo.
- [ ] Verify selected Artisan/Bardic components and custom NPC interactions.
- [ ] Add explicit collection identity/game profiles to shared app/engine contracts,
  including one-source staging and per-collection updates; preserve existing BG campaigns.
- [ ] Build and verify a self-contained IWD recipe, source hashes and component order.
- [ ] Complete an isolated installation with exact log/receipt reconciliation.
- [ ] Complete launch, creation/levelling, spell/kit/song, NPC and save/load smoke checks.
- [ ] Gather campaign gameplay/balance evidence separately from installation success.

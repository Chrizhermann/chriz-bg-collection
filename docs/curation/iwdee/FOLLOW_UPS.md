# Chriz Easy IWD — follow-ups

This queue tracks outstanding work; it does not select components or replace the
user's decisions. Started 2026-09-10.

## Curation and sources

- [x] Record the name, same-repo architecture and existing user choices.
- [x] Set up the BG-style worksheet/component-review workflow and additions intake.
- [x] Inventory shared candidate mods and exact source versions against the reviewed BG collection
  (2026-10-25/26: Tweaks Anthology, SCS, EEex, EE Fixpack, Hidden Gameplay Options, Spell Menu
  and Remote Console use the same versions as the BG recipe; Infinity UI++ is 1.31 here, 1.28 there).
- [x] Extract full candidate menus and separately verify IWDEE predicates. Keep pending
  inventories outside authoritative catalogs; preserve numeric IDs and semantic labels.
  (2026-10-25: all 37 selected mods, WeiDU 249 interactive pass plus static predicate check.)
- [ ] Component selection (sent to Christopher 2026-10-26 as a private selection page):
  434 installable parts; 386 preselected where the BG installer's decision carries over one to
  one (same version and label) or the choice is clear (a mod's main part, no save patching);
  48 open plus two questions. One question is SCS's spell tweaks: BG makes "Install all spell
  tweaks" mandatory, but on IWD:EE it also brings BG versions of nine IWD spells, against the
  2026-09-08 reason for leaving out Spell Revisions. Record the result in the worksheet.
- [ ] Review SCS and CDTweaks choices in spoiler-free batches (covered by the component selection).
- [ ] Complete Artisan's/Bardic and UI/buff component curation.
- [ ] Finish NPC choices, per-mod expansion checks and crossmod component selections.
- [ ] Review any new quest/item candidates; Frostforged remains deferred.
- [ ] Create the explicit IWD curation map and reconcile every catalog row to an outcome.

## SCS enhancement research

- [x] Inspect SCS v35.21's IWDEE mage/priest routes, generated spellbooks/prebuffs and
  authored override data; see [the source audit](SCS-RESEARCH.md). This is source evidence.
- [ ] Verify release-payload parity and trace the unresolved override-array plumbing
  before concluding that all authored individual spell choices take effect.
- [x] ~~Build an isolated SCS baseline later and measure actual coverage anonymously.~~
  Dropped on 2026-10-25: Christopher wants no separate SCS lab check, since others have
  confirmed SCS on IWD:EE; the full lab install of the finished recipe covers it.
- [ ] Distinguish absent coverage, delayed execution, weak choices and deliberately
  unprepared casters. Do not infer all enemies are affected from source support alone.
- [ ] Propose conservative caster-role recipes or targeted exceptions where justified,
  preserving identity/level, spell availability, resources and interruption behavior.
- [ ] Adapt existing Chriz readiness/prebuff fixes only after checking their dependencies.
- [ ] Validate with controlled cases and report aggregate results without encounter spoilers.

## Technical implementation and acceptance

- [x] Boot EEex 1.3.1 and Bubb's Spell Menu Extended 5.2 on IWD:EE 2.7.3.0 in a
  disposable lab (2026-10-25; main menu reached, both loaded).
- [ ] Confirm supported source builds and pin the matched EEex release.
- [ ] Complete BuffBot IWDEE enablement and focused UI/casting/persistence checks in its repo.
- [ ] Verify selected Artisan/Bardic components and custom NPC interactions.
- [ ] Add explicit collection identity/game profiles to shared app/engine contracts,
  including one-source staging and per-collection updates; preserve existing BG campaigns.
- [ ] Build and verify a self-contained IWD recipe, source hashes and component order.
  Artifact pins for all 37 mods are drafted on `claude/iwdee-recipe`. The engine's new
  `weaselmods-download` source kind (`claude/iwdee-engine`, 2026-10-26) lets the installer fetch
  Lava's 19 packages itself; switching and verifying those pins is in progress.
- [ ] Complete an isolated installation with exact log/receipt reconciliation.
- [ ] Complete launch, creation/levelling, spell/kit/song, NPC and save/load smoke checks.
- [ ] Gather campaign gameplay/balance evidence separately from installation success.

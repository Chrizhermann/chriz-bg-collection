# BG work overview — 12 October 2026

Current coordination snapshot for Christopher. This replaces the *status claims*
in older working lists, not their detailed designs or the user's decisions.
An unchecked box in a September document is not evidence that its feature is
still missing. Read the dated owner evidence below before assigning work.

This audit inspected the nine requested mod repositories, CEBG's active release
worktree and public distribution repository, relevant local branches/worktrees,
owner notes/chats, and current GitHub issues, PRs and releases. Adjacent Aura,
Radar, randomizer dashboard, voice-pack and website work is listed separately.
It is a source/status audit, not a new test run or a promise that every dormant
branch has been exhaustively reviewed. Active owners can advance after this snapshot.

No games were launched or changed, no existing owner chat was reassigned, and
no GitHub issue, PR or release was changed for this audit.

## October 12 implementation dispatch

After the audit, Christopher approved starting the very straightforward,
already-decided tasks and requested a Claude continuation handover.
The following work is now assigned, not merely proposed:

| Work | Status | Owner / reservation |
|---|---|---|
| Emotion301 optional/off in normal defaults | **Implemented locally; ready for next-release integration** | Native worker `/root/emotion_default` completed; edit reservation released. Baeloth stays off; every other default stays unchanged. No full Challenge UI. [Delivery and checks](2026-10-12-emotion-default-delivery.md). |
| Finish existing Blade/Battleluck candidate | **Local installable candidate ready** | Existing Bardic chat `01a07fe8-2fd5-79d3-bb42-4fef15d27fb8` finished `v2.9c-balance.7-rc.1`; edit reservation released. Package checks pass; brief player check remains. No Heroism or older bard backlog. |
| SoD relocated-item identification | **Implemented locally; ready for SoD integration** | Native worker `/root/sod_loot_identification` completed; edit reservation released. Isolated `codex/relocated-loot-identification-oct12` from v0.6.12. Components170/900 preserve source identification and existing inventory data; focused WeiDU checks pass. No new loot placement. |

The [Claude continuation handover](2026-10-12-claude-continuation-handover.md)
reserves these areas and identifies the independent Modpack, Flail and SoD
consolidation work. It separates settled decisions from remaining balance/UI
questions. No new release, live-game action or cleanup is part of this dispatch.
Later sections remain the audit baseline unless this dated dispatch says otherwise.
The Emotion source and generated recipe are checked, but are not a new published
collection. Give the next candidate its proper version; do not overwrite alpha.19.

Bardic candidate and exact delivery record are in
`C:\Users\chris\.codex\artifacts\bardic-blade-battleluck-20261012`:
`Bardic-Wonders-v2.9c-balance.7-rc.1.zip` and `DELIVERY.md`.
The archive is a normal full installable mod, not a late save repair. It is
unpublished; CEBG still pins balance.6. Source remains in the named owner worktree.

SoD fix and delivery note:
`C:\src\private\chriz-sod-rebalance\.worktrees\relocated-loot-identification-oct12\docs\relocated-loot-identification-oct12.md`.
Source is uncommitted/unreleased; merge this narrow change into the eventual SoD
candidate, not into the live game. Existing saved creatures/containers and the
broader audit of other loot-grant paths remain outside this fix.

All three dispatched tasks have now completed their bounded local delivery.
No worker from this dispatch still holds an edit reservation. Release integration
and the remaining player checks are not finished by those deliveries.

## The useful answer

There is enough nearly finished work for a worthwhile combined update. The best
first batch is **recruitment XP, reader-level scrolls, bard ceilings,
Blade/Battleluck, the Flail compatibility fix, Challenge Mode presentation, and
selected finished SoD work**.
XP banking is already installed privately for Garithos and needs a short player
check plus a usable display. Scroll scaling is already implemented in the newer
integrated candidate. Full potion uniqueness is still unfinished. The randomizer
has a playable Mode1 build but incomplete normal packaging and wider parity.

Do not make the next package wait for a complete script engine, a replacement
Spell Revisions, all Time Stop/Haste behavior, or every old compatibility idea.

### Status vocabulary

- **Shipped:** public mod release and/or CEBG inclusion explicitly stated.
- **Integration:** code exists; selection, consolidation or release remains.
- **Local candidate:** implementation exists, but not published or fully accepted.
- **Implementation:** important requested behavior is still missing.
- **Decision/design:** an answer or a bounded design is needed before implementation.
- **Later:** preserved backlog, not an implicit assignment or release blocker.

Public release, CEBG selection and the private stream installation are different
states. In particular, an archive can contain a component which CEBG never selects.

## 1. Baseline: what we already have

[CEBG app/collection alpha.19](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.19)
was published October 7 KST. The website, full changelog and updater feed were
published with it. Defender scans and the earlier candidate's launch/Radar tests
passed; Microsoft approval is **not an outstanding release gate** from that work.
This is not a guarantee against future antivirus false positives.

| Owner | Latest public owner release checked | Current CEBG pin / important distinction |
|---|---|---|
| Modpack | 0.2.0-alpha.10 | Same. Recruitment 237/238, banking 650/651 and replacement scroll runtime are not in this released baseline. |
| BG Rebalance | 0.8.1 | Same. Bard provider/policy 420/421 exist in the archive but are not selected by CEBG. |
| Bardic Wonders | 2.9c-balance.6 | Same. Adapter 3010 exists but is not selected by CEBG. |
| Spell Revisions | 4.21-chriz.5 | Same. Separate configurable-SR development branch is not this release. |
| Artisan Kitpack | 4.81a-dev.85928f9-chriz.1 | Same. Uses upstream version plus fork suffix, not a disconnected fork-only version. |
| SoD Remix | 0.6.12 | Same. Primary checkout is older; new recruitment/finale work lives elsewhere. |
| Item Randomizer | 8.1.1 | Same. Large new-runtime/configurator work is not shipped. |
| BuffBot | 1.10.0 | Same. Carried-item work is already merged/released; some in-game checks remain. |
| Script engine | No public release | Not part of the installer. Current prototype is separate from released enemy-readiness work. |

Already delivered; **do not build these again**:

- Alternative UIs: Infinity UI++, Dragonspear UI++, LeUI BG1/BG2/SoD, EET SoD style.
  These are new-install choices, not supported mid-run UI replacement.
- Advanced, multiple component-boundary manual-install breakpoints. Accepted manual
  additions disable CEBG game fixes/mod upgrades for that copy, not app updates.
- Stronghold/other authored exclusions, compact installation selection, names,
  progress/spinners, completion messages, deletion-size preview, optional Radar.
- Interval-save profile/settings fix, cache-layout recovery and UI launch/order fixes.
- Story-stage utility XP, with the smooth XP-based alternative; this is not XP banking.
- Evandra Sorcerer/portrait, Edwin Red Wizard necklace/slots, optional Baeloth235,
  generic unidentified-item visuals, optional talking while shapeshifted.
- Hardiness options, Pierce Magic/Spellstrike balance, SR-haste bracers, elemental
  arrows, 90% physical soft cap, damageable 75% combat clouds, Flail balance,
  Emotion exclusion and longer Wing Buffet default.
- Released Archer and Assassin work, earlier Bardic numerical balance, SoD bridge
  timer/skip/ending work. The new SoD finale below is a different, unreleased change.

The Assassin and earlier broad Bardic tests have recorded user acceptance. Old
blanket “waiting for playtest” notes must not reinstate those release blockers.
Some narrower feature-specific limitations remain, listed below.

Evidence: [alpha.19 release notes](../release-notes-alpha19.md),
[package/publication evidence](../package-acceptance-alpha19-2026-10-07.md),
`recipes/curated-full-current/mods/*.toml`, and owner releases.

## 2. Closest to a useful next package

| Work | Actual state | Concrete next step | Christopher's part |
|---|---|---|---|
| Fade/Evandra first-recruitment XP, Modpack237/238 | Local implementation and focused checks exist; not released | Consolidate on current Modpack, release when authorized, wire recipe | Brief natural recruitment check; not a saved-companion repair |
| Reader-level scroll casting430 | Implemented and packaged in the combined banking/scroll candidate; unreleased | Check actual casting/startup, then release/pin | One caster and one half-level user; no separate full install |
| Bard kit spell ceilings, Rebalance420/421 + Bardic3010 | Released; missing CEBG selection; privately installed October12 | Add ordered provider/policy/adapter choices to CEBG | Check Eldoth at normal level-up; public selection/default must be recorded |
| Blade specialization/TWF limits and Battleluck | Local installable balance.7-rc.1 candidate ready, unreleased | Intake the delivered package independently of Heroism | Brief level-up and Battleluck check |
| Flail compatibility correction | Open Rebalance PR7 | Review/merge/release correction; update CEBG pin | No new balance decision |
| Early SoD companion recruitment and Liia correction | Implemented in separate dirty worktree, not released | Consolidate/publish selected owner work | Confirm chosen new-install behavior in one short scenario |
| Interim Avernus finale | Implemented in another dirty worktree, not released | Review separately, combine only after resolving shared files with recruitment work | One finale check if included; not another whole SoD run |
| Challenge Mode group and Emotion default | Emotion optional/off implemented locally; broader group still design | Compact group, preserve individual choices/defaults; Baeloth remains off | Confirm initial group membership beyond those two |
| XP banking | Complete-enough local candidate installed privately; not general release-ready | Current-run smoke, small display design, finish documented remaining rules | Play/check current bank; one deferred dual-class decision |
| Out-of-combat Pathfinding Redux | Upstream preview installed privately | Evaluate feel/startup, then decide optional public integration | You explicitly wanted to judge this first |

“Close” does not mean automatically merge every dirty branch. It means there is
existing gameplay work to finish, not a need to restart its research.

## 3. Modpack

### Active work

- **Recruitment XP237/238 — local candidate.** First SoA recruitment uses the
  protagonist threshold ladder through 1.25 million. No continuous party sync,
  XP reduction or dismissal/rejoin farming. Existing saved actors are separate.
- **Campaign caps650/651 — local candidate/private trial.** BG1 161,000; SoD
  500,000; optional SoA 2,950,000. Existing-save adoption retains XP/levels;
  banked XP is released as later campaigns permit. Garithos has650, not651.
- **Reader-level scrolls430 — local candidate.** Relevant active caster level for
  casters; half highest active class level, rounded down/minimum1, for legitimate
  non-casters. Dormant dual classes do not supply the level. Original-reader and
  native runtime, resource reader and normal WeiDU packaging are implemented;
  players do not need Python. The combined candidate composes the banking and
  scroll hooks. Actual game startup/casting/save-load remain untested. The
  currently public component remains the fixed-level workaround; the October9
  early partial-hook checkpoint is superseded by commits2bd0ce0/1eadb92.
- **Companion stat choices230–234 — stranded local work.** Rasaad, Dorn, Eldoth,
  Garrick and Quayle options exist in the older companion-spellbooks worktree,
  but not the newer public TP2. Reconcile the authored changes and intended
  selections; do not silently overwrite the newer Modpack with this old checkout.

### XP bank display

There is **no agreed polished in-game display**. Current implementation provides
cap/release notices and `CBMBank_Status()` in the cheat console. That is useful
for testing, not a finished player-facing interface.

Suggested small design, **not yet approved/implemented**: show “Banked XP: N”
with the character's XP information; a tooltip/details view explains the current
cap and when the bank unlocks. Keep normal XP and bank separate. Start with the
default UI; establish how the same information remains accessible with the
supported alternative UIs before promising identical placement everywhere.
Avoid constant per-award spam or creating a whole new progression screen.

Remaining accounting question: a no-bank Fighter dualled at80,000 XP after
reaching level7 can debit either vanilla's64,000 old-class amount or the actual
80,000 earned. That choice is still unresolved; it does not block Garithos's
approved non-dual trial. Bank removal/migration is also unfinished: uninstalling
the component must not be presented as an automatic cash-out.
Reconcile the October12 private-install/adoption receipt with the older candidate
guide before release: its blanket “not installed/no existing-save use” wording
is no longer the full story. The combined source is clean at1eadb92 on
`codex/campaign-scroll-integration`, still carrying alpha.10 version markers.

### Preserved later work

- **Degrodel and the Helm of Balduran — October13 report.** Christopher reports
  that Degrodel runs away with the helmet when it is brought to him. Review
  dialogue/escape and item-transfer behavior before choosing a fix; cause and
  intended behavior are unverified. Backlog only, no game changes.
- Less companion chatter: start with Fade's selected comment-only interjections;
  preserve quests, romances/friendship progression and player-initiated talks.
  Concrete reduction list/strength needs approval. Evandra's shared story timers
  must not simply be slowed globally.
- Temple donation helper: show exact gold for the next reputation point and make
  that donation convenient; no cheaper reputation was requested.
- Companion pip customization and conversion-before-SCS compatibility, coordinated
  with CEBG's optional SCS4100 investigation; not an automatic full respec.
- Kivan/Jozzi report remains unconfirmed without the relevant saved encounter.
  Keep it distinct from the existing implemented Kivan dialogue guard.
- Private archive issues are not visible on public main: Swashbuckler description
  [#2](https://github.com/Chrizhermann/chriz-bg-modpack-private-archive/issues/2),
  Aura class mapping[#3](https://github.com/Chrizhermann/chriz-bg-modpack-private-archive/issues/3),
  missing Halfling Bard/small-race Monk animation INIs[#4](https://github.com/Chrizhermann/chriz-bg-modpack-private-archive/issues/4),
  and legacy-fix organization[#5](https://github.com/Chrizhermann/chriz-bg-modpack-private-archive/issues/5).
  These need current-provider reconciliation, not blind reuse of old string
  numbers or now-occupied component IDs. Some legacy work has already shipped.
- `save-npc-migration` contains a real historical Viconia/Shar-Teel save tool,
  not a general product for updating every saved companion. Preserve it without
  advertising broad migration support.

Owner evidence:
`modpack-release-alpha6/.../docs/recruitment-xp.md`,
`modpack-release-alpha6/.../research/2026-10-09-scroll-caster-levels.md`,
`campaign-xp-banking/.../docs/campaign-xp-banking.md`,
`campaign-xp-banking/.../docs/scroll-casting.md`,
`campaign-xp-banking/.../docs/testing/2026-10-09-campaign-scroll-candidate.md`,
`campaign-xp-banking/.../research/2026-10-09-campaign-xp-banking.md`,
`companion-spellbooks/.../docs/companion-stats.md`.
These abbreviated roots are expanded in the ownership section below.

## 4. BG Rebalance

### Ready for consolidation versus genuinely unfinished

- **Flail compatibility:** [PR7](https://github.com/Chrizhermann/chriz-bg-rebalance/pull/7)
  is a follow-up to shipped302, not a missing Flail balance implementation.
- **Potion uniqueness:** still an unfinished local prototype. The staged pilot
  covers six potion types and matching SCS copies, not the promised whole
  default-setup component. No ordinary330/331 component is released/available.
  Two exclusive variants remain agreed: unique effects for all temporary potion
  types, or the same with Potions of Genius exempt. Distinct potion types may
  overlap. Repeated Heroism/Power keep the first effects until expiry; other safe
  repeats restart their normal duration without stacking bonuses. Instant healing/cures and
  permanent/quest uses remain normal. Review the simplest complete delivery path
  before adding more infrastructure; two fresh test installs are not requested.
- **Heroism family:** approved first-active-wins across Heroism, Greater Heroism,
  Gallant copies and Potion of Heroism. No refresh or upgrade; each recipient
  independent. Song of Heroism, Aid and Potion of Power are outside this family.
  The shared prototype is still potion-specific; spell delivery and per-target
  admission are missing. One provider owner, then Bardic adapter—not duplicated
  hooks in two repos. Do not call metadata/registration alone a working feature.
- **Wish with SR:** design/source route identified, no implementation. Preserve
  Wish duration/party targeting while using SR haste mechanics. Decide whether
  to adjust SCS's separate enemy Wish clone too. This is distinct from limiting
  Wish to once per combat and from changing all other Wish outcomes.
- **95% physical-resistance option:** not implemented; the approved90% option
  already ships. Needs actual over100 raw-resistance handling, not a new checkbox
  which promises an unreachable limit.

### Separate longer projects / decisions

- Time Stop/Haste periodic-effect and duration behavior: research/prototype work
  exists; no general shipped fix. Recover and separate its work from the potion
  assignment before continuing. Caster/immune actors, Slow and area effects
  require deliberate scope, not an implicit universal tabletop conversion.
- General Free Action allowing Haste: broader policy remains separate from the
  already shipped Flail+5 change.
- Regeneration spells on rest: research only; successful-rest hook and mass/story/
  Wish-rest semantics unresolved.
- Spider immunity to all web types: later requested component, not implemented.
- Instant trap detection: deferred; preserve skill/range/detectability, no free
  map-wide revelation or auto-disarm implied.
- Weapon-dice-only rebalance; deity-appropriate cleric weapons; monk quarterstaves/
  headband; Kensai bracers; orphan SCS bonus-spell-scroll behavior: older backlog,
  not selected next-release additions. Recheck component numbering:310/320 now
  belong to physical resistance/clouds and cannot be reused from an old plan.
- Player-origin poison kills reportedly missing XP: unconfirmed report, not a
  diagnosed universal poison bug.
- Tempus Holy Power's proposed AC-on-hit rider ([issue1](https://github.com/Chrizhermann/chriz-bg-rebalance/issues/1))
  still needs a balance decision.
- Further dragon HP/aura/encounter work—Thaxll's shadows, Nizid's acid/dispel,
  Saladrex Time Stop/Alacrity and other individual fights—is separate from shipped110/111.

### Concrete compatibility follow-ups, not a blanket guard-removal task

The October8 owner audits identify additional bounded candidates beyond Flail:

- Bracers220 passive effects/ability/donor layouts; Emotion301 irrelevant
  casting/range/school assumptions; arrows130 extra riders/ability variations
  without accidentally multiplying split damage.
- Tempus helper/CLAB count assumptions.
- Clouds320: settle whether75% is a target or a maximum before touching a modded
  cloud already weaker than75%; never silently strengthen it as a “compatibility fix.”
- Modpack Slayer/Shapechange counts; dispel/Keldorn scaling; Sarah's relevant
  embedded-effect formats; Branwen hammer charge assumptions.
- Viconia table handling is an inspection candidate, not a proven defect.

Assign by owning component, preserve unrelated data, and test a few relevant
variations. Do not turn this into a universal-mod compatibility prerequisite.
Evidence: Flail worktree `docs/audits/2026-10-08-sr-rebalance-compatibility.md`
and `docs/audits/2026-10-08-modpack-sod-compatibility.md`.

Evidence: public0.8.1, open PR7, potion owner
`docs/handovers/2026-10-09-potion-uniqueness.md`,
`research/2026-10-09-wish-sr-haste-design.md`, and older owner roadmap.

## 5. Spell Revisions and deliberate spell balance

The source home and chat name differ: some SR balance work is correctly being
designed in **BG Rebalance**, while the SR fork owns upstream fixes and its overhaul.

### Small researched changes waiting for your choices

These are recommendations, **not approved new numbers**:

| Choice | Current SR behavior | Proposed first version |
|---|---|---|
| Luck duration | 300 seconds | 60 seconds; longer than vanilla18 |
| Bless duration | 300 seconds | 60 seconds; vanilla36 |
| Aid | Long level-scaled duration, increased HP | Keep distinct from Heroism;120 seconds, current bonuses |
| Druid Chaotic Commands | Excluded from ordinary druid list | Restore level5 access; no other spell changes |

Next comparisons: Free Action duration; Protection from the Elements duration
and potency; Protection from Energy; druid Stoneskin's fast combat casting.
Do not nerf every long spell merely because its duration is long.
Holy Word/Unholy Word deafness has a source investigation, but no approved change.

### Entropy Shield / Dispelling Screen / enemy AI

Design ready for discussion, not implementation:

- **Spellstrike already removes Entropy Shield.** A separate Spell Shield can
  absorb the first cast; no Entropy-alone failure was reproduced by this audit.
- Choose whether to remove Entropy's blanket Abjuration immunity only, or also
  make Breach remove Entropy itself. Removing the blanket also removes its
  Imprisonment protection; that tradeoff must be explicit.
- Preserve separate Spell Shield's protection unless changing it is deliberately
  chosen. This is not the same question as Entropy's own removability.
- SCS already recognizes Entropy in priest logic. There are narrower candidate
  counter-selection corrections, not a need to teach all AI about it from zero.
- Enemy Screen scarcity includes a **spell allocation problem** under the selected
  IWD-over-SR lists. More casting rules do little for enemies who never prepare it.
  Decide whether eligible enemies may replace an existing fifth-level spell with
  Screen, or only improve casters who already have it. Normal slots/casting apply.
- Older Screen reliability/grace-window and banning dispels in sequencers ideas
  remain undecided. Do not bundle them automatically with this work.

### Important recovered project: configurable SR

Local branch `codex/configurable-sr-catalog` at`3d3b083` contains substantial
catalog, selection resolver, generator and WeiDU compiler work; about45 unique
commits. No active worktree or open PR. It is **not merged into the public release**.
Its ledger has tasks1–5 complete and6–11 pending; migration is still marked
incomplete and the production entry retains legacy component0.

Before proposing “our own SR,” recover/reconcile this branch. Remaining work
includes compilation/parity completion, shared processors, optional-component
integration, migration, packaging and documentation. This is a separate project,
not a fast next-package addition. Old multi-copy test plans are not new authority.

Other preserved work: Death Ward versus physical-vorpal distinction; wider RR/SR
spell-identity/tactical compatibility. Existing Chaos/Selina corrections and
Lightning alternatives are released. SR issues are disabled, so GitHub's empty
issue list is not proof there is no work.

Evidence: Rebalance `research/2026-10-11-general-spell-balance-design.md`;
SR `docs/plans/2026-10-11-entropy-shield-and-screen-design.md`;
SR branch task ledger and `spell_rev/catalog/changes.json`;
`docs/handovers/2026-09-06-sr-death-ward-vorpal.md`.

## 6. Bardic Wonders

- **Ceilings:** released, CEBG integration missing. Blade/Skald/Dancer stop at6;
  most others including Jester at7; Kapellmeister/Darkbloom at8. Levels7/8 unlock
  at bard levels21/29. Existing characters update slots through normal level-up,
  not an automatic rewrite of known spells on load.
- **New Blade/Battleluck:** implemented locally. Two weapon pips for the written
  sword/dagger/axe/spear/halberd list; automatic Two-Weapon Style maximum3.
  Battleluck lasts five rounds for caster/allies within30ft and refreshes without
  stacking, including Gallant's copy. This supersedes the earlier three-pip idea.
- **Heroism:** agreed behavior, missing shared implementation; see Rebalance.
- **Other bard-only spells/long buffs:** inventory and discuss individually; no
  blanket removal approved. Bard Free Action/Chaotic Commands access stays.

Older correctness backlog, **separate from “finish bard ceilings”**:

1. [Darkbloom SR imports #2](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/issues/2): concrete wrong-spell problem; retain SR exclusion until fixed.
2. [Other priest spell clones #3](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/issues/3): New Bard Spells/Troubadour/Deathsinger identity audit.
3. [Gallant order dependency #4](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/issues/4): bounded repair candidate.
4. [Symphony opening/finale invisibility #5](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/issues/5): suspected, not a confirmed new regression.
5. Jester enhanced-song self-refresh; Freedom removing March twice rather than
   Heroism; Lingering Song130/140% discrepancy; Defensive Spin/parry behavior;
   Resonating Weapon overlap; Darkbloom healing/description mismatch; Checkmate
   below16INT fallback; Sound Burst/Mischance/Dirge prose mismatches.
6. Dwarf Gallant CHA minimum versus racial maximum is an older CEBG report to
   reconcile with owner code, not an observed creation freeze.
7. Casting-speed combinations/Kapellmeister restoration remain balance discussion.

Evidence: `docs/bard-spell-blade-followup.md`,
`docs/heroism-family-integration.md`, `docs/bard-balance-audit.md`, and the local
ignored `docs/plans/2026-10-11-bard-design-review.md` (preserve it before cleanup).

## 7. Artisan Kitpack

Released Archer/Assassin/shapeshift-talk work is not waiting for implementation.
Current fork is up to date with its verified upstream head; same for SR and Bardic.

- **Semantic spell identities:** [#4](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/issues/4)
  covers Pale Master, Red Wizard Edwin and Sacred Fist Rasaad. A defined compatibility
  lane. Related [#5](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/issues/5)
  fixtures belong to that delivery, not a standalone indefinite testing project.
- **Favored Soul redesign:** [#3](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/issues/3),
  replacing global priest-delivery changes with kit-specific handling. Larger
  separate work; keep the unsupported choice unavailable meanwhile.
- **Berserker Enrage markers:** [#6](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/issues/6),
  narrow source concern; demonstrated gameplay harm remains unconfirmed.
- **Fighter all-weapon specialization:** still active; decide keep/remove/reduce
  or restrict eligible Fighters. No silent nerf authorized.
- **Mystic Fire:** distinguish Shatter Spell from optional wand/scroll access.
  Deferred design, not a release blocker. Warlock investigation likewise deferred.
- **Older aggregate #1:** reconcile Red Bearskin usability, multiclass proficiency
  entries, animations, Edwin slots and Mazzy pips against upstream and Modpack
  repairs. The open issue does not prove all five remain broken today.
- **Berserker dual-class level14 gate:** older unverified behavior, separate from
  already released3/6/10 normalization.

Evidence: owner README, `docs/plans/2026-08-31-spell-revisions-phase-0-compatibility-design.md`,
`docs/releases/chriz-v1.5.0.md`, Fighter/RedWizard source and archer-worktree
`docs/plans/2026-09-28-mystic-fire-balance.md`.

## 8. SoD Remix

Published0.6.12 and CEBG already include the bridge timer; an older primary
checkout saying0.6.11 is not a reason to redo that release.

- **Early companion recruitment177/198 + Liia fix:** substantial local candidate
  in the dedicated early-recruits worktree. Consolidation/release and focused
  player acceptance remain.
  Liia accepts an unpaid Korlasz reward later, once. Skie is available before
  the first rest; the first recruitment set includes Yeslick, Branwen, Ajantis,
  Coran, Garrick, Kagain, Alora, Xan, Tiax, Kivan and Shar-Teel. Existing living
  actors keep their build/equipment; no resurrection or duplicate imported actors.
- **Interim Avernus finale285:** local candidate in a separate worktree. Keep its
  scope distinct from the already released shortened ending/full-skip features.
  A one-way portal skips lift/waves; Belhifet and Hephernaan have the agreed
  bounded fight changes while confrontation/victory remain. No automatic expansion
  into the long-term Caelar rewrite.
- **Darkness/autoblindness:** user-requested later change; exact desired replacement
  and provider must be chosen. Not implemented just because the request mentions SoD.
- Preserve the remaining owner encounter/companion roadmap and experimental-skip
  limitations; do not require a whole campaign replay for each small release.

The relocated-loot identification fix for900/170 is now implemented locally in
the October12 dispatch above, awaiting SoD integration. Other bounded candidates:
consolidated prologue XP notifications; reconcile opening-delay code in old
[PR12](https://github.com/Chrizhermann/chriz-sod-rebalance/pull/12). Relocated
chest position still needs a choice.

All open owner issues:

- [#4 opening delay](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/4), implemented in the old PR, not yet integrated.
- [#15 Ashatiel party fight](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/15), roster/builds/staging/reset/reward decisions remain.
- [#16 filler review](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/16), initial audit done; further cuts not approved.
- [#17 ending variants](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/17), narrower standalone/party-variant checks, not missing ending code.
- [#18 acquisition history](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/18), durable import-item history still needs design; don't revive the old pickup scanner without approval.
- [#19 skip transition polish](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/19), representative EET route worked, variants remain.

Creative/later queue: round-two Faldorn/Quayle/Eldoth/Xzar/Montaron continuity;
Coldhearth; travel ambush/rest policy; Skie estate/gear; Corwin; long-term Caelar;
Hephernaan story; Cyric temple; deeper Morentherene AI; broader Shadow Aspect;
chapter XP; Korlasz per-run sequencers; optional vignettes/proclamation/dreams.
Khalid and Scrying Pool work already shipped. The last Shadow Aspect summon
sample is explicitly deferred, not a new release gate. de Lancie poison quest
and BG1 Imoen death detection were explicitly left out of scope.

Evidence: early/finale worktree `docs/design/wave1/11-early-bg1-companions.md`
and `11-avernus-interim-finale.md`; root `10-playthrough-followups.md` and
`09-ashatiel-party-brawl.md` in that same design folder.

## 9. Item Randomizer

Public8.1.1 is not the new randomizer. A substantial dirty runtime/configurator
project is in progress; the owner completed its persistent Mode1 playtest setup
during this audit. The repository is now named
[chriz-item-randomiser](https://github.com/Chrizhermann/chriz-item-randomiser),
while the local directory still uses `bgee-itemrandomiser-fix`.
The small public issue list does not capture this work.

- Finish normal packaging/CEBG delivery and the listed remaining Mode2/special-option
  parity. Mode1 runtime/save persistence and a usable playtest already exist;
  don't restart that completed work or confuse the separate dashboard with it.
- The original Mode1 preset already works in the lab:321 rewards, per-game seed,
  saved assignments, shops and several quest deliveries, eight cursed appearances
  restored per save, extra Spell Shield scroll options and shop multipliers.
  A persistent **Configure Randomiser / Apply for next game** playtest is now
  prepared at `C:\Users\chris\Games\EET-IR-Mode1-20260922`; its owner checked
  cold startup and reload. Start BG1/use the provided test save; direct SoD/BG2/ToB
  starts remain unsupported. No need to create another copy merely for this preview.
- Preserve the agreed configurable pools, proficiency compatibility, cursed-item
  variety/per-run appearances and reward-placement scope in the owner's work.
- Generic unidentified-item visuals are **already shipped in Modpack630** and
  must remain exclusive with appearance randomization, not ordinary loot shuffling.
- Simplified weapon groups require coordinated Artisan/companion-pip handling;
  a randomizer-only fix does not establish collection-wide compatibility.
- Keep the new runtime out of the quick release batch if still unfinished;
  it can proceed in parallel without delaying recruitment/spell/UI work.

Remaining normal delivery: TP2/packaging/CEBG integration; original special-option
parity; Mode2 alternate-carrier/remove-regrant routes; original automatic equipment
placement; one reward-specific clue/description route. Item-pack membership,
tiers/chances/destinations/new curses remain creative decisions. The38 candidates
in `docs/item-pack-candidates.md` are not38 approved additions.
No fresh upstream comparison was completed for this repo during the audit.

Evidence: active runtime worktree `docs/campaign-runtime-prototype.md` (top/current
section), `docs/plans/2026-09-22-mode1-postinstall-profiles-design.md`.

## 10. Script engine

A separate longer-running project, not a prerequisite for the next collection
patch. There is a real limited mage-casting prototype and further host/planner/
spatial work, but no public replacement for all SCS behavior.

- Finish one explicit playable slice with the current owner rather than broadening
  it to every requested spell/encounter fix.
- Retain priest Turn work and its distinction between passive observation and
  actually choosing/executing actions.
- Preserve the older casts-cancel-on-sight-loss report and newer perceptive enemy
  spell-dodging request. They live in different branch histories.
- Enemy Entropy/Screen improvements can be a bounded SCS-facing balance change;
  do not reassign this engine chat or make its full rewrite a dependency.

Nearest current milestone: compare the two priest Turn branches to actual legacy
behavior, then decide migration effort before broadening. The load-after-cast
sample was already idle, not proof of loading mid-cast. A movement-endpoint failure
also happens under the legacy control; don't mislabel it as proven new-engine harm.

Longer roadmap remains: WorldFlow/quest-state compatibility, durable quest recovery,
dialogue/quest bridge, script/dialogue representation and lints, one low-risk real
quest migration, extension API, separately selectable WeiDU packaging, practical
compatibility/performance checks and preview release. Task counts are not a
percentage of a complete SCS replacement.
Evidence: active `README.md`, `docs/research/priest-turn-shadow-results.md`,
`docs/plans/2026-08-27-research-and-vertical-slices.md` and both research backlogs.

## 11. BuffBot

1.10.0 is released and pinned. The old `bg-eeex-buffbot-items` worktree is an
already-merged ancestor, not another unfinished item-power implementation.

- Brief real-game check remains for carried-item powers and equipment-swap recovery.
- Configurable one-key Cast All remains a real feature candidate.
- Old export/marshal issue59 and configuration-read issue42 have source changes
  addressing their reported paths. Reconcile/reproduce before treating them as
  unfixed blockers; do not blindly close them from this audit.
- Keep quick-cast/AI-lock and the remaining owner issue list separate from the
  already delivered item work; validate current symptoms before duplicating fixes.

All eight open issues at this snapshot:

| Issue | Next action |
|---|---|
| [65 Cast All hotkey](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/65) | Good bounded feature; choose binding/active-preset behavior, reuse normal casting. |
| [60 clone F12 innate](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/60) | Harder engine routing; current Summons view/Cast All is the workaround. |
| [59 safe export](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/59) | Source sanitizer exists; identify any remaining repro or close with evidence later. |
| [42 failed configuration read](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/42) | Reported fallthrough absent; check callers/logging before creating another fix. |
| [54 multiplayer](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/54) | Separate ownership/feedback/rename lane. |
| [46 UUID/listener modernization](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/46) | Re-scope; summons already work, no mandatory rewrite implied. |
| [34 Tracking classification](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/34) | Small resource investigation; manual override exists. |
| [23 EEex CI](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/23) | Maintenance, not missing gameplay. |

Current1.10.0 release workflows succeeded; an old owner “queued” status is stale.
Evidence: public changelog, `buffbot/BfBotPer.lua:675` and`:785`, current GitHub
issues/workflows. No BuffBot open PR at this snapshot.

## 12. Installer, release and adjacent work

### CEBG work worth finishing now

1. Implement Challenge Mode selection behavior: group on/off/mixed state, preserve
   explicit individual choices, don't disable ordinary balance defaults.
2. Wire released bard ceilings and newly completed owner components, in native
   order with dependencies/exclusions. Snapshot exact owner commits before intake.
3. Prepare one combined installation and separate installer/collection changelogs;
   label default versus optional. Update website only from the actual selected release.
4. Reconcile stale GitHub issues/docs rather than continually reassigning delivered work.

Challenge Mode's wider scope is preserved in the
[complete rule inventory](2026-09-13-challenge-mode-outlook.md#complete-rule-inventory):
wand restrictions/replacements; party traps; combat-only Skull Trap with one-round
expiry; dispelling/explosive arrows; one Wish per combat; one memorized Time Stop;
restricted protection consumables; Ravager; selected-fight exits; no mid-combat
party-dismissal escape; stealing limits; off-screen precasting; Balthazar; and
player-conduct rules. Existing dragon/SoD challenge options belong in the review
too. Most enforcement is not implemented. Preserve the SoD-filler exceptions and
do not attempt an exploit-proof mode before delivering a useful partial group.

### CEBG still open / later

- General installed-set updates, broad current-run/new-run/new-install classification,
  additional hotpatch routes: explicitly lower priority. Existing BuffBot1.8.3→1.8.4
  and description repair remain supported; not a general “update all” system.
- Targeted partial-run repair UI/executor: [issue3](https://github.com/Chrizhermann/chriz-bg-collection/issues/3).
- Intermittent Windows CLI fixture diagnosis: [issue5](https://github.com/Chrizhermann/chriz-bg-collection/issues/5).
  Do not infer a current production failure from an old intermittent test report.
- Open issues1/6/7/8 are substantially stale against shipped utility XP and the
  alpha.19 curation/ledger/profile repairs. Compare each acceptance point, then
  update/close with evidence in a later authorized housekeeping pass.
- Legacy `Chriz BG Collection` app-data migration; owned-shortcut cleanup and
  automatic diagnostic export; documented concurrent-name edge; Documents-folder
  robustness report. Managed deletion/names/size preview already exist.
- Optional simplified proficiencies/CDTweaks2161 and companion pip allocation/
  SCS4100 remain integration work. Preserve class/kit overrides and warn about
  any unavoidable XP/spellbook changes; don't equate this to pip-only behavior.
- Cloud-save opt-out is applied privately, not an approved global installer
  default. Consider an explicit choice/explanation for future installs.
- Vanishing of Skie Silvershield: later inclusion review, no chosen default.
- Older unresolved curation rows remain deliberate exclusions unless revisited;
  old “curate everything” checklists are not permission to enable them.

### Adjacent repositories, not silently forgotten

- **Aura fork:** no public release, one dirty worktree. Upstream gap is now eight
  commits; local bow/Magatama flag changes and earlier balance need reconciliation.
  [Issue1](https://github.com/Chrizhermann/Aura_BG1_BG2_EET-Chriz-Balance-Patch/issues/1)
  is the Spirit Blade/Shirayuri save-for-half bug. Separate owner consolidation
  before CEBG inclusion, not a quick pin update.
- **Radar:** third-party tapahob/Yuna project. Local fork has Twitch direct-redeem/
  random-encounter plans and build-modernization work. Coordinate feature intent
  with its owner; do not confuse this with already shipped optional CEBG launch.
- **Randomizer dashboard:** separate tracker/overlay with its own backlog, not
  the new randomizer configurator. Some open issues already have code. It does not
  establish gameplay randomizer completion.
- **Voice packs:** public1.0.1; local Ajantis2.0.0/Garithos1.1.0 builders/assets
  need reconciliation. Preserve unpublished/private source assets before cleanup.
- **Website:** alpha.19 collection content and SoD subsection already exist.
  Next release needs an accurate content refresh; broader vanilla/SCS comparison,
  roadmap presentation and optional mod catalogue remain separate. Keep short
  gamer-facing rules; no return to verbose internal compatibility notes as rules.

The dashboard has31 open issues and no public release. Preserve the grouped list
without assigning31 fresh implementations:

- Overlay/session/search:#26 minimize/F9,#27 tray/widget,#28 configurable key,
  #18 session/grouping,#32 search.
- Small UX:#6 search,#7 Escape,#8 first-session close,#9 assignment affordance,
  #10 duplicate names,#11 descriptions,#12 grouping,#13 counters,#14 labels,
  #19 favicon,#24 narrow layout,#31 collapsed-state persistence.
- Tracker features:#15 campaign filter,#16 reverse item view,#17 shortcuts,
  #20 undo/redo,#21 bulk actions,#22 progress,#23 scroll toggle,#25 Mode2 locations,
  #30 pool preview.
- Larger:#1 broadcast view,#2 game connector,#3 map pins,#4 accessibility logic,
  #5 multi-device backend.

Keybinding and Escape implementations already exist; reconcile the issues first.
[Dashboard issues](https://github.com/Chrizhermann/bg-item-randomizer-dashboard/issues).

Radar's Twitch plan still needs core ownership/upstream collaboration, roster and
encounter budgets, excess-spawn handling, Bits/offline queue/interior policies
and VM choice. The local `.NET8` build spike is not game-tested public Radar.
Its proposed resref/config-preservation bugs need current-upstream confirmation.
Voicepack source/assets require a publication/provenance decision distinct from
personal use. None of these is a collection-release prerequisite.

## 13. What can run in parallel

These are the broader proposed deliverables. Only the tasks explicitly marked
in the October12 dispatch above have now been started by this chat.

| Lane | Deliverable | Dependencies / boundaries |
|---|---|---|
| A — Modpack delivery | Consolidate/release the implemented recruitment, scroll and banking candidate; banking display/current-run follow-up | One owner integrates TP2/runtime/release changes. Preserve the already composed native hooks and don't replace new code with the old companion branch. |
| B — Bardic delivery | Blade/Battleluck finished and packaged; released ceilings integrated by CEBG | Do not wait for every old song/import issue or Heroism. |
| C — Rebalance completion | Flail PR; usable complete potion component; Heroism provider followed by Bardic adapter | Flail independent. Potion/Heroism share low-level ownership; don't build competing hooks. Time Stop stays a separate scope. |
| D — Small spell decisions | Implement only selected Luck/Bless/Aid/CC adjustments | Awaits exact choices; separate from Entropy. |
| E — Entropy/Screen | Agreed removability and practical enemy allocation/casting change | Awaits decisions; coordinate spell/AI ownership, not full script-engine rewrite. |
| F — SoD consolidation | Merge the two approved local feature branches without losing either | Their shared installer/release files need one integrator, not concurrent blind edits. |
| G — CEBG | Challenge grouping, intake, concise descriptions/defaults, combined package | Can prepare UI now; final pins wait for owner deliverables. |
| H — Existing longer owners | Normal release delivery for the playable Randomizer Mode1 build; script-engine bounded slice | Continue independently; neither automatically blocks next CEBG release. |
| I — Bounded compatibility | Artisan identities; Bardic imports/Gallant ordering | Optional additional workers with non-overlapping ownership; don't expand into every hypothetical mod combination. |

No one needs to start all nine lanes at once. The practical first wave is A/B/F/G,
plus the independent Flail correction, while Christopher answers the small spell
questions. Potion/Heroism and the two larger runtime projects can continue without
turning their incomplete scope into repeated requests for new full installations.

Every brief should state: exact player behavior, repository/worktree, files owned,
what is already decided, one concrete finish condition and any genuine open question.
Do not recycle an unrelated owner chat to fill an available agent slot.

## 14. Christopher's short decision list

**Useful now:**

1. Luck60s, Bless60s, Aid120s with current bonuses, druidCC level5—or different choices?
2. Entropy: normal dispels only, or also removable by Breach? Keep separate Spell
   Shield protection? Permit appropriate enemies to swap a fifth-level spell for Screen?
3. Initial Challenge Mode optional membership beyond Emotion/Baeloth. Existing
   normal Flail/arrows/physical-cap defaults remain unchanged, as already decided.
4. XP display placement; public banking stays optional. Decide the no-bank dual
   arithmetic before claiming general dual-class support, not before Garithos plays.
5. After trying it: include out-of-combat Pathfinding as an optional public choice?

**Can wait:** less-chatter reduction strength/list; Fighter/Mystic Fire changes;
broader bard spell restrictions; Wish enemy-clone scope; potion variant default;
full configurable-SR direction; larger encounter/rule enforcement policy.

Already settled: Heroism first buff stays; potion variants and distinct-type overlap;
scroll non-casters use half active level; Blade two pips/TWF3; Battleluck five rounds;
bard kit ceilings; no automatic move of existing balance defaults into Challenge Mode.

## 15. One combined test install, not a test-install project

First combine the chosen *finished* owner versions. Run ordinary automated checks
for the touched behavior and one recipe/order/default pass. Then one fresh
installation with the intended UI. Community testing can cover other UI choices.

Short player checks tied to what actually enters that build:

- Start, save/reload; existing app/Radar flows need repetition only if changed.
- Recruit Fade/Evandra naturally once; level a Blade/Jester when relevant.
- Use Battleluck; if included, exercise Heroism/potion repeats without free healing.
- For finished scroll scaling, compare one actual caster and one half-level user;
  for small spell changes, inspect availability and one expiration/counter scenario.
- Banking: ordinary cap crossing, saved bank after reload and campaign release.
  Garithos's approved private trial can supply early feedback. No dual experiment
  is required in his current run; general dual claims need their own later check.
- If included, a short SoD recruitment/finale scenario; no full campaign replay.
- Out-of-combat pathfinding: doorway movement and normal combat blocking.
- If useful in the same session, check one BuffBot carried-item power and an
  interrupted equipment swap/save-reload recovery. This is pending acceptance
  of released1.10.0, not a new feature or an automatic blocker for unrelated mods.

Choose a short first smoke session and let ordinary play/community feedback cover
the rest. If a feature is unfinished, leave it out of that batch honestly; do not
call the prototype complete, and do not hold all completed work for it.

## 16. Ownership and evidence map

CEBG active worktree is
`C:\Users\chris\.codex\worktrees\installer-v0-real-alpha\chriz-bg-collection`.
The current chat's `4d39` checkout is an old detached curation snapshot and is
**not** the release integration branch. Preserve its existing user edits.

| Work | Current source / existing chat |
|---|---|
| Modpack XP/scroll/recruitment | Consolidated source `C:\Users\chris\.codex\worktrees\campaign-xp-banking\chriz-bg-modpack`, branch `codex/campaign-scroll-integration`; older source `C:\Users\chris\.codex\worktrees\modpack-release-alpha6\chriz-bg-modpack`; chat **Review Dynaheir and Baeloth spells**, `01a0b92c-da6d-7701-8d51-b0df1a17fe08` |
| Older companion stats | `C:\Users\chris\.codex\worktrees\companion-spellbooks\chriz-bg-modpack` |
| Potion/Time Stop histories | `C:\Users\chris\.codex\worktrees\potion-uniqueness-20261009\chriz-bg-rebalance`; chat **Research Time Stop Haste hooks**, `01a0da17-c8c5-70c0-8029-88c652da7272`; preserve both scopes separately |
| Small spell design | `C:\src\private\chriz-bg-rebalance\.worktrees\sr-antimagic-research`; chat **Review Hardiness changes**, `01a10da6-02db-7842-94a5-23a4fff5618d` |
| Entropy/Screen | `C:\src\private\chriz-spell-revisions-patch\docs\plans\2026-10-11-entropy-shield-and-screen-design.md`; **Find Entropy Shield counters**, `01a0df45-df6a-7cb1-9e6a-0f70435f4419` |
| Bardic new work | `C:\Users\chris\.codex\worktrees\bard-spell-blade-followup\Bardic-Wonders-Chriz-Balance-Patch`; original audit chat **We had some commits in the past days. Check if we need to m…**, `01a07fe8-2fd5-79d3-bb42-4fef15d27fb8` |
| Artisan | `C:\src\private\The-Artisan-s-Kitpack-Chriz-Balance-Patch`; archer worktree retains Mystic Fire plan; **Compare Archer balance**, `01a0c7fa-4c66-79f3-9255-b3fcb0e6a452` |
| SoD recruitment | `C:\Users\chris\.codex\worktrees\early-sod-companions\chriz-sod-rebalance`; **Plan BG1 companion recruitment**, `01a10596-3537-7e91-9d1e-478d749ffd38` |
| SoD finale | `C:\Users\chris\.codex\worktrees\avernus-finale\chriz-sod-rebalance`; **Rework Belifid’s end fight**, `01a0e4cd-cca5-76e0-bb08-9ebbbe82ad30` |
| Randomizer | `C:\Users\chris\.codex\worktrees\mode1-postinstall-profiles\bgee-itemrandomiser-fix`; **Fix randomizer proficiency conflict**, `01a0bb24-3c8e-77b2-849a-b3d3b0300eb0`; owner just delivered persistent Mode1 playtest |
| Script engine | `C:\Users\chris\.codex\worktrees\ff18\chriz-bg-script-engine`; **Begin script engine Task 1**, `01a042de-9380-7a52-9275-11f87682075e`; active owner |
| BuffBot items | **Add equipped item actives**, `01a10db2-9078-7903-b8da-f45d055bf376`; released work, remaining gameplay check |
| Website | **Build interactive BG run page**, `01a061fe-a68c-74b1-8272-5bfa9251a731`; released content reference `C:\Users\chris\.codex\worktrees\bg-release-oct07\twitch-setup-chriz` |

Before deletion/branch cleanup, preserve dirty/untracked/ignored work and establish
which commits are merged. A stale checkout is not necessarily disposable. Never
include private game captures, save data or local asset sources in a public cleanup commit.

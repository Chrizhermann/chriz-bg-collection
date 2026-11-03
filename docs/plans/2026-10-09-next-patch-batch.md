# Next patch batch — Challenge Mode and gameplay fixes

**October 12 status reconciliation:** the
[cross-repository overview](2026-10-12-bg-work-overview.md) is the current status
map. In particular, the integrated Modpack candidate at1eadb92 now contains a
normal packaged reader-level scroll430 implementation, recruitment237/238 and
banking650/651. Earlier partial-hook checkpoints below are historical. Banking
and bard ceilings have since been applied privately; public integration is separate.
Full potion uniqueness is still unfinished. Preserve the decisions below, but
do not reassign already completed implementation from an older checkpoint.

Christopher approved planning this batch on October 9, then assigning execution
to the owning repositories. This is the current planning entry point, not a
claim that the components below have been implemented or released.

## October 12 additions

### Spider immunity to webs

- [ ] Make **all kinds of spiders immune to all kinds of webs**, including
  mod-added spider and web variants. Requested for later; no implementation,
  current-game changes or installer-default decision yet.

### Temple donation helper

- [ ] Add a small QoL component, proposed home `chriz-bg-modpack`, showing the gold
  needed for the **next +1 reputation** on the temple donation screen.
- Suggested interaction: one button fills/donates exactly that amount, with clear
  feedback if gold is insufficient or another donation cannot raise reputation.
  Christopher asked for easier use, not cheaper reputation; preserve the installed
  donation costs and reputation rules unless he requests a separate balance change.
- Read the actual installed rules rather than embedding a vanilla price table.
  During implementation check the default UI first and existing alternative UI support.
  This is a backlog item, not implemented or assigned to an unrelated owner chat.

### Pathfinding Redux for the current Garithos run

Christopher now requests preparation of **out-of-combat-only** movement for the
current game. This supersedes the October 9 hold for this private trial, not for
CEBG's public defaults. [v0.1.10-preview](https://github.com/zach-hopkins/bg-pathfinding-redux/releases/tag/v0.1.10-preview)
adds the requested mode; it is WeiDU component **0** with `OutOfCombatOnly=1`,
not a separate component. Keep `EnemyPrototype=0`. Allied spacing helpers also
pause during combat. Combat is the native battle/damage timer state, including
cooldown; already-overlapping allies can separate, but cannot newly pass through
each other in combat.

Verified release downloaded and prepared outside the game at
`C:\tmp\cebg-pathfinding-ooc-20261012`; its README records source/hash, selected
settings, installation/undo steps and a short user playtest. Target remains
`%USERPROFILE%\Games\Chriz Easy BG\game`. Its executable matches the release's
BG2EE 2.7.3 profile. Installed EEex is 1.2.0; upstream's published 2.7.3 test used
1.3.0, so startup with this exact pairing remains unverified.

**October 12 application:** Christopher explicitly requested installation and
confirmed the game closed. Installed component 0 with the above settings;
WeiDU exit 0. All 444 preceding component identities/order retained, one row added.
Game/EEex/loader executable, dialog.tlk and original latest-save hashes unchanged.
Backup: `%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct12-pathfinding-ooc`.
Only the previously blank loader log setting changed in existing configuration.
No EEex upgrade, save edit, game launch, public recipe edit or publication.
Installed; user startup/movement check pending.

## Decisions already made

- Keep existing recommended defaults. Only Emotion no-stacking moves out of the
  normal selection; Baeloth's legal spellbook stays optional and unchecked.
- Challenge Mode must cover the existing wider set of challenge choices, not
  just those two components. Keep individual choices visible and editable.
- Proceed with Fade/Evandra recruitment XP, reader-level scroll casting, and
  potion-effect non-stacking. Gameplay code stays in its owning repository.
- Current game and saves are outside this batch. The October 9 Garithos repair
  is complete; it is not permission for further private repairs.
- No publication, new live-game test, wholesale default change, or promise of
  hotpatch eligibility follows from this plan. Automated checks should cover
  normal input variations; keep any necessary manual checks short and explicit.

## 1. Challenge Mode selection — CEBG

Present a prominent optional Challenge Mode section with independently selectable
choices. Existing component identities and install order remain authoritative;
this is not a second installation of the same effects.

### Already shipped choices and balance components

| Components | Existing behavior | Next action |
| --- | --- | --- |
| Rebalance 301 | Courage and Hope replace each other | Make optional/off in normal defaults; selected with Challenge Mode. |
| Modpack 235 | Legal Baeloth Sorcerer spells known, SR/non-SR | Already optional; group and select with Challenge Mode. |
| Rebalance 110 | Lethal dragon melee/death-protection adjustments | Existing optional challenge candidate; retain SCS/EEex dependencies. |
| SoD Remix 257 | Extra Insane bridge sequencers | Existing optional challenge candidate; retain base Remix/256 requirements. |
| Rebalance 302, 130 | Flail rebalance; BG2-strength EET elemental arrows | Expose as existing balance choices where useful; keep normal defaults. |
| Rebalance 310 | Party physical resistance diminishes above 80%, maximum 90% | Keep normal default and EEex requirement. |
| Rebalance 210, 211, 220 | Pierce Magic, Spellstrike, SR-haste bracers | Keep normal defaults and actual SR/EEex requirements. |
| Rebalance 320 | Damageable combat cloud forms | Keep normal default; not every balance change makes enemies harder. |
| Rebalance 111 and existing SoD/class/kit changes | Longer Wing Buffet spacing, existing encounter and kit adjustments | Keep prior defaults; do not move whole overhaul bundles merely because some changes affect difficulty. |

Do not claim this table is every rule or every kit nerf. The complete earlier
[rule inventory](2026-09-13-challenge-mode-outlook.md#complete-rule-inventory)
remains part of the plan: wands, party traps, Skull Trap timing/combat limits,
dispelling/explosive arrows, Wish frequency, Time Stop memorization, consumable
protections, Ravager, selected-fight exits, and player-conduct rules. Many are
still designs, not working components. Check future owner deliveries before
labeling them available. Preserve SoD filler exceptions and quest dependencies.

Selection design must keep one source of truth. Turning Challenge Mode off must
not turn off unrelated recommended balance defaults or erase explicit individual
choices without a clear user action. Define group-on/group-off and mixed-state
behavior before implementation. The first bundle's exact optional membership
beyond the explicitly agreed Emotion/Baeloth pair still needs to be displayed
to Christopher, not inferred from a component being a nerf.

Acceptance: normal defaults unchanged except 301; individual choices preserved;
SR off/EEex unavailable/SoD omitted handled honestly; native component declaration
order preserved; both shipped profiles, generator, curation, selection tests and
next changelog agree. No current game or published alpha.19 selection changes.

## 2. Fade and Evandra recruitment XP — Modpack

Restore normal SoA first-recruitment catch-up, using protagonist-XP thresholds
up to 1,250,000 XP. Never reduce an NPC's earned XP or make XP farming possible
through dismissal/rejoining. Preserve normal level-up, class, kit, spellbooks,
proficiencies, equipment and story/romance state. This is not continuous party
XP synchronization and does not copy Imoen 620's separate party-average policy.

Source check: Evandra v2.2's `scripts/bgee.baf:35-124` contains the intended
125k/250k/500k/750k/1m/1.25m ladder, but `setup-evandra.tp2:123-127` compiles
`dialogue/bgee.d`, not that BAF. The effective SoA script lacks the ladder.
Fade's SoA recruitment likewise has no equivalent adjustment. Both have ToB
catch-up handling; preserve it rather than applying a SoA ceiling everywhere.
Evandra 224/225 already ship and intentionally preserve XP; do not rewrite them
as if the Sorcerer/portrait feature caused this defect.

Owner should add only the missing recruitment behavior, not append Evandra's
whole orphan BAF (it contains unrelated romance rules). Use an owned completion
marker, complete evaluation even if no award is needed, and avoid duplicate
awards if upstream or another supported component already supplies catch-up.
Existing-save eligibility requires a separate design; no automatic save edits.

Checks: each threshold boundary, NPC already above target, first join below
minimum, reload/rejoin, already-working upstream behavior, normal and converted
builds, SR/non-SR, BG2EE/EET, ToB preservation, script priority/interactions with
SCS NPC customization, repeated install and uninstall. A brief natural recruitment
check per NPC is the relevant manual test, not console force-joining.

## 3. Scroll casting levels — Modpack

Replace the blanket fixed-level compromise with the reader's relevant caster
level where technically supported. Preserve separate spell-learning behavior,
genuine fixed-level special scrolls, charge consumption, spell target, casting
rules and other mods' effects. Cover Bards, Sorcerers, specialist/Mage classes,
multi/dual classes, dormant dual-class paths, and legitimate non-caster access.

Current component 430 (`lib/cbm_uai_caster_level.tpa`) sets eligible direct-scroll
cast effects to level 15 for everyone, overriding the intended CDTweaks 2170
reader-level behavior. The June EEex 0.11 limitation is historical evidence,
not proof that current EEex cannot support a better implementation. No newer
replacement was found in the bounded owner-source check.

First deliver a small source-backed feasibility proof of the actual scroll-use
hook and caster/target attribution. Do not solve this by globally changing spell
levels, equippable item levels, or every cast. If the necessary hook is missing,
report the exact limitation and a bounded alternative before implementing it.

**Approved non-caster rule:** Christopher chose **half their level**, not fixed
level 15. The owner's later design records the confirmed basis: half the highest
active class level, rounded down, minimum one; ignore dormant dual-class levels.
Actual spellcasters use the relevant active caster level. Do not sum class levels
or borrow casting capability from the wrong spell type. Preserve the spell's
normal scaling caps. Do not claim all readers are covered after testing a Mage only.

Acceptance: actual use path, casts by every supported class family, special
scroll exceptions, linked resource variations, SR/non-SR, save/load, charge and
target preservation. Keep an unsupported engine branch out of the default setup
until the owned behavior is established; do not mask failure with fixed CL15 for
real casters.

## 4. Potion-effect non-stacking — BG Rebalance

Prevent accumulating timed bonuses through repeated potions, including Heroism
and Master Thievery. This concerns effects, not inventory stack sizes. Healing,
antidotes and other instant-use effects should remain usable; permanent and
quest potions need explicit classification rather than blanket removal.

RR component 10 is not a potion-only substitute: it also changes fencing/theft,
covers only Master Thievery/Perception, skips its changes under an Item Revisions
marker, and blocks repeat effects rather than supplying the proposed refresh
policy. CDTweaks 2640 introduces mixing risks such as explosion/death, which
Christopher did not request. Prefer an independent component unless the owner
finds an existing implementation with the exact intended behavior.

**Approved policy:** different potion types may have overlapping bonuses. Only
repeated doses of the same potion must not accumulate. Provide two mutually
exclusive variants:

1. All temporary potion bonuses are unique per potion type.
2. The same rule, except Potions of Genius retain their normal stacking behavior
   so players can raise Intelligence for spell learning.

The exception is specifically Potions of Genius, not every Intelligence potion
or effect. Classify semantically equivalent mod/SCS copies as the same potion
type, without merging distinct potions merely because they affect the same stat.
For example, Heroism plus Power may overlap; Heroism plus Heroism may not. Preserve
normal engine limits rather than forcing distinct effects to stack if they did
not before. Potion plus spell/kit/equipment interactions stay unchanged.

The owner's later handover records the confirmed repeat policy: ignore repeat
Heroism/Power bonuses until expiry (the dose is consumed), and refresh other
safe potion types without accumulating potency or duration. Validate HP/healing,
dispel and expiration behavior; raise any unsafe case rather than introducing
a repeat-healing exploit or HP loss.
Healing, antidotes, permanent gains and quest consumables retain their normal use
in both variants. Which variant is selected with Challenge Mode or normally is
still open; do not choose a new recipe default in the owner implementation.

Inventory current default resources and SCS enemy potion variants. Test reordered
effects, multiple abilities, linked SPL/EFF payloads, already-nonstacking inputs,
SR/non-SR and relevant Item Revisions variations; preserve charges, targeting,
duration, adverse effects and unrelated data. Report any genuinely ambiguous
resource rather than silently claiming all-potion coverage.

## 5. Wish Improved Haste — coordinated spell owner task

Pending Challenge Mode addition: with SR, Wish's haste result should grant SR
Improved Haste rather than vanilla doubled attacks. Preserve the Wish result's
intended party targeting, duration and other effects; non-SR behavior is unchanged.
Reuse the public bracers fix's relevant knowledge, not its self-only/20-second
parameters. This is independent of enforcing one Wish per combat.

Gameplay home: BG Rebalance SR-adjustment family, coordinated with the SR owner.
Verify the current provider and whether an owner already implemented it before
creating another component. Component number/default and delivery timing are not
invented here. Include only after its owner can demonstrate the real Wish path.

## 6. Campaign XP caps with banking — Modpack

Christopher promoted this from deferred work on October 9 and asked for design
and owner execution, aiming for a candidate tomorrow. See the
[dedicated implementation brief](2026-10-09-campaign-xp-banking.md): optional,
off-by-default BG1/SoD caps with a per-character saved XP bank, plus an optional
SoA cap. Current games remain untouched; complete award interception and
save/transition persistence must be established before claiming readiness.

## Assignment and delivery

### Practical hobby-mod scope — latest October 9 direction

**Clarification after the limited potion pilot:** reduce testing/infrastructure,
not the requested feature. The six-potion trial bundle is intermediate evidence,
not an approved replacement for the default-setup potion component. Its two
variants are simply the requested Genius exception choices; they do not require
Christopher to accept two new full game copies or a separate lab project.
Christopher will test himself later. Game tests and new copied test installations
are deferred; keep prepared bundles untouched and continue feasible source work.
No agent-run game testing, installation into a game or new copy creation is
authorized. Report what the actual component still lacks, not a completed pilot
as if the feature were finished.

Christopher explicitly rejects turning these features into exhaustive engine or
compatibility projects. Prioritize a playable version in the default CEBG/EET
setup, a short focused playtest and clear limitations. Common, inexpensive
compatibility remains welcome; hypothetical mod combinations, remote/network
paths, general-purpose recovery frameworks and repeated review cycles are not
prerequisites for the first usable version. Minor documented quirks may be fixed
later. This takes precedence over exhaustive interpretations of the older briefs.

Reuse existing regression evidence. Run affected checks and proportionate final
integration checks, not every suite after every small edit. Do not spawn more
research/review work unless it resolves a named obstacle on the actual target.
When the next useful evidence is a small native playtest, prepare that test and
request its approval instead of substituting more offline proof indefinitely.

Keep concrete crash/memory/bounds and XP/save-preservation checks; do not turn
reference-install fingerprints or hypothetical incompatibilities into arbitrary
requirements. Choose simple existing mod mechanisms when they meet the agreed
behavior. If a small gameplay simplification would remove most of the engineering
work, explain the tradeoff for Christopher to choose rather than silently changing
the feature or building an unnecessarily general framework.

In particular, XP banking's deferred old-class level-ups really extend vanilla
dual-class behavior. Separate that cost from ordinary banking and offer a scoped
first-version alternative if useful; do not silently drop the agreed behavior.
A complete automated uninstall/recovery system or every other UI/mod combination
is not a prerequisite for a clearly limited local trial. Unsupported operations
must remain explicit; this does not authorize losing stored XP or editing saves.
No game launch, current-game change or release authority is added by this scope reset.

Both owners acknowledged the reset. Modpack reports banking's cap, persistence
and agreed dual-class paths assembled, with scroll startup wiring still needed;
its next deliverable is a disposable default-setup trial bundle. Rebalance is
restricting the next bundle to real default-install potion families plus usable
startup/install wiring. Neither report establishes in-game acceptance.

A potion simplification has been put to Christopher: ignore duplicate temporary
bonuses until expiry instead of refreshing safe types. Healing/cures remain
normal and the Genius-exception variant remains. No answer is recorded here;
keep the existing approved policy until he chooses. Do not hold unrelated startup
or default-resource work for this question.

### Completion standard clarified by Christopher, October 9

The assignment is a usable, complete local component, not a succession of
prototype reports. Continue implementation, owned-repository integration and
proportionate verification while authorized work remains. A completed worker
subtask or a large passing test count is not a reason for the owner to end with
ordinary implementation left undone. Report actual player behavior delivered,
unfinished behavior and concrete blockers before test totals.

Stop when the agreed deliverable is complete or a real missing user decision,
permission or external prerequisite prevents further progress; first continue
any independent work. Preserve safety/compatibility checks and honest evidence
boundaries. This does not authorize game launches, game/save edits, publication,
CEBG recipe changes or replacing native acceptance with simulated test claims.

### October 9 owner checkpoint and continuation

- **Fade/Evandra XP:** implemented locally as Modpack 237/238, commit `4687538`
  on `codex/recruitment-xp-scroll-levels`. Owner evidence reports 50 focused
  checks, 10 public-surface/release checks and six copied-source install/uninstall
  fixtures. Natural in-game recruitment remains untested. This is for actors
  not yet created in the save, not a repair of existing saved companions. Source:
  `%USERPROFILE%\.codex\worktrees\modpack-release-alpha6\chriz-bg-modpack\docs\recruitment-xp.md`.
- **XP banking:** locally implemented accounting now includes the agreed dual
  split, storage and cap-hook prototypes (480 focused checks reported passing).
  Real engine integration, coherent save/identity handling and actual deferred
  old-class rewards remain unfinished; no selectable component exists. After
  completing that milestone, the owner was assigned continuation again. The
  separate question about an ordinary dual with no bank is still open; it must
  not reopen the settled immediate dual/class-owned bank policy or halt
  independent work. See the current owner research and banking brief.
- **Reader-level scrolls:** `808c5f8` adds compiled original-reader/cast tracking
  and native binding after `b6daaa9`'s selector evidence. Actual engine detours,
  resource classification and pending-cast persistence remain unfinished;
  component 430 is still the fixed-level implementation. Continue alongside
  banking, preserving targeting,
  reflection and charges. The settled fallback is half the highest active class
  level, rounded down, minimum one; ignore dormant dual-class levels. Source:
  `%USERPROFILE%\.codex\worktrees\modpack-release-alpha6\chriz-bg-modpack\research\2026-10-09-scroll-caster-levels.md`.
- **Potion uniqueness:** the original-effect replacement now implements both
  policy modes for a narrow direct-effect subset (106 offline checks and 44
  connected instruction cases reported passing). Broad/linked coverage,
  watchdog-preserving hook compatibility, native acceptance and usable packaging
  remain unfinished. The three failing rejected-draft repros are retained, not
  failures of the new runtime. The owner was assigned concrete continuation
  rather than treating this prototype milestone as delivery. Its handover records
  the confirmed exception: repeat Heroism/Power bonuses
  are ignored until expiry; refresh other types only where safe. Source:
  `%USERPROFILE%\.codex\worktrees\potion-uniqueness-20261009\chriz-bg-rebalance\docs\handovers\2026-10-09-potion-uniqueness.md`.
- **Wish:** the requested source/design check exists in the potion worktree's
  `research/2026-10-09-wish-sr-haste-design.md`. Player and enemy Wish paths need
  an explicit scope decision before implementation; no new authority inferred.

Both owner follow-ups were delivered and both chats were active afterward.
This checkpoint does not promise overnight completion, publication or automatic
CEBG inclusion. Review owner delivery before updating pins and selections.

### Pathfinding Redux — on hold because of allied pass-through

Historical October 9 review. The October 12 preparation above supersedes this
hold for Garithos's out-of-combat-only trial; public/default inclusion stays undecided.

Christopher requested an evaluation on October 9, not installation or default
selection. Reviewed [v0.1.5-preview](https://github.com/zach-hopkins/bg-pathfinding-redux/releases/tag/v0.1.5-preview),
released October 8. The mod lets allies/friendly summons pass through each other,
with separately switchable spacing/settling/route features. This can ease combat
retreats as well as doorway congestion; it is not solely a pathfinding correction.
See the [author's feature description](https://github.com/zach-hopkins/bg-pathfinding-redux#features).

Christopher's follow-up condition: allied pass-through is too strong for his
intended setup if it cannot be skipped. Code inspection of all four release
profiles shows that `Movement=0` also stops AttackSpacing, GentleSettle and
RoutePreference: their preferences remain configurable, but their execution
requires the shared movement/pass-through gate. See the
[master gate](https://github.com/zach-hopkins/bg-pathfinding-redux/blob/v0.1.5-preview/bg-redux-movement/runtime/profiles/bg2ee-2.6.6.0.lua#L2133-L2149),
[attack gate](https://github.com/zach-hopkins/bg-pathfinding-redux/blob/v0.1.5-preview/bg-redux-movement/runtime/profiles/bg2ee-2.6.6.0.lua#L1418-L1422),
[settle tick](https://github.com/zach-hopkins/bg-pathfinding-redux/blob/v0.1.5-preview/bg-redux-movement/runtime/profiles/bg2ee-2.6.6.0.lua#L2261-L2267)
and [route gate](https://github.com/zach-hopkins/bg-pathfinding-redux/blob/v0.1.5-preview/bg-redux-movement/runtime/profiles/bg2ee-2.6.6.0.lua#L3449-L3458).
There is no useful collision-preserving configuration in this release. Leave it
out of the planned install; reconsider only if that changes or Christopher
explicitly accepts the combat-positioning change. This is source evidence, not
a live test of switching modes during an existing movement command.

The [installation guide](https://github.com/zach-hopkins/bg-pathfinding-redux/blob/main/docs/INSTALLATION.md)
records BG2EE 2.6.6 + EEex 1.2 and BG2EE 2.7.3 + EEex 1.3 checks; EET gameplay
acceptance is still pending and multiplayer is unsupported. CEBG's current base
manifest pins EEex 1.2: verify the actual executable/loader combination first.
Use a disposable full install/save copy, game closed, normal WeiDU installation
after EEex and normal uninstall afterward. Retain native compatibility guards.

If reconsidered, proposed brief playtest: crowded doorway, retreat through allies, hostile/neutral
blockers, melee approach, summons, area transition and save/reload. Compare the
optional features independently if movement feels wrong. No trial, recipe edit,
download, game launch or native compatibility proof was performed in this review.

Three read-only preparation agents checked XP, scrolls and potions during this
planning turn. They did not implement the features. After Christopher settled
the non-caster scroll and potion policies, execution briefs were sent to the
existing owner chats on October 9:

- `Review Dynaheir and Baeloth spells`
  (`01a0b92c-da6d-7701-8d51-b0df1a17fe08`): implement XP catch-up, then prove the
  scroll-use hook and implement reader scaling where feasible. Return material
  unresolved multi/dual-class rules instead of silently choosing them.
- `Research Time Stop Haste hooks`
  (`01a0da17-c8c5-70c0-8029-88c652da7272`): leave its old engine research deferred;
  implement the two potion variants, then perform a source/design check only of
  Wish's SR Improved Haste result.

Dispatch is not completion or acceptance. Review their final results and tests
before CEBG integration. The CEBG Challenge Mode section remains design work;
neither owner may change collection defaults or published profiles. Owner briefs
include the settled policy, bounded tests, compatibility preservation and the
current released source baseline. Some primary local checkouts are stale
development branches; preserve their unrelated work and use a suitable owner
worktree instead of blindly editing the primary directory.

Modpack owns XP and scrolls, but their integration edits should be serialized or
kept in separate branches. BG Rebalance owns potion balance and the coordinated
Wish adjustment. CEBG owns selection, dependency/default checks, version pins
and accurate changelog/UI copy. Do not publish a new mod or CEBG release as part
of a planning/implementation handoff without explicit release authority.

As of October 9, still deferred: general installed-set updates/hotpatch expansion,
Time Stop/Haste engine work, Entropy Shield/Dispelling Screen redesign, and new
Fighter/Mystic Fire balance. The October 11 section below resumes only the
specified spell/bard design and research. Less companion chatter has its own approved direction
but still needs a concrete reduction list; do not silently fold dialogue cuts
into this batch.

## October 11 — spell and bard design assignments

Christopher requested that the relevant existing owner chats start design and
research, returning open questions rather than guessing scope or values. This
is not implementation, live-game/save editing, test-install, release or recipe
authorization. Keep research practical and source-backed; no exhaustive lab
programme. Existing approved changes remain intact. The earlier Time Stop/potion
scope mix-up must not be repeated: neither that chat nor its work was reassigned.

### Requested changes and unresolved choices

- **Heroism always unique:** settled October 11 after the owner discussion:
  Heroism, Greater Heroism and Potion of Heroism exclude one another; the first
  active buff remains until expiry and subsequent applications add nothing.
  Song of Heroism and Aid remain outside. Decision sent to the Bardic owner for
  its pending local implementation; no live-game or release approval inferred.
- **Aid:** compare replacement with Heroism against a smaller nerf; no option
  chosen yet.
- **Other bard-only spells / long buffs:** inventory the actual spells and
  propose removal/restriction versus shorter buffs. No blanket deletion approved.
- **Finish bard rebalance:** Christopher clarified this means the different
  bard kits' SPELL-LEVEL CEILINGS, not the old song/spin/import backlog. Provider
  420, base policy 421 and Bardic adapter 3010 already exist; verify delivery and
  integration rather than expanding that phrase into unrelated repairs.
- **Blade:** subsequent owner-chat decisions supersede the initial three-pip
  proposal: two weapon pips in swords, daggers, axes, spears and halberds;
  automatic Two-Weapon Style capped at three. Owner reports local implementation
  with Battleluck (five rounds, caster/allies within 30 feet, self-refresh).
  Neither is released or applied to Garithos by this coordination work.
- **Luck:** shorter than our current version, longer than vanilla; duration open.
- **Bless:** shorter duration; amount/level scaling open.
- **Chaotic Commands for druids:** check current access and smallest change;
  unresolved level/access decisions come back to Christopher.
- **Other spell durations / major SR differences:** focused vanilla-versus-SR-
  versus-current-fork comparison and a short recommendation list.
- **Our own spell revision approach:** assess patch layer versus maintaining the
  fork versus larger replacement. No wholesale rewrite approved.
- **Entropy Shield:** weaken its resistance to removal and offer more counters.
  Exact dispelling rules and any further numerical nerfs remain open.
- **Spellstrike must counter Entropy Shield:** reconcile the older source report
  that it already can with current resources/protection layers; do not assume a
  new effect is needed without identifying the problem.
- **Enemy AI:** research recognizing/removing Entropy Shield and casting
  Dispelling Screen on allies more often. Use normal spell availability/casting
  as the design baseline; no free casts or unrelated prebuff/engine rewrite.

### Owner briefs delivered

| Existing chat | Scope | Thread ID |
|---|---|---|
| Review Hardiness changes | General SR balance: Aid, Luck/Bless, druid access, other major differences and patch/fork options | `01a10da6-02db-7842-94a5-23a4fff5618d` |
| Find Entropy Shield counters | Entropy Shield, Spellstrike counter behavior, enemy dispelling and party Screen AI | `01a0df45-df6a-7cb1-9e6a-0f70435f4419` |
| We had some commits in the past days. Check if we need to m… | Original full Bardic audit owner: unfinished bard rebalance, Heroism family, bard spells and Blade | `01a07fe8-2fd5-79d3-bb42-4fef15d27fb8` |
| Compare Archer balance | Bounded Artisan interaction check only; no duplicate Bardic redesign or unrelated kit work | `01a0c7fa-4c66-79f3-9255-b3fcb0e6a452` |

Read the current owner source rather than treating old chat release status as
current. Bardic's existing entry points are
`C:\src\private\Bardic-Wonders-Chriz-Balance-Patch\docs\bard-balance-audit.md`
and `BardicWonders/lib/blade.tpa`. General spell research should account for
the later BG Rebalance Hardiness/Pierce Magic/Spellstrike components as well as SR.

Each owner was asked for current versus proposed behavior, source pointers,
repository/component ownership, and a short prioritized question list with
recommendations. They may save a small local design note, but must not implement
this new batch, publish, or modify games. Dispatch is not finished research or
acceptance; collect their results before seeking implementation approval.

### Garithos ceilings — October 12 installation follow-up

Christopher approved applying the prepared features to his current installation,
with in-game testing afterwards. Installed Modpack650 (BG1/SoD caps with banking,
not optional SoA651), Rebalance420/421 and Bardic3010. Backed up latest save
`000000990-NEXT IRON THRONE LOSERS`; original save files remain unchanged.
All445 prior component identities/order preserved; four requested additions.
Existing XP/levels are kept; normal next level-up applies bard capacity.

Do not close the remaining work merely because installation succeeded: startup,
ordinary cap-crossing award, save/reload, campaign release, and bard level-up
checks remain pending. The between-threshold unbanked-dual rule and safe bank
removal/migration remain deferred work; no dual-class experiment is requested for
Garithos. Public integration and the rest of this batch are not completed by this
private install. Detailed continuing checklist:
`%USERPROFILE%\Games\Chriz Easy BG\repair-backups\oct12-xp-bard-ceilings\README.md`.

### Garithos installed ceilings — October 11 read-only check (historical)

Target: `%USERPROFILE%\Games\Chriz Easy BG\game`. This check made no game or
save changes. Bardic provider/policy/adapter 420/421/3010 are absent from WeiDU.log;
CBRSPKIT/CBRSPVER/CBRSPAPI resources are absent. CDTweaks 2270 is installed and the
effective MXSPLBRD grants seventh-level spells at bard level 21 and eighth-level
spells at 29. The per-kit ceilings therefore are NOT active in this installation.
The existing provider supports normal level-up recalculation on an existing run,
but does not automatically rewrite already-saved spellbooks on load.

Separately, XP banking/campaign caps are not installed here: CDTweaks 2090 removes
the XP cap, all XPCAP class rows are -1, and CAMPAIGN has no BG1/SoD/SoA/ToB cap
override. Existing story-based utility XP is reward scaling, not an XP ceiling.

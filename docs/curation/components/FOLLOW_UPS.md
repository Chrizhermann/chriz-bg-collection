# Curation follow-ups

Last consolidated: 2026-10-17, against the next-release branch
`claude/cebg-installer-fixes-20261014` (previously 2026-09-04).

This is the single revisit queue for unresolved curation work. Component choices remain
authoritative in their individual catalogs; this file does not create or change a Chris
curation decision. Keep detailed reasoning in the linked catalog and check an item here
only when the required artifact, static validation, or live evidence actually exists.

Items under **Release gates** block the corresponding selected feature or first complete
stack. Items under **Conditional/future gates** do not block release while that feature
remains unavailable or excluded. Items finished since the previous consolidation are
listed with their evidence under [Done since 2026-09-04](#done-since-2026-09-04).

## Work queue at a glance — 2026-10-26

Cross-repository status. Component detail stays in the sections below and in the catalogs.

**In progress (separate sessions)**

- CEBG alpha.22, the next release (Christopher's plan of 2026-10-25), in this order:
  1. One pin batch: Modpack alpha.14 (release candidate ready, see below), BG Rebalance
     v0.8.5, Spell Revisions fork v4.21-chriz.7, BuffBot v1.11.0, the Artisan's Kitpack
     fork `v4.81a-dev.85928f9-chriz.2` (sorcerer weapon fix, below), EEex `9` (Attack
     Info) optional and off, and maybe the Campaign Item Randomiser (below). Time Clocks is
     likely in the release too (see Ready to start). BG Rebalance, Spell Revisions,
     BuffBot, the Kitpack and EEex `9` are pinned in both recipes on
     `claude/pin-alpha22`, stacked on PRs #32 and #33 (fresh-download and native-order
     evidence in `manifest/evidence/release-intake-20261026.toml`); Modpack alpha.14
     follows there once it is published.
  2. Issues [#7](https://github.com/Chrizhermann/chriz-bg-collection/issues/7) and
     [#8](https://github.com/Chrizhermann/chriz-bg-collection/issues/8).
  3. A full lab run of the Recommended selection.
  4. Acceptance, build, signing and publishing, and the website.
- EEex v1.3.1 everywhere (Christopher, 2026-10-18: test as soon as possible and use it
  everywhere): pinned in both recipes since
  [PR #18](https://github.com/Chrizhermann/chriz-bg-collection/pull/18) (merged on
  2026-10-20) with fresh-download and native-order evidence. On 2026-10-18 a disposable
  2.7.3.0 lab with the full recipe booted every EEex user cleanly; a new BG1 game in that
  lab then exercised the EEex modules, Remote Console and BuffBot (see Done).
  Remote Console v0.2.1 with [eeex-remote-console#3](https://github.com/Chrizhermann/eeex-remote-console/pull/3)
  and the `START_SP` fix [eeex-remote-console#4](https://github.com/Chrizhermann/eeex-remote-console/pull/4) is pinned (below). Open from that lab: Modpack
  `430` on v1.3.1 (see Release gates). The creature `LOCALS` lost after save/load
  ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19)) were not an EEex fault: Modpack `650` caused them (see Modpack
  alpha.14 below); `650` works on v1.3.1 since alpha.13.
  - Attack Info `9` (Christopher, 2026-10-25): optional and off by default; players can
    also switch it on in EEex's in-game options. `claude/pin-alpha22` offers it that way,
    in EEex's own order directly after `1`; static checks only, not installed in a lab
    yet.
  - Still open: how existing installations move from v1.2.0 (see Waiting for
    Christopher).
- SoD Remix v0.6.15, released on 2026-10-23, pinned in both recipes since
  [PR #27](https://github.com/Chrizhermann/chriz-bg-collection/pull/27) (merged on 2026-10-23; fresh-download evidence in
  `manifest/evidence/release-intake-20261023.toml`):
  - Imoen's handover from BG1/SoD into SoA (`292`/`293`/`294`, since v0.6.14): one
    choice group with the vanilla Thief→Mage dual `292` as the default and `293`/`294`
    selectable (Christopher, 2026-10-16: three choices, each working for every thief
    kit). The owner lab covered `C0TRICKS` and no kit, with Spell Revisions only.
  - `256` × Tweaks Anthology `3500` ([chriz-sod-rebalance#31](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/31)):
    fixed in v0.6.15 (option 1, Christopher's choice of 2026-10-18); the combined
    Recommended lab install with `3500` on is still a release gate (see Release gates).
  - [chriz-sod-rebalance#34](https://github.com/Chrizhermann/chriz-sod-rebalance/pull/34),
    merged on 2026-10-23: `265` and six other components read timer names from the
    game's own `GTIMES.IDS`, so a mod that gives a timer value another name no longer
    stops them. It ships with the next SoD Remix release.
- BuffBot v1.10.1 (released on 2026-10-20) and EEex Remote Console v0.2.1 (tagged on
  2026-10-23), pinned in both recipes since
  [PR #28](https://github.com/Chrizhermann/chriz-bg-collection/pull/28) (merged on 2026-10-23; fresh-download
  evidence in `manifest/evidence/release-intake-20261023-buffbot-eeexremote.toml`):
  - BuffBot v1.10.1 shows the combat-stop notice, which earlier versions always lost
    ([bg-eeex-buffbot#76](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/76)). Its
    components `1` and `0` are unchanged, and the run stays after the voicepacks and
    before the scroll policy `430`.
  - Remote Console v0.2.1 carries the EEex v1.2/v1.3 guidance and the `START_SP` fix.
    Component `0` is unchanged and stays optional.
  - BuffBot v1.11.0 (released on 2026-10-24) replaces v1.10.1 on `claude/pin-alpha22`
    (alpha.22 pin batch, Christopher, 2026-10-25). Components `1` and `0`, their order and
    the run position are unchanged. Not run in a collection lab yet; its own validation
    ran without Modpack `430` (see Live/runtime acceptance).
- Modpack v0.2.0-alpha.13, released on 2026-10-20: campaign XP caps `650`/`651` work
  again (alpha.11 and alpha.12 refused every permanent XP award with `650`), lab-verified
  on 2.7.3.0 with EEex v1.3.1. It adds EET join XP `239` for NPCs without EET's
  `BD_JOINXP` block (see Sarah below), which the collection offers as a default (decided
  2026-10-18). Pinned in both recipes since
  [PR #23](https://github.com/Chrizhermann/chriz-bg-collection/pull/23) (merged on 2026-10-22; fresh-download and
  native-order evidence in `manifest/evidence/release-intake-20261020.toml`); `650`/`651`
  stay optional, experimental and off.
- Modpack alpha.14 (not released yet; the collection pins it in the alpha.22 batch):
  - Release candidate ready on 2026-10-26, on the Modpack's private archive only. It
    merges the `430` and `650` work onto the released alpha.13. The full suite passes:
    3,050 tests, two byte-identical release builds, and the archive's own checks.
  - A combined lab smoke with the candidate package (2.7.3.0, EEex v1.3.1) passed: a
    normal start without a box, a scroll at the reader's level, an XP award banked, and
    the #19 check (script variables survived a save made while the bank refused, in the
    same and in a new game process).
  - Publishing waits for Christopher's OK (see Waiting for Christopher).
  - Christopher, 2026-10-25: the collection offers `135` (Dorn ambush camera fix) and
    `460` (party AI Defensive Stance obeys "Use special abilities") as defaults, and
    `652` (chapter-by-chapter XP caps) as an option that starts off, because it needs
    `650`. They reach the catalogs and recipes with the alpha.14 pin.
  - `650` caused [#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19): up to alpha.13 its XP effect (opcode 104) returned 0 when the
    bank could not take a change, so the engine stopped resolving the rest of that
    creature's timed effect list. The effects queued behind it stayed frozen, and the
    creature `LOCALS` they set were lost at the next save. EEex is not the cause.
    - The never-return-0 fix is done for alpha.14. With it, the #19 scenario passed in a
      disposable lab on 2026-10-23: the locals and all local-variable effects survived
      save/load in the same process and after a restart, also with refused XP changes.
    - Accepted rare cases (Christopher, 2026-10-25): when the bank cannot tell what an
      XP change stored, that change can end without adding XP, and the character then
      gets XP the normal way, without the campaign cap, also after saving and loading.
      If a character were ever created or removed outside the game's main thread (no
      known way on 2.7.3.0), banking stops for everyone until the game restarts, a
      script XP change caught at that moment can be lost, and saves made after that keep
      those characters out of banking.
    - `650` stays optional, experimental and off in the collection. A fifth review round
      follows after the release.
    - #19 is closed once `650` ships.
  - `430` (scrolls at the reader's level): plan A and the last small round are done (see
    Release gates). Christopher, 2026-10-25: the collection offers `430` as a default. It
    reaches the catalogs and recipes with the alpha.14 pin.
  - No technical notices inside the game: fallback information from our mods goes to the
    log only (Christopher, 2026-10-23, confirmed 2026-10-25). `650` and `430` now do
    this. The one exception (Christopher, 2026-10-25) is in `430`. If it cannot start
    cleanly and cannot fully undo its start, one short box asks the player to restart the
    game before loading or saving. The box appears while the game starts, just before the
    main menu, not on the visible main menu; whether that placement is fine is waiting for
    Christopher.
- Artisan's Kitpack fork `v4.81a-dev.85928f9-chriz.2`, released on 2026-10-26:
  - The bug: sorcerer kits, the Dragon Disciple included, could not equip daggers,
    quarterstaves or darts without a proficiency point. It was reported by stream viewers.
    Every fork release since chriz-v1.0.0 has it, including the alpha.21 pin.
  - The cause: upstream commit 7464043 builds the class weapon grants from `WEAPPROF.2DA`,
    which has no sorcerer column. The fix gives sorcerers the mage column.
  - Lab-verified on 2.7.3.0 with EEex v1.3.1: sorcerer, Dragon Disciple and mage, after
    joining, after a level-up and after a reload. Saves need no change.
  - Pinned for all three Kitpack installers on `claude/pin-alpha22` (alpha.22 pin
    batch); components, order and decisions are unchanged. Christopher posts the
    upstream report himself.
  - Existing installations get the fix only from the fork's `AKCB_SORCERER_WEAPONS`
    tail patch (in the release archive), which CEBG does not install; how they get it
    is open.
  - A separate, older Kitpack bug showed up in the same lab: Eldritch Knight and Arcane
    Trickster sorcerers lose their kit's extra weapons at every level-up until the next
    reload or area change. A fix is waiting for Christopher.
- ChrizFader's voicepacks v1.1.0, released on 2026-10-22: all 18 player voicepacks are
  in both recipes since [PR #24](https://github.com/Chrizhermann/chriz-bg-collection/pull/24) (merged on 2026-10-22) as defaults under "Voicepacks"
  (Christopher's decision), one catalog each (`CD_*.md`), in their own runs after the
  other post-EET writers and before BuffBot. Fresh-download evidence:
  `manifest/evidence/release-intake-20261022.toml`. Still needed before the next CEBG
  release: one combined lab install (see Release gates).
- Optional quest mods (Christopher's decisions of 2026-10-23), in both recipes on
  `claude/optional-mods-20261022` ([PR #32](https://github.com/Chrizhermann/chriz-bg-collection/pull/32); fresh-download and native-order evidence in
  `manifest/evidence/release-intake-20261023-quest-mods.toml`): Northern Tales of the
  Sword Coast (upstream `master` f0ef953, not v5.0.0) and Ascalon's Questpack v7.0 are in
  the new category "Quest mods"; Thalantyr Item Upgrade v4.5.1 is in "Items", the Icewind
  Dale Voice Pack 1.2.0 in "Audio" and Portraits Portraits Everywhere (DaftHunk fork
  417226ea, without `310`/`320`) in "Visuals" (Christopher, 2026-10-25: existing
  categories where they fit). All are optional, unchecked, experimental and offered as is.
  - BG1 Unfinished Business is now optional and unselected, like UB for BG2. NTotSC
    `4`/`6` and BG1UB `30`/`32` are either-or; with both mods ticked, BG1UB's version is
    kept.
  - Black Hearts v2.05 is offered with a declared incompatibility: it is unavailable while
    the SoD Remix is selected.
  - Still needed: the component defaults inside the mods (see Waiting for Christopher) and
    one combined lab install (see Release gates).
- Hobgoblin challenge run (design with Christopher).
- Campaign Item Randomiser (CIR) v1.0.2 (released on 2026-10-24): it replaces Randomiser
  v8.1.1 only after CEBG integration and a lab test (Christopher, 2026-10-25). Its next
  version is in progress: draft
  [chriz-item-randomiser#23](https://github.com/Chrizhermann/chriz-item-randomiser/pull/23)
  adds the Sahuagin treasury to the extended pools; the lab play test is pending.
- Vanguard Defensive Stance AI toggle: the fix shipped as Modpack `460` in alpha.13; the
  collection offers it as a default with the alpha.14 pin (Christopher, 2026-10-25).
- InfinityLoader crash analysis: causes found; follow-up decisions are Christopher's.
- BG Rebalance and the Spell Revisions fork:
  - The seven compatibility follow-ups from the October 8 audit
    (`docs/audits/2026-10-08-sr-rebalance-compatibility.md` in BG Rebalance) are
    released: BG Rebalance v0.8.4 (2026-10-23,
    [PR #10](https://github.com/Chrizhermann/chriz-bg-rebalance/pull/10)) and the fork's
    v4.21-chriz.6 (2026-10-23,
    [PR #2](https://github.com/Chrizhermann/chriz-spell-revisions-patch/pull/2) with the
    lab correction [PR #3](https://github.com/Chrizhermann/chriz-spell-revisions-patch/pull/3)).
    v4.21-chriz.7 (2026-10-24) corrects only its update notes. Cloud forms (`320`): 75%
    stays the target for immunity, not a cap (decided 2026-10-20).
  - v0.8.4 also fixes Divine Protection under the party physical-resistance cap `310`:
    short immunity windows stay uncapped
    ([PR #11](https://github.com/Chrizhermann/chriz-bg-rebalance/pull/11)).
  - v0.8.5 (released on 2026-10-25) adds `340` Chris Revisions
    ([PR #13](https://github.com/Chrizhermann/chriz-bg-rebalance/pull/13), merged on
    2026-10-25; Christopher's decisions of 2026-10-23 and 2026-10-25): Aid gives only hit
    points, Luck lasts 90 seconds, Bless 30 seconds + 1 round per level, druids can cast
    Bless and Chaotic Commands again, and Prayer and Recitation no longer stack. Not run
    in a game yet. The collection offers `340` as a default (Christopher, 2026-10-25).
  - Pinned in both recipes on `claude/pin-alpha22` (alpha.22 pin batch): Rebalance
    v0.8.5, with `340` as a default at the end of its run, and the fork's v4.21-chriz.7
    with `srcb-rr-compat` from the same release. All other components, decisions and
    their order are unchanged.
  - Open in-game checks: the fork's level-5 replacements for Rogue Rebalancing, and BG
    Rebalance `220` as released and `400`-`403`.
- Icewind Dale: Enhanced Edition installer on the CEBG framework, for Christopher's
  November stream (2026-10-25; urgent, in its own session): only a few mods, plus a
  small own mod with SCS-style scripts for selected IWD fights, because the script
  engine (Larger projects) will not be ready in time.

**Ready to start**

- Time Clocks, the Time Stop/Haste timing fix as a standalone EEex mod: likely in the
  next release (Christopher, 2026-10-25). Its repository is not public yet; that needs
  Christopher's go. The engine research and a disposable 2.7.3.0 lab on EEex v1.3.1 are
  on BG Rebalance branch `claude/timestop-lab-eeex-131`
  (`research/10-timestop-haste-engine-clocks.md` and `research/timestop-lab/`), unmerged.
  Native captures and a first correction prototype for one test actor and two test effects
  ran there. Through Time Stop the buff kept its remaining time and the regeneration all its
  healing; under Haste the healing kept its normal rate. Christopher closed that step on
  2026-10-24. The next lab run takes two one-tick Haste residuals (R7, R8), attack and
  movement rates under Haste, Improved and SR-shaped Haste, a pause during the correction,
  consecutive stops, explicit removal and save/load.
- Remaining compatibility follow-ups from the October 8 owner audits
  (`docs/audits/2026-10-08-*.md` in BG Rebalance): the Modpack and SoD items and the
  forks audit. The BG Rebalance and Spell Revisions items are released (above).
- Small documentation fixes: the Remote Console README's EEex v1.3.1 lab note still says
  "this branch". Also open: whether Remote Console gets GitHub releases (it has tags
  only).

**Waiting for Christopher**

- Modpack alpha.14: the OK to publish the release candidate (above).
- Modpack `430`: whether the restart box may appear while the game starts, before the
  main menu, or must be shown on the visible main menu (a follow-up change).
- Artisan's Kitpack fork: whether to fix the Eldritch Knight / Arcane Trickster extra
  weapons in a next fork version, and whether the upstream report mentions them. Also
  how existing installations get the sorcerer weapon fix (the fork's tail patch, above).
- EEex v1.3.1: how existing installations move from v1.2.0.
- Entropy Shield / Dispelling Screen: Breach does not remove Entropy Shield (decided
  2026-10-17); the remaining questions are open.
- Quest mods (2026-10-23): which components start selected when a quest mod is ticked.
  Provisional: only the main component (NTotSC `0`, PPE core `0`); Ascalon's Questpack,
  which has none, starts with the decided first variant of each pair (`2`, `5`, `10`).
  Also open: whether ticking BG1 Unfinished Business preselects its 16 former defaults
  (now every component starts unchecked, like UB), and whether Black Hearts may stay
  selectable for players who untick the SoD Remix. Christopher (2026-10-25): go through
  the component choices together in detail; his principle for UB and BG1UB is that
  ticking everything installs everything, otherwise components are picked one by one.
  Black Hearts stays as it is for now.
- Installer link safety, remaining writers: staging publication, mod materialization and
  the WeiDU setup copy, then Radar and the `.chriz` state writers, still check a path and
  then write by path (inventory in PR #16). Moving them over waits for Christopher's go.
- The [curation queue](#curation-decisions-still-in-chriss-review-queue) at the end of this file.

**Deferred by decision**

- Aura fork work: parked (2026-10-17).

**After alpha.22**

- Christopher's own Elementalist Druid mod: the design continues right after alpha.22
  (Christopher, 2026-10-25). A design record from May/June 2026 exists; which sprites
  the forms use is still open.

**Ideas for later (community, recorded 2026-10-20)**

- SoD Coalition Camp defence: after the first troll group fell, the whole allied army
  turned hostile; a reload fixed it (alpha.13 stream-collection install). Home: SoD Remix
  (`chriz-sod-rebalance`) for triage, because Remix `230` cuts BD7100 trolls and `260`
  rewrites coalition-field areas. First reproduce in a lab with and without Remix; only
  if vanilla shows it too, fix it ourselves and file an upstream EEFixes issue with the
  repro.
- Optional "un-hired companions catch up" component: a companion recruited late joins
  at the current chapter cap (or the active party's level) instead of EET's fixed join
  XP. Home: Modpack campaign-XP family, next to `239` (join-XP catch-up) and `652`
  (chapter caps). It must stay off in Challenge Mode, whose point is that the party
  never outlevels content and banked XP goes only to active members (Christopher,
  2026-10-19).
- Companion randomizer: randomize NPC classes, either fully or within their class
  group. Home: proposed as a new standalone mod repository, because it conflicts with
  Modpack's fixed companion class changes (`190`-`198`), has to install after all NPC
  mods, and is useful outside CEBG. Needs a design with Christopher (kits, multi/dual
  classes, proficiencies, spells, SoD/SoA hand-over, saves).

**Ideas for later (Christopher, recorded 2026-10-22)**

- Challenge-run proof: a visible "Challenge Mode active" marker in the game menu that
  works with any UI mod, ideally also an indicator while playing, plus an optional
  certificate or end-of-run statistics that can be shown on streams. Ironman/Hardcore
  must not be forced. Brainstorm with Christopher before any work.
- German language support: later, low priority.

**Ideas for later (Christopher, 2026-10-23)**

- Challenge Mode rules: Time Stop at most once per caster per rest and never chained,
  also not through a Wild Mage's Reckless Dweomer; no rest loops through Wish. They add
  to the remaining Challenge Mode rules under Larger projects.

**Larger projects:** Time Stop/Haste (Time Clocks, above); script engine / Fight Director
(design with Christopher); configurable Spell Revisions (recover branch
`codex/configurable-sr-catalog`); dragon encounters (Thaxll, Nizid, Saladrex); the
remaining Challenge Mode rules (wands, traps, one Wish per combat, one memorized Time Stop
and others); optional or regular SoD; long-term, Unfinished Business's BG1 and BG2 cut
content, evaluated component by component (keep, fix or extend).

## Release records

- [ ] Runtime acceptance is still `pending` in both `acceptance.toml` records (`manifest/`
  and `recipes/curated-full-current/`). A full 2.7.3 collection install has since booted
  and been played with saves, but no acceptance record captures loader boot, BG1 start or
  save/reload, and the record's "no clean BG1 source" reason is outdated. The `430`
  record still says the alpha.11 pin awaits publication although alpha.13 is pinned.
- [ ] The release gate evaluates only the Recommended selection
  (`engine/src/release_validate.rs`). The optional post-EET runs `eeexremote-bg2` and
  `chriz-bg-modpack-campaign-xp-bg2` are not in `approved_tail_runs`, so a user who
  enables them gets runs without tail approval. Approve or gate them. The six optional
  quest-mod runs (2026-10-23) install before EET_end and need no tail approval, but, like
  `eeexremote-bg2`, they have no static-test evidence record.
- [ ] Four deferred catalog defaults (Ascension `40` / UB `19`, Warlock `0`, IWDification
  `120`, Aura) are recorded only as omission targets in `manifest/curation-map.toml`, not
  in `known-limitations.toml`.

## Release gates — source and immutable pins

- [ ] [EET](EET.md): at each manifest freeze, recheck official releases (v14.1 is still the
  newest). Upstream `master` is six commits ahead of the pinned `74e91d72` archive,
  including a Windows shortcut-path fix (#206) that may matter for EET `100`.
- [ ] [EE Fixpack](EEFIXPACK.md): at each freeze, recheck packaged releases (`beta2` is
  pinned and still the newest); never follow unreleased `master`.
- [ ] [Warlock](C0WARLOCK.md): pin exact commit
  `a821992228c87a4bef75be4bc2fc833f8db46819` (still upstream head), update the renamed
  `Artisans_Warlock/Artisans_Warlock.TP2` path, and stop describing it as a v3.0 target.
  Waits for the Contingency prompt answer below.
- [ ] Stale catalog wording: `BARDICWONDERS.md` cites `balance.4`/`balance.6`; `ABETTORHLAREBALANCE.md` still says
  "keep unavailable" although the finite Abettor HLA work ships inside Bardic Wonders.
- [ ] [Sarah](SARAHTOB.md) portraits: captured with hashes on Modpack branch
  `codex/sarah-options`, but their artist and licence are unknown, so they are not
  published and the recipe has no feature for them.

## Release gates — installer and static integration

- [ ] SoD Remix `256` × Tweaks Anthology `3500`: fixed in v0.6.15, pinned; needs the
  combined Recommended lab install with `3500` on. v0.6.13's `256` refused the
  fire-elemental donor `BDELFIRM`, because `3500` (a default since the October 14
  intake) gives every summon a `CDTFOW` cast-spell effect
  ([chriz-sod-rebalance#31](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/31)).
  v0.6.15's `256` accepts exactly that effect and leaves it out of its hostile clones.
  The October 14 and 18 labs installed with `3500` off; with the fixed library, the
  owner reinstalled `256` and the rest of the remix tail in the disposable October 18
  lab with `3500` on (chriz-sod-rebalance PR #32). The combined install now also
  includes the default Imoen choice `292`.
- [ ] Modpack `430` (scrolls at the reader's caster level) fails on 2.7.3.0 with EEex
  v1.3.1, in alpha.12 and alpha.13: on the first load, a new game or an NPC join it shows
  its EEex error box ("copied action construction overlaps an existing lifetime") and
  disables itself, because the engine destroys most `CAIAction` objects inline, without
  the destructor `430` hooks. It is probably also the cause of a player's container-open
  error report. Plan A for alpha.14 is done on an unreleased Modpack branch, and `430`
  stays a default (Christopher, 2026-10-25):
  - every fallback is quiet and written to the log only, and a second start turns the
    runtime off;
  - wording fixes and documented known limitations;
  - in a 2026-10-25 lab, normal play kept `430` on: save and reload, area changes, the
    reader left behind, the reader leaving and rejoining the party. It turns itself off
    for one session only when a save holds a running scroll effect whose reader is
    neither in the party nor in a loaded area when the effect's delayed part comes due.
    Scrolls then cast at their normal level and saves stay usable; this is documented as
    a known limitation.
  - Done on 2026-10-26:
    - the restart box for the one exception (see Modpack alpha.14 above);
    - the `[CBM scroll]` tag for its loader-log lines;
    - the typos.
    A lab smoke showed no box in normal play.
  - Still open: check the loader log in the Recommended lab run before the release,
    because no ongoing playthrough has run with `430` yet.
- [ ] Voicepacks (`CD_*` catalogs, release v1.1.0): the 18 default runs
  `voicepack-*-bg2` have only static recipe checks and the owner's per-pack fixture
  installs with the pinned WeiDU 249 (six BG:EE/SoD/BG2:EE/EET fixtures, English and
  German). Install them once together with the full Recommended selection in a lab
  before the next CEBG release, and check that the voices appear in character creation.
- [ ] Quest mods ([NTOTSC](NTOTSC.md), [AC_QUEST](AC_QUEST.md), [BLACKHEARTS](BLACKHEARTS.md),
  [THALAN](THALAN.md), [ZG_IWD_VOICES](ZG_IWD_VOICES.md), [PPE](PPE.md)) and the now
  optional [BG1UB](BG1UB.md): static recipe checks only, nothing was installed. They stay
  experimental until one combined lab install with the full Recommended selection and the
  quest mods ticked. Open points for that lab:
  - Icewind Dale Voice Pack on EET: probably only three of the seven "order acknowledged"
    lines of each set play, because Graion's soundset helper uses another EET row layout.
  - PPE: which portrait Edwina shows with Tweaks Anthology `1180` and PPE `25`; the
    Thalantyr dialogue line that PPE's core and Thalantyr both patch; and which portraits
    of other mods PPE's core replaces in a CEBG install (30 creatures in the reference
    install, Pai'Na's SoA party portrait among them).
  - Black Hearts can only be selected with the SoD Remix unticked; neither that
    combination nor its SoD chapters have been installed.
- [ ] [EEex](EEEX.md) options: decide whether to seed or document
  `[EEex] Uncap FPS Limit Enabled`. The installer sets no EEex options, and a played
  collection install runs with the limit off. (Clean stage, loader launch and the absence
  of stale `EEex_*.lua` are confirmed.)
- [ ] Red Wizard source follow-up: the pinned Artisan tag still removes fixed vanilla spell
  resrefs (`ArtisansKitpack/lib/redwizard.tpa`) instead of checking effective prohibited
  schools, and SR `60` custom-kit school cleanup is uninvestigated. EET `EDWIN7_` coverage
  is done (Modpack `236`). Not a reinstated blanket SR ban. Details:
  `../../handoffs/2026-09-10-red-wizard-sr-default.md`.
- [ ] Validate [Tweaks Anthology](CDTWEAKS.md) `2312` against the final SR/IWDification/SCS
  spell set, `2420/2430` against the final Artisan proficiency and item rules, and `3390`
  against final tracking-area scripts.
- [ ] Artisan: add hidden, predicate-driven activation for tweak `1110` (`1010`+EEex) and
  `1010000` (`C0PR#CL.SPL` present); the feature schema has no file-exists predicate yet.
  Keep `30001`, `300010` and `8102` unavailable until priest delivery is redesigned.
  Latent: NPC `10004` does not require main `10003` (unselectable today because SR blocks
  it).
- [ ] Fix legacy EEex dependency checks: Artisan (`v4.81a-dev.85928f9-chriz.2`), Bardic
  Wonders (`v2.9c-balance.8`) and Bubb's Spell Menu still test EEex component `0`, which
  is only Quick Menu Core since EEex v1.2 (unchanged in v1.3.1). Require main component `1` or use a
  version-independent capability check; the mandatory EEex `0`–`8` only masks this.
- [ ] Implement and fixture-test the [IWDification](IWDIFICATION.md) `120` tail patch that
  adds level-7 Evasion to Arcane Trickster `C0ATR.2DA`; it stays excluded until then.
- [ ] [BG2EE/EET Fixpack](BG2EE-EET-FIXPACK.md): migrate the Branwen kit transfer (`101`)
  to a maintained conditional component and redesign NPC stat transfer `200`. (`300` is
  replaced by Tweaks Anthology `3121`, Edwin `400` by Modpack `236`, and the historical
  package is not exposed.)
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md): the README (v0.8.5) covers `401`–`403`
  but not `400` or `404`–`409`; recheck whether final SCS supersedes `100` or `101`. Its
  status column still calls the in-engine checks of `130` and `310`, the combat check of
  `320` and a status-entry retest of `301` pending, although v0.8.4's release notes
  record lab runs of all four (`130`, `301` and `320` in game on 2026-10-20, `310` in two
  engine labs); update the column after checking what each run covered.
- [ ] [22-fix migration inventory](COLLECTION_TAIL_FIXES.md): still blocked are
  `BARD_SPELL_FIX` (old modpack `100`), `SAFANA_SNARE_FIX` (`150`), `SAFANA_LATE_SPELLS`
  and `PRIEST_DELIVERY_FIX`. Its Evandra line is stale: `224` is released and in the
  recipe. Do not replay the historical tail installers or their captured order.
- [ ] Resolve the eight ownerless CIMC/CDMC animation INIs and the documented
  silent-writer coverage limit in the tail-fix inventory before claiming that every
  manually copied install-folder file has been reproduced.

## Live/runtime acceptance

- [ ] [Sarah](SARAHTOB.md): XP catch-up is not applied (reported by Christopher on
  2026-10-16).
  - Cause (verified 2026-10-17): EET catches NPCs up through a `BD_JOINXP` block in
    each NPC's override script. EET adds it to vanilla NPCs, and some NPC mods ship
    their own; SarahTOB v8 has none. So her SoA version always joins at 89,000 XP
    (level 7).
  - Fix released in Modpack alpha.13 (2026-10-20): component `239` reads EET's own
    rules and adds its SoA ladder or ToB raise to every joinable NPC script that lacks
    one (Fade, Sarah, Yeslick, Xan and Evandra get the ladder; Wilson and Yoshimo get the
    raise). In lab copies of a SoA save (October 17, and October 20 on EEex v1.3.1 with
    the release package), a fresh Sarah joined at 750,000 XP. Mod NPCs that got the
    ladder are then no longer raised to 2,500,000 at the ToB transition, which matches
    EET's vanilla NPCs. Christopher decided on 2026-10-18 that the collection offers
    `239` as a default, because it is a fix; it is a default in both recipes since
    [PR #23](https://github.com/Chrizhermann/chriz-bg-collection/pull/23), in the late-companion run before campaign XP `650`. Open:
    the ToB raise was not exercised in a game, and the fix reaches players with the next
    collection release.
- [ ] Vanguard Defensive Stance: PraetorFallen reports that disabling "Use special
  abilities" in the AI settings stops companions from using the ability, but the main
  character still uses it automatically. Recorded on 2026-10-16. Cause: EET's BG1
  Defensive Stance block in the default party AI script `BDDEFAI` lacks that check
  ([Gibberlings3/EET#210](https://github.com/Gibberlings3/EET/issues/210)). The fix,
  first built on Artisan branch `claude/defensive-stance-ai-toggle`, shipped as Modpack
  `460` in alpha.13; in the October 20 lab there was no stance with the option off and
  the stance was used with it on. Christopher decided on 2026-10-25 that the collection
  offers `460` as a default, with the alpha.14 pin. Open: whether the Artisan branch is
  still needed.
- [ ] [EEex](EEEX.md): the modules ran in the 2026-10-18 v1.3.1 lab (see Done), but that
  lab is hidden, so nothing was seen on screen. Check `B3Scale` at a resolution wider
  than 4:3 and the empty-container highlight in a visible game.
- [ ] Creature `LOCALS` survive save/load ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19)). In the 2026-10-18
  EEex v1.3.1 lab they did not, in the same process and after a restart; `GLOBAL`
  variables did. The cause was Modpack `650`'s XP effect, not EEex; the fix is done for
  alpha.14 (see In progress), and the check passed with it in a lab on 2026-10-23.
  Open: repeat it with the released alpha.14 package and `650` on.
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md) Tempus APR: under the pinned EEex (v1.3.1), verify
  steady 1.5 APR with a two-pip weapon, 2.5 under Holy Power, no cycling, and immediate
  loss of the bonus with a zero-pip weapon.
- [ ] [CHRIZ-SOD-REMIX](CHRIZ-SOD-REMIX.md) `225`: obtain all five items naturally and
  verify a live one-item-short refusal that consumes nothing and grants no XP.
  Activation, dormant re-click and save/reload were accepted on 2026-09-05.
- [ ] [Randomiser](RANDOMISER.md) v8.1.1 `1100` on a clean rebuilt collection and new
  game: Tarnesh loot smoke and the legacy-BCS fallback for EEex v1.x (the fork rejects the root `EEex_scripts` layout of
  v1.2 and v1.3). (Ordering, `510`
  numeric input, the `570` conflict and the `10300` omission with SCS `8040` are covered
  statically.)
- [ ] Run targeted live checks for the migrated tail fixes, especially modpack `400`,
  `410`, `430`, `440` and `450`; so far only file-level checks exist.
- [ ] [BuffBot](BUFFBOT.md): the BG1 start/save/reload smoke passed with v1.10.0 (see
  Done); the pinned v1.11.0 has not run in a collection lab. Open: toggle
  stances with opcode 214 variants count as buffs, so a new party's Long preset enables
  Artisan's Power Attack ([#78](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/78)); two
  in-game test suites have stale or install-specific expectations
  ([#79](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/79)). The combat-stop notice
  fix (since v1.10.1, [bg-eeex-buffbot#76](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/76))
  was checked in a lab through the engine's text object only; nobody has seen the notice
  on screen in a real combat stop yet. v1.11.0's own EET/SR/SCS/Modpack labs ran with
  Modpack `430` uninstalled, and its notes do not accept the configuration with `430`
  yet, which the recommended preset installs: check BuffBot with `430` in the
  Recommended lab run. Also check there its new column headers and control artwork, and
  that no other mod in the stack writes its PVRZ pages `MOS88809`–`MOS88993`.
- [ ] Game-log switch (see Done): no game has started with it yet. In a lab, turn it on
  in CEBG, start and quit the game twice with **Play**, and check that
  `InfinityLoader.log` and `InfinityLoader-previous.log` hold the second and first
  session and that the loader shows no log warning.
- [ ] Visually verify the Evandra portrait override in game.

## Conditional/future feature gates

These are nonblocking while their parent feature remains unavailable.

- [ ] Implement before exposing: a Safana-only semantic replacement for the old coupled
  modpack `100`; `150`; `SAFANA_LATE_SPELLS`; and `PRIEST_DELIVERY_FIX`. Follow each
  activation rule in [the tail-fix inventory](COLLECTION_TAIL_FIXES.md).
- [ ] [Safana](SAFANA.md): rewrite the Bard spell/snare/class preset against effective
  Spell Revisions identities. (Her SoD-item cleanup is done: Modpack `189`.)
- [ ] Prove fresh-install retirement of modpack `200`, `210` and `300`, and verify all six
  legacy Aura item changes before retiring that snapshot.
- [ ] Keep Randomiser's expanded Mode-1 pool unavailable until separately curated and
  accepted. The same applies to the Campaign Item Randomiser (CIR), released since
  2026-10-19, until its CEBG integration and a lab test (see In progress).
- [ ] [Warlock](C0WARLOCK.md): script the chosen answer to the hidden Contingency UI
  spell-learning prompt, then test its pit-fiend spell loadout under SR, copied demon AI
  under SCS, Bubb menu integration and EEex v1.3.1 behavior. Warlock is not in the recipe.
- [ ] [Aura](AURA_BG1_2_EET.md), parked on 2026-10-17: the unreleased balance fork lacks
  8 upstream commits and has 12 fork-only commits; compare its spell changes with the
  historical [Aura Balance Patch - Spells](AURA_BALANCE_PATCH_SPELLS.md) v1.0 before
  publishing and pinning. Aura is excluded from the recipe, so this blocks nothing.

## Curation decisions still in Chris's review queue

These are genuine choices that remain after the technical questions in the completed
catalogs were normalized. None has a recorded decision as of 2026-10-17.

- [ ] Curate newly discovered Artisan main components `1010`, `2003`, `5300`, `5200`,
  `7000`, `7007`, and `7008`, plus tweak components `20021`, `1020000`, and `2000001`.
- [ ] Curate newly discovered Bardic Wonders components `2011`, `2012`, and `2013`.
- [ ] [BGGO](BGGO.md) `4`: decide whether Alternate Graphics is excluded, optional, or default.
- [ ] [Warlock](C0WARLOCK.md): choose Yes or No for the Contingency UI-based
  spell-learning prompt. The component remains a default either way.
- [ ] Decide whether to include the separate BG1 Sirene v3.1 package in addition to the
  already curated BG2/EET Sirene route.
- [ ] Choose one restored-Bhaalspawn-powers provider: Ascension `40` (technical
  recommendation) or UB `19`; never expose both together. Since 2026-10-17 every
  Unfinished Business component, including `19`, is optional and unselected.
- [ ] Confirm the shipped Modpack defaults: Dynaheir `197` gets Haste as a known spell
  only (no memorized copy), and Kivan `198` is a kit-only Archer conversion with no
  proficiency or loadout rewrite. Sarah stays an Archer (`190`), and Shar-Teel is settled
  as a default Wizard Slayer.
- [ ] Quest mods (2026-10-23): choose the component defaults inside
  [NTotSC](NTOTSC.md), [Ascalon's Questpack](AC_QUEST.md) and [PPE](PPE.md) (provisional:
  the main component, and for Ascalon's Questpack only its decided first variants),
  whether [BG1UB](BG1UB.md)'s components start selected when it is ticked, and whether
  [Black Hearts](BLACKHEARTS.md) stays selectable without the SoD Remix. 2026-10-25: a
  detailed walkthrough with Christopher is pending (his principle and Black Hearts: see
  Waiting for Christopher).

## Done since 2026-09-04

Done on 2026-10-24 ([PR #17](https://github.com/Chrizhermann/chriz-bg-collection/pull/17),
approved for the next CEBG release on 2026-10-23):

- Installer link safety: patch restore with its adapter payload, BuffBot update writes
  and their restore, and uninstall now read, write and delete through held folder
  handles; uninstall also refuses a root that no longer has the confirmed identity.
  Evidence in the pull request: junction and reparse-point tests at every level, and a
  mutation check whose 20 mutations the new tests all caught. The ancestor checks above
  an installation stay as they are: the planned OneDrive classification is dropped
  (Christopher, 2026-10-19), because Windows shows OneDrive's placeholders to the
  installer as plain folders and files.

Done on 2026-10-23 ([PR #30](https://github.com/Chrizhermann/chriz-bg-collection/pull/30),
merged on 2026-10-24):

- Installer: the hidden game-log switch for crash diagnosis (requested on 2026-10-17,
  confirmed for the next release on 2026-10-23).
  **Troubleshooting** at the end of an installation on the home screen stays closed
  and reads nothing until opened; **Write a game log for crash reports** sets
  InfinityLoader's `LogFile` in that installation's `InfinityLoader.ini` to
  `InfinityLoader.log` or back to blank, changing no other byte, through held folder
  handles and only while no program from the game folder runs. Before each **Play**,
  CEBG keeps the last session's log as `InfinityLoader-previous.log` (its last 64 MiB at
  most); a failure is reported and never blocks the game. The section shows where the
  log, that copy and the crash reports (`InfinityLoader_Crash`) are and which files to
  send. Turning the log on or off keeps a BuffBot update's Undo available. Evidence:
  engine tests in `engine/src/game_log.rs` and `engine/src/mod_upgrades/tests.rs`,
  command tests in `app/src-tauri/tests/command_contract.rs` and UI tests in
  `app/tests/game-log.test.ts`; user help in [troubleshooting](../../troubleshooting.md).
  A Windows test compares the settings parser with `GetPrivateProfileStringW` itself on
  36 edge cases (line endings, section names, byte order mark, quotes, duplicates). The
  opt-in engine test `real_loader_settings_change_only_in_their_log_file_line` switched
  copies of two real `InfinityLoader.ini` files, as EEex v1.3.1 ships it (log off) and
  from a played game (log on), and back: only their `LogFile` line changed, and
  switching back gave the same bytes. The lab check with a real game start is under
  Live/runtime acceptance.

Done on 2026-10-23 ([PR #26](https://github.com/Chrizhermann/chriz-bg-collection/pull/26)):

- Installer downloads: archives (for the collection cache and `chriz-bg-author artifact
  inspect`/`verify`) no longer have a total time limit. An attempt fails only when no
  data arrives for 60 seconds; connecting stays bounded. Retry, resume and hash checks
  are unchanged. Evidence: loopback engine tests with injected limits, mutation checks,
  an updater test and live `artifact verify` runs from a fresh cache
  (`docs/download-stall-timeout-2026-10-22.md`). Not fixed: the app's own setup download
  still has no stall detection.

Done on 2026-10-20 ([PR #18](https://github.com/Chrizhermann/chriz-bg-collection/pull/18)):

- [EEex](EEEX.md) v1.3.1 (2026-10-07) replaces v1.2.0 in both recipes: fresh download,
  published release, archive shape and a native-order source audit in
  `manifest/evidence/release-intake-20261018.toml`. The lab smoke test of every EEex
  user is recorded in the pull request (2026-10-18).

Done on 2026-10-18 in the EEex v1.3.1 runtime lab (EET over BG2:EE 2.7.3.0, full recipe,
new BG1 game with a throwaway character, hidden quiet launches;
[PR #18](https://github.com/Chrizhermann/chriz-bg-collection/pull/18)). Checked through Lua state, not on screen:

- [EEex](EEEX.md) modules: the effect menu opened while Left Shift was held and listed
  Bless after a cast; the empty-container hook was installed and gave the empty
  container in Candlekeep its own colour; a temporary two-key hotkey fired for j then
  k and not for k then j; `B3Scale` left 1024x768 unchanged and computed 1365x768,
  1820x768 and 1228x768 for 1920x1080, 2560x1080 and 1280x800; time step (`d`) advanced one tick
  from pause, ran while held and paused again on release; the timer's cast and modal
  bars ran during a cast and a modal action.
- [EEex Remote Console](EEEXREMOTE.md): README checklist steps 1-6 on the target. On the
  world screen `ping.lua` answered `pong`, the 22-case smoke with the watchdog passed and
  the next `pong` came from the same game process, both with the pinned v0.2.0 loader
  and with the fix from [eeex-remote-console#4](https://github.com/Chrizhermann/eeex-remote-console/pull/4).
- [BuffBot](BUFFBOT.md): BG1 start and save; after loading, the saved preset name and
  active preset were back and an unsaved change was gone (UDAux). `BfBot.Test.RunAll()`
  passed 22 of 25 phases; the three failures are tracked in
  [bg-eeex-buffbot#78](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/78) and
  [#79](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/79).

Done on 2026-10-18 ([PR #14](https://github.com/Chrizhermann/chriz-bg-collection/pull/14)):

- Both `known-limitations.toml` files name Artisan `8101` and NPC `10004` correctly,
  checked against the Artisan tag `v4.81a-dev.85928f9-chriz.1`.
- The pin tables in [the 2.7 pin list](../../pin-list-2.7.md) are generated from both
  recipes and checked by `engine/tests/pin_list.rs`; the 2026-08/09 planning list stays
  there as history. `manifest/mod-sources.tsv` is captured research data, not a pin
  list, and the catalog notes that called it one now point at the recipe artifacts.
- [Crossmod](CROSSMODBG2.md) Sirene audit: detection fails upstream, but Crossmod's only
  Sirene content needs the Will NPC, which the collection does not include. Order
  before CDTweaks and `1 -> 0` are enforced; component `2` is moot without a
  multi-romance route.
- [Tweaks Anthology](CDTWEAKS.md) `3347` ships its reference `cdtweaks.txt` (`125`
  percent movement, casting-speed increase off, portrait icon on). Recipe components
  can now declare configuration files, which the installer places in the mod's
  published folder before the run and records as `config-files.json` evidence
  ([PR #15](https://github.com/Chrizhermann/chriz-bg-collection/pull/15)).

Verified on 2026-10-17 against `claude/cebg-installer-fixes-20261014`:

- `public-alpha` gates require usable artifacts, reject FAIL placeholders and require
  static evidence (`engine/src/release_validate.rs`, 62cad05); post-EET runs and unresolved
  defaults carry Christopher's approvals in `known-limitations.toml`.
- One official EET `master` archive (`74e91d72`, pinned 2026-09-03) serves EET, EET_END
  and EET_gui, replacing the stale v14.0 source. EE Fixpack is pinned to packaged `beta2`.
- Pins: EEex v1.2.0 with its nine-component preset; Hidden Gameplay Options v5.2;
  Randomiser v8.1.1 (`f4a9dfb1`); Bubb's Spell Menu v5.2 (53d6f77); Artisan's Kitpack
  `4.81a-dev.85928f9-chriz.1` for all three installer identities (ef62225; curated `5110`
  available); Bardic Wonders `v2.9c-balance.8` (c2da5e0) with the finite Abettor HLA work
  inside since `balance.3`; Spell Revisions `v4.21-chriz.5` with `srcb-rr-compat` from the
  same release, component `60` owning `SR_SUBSPELL_FIX`; SoD Remix v0.6.13 with 40
  mandatory components including the now-implemented `290`; BG Rebalance v0.8.3 (its `401`
  correction for Artisan-packed `AP_C0PR#CL` cells shipped in v0.3.1).
- Evandra: approved upfront manual download or Skip (`manual-user-supplied` artifact);
  her portrait ships in Modpack `225`, redistribution approved on 2026-09-29.
- EE Fixpack `0+2` runs twice from one artifact (after DLC Merger and before EET `0`);
  EET and EET_END share one artifact, enforced by validation and tests.
- EEex Remote Console `0` is encoded as optional after EEex and requires EEex `0`–`8`.
- Spell Revisions `60` installs after every NPC/class assignment; SCS `4240` is enforced.
  The Artisan `8101` and NPC `10004` conflict edges remain; the unsupported Edwin `5102`
  edge was removed on 2026-09-10 and Edwin is default-checked with SR again.
- Hidden Gameplay Options late ordering and special handling are covered statically;
  `40` stays blocked because it needs Tweaks Anthology `3400`, which is not in the
  collection.
- Bardic Wonders `1008` answers No to its Garrick prompt and Artisan NPC `99001` requires
  it; Bardic `balance.8` has SR fixture coverage for `1008`, `1009` and `2008`.
- SCS 35.21 with WeiDU 249 installed and has been played on a 2.7.3.0 collection install;
  upstream-author support remains unconfirmed.
- EET Tweaks `2042` is superseded: EET Tweaks is not in the recipe, and Modpack `610`/`611`
  provide progressive and story utility XP.
- Modpack `440`/`450` have fail-closed real-WeiDU tests and clean installs; `430` is now
  the scroll-casting policy (470 scrolls analysed in the 2026-10-13 combined test); the
  old `600` was removed on 2026-09-04.
- The merged 2.7 EET result booted after both EE Fixpack runs and has been played.
- Modpack `110`, `130`, `140`, `160`, `400` and `410`, the NPC assignments `190`–`198`,
  `220`–`223` and `230`–`234`, and Evandra's sorcerer `224` are implemented. Xan is covered
  by the maintained Modpack `170` plus Artisan NPC `20002`, including EET variants `XAN4`,
  `XAN6` and `TTXAN`; legacy `120`, `180` and `500` are retired.
- Fade's default Fighter/Thief choice is exposed; it requires Fade and conflicts with
  Fade `2`.
- BuffBot keeps `1` before `0`; Modpack `430` deliberately runs after BuffBot, so BuffBot
  is no longer the absolute final layer.

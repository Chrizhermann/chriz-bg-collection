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

## Work queue at a glance — 2026-10-18

Cross-repository status. Component detail stays in the sections below and in the catalogs.

**In progress (separate sessions)**

- EEex v1.3.1 everywhere (Christopher, 2026-10-18: test as soon as possible and use it
  everywhere): pinned in both recipes on `claude/eeex-131-pin` with fresh-download and
  native-order evidence. On 2026-10-18 a disposable 2.7.3.0 lab with the full recipe
  booted every EEex user cleanly; a new BG1 game in that lab then exercised the EEex
  modules, Remote Console and BuffBot (see Done). Remote Console: [eeex-remote-console#3](https://github.com/Chrizhermann/eeex-remote-console/pull/3)
  and the `START_SP` fix [eeex-remote-console#4](https://github.com/Chrizhermann/eeex-remote-console/pull/4). Open from that lab: creature
  `LOCALS` are missing after save/load ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19), cause open) and Modpack `650`
  (below). Attack Info `9` waits for Christopher.
- Imoen's handover from BG1/SoD into SoA (SoD Remix): three choices with the vanilla
  Thief→Mage dual as the default, working for every thief kit.
- Modpack: lab-verified, unreleased fixes for campaign XP caps `650`/`651` and a new
  join-XP catch-up `239` for mod NPCs without EET's `BD_JOINXP` block (see Sarah below)
  ship together with chapter-bound XP caps (`652`, values: Christopher), which come next
  (decided 2026-10-17). The collection then offers `239` as a default (decided
  2026-10-18). The next collection release pins only a lab-verified `650`. In the
  2026-10-18 EEex v1.3.1 lab, released alpha.12 still gave no actor an identity (the
  EEex v1.2 failure); the unreleased fix is lab-verified on v1.2.0 only and needs the
  same check on v1.3.1
  ([modpack archive #10](https://github.com/Chrizhermann/chriz-bg-modpack-private-archive/issues/10)).
- Hobgoblin challenge run (design with Christopher).
- Campaign Item Randomiser (CIR): next version.
- Vanguard Defensive Stance AI toggle (see below).
- InfinityLoader crash analysis: causes found; follow-up decisions are Christopher's.
- Installer link safety: patch restore, mod upgrades and uninstall now read and write
  through held folder handles too (PR #17, in review). Next: ancestor checks above an
  installation accept OneDrive (cloud-files) folders but keep refusing junctions, symlinks
  and unknown reparse points (decided 2026-10-18).

**Ready to start**

- Creature `LOCALS` after save/load ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19)): in the EEex v1.3.1 lab,
  locals set before saving were missing after loading, in the same process and after a
  restart; globals survived. Next: compare the vanilla 2.7.3.0 effect-list loader with
  EEex's replacement and repeat the save/reload without EEex, on 2.6.6.0 and with a
  smaller stack. The result decides whether this blocks the v1.3.1 rollout.
- Time Stop/Haste timing fix as a standalone EEex mod. Engine research and an observation
  lab exist (BG Rebalance branch `wip/2026-10-17/main-checkout`,
  `research/10-timestop-haste-engine-clocks.md` and `research/timestop-lab/`); nothing
  has run in the game yet. The lab scripts pin EEex v1.3.0 (commit, version, tree hash,
  WeiDU.log pattern) and refuse v1.3.1 until they are re-pinned.
- Compatibility follow-ups from the October 8 owner audits (`docs/audits/2026-10-08-*.md`
  in BG Rebalance): Rebalance and the Spell Revisions fork first, Modpack and SoD items
  after the sessions above land.
- Installer: a hidden setting that turns on the game log for crash diagnosis
  (InfinityLoader's `LogFile`), requested on 2026-10-18. The loader overwrites the log on
  every start, so the feature has to keep the previous session's log; Windows crash dumps
  are the other half of the evidence.

**Waiting for Christopher**

- SoD Remix `256` × Tweaks Anthology `3500`: Recommended does not install until `256`
  handles the summon effect; which fix
  ([chriz-sod-rebalance#31](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/31)).
- EEex v1.3.1: whether to offer Attack Info `9`, and how existing installations
  (including the stream game) move from v1.2.0.
- Small spell durations (Luck, Bless, Aid) and druid Chaotic Commands: in discussion.
- Entropy Shield / Dispelling Screen: Breach does not remove Entropy Shield (decided
  2026-10-17); the remaining questions are open.
- Chapter-cap values for `652`: Christopher posted the full table on 2026-10-18 (Ch. 1-2
  16,000 through ToB Ch. 22 8,900,000); Modpack branch `feat/chapter-xp-caps-652` uses
  the same values. What remains is the `652` release itself.
- Installer link safety, remaining writers: staging publication, mod materialization and
  the WeiDU setup copy, then Radar and the `.chriz` state writers, still check a path and
  then write by path (inventory in PR #16). Moving them over waits for Christopher's go.
- The [curation queue](#curation-decisions-still-in-chriss-review-queue) at the end of this file.

**Deferred by decision**

- Next collection release (lab run with Bardic `balance.8`, optional Unfinished Business
  and the fixed Modpack): later.
- Aura fork work: parked (2026-10-17).

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

**Larger projects:** Time Stop/Haste (above); script engine / Fight Director (design with
Christopher); configurable Spell Revisions (recover branch `codex/configurable-sr-catalog`);
dragon encounters (Thaxll, Nizid, Saladrex); the remaining Challenge Mode rules (wands,
traps, one Wish per combat, one memorized Time Stop and others); optional or regular SoD.

## Release records

- [ ] Runtime acceptance is still `pending` in both `acceptance.toml` records (`manifest/`
  and `recipes/curated-full-current/`). A full 2.7.3 collection install has since booted
  and been played with saves, but no acceptance record captures loader boot, BG1 start or
  save/reload, and the record's "no clean BG1 source" reason is outdated. The `430`
  record still says the alpha.11 pin awaits publication although alpha.12 is pinned.
- [ ] The release gate evaluates only the Recommended selection
  (`engine/src/release_validate.rs`). The optional post-EET runs `eeexremote-bg2` and
  `chriz-bg-modpack-campaign-xp-bg2` are not in `approved_tail_runs`, so a user who
  enables them gets runs without tail approval. Approve or gate them.
- [ ] Four deferred catalog defaults (Ascension `40` / UB `19`, Warlock `0`, IWDification
  `120`, Aura) are recorded only as omission targets in `manifest/curation-map.toml`, not
  in `known-limitations.toml`.

## Release gates — source and immutable pins

- [ ] [EEex](EEEX.md): v1.3.1 (2026-10-07) replaces v1.2.0 on `claude/eeex-131-pin`:
  fresh download, published release, archive shape and a native-order source audit in
  `manifest/evidence/release-intake-20261018.toml`. The lab smoke test of every EEex
  user is recorded in the pin pull request (2026-10-18); check this once the branch is
  merged.
- [ ] [EET](EET.md): at each manifest freeze, recheck official releases (v14.1 is still the
  newest). Upstream `master` is six commits ahead of the pinned `74e91d72` archive,
  including a Windows shortcut-path fix (#206) that may matter for EET `100`.
- [ ] [EE Fixpack](EEFIXPACK.md): at each freeze, recheck packaged releases (`beta2` is
  pinned and still the newest); never follow unreleased `master`.
- [ ] [Warlock](C0WARLOCK.md): pin exact commit
  `a821992228c87a4bef75be4bc2fc833f8db46819` (still upstream head), update the renamed
  `Artisans_Warlock/Artisans_Warlock.TP2` path, and stop describing it as a v3.0 target.
  Waits for the Contingency prompt answer below.
- [ ] Stale catalog wording: `CHRIZ-SOD-REMIX.md` still names v0.6.11 as the pin;
  `BARDICWONDERS.md` cites `balance.4`/`balance.6`; `ABETTORHLAREBALANCE.md` still says
  "keep unavailable" although the finite Abettor HLA work ships inside Bardic Wonders.
- [ ] [Sarah](SARAHTOB.md) portraits: captured with hashes on Modpack branch
  `codex/sarah-options`, but their artist and licence are unknown, so they are not
  published and the recipe has no feature for them.

## Release gates — installer and static integration

- [ ] The Recommended selection does not install: SoD Remix v0.6.13 `256` refuses the
  fire-elemental donor `BDELFIRM`, because Tweaks Anthology `3500` (a default since the
  October 14 intake) gives every summon a `CDTFOW` cast-spell effect
  ([chriz-sod-rebalance#31](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/31)).
  The October 14 and 18 labs install with `3500` off. Fix `256`, then install the whole
  Recommended selection once with `3500` on.
- [ ] [EEex](EEEX.md) options: decide whether to seed or document
  `[EEex] Uncap FPS Limit Enabled`. The installer sets no EEex options, and a played
  collection install runs with the limit off. (Clean stage, loader launch and the absence
  of stale `EEex_*.lua` are confirmed.)
- [ ] [EEex Remote Console](EEEXREMOTE.md): its repository documentation at the pinned
  commit still describes EEex 1.0.x. The update is [eeex-remote-console#3](https://github.com/Chrizhermann/eeex-remote-console/pull/3):
  v1.2 and v1.3 guidance, v1.3.1 lab evidence and a PowerShell 7 fix for its smoke
  suite. Re-pin after it merges.
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
- [ ] Fix legacy EEex dependency checks: Artisan (`v4.81a-dev.85928f9-chriz.1`), Bardic
  Wonders (`v2.9c-balance.8`) and Bubb's Spell Menu still test EEex component `0`, which
  is only Quick Menu Core since EEex v1.2 (unchanged in v1.3.1). Require main component `1` or use a
  version-independent capability check; the mandatory EEex `0`–`8` only masks this.
- [ ] Implement and fixture-test the [IWDification](IWDIFICATION.md) `120` tail patch that
  adds level-7 Evasion to Arcane Trickster `C0ATR.2DA`; it stays excluded until then.
- [ ] [BG2EE/EET Fixpack](BG2EE-EET-FIXPACK.md): migrate the Branwen kit transfer (`101`)
  to a maintained conditional component and redesign NPC stat transfer `200`. (`300` is
  replaced by Tweaks Anthology `3121`, Edwin `400` by Modpack `236`, and the historical
  package is not exposed.)
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md): the v0.8.3 README covers `401`–`403`
  but not `400` or `404`–`409`; recheck whether final SCS supersedes `100` or `101`.
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
  - Fix lab-verified, not yet released: optional Modpack component `239` reads EET's
    own rules and adds its SoA ladder or ToB raise to every joinable NPC script that
    lacks one (Fade, Sarah, Yeslick, Xan and Evandra get the ladder; Wilson and Yoshimo
    get the raise). In a lab copy of a SoA save, a fresh Sarah joined at 750,000 XP.
    Mod NPCs that got the ladder are then no longer raised to 2,500,000 at the ToB
    transition, which matches EET's vanilla NPCs. Christopher decided on 2026-10-18
    that the collection offers `239` as a default, because it is a fix.
- [ ] Vanguard Defensive Stance: PraetorFallen reports that disabling "Use special
  abilities" in the AI settings stops companions from using the ability, but the main
  character still uses it automatically. Recorded on 2026-10-16; a fix is in progress on
  Artisan branch `claude/defensive-stance-ai-toggle`.
- [ ] [EEex](EEEX.md): the modules ran in the 2026-10-18 v1.3.1 lab (see Done), but that
  lab is hidden, so nothing was seen on screen. Check `B3Scale` at a resolution wider
  than 4:3 and the empty-container highlight in a visible game.
- [ ] [EEex Remote Console](EEEXREMOTE.md): with the pinned v0.2.0, once `START` reopens
  from character creation or the load screen, New Game (seen in the 2026-10-18 lab) or
  Load Game leaves `START_SP` on the menu stack under the game's screens. The fix [eeex-remote-console#4](https://github.com/Chrizhermann/eeex-remote-console/pull/4) is lab-verified and
  waits for review; the pin moves after a release (Christopher's OK).
- [ ] Creature `LOCALS` survive save/load ([#19](https://github.com/Chrizhermann/chriz-bg-collection/issues/19)). In the 2026-10-18
  EEex v1.3.1 lab they did not, in the same process and after a restart; `GLOBAL`
  variables did.
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
- [ ] [BuffBot](BUFFBOT.md): the BG1 start/save/reload smoke passed (see Done). Open: the
  combat-stop notice never shows
  ([bg-eeex-buffbot#76](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/76), fix in
  [#77](https://github.com/Chrizhermann/bg-eeex-buffbot/pull/77)); toggle stances with
  opcode 214 variants count as buffs, so a new party's Long preset enables Artisan's
  Power Attack ([#78](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/78)); two
  in-game test suites have stale or install-specific expectations
  ([#79](https://github.com/Chrizhermann/bg-eeex-buffbot/issues/79)).
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
  accepted. The same applies to the Campaign Item Randomiser (CIR) until its release.
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
- [ ] [EEex](EEEX.md) `9` Attack Info (new in v1.3.0; to-hit chances on the attack cursor):
  decide whether it stays out or becomes optional, default or mandatory. Players can switch
  the module on in EEex's in-game options either way.
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

## Done since 2026-09-04

Done on 2026-10-18 in the EEex v1.3.1 runtime lab (EET over BG2:EE 2.7.3.0, full recipe,
new BG1 game with a throwaway character, hidden quiet launches; branch
`claude/eeex-131-pin`). Checked through Lua state, not on screen:

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

Done on 2026-10-18 on `claude/recipe-fixes-20261018`:

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
  (branch `claude/mod-config-files-20261018`).

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

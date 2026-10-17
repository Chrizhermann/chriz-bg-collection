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

- Imoen's handover from BG1/SoD into SoA (SoD Remix): three choices with the vanilla
  Thief→Mage dual as the default, working for every thief kit.
- Modpack: lab-verified, unreleased fixes for campaign XP caps `650`/`651` and a new
  join-XP catch-up `239` for mod NPCs without EET's `BD_JOINXP` block (see Sarah below)
  ship together with chapter-bound XP caps (`652`, values: Christopher), which come next
  (decided 2026-10-17). The collection then offers `239` as a default (decided
  2026-10-18). The next collection release pins only a lab-verified `650`.
- Hobgoblin challenge run (design with Christopher).
- Campaign Item Randomiser (CIR): next version.
- Vanguard Defensive Stance AI toggle (see below).
- InfinityLoader crash analysis: causes found; follow-up decisions are Christopher's.

**Ready to start**

- Time Stop/Haste timing fix as a standalone EEex mod. Engine research and an observation
  lab exist (BG Rebalance branch `wip/2026-10-17/main-checkout`,
  `research/10-timestop-haste-engine-clocks.md` and `research/timestop-lab/`); nothing
  has run in the game yet.
- Compatibility follow-ups from the October 8 owner audits (`docs/audits/2026-10-08-*.md`
  in BG Rebalance): Rebalance and the Spell Revisions fork first, Modpack and SoD items
  after the sessions above land.
- Installer: a hidden setting that turns on the game log for crash diagnosis
  (InfinityLoader's `LogFile`), requested on 2026-10-18. The loader overwrites the log on
  every start, so the feature has to keep the previous session's log; Windows crash dumps
  are the other half of the evidence.

**Waiting for Christopher**

- Small spell durations (Luck, Bless, Aid) and druid Chaotic Commands: in discussion.
- Entropy Shield / Dispelling Screen: Breach does not remove Entropy Shield (decided
  2026-10-17); the remaining questions are open.
- Chapter-cap values for the new XP-cap component.
- The [curation queue](#curation-decisions-still-in-chriss-review-queue) at the end of this file.

**Deferred by decision**

- Next collection release (lab run with Bardic `balance.8`, optional Unfinished Business
  and the fixed Modpack): later.
- Aura fork work: parked (2026-10-17).

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
- [ ] Correct the user-facing omission texts in both `known-limitations.toml` files:
  Artisan `8101` is "Pale Master Spells: Unholy Necromancy Spells Only" (not "Revised
  Stormlord") and Artisan NPC `10004` is "Make Rasaad a Sacred Fist" (not "Stormlord
  Yeslick").
- [ ] The release gate evaluates only the Recommended selection
  (`engine/src/release_validate.rs`). The optional post-EET runs `eeexremote-bg2` and
  `chriz-bg-modpack-campaign-xp-bg2` are not in `approved_tail_runs`, so a user who
  enables them gets runs without tail approval. Approve or gate them.
- [ ] Four deferred catalog defaults (Ascension `40` / UB `19`, Warlock `0`, IWDification
  `120`, Aura) are recorded only as omission targets in `manifest/curation-map.toml`, not
  in `known-limitations.toml`.

## Release gates — source and immutable pins

- [ ] [EEex](EEEX.md): v1.2.0 is pinned; upstream v1.3.0 (2026-09-16) and v1.3.1
  (2026-10-07) have no intake review on the branch yet.
- [ ] [EET](EET.md): at each manifest freeze, recheck official releases (v14.1 is still the
  newest). Upstream `master` is six commits ahead of the pinned `74e91d72` archive,
  including a Windows shortcut-path fix (#206) that may matter for EET `100`.
- [ ] [EE Fixpack](EEFIXPACK.md): at each freeze, recheck packaged releases (`beta2` is
  pinned and still the newest); never follow unreleased `master`.
- [ ] [Warlock](C0WARLOCK.md): pin exact commit
  `a821992228c87a4bef75be4bc2fc833f8db46819` (still upstream head), update the renamed
  `Artisans_Warlock/Artisans_Warlock.TP2` path, and stop describing it as a v3.0 target,
  also in `mod-sources.tsv`. Waits for the Contingency prompt answer below.
- [ ] [The 2.7 pin list](../../pin-list-2.7.md) has not changed since 2026-09-04 and
  contradicts the recipe for Artisan, Bardic Wonders, Spell Revisions, Evandra and the
  Tweaks Anthology; `mod-sources.tsv` is stale for BuffBot, the Modpack, SoD Remix, Spell
  Revisions and Bardic Wonders. Regenerate both from
  `recipes/curated-full-current/artifacts/`, or mark them historical with the artifacts as
  the source of truth.
- [ ] Stale catalog wording: `CHRIZ-SOD-REMIX.md` still names v0.6.11 as the pin;
  `BARDICWONDERS.md` cites `balance.4`/`balance.6`; `ABETTORHLAREBALANCE.md` still says
  "keep unavailable" although the finite Abettor HLA work ships inside Bardic Wonders.
- [ ] [Sarah](SARAHTOB.md) portraits: captured with hashes on Modpack branch
  `codex/sarah-options`, but their artist and licence are unknown, so they are not
  published and the recipe has no feature for them.

## Release gates — installer and static integration

- [ ] [EEex](EEEX.md) options: decide whether to seed or document
  `[EEex] Uncap FPS Limit Enabled`. The installer sets no EEex options, and a played
  collection install runs with the limit off. (Clean stage, loader launch and the absence
  of stale `EEex_*.lua` are confirmed.)
- [ ] [EEex Remote Console](EEEXREMOTE.md): its repository documentation at the pinned
  commit still describes EEex 1.0.x; the update sits on its unmerged branch
  `codex/eeex-1.2-compat`.
- [ ] [Crossmod](CROSSMODBG2.md): fix Sirene detection. Crossmod looks for `bc0siren.dlg`,
  but the installed Sirene ships `bc0sire2.dlg`, so its Sirene content is probably
  skipped. (Order before CDTweaks and `1 -> 0` are enforced; component `2` is moot
  without a multi-romance route.)
- [ ] Red Wizard source follow-up: the pinned Artisan tag still removes fixed vanilla spell
  resrefs (`ArtisansKitpack/lib/redwizard.tpa`) instead of checking effective prohibited
  schools, and SR `60` custom-kit school cleanup is uninvestigated. EET `EDWIN7_` coverage
  is done (Modpack `236`). Not a reinstated blanket SR ban. Details:
  `../../handoffs/2026-09-10-red-wizard-sr-default.md`.
- [ ] Validate [Tweaks Anthology](CDTWEAKS.md) `2312` against the final SR/IWDification/SCS
  spell set, `2420/2430` against the final Artisan proficiency and item rules, and `3390`
  against final tracking-area scripts. Package the `3347` custom configuration (`125`
  percent movement, casting-speed increase off, portrait icon on); today `3347` installs
  without it.
- [ ] Artisan: add hidden, predicate-driven activation for tweak `1110` (`1010`+EEex) and
  `1010000` (`C0PR#CL.SPL` present); the feature schema has no file-exists predicate yet.
  Keep `30001`, `300010` and `8102` unavailable until priest delivery is redesigned.
  Latent: NPC `10004` does not require main `10003` (unselectable today because SR blocks
  it).
- [ ] Fix legacy EEex dependency checks: Artisan (`v4.81a-dev.85928f9-chriz.1`), Bardic
  Wonders (`v2.9c-balance.8`) and Bubb's Spell Menu still test EEex component `0`, which
  is only Quick Menu Core in EEex v1.2. Require main component `1` or use a
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
- [ ] [EEex](EEEX.md): loader launch and LuaJIT are shown in play; hotkey, effect-menu,
  empty-container, scale, time-step and timer behavior have no record.
- [ ] [EEex Remote Console](EEEXREMOTE.md): on target EET 2.7.3, record the ready
  handshake, `pong`, main-menu/world-screen polling and watchdog smoke. Ready files appear
  in 2.7.3 labs; earlier live evidence used BG2EE 2.6.6.
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md) Tempus APR: under EEex v1.2, verify
  steady 1.5 APR with a two-pip weapon, 2.5 under Holy Power, no cycling, and immediate
  loss of the bonus with a zero-pip weapon.
- [ ] [CHRIZ-SOD-REMIX](CHRIZ-SOD-REMIX.md) `225`: obtain all five items naturally and
  verify a live one-item-short refusal that consumes nothing and grants no XP.
  Activation, dormant re-click and save/reload were accepted on 2026-09-05.
- [ ] [Randomiser](RANDOMISER.md) v8.1.1 `1100` on a clean rebuilt collection and new
  game: Tarnesh loot smoke and the EEex-v1.2 legacy-BCS fallback. (Ordering, `510`
  numeric input, the `570` conflict and the `10300` omission with SCS `8040` are covered
  statically.)
- [ ] Run targeted live checks for the migrated tail fixes, especially modpack `400`,
  `410`, `430`, `440` and `450`; so far only file-level checks exist.
- [ ] [BuffBot](BUFFBOT.md): record the collection-specific InfinityLoader boot plus BG1
  start/save/reload smoke.
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
  under SCS, Bubb menu integration and EEex v1.2 behavior. Warlock is not in the recipe.
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

## Done since 2026-09-04

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

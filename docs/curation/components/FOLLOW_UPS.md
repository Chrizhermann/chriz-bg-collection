# Curation follow-ups

Last consolidated: 2026-09-01.

This is the single revisit queue for unresolved curation work. Component choices remain
authoritative in their individual catalogs; this file does not create or change a Chris
curation decision. Keep detailed reasoning in the linked catalog and check an item here
only when the required artifact, static validation, or live evidence actually exists.

Items under **Release gates** block the corresponding selected feature or first complete
stack. Items under **Conditional/future gates** do not block release while that feature
remains unavailable or excluded.

## Release gates — source and immutable pins

- [ ] [EET](EET.md) and [EET_END](EET_END.md): recheck official releases at manifest
  freeze. Otherwise resolve one exact official `master` SHA/archive for both, replace the
  stale v14.0 manifest source, and re-list both menus if the SHA changes.
- [ ] [EE Fixpack](EEFIXPACK.md): recheck packaged releases at freeze and pin one explicit
  artifact. Never silently follow unreleased `master`.
- [x] [EEex](EEEX.md): update the manifest from v0.11 to v1.2.0 and encode its refreshed
  nine-component all-on preset.
- [x] [Hidden Gameplay Options](HIDDENGAMEPLAYOPTIONS.md): update the manifest/pin list
  from the old v5.0/v5.1 state to the reviewed v5.2 target.
- [x] [Randomiser](RANDOMISER.md): immutable maintained-fork release v8.1.1 is pinned at
  commit `f4a9dfb1281269629e4a6fcedb6a064147dfde32` with its release-asset SHA-256 and size.
- [ ] [Artisan's Kitpack](ARTISANSKITPACK.md): replace all three stale upstream
  manifest entries with the released `chriz-v1.2.0` fork artifact at commit
  `f623045f58cb5c84ebb438f9ce32b1741405c637`. Publish a later fork release containing
  the Shapeshifter pathfinding-footprint fix before allowing `5110/5111`.
- [ ] [Bardic Wonders](BARDICWONDERS.md): replace the upstream snapshot with Chris's
  released `v2.9c-balance.2` artifact at commit
  `db0cf81504fd3f84e4e74eb8ab30e65499135512`. Integrate and release the later local
  Abettor finite-HLA work before retiring its standalone tail patch.
- [ ] [Aura](AURA_BG1_2_EET.md): reconcile Chris's unreleased balance fork with the seven
  newer upstream commits it lacks, then publish and pin the resulting exact source.
- [ ] [Bubb's Spell Menu](BUBB_SPELL_MENU_EXTENDED.md): update the manifest from v5.1
  to the reviewed v5.2 target.
- [ ] [Warlock](C0WARLOCK.md): pin exact commit
  `a821992228c87a4bef75be4bc2fc833f8db46819`, update the renamed
  `Artisans_Warlock/Artisans_Warlock.TP2` path, and stop describing it as a v3.0 target.
- [ ] [Spell Revisions](SPELL_REV.md): publish an immutable, fetchable
  `v4.21-chriz.1` fork source before replacing upstream v4.21.
- [ ] [CHRIZ-SOD-REMIX](CHRIZ-SOD-REMIX.md): the public v0.6.4 release is pinned and its
  complete 30-component bundle is ready. Keep unimplemented component `290` out of this
  preset and complete the natural five-item/save-reload acceptance for component `225`.
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md): publish and pin the test-first
  component-`401` compatibility correction for Artisan-packed `AP_C0PR#CL` cells. Release
  v0.3.0 now contains the accepted 14-component menu, including `120`/`121`, but its
  unmodified `401` cannot install on the curated stack.
- [ ] Resolve every “master SHA at authoring” and own-repository row in
  [the 2.7 pin list](../../pin-list-2.7.md) to an immutable SHA or release. Synchronize its
  stale pending conclusions for HGO, Spell Revisions, HQ soundclips, EEex,
  EE Fixpack, and EET with the completed catalog reviews.
- [ ] [Evandra](EVANDRA.md): define the approved user-provided/manual-archive acquisition
  path; the page-gated archive may not be rehosted.
- [ ] [Evandra](EVANDRA.md) and [Sarah](SARAHTOB.md): capture the actual custom portrait
  files, provenance, and hashes in an approved asset/manual layer.

## Release gates — installer and static integration

- [ ] Teach the manifest/engine to run [EE Fixpack](EEFIXPACK.md) `0+2` twice from one
  artifact: on staged BGEE+SoD after DLC Merger, and on staged BG2EE before EET `0`.
  Validate both 2.7 runs and the resulting merge.
- [ ] Enforce identical EET/EET_END artifact hashes. After EET_END `0`, permit only the
  explicitly inventoried collection patch layers, including the selected CHRIZ layers.
- [ ] Install [EEex](EEEX.md) `0`–`8` into a clean stage, not over v0.11. Audit stale
  loader, LuaBindings, and old `override/EEex_*.lua` files; verify loader launch and
  `[EEex] Uncap FPS Limit Enabled`.
- [ ] Encode [EEex Remote Console](EEEXREMOTE.md) `0` as optional, after EEex, and gated
  on EEex `1+8`. Refresh its repository documentation, which still describes EEex 1.0.x.
- [ ] Generate [Crossmod](CROSSMODBG2.md) after every supported NPC/quest mod and before
  CDTweaks; enforce `1 -> 0` and disable `2` under a multi-romance route. Check actual
  detection blocks rather than trusting its incomplete Project Infinity `After=` list.
- [ ] Install [Spell Revisions](SPELL_REV.md) `60` after every NPC/class assignment.
  Enforce the documented conflicts with SCS `4240`, Artisan `8101/8102`, Artisan NPC
  `5102/10004`, and Bardic Wonders `1006`; migrate `SR_SUBSPELL_FIX` into maintained code.
- [ ] Validate [Randomiser](RANDOMISER.md) v8.1.1 `1100` after `500`–`570` and after SCS
  on a clean rebuilt collection and new game. Include the Tarnesh loot smoke, EEex-v1.2
  legacy-BCS fallback, numeric input for `510`, the `570` conflict, and disabling `10300`
  when SCS `8040` is selected.
- [ ] Validate [Hidden Gameplay Options](HIDDENGAMEPLAYOPTIONS.md) late ordering and the
  special handling for `38`, `40`, `200/201`, key-binding choices, and GUI exclusions.
- [ ] Validate [Tweaks Anthology](CDTWEAKS.md) `2312` against the final SR/IWDification/SCS
  spell set; `2420/2430` against the final Artisan proficiency and item rules; `3390`
  against final tracking-area scripts; and package the `3347` custom configuration
  (`125` percent movement, casting-speed increase off, portrait icon on).
- [ ] Keep Artisan `30001` and its tweak `300010` unavailable until priest delivery is
  redesigned. Gate `8101/8102`, NPC `5102/10004`, and the documented Bardic/SR conflicts;
  enforce all refreshed main/NPC/tweak prerequisites and late-pass ordering. Add hidden,
  predicate-driven activation for tweak `1110` (`1010+EEex`) and `1010000`
  (`C0PR#CL.SPL` present); neither is globally mandatory.
- [ ] Script Bardic Wonders `1008` to answer **No** to its Garrick prompt and use Artisan
  NPC `99001` as the sole deterministic Troubadour assignment; enforce `99001→1008`.
- [ ] Fix legacy EEex dependency checks in the chosen Artisan, Aura `6`, Bardic
  `1012/2009`, Bubb `0`, and Warlock `0` sources. They still test EEex component `0`,
  which is only Quick Menu Core in EEex v1.2; require main component `1` or use a
  version-independent capability check. The all-on preset must not merely mask this.
- [ ] Script the chosen answer to Warlock's hidden Contingency UI spell-learning prompt;
  then test its pit-fiend spell loadout under SR, copied demon AI under SCS, Bubb menu
  integration, and EEex v1.2 behavior.
- [ ] Compare the effective spell changes made by historical
  [Aura Balance Patch - Spells](AURA_BALANCE_PATCH_SPELLS.md) v1.0 with Chris's maintained
  Aura fork. Do not declare the old component superseded or install both blindly.
- [ ] Validate [SCS](STRATAGEMS.md) 35.21 on game 2.7.3.0 with WeiDU 249. Upstream-author
  support for that combination remains unconfirmed.
- [ ] Implement and fixture-test the [IWDification](IWDIFICATION.md) `120` tail patch that
  adds level-7 Evasion to Arcane Trickster `C0ATR.2DA`.
- [ ] Keep upstream [EET Tweaks](EET_TWEAKS.md) `2042` excluded until the maintained
  progressive-XP replacement with the curated values exists and is tested.
- [ ] Validate maintained [chriz-bg-modpack](CHRIZ-BG-MODPACK.md) components `430`, `440`,
  and `450` on the resolved target stack. Keep `600` out of the fresh preset while UB `3`
  is excluded; validate it only in a dedicated legacy-UB fixture.
- [ ] [BG2EE/EET Fixpack](BG2EE-EET-FIXPACK.md): migrate Branwen `101` to a maintained
  conditional component, redesign NPC stat transfer `200`, audit Edwin `400`, and retire
  duplicate `300`. Never expose the historical package as a selectable parent.
- [ ] Add Spell Revisions fixture coverage for Bardic Wonders `1008`, `1009`, and `2008`
  before claiming broad compatibility.
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md): refresh stale repository component
  documentation and recheck whether final SCS supersedes `100` or `101`.
- [ ] Retire or implement every applicable item in the complete
  [22-fix migration inventory](COLLECTION_TAIL_FIXES.md); do not replay the historical
  tail installers or their captured order.
- [ ] Resolve the eight ownerless CIMC/CDMC animation INIs and the documented
  silent-writer coverage limit in the tail-fix inventory before claiming that every
  manually copied install-folder file has been reproduced.

## Live/runtime acceptance

- [ ] Boot and exercise the merged 2.7 EET result after both EE Fixpack runs.
- [ ] [EEex](EEEX.md): launch through its loader and exercise LuaJIT, hotkey, effect-menu,
  empty-container, scale, time-step, and timer behavior.
- [ ] [EEex Remote Console](EEEXREMOTE.md): on target EET 2.7.3, rerun ready handshake,
  `pong`, main-menu/world-screen polling, and watchdog smoke. Earlier live evidence used
  BG2EE 2.6.6 and is not target-stack acceptance.
- [ ] [CHRIZ-BG-REBALANCE](CHRIZ-BG-REBALANCE.md) Tempus APR: under EEex v1.2, verify
  steady 1.5 APR with a two-pip weapon, 2.5 under Holy Power, no cycling, and immediate
  loss of the bonus with a zero-pip weapon.
- [ ] [CHRIZ-SOD-REMIX](CHRIZ-SOD-REMIX.md) `225`: obtain all five items naturally;
  verify a one-item-short rejection consumes nothing and grants no XP; then verify exact
  activation, save/reload persistence, and dormant re-click with no cutscene/travel/actors.
- [ ] Run targeted live checks for every selected migrated tail fix, especially modpack
  `400`, `410`, `430`, `440`, and `450`. Test `600` only in its dedicated legacy-UB fixture.
- [ ] Visually verify the captured Sarah and Evandra portrait overrides in game.

## Conditional/future feature gates

These are nonblocking while their parent feature remains unavailable.

- [ ] Implement before exposing: a Safana-only semantic replacement for the old coupled
  modpack `100`; modpack `110`, `130`, `140`, `150`, `160`, `400`, and `410`; the new
  semantic per-NPC assignments accepted in the
  [tail-fix inventory](COLLECTION_TAIL_FIXES.md); `EVANDRA_SORCERER`; `SAFANA_LATE_SPELLS`; and
  `PRIEST_DELIVERY_FIX`. Move Xan coverage to Artisan NPC `20002` and `SR_SUBSPELL_FIX`
  to the Spell Revisions fork instead of implementing legacy modpack `170/180`. Follow
  each activation rule in
  [the tail-fix inventory](COLLECTION_TAIL_FIXES.md). Retire monolithic modpack `500`.
- [ ] [Safana](SAFANA.md): implement semantic SoD-item cleanup before making the parent
  available; separately rewrite the Bard spell/snare/class preset against effective
  Spell Revisions identities.
- [ ] Prove fresh-install retirement of modpack `200`, `210`, and `300`, and verify all six
  legacy Aura item changes before retiring that snapshot. Retire modpack `120` outright;
  never expose it separately.
- [ ] [Fade](FADE.md): expose the default Fighter/Thief choice only after modpack `110`
  exists; keep it mutually exclusive with Fade `2`.
- [ ] [Xan](XAN.md): move coverage for EET variants `XAN4`, `XAN6`, and `TTXAN` into the
  maintained Eldritch Knight path before enabling that dependent conversion.
- [ ] Keep Randomiser's expanded Mode-1 pool, SoD remix `290`, and other
  planned-but-unimplemented work unavailable until separately curated and accepted.
- [ ] [BuffBot](BUFFBOT.md): preserve `1` then `0` as the absolute final mod layer and
  complete the collection-specific InfinityLoader boot plus BG1 start/save/reload smoke.

## Curation decisions still in Chris's review queue

These are genuine choices that remain after the technical questions in the completed
catalogs were normalized:

- [ ] Curate newly discovered Artisan main components `1010`, `2003`, `5300`, `5200`,
  `7000`, `7007`, and `7008`, plus tweak components `20021`, `1020000`, and `2000001`.
- [ ] Curate newly discovered Bardic Wonders components `2011`, `2012`, and `2013`.
- [ ] [BGGO](BGGO.md) `4`: decide whether Alternate Graphics is excluded, optional, or default.
- [ ] [Warlock](C0WARLOCK.md): choose Yes or No for the Contingency UI-based
  spell-learning prompt. The component remains a default either way.
- [ ] Decide whether to include the separate BG1 Sirene v3.1 package in addition to the
  already curated BG2/EET Sirene route.
- [ ] Choose one restored-Bhaalspawn-powers provider: Ascension `40` (technical
  recommendation) or UB `19`; never expose both together.
- [ ] Confirm the remaining joint modpack details: retain Sarah as Archer; give Dynaheir
  Haste as known-only or known plus one memorized copy; and confirm Kivan is a kit-only
  Archer conversion with no automatic proficiency or loadout rewrite. Shar-Teel is now
  explicitly settled as a default Wizard Slayer.

# CHRIZ-SOD-REMIX — components

Pinned public release **v0.6.15** from `Chrizhermann/chriz-sod-rebalance` (published
2026-10-23). This catalog has 46 current or historical entries; the recommended EET
setup installs 41 components: the 40-row mandatory bundle plus Imoen choice `292`.
Rows `177`, `198` and `285` arrived with v0.6.13 (October 13 intake, see
[next batch intake](../../plans/2026-10-13-next-batch-intake.md)) and are mandatory
bundle rows. v0.6.14 added the Imoen choice `292`/`293`/`294`, and v0.6.15 lets `256`
install alongside Tweaks Anthology `3500`
([chriz-sod-rebalance#31](https://github.com/Chrizhermann/chriz-sod-rebalance/issues/31)).
Existing-save repairs `176` and `291` and alternative `901` stay out of the recipe. The [September 16 candidate](../../alpha16-candidate-2026-09-16.md)
approved normal components `115`, `135`, `256`, `265`, and `266` in the mandatory bundle,
with `257` optional and unchecked. Component `901` remains the unselected alternative
to `900`, and component `291` is a repair-only tail for existing `290` installations.
The Dev marks below retain their historical scope and do not certify the new additions.

## UI/preset behavior

- Present the parent-level checkbox **Include CHRIZ-SOD-REMIX**.
- The parent checkbox is `default` (checked initially) and can disable the whole mod.
- Except for optional Extra Challenge `257` and the Imoen choice below, do not expose
  individual component toggles. When the parent is enabled, install every
  row marked `mandatory` below in TP2 source order. This extends the reviewed setup with
  the accepted victory ending `290`, treasure choice `900`, and accepted EET SoD skip
  `910`; exclude repair `291` and alternative `901`.
- **Imoen in Shadows of Amn** (`292`/`293`/`294`, choose one; Christopher's decision,
  2026-10-16): at the end of SoD, Imoen's gear is handed over like the party's and she
  keeps only her BG2 belt. Then:
  1. `292` (**default**): the vanilla BG2 thief→mage dual-class Imoen (EET's `IMOEN10`)
     plus her BG1/SoD tome boosts. No EEex needed.
  2. `293` (optional): she continues as taken along.
  3. `294` (optional, requires EEex): she dual-classes to mage at her end-of-SoD thief
     level, and SoA's catch-up XP goes into mage.

  Each choice must work for every thief kit. All three need EET, EET_end and the
  mandatory bundle (`160` and `290`), and install after `910` in TP2 order.
- New-start components `177` (report Korlasz to Liia after the first night; needs `175`
  and `180`), `198` (early BG1 companion recruitment and camp follow-up; needs `110` and
  `197`) and `285` (Avernus portal and a bounded final battle) are mandatory bundle rows,
  in TP2 order after `175`, `197` and `280` respectively.
- Extra Challenge `257` is unchecked and depends on the mandatory bundle, including
  revised bridge `256`. It adds the Insane bridge sequencers without removing the
  normal bridge improvements when disabled. Component `266` separately limits the
  Shadow Aspect's Mislead; `265` does not provide that correction.
- The mod requires loose Siege of Dragonspear `BD*` content, either from BG:EE+SoD or an
  EET game with SoD imported. In this EET-only collection, make the parent unavailable if
  the staged game does not contain the required SoD resources.
- Install it as an explicitly inventoried post-EET_end collection patch layer, after the
  content mods whose final SoD resources it adjusts.

## Source/version and follow-up checks

- The pinned `v0.6.15` release changes only `256` against v0.6.14: Tweaks Anthology
  `3500` gives every summoned creature a `CDTFOW` self-cast, which v0.6.13's `256`
  refused on the fire-elemental donor `BDELFIRM`, so the Recommended selection did not
  install with `3500` on. `256` now accepts exactly that effect and leaves it out of its
  hostile clones. The owner lab reinstalled `256` and the rest of the remix tail on a
  disposable EET 2.7.3.0 copy of the Recommended stack with `3500` on; a complete
  Recommended collection install from this pin is still a release gate (see
  [FOLLOW_UPS](FOLLOW_UPS.md)).
- v0.6.14 added the Imoen choice `292`/`293`/`294` (EET only; `294` also needs EEex).
  The owner lab on 2.7.3.0 with EEex and Spell Revisions passed `292` and `293` on both
  routes (`290`'s ending and `910`'s bedroom skip) and `294` on the ending route, with
  and without a thief kit. Not covered there: a game without Spell Revisions, thief kits
  other than `C0TRICKS`, `294` on the `910` route and its mage level-up screen. A dead
  Imoen at the end of SoD stops SoA's opening scene; this already happens with `160` and
  `290` alone (owner known issue).
- The earlier `v0.6.11` release fixed Skie recruitment installation alongside Khalid
  continuity and the bridge changes, retaining the September 16 component choices.
  The earlier `v0.6.8` release made component `900` preserve existing camp-chest
  item records rather than rejecting anything except one sword; this fixes the reported
  three-item chest install failure. Component `910` is unchanged from v0.6.7.
  The release retains component `210` before dependent component
  `197`, and declares the fresh-install victory ending `290`, repair-only `291`, treasure
  alternatives `900`/`901`, then `910`; since v0.6.14 the Imoen choice `292`/`293`/`294`
  is the final native declaration.
- Include `290`; never select `291` in a fresh recipe. The release records native EET guard,
  victory, celebration save/reload, and one exact inventory-handoff acceptance. Expanded
  party variants, multiplayer, and native credits testing remain outside that evidence.
- Include EET-only `910` after selected treasure route `900`. Its TP2 requires EET,
  EET_end component 0, and remix components `110`, `140`, `150`, and `160`; it has no
  dependency on `290` or EEex. The owner reported 82 Windows/Linux tests, and the user
  accepted the native six-person Yes route: protagonist XP advanced from 162,209 by the
  configured 250,000 to 412,209, with the normal handoff reaching AR0602. No/reload and
  imported-Imoen variants remain follow-ups in owner issue #19 and are not release blockers.
- The sandbox and installed-state checks are green, and the earlier 30-component bundle
  is installed on the dev EET copy. Component `290` has bounded victory and inventory-handoff
  acceptance, but the new 32-component alpha.11 combination has not been freshly installed
  end to end. Runtime acceptance is not complete: component `225` still needs its natural
  five-item activation, one-item rejection, save/reload, and dormant re-click playthrough
  checks. Keep that work on the central curation follow-up checklist rather than treating
  the source checks as live acceptance.

| # | Component | Group | Subgroup | Dev | Decision |
|---|---|---|---|---|---|
| 100 | SoD remix: rest-ambush chance reduced 5x (felt rate) | Wave 1 - global system levers |  | ✓ | mandatory |
| 110 | SoD remix: keep all companions at SoD start | Wave 1 - global system levers |  | ✓ | mandatory |
| 115 | SoD remix: Khalid continuity at Bridgefort | Chapter pass - the road north |  |  | mandatory |
| 120 | SoD remix: remove the hooded man from the mid-campaign | Wave 1 - global system levers |  | ✓ | mandatory |
| 130 | SoD remix: skip the four chapter rest-dreams | Wave 1 - global system levers |  | ✓ | mandatory |
| 135 | SoD remix: assassin ambush without dead magic | Wave 1 - global system levers |  |  | mandatory |
| 140 | SoD remix: prologue - skip the Korlasz dungeon | Chapter pass - prologue |  | ✓ | mandatory |
| 150 | SoD remix: prologue - no assassination night, crusade council | Chapter pass - prologue |  | ✓ | mandatory |
| 145 | SoD remix: prologue - no auto-granted starting party (new/imported SoD) | Chapter pass - prologue |  | ✓ | mandatory |
| 160 | SoD remix: prologue - Imoen stays and is recruitable | Chapter pass - prologue |  | ✓ | mandatory |
| 170 | SoD remix: prologue - the Korlasz jailbreak | Chapter pass - prologue |  | ✓ | mandatory |
| 180 | SoD remix: prologue - celebration and Caelar's proclamation | Chapter pass - prologue |  | ✓ | mandatory |
| 175 | SoD remix: prologue - the XP ledger: Liia's reward after the jailbreak | Chapter pass - prologue |  | ✓ | mandatory |
| 177 | SoD remix: prologue - report Korlasz to Liia after the first night | Chapter pass - prologue |  |  | mandatory |
| 185 | SoD remix: prologue - Entar Silvershield removed (stays dead) | Chapter pass - prologue |  | ✓ | mandatory |
| 190 | SoD remix: prologue - Skie's second-night bedroom visit removed | Chapter pass - prologue |  | ✓ | mandatory |
| 195 | SoD remix: prologue - assassination and poison references scrubbed | Chapter pass - prologue |  | ✓ | mandatory |
| 210 | SoD remix: Coast Way Forest removed - Rasaad recruits at the camp | Chapter pass - Coast Way |  | ✓ | mandatory |
| 197 | SoD remix: prologue - Skie: talk-to-join recruit at the palace, SoD plot removed | Chapter pass - prologue |  | ✓ | mandatory |
| 198 | SoD remix: early BG1 companion recruitment and camp follow-up | Chapter pass - prologue |  |  | mandatory |
| 187 | SoD remix: prologue - the assassination night-set never spawns | Chapter pass - prologue |  | ✓ | mandatory |
| 200 | SoD remix: Coast Way Crossing - fewer spiders, fairer bridge fight | Chapter pass - Coast Way |  | ✓ | mandatory |
| 215 | SoD remix: the XP ledger - Coast Way Forest removal compensated | Chapter pass - Coast Way |  | ✓ | mandatory |
| 220 | SoD remix: the dwarven dig site re-garrisoned | Chapter pass - Coast Way |  | ✓ | mandatory |
| 225 | SoD remix: one text-only Caelar omen at the scrying pool | Chapter pass - Coast Way |  | ✓ | mandatory |
| 245 | SoD remix: Coast Way bridge - the wall stays gone when the cutscene is skipped | Chapter pass - Coast Way |  | ✓ | mandatory |
| 230 | SoD remix: the road north - fewer, cleaner enemy camps | Chapter pass - the road north |  | ✓ | mandatory |
| 240 | SoD remix: Forest of Wyrms - bugbear cave removed, temple behind the dragon | Chapter pass - the road north |  | ✓ | mandatory |
| 250 | SoD remix: Morentherene - a real dragon on Hard and Insane | Chapter pass - the road north |  | ✓ | mandatory |
| 255 | SoD remix: Boareskyr battle - durable explosive barrels | Chapter pass - the road north |  | ✓ | mandatory |
| 256 | SoD remix: bridge elemental finale | Chapter pass - the road north |  |  | mandatory |
| 257 | SoD remix: optional Insane bridge sequencers | Chapter pass - the road north |  |  | optional |
| 260 | SoD remix: coalition camp arc - fewer, cleaner enemies on the scouting maps | Chapter pass - the coalition camp |  | ✓ | mandatory |
| 265 | SoD remix: Mizhena amulet, filler cuts, and XP corrections | Chapter pass - the coalition camp |  |  | mandatory |
| 266 | SoD remix: finite Shadow Aspect Mislead | Chapter pass - the coalition camp |  |  | mandatory |
| 270 | SoD remix: Kanaglym - fewer undead | Chapter pass - the coalition camp |  | ✓ | mandatory |
| 280 | SoD remix: no party dispel at the basement reveal | Chapter pass - the coalition camp |  | ✓ | mandatory |
| 285 | SoD remix: Avernus portal and a fairer final battle | Chapter pass - the coalition camp |  |  | mandatory |
| 290 | SoD remix: end after the playable victory celebration | Chapter pass - victory ending |  |  | mandatory |
| 291 | Repair an existing component 290 EET handoff | Chapter pass - victory ending |  |  | excluded |
| 900 | Collected in the camp chest at the Coast Way crossing | Chapter pass - Coast Way | SoD remix: treasure from removed content | ✓ | mandatory |
| 901 | Removed along with the content | Chapter pass - Coast Way | SoD remix: treasure from removed content |  | |
| 910 | Offer to skip SoD at palace arrival (EET, experimental) | Chapter pass - prologue |  |  | mandatory |
| 292 | Shadows of Amn's Imoen (the BG2 dual-class build) plus her tome boosts | Chapter pass - victory ending | SoD remix: Imoen in Shadows of Amn (EET; her SoD gear is handed over like the party's) |  | default |
| 293 | Imoen as taken along (her own build continues) | Chapter pass - victory ending | SoD remix: Imoen in Shadows of Amn (EET; her SoD gear is handed over like the party's) |  | optional |
| 294 | Imoen dual-classes to mage at her end-of-SoD thief level (requires EEex) | Chapter pass - victory ending | SoD remix: Imoen in Shadows of Amn (EET; her SoD gear is handed over like the party's) |  | optional |

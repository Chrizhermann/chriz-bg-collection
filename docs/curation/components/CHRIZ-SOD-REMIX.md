# CHRIZ-SOD-REMIX — components

Target the newest implemented package version, **v0.6.4**, from
`Chrizhermann/chriz-sod-rebalance`. It has 31 component declarations; the established
dev setup installs 30 of them. Component `901` is the unselected alternative to `900`.

## UI/preset behavior

- Present exactly one parent-level checkbox: **Include CHRIZ-SOD-REMIX**.
- The parent checkbox is `default` (checked initially) and can disable the whole mod.
- Do not expose individual component toggles. When the parent is enabled, install every
  row marked `mandatory` below in TP2 source order. This reproduces the 30-component dev
  setup, including treasure choice `900` and excluding its alternative `901`.
- The mod requires loose Siege of Dragonspear `BD*` content, either from BG:EE+SoD or an
  EET game with SoD imported. In this EET-only collection, make the parent unavailable if
  the staged game does not contain the required SoD resources.
- Install it as an explicitly inventoried post-EET_end collection patch layer, after the
  content mods whose final SoD resources it adjusts.

## Source/version and follow-up checks

- Pin the public `v0.6.4` release. It orders component `210` before dependent component
  `197`, so the complete 30-component bundle can install in one run.
- Component `290` (the post-victory ending) remains unimplemented and is not part of this
  preset.
- The sandbox and installed-state checks are green, and all 30 selected components
  are installed on the dev EET copy. Runtime acceptance is not complete: component `225`
  still needs its natural five-item activation, one-item rejection, save/reload, and
  dormant re-click playthrough checks. Keep that work on the central curation follow-up
  checklist rather than treating the source checks as live acceptance.

| # | Component | Group | Subgroup | Dev | Decision |
|---|---|---|---|---|---|
| 100 | SoD remix: rest-ambush chance reduced 5x (felt rate) | Wave 1 - global system levers |  | ✓ | mandatory |
| 110 | SoD remix: keep all companions at SoD start | Wave 1 - global system levers |  | ✓ | mandatory |
| 120 | SoD remix: remove the hooded man from the mid-campaign | Wave 1 - global system levers |  | ✓ | mandatory |
| 130 | SoD remix: skip the four chapter rest-dreams | Wave 1 - global system levers |  | ✓ | mandatory |
| 140 | SoD remix: prologue - skip the Korlasz dungeon | Chapter pass - prologue |  | ✓ | mandatory |
| 150 | SoD remix: prologue - no assassination night, crusade council | Chapter pass - prologue |  | ✓ | mandatory |
| 145 | SoD remix: prologue - no auto-granted starting party (new/imported SoD) | Chapter pass - prologue |  | ✓ | mandatory |
| 160 | SoD remix: prologue - Imoen stays and is recruitable | Chapter pass - prologue |  | ✓ | mandatory |
| 170 | SoD remix: prologue - the Korlasz jailbreak | Chapter pass - prologue |  | ✓ | mandatory |
| 180 | SoD remix: prologue - celebration and Caelar's proclamation | Chapter pass - prologue |  | ✓ | mandatory |
| 175 | SoD remix: prologue - the XP ledger: Liia's reward after the jailbreak | Chapter pass - prologue |  | ✓ | mandatory |
| 185 | SoD remix: prologue - Entar Silvershield removed (stays dead) | Chapter pass - prologue |  | ✓ | mandatory |
| 190 | SoD remix: prologue - Skie's second-night bedroom visit removed | Chapter pass - prologue |  | ✓ | mandatory |
| 195 | SoD remix: prologue - assassination and poison references scrubbed | Chapter pass - prologue |  | ✓ | mandatory |
| 210 | SoD remix: Coast Way Forest removed - Rasaad recruits at the camp | Chapter pass - Coast Way |  | ✓ | mandatory |
| 197 | SoD remix: prologue - Skie: talk-to-join recruit at the palace, SoD plot removed | Chapter pass - prologue |  | ✓ | mandatory |
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
| 260 | SoD remix: coalition camp arc - fewer, cleaner enemies on the scouting maps | Chapter pass - the coalition camp |  | ✓ | mandatory |
| 270 | SoD remix: Kanaglym - fewer undead | Chapter pass - the coalition camp |  | ✓ | mandatory |
| 280 | SoD remix: no party dispel at the basement reveal | Chapter pass - the coalition camp |  | ✓ | mandatory |
| 900 | Collected in the camp chest at the Coast Way crossing | Chapter pass - Coast Way | SoD remix: treasure from removed content | ✓ | mandatory |
| 901 | Removed along with the content | Chapter pass - Coast Way | SoD remix: treasure from removed content |  | |

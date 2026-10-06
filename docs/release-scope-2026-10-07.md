# October 7 release checklist

Christopher authorized finishing the approved mods, the CEBG release, a complete
changelog, website publication and scoped test cleanup. This is the working
checklist, not evidence that unfinished steps have passed.

## Include

| Change | Owner / status | Collection and release action |
| --- | --- | --- |
| Hardiness: 40% physical; 30% alternative; extra elemental/magic protection optional | Rebalance 200/201/202, implemented | Publish owner release; wire choices and dependencies |
| Party physical resistance: half effectiveness above 80%, maximum 90% | Rebalance 310, implemented | Include normal selection; require EEex |
| Approved Pierce Magic and Spellstrike adjustments | Rebalance 210/211, implemented | Include with SR prerequisites; 210 also requires EEex |
| Damageable combat cloud forms, 75% resistance | Rebalance 320, implemented | Integrate owner changes and publish; preserve noncombat escapes |
| Bracers of Blinding Strike use SR Improved Haste | Published Rebalance 0.8.1, component 220 | Selected with SR |
| EET elemental arrows use BG2 values | Published Rebalance 0.8.1, component 130 | Selected with EET/SCS |
| Edwin's Red Wizard amulet/slot correction | Published Modpack alpha.10, component 236 | Selected only with Artisan Red Wizard |
| Baeloth's legal spells known | Modpack 235 implemented | Optional Challenge-related choice, after SR spellbook fixes; not an unconditional default |
| BuffBot 1.10.0 | Published | Update fresh-install pin; preserve the separately supported historical update route |

Already present in the October 1 candidate and still part of this release:
alternative UIs, manual install breakpoints, clearer progress/completion,
unique install names, optional Radar launch, Evandra conversion/portrait defaults,
story-stage utility XP (smooth progression optional), optional generic unidentified
item visuals, optional talking while shapeshifted, Emotion no-stacking, Flail of
the Ages balance and default longer Wing Buffet cooldown.

The changelog must compare against the last **public** release, not merely the
October 1 candidate. Every row above needs an immutable released source and actual
recipe selection before it may be described as included.

## Keep out

Unimplemented 95% cap; XP banking; Entropy Shield/Dispelling Screen redesigns;
instant trap detection; reduced NPC chatter; new Mystic Fire/Fighter balance;
general whole-install updating; a complete Challenge Mode umbrella; unverified
Fade/Evandra XP catch-up fixes; unresolved scroll reader-level policy.

## Release gates

- [x] Owner changes combined without losing unrelated work; focused checks pass.
- [x] Owner releases published and downloaded hashes verified.
- [x] Both shipped recipes, generator, defaults, dependencies and exclusions agree.
- [x] New collection version and complete change ledger recorded.
- [ ] Automated installer/recipe/package checks pass; failures investigated.
- [ ] Final signed package scanned with Defender protections unchanged.
- [x] October 1 app: Radar-off launch checked with no existing Radar process.
- [x] October 1 app: Radar approval launch checked; decline remains untested.
- [x] Complete installer + collection changelog verified against shipped selection.
- [ ] Versioned release assets verified before updating the alpha feed.
- [ ] Website download, versions and changelog updated and checked live.
- [ ] Only this session's disposable test artifacts cleaned; evidence retained.

## Testing boundaries

Use Test19, not the stream installation. No stream saves/game edits, quarantine
restoration, Defender exclusions, or protection changes. Windows administrator
consent must be clicked by Christopher; it cannot be automated. Test19 shares
Steam's cloud namespace despite its separate local profile: do not load, save or
delete cloud saves during launch tests. The completed October 7 Defender result
is recorded in `defender-smoke-2026-10-07.md`; it is bounded reproduction evidence,
not a Microsoft guarantee for future builds.

### Radar follow-up, 06:25-06:29 KST

After Christopher closed old Radar, the exact October 1 setup was temporarily
installed under `target/radar-smoke-oct07` (NSIS exit 0; app SHA256
`d1850439a0eec13c4c8200f268df53a025f018e559718ad0473b024e1f42ad4f`).
Unchecked Play reached Test19's main menu without starting Radar. After exiting,
checked Play launched Test19 and its own `game/BG Radar Overlay/BG Radar Overlay.exe`;
Christopher confirmed approving the administrator prompt. CEBG displayed
"Game and Radar started" and the overlay window was present. This verifies launch,
not combat readings. No save was loaded or manually saved/deleted.

Physical Escape stopped Computer Use; no further UI input was sent. At 06:29 KST
the game, loader and CEBG were closed, while the elevated test Radar PID 89940
remained. Defender had zero new 1116/1117 events since 06:25 and all four protection
flags remained enabled, signature 1.459.576.0. Decline was not tested. The
temporary CEBG app was uninstalled separately; retain packages and test game.

Owner mod releases are published; the CEBG package and website are not yet published.
Target remains app alpha.19; the new collection payload will use alpha.19.
Public feed verified today remains app alpha.18 / collection alpha.16.

### Active release continuation

The earlier pause was an error; Christopher reconfirmed that release approval
remains in force. No further computer control is needed for source/package work.

- Rebalance **v0.8.1 is published Latest**, tag commit
  `09db97dc5cbd14f1b8bf7fe3af64f69bee313585`. Full owner suite: 539 tests,
  26 absent-private-capture skips, exit 0; six subsequent public/package checks
  passed. Independent review passed. Main freshly downloaded and verified its
  production archive contract and TP2 order in both recipes.
- Modpack 235/236 passed 967 Windows tests with two skips, including real WeiDU
  and extracted-package checks. Independent review passed. Linux CI caught a
  mixed-case fixture log filename, corrected before protected merge. Main Linux
  CI and tagged Windows packaging passed. **Alpha.10 is published Latest**, commit
  `210472d98cb0a66fc2003af886ce545af11e8d56`. Fresh download, production extraction,
  native TP2 order and ten extracted-package tests passed. The prematurely pushed
  alpha.9 tag remains unpublished and was not moved.
  Order: 236 after Artisan NPC5102/before SR60; 235 after SR60/before BuffBot.
- Website source and complete collection release notes are prepared. 115 site
  tests and the build passed. Existing approved rules and non-collection content
  are unchanged. Download remains explicitly pending final public setup bytes.
- Collection features/defaults, native run order, curation and regression tests
  are authored. Default expansion: 462 components in 51 runs. Frontend 396 tests,
  typecheck and build pass. Native compilation completed exit 0; 35 focused Rust
  recipe tests and six October intake/selection Python tests passed. Independent
  recipe review found no blockers. Both public-alpha profiles validate with zero
  findings; 24 intake/metadata/expanded-recipe tests pass. Static evidence cites
  source commit `fd0806d3db056239b833e424411807bb11dd1aef`. The combined suite,
  package, final Defender scan and publication remain.

# Chriz Easy IWD — follow-ups

This queue tracks outstanding work; it does not select components or replace the
user's decisions. Started 2026-09-10.

## First public release (0.1.1)

Christopher, 2026-10-28: the hotpatch is the last app feature he needs before the first
community release; he proposed the version 0.1.1.

Christopher, 2026-10-29: finish the installer with the hotpatch for the community, publish the
download repository and an English IWD page on the website as discussed. His message is the
go for merging #42 and #52, the three hotpatch defaults, the release and the website.

- [x] Hotpatch: built with Christopher on `claude/iwd-hotpatch`, merged into `claude/iwdee-app`
  as PR #52 (2026-10-29). Updates → Hotpatch with a changelog per entry, offered only to
  installations close to the recommended setup; first use is boss upgrades from Chriz IWD
  Encounters.
- [x] Chriz IWD Encounters 0.1.0 in the recipe: PR #42, merged 2026-10-29.
- [x] Own version line for the IWD app, independent of Chriz Easy BG (f97631d): `version` 0.1.1
  in `tauri.iwd.conf.json`, which the app now reports everywhere (receipts, Updates, traces).
  The `v0.1.0-alpha.1` records stay, since the public-alpha profile reads its acceptance and
  limitations there; the new ledger `releases/v0.1.1/ledger.toml` describes what 0.1.1 adds.
  The hotpatch catalog's base is the 0.1.1 recipe digest.
- [x] Release lab (2026-10-29, design doc "Release lab check"): the first game start writes
  `InfinityLoader.db`, `EEex_scripts/extra_pattern_dbs/v2.7.3.0.db` and the empty folder
  `EEex_scripts/override`, so every played installation would have been refused as edited by
  hand. Fixed in 9818633 and bf9f763 (runtime files, also the remote console's). Then a played
  and saved installation passed; a stand-in hotpatch writing `dialog.tlk` and `override`
  applied, reached a save made before it, undid byte for byte and applied again; a deviating
  installation was refused with both reasons.
- [x] Installer built from bf9f763 (CI green) and verified: version 0.1.1, the bundled recipe and
  hotpatch catalog identical to the branch. SHA-256
  `3c07a3913f99181124511192653dfc6b87bd706743473ef9b382072c698b22a1`.
- [x] Overview in `docs/releases/iwd/`: `public-README.md` (README of the download repository)
  and `0.1.1.md` (release notes with checksum and source commit).
- [x] Public download repository `Chrizhermann/chriz-easy-iwd` (2026-10-29), like
  `chriz-easy-bg`: only the README and the release files, no code. Release v0.1.1 with the
  setup, `SHA256SUMS`, `LICENSE` and `THIRD_PARTY_NOTICES.md`; the downloaded asset matched.
- [x] Website: English IWD page `/iwd` on bg.chrizfader.org (`bg-site` in `twitch-setup-chriz`),
  same layout as the BG page, with download link and checksum. Rebuilt 2026-10-29 together
  with the BG page as short offer pages in the apps' themes (Glacier here, Parchment for BG):
  site PR 39, verified live.
- [x] "17 new companions" reads "17 companions" on the page, in the release notes, the download
  README and the Discord draft (Christopher, 2026-10-29: nobody had them before installing).
- [x] Discord announcement: posted by Christopher on 2026-10-29.
- [ ] 0.1.2: Marketh's Ring (Unfinished Business) is offered but off, since a player reports that
  Marketh can vanish with his signet and the quest cannot be finished (Christopher, 2026-10-29).
  PR #65 on `claude/iwdee-app` also numbers recipe and app 0.1.2 and adds base `iwd-0.1.2`. It
  awaits his merge OK; building and publishing the 0.1.2 installer waits for his OK too.
- [ ] A later release that changes the recipe needs a new `[[bases]]` entry with its digest;
  hotpatches name every base they fit (`iwd-0.1.1`, `iwd-0.1.2`).

## Curation and sources

- [x] Record the name, same-repo architecture and existing user choices.
- [x] Set up the BG-style worksheet/component-review workflow and additions intake.
- [x] Inventory shared candidate mods and exact source versions against the reviewed BG collection
  (2026-10-25/26: Tweaks Anthology, SCS, EEex, EE Fixpack, Hidden Gameplay Options, Spell Menu
  and Remote Console use the same versions as the BG recipe; Infinity UI++ is 1.31 here, 1.28 there).
- [x] Extract full candidate menus and separately verify IWDEE predicates. Keep pending
  inventories outside authoritative catalogs; preserve numeric IDs and semantic labels.
  (2026-10-25: all 37 selected mods, WeiDU 249 interactive pass plus static predicate check.)
- [x] Component selection (sent to Christopher 2026-10-26 as a private selection page):
  434 installable parts; 386 preselected where the BG installer's decision carries over one to
  one (same version and label) or the choice is clear (a mod's main part, no save patching);
  48 open plus two questions. One question is SCS's spell tweaks: BG makes "Install all spell
  tweaks" mandatory, but on IWD:EE it also brings BG versions of nine IWD spells, against the
  2026-09-08 reason for leaving out Spell Revisions. Done 2026-10-26: his choices are in the
  worksheet, SCS gets all spell tweaks, and the recipe holds every decision.
- [x] Hidden Gameplay Options is on by default (Christopher, 2026-10-28), with its 28 default
  options (`claude/iwdee-app`, 13bdc15; release evidence c9791b5). Next to Infinity UI++ or
  Lefreut's interface the mod refuses four or five of them; the installer leaves those out.
- [x] Druids keep their own XP table by default (Christopher, 2026-10-27, as in the BG
  installer): Tweaks Anthology's cleric XP table for druids is offered but off, since on it
  druids reach level 15 far too early (`claude/iwdee-app`, 0f58707; release evidence 5b2ad9b).
  His first IWD test installation still has it; installations from the rebuilt app do not.
- [x] Review SCS and CDTweaks choices in spoiler-free batches (covered by the component selection).
- [ ] Move the recipe generator and a readable decision catalog into the repository. The
  2026-10-26 recipe was generated by a script outside the repository from the selection
  page; until the generator is here, change a decision by hand in `collection.toml` and
  record it in the worksheet. The generator is kept in Christopher's private backup. Since
  2026-10-26 it also writes the exclusive choice groups and the "Chriz IWD Collection"
  records folder. Since 2026-10-27 it also adds BuffBot, which Christopher chose after the
  selection page, and again reproduces the committed recipe exactly. Since 2026-10-28 it also
  turns Hidden Gameplay Options on, which the selection page had left open.
- [x] Alternatives are exclusive: the options of each of the 30 choice groups conflict
  pairwise, so the installer offers them as one choice and installs at most one
  (`claude/iwdee-app`, b4b0a2c; pinned by a static test, 17a563a).
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
- [x] BuffBot: version 1.12.0 supports IWD:EE 2.7.3.0 (tested in its own repository with
  EEex 1.3.1 and Bubb's Spell Menu: panel, presets, casting, save/load). Christopher put it
  into the collection on 2026-10-27, on by default (`claude/iwdee-app`, 672940d). It installs
  late: its LuaJIT support, then the buff automation. A lab install on top of the full lab
  of 2026-10-26 (SCS, companions, Tweaks Anthology, Hidden Gameplay Options) passed without
  warnings; the LuaJIT support found EEex's LuaJIT active and changed nothing. Its readme
  lists the IWD:EE interface mods, SCS and the companion mods as not yet tested together
  with it in play.
- [ ] BuffBot updates inside the IWD app stay off; Christopher does not need them for now
  (2026-10-27). A new BuffBot version comes with a new collection version.
- [ ] Verify selected Artisan/Bardic components and custom NPC interactions.
- [x] Add explicit collection identity/game profiles to shared app/engine contracts,
  including one-source staging; preserve existing BG campaigns. `claude/iwdee-app` builds
  the app "Chriz Easy IWD" from the same source with `tauri.iwd.conf.json` (78a1370,
  e37b73d): its own name, app and records folders ("Chriz IWD Collection"), one IWD:EE
  source, IWD texts, and no game fixes or BuffBot updates. Each app refuses the other's
  recipe. The BG app is unchanged; all tests pass in both builds, and CI checks both. The
  interface stays English (Christopher, 2026-10-27: his community is English-speaking).
- [x] BG Radar Overlay in the IWD app (Christopher, 2026-10-27; ae99433): the overlay finds
  IWD:EE too. The app adds it after an installation, offers it on Updates and starts it with
  the game, as in CEBG.
- [ ] Per-collection updates: the IWD app has no update channel yet, so a new version is a
  new download.
- [x] Manual pauses (stopping an install after a chosen mod to add something by hand) in the
  IWD app (Christopher, 2026-10-27; ae99433). Their screens name the managed game folder
  instead of BG1/BG2 and say that manual mods turn off mod update checks for that
  installation. An engine test covers a pause in a one-game installation.
- [x] An app icon of its own in the "Glacier" look (Christopher, 2026-10-28). He picked the
  "Glacier book": the CEBG icon's book and badge in Glacier colors, with a snowflake in the
  badge (`claude/iwdee-app`, 4bfd378; source `app/src-tauri/icons/iwd/icon.svg`). The setup
  file shows it too (aa28e96); the BG setup still has NSIS's default icon.
- [x] Each app has its own look (Christopher, 2026-10-27): "Glacier" for Chriz Easy IWD
  (frost white, glacier blue, Marcellus headings), "Parchment" for Chriz Easy BG; only colors,
  fonts and ornaments change (`claude/app-themes`, 8c2af38). With Christopher's OK it was
  merged into `claude/iwdee-app` on 2026-10-28 (1ac2de8), so it reaches main with the IWD app
  after alpha.22. IWD installers built since then have the theme.
- [x] Release records: `recipes/iwd-current` passes the public-alpha check with 0 findings
  (f9b1725; with BuffBot again since 1d56286). The check now counts only parts a release
  promises, not the parts of a mod it leaves off (f413da4). Runtime acceptance stays pending
  until an install through the app is accepted.
- [x] Build and verify a self-contained IWD recipe, source hashes and component order.
  Artifact pins for all 37 mods are on `claude/iwdee-recipe`; Lava's 19 packages use the
  engine's `weaselmods-download` source kind (`claude/iwdee-engine`) and were verified live.
  `recipes/iwd-current` (collection, preset, game profile) followed on 2026-10-26 (6efb95b,
  validate: 0 findings); the order follows the mods' readmes and each run says why.
- [x] Engine: a pre-spawn process check stopped the lab install of 2026-10-26 when another
  program's WeiDU was exiting: Windows denies the image query of an exiting process until it
  is signaled. Fixed on `claude/iwdee-engine` (9185bc1): such a process gets up to 5 s to
  finish exiting; one that keeps running still stops the install.
- [ ] Carry that process-check fix into CEBG's engine once alpha.22 has landed; the BG app
  has the same code and the same exposure while other programs run WeiDU.
- [x] Complete an isolated installation with exact log/receipt reconciliation. 2026-10-26, in a
  disposable lab with the engine CLI: the recommended preset plus Hidden Gameplay Options,
  123 components from 26 mods, final verification and receipt passed; the game got its own
  profile. Only warning: SCS's "Install all spell tweaks" found nothing to change in two
  spells (SPIN150, SPIN191).
- [ ] Complete launch, creation/levelling, spell/kit/song, NPC and save/load smoke checks.
  Launch done 2026-10-26: that lab booted quietly to the main menu with EEex (LuaJIT) and
  Bubb's Spell Menu active and no error box; the rest is open.
- [x] Repeat the install through the IWD app itself, so its screens are tested too.
  Christopher's own run on 2026-10-27 (a build before BuffBot, 94 components) installed and
  played fine.
- [ ] Gather campaign gameplay/balance evidence separately from installation success.

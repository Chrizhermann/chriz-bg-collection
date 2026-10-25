# IWD:EE mod inventory: Chriz Easy IWD

Status 2026-10-25. Read-only research by a subagent. Neutral facts only: no recommendations, no preselected defaults, no curation. Spoiler-free: no recruitment locations, plot, quest solutions, enemy identities or rewards.

- **Target game:** IWD:EE 2.7.3.0. A read-only check of the local reference install found `Icewind.exe` 2.7.3.0, `data/PATCH26.BIF`, and `BANTTIMG` in `chitin.key`.
- **BG collection (CEBG) reference:** `origin/claude/cebg-installer-fixes-20261014` at 5106b53 (2026-10-25). Sources: `manifest/mods`, `manifest/artifacts`, `manifest/collection.toml` and `recipes/curated-full-current`.
- **Owner update (via the coordinator):** "No reload" is a stream rule only, so the installer needs no mechanism for it. The stat-roll limit is covered in section 9.
- **Licence wording:** clauses are described in plain words, with the file and link. Only one clause is quoted word for word (section 12), because of quoting limits. Read the linked file for the exact wording.

## Keys

**Fetch**
- **GH-rel:** a versioned GitHub release asset, so a stable URL.
- **tag-zip:** a GitHub tag archive (`archive/refs/tags/<tag>.zip` or codeload). It is stable unless the tag is moved.
- **branch:** there is no release or tag for the current state. It needs a commit pin (CEBG kind `github-commit-zip`).
- **WPDM:** a weaselmods download page (WordPress Download Manager), at `https://downloads.weaselmods.net/download/<slug>/`. The page URL is stable, but the file link on the page carries a per-visit `refresh` token. The token-free `?wpdmdl=<post id>` link serves the same archive without cookies or login: on 2026-10-26 all 19 pinned Lava packages downloaded through it with the pinned length and SHA-256, and the installer fetches them that way. A Dropbox folder mirror also exists: `https://www.dropbox.com/sh/56dr4q8h2q7djri/AACJfi0A2mV7hFlH3eqDLTcEa?dl=0`.
- **forum:** a forum attachment.

**IWD:EE**
- **explicit:** the author's documentation says so.
- **gate:** a tp2 `GAME_IS` predicate admits `iwdee`.
- **implied:** IWD-specific content, or no gate, but no statement.
- **none:** a gate or the documentation excludes IWD:EE.
- **unclear:** none of the above was found.

**Terms**
- **none found (X):** the listed files or pages were read and contain no licence or redistribution clause.
- **G3 text:** the standard Gibberlings3 readme paragraph. It names hosting and redistributing a mod without its authors' permission as one of two ways to upset the community.

**CEBG:** the pin the BG collection uses, or "—".

## 1. Core / engine

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| EEex | Bubb | v1.3.1 (2026-10-07) | GH-rel `https://github.com/Bubb13/EEex/releases/download/v1.3.1/eeex-v1.3.1.zip` (also .exe and .iemod) | **explicit:** README support table lists IWD:EE v2.7 with EEex ≥ v1.1.0. **gate:** `GAME_IS ~bgee bg2ee eet iwdee~ AND FILE_EXISTS ~data/PATCH26.BIF~`; the local install has PATCH26.BIF | none found (README; repo has no LICENSE) | Windows only. Start the game through InfinityLoader.exe. Install early | Engine extender (Lua); many mods below need it | eeex-1.3.1 (same zip) |
| EE Fixpack | Gibberlings3 team | Beta_2, a GitHub prerelease (2026-05-04; readme says May 3, 2026) | GH-rel `https://github.com/Gibberlings3/EE_Fixpack/releases/download/Beta_2/enhanced-edition-fixpack-beta-2.iemod` (also exe and lin/mac zips). `releases/latest` returns nothing because it is a prerelease | **gate:** bgee, bg2ee, iwdee, pstee; `!GAME_IS ~eet~` | G3 text | Install first (ini `Before` lists cdtweaks, iwdfixpack, ub_iwd …). Readme names 2.6.6.0 as the official patch | Data fixes for the EEs | eefixpack-beta2 (components 0, 2) |
| Bubb's Spell Menu Extended | Bubb | v5.2 (2026-05-11) | tag-zip `https://codeload.github.com/Bubb13/Bubbs-Spell-Menu-Extended/zip/refs/tags/v5.2` (no assets; master holds only the README) | **explicit:** repo description names IWD:EE. **gate:** `bgee bg2ee eet iwdee` at the tag | none found (README) | Needs EEex and UI.MENU. Install after EET, Dragonspear UI and LeUI. Layouts exist for Infinity UI++ and for Dragonspear UI++ (the latter with an IWD offset); LeUI layouts exist only for the BG branch, so LeUI-IWDEE gets the default layout. **Conflict:** Tweaks Anthology readme says its Cantrips feat (6330) does not work with this menu | Spell-selection UI | bubb-spell-menu-5.2 (same URL) |
| EEex Remote Console | Chrizhermann | v0.2.1 (commit 3d98990; no GitHub release) | branch → commit zip, as in CEBG | **gate:** `bgee bg2ee eet iwdee` | MIT | EEex | Tooling: remote Lua console | eeexremote-0.2.1 |
| WeiDU | WeiDU maintainers | 249 | per CEBG artifact | Works with any game | — | — | Installer | weidu-249-amd64 |

## 2. Gameplay / AI / rule overhauls

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| Sword Coast Stratagems (SCS) | DavidW | v35.21 (2024-11-21) | GH-rel `https://github.com/Gibberlings3/SwordCoastStratagems/releases/download/v35.21/lin-stratagems-35.21.zip` (also osx zip and exe) | **explicit:** readme calls IWD:EE support experimental and limited and lists IWD:EE as version 2.6 | Asks that the mod and modified versions not be mirrored elsewhere, and asks for credit when code is reused (readme) | v35.19 rebuilt the IWD:EE help-call system. Talents of Faerûn 80100 (point-buy, 75 points), 80150 and 80160 carry `FORBID_COMPONENT` for SCS 5900 (AI initialisation); the ToF readme says to install ToF before SCS | On IWD:EE: general AI with calls for help, mages, priests, potions for NPCs, dragons, beholders, fiends, IWD:EE-only revised undead; inquisitor dispel, shapeshifting, thief points in 5s, ease-of-use party AI, IWD:EE-only wider wizard-scroll availability; EE-only revised elementals and inns; most spell tweaks, new spells (9 arcane, 6 divine), IWD:EE-only BG-style insect spells and Mordenkainen's Sword. Not on IWD:EE: see section 11. Taken from the readme; gates were not checked per component | stratagems-35.21 |
| Tweaks Anthology | Gibberlings3 (CamDawg et al.) | v18 (2025-08-22); main 7649ced (2026-08-31) | GH-rel `https://github.com/Gibberlings3/Tweaks-Anthology/releases/download/v18/the-tweaks-anthology-v18.iemod` (also exe and lin/mac zips). For main, use a commit pin | **gate/readme:** 105 of 256 components list IWDEE | G3 text | Luke's feats (6000–6360) and 2670 need EEex. 6330 (Cantrips) conflicts with Bubb's menu. 3210 excludes Talents of Faerûn's ability-score components | IWD-relevant examples: 230 IWD tooltips; 1290 NPCs answer the main character; 1361 automatic transition to HoW; 2320 trap-cap removal; 2510 level-locked scrolls; 2670 limit resting (EEex); 3210 minimum-stats cheat; 3358 interval saves; 3400 loading hints; 3520 "bank" XP when dual-classing (EEex) | cdtweaks at 7649ced (a cdtweaks-18 artifact is also defined) |
| Hidden Gameplay Options | argent77 | v5.2 (2026-08-26) | GH-rel `https://github.com/Argent77/A7-HiddenGameplayOptions/releases/download/v5.2/win-A7-HiddenGameplayOptions-v5.2.zip` (also iemod, lin and mac) | **explicit and gate.** 303 is IWD:EE-only. 17 is not offered on IWD:EE because the game already has that option | CC BY-SA 4.0 | 300, 301 and 304 (death, quit and load prompts) do not install with LeUI, LeUI-IWDEE, IUI or DUI. 28, 29 and 30 are blocked with LeUI. Conflicts with EEUITweaks 1010. A commit after 5.2 adds LeUI-IWDEE support (unreleased) | Exposes hidden engine options | hidden-gameplay-options-5.2 |
| Talents of Faerûn (ToF) | DavidW | Beta 15 (2024-11-21) | GH-rel `https://github.com/Gibberlings3/TalentsOfFaerun/releases/download/ToF-beta-15/lin-dw_talents-beta-15.zip` (also exe and osx) | **explicit:** readme lists IWD:EE version 2.6. Gated per component: the IWD spell and bard-song imports (1500, 1510, 1520) are not for IWD:EE; 1600, 1610 and 2020 are IWD:EE-only | Same request as SCS not to mirror the mod; material in `hla_feat` is CC BY-NC-SA 3.0 | Install as late as possible: before SCS, and probably before Tweaks Anthology. 80100, 80150 and 80160 forbid SCS 5900; all point-buy options (80100–80104) exclude cdtweaks 3210. The subrace component cannot be used when starting a new game in HoW | About 85 mini-mods: feats, HLAs, kits, Favored Soul class, subraces, spell spheres, ability-score rules including point-buy | — |
| Improved Heart of Fury Mode | OlvynChuru | README says 4.1.0, tp2 says 4.0.2; no releases; master 40b263e (2022-12-07) | branch → commit pin | IWD:EE-only mod | none found (README, tp2) | One component needs EEex | Reworks Heart of Fury mode | — |
| ZSTweaks | szaumoor | 1.7.0c (2026-09-29) | GH-rel `https://github.com/szaumoor/ZSTweaks/releases/download/1.7.0c/win-ZSTweaks-1.7.0c.zip` | **gate:** many components admit iwdee | GPL-3.0 | Install after content mods (items, quests, NPCs, IWD:EE spell pack) | Tweak collection | — |
| IWDEE Tweaks and Fixes | Sauler89 | v0.4 (2026-08-22) | GH-rel `https://github.com/Sauler89/IWDEE-Tweaks-and-Fixes/releases/download/v0.4/IWDEE-Tweaks-and-Fixes-v0.4.zip` | **gate:** `iwdee` | Custom licence: use, copying, modification and redistribution of the mod's own code are allowed under conditions, credit among them | — | IWD:EE tweaks, e.g. BG2-style HLAs | — |
| Polymorph fixes for IWDEE | D2-mods (Dan_P) | v2.6 (2025-08-20) | GH-rel `https://github.com/D2-mods/Polymorph-fixes-for-IWDEE/releases/download/v2.6/polymorph-fixes-for-iwdee-v2.6.iemod` | **gate:** `iwdee` | none found (README, tp2, ini) | — | Shapeshift/polymorph fixes plus small fixes | — |
| Better IWD Pregen | D2-mods | v9.2 (2026-10-13) | GH-rel `https://github.com/D2-mods/Better-IWD-Pregen/releases/download/v9.2/better-iwd-pregen-v9.2.iemod` | **gate:** multi-game, including iwdee | none found (README; only third-party "used with permission" credits) | — | Script and tweak pack | — |
| Enhanced Powergaming Scripts | SarahG | tags v5.1 and 5.2; no assets; HEAD 2021-10-02 | tag-zip | **explicit:** README names IWD:EE. **gate:** some components | Forbids mirroring, borrowing or altering its content unless the author explicitly permits it | ini `After`: EET_end, spell_rev, item_rev, …, stratagems, iwdification | Party AI scripts | — |
| klatu tweaks and fixes | The Gate Project | Version-1.7.4 (2024-09-10); main updated 2026-10-24 | GH-rel `https://github.com/The-Gate-Project/klatu-tweaks-and-fixes/releases/download/Version-1.7.4/klatu-tweaks-and-fixes-1.7.4.iemod` | **explicit:** README game list includes IWDEE. **gate:** some components | none found (README, tp2) | — | Tweaks and fixes | klatu-tweaks-1.7.4 (curated recipe) |
| randomrolls | drake127 | v1.2.1 (2026-09-27) | GH-rel `https://github.com/drake127/randomrolls/releases/download/v1.2.1/randomrolls-win.zip` | **explicit:** README lists IWD:EE for game versions 2.6 and 2.7. **gate:** includes iwdee | GPL-3.0 | Patches the game executable (`Icewind.exe` among others) | Hardware random numbers (RDRAND) for game rolls | — |
| Achievements! | argent77 | v3.0 (2025-06-22) | GH-rel `https://github.com/Argent77/A7-Achievements/releases/download/v3.0/A7-Achievements-v3.0.iemod` | **gate:** includes iwdee | CC BY-NC-SA 4.0 | — | Steam achievements as journal entries | — |
| S9 House Rules | repo mleduque/S9HouseRules | tag v1.1; HEAD 2022-04-19 | tag-zip | **explicit:** README says it works on IWD:EE | CC BY-NC-SA 3.0 | — | House-rule tweaks, including IWD:EE kit tomes | — |
| Spell Revisions (record only) | Gibberlings3 repo | v4.21 (2026-08-17) | GH-rel `https://github.com/Gibberlings3/SpellRevisions/releases/download/v4.21/lin-spell_rev-v4-21.tar.gz` (3 assets) | **gate:** some components admit iwdee | Consent clause (section 12) | Kitpack fork lists components to leave out with SR | Spell overhaul | spell-revisions-4.21-chriz.5 (Chriz fork, curated recipe) |

## 3. Kits

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| The Artisan's Kitpack (upstream) | Artemius_I / The Artisan / AionZ | release v4.81a (2024-05-05, no assets); master 85928f9 (2026-05-31) | tag-zip v4.81a, or branch (`https://github.com/TheArtisanBG/The-Artisan-s-Kitpack/archive/refs/heads/master.zip`) → commit pin | **implied:** main tp2 has no gate and ships an `iwdspells` folder (Pale Master); no IWD statement in the repo or on the website's kitpack page | none found (repo has no README; website kitpack page) | EEex needed by 8004, 10003, 10004 and 30001. `_npc` components are gated bgee/bg2ee/eet | Large kitpack | see fork |
| Kitpack – Chriz Balance Patch | Chrizhermann | v4.81a-dev.85928f9-chriz.1 (2026-09-29); master 0798dc4 | GH-rel `https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/v4.81a-dev.85928f9-chriz.1/ArtisansKitpack-Chriz-Balance-Patch-v4.81a-dev.85928f9-chriz.1.zip` | Same as upstream. Fork tweak 51010 is gated bgee/bg2ee/eet | README asks that any redistribution credit Artemius_I first and link to the current website | Fork README: with Spell Revisions, leave out 30001, 8101/8102, and npc 5102 and 10004 | Balance patch on the kitpack | artisans-kitpack-4.81a-dev.85928f9-chriz.1 |
| Bardic Wonders (upstream) | The Artisan | tags up to v2.9c (2024-01-31); no releases; master 4e72d97 (2026-05-31) | tag-zip v2.9c, or branch (`https://github.com/TheArtisanBG/Bardic-Wonders/archive/refs/heads/master.zip`) → commit pin | **explicit:** the website says the kits and gameplay additions work with IWD:EE. Component 2001 (items) is gated bgee/bg2ee/eet | none found (README is a title only; website page) | 1012 and 2009 need EEex. Website: install after mods that add or change armour | Bard kits and songs | see fork |
| Bardic Wonders – Chriz Balance Patch | Chrizhermann | v2.9c-balance.8 (2026-10-14); fork 8b454bf | GH-rel `https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/download/v2.9c-balance.8/Bardic-Wonders-v2.9c-balance.8.zip` | Same as upstream. 3010 needs EEex and `CBRSPVER.2DA` from chriz-bg-rebalance (bg2ee/eet only), so 3010 cannot install on IWD:EE | not checked separately | — | Balance patch | bardicwonders-v2.9c-balance.8 (curated recipe; components 1001–1012, 2002, 2007, …) |
| Faiths and Powers | UnearthedArcana (subtledoctor, Raduziel et al.) | 0.92 (2026-09-03) | tag-zip `https://github.com/UnearthedArcana/Faiths_and_Powers/archive/refs/tags/0.92.zip` (the release has no assets) | **gate:** many components admit iwdee; a few are `!iwdee` | none found (README has no copyright section) | README: install it as the very last kit mod, after all NPC mods | Deities, spheres, priest kits | — |
| Scales of Balance | subtledoctor | 7.0.1 (2026-09-23) | tag-zip | **gate:** includes iwdee | Contact the author before reuse; credit expected; if the author cannot be reached, free non-commercial use is allowed; no profit-making or commercial use without express permission | — | Rules/kit mod | — |
| Might & Guile | subtledoctor | 5.9.1 (2026-10-04) | tag-zip | **gate:** includes iwdee | Same terms as Scales of Balance | — | Rules/kit mod | — |
| Tome & Blood | subtledoctor | 1.4.1 (2026-09-03) | tag-zip | **gate:** includes iwdee | none found (README credits only) | — | Arcane kits and rules | — |
| Will to Power | subtledoctor | 2.14 (2026-09-04) | tag-zip | **gate:** includes iwdee | Same terms as Scales of Balance | — | Class/kit mod | — |
| 5E Spellcasting | subtledoctor | 2.7.2 (2026-09-03) | tag-zip | **unclear:** no `GAME_IS` gate in the tp2 scan; no IWD statement found | Contact the author before reuse; credit expected (README) | After spell-adding mods (SR, IWDification, SCS IWD spells) and after T&B and FnP; before M&G and FnP Multiclass | Semi-spontaneous casting | — |
| Deities of Faerûn (DoF) | Raduziel | 2.0 (2026-05-04; head "2.0 Alpha") | tag-zip (the release has no assets) | **explicit:** the licence names IWD:EE. **gate:** some components are iwdee-only | Proprietary licence, see section 12 | — | Deity kits | — |
| Chaos Sorcerer Kit | argent77 | v2.11 (2025-10-16) | GH-rel `https://github.com/Argent77/A7-ChaosSorcerer/releases/download/v2.11/A7-ChaosSorcerer-v2.11.iemod` | **gate:** includes iwdee | CC BY-SA 4.0 | — | Sorcerer kit | — |
| Improved Archer Kit | argent77 | v4.4 (2024-12-07) | GH-rel `https://github.com/Argent77/A7-ImprovedArcher/releases/download/v4.4/A7-ImprovedArcher-v4.4.iemod` | **gate:** includes iwdee | CC BY-SA 4.0 | — | Archer kit rework | — |
| Multiclass Kits | argent77 | v1.1 (2025-10-24) | GH-rel `https://github.com/Argent77/A7-MultiKits/releases/download/v1.1/A7-MultiKits-v1.1.iemod` | **gate:** includes iwdee | CC BY-NC-SA 4.0 | — | Kits for multiclass characters | — |
| The Workshop Kitpack | D2-mods | v5.29 (2025-07-29) | GH-rel `https://github.com/D2-mods/The-Workshop-Kitpack/releases/download/v5.29/the-workshop-kitpack-v5.29.iemod` | **gate:** includes iwdee | none found (README, tp2) | — | Kitpack | — |
| A Frosty Journey (IWDEE Kitpack) | GwendolyneFreddy | v2.92 (2020-11-04) | GH-rel; the v2.92 release holds 22 .rar assets, older versions included | IWD:EE-specific mod; no gate found | none found (readme has credits only) | — | IWD-themed kits | — |
| Geomantic Sorcerer | Caedwyr (G3) | v9 (2025-05-07) | GH-rel `https://github.com/Gibberlings3/Geomantic_Sorcerer/releases/download/v9/geomantic-sorcerer-kit-v9.iemod` | **gate:** includes iwdee | Consent clause, same wording as section 12 | Required by Tipps (Lava) | Sorcerer kit | — |
| ZS Shadowdancer Overhaul | szaumoor | 1.3.0a (2026-07-26) | GH-rel `https://github.com/szaumoor/ZS_ShadowdancerOverhaul/releases/download/1.3.0a/win-ZS_ShadowdancerOverhaul-1.3.0a.zip` | **gate:** includes iwdee | GPL-3.0 | — | Shadowdancer rework | — |
| Divine Remix | Gibberlings3 (CamDawg et al.) | v9_test_build_2 (2025-04-28) | GH-rel `https://github.com/Gibberlings3/Divine_Remix/releases/download/v9_test_build_2/the-divine-remix-v9_test_build_2.iemod` | **explicit:** readme lists IWD:EE. **gate:** most components; some exclude iwdee | G3 text | Some components check for Tweaks Anthology components (e.g. one requires cdtweaks 4050 to be absent) | Divine-caster rework | — |
| Of Paths and Ways | Lava | 1.9.2 (2026-05-18) | WPDM `of-paths-and-ways` (post 763) | **explicit:** tags BG1EE, BG2EE, IWDEE; page says EE-only | none found (page) | Page says it is not traified | 13 kits | — |
| Warlock | The Artisan | v2.51 (2024-02-05); master 2026-06-04 | tag-zip v2.51, or branch | **gate:** the tp2 has an iwdee branch | none found | — | Warlock | — |
| House Rule Tweaks | The Artisan | v2.5a (2024-01-08); master 2026-05-29 | tag-zip, or branch | **gate:** some components are iwdee | none found | — | Rule tweaks | — |
| Shadow Magic | The Artisan | tags up to v3.11; master 2026-05-31 | tag-zip, or branch | **unclear:** 3 components are gated bg2ee/eet; the rest have no gate; no statement | none found (repo has no README) | Detects EEex | Shadow-magic kits | — |
| Artisan's HLAs | The Artisan | no tags; main 2026-05-27 | branch | **unclear:** no gate, no statement | none found (README) | Needs EEex | HLA rework | — |
| Artisan's Shaman Tweaks | The Artisan | no tags; main 2024-07-17 | branch | **unclear** | none found (README) | — | Shaman tweaks | — |
| Two-Handed Fighter Kit | Arvendor | no tags; master 2022-10-08 | branch | **unclear:** no gate | none found (repo has no README) | — | Fighter kit | — |

## 4. Companions (NPCs)

Expansion coverage (HoW/TotL) appears in the Scope column wherever a source states it.

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| Minerva | The Artisan | release v1.1 (2021-11-03, no assets; tp2 VERSION says 1.0); master d3d4e57 (2026-04-26) | tag-zip v1.1, or branch → commit pin | **gate:** `iwdee` | none found (no README; artisans-corner.com page) | — | Gnome fighter/artificer, NG. Romance for any gender or race. New voiced soundset. Expansion coverage not stated | — |
| Karihi | The Artisan | v1.02 (2024-12-20, no assets); master 7478ae3 (2026-02-06, "traify") | tag-zip v1.02, or branch | **gate:** `iwdee` | none found | — | Fire genasi pyromancer, CN. Romance for a male human, elf or half-elf PC with CHA 10+. Expansion coverage not stated | — |
| Athanasios | The Artisan | tp2 VERSION 1.0; main 035784e (2025-04-11); no tags | branch → commit pin | **gate:** `iwdee` | none found | Optional fighter/cleric component | Halfling priest of Tempus, CG. Voiced soundset. No romance. Expansion coverage not stated | — |
| Guldruk | The Artisan | main 1522ddb (2025-04-11); no tags | branch | **gate:** `iwdee` | none found | — | Half-orc kensai, LG. No romance. Expansion coverage not stated | — |
| Nirik | The Artisan | main 2a1e9a9 (2025-04-11); no tags | branch | **gate:** `iwdee` | none found | — | Dwarf avenger, TN. No romance. Expansion coverage not stated | — |
| Cassia | Arvendor | readme v1.05 Beta (tp2 v1.05b); main 8f9af4c (2024-01-20) | branch | **gate:** `iwdee` | none found (readme) | Crossmod talks with Lava's quest mods. Optional patch lets healing spells affect tieflings (10 SPPR files) | Tiefling cleric of Helm. Small personal quest. Romance. Readme: reacts in the main campaign and in HoW | — |
| IWD NPC Project | Kulyok (Pocket Plane Group) | v9 (2022-09-23) | GH-rel `https://github.com/Pocket-Plane-Group/IWD_NPC/releases/download/v9/iwdnpc-v9.iemod` (also lin/mac/win zips) | **explicit:** readme says IWD:EE is supported since v5; classic IWD needs HoW. No tp2 gate | Consent clause (verbatim in section 12) | Dusky (Lava) installs after it | 5 voiced NPCs with romances. EE expansion coverage not stated | — |
| Bethany | Lava | 1.03 (2025-11-15), 3.58 MB | WPDM `bethany` (751) | **explicit:** page says EE-only, IWD:EE | none found (page, forum info thread) | — | Shaman-kit companion | — |
| Dendjelion | Lava | 2.1 (2025-12-11), 10.9 MB | WPDM `dendjelion` (286) | **explicit** | none found | — | Halfling blackguard. The mod also contains a second character, Noon | — |
| Dusky | Lava | 2.5 (2024-11-13), 5.57 MB | WPDM `dusky-iwdee-npc` (129) | **explicit** | none found | Install after Kulyok's IWD NPC Project; has talks with its NPCs | Companion. Romance (site tag). Mature content | — |
| Hommet | Lava | 1.2 (2026-02-01), 4.20 MB | WPDM `hommet` (441) | **explicit** | none found | — | Companion | — |
| Ina | Lava | 2.5 (2025-12-28), 4.91 MB | WPDM `ina` (288) | **explicit** | none found | — | Companion. Romance for a female PC | — |
| L'anna | Lava | 1.8 (2024-04-30), 13.60 MB | WPDM `lanna` (437) | **explicit** | none found | — | Companion. Romance (tag) | — |
| Oak-Maw | Lava | 3.3 (2024-09-06), 4.65 MB | WPDM `oak-maw` (290) | **explicit** | none found | — | Companion. Romance (tag) | — |
| Orra | Lava | 2.7 (2025-12-13), 3.29 MB | WPDM `orra` (389) | **explicit** | none found | — | Companion | — |
| Tipps | Lava | 2.4 (2025-12-09), 6.35 MB | WPDM `tipps` (292) | **explicit** | none found | Requires G3 Geomantic Sorcerer, which carries a consent clause | Companion. Romance for short-race PCs | — |
| T'viy | Lava | 2.4 (2024-09-06), 6.13 MB | WPDM `tviy` (309) | **explicit** | none found | — | Chaotic evil gnome companion | — |
| Urchin | Lava | 2.5 (2024-01-27), 35.67 MB | WPDM `urchin` (294) | **explicit** | none found | — | Companion | — |
| Vito | Lava | 1.1 (2026-01-14), 7.2 MB | WPDM `vito` (878) | **explicit** | none found | — | Companion. Non-physical romance; the PC needs CHA 13+ | — |
| Arszyk's Wonderful Decay | Lava | BETA1.4 (2025-10-25), 5.99 MB | WPDM `arszyk` (815) | **explicit** | none found | — | Companion plus quest (beta) | — |
| Xikâsz's Grove of Chaos | Lava | 2.1 (2026-08-05), 20.0 MB | WPDM `xikaszs-grove-of-chaos` (722) | **explicit** | none found | — | Quest plus companion | — |
| Afaaq, the Djinni Companion | argent77 | v2.11 (2024-12-07) | GH-rel `https://github.com/Argent77/DjinniCompanion/releases/download/v2.11/A7-DjinniCompanion-v2.11.iemod` | **explicit:** has an IWD:EE "light" version with its own IWD:EE text | CC BY-SA 4.0 | An optional component disables its IWD quest | Djinni companion | — |
| Turald | Sinaherib | v0.9 beta (September 2021) | forum `https://forums.beamdog.com/uploads/editor/86/t5bkcgdc4l0w.rar` (Beamdog thread 71342) | **explicit:** IWD:EE thread | none found (thread) | Crossmod with Kulyok's NPCs and Dusky | Companion. Main campaign only | — |

Expansion-area data, for verification only: the Artisan NPC repos contain area scripts named in the AR8xxx/AR9xxx ranges (Athanasios 8, Guldruk 8, Nirik 9, Karihi 1, Minerva 0). Cassia has 1 and Kulyok's mod has 43. Which campaign those area numbers belong to was not verified.

## 5. Companion integration

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| IWD Crossmod Banter Pack | The Artisan, Arvendor, Kulyok, Lava (Gibberlings3) | v5 (2024-12-26) | GH-rel `https://github.com/Gibberlings3/iwdcrossmod/releases/download/v5/icewind-dale-crossmod-banter-pack-v5.iemod` (4 assets) | IWD:EE mod | G3 text; Kulyok is a co-author | Install after all other NPC mods | Covers banters for Lava's Bethany, Dendjelion, Dusky, Hommet, Ina, L'anna, Noon, Oak-Maw, Orra, T'viy, Tipps, Urchin and Xikâsz; Kulyok's 5 NPCs; Athanasios, Guldruk, Karihi, Minerva, Nirik and Cassia. Also configurable banter timers, romance conflicts, and an optional Holvir–Dendjelion conflict | — |
| Crossmod (The Artisan's fork) | The Artisan | branch only; head 3ee39d0 (2024-12-25) | branch | — | — | Fork of the G3 repo | Last commit adds a Guldruk–Karihi banter, one day before G3 v5 | — |
| Banter Accelerator | argent77 | v1.6 (2024-07-27) | GH-rel `https://github.com/Argent77/A7-BanterAccelerator/releases/download/v1.6/A7-BanterAccelerator-v1.6.iemod` | **gate:** needs `banttimg.2da` in the game; the local IWD:EE index contains BANTTIMG. No IWD statement | CC BY-SA 4.0 | — | Changes banter frequency | — |

## 6. Quests / content

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| IWD Unfinished Business | Gibberlings3 (CamDawg et al.) | v13 (2025-02-12) | GH-rel `https://github.com/Gibberlings3/iwd_unfinished_business/releases/download/v13/unfinished-business-for-icewind-dale-v13.iemod` | **explicit:** readme says a handful of components work on IWD:EE (3); the rest is already part of IWD:EE | G3 text | Item Upgrade's ini puts it before ub_iwd | Restored content | — |
| HoWMod | The Artisan | main 7c87f9c (2024-12-25, "initial release") | branch | IWD-specific; gate not captured | none found (no README) | — | Adds content to the end of Heart of Winter when HoW is played as a continuation of the main campaign | — |
| IWD:EE Reborn | peteSmith1229 | v1.02 (2026-02-04) | GH-rel `https://github.com/peteSmith1229/Icewind-Dale-Enhanced-Edition-Reborn/releases/download/v1.02/IWEE_Reborn_v1.02.zip` (≈210 MB) | IWD:EE mod | GPL-2.0 (repo licence) | README says it uses third-party names, animations, icons, voices and music | A self-described lore-breaking reimagining for Heart of Fury mode: items, quests, NPCs | — |
| Below & Below Inn | Lava | 3.1 (2026-04-26), 13.01 MB | WPDM `below-below-inn` (369) | **explicit** | none found | — | Quest mod | — |
| Night of the Blinking Dead | Lava | 3.1 (2026-02-21), 10.24 MB | WPDM `night-of-the-blinking-dead` (376) | **explicit** | none found | — | Quest mod | — |
| Snowytoes Hamlet | Lava | 1.6 (2026-02-21), 16.8 MB | WPDM `snowytoes-hamlet` (557) | **explicit** | none found | — | Quest mod | — |
| Tale of Our Lady Dreamless | Lava | 2.7 (2026-02-21), 10.25 MB | WPDM `tale-of-our-lady-dreamless` (365) | **explicit** | none found | — | Quest mod | — |
| Terror of the Skineater | Lava | 2.6 (2026-04-26), 32.34 MB | WPDM `terror-of-the-skineater` (372) | **explicit** | none found | — | Quest mod | — |
| The Rediscovery of Kuldahar | Lava | 2.6.1 (2026-08-23), 37.50 MB | WPDM `the-rediscovery-of-kuldahar` (420) | **explicit** | none found | Replaces Kuldahar area files (are/tis/wed), so it is incompatible with mods that replace them. IWDGO must be installed before it. Needs a new game | Reworks the Kuldahar area, including night maps and tilesets | — |
| The War of Bloom and Rot | Lava | 1.3 (2025-12-14), 39.1 MB | WPDM `the-war-of-bloom-and-rot` (866) | **explicit** | none found | — | Quest mod (20 areas) | — |
| Will of the Wisps | Lava | 3.6 (2026-08-18), 49.30 MB | WPDM `will-of-the-wisps` (91) | **explicit:** tags BG1EE, BG2EE, EET, IWDEE | none found | — | Multi-game mod: NPC, quest, items, kits | — |
| (also: Arszyk, Xikâsz, Cassia) | | | | | | | Quest and companion mods, listed in section 4 | |

## 7. Items

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| Frostforged | The Artisan | main 1652f9d (2025-09-25); no tags or releases | branch → commit pin | IWD:EE-specific (component names and repo description); no gate | none found (no README; website) | — | 3 components: item rebalance (weak named items strengthened, the strongest toned down), new and restored items, a key ring. Website says work in progress | — |
| The Artisan's Armory | The Artisan | main 0f68637 (2026-05-28); no tags | branch | **unclear:** no gate, no README | none found | tp2: incompatible with Forgotten Armament. The speed/APR rework component needs EEex | New items; weapon-type rebalance; armour, shield and DEX AC revision; incremental attribute bonuses; item value options (80/60/40/20%) | — |
| FindIt-IWD | Lava | 2.6 (2023-03-03), 1.39 MB | WPDM `findit-iwd` (144) | **explicit** | none found | — | About 30 new items | — |
| ForgeIt-IWD | Lava | 1.9 (2024-09-06), 1.49 MB | WPDM `forgeit-iwd` (276) | **explicit** | none found | — | 8 craftable artifacts spread across the main game, HoW and TotL. Optional: strengthens one dragon-scale armour | — |
| Weasels! | Lava | 3.4 (2022-08-21), 1.62 MB | WPDM `weasels` (141) | **explicit:** tags include IWDEE | none found | — | Items and spells (multi-game) | — |
| D2 Weapon | D2-mods | v3.6 (2026-10-02) | GH-rel `https://github.com/D2-mods/D2-Weapon/releases/download/v3.6/d2-weapon-v3.6.iemod` | **explicit:** ini names IWD:EE. **gate** | CC BY-SA 3.0 Unported | — | 15 Souls-inspired weapons plus an optional merchant | — |
| The Big Metal Unit mod (D2-Mira) | D2-mods | v4.23 (2026-10-02) | GH-rel `https://github.com/D2-mods/D2-Mira/releases/download/v4.23/the-big-metal-unit-mod-v4.23.iemod` | **explicit:** README covers classic and EE BG1, BG2, IWD1 and IWD2 | none found (README, readme.txt) | ini `After`: EET, iwd2ee, iwdfixpack, iu_iwd, ub_iwd. `Before`: EET_end, iwd2ee compatibility patch | Item/tweak mod | — |
| IWD Bonus merchants | Deratiseur | v1 (2025-07-16) | GH-rel `https://github.com/Deratiseur/IWD_Bonus/releases/download/v1/Derats_IWD_Bonus-v1.iemod` (27 KB) | **gate:** `IWDEE` | none found (README) | — | Adds BG:EE's two bonus merchants, Deirdre and Joluv, to IWD:EE. The archive ships their item, store and creature files | — |
| More Style for Mages | Pecca | 1.57 (2021-01-10); no assets | tag-zip | **gate:** some components iwdee; the repo description names IWD:EE | none found (readme) | — | Mage-style extras, e.g. a Staff of Wizardry for BG2:EE and IWD:EE | — |
| Trollkiller's Item, Spell & Kitpack | DaerosTrollkiller | v0.1-Beta prerelease (2024-08-09; tp2 says v3.0.0-BETA) | GH-rel (prerelease) `k0_iskp-Beta1.zip` | **partial:** one items component admits iwdee; the rest are BG2/EET or BG | CC0-1.0 | — | Mostly SoD-to-BG2 item upgrades | — |

## 8. UI / visuals

| Mod | Author | Latest (date) | Fetch | IWD:EE | Terms | Deps / order / conflicts | Scope | CEBG |
|---|---|---|---|---|---|---|---|---|
| LeUI-IWDEE | lefreut; IWD:EE port by r-e-d | v2.0 (2022-08-11); master 137bbb9 (2024-04-27, fixes after v2.0) | GH-rel `https://github.com/r-e-d/LeUI-IWDEE/releases/download/v2.0/lefreuts-enhanced-ui-for-iwdee-2.0.zip` (also .iemod) | **gate:** `iwdee` plus UI.MENU. README pairs v2.0 with EE 2.6; no 2.7 statement | none found (README, ini) | README: install before EEUITweaks and before mods that patch the UI (SCS, T&B, M&G, Shadow Magic, DoF, FnP); not compatible with other UI overhauls such as DUI++. HGO 300/301/304 do not install with it; EEUITweaks 1080 excludes it | UI overhaul | — (CEBG uses the BG skins leui-4.9.1, leui-bg1ee and leui-sod) |
| Infinity UI++ | Pecca (repo Renegade0/InfinityUI) | v1.31 (2026-10-01); no assets | tag-zip `https://codeload.github.com/Renegade0/InfinityUI/zip/refs/tags/v1.31` | **gate:** `bgee bg2ee eet iwdee`; the tp2 has an IWD:EE branch | none found (repo has no README or LICENSE; tp2 only) | With EEex it copies EEex content and enables EEex's B3Scale module | UI overhaul | infinity-ui-1.28 (commit f755721, ≈256 MB zip) |
| EEUITweaks (r-e-d fork) | AncientCowboy (original), r-e-d | v4.0.7 (2024-09-15) | GH-rel `https://github.com/r-e-d/EEUITweaks/releases/download/v4.0.7/eeuitweaks-4.0.7.zip` (also .iemod) | **partial:** the predicates of about 10 components include iwdee (e.g. 1020, 1030, 1050, 1070, 1080, 1100, 2000, 2120, 2130); install conditions not evaluated. The original AncientCowboy v1.5 has no iwdee | not checked | Install after LeUI. 1080 excludes LeUI-IWDEE. HGO conflicts with 1010 | UI tweaks | — |
| Shader Pack: Lighting adjustments | argent77 | v3.2 (2024-12-07) | GH-rel `https://github.com/Argent77/A7-LightingPackEE/releases/download/v3.2/win-LightingPack-v3.2.zip` (also iemod, lin, mac) | **gate:** includes iwdee. Changelog records added IWD:EE support | CC BY-SA 4.0 | — | Lighting shaders | — |
| Icewind Dale Graphical Overhaul (IWDGO) | Weigo (Spellhold Studios) | v1.1 (2026-03-22; tag is `v1,1`, tp2 says v1.0); main 2026-09-05 | GH-rel `https://github.com/Spellhold-Studios/Icewind-Dale-Graphical-Overhaul/releases/download/v1%2C1/iwdgo-v1.1.zip` (340,327,596 bytes) or `.iemod` (337,539,399 bytes) | **gate:** iwdee, plus IWD-in-EET and IWD2 variants | none found (README copyright section lists only tool and game rights) | README: compatible with mods that do not overwrite are/wed/tis/pvrz/bmp. Install before The Rediscovery of Kuldahar. Separate Windows and Android components | Area graphics overhaul | — (CEBG uses BGGO 3.6, also by Spellhold Studios) |
| Artisan's Font Pack | The Artisan | main 57f7fc1 (2025-11-11) | branch | **unclear:** no gate | none found (README says the fonts are licensed for commercial use) | Patches BGEE.lua, M_STYLES.lua and UI.menu | Font replacement | — |
| Premade Party Adder | D2-mods | v1.5 (2026-05-29; tp2 says 1.6) | GH-rel `https://github.com/D2-mods/Premade-Party-Adder/releases/download/v1.5/premade-party-adder-v1.5.iemod` | **explicit and gate:** IWD:EE components | none found | — | Custom premade parties on the New Game screen. Also fixes an IWD:EE bug where imported characters get the wrong biography | — |
| (also: Bubb's Spell Menu, HGO) | | | | | | | See sections 1 and 2 | |

## 9. Rules / challenge, including stat rolls

| Item | Facts |
|---|---|
| No reload / ironman | Owner update: this is a stream rule only, and the installer needs no mechanism. No existing IWD:EE ironman mod was found earlier; this was not researched further. |
| Stat-roll limit (max 10 rerolls, 2 stored rolls) | **No existing mod found** that limits rerolls or stored rolls. |
| Native behaviour | IWD:EE character creation has one Store/Recall slot. The reroll/store/recall logic lives in the UI.MENU Lua (CHARGEN_ABILITIES). |
| Related existing components | **ToF point-buy** (80100–80104: 75/80/85/90 points, or a random total that can be rerolled). The first option carries the gate `bgee bg2ee eet iwdee` and `FORBID_COMPONENT` SCS 5900; all five options exclude cdtweaks 3210. **cdtweaks 3210 "Minimum Stats Cheat"** rerolls until a minimum is met. Its tp2 gate `NOT GAME_IS ~iwd2 pst pstee~` admits IWD:EE, but the readme games line excludes "IWD", which is a discrepancy. BG-EE-Point-Buy (2016) is BG:EE-only and archived. |
| UI interplay | LeUI-IWDEE copies its own `UI.menu`. Infinity UI++ also replaces the UI. A custom chargen limit would sit in the same file area. |
| Other IWD:EE-capable rule components | cdtweaks 2670 Limit Resting (EEex); cdtweaks 2510 Level-Lock Spell Scrolls; cdtweaks 3358 Create Interval Saves; Improved HoF Mode; SCS AI; ToF; randomrolls (RNG source); Achievements (journal entries). |

## 10. CEBG Challenge Mode: how it is built, and IWD:EE counterparts

**How it is built** (ref 5106b53)

- **Definition:** one `[[feature_groups]]` entry in `manifest/collection.toml`, with `id = "challenge-mode"` and title "Challenge Mode". Its description says one switch turns on every member, and each member stays switchable under its own mod.
- **Members:**
  - `chriz-bg-rebalance` 301: Emotion Courage and Emotion Hope become mutually exclusive.
  - `chriz-bg-rebalance` 110: EEex + SCS apex dragons, whose claws and bites can be lethal.
  - `chriz-bg-modpack` 235: Baeloth-related.
  - `chriz-sod-remix` 257: extra challenge at the SoD bridge; requires 256.
- **Variant:** choice group `potion-uniqueness`, option `chriz-bg-rebalance` 330. This is the strict setting: one dose of each temporary potion type, Potions of Genius included.
- **Consumers:** `app/src/screens/setup.ts` (setup screen) and `engine/tests/production_challenge_mode.rs` (test).
- **Rule inventory and outlook:** `docs/plans/2026-09-13-challenge-mode-outlook.md`. Its rules cover:
  - Exploits and farming: no exploits; no farming; no Balthazar; no Ravager.
  - Wands, traps and arrows: wands replaced by scrolls; traps and Skull Trap only after combat starts; Arrows of Dispelling; no explosive arrows.
  - Spells: one Wish per combat; Wish gives SR Improved Haste; at most one Time Stop memorised; Emotion spells do not stack; Baeloth spells.
  - Items and stores: stealing capped at 5 per shopkeeper; no Protection from Magic or Undead scrolls or potions.
  - Combat conduct: no off-screen casting before combat; no removing party members during combat; selected-fight exit locks.
  - Campaign-specific: the SoD filler exception; stronger dragons; SoD changes.
- **Gates:** every chriz-bg-rebalance component is gated `REQUIRE_PREDICATE (GAME_IS ~bg2ee eet~)` (v0.8.5, origin/main 38a39f6; 110 at line 218, 301 at line 438, 330 at line 1615). 235 and 257 are BG and SoD content. **None of the four members, nor the variant, can install on IWD:EE.**

**IWD:EE counterparts (facts only)**

- **The group mechanism itself** (manifest group, setup-screen switch, test) does not depend on the game. Its members would need components that install on IWD:EE, and the chriz layer has none today.
- **Rule targets that exist in IWD:EE:**
  - Emotion spells exist (the SCS readme mentions them), so a 301-type rule has a target.
  - Dragons exist, and SCS Smarter Dragons installs on IWD:EE, so a 110-type rule has a base; 110 itself needs EEex + SCS and is BG-gated.
  - Temporary potions exist, so a 330-type rule has a target.
- **BG-only targets with no IWD:EE counterpart by name:** Baeloth (235, Baeloth spells), the SoD bridge (257), the SoD filler exception, SoD changes, Balthazar and Ravager.
- **Rules that do not depend on particular game content:** no exploits, no farming, no off-screen pre-combat casting, no removing party members in combat, the stealing cap, selected-fight exit locks.
- **Not verified here:** whether IWD:EE has the specific spells and items some rules target (Wish, Time Stop, Skull Trap, Arrows of Dispelling, explosive arrows, Protection from Magic/Undead scrolls and potions), and the SR-specific Wish rule. A separate fight audit is running for the coordinator.
- **No reload:** a stream rule only (owner).

## 11. Mods and components that do NOT support IWD:EE

- **BuffBot** (Chrizhermann): v1.11.0 (2026-10-23) is the latest release, the main tp2 says v1.12.0, and CEBG pins 1.10.1. Components 0 and 1 are gated `REQUIRE_PREDICATE (GAME_IS ~bgee bg2ee eet~)`. The README names BG:EE, BG2:EE or EET. Licence MIT.
- **Chriz Item Randomiser 1.0.2 and Item Randomiser 8.1.1** (Chriz fork of Wisp's randomiser; CEBG `randomiser-8.1.1`): gates are BG-only. The upstream FredrikLindgren/randomiser (latest release v7, 2018) is also BG-only.
- **Dragonspear UI++** (CEBG `dragonspear-ui-22.01.3`): gated `bgee bg2ee eet`.
- **LeUI BG skins** (CEBG leui-4.9.1, leui-bg1ee, leui-sod): BG skins only. The IWD:EE port is LeUI-IWDEE.
- **EEUITweaks original** (AncientCowboy v1.5): no iwdee. The r-e-d fork is partial (section 8).
- **IWDEE-UI-BGEE** (AedrianRansom v1.9): puts IWD:EE's UI into BG games, so it has no IWD:EE target.
- **IWDification** (CEBG curated `iwdification-11`): the readme scopes it to bringing IWD elements into the Baldur's Gate series. IWD:EE is not in its compatibility list.
- **chriz-bg-rebalance** (all components bg2ee/eet), **chriz-bg-modpack 235**, **chriz-sod-remix 257**.
- **Bardic Wonders fork 3010**, **Kitpack fork 51010**, and the Kitpack `_npc` components.
- **Item Upgrade for IWD v7** (G3): `GAME_IS ~iwd how totlm iwd-in-bg2~`, i.e. classic IWD and IWD-in-BG2 only.
- **IWD Fixpack v11** (G3): `GAME_IS ~iwd how totlm~`, classic only.
- **1pp IWD Fixes v1.0** (Spellhold): classic TotLM only (.rar).
- **Erevain** (billbisco): classic only (2016, MIT).
- **BG-EE-Point-Buy:** BG:EE only (2016, archived, MIT).
- **lambda-dom/spell_revisions:** bgee, bg2ee and eet only (BSD-3).
- **Trollkiller's kitpack:** everything except one items component.
- **Talents of Faerûn 1500, 1510 and 1520:** the IWD spell and bard-song imports.
- **SCS components not available on IWD:EE:** IWD spell and bard-song imports, fine arrows and weapons, reputation, the Arrows of Dispelling reduction, bears, spiders, doppelgangers, and the BG encounter components.
- **Shadow Magic:** 3 components are gated bg2ee/eet.
- **Hidden Gameplay Options 17:** not offered on IWD:EE, because the game already has the option.
- **subtledoctor/IWD_EET_Integration 1.2.1:** some components need IWD1_EET/IWD2_EET (EET-based). The repo description mentions IWD:EE dual-class rules, and no `GAME_IS` gates were found, so its IWD:EE status is unclear.

## 12. Licence and compilation clauses found

**Verbatim (one quote):** Kulyok's IWD NPC Project readme says the mod "may not be sold, published, compiled or redistributed in any form without the consent" of its author. Sources: `iwdnpc/Readme-IWDNPC.txt` and https://pocket-plane-group.github.io/readmes/Readme-IWDNPC.txt.

**The same wording**, as recorded in the research notes, appears in:
- the Spell Revisions readme (Gibberlings3/SpellRevisions);
- the Geomantic Sorcerer readme (Gibberlings3/Geomantic_Sorcerer).

**Paraphrased (read the source for exact wording):**

| Mod | Clause |
|---|---|
| SCS; Talents of Faerûn | Asks that the mod, or any modified version, not be mirrored elsewhere, and asks for full credit when code is borrowed (readme copyright section). ToF adds that material in `hla_feat` is CC BY-NC-SA 3.0. |
| Deities of Faerûn | Proprietary licence (`LICENCE.md`). Personal, non-commercial use is granted. Without the author's written permission: no modifying or derivative works; no redistributing, re-uploading or mirroring outside the original distribution source; no incorporating, merging or bundling any part into another mod, tool or project; no reuse of its assets. No commercial use. The licence terminates on violation. |
| Enhanced Powergaming Scripts | README forbids mirroring, borrowing or altering its content unless the author explicitly permits it. |
| Scales of Balance, Might & Guile, Will to Power | Contact the author before using or adapting any part; credit is expected. If the author cannot be reached, free non-commercial use for IE players is allowed. Profit-making or commercial use needs express permission. |
| 5E Spellcasting | Contact the author before using or adapting any part; credit is expected. |
| G3 text | EE Fixpack, Tweaks Anthology, IWD Crossmod Banter Pack, IWD Unfinished Business, Item Upgrade for IWD, Divine Remix. |
| Kitpack (Chriz fork) README | Any redistribution should credit Artemius_I first and link to the current website. |
| IWDEE Tweaks and Fixes | Custom licence: use, copying, modification and redistribution of the mod's own code are allowed under conditions, including credit. |
| Lava (weaselmods forum thread `https://www.weaselmods.net/showthread.php?tid=3091`, 2026-10-07) | Lava does not make forks or alternative versions of the mods and does not allow others to do so. No licence clause was found on the weaselmods download pages or forum info threads. Inside the 19 pinned archives (checked 2026-10-26) the readmes hold credits, a version history and the usual trademark disclaimer; no archive has a licence file or a redistribution or compilation clause. The installer installs them unchanged from weaselmods. |

**Open licences:**
- MIT: BuffBot, Chriz Item Randomiser, EEex Remote Console, Erevain, BG-EE-Point-Buy.
- CC BY-SA 4.0: Hidden Gameplay Options, Lighting Pack, Banter Accelerator, Afaaq, Chaos Sorcerer, Improved Archer.
- CC BY-NC-SA 4.0: Achievements, Multiclass Kits.
- CC BY-SA 3.0: D2 Weapon.
- CC BY-NC-SA 3.0: S9 House Rules.
- GPL-3.0: ZSTweaks, ZS Shadowdancer, randomrolls.
- GPL-2.0: IWD:EE Reborn.
- BSD-3: lambda-dom spell_revisions.
- CC0: Trollkiller's kitpack.

**No clause found:** EEex, Bubb's Spell Menu, Artisan's Kitpack upstream, Bardic Wonders, all Artisan IWD NPCs, HoWMod, Frostforged, Armory, Warlock, House Rule Tweaks, Shadow Magic, Artisan's HLAs, Shaman Tweaks, Two-Handed Fighter, Cassia, Turald, Faiths and Powers, Tome & Blood, Workshop Kitpack, A Frosty Journey, Polymorph fixes, Better IWD Pregen, Premade Party Adder, Big Metal Unit, IWD Bonus merchants, klatu, Improved HoF, LeUI-IWDEE, Infinity UI++, IWDGO (copyright section only), More Style for Mages, and every Lava page.

## 13. Open questions

1. **Lava archives:** resolved 2026-10-26 for the 19 pinned packages: no licence or compilation clause inside (section 12). The 2026-10-07 statement concerns forks and alternative versions.
2. **WPDM links:** resolved 2026-10-26: the token-free `?wpdmdl=<id>` link works for automatic fetching (see "WPDM" under Keys). The Dropbox mirror was not needed and stays untested.
3. **No release assets** (would need commit or tag pins):
   - The Artisan's IWD NPCs: Minerva and Karihi have tags but no assets; Athanasios, Guldruk and Nirik have neither.
   - HoWMod, Frostforged, Armory, Font Pack, HLAs, Shaman Tweaks, Shadow Magic (tags only), Warlock and House Rule Tweaks (releases without assets).
   - Bardic Wonders upstream (tags only) and Kitpack upstream (release without assets).
   - Cassia, Improved HoF, Two-Handed Fighter.
   - Mods with releases but no assets (tag zips): every UnearthedArcana mod, DoF, Infinity UI++, Bubb's menu, EPS, S9, More Style for Mages.
   - Turald is a forum attachment.
4. **Game version 2.7.3.0:**
   - Documented: EEex (v2.7) and randomrolls (2.6 and 2.7).
   - Documented for 2.6 only: SCS (IWD:EE 2.6), EE Fixpack (names 2.6.6.0), ToF (2.6), LeUI-IWDEE (v2.0 for 2.6).
   - Older, with no version statement: Kulyok v9 (2022), A Frosty Journey (2020), More Style for Mages (2021), Improved HoF (2022), Enhanced Powergaming Scripts (2021).
5. **IWD:EE support not stated:**
   - Artisan's Kitpack upstream: implied only (no gate; `iwdspells` folder).
   - Shadow Magic, Artisan's HLAs, Shaman Tweaks, Two-Handed Fighter, 5E Spellcasting, Armory, Font Pack.
   - The r-e-d EEUITweaks components: predicate text only, install conditions not evaluated.
6. **Consent clauses** (the BG collection already holds 3 Kulyok mods pending consent):
   - Kulyok's IWD NPC Project carries the consent clause.
   - Kulyok co-authored the Crossmod Banter Pack, which carries the G3 text.
   - Tipps depends on Geomantic Sorcerer, which carries the same consent clause.
   - Spell Revisions carries it too; it is recorded only, and CEBG uses a Chriz fork.
7. **DoF:** the licence forbids bundling, mirroring outside the original source, and derivative works without written permission. Whether an installer that fetches from the original source falls under it is a question of interpretation.
8. **Third-party assets:**
   - IWD:EE Reborn's README says it uses third-party names, animations, icons, voices and music.
   - IWD Bonus merchants ships item, store and creature files for BG:EE's bonus merchants.
   - Where these files come from was not checked.
9. **HoW/TotL coverage:** not stated for most NPCs (Artisan NPCs, Lava NPCs, Afaaq). Area-script counts are in section 4, but the AR number-to-campaign mapping was not verified.
10. **Packaging oddities:** A Frosty Journey and 1pp ship as .rar. IWDGO's tag contains a comma (`v1,1`), is about 340 MB, and its tp2 says v1.0. randomrolls patches the game executable.
11. **cdtweaks 3210:** the readme games line and the tp2 gate disagree about IWD.
12. **SCS component list:** taken from the readme only; IWD:EE gates were not verified per component. A separate SCS coverage analysis is running for the coordinator.

## 14. Per-mod sources (read online; no archives downloaded)

**Core / engine**
- **EEex:** github.com/Bubb13/EEex (README support table, `EEex.tp2` line 67, releases).
- **EE Fixpack:** github.com/Gibberlings3/EE_Fixpack (readme, ini, tp2, releases).
- **Bubb's Spell Menu:** github.com/Bubb13/Bubbs-Spell-Menu-Extended (README; tp2 at tag v5.2).
- **EEex Remote Console:** github.com/Chrizhermann/eeex-remote-console (`setup-eeexremote.tp2`, LICENSE).

**Gameplay / AI / rules**
- **SCS:** github.com/Gibberlings3/SwordCoastStratagems (readme; `stratagems/setup-stratagems.tp2` 5900; `lang/english/setup.tra`).
- **Tweaks Anthology:** github.com/Gibberlings3/Tweaks-Anthology (readme component list; `cdtweaks/setup-cdtweaks.tp2` 3210/1361).
- **Hidden Gameplay Options:** github.com/Argent77/A7-HiddenGameplayOptions (readme, tp2, commits).
- **Talents of Faerûn:** github.com/Gibberlings3/TalentsOfFaerun (README.md, `dw_talents/doc/readme-dw_talents.html`, `dw_talents/dw_talents.tp2`, `lang/english/setup.tra`).
- **Improved HoF:** github.com/OlvynChuru/Improved-Heart-Of-Fury-Mode.
- **ZSTweaks:** github.com/szaumoor/ZSTweaks.
- **IWDEE Tweaks and Fixes:** github.com/Sauler89/IWDEE-Tweaks-and-Fixes (LICENSE).
- **Polymorph fixes:** github.com/D2-mods/Polymorph-fixes-for-IWDEE.
- **Better IWD Pregen:** github.com/D2-mods/Better-IWD-Pregen.
- **Enhanced Powergaming Scripts:** github.com/SarahG-579462/Enhanced-Powergaming-Scripts (README).
- **klatu:** github.com/The-Gate-Project/klatu-tweaks-and-fixes.
- **randomrolls:** github.com/drake127/randomrolls (README, tp2).
- **Achievements:** github.com/Argent77/A7-Achievements.
- **S9 House Rules:** github.com/mleduque/S9HouseRules.
- **Spell Revisions:** github.com/Gibberlings3/SpellRevisions; github.com/lambda-dom/spell_revisions.

**Kits**
- **Artisan's Kitpack:** github.com/TheArtisanBG/The-Artisan-s-Kitpack; github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch (origin/master: README, tp2); the artisans-corner.com kitpack page.
- **Bardic Wonders:** github.com/TheArtisanBG/Bardic-Wonders; github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch; the artisans-corner.com Bardic Wonders page.
- **UnearthedArcana mods:** github.com/UnearthedArcana/{Faiths_and_Powers, Scales_of_Balance, Might_and_Guile, TomeAndBlood, Will_to_Power, 5E_spellcasting} (READMEs and tp2s).
- **Deities of Faerûn:** github.com/Raduziel/Deities-Of-Faerun (`LICENCE.md`, tp2).
- **argent77 kits:** github.com/Argent77/{A7-ChaosSorcerer, A7-ImprovedArcher, A7-MultiKits}.
- **Workshop Kitpack:** github.com/D2-mods/The-Workshop-Kitpack.
- **A Frosty Journey:** github.com/GwendolyneFreddy/A_Frosty_Journey_IWDEE_Kitpack.
- **Geomantic Sorcerer:** github.com/Gibberlings3/Geomantic_Sorcerer.
- **ZS Shadowdancer:** github.com/szaumoor/ZS_ShadowdancerOverhaul.
- **Divine Remix:** github.com/Gibberlings3/Divine_Remix.
- **Other Artisan mods:** github.com/TheArtisanBG/{Warlock, House-Rule-Tweaks, Shadow-Magic, Artisan-HLAs, Artisans-Shaman-Tweaks}.
- **Two-Handed Fighter:** github.com/Arvendor/Two-Handed-Fighter-Kit-for-EE.

**Companions and integration**
- **Artisan NPCs:** github.com/TheArtisanBG/{MinervaNPC, KarihiNPC, IWD-Athanasios-NPC, IWD-Guldruk-NPC, IWD-Nirik-NPC, HoWMod} (tp2s, trees); artisans-corner.com NPC pages.
- **Cassia:** github.com/Arvendor/CassiaNPC (readme.txt, tp2).
- **Kulyok:** github.com/Pocket-Plane-Group/IWD_NPC and the Pocket Plane readme URL above.
- **Crossmod:** github.com/Gibberlings3/iwdcrossmod; the TheArtisanBG fork.
- **Afaaq:** github.com/Argent77/DjinniCompanion.
- **Banter Accelerator:** github.com/Argent77/A7-BanterAccelerator.
- **Turald:** Beamdog forums, discussion 71342.
- **Erevain:** github.com/billbisco/erevainmod.

**Lava**
- weaselmods.net download pages (`downloads.weaselmods.net/download/<slug>/`: version, dates, size, tags) and forum info threads (showthread.php?tid=…). Thread tid=3091 is the 2026-10-07 statement.

**Quests and items**
- **Unfinished Business:** github.com/Gibberlings3/iwd_unfinished_business.
- **IWD:EE Reborn:** github.com/peteSmith1229/Icewind-Dale-Enhanced-Edition-Reborn.
- **Item Upgrade:** github.com/Gibberlings3/itemupgrade_iwd.
- **IWD Fixpack:** github.com/Gibberlings3/iwdfixpack.
- **1pp fixes:** github.com/Spellhold-Studios/1pp-IWD-Fixes.
- **Frostforged:** github.com/TheArtisanBG/IWDEE-Frostforged and the artisans-corner.com page.
- **Armory:** github.com/TheArtisanBG/The-Artisan-s-Armory.
- **D2 Weapon, Big Metal Unit, Premade Party Adder:** github.com/D2-mods/{D2-Weapon, D2-Mira, Premade-Party-Adder}.
- **IWD Bonus merchants:** github.com/Deratiseur/IWD_Bonus.
- **Trollkiller's kitpack:** github.com/DaerosTrollkiller/Trollkillers-Item-Spell-Kitpack.
- **More Style for Mages:** github.com/Renegade0/msfm.

**UI / visuals**
- **LeUI:** github.com/r-e-d/{LeUI-IWDEE, LeUI, EEUITweaks}.
- **Infinity UI++ and Dragonspear UI++:** github.com/Renegade0/InfinityUI; github.com/anongit/DragonspearUI.
- **EEUITweaks original:** github.com/AncientCowboy/EEUITweaks.
- **IWDEE-UI-BGEE:** github.com/AedrianRansom/IWDEE-UI-BGEE.
- **Lighting Pack:** github.com/Argent77/A7-LightingPackEE.
- **Font Pack:** github.com/TheArtisanBG/Artisans-Font-Pack.
- **IWDGO:** github.com/Spellhold-Studios/Icewind-Dale-Graphical-Overhaul.

**CEBG and the chriz layer**
- `git show` / `git ls-tree` on `origin/claude/cebg-installer-fixes-20261014` (`manifest/*`, `recipes/curated-full-current/*`).
- chriz-bg-rebalance (origin/main: `setup-chriz-bg-rebalance.tp2`, english setup.tra).
- chriz-sod-rebalance (origin/master: `chriz-sod-remix/setup-chriz-sod-remix.tp2`).

## 15. Method and deviations

- **GitHub:** read through the API (`gh api`, with the token set per command and removed afterwards).
- **weaselmods:** pages read over plain HTTP. No archives were downloaded.
- **Deviation:** `git fetch --quiet origin` was run once in the main checkout of this repository. It updated only the remote-tracking refs; nothing was checked out or edited.
- **Local reads:** the chriz-layer repos were read with `git show`, `git ls-tree` and `git rev-parse` only. The local IWD:EE install was read only to check the exe version, the PATCH26.BIF presence, and the BANTTIMG name in `chitin.key`.

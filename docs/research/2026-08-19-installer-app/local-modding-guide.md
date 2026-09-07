# EET_MODDING_GUIDE.md + Mod Archive Summary

Historical research snapshot from 2026-08-19. Counts, ordering statements and
local-only acquisition assumptions below describe that source material and are
superseded where the current handover and curated recipe differ. Start with
[the current handover](../../handover.md) for supported installation/build work.

Source files: `C:\Games\Baldur's Gate II Enhanced Edition modded\EET_MODDING_GUIDE.md` (read), archive root `C:\Games\Baldurs Gate 1 and 2 mods\` (listed). Note: `README.md` in the game dir is a leftover SCS repo readme, not a download doc.

## 1. Install procedure / order philosophy

- **Reference install is COMPLETE**: 364 WeiDU.log entries, `EET_end` last. Launch via `InfinityLoader.exe` (EEex requirement).
- **Procedure**: extract mod folder into the BG2EE game dir → run setup from there. WeiDU.log is source of truth (never hand-edit). Uninstall only via setup, never file deletion. `Setup-Branwen.exe` is the WeiDU v24900 template binary — copy it as `Setup-<modname>.exe` for mods shipping without one.
- **Command pattern**: `./Setup-<mod>.exe --force-install-list <components> --language 0 --no-exit-pause`; interactive prompts handled via `echo <answer> | ./Setup-<mod>.exe ...`.
- **Ordering rules** (guide's "MUST KNOW"): EET_end absolute last; SCS near-last (only Randomiser + EET_end after); Spell Revisions core BEFORE CDTweaks/SCS; SR #60 (NPC Spellbooks) AFTER all NPC mods + kit assignments; kit mods BEFORE their NPC-assignment components; IEP Banters + Crossmod Banter Pack AFTER all NPC mods; Ascension BEFORE SCS; Artisan's Kitpack Tweaks AFTER other kit mods AND CDTweaks (split into pre/post-CDTweaks batches); Randomiser AFTER all item/store-modifying mods (after CDTweaks AND SCS).
- **18-phase canonical order** (for reinstalls): 1 Core infra (EEFIXPACK → EET → EEex → Bubb's Spell Menu → BGGO → Hidden Gameplay Options) → 2 Rogue Rebalancing → 3 NPC mods (Branwen, Evandra, Fade, Pai'Na, Bristlelick, Sarah, Aura, Xan, Yeslick, Ajantis BG2, Sirene BG2, Wings) → 4 UB BG2 → 5 Ascension → 6 SR core → 7 Kits (Artisan's Kitpack, Bardic Wonders, C0 Warlock) → 8 Kit NPC assignments → 9 SR #60 → 10 IWDification → 11 Banters (IEP, Crossmod) → 12 AK Tweaks batch 1 → 13 Soundsets (Bastila, Pathfinder, HQ Soundclips) → 14 CDTweaks (**user picks interactively**) → 15 AK Tweaks batch 2 → 16 SCS (**user picks interactively**) → 17 Randomiser → 18 EET_end. Guide lists exact component numbers per mod.
- **EET model**: BG1EE content merged into BG2EE; BG1EE dir was merge SOURCE only — never install there again. Pre-merge mods that came in via EET: DLC Merger, EEFIXPACK, BG1 UB (14 comps), BG1 NPC Project.
- **Adding mods later**: uninstall EET_end → install mod at correct position → reinstall EET_end. WeiDU uninstalls in reverse order — removing a mid-order mod force-uninstalls everything after it (usually game-breaking).

## 2. Mod download sources / URLs

**The guide documents ZERO download URLs.** No mod → URL table exists anywhere in it. The only "source" documented is local:

| Location | Role |
|---|---|
| `C:\Games\Baldurs Gate 1 and 2 mods\` | downloaded/archived mods (the only documented source) |
| `C:\Games\Baldur's Gate II Enhanced Edition modded\` | install target |
| `C:\Games\Baldur's Gate Enhanced Edition modded\` | BG1EE pre-merge source, read-only |

Provenance is only *inferable* from archive folder naming: GitHub clones/releases (`-master`/`-main` suffixes, git-describe like `ArtemiusI-Bardic-Wonders-v2.9c-103-gcfcd1c4`), Nexus Mods downloads (`Name-<modid>-<version>-<timestamp>`, e.g. `Garion's Portrait Pack - Baldurs Gate EE-36-1-1-1666307323`, `One Big Pack-48-1-0-1714837958`, `Racial Portrait Pack-25-1-0-1547369459`), and versioned release folders typical of G3/SHS/PPG zips (`ub-v28`, `iwdification-v11`, `win-A7-*` = Argent77 GitHub releases). One explicit provenance note: Morpheus562's Kitpack "author removed from GitHub" (dropped). **An installer app will need URLs sourced elsewhere — the guide cannot answer this.**

## 3. Per-mod notes / gotchas / interactive prompts

- **Interactive prompts (piped answers)**: Bardic Wonders Troubadour #1008 → `echo 1` (Garrick becomes Troubadour = Yes); C0 Warlock #0 → `echo 2` (Contingency UI = No, safer with EEex/Bubb's); Randomiser #1100 → `echo y` (leave items where they are = Yes).
- **Fully interactive (user picks)**: CDTweaks (55 comps) and SCS (76 comps) — copy mod folder + `Setup-Branwen.exe` renamed, run setup with no args.
- **SCS**: has Ascension-specific comps #6840/#6850/#8085 (require Ascension first); spell tweaks #2000 intentionally layer on SR.
- **Randomiser**: readme says install last/near-last; #1100 uses runtime scripts (order-tolerant) but #530/#9000/#10200/#10210 ARE order-sensitive. **Known defect in reference install**: Randomiser went in BEFORE CDTweaks/SCS — accepted as low-risk, but fresh reinstalls must use step 17.
- **Ajantis BG2**: installs with `--language 1` (others use 0).
- **Benign warning**: PARSE WARNING `DMWW_SLOT_186` in `C0AURA2J.DLG` during CDTweaks — EEex stat resolved at runtime.
- **Hard prohibitions**: Divine Remix (SR-incompatible), Wheels of Prophecy (buggy w/ Ascension+SCS), Sandrah Saga; never change Imoen's or Mazzy's kits; never delete override folder or use `git clean`; nothing installs after EET_end.
- **Dropped mods** (present in archive but deliberately NOT installed): Skitia NPCs, Brage's Redemption, Imoen 4 Ever, Angelo, Dvaradime, Drake, Juniper & the Stone Leech, Kitanya, all friendship mods, Transitions, Romance Expanded, Morpheus562's Kitpack, Solaufein Romance, Xan BG1 Voice, Crucible, Oblivion Guard Soundset.
- **Compatibility notes**: UB #5 + Pai'Na NPC coexist; UB #19 + Ascension #40 both fine (Ascension overwrites); AK race/class unlocks before CDTweaks equivalents (overlap OK); Garrick = Troubadour via Bardic Wonders.

## 4. Archive layout (`C:\Games\Baldurs Gate 1 and 2 mods\`)

- **Mods stored EXTRACTED, not zipped**: ~80 top-level folders, each an extracted release containing the WeiDU mod folder inside (e.g. `Fade-v5.6\fade\...`, `stratagems-35.21\stratagems\...`). `find` depth ≤2 found **zero** .zip/.7z/.rar/.iemod.
- **4 self-extracting .exe installers** at root: `cdtweaks-v18.exe`, `eefixpack-Alpha_3.exe`, `rr-v492.exe`, `stratagems-35.21.exe` (kept alongside their extracted folders; RR exists only as the .exe).
- **Superset of the install**: contains all dropped mods (SkitiaNPCs-1.14, imoen-4-ever, all `*-friendship-*`, transitions-v2.4, Romance_Expanded, Drake-1.7a, Dvaradime-v1.6, angelo-v9, JuniperAndTheStoneLeech, KitanyaSoAv6-41, brages-redemption) plus never-mentioned extras (ItemRevisions ×2 variants, InfinityUI-1.17, eeuitweaks, lefreuts-enhanced-ui, enhanced-powergaming-scripts, RemasteredSpellIcons, Portraits-Portraits-Everywhere, House-Rule-Tweaks). Some dupes (`Aura_BG1-master` + `Aura_BG1_BG2_EET-main`; `SwordCoastStratagems` + `stratagems-35.21`; Sirene BG1 + BG2).
- **Non-mod assets/tooling**: `BG_portraits\`, `soundsets\`, `bastilla sound\`, `bg1npcmusic\`, portrait packs; `BigWorldSetup-Windows\`, `NearInfinity-portable-win-2.4.2025.0611\`.
- **`InstallSequence.csv`** at root: semicolon-delimited `modkey:component;label` list (~444 rows) — an EARLIER planned selection that still includes later-dropped mods (Imoen4Ever, SkitiaNPCs, Transitions, friendships, Brage, Drake, Dvaradime, Romance Expanded, EEUITweaks, lefreut's UI) and differs from the final install (e.g. Fade Shadowdancer kit #2, Aura comps). **Stale — the guide's order + reference WeiDU.log override it**, but it's useful as a component-label dictionary.

**Bottom line for the installer app**: mod payloads come from the local extracted archive (copy folder → game dir → run renamed WeiDU template); no download URLs are documented anywhere in the guide — URL provenance must be reconstructed from folder-name conventions or external research.

# Pin list for the 2.7.3.0 stack

The pins are the recipe files: each installer in `mods/*.toml` names an archive in
`artifacts/*.toml` with its exact URL, version, size and SHA-256. The tables below are
generated from those files, and `engine/tests/pin_list.rs` fails when they drift. After a
pin change, regenerate them with
`CEBG_UPDATE_PIN_LIST=1 cargo test -p chriz-bg-engine --test pin_list`. Archive files that
no installer references, such as older versions, are not listed.

`manifest/mod-sources.tsv` is captured research data from the reference install (August
2026), not a pin list. Open source and release work is indexed in
[the to-do list](curation/components/FOLLOW_UPS.md).

<!-- BEGIN GENERATED PINS -->
### Shipped recipe: `recipes/curated-full-current/`

68 archives for 71 installers.

| Archive | Version | Used by | SHA-256 | Source |
|---|---|---|---|---|
| Ascalon's Questpack | 7.0 | `ac-quest` | `87c2feb30c89…` | [release asset](https://github.com/Gitjas/Ascalons_Questpack/releases/download/v7.0/ascalons-questpack-7.0.iemod) |
| The Artisan's Kitpack - Chriz Balance Patch | 4.81a-dev.85928f9-chriz.4 | `artisanskitpack`, `artisanskitpack-npc`, `artisanskitpack-tweak` | `8d08fc2be48c…` | [release asset](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/v4.81a-dev.85928f9-chriz.4/ArtisansKitpack-Chriz-Balance-Patch-v4.81a-dev.85928f9-chriz.4.zip) |
| Ascension | 2.1.0 | `ascension` | `bd0ee14b3770…` | [release asset](https://github.com/Gibberlings3/Ascension/releases/download/2.1.0/ascension-2.1.0.zip) |
| Bardic Wonders — Christopher's balance fork | 2.9c-balance.8 | `bardicwonders` | `fa99ef91f39d…` | [release asset](https://github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/releases/download/v2.9c-balance.8/Bardic-Wonders-v2.9c-balance.8.zip) |
| The BG1 NPC Project | 32 | `bg1npc` | `8e34f397e960…` | [release asset](https://github.com/Gibberlings3/BG1NPC/releases/download/v32/the-bg1npc-project-v32.iemod) |
| BG1 Unfinished Business | 17.1 | `bg1ub` | `5a525eb37f68…` | [release asset](https://github.com/Pocket-Plane-Group/bg1ub/releases/download/v17.1/bg1-unfinished-business-v17.1.iemod) |
| Baldur's Gate Graphical Overhaul | 3.6 | `bggo` | `59e01a3dcf48…` | [release asset](https://github.com/Spellhold-Studios/Baldurs-Gate-Graphical-Overhaul/releases/download/v3.6/baldurs-gate-graphical-overhaul-v3.6.zip) |
| Black Hearts | 2.05 | `blackhearts` | `a1a6257a2c21…` | [release asset](https://github.com/A-Wizard-Did-It/Black-Hearts/releases/download/v2.05/BlackHearts-v2.05.iemod) |
| Branwen for BGII | 8 | `branwen` | `f82c93e67f19…` | [release asset](https://github.com/Pocket-Plane-Group/Branwen_for_BGII/releases/download/v8/branwen-v8pre.zip) |
| Bubb's Spell Menu Extended | 5.2 | `bubb-spell-menu` | `ac5380e99a53…` | [tag archive](https://codeload.github.com/Bubb13/Bubbs-Spell-Menu-Extended/zip/refs/tags/v5.2) |
| BuffBot | 1.11.0 | `buffbot` | `c147919db1e6…` | [release asset](https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v1.11.0/buffbot-v1.11.0.zip) |
| The Tweaks Anthology | 7649ced6cd25865874d787ec1a9abbc67b068729 | `cdtweaks` | `117cd79cb518…` | [commit archive](https://codeload.github.com/Gibberlings3/Tweaks-Anthology/zip/7649ced6cd25865874d787ec1a9abbc67b068729) |
| chriz-bg-modpack | 0.2.0-alpha.14 | `chriz-bg-modpack` | `45c767b21342…` | [release asset](https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/v0.2.0-alpha.14/chriz-bg-modpack-v0.2.0-alpha.14.zip) |
| CHRIZ-BG-REBALANCE | 0.8.5 | `chriz-bg-rebalance` | `48e12ed83653…` | [release asset](https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.5/chriz-bg-rebalance-v0.8.5-windows.zip) |
| CHRIZ-SOD-REMIX | 0.6.15 | `chriz-sod-remix` | `3a295832f9af…` | [release asset](https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.15/chriz-sod-remix-v0.6.15.zip) |
| Crossmod Banter Pack for Baldur's Gate II | 30 | `crossmodbg2` | `6b1555d1cf6f…` | [release asset](https://github.com/Gibberlings3/Crossmod_Banter_Pack_for_Baldurs_Gate_II/releases/download/v30/crossmod-banter-pack-for-baldurs-gate-ii-v30.iemod) |
| DLC Merger | 2.1 | `dlcmerger` | `ea7584fd7285…` | [release asset](https://github.com/Argent77/A7-DlcMerger/releases/download/v2.1/DlcMerger-v2.1.iemod) |
| Dragonspear UI++ (anongit continuation) | 22.01.3 | `dragonspear-ui` | `81bf0403bc5f…` | [commit archive](https://codeload.github.com/anongit/DragonspearUI/zip/d86bdddd0d74ebadc9ebcd40351021d6d1dadbc0) |
| EEex | 1.3.1 | `eeex` | `9c8104bd63fe…` | [release asset](https://github.com/Bubb13/EEex/releases/download/v1.3.1/eeex-v1.3.1.zip) |
| EEex Remote Console | 0.2.1 | `eeexremote` | `f417cb993fff…` | [commit archive](https://codeload.github.com/Chrizhermann/eeex-remote-console/zip/3d98990766abe9ed1024c79ff43d609101f37836) |
| Enhanced Edition Fixpack | Beta 2 | `eefixpack` | `7a57e43bd6c3…` | [release asset](https://github.com/Gibberlings3/EE_Fixpack/releases/download/Beta_2/enhanced-edition-fixpack-beta-2.iemod) |
| Enhanced Edition Trilogy | 74e91d72bca5d073fa11c1d088b90d7ff0c7105d | `eet`, `eet-end`, `eet-gui` | `9834a53322b7…` | [commit archive](https://codeload.github.com/Gibberlings3/EET/zip/74e91d72bca5d073fa11c1d088b90d7ff0c7105d) |
| Evandra NPC | 2.2 | `evandra` | `21724b6d4679…` | [manual download](https://www.gibberlings3.net/files/file/1008-evandra/?do=download) |
| Fade NPC | 5.6 | `fade` | `6d3cd8d528cf…` | [release asset](https://github.com/Spellhold-Studios/Fade-NPC/releases/download/v5.6/Fade-v5.6.zip) |
| Hidden Gameplay Options | 5.2 | `hidden-gameplay-options` | `a7b173f811b1…` | [release asset](https://github.com/Argent77/A7-HiddenGameplayOptions/releases/download/v5.2/win-A7-HiddenGameplayOptions-v5.2.zip) |
| HQ SoundClips BG2EE | 1.3 | `hq-soundclips-bg2ee` | `712af8ab2104…` | [release asset](https://github.com/Argent77/HQ-SoundClips-BG2EE/releases/download/v1.3/win-A7-HQ-SoundClips-BG2EE-v1.3.zip) |
| IEP Extended Banters | 5.9 | `iepbanters` | `a79b06ec3b70…` | [release asset](https://github.com/Spellhold-Studios/IEP-Extended-Banters/releases/download/v5.9/iep-extended-banters-v5.9.iemod) |
| Infinity UI++ | 1.28 | `infinity-ui` | `4bb0b81414b4…` | [commit archive](https://codeload.github.com/Renegade0/InfinityUI/zip/f755721ef88f2370f35ee8d342abacf25b48e200) |
| IWDification | 11 | `iwdification` | `4af9d463d3b4…` | [release asset](https://github.com/Gibberlings3/iwdification/releases/download/v11/iwdification-v11.iemod) |
| Klatu Tweaks and Fixes | 1.7.4 | `klatu` | `7393116098ee…` | [release asset](https://github.com/The-Gate-Project/klatu-tweaks-and-fixes/releases/download/Version-1.7.4/klatu-tweaks-and-fixes-1.7.4.iemod) |
| Lefreut's Enhanced UI: BG2 style | 4.9.1 | `leui` | `93f78e68f371…` | [release asset](https://github.com/r-e-d/LeUI/releases/download/v4.9.1/lefreuts-enhanced-ui-bg2ee-skin-4.9.1.iemod) |
| Lefreut's Enhanced UI: BG1 style | 4.9.1 | `leui-bg1ee` | `d22adb826f6d…` | [release asset](https://github.com/r-e-d/LeUI-BG1EE/releases/download/v4.9.1/lefreuts-enhanced-ui-bg1ee-skin-4.9.1.iemod) |
| Lefreut's Enhanced UI: SoD style | 4.9.1 | `leui-sod` | `09de7721be81…` | [release asset](https://github.com/r-e-d/LeUI-SoD/releases/download/v4.9.1/lefreuts-enhanced-ui-sod-skin-4.9.1.iemod) |
| Northern Tales of the Sword Coast | f0ef95323630bc38df7dd4c969a2b48a71ebec82 | `ntotsc` | `b507d4b83367…` | [commit archive](https://codeload.github.com/Spellhold-Studios/NTotSC/zip/f0ef95323630bc38df7dd4c969a2b48a71ebec82) |
| Pai'Na NPC | 1.9 | `paina` | `8b448170062e…` | [tag archive](https://codeload.github.com/TheArtisanBG/Pai-Na-NPC-mod-for-BG2-EE/zip/refs/tags/v1.9) |
| Portraits Portraits Everywhere (DaftHunk fork) | 417226ea6642b8c682b2d049d0ec6e86f38b7f47 | `ppe` | `fc250193d9cc…` | [commit archive](https://codeload.github.com/DaftHunk/Portraits-Portraits-Everywhere/zip/417226ea6642b8c682b2d049d0ec6e86f38b7f47) |
| Item Randomiser | 8.1.1 | `randomiser` | `b5d95e801269…` | [release asset](https://github.com/Chrizhermann/chriz-item-randomiser/releases/download/v8.1.1/randomiser-v8.1.1.zip) |
| Rogue Rebalancing | 4.92 | `rr` | `692297f6da8f…` | [release asset](https://github.com/Spellhold-Studios/Rogue-Rebalancing/releases/download/v4.92/rr-v492.zip) |
| Safana in Amn | 05 | `safana` | `29672c878cfb…` | [tag archive](https://github.com/RoxanneSHS/SafanaBG2/archive/refs/tags/v05.zip) |
| Sarah NPC Romance Mod for BG2:ToB | 8 | `sarahtob` | `061b1da90d8c…` | [release asset](https://github.com/Gibberlings3/Sarah/releases/download/v8/sarah-npc-romance-mod-for-bg2-tob-v8.iemod) |
| Sirene NPC for BG2:EE | 00beda909a4a791a35016069b4ebb77f27279723 | `sirene-bg2` | `1a6d517b7b02…` | [commit archive](https://codeload.github.com/TheArtisanBG/Sirene-NPC-for-BG2-EE/zip/00beda909a4a791a35016069b4ebb77f27279723) |
| Spell Revisions | 4.21-chriz.7 | `spell-rev` | `613fca7d637b…` | [release asset](https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/download/v4.21-chriz.7/spell_rev-v4-21-chriz-7.zip) |
| Spell Revisions / Rogue Rebalancing compatibility | 4.21-chriz.7 | `srcb-rr-compat` | `705289651b99…` | [release asset](https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/download/v4.21-chriz.7/SRCB_RR_COMPAT-v4-21-chriz-7.zip) |
| Sword Coast Stratagems | 35.21 | `stratagems` | `dd6b0dbe3b9f…` | [release asset](https://github.com/Gibberlings3/SwordCoastStratagems/releases/download/v35.21/lin-stratagems-35.21.zip) |
| Thalantyr Item Upgrade | 4.5.1 | `thalan` | `ac5a2dff1feb…` | [release asset](https://github.com/CrevsDaak/thalan/releases/download/v4.5.1/thalantyr-item-upgrade-v4.5.1.iemod) |
| Unfinished Business for BG2 | 28 | `ub` | `1d5aff32d760…` | [release asset](https://github.com/Pocket-Plane-Group/UnfinishedBusiness/releases/download/v28/ub-v28.zip) |
| Aerie player voicepack | 1.0.1 | `voicepack-aerie` | `04ce446df621…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/aerie-player-voicepack-v1.0.1.zip) |
| Ajantis player voicepack | 2.0.0 | `voicepack-ajantis` | `7d2150ba9202…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/ajantis-player-voicepack-v2.0.0.zip) |
| Amelyssan player voicepack | 1.0.1 | `voicepack-amelyssan` | `f67c4e4b87cc…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/amelyssan-player-voicepack-v1.0.1.zip) |
| Edwin (BG1) player voicepack | 1.0.1 | `voicepack-edwin-bg1` | `6c02995b38ed…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/edwin-bg1-player-voicepack-v1.0.1.zip) |
| Edwin (BG2) player voicepack | 1.0.1 | `voicepack-edwin-bg2` | `d32e6f5577b9…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/edwin-bg2-player-voicepack-v1.0.1.zip) |
| Garithos player voicepack | 1.1.0 | `voicepack-garithos` | `24abfd30cd8c…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/garithos-player-voicepack-v1.1.0.zip) |
| Gromnir Il-Khan player voicepack | 1.0.1 | `voicepack-gromnir-il-khan` | `7aaad0379a6c…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/gromnir-il-khan-player-voicepack-v1.0.1.zip) |
| Imoen (BG1) player voicepack | 1.0.1 | `voicepack-imoen-bg1` | `cda87d6b56a1…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/imoen-bg1-player-voicepack-v1.0.1.zip) |
| Imoen (BG2) player voicepack | 1.0.1 | `voicepack-imoen-bg2` | `043d6216b4cf…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/imoen-bg2-player-voicepack-v1.0.1.zip) |
| Jaheira (BG1) player voicepack | 1.0.1 | `voicepack-jaheira-bg1` | `d8a3bd683fc1…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/jaheira-bg1-player-voicepack-v1.0.1.zip) |
| Jaheira (BG2) player voicepack | 1.0.1 | `voicepack-jaheira-bg2` | `5efe01db1594…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/jaheira-bg2-player-voicepack-v1.0.1.zip) |
| Neera player voicepack | 1.0.1 | `voicepack-neera` | `f2adbfd775f8…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/neera-player-voicepack-v1.0.1.zip) |
| Safana (BG1) player voicepack | 1.0.1 | `voicepack-safana-bg1` | `e5b3e7fa6946…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/safana-bg1-player-voicepack-v1.0.1.zip) |
| Safana (SoD) player voicepack | 1.0.1 | `voicepack-safana-sod` | `edd486e85c95…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/safana-sod-player-voicepack-v1.0.1.zip) |
| Sarevok player voicepack | 1.0.1 | `voicepack-sarevok` | `e105f9b296d0…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/sarevok-player-voicepack-v1.0.1.zip) |
| Viconia (BG1) player voicepack | 1.0.0 | `voicepack-viconia-bg1` | `bd56af0ef69e…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/viconia-bg1-player-voicepack-v1.0.0.zip) |
| Viconia (BG2) player voicepack | 1.0.0 | `voicepack-viconia-bg2` | `dde9cfca2692…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/viconia-bg2-player-voicepack-v1.0.0.zip) |
| Yeslick player voicepack | 1.0.1 | `voicepack-yeslick` | `c3c1213cc4ec…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/yeslick-player-voicepack-v1.0.1.zip) |
| WeiDU 249 for Windows x64 | 249.00 | WeiDU for every installer | `b156910cbec6…` | [release asset](https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip) |
| Xan NPC for BGII | 19 | `xan` | `47cd6622678f…` | [release asset](https://github.com/Pocket-Plane-Group/Xan_for_BGII/releases/download/v19/Xan_v19.zip) |
| Yeslick NPC | 5.0 | `yeslicknpc` | `d9b403a6feb8…` | [release asset](https://github.com/Spellhold-Studios/Yeslick-NPC/releases/download/v5.0/yeslick-npc-v5.0.zip) |
| Icewind Dale Voice Pack for EE 2.6+ | 1.2.0 | `zg-iwd-voices` | `7677e0d8fdef…` | [release asset](https://github.com/GraionDilach/Icewind-Dale-Voice-Pack-For-EE-2.6/releases/download/version-1.2.0/zg-iwd-voices-1.2.0.iemod) |

### Base recipe: `manifest/`

62 archives for 65 installers.

| Archive | Version | Used by | SHA-256 | Source |
|---|---|---|---|---|
| Ascalon's Questpack | 7.0 | `ac-quest` | `87c2feb30c89…` | [release asset](https://github.com/Gitjas/Ascalons_Questpack/releases/download/v7.0/ascalons-questpack-7.0.iemod) |
| The Artisan's Kitpack - Chriz Balance Patch | 4.81a-dev.85928f9-chriz.4 | `artisanskitpack`, `artisanskitpack-npc`, `artisanskitpack-tweak` | `8d08fc2be48c…` | [release asset](https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/v4.81a-dev.85928f9-chriz.4/ArtisansKitpack-Chriz-Balance-Patch-v4.81a-dev.85928f9-chriz.4.zip) |
| Ascension | 2.1.0 | `ascension` | `bd0ee14b3770…` | [release asset](https://github.com/Gibberlings3/Ascension/releases/download/2.1.0/ascension-2.1.0.zip) |
| The BG1 NPC Project | 32 | `bg1npc` | `8e34f397e960…` | [release asset](https://github.com/Gibberlings3/BG1NPC/releases/download/v32/the-bg1npc-project-v32.iemod) |
| BG1 Unfinished Business | 17.1 | `bg1ub` | `5a525eb37f68…` | [release asset](https://github.com/Pocket-Plane-Group/bg1ub/releases/download/v17.1/bg1-unfinished-business-v17.1.iemod) |
| Baldur's Gate Graphical Overhaul | 3.6 | `bggo` | `59e01a3dcf48…` | [release asset](https://github.com/Spellhold-Studios/Baldurs-Gate-Graphical-Overhaul/releases/download/v3.6/baldurs-gate-graphical-overhaul-v3.6.zip) |
| Black Hearts | 2.05 | `blackhearts` | `a1a6257a2c21…` | [release asset](https://github.com/A-Wizard-Did-It/Black-Hearts/releases/download/v2.05/BlackHearts-v2.05.iemod) |
| Bubb's Spell Menu Extended | 5.2 | `bubb-spell-menu` | `ac5380e99a53…` | [tag archive](https://codeload.github.com/Bubb13/Bubbs-Spell-Menu-Extended/zip/refs/tags/v5.2) |
| BuffBot | 1.11.0 | `buffbot` | `c147919db1e6…` | [release asset](https://github.com/Chrizhermann/bg-eeex-buffbot/releases/download/v1.11.0/buffbot-v1.11.0.zip) |
| chriz-bg-modpack | 0.2.0-alpha.14 | `chriz-bg-modpack` | `45c767b21342…` | [release asset](https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/v0.2.0-alpha.14/chriz-bg-modpack-v0.2.0-alpha.14.zip) |
| CHRIZ-BG-REBALANCE | 0.8.5 | `chriz-bg-rebalance` | `48e12ed83653…` | [release asset](https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.5/chriz-bg-rebalance-v0.8.5-windows.zip) |
| CHRIZ-SOD-REMIX | 0.6.15 | `chriz-sod-remix` | `3a295832f9af…` | [release asset](https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.15/chriz-sod-remix-v0.6.15.zip) |
| Crossmod Banter Pack for Baldur's Gate II | 30 | `crossmodbg2` | `6b1555d1cf6f…` | [release asset](https://github.com/Gibberlings3/Crossmod_Banter_Pack_for_Baldurs_Gate_II/releases/download/v30/crossmod-banter-pack-for-baldurs-gate-ii-v30.iemod) |
| DLC Merger | 2.1 | `dlcmerger` | `ea7584fd7285…` | [release asset](https://github.com/Argent77/A7-DlcMerger/releases/download/v2.1/DlcMerger-v2.1.iemod) |
| Dragonspear UI++ (anongit continuation) | 22.01.3 | `dragonspear-ui` | `81bf0403bc5f…` | [commit archive](https://codeload.github.com/anongit/DragonspearUI/zip/d86bdddd0d74ebadc9ebcd40351021d6d1dadbc0) |
| EEex | 1.3.1 | `eeex` | `9c8104bd63fe…` | [release asset](https://github.com/Bubb13/EEex/releases/download/v1.3.1/eeex-v1.3.1.zip) |
| EEex Remote Console | 0.2.1 | `eeexremote` | `f417cb993fff…` | [commit archive](https://codeload.github.com/Chrizhermann/eeex-remote-console/zip/3d98990766abe9ed1024c79ff43d609101f37836) |
| Enhanced Edition Fixpack | Beta 2 | `eefixpack` | `7a57e43bd6c3…` | [release asset](https://github.com/Gibberlings3/EE_Fixpack/releases/download/Beta_2/enhanced-edition-fixpack-beta-2.iemod) |
| Enhanced Edition Trilogy | 74e91d72bca5d073fa11c1d088b90d7ff0c7105d | `eet`, `eet-end`, `eet-gui` | `9834a53322b7…` | [commit archive](https://codeload.github.com/Gibberlings3/EET/zip/74e91d72bca5d073fa11c1d088b90d7ff0c7105d) |
| Fade NPC | 5.6 | `fade` | `6d3cd8d528cf…` | [release asset](https://github.com/Spellhold-Studios/Fade-NPC/releases/download/v5.6/Fade-v5.6.zip) |
| Hidden Gameplay Options | 5.2 | `hidden-gameplay-options` | `a7b173f811b1…` | [release asset](https://github.com/Argent77/A7-HiddenGameplayOptions/releases/download/v5.2/win-A7-HiddenGameplayOptions-v5.2.zip) |
| HQ SoundClips BG2EE | 1.3 | `hq-soundclips-bg2ee` | `712af8ab2104…` | [release asset](https://github.com/Argent77/HQ-SoundClips-BG2EE/releases/download/v1.3/win-A7-HQ-SoundClips-BG2EE-v1.3.zip) |
| IEP Extended Banters | 5.9 | `iepbanters` | `a79b06ec3b70…` | [release asset](https://github.com/Spellhold-Studios/IEP-Extended-Banters/releases/download/v5.9/iep-extended-banters-v5.9.iemod) |
| Infinity UI++ | 1.28 | `infinity-ui` | `4bb0b81414b4…` | [commit archive](https://codeload.github.com/Renegade0/InfinityUI/zip/f755721ef88f2370f35ee8d342abacf25b48e200) |
| Lefreut's Enhanced UI: BG2 style | 4.9.1 | `leui` | `93f78e68f371…` | [release asset](https://github.com/r-e-d/LeUI/releases/download/v4.9.1/lefreuts-enhanced-ui-bg2ee-skin-4.9.1.iemod) |
| Lefreut's Enhanced UI: BG1 style | 4.9.1 | `leui-bg1ee` | `d22adb826f6d…` | [release asset](https://github.com/r-e-d/LeUI-BG1EE/releases/download/v4.9.1/lefreuts-enhanced-ui-bg1ee-skin-4.9.1.iemod) |
| Lefreut's Enhanced UI: SoD style | 4.9.1 | `leui-sod` | `09de7721be81…` | [release asset](https://github.com/r-e-d/LeUI-SoD/releases/download/v4.9.1/lefreuts-enhanced-ui-sod-skin-4.9.1.iemod) |
| Northern Tales of the Sword Coast | f0ef95323630bc38df7dd4c969a2b48a71ebec82 | `ntotsc` | `b507d4b83367…` | [commit archive](https://codeload.github.com/Spellhold-Studios/NTotSC/zip/f0ef95323630bc38df7dd4c969a2b48a71ebec82) |
| Pai'Na NPC | 1.9 | `paina` | `8b448170062e…` | [tag archive](https://codeload.github.com/TheArtisanBG/Pai-Na-NPC-mod-for-BG2-EE/zip/refs/tags/v1.9) |
| Portraits Portraits Everywhere (DaftHunk fork) | 417226ea6642b8c682b2d049d0ec6e86f38b7f47 | `ppe` | `fc250193d9cc…` | [commit archive](https://codeload.github.com/DaftHunk/Portraits-Portraits-Everywhere/zip/417226ea6642b8c682b2d049d0ec6e86f38b7f47) |
| Item Randomiser | 8.1.1 | `randomiser` | `b5d95e801269…` | [release asset](https://github.com/Chrizhermann/chriz-item-randomiser/releases/download/v8.1.1/randomiser-v8.1.1.zip) |
| Rogue Rebalancing | 4.92 | `rr` | `692297f6da8f…` | [release asset](https://github.com/Spellhold-Studios/Rogue-Rebalancing/releases/download/v4.92/rr-v492.zip) |
| Safana in Amn | 05 | `safana` | `29672c878cfb…` | [tag archive](https://github.com/RoxanneSHS/SafanaBG2/archive/refs/tags/v05.zip) |
| Sarah NPC Romance Mod for BG2:ToB | 8 | `sarahtob` | `061b1da90d8c…` | [release asset](https://github.com/Gibberlings3/Sarah/releases/download/v8/sarah-npc-romance-mod-for-bg2-tob-v8.iemod) |
| Sirene NPC for BG2:EE | 00beda909a4a791a35016069b4ebb77f27279723 | `sirene-bg2` | `1a6d517b7b02…` | [commit archive](https://codeload.github.com/TheArtisanBG/Sirene-NPC-for-BG2-EE/zip/00beda909a4a791a35016069b4ebb77f27279723) |
| Spell Revisions | 4.21-chriz.7 | `spell-rev` | `613fca7d637b…` | [release asset](https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/download/v4.21-chriz.7/spell_rev-v4-21-chriz-7.zip) |
| Spell Revisions / Rogue Rebalancing compatibility | 4.21-chriz.7 | `srcb-rr-compat` | `705289651b99…` | [release asset](https://github.com/Chrizhermann/chriz-spell-revisions-patch/releases/download/v4.21-chriz.7/SRCB_RR_COMPAT-v4-21-chriz-7.zip) |
| Sword Coast Stratagems | 35.21 | `stratagems` | `dd6b0dbe3b9f…` | [release asset](https://github.com/Gibberlings3/SwordCoastStratagems/releases/download/v35.21/lin-stratagems-35.21.zip) |
| Thalantyr Item Upgrade | 4.5.1 | `thalan` | `ac5a2dff1feb…` | [release asset](https://github.com/CrevsDaak/thalan/releases/download/v4.5.1/thalantyr-item-upgrade-v4.5.1.iemod) |
| Unfinished Business for BG2 | 28 | `ub` | `1d5aff32d760…` | [release asset](https://github.com/Pocket-Plane-Group/UnfinishedBusiness/releases/download/v28/ub-v28.zip) |
| Aerie player voicepack | 1.0.1 | `voicepack-aerie` | `04ce446df621…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/aerie-player-voicepack-v1.0.1.zip) |
| Ajantis player voicepack | 2.0.0 | `voicepack-ajantis` | `7d2150ba9202…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/ajantis-player-voicepack-v2.0.0.zip) |
| Amelyssan player voicepack | 1.0.1 | `voicepack-amelyssan` | `f67c4e4b87cc…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/amelyssan-player-voicepack-v1.0.1.zip) |
| Edwin (BG1) player voicepack | 1.0.1 | `voicepack-edwin-bg1` | `6c02995b38ed…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/edwin-bg1-player-voicepack-v1.0.1.zip) |
| Edwin (BG2) player voicepack | 1.0.1 | `voicepack-edwin-bg2` | `d32e6f5577b9…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/edwin-bg2-player-voicepack-v1.0.1.zip) |
| Garithos player voicepack | 1.1.0 | `voicepack-garithos` | `24abfd30cd8c…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/garithos-player-voicepack-v1.1.0.zip) |
| Gromnir Il-Khan player voicepack | 1.0.1 | `voicepack-gromnir-il-khan` | `7aaad0379a6c…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/gromnir-il-khan-player-voicepack-v1.0.1.zip) |
| Imoen (BG1) player voicepack | 1.0.1 | `voicepack-imoen-bg1` | `cda87d6b56a1…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/imoen-bg1-player-voicepack-v1.0.1.zip) |
| Imoen (BG2) player voicepack | 1.0.1 | `voicepack-imoen-bg2` | `043d6216b4cf…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/imoen-bg2-player-voicepack-v1.0.1.zip) |
| Jaheira (BG1) player voicepack | 1.0.1 | `voicepack-jaheira-bg1` | `d8a3bd683fc1…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/jaheira-bg1-player-voicepack-v1.0.1.zip) |
| Jaheira (BG2) player voicepack | 1.0.1 | `voicepack-jaheira-bg2` | `5efe01db1594…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/jaheira-bg2-player-voicepack-v1.0.1.zip) |
| Neera player voicepack | 1.0.1 | `voicepack-neera` | `f2adbfd775f8…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/neera-player-voicepack-v1.0.1.zip) |
| Safana (BG1) player voicepack | 1.0.1 | `voicepack-safana-bg1` | `e5b3e7fa6946…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/safana-bg1-player-voicepack-v1.0.1.zip) |
| Safana (SoD) player voicepack | 1.0.1 | `voicepack-safana-sod` | `edd486e85c95…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/safana-sod-player-voicepack-v1.0.1.zip) |
| Sarevok player voicepack | 1.0.1 | `voicepack-sarevok` | `e105f9b296d0…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/sarevok-player-voicepack-v1.0.1.zip) |
| Viconia (BG1) player voicepack | 1.0.0 | `voicepack-viconia-bg1` | `bd56af0ef69e…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/viconia-bg1-player-voicepack-v1.0.0.zip) |
| Viconia (BG2) player voicepack | 1.0.0 | `voicepack-viconia-bg2` | `dde9cfca2692…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/viconia-bg2-player-voicepack-v1.0.0.zip) |
| Yeslick player voicepack | 1.0.1 | `voicepack-yeslick` | `c3c1213cc4ec…` | [release asset](https://github.com/Chrizhermann/chriz-bg-voicepacks/releases/download/v1.1.0/yeslick-player-voicepack-v1.0.1.zip) |
| WeiDU 249 for Windows x64 | 249.00 | WeiDU for every installer | `b156910cbec6…` | [release asset](https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip) |
| Xan NPC for BGII | 19 | `xan` | `47cd6622678f…` | [release asset](https://github.com/Pocket-Plane-Group/Xan_for_BGII/releases/download/v19/Xan_v19.zip) |
| Yeslick NPC | 5.0 | `yeslicknpc` | `d9b403a6feb8…` | [release asset](https://github.com/Spellhold-Studios/Yeslick-NPC/releases/download/v5.0/yeslick-npc-v5.0.zip) |
| Icewind Dale Voice Pack for EE 2.6+ | 1.2.0 | `zg-iwd-voices` | `7677e0d8fdef…` | [release asset](https://github.com/GraionDilach/Icewind-Dale-Voice-Pack-For-EE-2.6/releases/download/version-1.2.0/zg-iwd-voices-1.2.0.iemod) |
<!-- END GENERATED PINS -->

## History: the planning list of 2026-08-20 to 2026-09-04

This list guided the first pins. It is kept as a record and no longer updated; where it
differs from the tables above, the tables are right.

Originally generated 2026-08-20; synchronized with completed component review on
2026-09-01, with the Randomiser pin refreshed on 2026-09-03 and the first
release-ready official artifact block frozen on 2026-09-04. Immutable
source/release work still open is indexed in
`docs/curation/components/FOLLOW_UPS.md`.
Principle (Chris, 2026-08-19): the compilation targets CURRENT versions, not the
reference install's. Every pin below is validated by the Phase 1 test-install
milestone before it ships. Dropped mods (worksheet) are excluded.

**Global pins:** game 2.7.3.0 · WeiDU **249.00** for every mod (battle-tested,
SCS-safe; revisit >=251 only when Linux support lands) · language 0 (en_US)
except AJANTISBG2 (language 1).

### Third-party mods (kept)

| Mod | Installed | Proposed target | Notes |
|---|---|---|---|
| AJANTISBG2 | 21 | v21 | unchanged (installed = latest) |
| ARTISANSKITPACK | 6.0 | chriz-v1.3.1 fork @ `ac718614` | Immutable maintained release; curated Shapeshifter `5110` includes the footprint fix. |
| ARTISANSKITPACK_NPC | — | same chriz-v1.3.1 artifact | One immutable archive supplies all three TP2 identities. |
| ARTISANSKITPACK_TWEAK | — | same chriz-v1.3.1 artifact | One immutable archive supplies all three TP2 identities. |
| ASCENSION | 2.1.0 | 2.1.0 | Official release ZIP frozen; component `40` remains omitted until its duplicate-provider decision is resolved. |
| AURA_BG1_2_EET | — | reconciled Chris fork, pending release | Reconcile local balance work with seven newer upstream commits beyond reviewed upstream `285dabbc`, then publish an immutable source. |
| BARDICWONDERS | — | v2.9c-balance.2 @ `db0cf815` | Released selected fork; later finite-Abettor-HLA work still needs integration/release. |
| BGGO | v3.5 | v3.6 | Official v3.6 release ZIP frozen. |
| BRANWEN | v8pre | v8 | upgrade v8pre -> v8 |
| BUBB_SPELL_MENU_EXTENDED | v5.1 | v5.2 | Immutable official v5.2 tag archive frozen. |
| C0WARLOCK | 3.0 | commit `a8219922` | Pin the exact post-v4.0 commit and renamed `Artisans_Warlock/Artisans_Warlock.TP2` path. |
| CDTWEAKS | v18 | v18 | unchanged; reference already on v18 |
| CROSSMODBG2 | v30 | v30 | unchanged (installed = latest) |
| EEEX | v0.11.0-alpha | v1.2.0 | Official v1.2.0 release ZIP frozen; clean-stage runtime acceptance remains open. |
| EEFIXPACK | Alpha 3 | Beta_2 | Alpha_3 -> Beta_2 (2026-05-04); still prerelease - beta risk accepted per design |
| EET | v14.0 | master @ 74e91d72bca5d073fa11c1d088b90d7ff0c7105d (2026-08-06) | v14.1 tag is NOT 2.7-clean; pin this SHA until v14.2 tags |
| EET_END | — | same EET SHA | ships inside EET |
| EET_TWEAKS | 1.12 | v1.12 | unchanged (installed = latest) |
| EVANDRA | v2.2 | v2.2 | Blocked: no exact user-supplied archive contract is available yet; never rehost the page-gated archive. |
| FADE | 5.6 | v5.6 | unchanged (installed = latest) |
| HIDDENGAMEPLAYOPTIONS | 5.0 | v5.2 | Official Windows v5.2 release ZIP and menu frozen; execution remains deferred to the late Tweaks/UI ordering slice. |
| HQ_SOUNDCLIPS_BG2EE | 1.3 | v1.3 | Kept and default; installed equals latest reviewed release. |
| IEPBANTERS | v5.9 | v5.9 | unchanged (installed = latest) |
| IWDIFICATION | v11 | v11 | unchanged (installed = latest) |
| PAINA | — | v1.9 | upgrade ? -> v1.9 |
| RANDOMISER | 7 | maintained fork v8.1.1 @ `f4a9dfb` | Immutable release asset pinned; same component ids and documented EEex-v1.2 legacy-BCS fallback. Clean rebuild and new-game live validation remain open. |
| RR | v4.92 | v4.92 | unchanged (installed = latest) |
| SAFANA | v0.5 | v05 | unchanged (tag is spelled v05); SoD-items carryover fix component still to be built |
| SARAHTOB | v8 | v8 | Official v8 IEMOD frozen; custom portraits remain a separate blocked/manual asset decision. |
| SIRENE_BG2 | — | commit `00beda90` | Exact official commit archive frozen; the separate BG1 package remains a curation decision. |
| SPELL_REV | v4.19 | v4.21-chriz.3 | Immutable successor release pinned with the protection-refresh and hidden-subspell work; component `60` runs late. |
| STRATAGEMS | 35.21 | v35.21 | unchanged; MUST run under WeiDU 249 (breaks on 251); unverified-by-author on 2.7, community installs work |
| UB | v28 | v28 | Official v28 release ZIP frozen; component `19` remains omitted with Ascension `40` pending the canonical provider decision. |
| XAN | v19 | v19 | unchanged (installed = latest) |
| YESLICKNPC | v5.0 | v5.0 | unchanged (installed = latest) |

### Chriz-layer repos

Pinned at authoring time to a tagged release (or SHA) of each own repo:
ABETTORHLAREBALANCE, AKCB_BERSERKER, AKCB_SHAPESHIFTER, AURA_BALANCE_PATCH_SPELLS, BG2EE-EET-FIXPACK, CHRIZ-BG-MODPACK, CHRIZ-BG-REBALANCE, CHRIZ-SOD-REMIX, EEEXREMOTE.

### Local hotfixes (22)

No public versions to pin — each needs its home decision (worksheet section 4)
before it can be pinned to a chriz-repo release.

### Remaining freeze checks

- Recheck packaged EET and EE Fixpack releases immediately before manifest freeze; do not
  silently follow moving branches.
- Publish or pin every selected Chris-owned fork listed in the central follow-up queue.
- A reviewed target is not runtime acceptance: EEex 1.2, SCS 35.21/WeiDU 249, both EE
  Fixpack runs, and the EET merge still require the staged 2.7 tests.

# BIO / IE Mod Manager Landscape Brief (as of Aug 2026)

## "BiO" identified: Born2BSalty's Infinity Orchestrator (BIO)

**Repo:** https://github.com/Born2BSalty/Born2BSaltys_Infinity_Orchestrator
Self-description: "BIO is a WeiDU mod installer and mod-order orchestrator for Baldur's Gate Enhanced Edition mod setups" — currently focused on **BGEE, BG2EE, EET**. Announced on Beamdog Forums (General Modding) as a "modlist builder and one-click installer," thread active ~June 2026 — this is a **new, post-2024 tool**, matching the user's recollection.

- **What it is:** GUI orchestrator that scans TP2 components, lets you select/reorder, validates compatibility rules, then drives the actual install by running dark0dave's `mod_installer` with live console control.
- **Tech/platform:** Rust (edition 2024, stable 1.85+), egui UI, vendored ANTLR-generated TP2 parser. **Windows, Linux, macOS.** GPLv3+.
- **Component-selection UX:** 5-step wizard. Step 2: scan mods folder, search/filter components, bulk select, or import an existing WeiDU.log; "compatibility pills" surface issues inline before install. Step 3: reorder + rule validation (dependency rules, conflict/forbid patterns, game-target predicates, conditional patch logic).
- **Auto-download:** optional (`--download` flag in Step 1); mod sources must be configured separately (Downloads GUIDE.md in repo).
- **Maintained?** Yes, very actively — alpha stage. Latest release **v0.2.1-alpha (Jul 21, 2026)**; ~10 releases Mar–Jul 2026, 72 commits, Discord community, BEGINNERS_GUIDE.md, diagnostics-bundle export.
- **Caveat:** alpha software, small user base, no IWD/PST support yet.

## Broader landscape

**Project Infinity (PI)** — ALIENQuake — https://github.com/ALIENQuake/ProjectInfinity | thread: https://forums.beamdog.com/discussion/74335/project-infinity-mod-manager-for-baldurs-gate-icewind-dale-planescape-torment-and-eet
Windows-only .NET/PowerShell GUI for BGEE/BG2EE/IWDEE/PSTEE/EET. Component UX: expand mod tree → tick components; import WeiDU.log or paste install-sequence text (localized component names); flexible order editing. **No auto-download** (you supply extracted mods). Maintained?: semi — issue activity into 2025 but **no GitHub releases** (exe shipped in-repo); long-running public beta, development slow.

**EE Mod Setup Tool (BiG World Setup fork)** — https://github.com/cmorganbg/EE-Mod-Setup (forks: DatuStram, southfla79, etc.) | G3 thread: https://www.gibberlings3.net/forums/topic/29337-eeeet-mod-setup-tool/
AutoIt/VBScript, **Windows-only**. BWS-style curated pipeline: mod selection UI over a maintained compilation, **auto-downloads current mod versions**, resolves conflicts/dependencies, enforces known-good install order, SoD DLC merging. Maintained?: fragmented across community forks; original BWS discontinued Feb 2019; activity is sporadic — the community has largely moved toward PI/BIO-style tools.

**WeiDU Install Tool (WIT)** — Argent77 — https://github.com/InfinityTools/WeiduInstallTool | https://www.gibberlings3.net/mods/tools/wit/
Java/JavaFX graphical front end for a **single WeiDU run** (all IE games). Installers for Windows/Linux/macOS, .tp2 file association, usability niceties over the WeiDU CLI prompts. **Not an orchestrator**: no auto-download, no multi-mod order management. Apache-2.0; actively maintained.

**mod_installer ("Infinity Engine Mod Installer")** — dark0dave — https://github.com/dark0dave/mod_installer | https://crates.io/crates/mod_installer | G3: https://www.gibberlings3.net/forums/topic/39781-mod-installer/
Rust **CLI** that replays a backed-up `weidu.log` to unattended-install a full mod stack in order. Cross-platform, actively maintained (updated Jul 23, 2026). No selection UI of its own — it is the install engine BIO wraps. Directly relevant to this repo's `manifest/install-order.tsv` + install-driver approach.

**Infinity Mod Forge** — https://www.gibberlings3.net/forums/topic/41107-infinity-mod-forge-web-based-install-order-builder/ | https://forums.beamdog.com/discussion/90593/infinity-mod-forge-build-your-ultimate-eet-install
New web-based install-order builder for EET (and wider IE family): browse mods, build/validate an ordered list in the browser. Planning/ordering aid only — does not install. (G3 page 403'd on fetch; details from search snippets.)

**"Infinity Auto Installer":** no tool found under that exact name — closest matches are dark0dave's `mod_installer` and BIO. Minor extra: subtledoctor's MacOS_Weidu_Launcher (https://github.com/subtledoctor/MacOS_Weidu_Launcher) for macOS WeiDU runs.

## Bottom line

"BiO/BIO" = **Born2BSalty's Infinity Orchestrator**, a 2026 Rust/egui alpha-stage GUI orchestrator (BGEE/BG2EE/EET) with wizard-based component picking, compatibility validation, optional downloads, and `mod_installer` as its execution engine. The 2026 actively-maintained set is: BIO (GUI orchestrator, alpha), mod_installer (CLI replay engine), WeiDU Install Tool (single-mod GUI front end), Infinity Mod Forge (web order planner); Project Infinity is semi-dormant Windows-only; EE Mod Setup Tool survives as sporadically maintained BWS forks.

Sources: [BIO repo](https://github.com/Born2BSalty/Born2BSaltys_Infinity_Orchestrator) | [BIO releases](https://github.com/Born2BSalty/Born2BSaltys_Infinity_Orchestrator/releases) | [bgee GitHub topic](https://github.com/topics/bgee?o=desc&s=updated) | [Project Infinity repo](https://github.com/ALIENQuake/ProjectInfinity) | [PI Beamdog thread](https://forums.beamdog.com/discussion/74335/project-infinity-mod-manager-for-baldurs-gate-icewind-dale-planescape-torment-and-eet) | [EE-Mod-Setup (cmorganbg)](https://github.com/cmorganbg/EE-Mod-Setup) | [EE/EET Mod Setup Tool G3 thread](https://www.gibberlings3.net/forums/topic/29337-eeeet-mod-setup-tool/) | [WeiduInstallTool](https://github.com/InfinityTools/WeiduInstallTool) | [WIT on G3](https://www.gibberlings3.net/mods/tools/wit/) | [mod_installer](https://github.com/dark0dave/mod_installer) | [mod_installer G3 thread](https://www.gibberlings3.net/forums/topic/39781-mod-installer/) | [Infinity Mod Forge G3 thread](https://www.gibberlings3.net/forums/topic/41107-infinity-mod-forge-web-based-install-order-builder/) | [Infinity Mod Forge Beamdog thread](https://forums.beamdog.com/discussion/90593/infinity-mod-forge-build-your-ultimate-eet-install) | [MacOS_Weidu_Launcher](https://github.com/subtledoctor/MacOS_Weidu_Launcher)
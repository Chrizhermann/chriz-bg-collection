## Contradictions

1. **WeiDU version pin — direct conflict.** `weidu-automation` recommends "pin one bundled WeiDU ≥ v251.00" (needed for Linux-without-tolower); `game-version-landscape` says SCS 35.21 breaks on 251 (`finbalth.bcs` PARSE ERROR) and to pin 249 per-mod. Architecture consequence: the engine must support per-mod WeiDU binaries, and Linux support + SCS cannot share one binary. Unresolved.
2. **PI auto-download.** `project-infinity` brief: PI has a GitHub-backed "Download Mods" panel + Delta Updates; `bio-and-other-managers`: PI has "**No auto-download**." The detailed brief is almost certainly right; the landscape brief's framing of the gap your app fills is wrong on this axis.
3. **Manifest ground truth.** Repo bootstrap says 414 components; `local-modding-guide` says the reference WeiDU.log has 364 entries (and the 444-row CSV is stale). A 50-component discrepancy in the thing the whole app replays needs reconciling before schema design.

## Load-bearing unknowns

4. **Which game build does a public user install onto?** Recipe is validated on 2.6.6.0; Steam/GOG now ship 2.7.3.0, and no brief addresses whether users *can* obtain 2.6.6 (Steam beta branch? GOG rollback?). If they can't, the manifest must move to 2.7 — where EET is only fixed on an untagged master SHA and SCS is author-unverified. This forks the entire pin-list and dictates build-detection + per-build manifest architecture. Biggest open hole.
5. **EEex is Windows-only, so the *compilation* is Windows-locked** (EEex, Bubb's Spell Menu, InfinityLoader launch, EEex-variant cdtweaks components). The GUI-stack brief evaluates macOS/Linux ports of the *app* without noting the mod stack can't install on native non-Windows games. "Cross-platform foundation" needs a definition: degraded no-EEex variant stack, Wine-prefix management (case-sensitivity, paths), or Windows-only in practice.
6. **No per-mod URL resolution exists for THIS manifest.** The 85–90% GitHub figure is ecosystem-level; the guide documents zero URLs, provenance includes Nexus and at least one mod deleted from GitHub. Whether the manual-download fraction is 2 mods or 15 (weaselmods cluster) determines how heavy the manual-drop-queue UX must be and whether the simple-persona promise holds. Also unaddressed: public release can't ship the private dead-link archive — a single unfetchable required mod blocks the release.
7. **The two-game EET pipeline is uncovered by every brief.** Pre-merge BG1EE phase (DLC Merger / Steam-vs-GOG SoD handling, BG1 NPC Project, BG1 UB), locating/validating two installs, then merge into BG2EE — the copy-then-install model and wizard must handle all of it; no brief researched it.
8. **Manifest schema must pin unversioned artifacts** — EET fix exists only as a master commit, Artisan's Kitpack only as a moving branch zip. Needs commit-SHA archive URLs + content hashes; affects manifest format and update-check semantics.

## Stale/wrong claims

9. **Dates off by a year:** `weidu-automation` says WeiDU 251.00 = 2025-03-01 (game-version + SCS-breakage thread put it at 2026-03-01); `mod-hosting-downloads` says cdtweaks v18 = Aug 2024 (game-version: 2025-08-22, after v17 2025-08-07).
10. **"BG tooling scene skews C#" (gui-stack, propping up Avalonia's flip-to-#1 case) is stale.** PI is closed-source (no contributor pool), and every actively developed 2026 tool — BIO, mod_installer, modda, the WeiDU fork — is Rust. This strengthens the Tauri/Rust recommendation and weakens Avalonia's.

Otherwise the briefs are sufficient: WeiDU invocation semantics, exit-code/log verification, stack-model/append-only design, hosting/rate-limit strategy, and signing are well-covered and mutually consistent.
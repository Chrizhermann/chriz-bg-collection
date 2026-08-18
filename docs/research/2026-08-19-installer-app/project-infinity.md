# Project Infinity (PI) — Research Brief (as of 2026-08)

## 1. Maintenance status

- **Semi-maintained, low cadence — not formally abandoned.** Repo: [github.com/ALIENQuake/ProjectInfinity](https://github.com/ALIENQuake/ProjectInfinity) (created 2018-10-14; 92 stars, 11 forks, 27 open issues; not archived).
- **Latest version: 0.10.7, committed 2026-03-29** (commit message "0.10.7") — after a ~3.5-year gap (0.10.6 was 2022-08-22). The 0.10.7 fix was promised as far back as 2024 ("Fixed in 0.10.7, no ETA"). Last push per GitHub API: 2026-03-29. Changelog carries no dates ([CHANGELOG.md](https://github.com/ALIENQuake/ProjectInfinity/blob/master/CHANGELOG.md)).
- **Distribution is unusual:** no GitHub Releases at all; the 3.3 MB `ProjectInfinity.exe` is committed directly to `master` alongside `ProjectInfinity.exe.config`, `version.json`, `CHECKSUM.md`. **Closed source** — repo contains only the binary + docs (deliberate; AL|EN cites preventing malicious forks promoting bad mods, unlike open-source BWS).
- **Author activity:** AL|EN answered the [Beamdog thread](https://forums.beamdog.com/discussion/74335/project-infinity-mod-manager-for-baldurs-gate-icewind-dale-planescape-torment-and-eet) actively through 2023–2024; by May 2025 only a brief cameo ("I will be active for a couple of days next week"), with community member Morpheus562 fielding questions. Thread still has user activity through Nov 2025 (p41).
- **Platform: Windows-only .NET Framework app, unchanged.** Win 10/11 x64: no extra requirements; Win 7/8.1: ".NET Framework 4.5.2 or above" + "PowerShell 5.1". PowerShell is a backend dependency (UI verbs like `Set-InstallSequence` are PS cmdlet-styled). No macOS/Linux client, though its `.iemod` package format is explicitly cross-platform.

## 2. Architecture

**Mod discovery.** User manually extracts mods into one folder of their choice; PI recursively scans it (unlimited subfolder nesting), finds `.tp2` mods, and shows a hierarchical mod/component tree with localized component names. Rescan/refresh is a known weak spot (see §5).

**Metadata format** ([wiki: Adding metadata for mod](https://github.com/ALIENQuake/ProjectInfinity/wiki/Adding-metadata-for-mod)): a plain-ini file **next to the mod's `.tp2`, named after the tp2 basename** (`setup-ModExample.tp2` → `ModExample.ini`), UTF-8 **without BOM**, single `[Metadata]` section. Keys: `Name`, `Author`, `Description`, `Readme` (comma-list; txt/md/html/pdf; case-sensitive), `Forum`, `Homepage`, `Download` (GitHub repo root URL, **without** `/releases`), `LabelType` (e.g. `GloballyUnique`), and Dynamic Install Order keys `Before` / `After` (comma-separated mod IDs). Canonical example:

```ini
[Metadata]
Name = Mod Example - Optional Features
Author = AL|EN
Description = This mod is a guide on how to use optional 'Project Infinity' features.
Readme = https://example.com/ModName-Readme.html
Forum = https://www.example.com/forums/categories/forum-thread
Download = https://github.com/AccountOrOrgName/ModExample-OptionalFeatures
LabelType = GloballyUnique
Before = ModX, EET_end
After = EET, ModY
```

This ini format has become the de-facto community metadata standard (the newer "WeiDU Install Tool" G3 forum discusses the same ini scheme: [On mod metadata, and INI files](https://www.gibberlings3.net/forums/topic/37274-on-mod-metadata-and-ini-files/)).

**Install order expression** (several layered mechanisms):
- **Install Order Groups** — coarse drag-and-drop buckets; explicitly "by no means correct or perfect install order" ([wiki](https://github.com/ALIENQuake/ProjectInfinity/wiki/Install-Order-Groups)).
- **Dynamic Install Order** — `Before`/`After` metadata rules; since 0.10.0 "Install Order Rules are checked only once" (breaking change).
- **Install sequence** — the concrete to-install list shown in a window; copy/pasteable as text (Ctrl-A/C/V) and **saved into the `Logs` folder on Start-Installation**; entry format is mod + component with `;` separating component ID from description (e.g. `stratagems:4040` cited by users). Sharing mod lists / sequences is a headline feature.
- **Sorting Order file** (`SortingOrder.csv`, [wiki](https://github.com/ALIENQuake/ProjectInfinity/wiki/Sorting-Order-feature)) — order template that **only sorts already-selected components, never adds/removes**. Syntax: `ModName *` (wildcard = all selected components), explicit component splits for interleaving, e.g. `Ascension *` … `Ascension 101 102` to push 101–102 last; unlisted mods fall to the end. Convertible from a `weidu.log` (regex recipe) or old BWS data ([wiki](https://github.com/ALIENQuake/ProjectInfinity/wiki/Converting-weidu.log-file-into-sorting-order-data)).
- Direct **weidu.log import** as an install sequence is also supported.

**IEMOD package format** ([wiki spec](https://github.com/ALIENQuake/ProjectInfinity/wiki/Specification-of-the-IEMOD-file-format)): ZIP renamed `.iemod`, DEFLATE or stored, UTF-8-no-BOM filenames, no encryption/multi-volume/self-extract, tp2 must sit in a top-level dir matching its name, forbidden reserved names and game-file dirs. Generated automatically by the **Infinity Auto Packager** GitHub Action ([InfinityTools/InfinityAutoPackager](https://github.com/InfinityTools/InfinityAutoPackager)) on release publish (also emits standardized `.zip` with Win/macOS WeiDU binaries; package name from metadata ini, version from tp2).

## 3. Auto-download capability

- **Yes — "Download Mods" panel, GitHub-only.** The source list is a **curated set of GitHub accounts/orgs baked into PI** and extended release-by-release (changelog: Spellhold Studios added 0.7.9, Pocket Plane Group 0.7.10, BGforgeNet 0.7.15, Sampsca 0.8.7; 0.10.7 "excluded .github repository by default"). Users request additions to the list; the metadata `Download` key must point at a GitHub repo root.
- **Only `.iemod` packages auto-extract**; custom `.zip`/`.rar`/`.7z` downloads still require manual extraction. Mods without GitHub hosting/direct links are simply outside the system: the baseline workflow is "extract all the mods you want into a folder yourself", so non-downloadable mods are manually acquired and dropped into the scan folder.
- **Delta Updates**: one-click incremental updates for GitHub-hosted mods ([wiki](https://github.com/ALIENQuake/ProjectInfinity/wiki/Delta-Updates-for-mods-hosted-at-Github)).
- PI also **auto-downloads WeiDU** into `{ProjectInfinity}\Tools` on first run (this broke when the upstream WeiDU release archive was renamed; fixed only in 0.10.7). [Offline mode](https://github.com/ALIENQuake/ProjectInfinity/wiki/Using-Project-Infinity-offline): manually place WeiDU exe from [WeiDUorg releases](https://github.com/WeiDUorg/weidu/releases/latest) into the versioned Tools folder.

## 4. WeiDU invocation

- Confirmed flags: **`--skip-at-view`** (suppresses `AT_INTERACTIVE_EXIT` readme popups that would pause batch installs). Historically used `--quick-log` (added 0.4.5, removed 0.7.5). Component names are read per-language (localized names in the tree/sequence).
- **Error handling: pause-on-error/pause-on-warning** during the batch run (since 0.6.3/0.6.8 "mod errors will pause installation"), letting the user intervene, then continue; can be disabled via experimental ini settings `ExpDisablePauseOnWarning = true` / `ExpDisablePauseOnError = true` ([wiki: Experimental Features](https://github.com/ALIENQuake/ProjectInfinity/wiki/Experimental-Features) — with the caveat that "error reporting is essential for modders").
- Install logs + the executed install sequence are written to a `Logs` folder. Note: users repeatedly observed PI **uninstalling and reinstalling already-installed components** mid-run (WeiDU stack-reordering behavior surfacing through PI), and occasional silently skipped mods.

## 5. Known limitations / community complaints

- **Antivirus false positives are the #1 recurring complaint** (Defender/AVG/Malwarebytes flag the closed-source exe as trojan; "app won't launch" reports usually trace to this). AL|EN: a code-signing certificate at ~$300/year is not worth it; closed source is intentional. ([thread p37](https://forums.beamdog.com/discussion/74335/project-infinity-mod-manager-for-baldurs-gate-icewind-dale-planescape-torment-and-eet/p37), p39–p41)
- **Release cadence/maintenance risk**: 3.5 years between 0.10.6 and 0.10.7; author only sporadically present since 2024; 27 open issues; closed source means nobody can fork-fix.
- **Selection fragility**: adding a mod or clicking refresh **clears selected components unless the mod uses GloballyUnique labels**; selections don't persist across sessions (users re-tick dozens of boxes); no "select all" for big component lists.
- **Import quirks** (author-acknowledged): importing install sequences doesn't select or sort mods in the left panel (weidu.log import does select); 2025 reports of CSV/weidu.log import being effectively non-functional.
- **Scanning flakiness**: intermittent missed mods needing rescans; "non-existent mods" install errors despite mods present.
- **UI**: hangs during mod update/copy operations; DPI/font issues at 150% scaling on Win11; no progress/sound notification for multi-hour installs.
- **Documentation** widely criticized — the forum thread + wiki don't amount to a manual; EET two-phase install and "install order vs install sequence vs sorting order" terminology confuse newcomers; the tutorial video was pulled after import UX changed.
- **Windows-only**; macOS/Linux users are pushed toward the `.iemod`/zip packages + manual WeiDU or newer alternatives.

**Key sources:** [GitHub repo](https://github.com/ALIENQuake/ProjectInfinity) · [CHANGELOG](https://github.com/ALIENQuake/ProjectInfinity/blob/master/CHANGELOG.md) · [Wiki: metadata](https://github.com/ALIENQuake/ProjectInfinity/wiki/Adding-metadata-for-mod), [Sorting Order](https://github.com/ALIENQuake/ProjectInfinity/wiki/Sorting-Order-feature), [IEMOD spec](https://github.com/ALIENQuake/ProjectInfinity/wiki/Specification-of-the-IEMOD-file-format), [offline use](https://github.com/ALIENQuake/ProjectInfinity/wiki/Using-Project-Infinity-offline), [experimental](https://github.com/ALIENQuake/ProjectInfinity/wiki/Experimental-Features) · [Beamdog master thread (OP + p37/p39/p40/p41)](https://forums.beamdog.com/discussion/74335/project-infinity-mod-manager-for-baldurs-gate-icewind-dale-planescape-torment-and-eet) · [Infinity Auto Packager](https://github.com/InfinityTools/InfinityAutoPackager) · [G3 metadata/ini discussion](https://www.gibberlings3.net/forums/topic/37274-on-mod-metadata-and-ini-files/) (G3 forum blocks direct fetch; G3 modder-thread details corroborated via wiki + Beamdog mirror of the same content).
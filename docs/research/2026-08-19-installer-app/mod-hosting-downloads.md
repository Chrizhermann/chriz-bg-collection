# BG:EE / Infinity Engine mod hosting & auto-download feasibility (as of Aug 2026)

## 1) Where mods live and what fraction is directly fetchable

**GitHub is now the dominant host.** The three classic communities have all migrated their catalogs to GitHub orgs:

| Host | Status | Direct download? |
|---|---|---|
| **Gibberlings3** — [github.com/Gibberlings3](https://github.com/Gibberlings3) | ~115 repos; all flagship mods publish tagged GitHub releases with zip assets | Yes — stable `releases/download/` URLs |
| **Spellhold Studios** — [github.com/Spellhold-Studios](https://github.com/Spellhold-Studios) (new org, hyphenated) | ~172 repos, active (updates Aug 2026: Infinity-Sounds, BP-BGT-Worldmap, BG Graphical Overhaul). Old org [github.com/SpellholdStudios](https://github.com/SpellholdStudios) (74 repos) is a dormant archive after the 2022 owner disappearance; README redirects to the new org | Yes — GitHub, no registration ([SHS announcement](https://www.shsforums.net/topic/62258-problems-with-file-downloads/): forum download centre **shut down**, all eligible files moved to GitHub) |
| **Pocket Plane Group** — [pocketplane.net](https://www.pocketplane.net/) | Self-describes as "archives of a once-mighty modding empire"; mods migrated to [github.com/Pocket-Plane-Group](https://github.com/Pocket-Plane-Group) (~34 repos) | Yes — GitHub |
| **weaselmods.net** (Lava Del'Vortel, ~50+ NPC/quest mods) | [downloads.weaselmods.net](https://downloads.weaselmods.net/) — WordPress Download Manager (`wpdmdl`) pages | **No stable direct URL.** Download button is JS/dynamic (`[Download](#)` in static HTML); no login/ad gate, but not automation-friendly. Official fallback is a shared [Dropbox folder](https://www.dropbox.com/sh/56dr4q8h2q7djri/AACJfi0A2mV7hFlH3eqDLTcEa?dl=0). Treat as **manual-download** host |
| **Artisan's Corner** (ArtemiusI) | Site migrated: artisans-corner.com → [theartisanbg.github.io/The-Artisans-Corner](https://theartisanbg.github.io/The-Artisans-Corner/) | Yes, but as **branch zips**, not releases: e.g. Kitpack = `https://github.com/TheArtisanBG/The-Artisan-s-Kitpack/archive/refs/heads/master.zip` — direct-fetchable but **unversioned/unstable content** (moving branch snapshot) |
| **Beamdog forums** | Attachments under `forums.beamdog.com/uploads/…` are direct, unauthenticated URLs (e.g. [this uploaded PDF](https://forums.beamdog.com/uploads/editor/bi/d8lihts3kotg.pdf)); few mods are hosted this way — the forum mostly links out to GitHub | Mostly yes for the rare attachment-hosted mod, but URLs are opaque and unversioned |
| **shsforums.net attachments** | Remaining forum-post attachments **require forum registration** (auth wall); the curated stuff moved to GitHub | **No** |

**Your named examples — all GitHub releases with zip assets:**
- **EET** 14.1 (Apr 2025) — [Gibberlings3/EET/releases](https://github.com/Gibberlings3/EET/releases)
- **SCS** 35.21 (Nov 2024) — [Gibberlings3/SwordCoastStratagems/releases](https://github.com/Gibberlings3/SwordCoastStratagems/releases)
- **cdtweaks** v18 (Aug 2024) — [Gibberlings3/Tweaks-Anthology/releases](https://github.com/Gibberlings3/Tweaks-Anthology/releases)
- **Spell Revisions** 4.21 (Aug 2026!) — [Gibberlings3/SpellRevisions/releases](https://github.com/Gibberlings3/SpellRevisions/releases)
- **IWDification** v11 (Jul 2026) — [Gibberlings3/iwdification/releases](https://github.com/Gibberlings3/iwdification/releases)
- **Ascension** 2.1.0 (May 2026) — [Gibberlings3/Ascension/releases](https://github.com/Gibberlings3/Ascension/releases)
- **EEex** 1.2.0 (Aug 2026) — [Bubb13/EEex/releases](https://github.com/Bubb13/EEex/releases)
- **Bubb's Spell Menu Extended** v5.2 — [Bubb13/Bubbs-Spell-Menu-Extended/releases](https://github.com/Bubb13/Bubbs-Spell-Menu-Extended/releases)
- **Artisan's Kitpack** — branch zip only (above), no tagged releases.

Many G3/SHS repos use [InfinityAutoPackager](https://github.com/InfinityTools/InfinityAutoPackager) to auto-attach platform packages (win/mac/lin + `.iemod`) on release — asset naming is fairly standardized (that's why SCS/EET/SR releases carry 4–6 assets).

**Fraction estimate:** by mod count in a typical modern EET stack (like this repo's 84-mod manifest), roughly **85–90% have stable GitHub release/tag URLs** (G3 + SHS + PPG + Bubb13 + Argent77 + UnearthedArcana/subtledoctor + K4thos etc.). The main structural holdouts: **weaselmods** (largest single manual-download cluster), a handful of Beamdog-forum-attachment UI mods, non-English community hosts (Clan DLAN, some French/Russian forums), and unversioned branch-zip hosts like Artisan's Corner.

## 2) GitHub rate limits / reliability for an end-user app

- **REST API unauthenticated: 60 req/hr/IP** ([docs](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)). Enough for a one-shot "resolve latest release" pass over ~60 mods, but shared NAT/CI will hit it; Scoop tracks this exact pain ([ScoopInstaller/Scoop#6609](https://github.com/ScoopInstaller/Scoop/issues/6609)).
- **May 2025: GitHub tightened anonymous limits** for HTTPS cloning, anonymous REST, and `raw.githubusercontent.com` due to scraping ([changelog](https://github.blog/changelog/2025-05-08-updated-rate-limits-for-unauthenticated-requests/) — numbers undisclosed; they push authentication).
- **Release asset downloads via `browser_download_url` do not consume the REST core limit** — assets redirect to short-lived JWT-tokenized `objects.githubusercontent.com` URLs; the API asset endpoint with `Accept: application/octet-stream` redirects likewise ([REST release-assets docs](https://docs.github.com/en/rest/releases/assets)). Practical implications: (a) don't cache resolved asset URLs beyond minutes; (b) follow redirects; (c) hardcoded `github.com/<org>/<repo>/releases/download/<tag>/<file>` URLs are the stable, rate-limit-friendly form.
- **Branch zips** (`archive/refs/heads/...` via codeload) work unauthenticated but are subject to the tightened anonymous/abuse limits and are content-unstable — pin **tag** archives (`archive/refs/tags/vX.zip`) instead when a repo has no release assets.
- Design guidance: embed pinned release-asset URLs in the manifest (no API needed at install time), support an optional user PAT for API-based "check for updates", back off on 403/429, set a real User-Agent.

## 3) How existing tools do it

**Project Infinity** ([ALIENQuake/ProjectInfinity](https://github.com/ALIENQuake/ProjectInfinity)):
- Pull model driven by **author-maintained metadata**: an ini next to the `.tp2` with a `[Metadata]` section — `Name, Author, Description, Readme, Forum, Homepage, Download, LabelType, Before, After` ([wiki: Adding metadata for mod](https://github.com/ALIENQuake/ProjectInfinity/wiki/Adding-metadata-for-mod)). For GitHub mods, `Download=` must be the **repo URL** (not /releases) — this enables PI's **Delta Updates** (partial re-download for GitHub-hosted mods, [wiki page](https://github.com/ALIENQuake/ProjectInfinity/wiki/Delta-Updates-for-mods-hosted-at-Github)).
- Non-fetchable hosts → manual UX: PI supports a fully **offline workflow** ([wiki: Using Project Infinity offline](https://github.com/ALIENQuake/ProjectInfinity/wiki/Using-Project-Infinity-offline)) where the user pre-downloads/extracts archives into the mods folder and PI just rescans. In practice that's the weaselmods path.
- Also defines an `.iemod` package format ([wiki](https://github.com/ALIENQuake/ProjectInfinity/wiki/Specification-of-the-IEMOD-file-format)).

**EE Mod Setup Tool** (BWS fork, e.g. [EE-Mod-Setup mirrors](https://github.com/cmorganbg/EE-Mod-Setup)):
- BWS model: a **centrally maintained link database** shipped with the tool; auto-downloads from those URLs, opens a browser and waits for the user to drop the file into the download dir when a host blocks automation (classic BWS "manual download" prompt).
- Cautionary tale: it bundled/pointed to **rehosted, modified, outdated mod copies against authors' wishes** (Roxanne / baldursextendedworld.com), which the community blacklisted — see [G3: Roxanne's Inofficial Mod Versions](https://www.gibberlings3.net/forums/topic/31275-roxannes-inofficial-mod-versions/) and the [EE/EET Mod Setup Tool thread](https://www.gibberlings3.net/forums/topic/29337-eeeet-mod-setup-tool/). Lesson for your installer: **fetch from official hosts only, never rehost** — exactly this repo's "bundle the recipe, not the mods" rule.

## 4) Licensing / ToS considerations

- **Redistribution ≠ downloading.** Most classic mod readmes forbid redistribution/rehosting without permission (see also [G3: Endarire's unauthorized mod uploads](https://www.gibberlings3.net/forums/topic/37157-endarires-unauthorized-mod-uploads/)); an installer that fetches from the **author's official host** is the community-blessed pattern — PI's metadata standard exists precisely to let authors opt in with their own links.
- **GitHub-hosted mods:** programmatic downloads are fine under GitHub ToS within rate limits; many G3 repos now carry explicit OSS-ish or "no redistribution" licenses per-repo — irrelevant to fetching, relevant only to mirroring.
- **weaselmods:** no explicit anti-hotlink policy found, but the site's download counters (e.g. Southern Edge: 29,454 downloads tracked) matter to the author, downloads are deliberately funneled through mod pages, and the official mirror is a Dropbox folder Lava controls. Scraping the `wpdmdl` endpoint would be fragile and impolite — do a "open page in browser + watch download folder" manual step instead.
- **shsforums attachments:** behind registration — don't automate; the maintained copies are on the GitHub org anyway.
- **Private archiving** of dead-link zips (your current practice, repo private) stays within the norm; publishing them would not.

**Bottom line for the installer:** pin `releases/download/` URLs (or tag archives) for the ~85–90% GitHub-hosted majority; implement a PI-style manual-download queue (open URL, watch a drop folder, verify hash) for weaselmods and stragglers; optional PAT for update checks; never mirror.

Sources: [Gibberlings3 org](https://github.com/Gibberlings3) · [SpellholdStudios (old)](https://github.com/SpellholdStudios) · [Spellhold-Studios (new)](https://github.com/Spellhold-Studios) · [SHS downloads shutdown](https://www.shsforums.net/topic/62258-problems-with-file-downloads/) · [pocketplane.net](https://www.pocketplane.net/) / [Pocket-Plane-Group org](https://github.com/Pocket-Plane-Group) · [downloads.weaselmods.net](https://downloads.weaselmods.net/) · [Weasel Mods announcement](https://www.gibberlings3.net/forums/topic/31205-weasel-mods-the-new-modding-site/) · [Artisan's Corner (GitHub Pages)](https://theartisanbg.github.io/The-Artisans-Corner/) · [PI wiki metadata](https://github.com/ALIENQuake/ProjectInfinity/wiki/Adding-metadata-for-mod) · [PI Beamdog thread](https://forums.beamdog.com/discussion/74335/project-infinity-mod-manager-for-baldurs-gate-icewind-dale-planescape-torment-and-eet) · [EE-Mod-Setup](https://github.com/cmorganbg/EE-Mod-Setup) · [G3 Roxanne thread](https://www.gibberlings3.net/forums/topic/31275-roxannes-inofficial-mod-versions/) · [GitHub rate-limit docs](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api) · [GitHub changelog May 2025](https://github.blog/changelog/2025-05-08-updated-rate-limits-for-unauthenticated-requests/) · [release-assets API](https://docs.github.com/en/rest/releases/assets) · [Scoop#6609](https://github.com/ScoopInstaller/Scoop/issues/6609) · [InfinityAutoPackager](https://github.com/InfinityTools/InfinityAutoPackager) · releases pages linked inline above.
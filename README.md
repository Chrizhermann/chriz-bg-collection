# Chriz Easy BG (CEBG)

Chriz Easy BG is a one-click Windows installer and launcher for a curated, heavily modded
**Baldur's Gate: Enhanced Edition + Siege of Dragonspear + Baldur's Gate II: Enhanced
Edition** playthrough using EET.

The goal is a polished experience for players, not another expert-only mod manager. CEBG
finds supported clean games, starts with the recommended collection choices, builds a
separate installation, verifies the result, and reopens later as a simple launcher.

## Current status (2026-09-07)

The current release is Windows app **0.1.0-alpha.15**, with recipe **0.1.0-alpha.13**
and 434 recommended components. BG Rebalance v0.3.2 fixes Tempus installation without
Spell Revisions. Other mod pins and default choices are unchanged. It retains the
guarded recovery, shortcut handling and restored setup preferences from alpha.14.

Get the installer and guide from [the collection page](https://bg.chrizfader.org/collection),
or [public releases](https://github.com/Chrizhermann/chriz-easy-bg/releases).
Each release records its package, tested scope and known limitations; source changes
do not by themselves establish that a new binary has been published.
Supported source games are clean English Steam BG:EE + SoD and BGII:EE 2.7.3 installations.

CEBG currently provides:

- automatic game discovery plus clear clean-source checks for the currently verified Steam
  2.7.3 builds;
- an install-first UI with recommended choices already selected;
- verified downloads and guided handling of the few manually supplied archives;
- a separate, resumable game installation with an immutable receipt and diagnostics;
- a returning-player launcher with **Play**, **Open game folder**, and multiple-install
  switching;
- an optional verified desktop shortcut, enabled by default; and
- separate application, recipe, and installation update guidance without unsafe in-place
  WeiDU surgery.

The public alpha updater channel is active. Packages carry Tauri updater signatures;
the current signing setup does **not provide Windows Authenticode**, so Windows may show
an unknown publisher. Follow the
[alpha.15 acceptance note](docs/patch-acceptance-alpha15-2026-09-07.md) for source, package
and installation evidence. The existing no-SR test was recovered in place and its
429 components verified; gameplay and native updater apply/restart remain separate checks.
See [docs/handover.md](docs/handover.md) for the maintained development status.

## Source and downloads

This repository contains CEBG's installer/engine source and recipe authoring data. The
separate [chriz-easy-bg repository](https://github.com/Chrizhermann/chriz-easy-bg) hosts public
binary releases and the updater feed; its release downloads are not a source checkout.
CEBG's own source and recipe tooling are available under MIT. Third-party dependencies
retain their terms, collected in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
The source and binary distribution repositories have separate roles; existing release
downloads and updater URLs remain the supported player entry points.

The alpha.15 package was built from commit
`255be75aa232f3af5ad1e1969ccba313fffcccce`, including its versioned notices.
The preceding alpha.14 source is `5610783590ad49b24101d5d3a6a85b018abadb7e`.
The earlier alpha.13 implementation is `9f89830be760338c74a2f0839a25e2cd1980faad`.
[BUILDING.md](docs/BUILDING.md) explains local Windows builds, hosted CI evidence,
and release signing.

## Repository principle

**Bundle the recipe, not the mods.** Third-party mods are not redistributed here. The
collection pins their versions, sources, hashes, install order, and curated component
choices; every mod retains its own license.

## Layout

```text
app/                    Tauri desktop installer and launcher
engine/                 Rust discovery, planning, acquisition, install, and verification
manifest/               Versioned recipe, sources, order, profiles, and release evidence
docs/curation/components/
                        Human curation decisions and follow-ups per mod
docs/handover.md         Live agent/developer entry point
docs/BUILDING.md         Windows build and release-provenance guide
```

## Development guardrails

- Read applicable `AGENTS.md` (or legacy `CLAUDE.md`) and `docs/handover.md` before changing
  the project.
- Work in an isolated Git worktree.
- Treat the documented reference game and archive directories as read-only.
- Run Cargo from PowerShell on Windows.
- Never hand-edit `manifest/install-order.tsv`; it is captured ground truth.

## License

MIT for CEBG's own recipe and tooling; see [LICENSE](LICENSE). Dependencies retain their
licenses, including the UnRAR extraction restriction in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
Third-party mods keep their own licenses and are not included in the installer bundle.

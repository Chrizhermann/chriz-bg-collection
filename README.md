# Chriz Easy BG (CEBG)

Chriz Easy BG is a one-click Windows installer and launcher for a curated, heavily modded
**Baldur's Gate: Enhanced Edition + Siege of Dragonspear + Baldur's Gate II: Enhanced
Edition** playthrough using EET.

The goal is a polished experience for players, not another expert-only mod manager. CEBG
finds supported clean games, starts with the recommended collection choices, builds a
separate installation, verifies the result, and reopens later as a simple launcher.

## Current status (2026-09-07)

Windows app **0.1.0-alpha.13** is publicly released with recipe **0.1.0-alpha.12** and
434 recommended components. It fixes EET's handling of Windows Documents paths containing
spaces and clarifies notices during quiet installations. The recipe choices and source
pins are unchanged by this app release.

Get the installer and guide from [the collection page](https://bg.chrizfader.org/collection),
or the [versioned alpha.13 release](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.13).
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
alpha.13 is **not Windows Authenticode-signed**, so Windows may show an unknown publisher.
Focused automated checks and package/download/signature verification are recorded in
[the alpha.13 acceptance note](docs/patch-acceptance-alpha13-2026-09-07.md). Those checks do
not establish a new full game-install acceptance or native updater apply/restart acceptance.
See [docs/handover.md](docs/handover.md) for the maintained development status.

## Source and downloads

This repository contains CEBG's installer/engine source and recipe authoring data. The
separate [chriz-easy-bg repository](https://github.com/Chrizhermann/chriz-easy-bg) hosts public
binary releases and the updater feed; its release downloads are not a source checkout.
CEBG's own source and recipe tooling are available under MIT. Third-party dependencies
retain their terms, collected in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
The source and binary distribution repositories have separate roles; existing release
downloads and updater URLs remain the supported player entry points.

App alpha.13's implementation is commit `9f89830be760338c74a2f0839a25e2cd1980faad`.
Commit `bdb040e0d0ab7c63eac260497f3b828116fcb6aa` adds publication evidence only.
[BUILDING.md](docs/BUILDING.md) explains local Windows builds, signing boundaries, and the
remaining source-to-binary CI work.

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

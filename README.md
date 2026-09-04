# Chriz Easy BG (CEBG)

Chriz Easy BG is a one-click Windows installer and launcher for a curated, heavily modded
**Baldur's Gate: Enhanced Edition + Siege of Dragonspear + Baldur's Gate II: Enhanced
Edition** playthrough using EET.

The goal is a polished experience for players, not another expert-only mod manager. CEBG
finds supported clean games, starts with the recommended collection choices, builds a
separate installation, verifies the result, and reopens later as a simple launcher.

## Current status

The `0.1.0-alpha.1` release candidate is functional but not public yet. The current recipe
resolves 35 ordered install runs and 30 pinned artifacts. Automated engine, UI, acquisition,
packaging, and recovery checks pass; a complete installer-driven build from clean BG:EE and
BGII:EE 2.7.3 sources still needs live acceptance before publication.

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

The production updater/signing channel and the final clean-game EET acceptance run remain
release gates. See [docs/handover.md](docs/handover.md) for the live status and exact next
steps.

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
```

## Development guardrails

- Read `AGENTS.md` and `docs/handover.md` before changing the project.
- Work in an isolated Git worktree.
- Treat the documented reference game and archive directories as read-only.
- Run Cargo from PowerShell on Windows.
- Never hand-edit `manifest/install-order.tsv`; it is captured ground truth.

## License

MIT for this repository's recipe and tooling. Third-party mods keep their own licenses and
are not included.

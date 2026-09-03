# BG Radar Overlay integration design

**Status:** Approved for the installer roadmap on 2026-09-03. It must not delay the
first installer-built local test copy, but it is part of the public-alpha update UX.

## Outcome

Treat [BG Radar Overlay](https://github.com/tapahob/BG2RadarOverlay) as an optional
managed companion utility, not as a WeiDU mod and not as part of the reproducible EET
component order. The installer checks for its newest stable GitHub release once per app
session and whenever the player explicitly checks for updates. It never installs or
launches Radar without an explicit player action.

## Approaches considered

1. **Only ship a collection-pinned version.** This is fully reproducible, but it does
   not meet the requirement to offer a newly published Radar release immediately.
2. **Blindly install GitHub's latest asset.** This is convenient, but silently replacing
   an executable that the collection has not reviewed is too much trust for a general
   audience.
3. **Offer latest upstream and distinguish its trust state.** This is the selected
   approach. The app discovers the newest stable upstream release, labels whether it is
   collection-tested, and requires confirmation before download or replacement.

## Source and update behavior

- Query only `tapahob/BG2RadarOverlay` through GitHub's stable `releases/latest`
  endpoint. Ignore drafts and prereleases.
- Run one asynchronous check after app startup. A manual **Check for updates** action
  bypasses the once-per-session throttle.
- Cache GitHub's ETag and last successful result. Offline, rate-limited, malformed, or
  unavailable responses are nonfatal and leave the last known state visible.
- Select exactly one regular asset named `BG.Radar.Overlay.7z`. A missing or ambiguous
  asset fails closed instead of guessing.
- Show publisher, tag, publication date, release-page link, and one of two trust labels:
  **Collection-tested** when signed recipe metadata names the same release digest, or
  **Latest upstream — not yet collection-tested** otherwise.
- Download only after explicit confirmation. Record the resolved tag, immutable release
  and asset URLs, byte length, and calculated SHA-256 in the managed utility receipt.
  GitHub HTTPS plus a post-download digest does not turn an unreviewed release into a
  collection-signed artifact; the UI must preserve that distinction.

At design time, the newest stable release is `2.1.0.0` (2026-09-03). It exposes one
68,124,835-byte asset named `BG.Radar.Overlay.7z`. This observation is evidence for the
design, not a permanent version pin.

## Installation boundary

Install each managed copy's Radar payload beneath
`<managed game root>/Tools/BG Radar Overlay/`. The upstream utility auto-detects the
running game, so scattering files through the game root is unnecessary. Never write to
the store-owned source games or the protected `C:\Games` reference installation.

Validate the 7z central directory before publication with the same principles as the
ZIP extractor: bounded entry count, depth, per-entry and aggregate expanded sizes,
compression ratio, no absolute/parent/device-name paths, no links or reparse points, and
an exact required `BG Radar Overlay.exe` payload. Extract into an owned temporary
directory and publish by atomic directory replacement. Preserve only the existing
`config.cfg`; do not carry arbitrary stale binaries forward.

Refuse installation or replacement while the managed game, InfinityLoader, Radar, or an
installer build for that target is running. A failed download or extraction leaves the
previous working Radar directory untouched. Never add antivirus exclusions; the UI may
link to upstream's false-positive guidance.

## UI and scope

Radar appears as a fourth Updates target beside the application, recipe, and managed
game copies. Actions are **Install Radar**, **Update Radar**, and eventually **Launch
Radar**. It remains optional and does not affect whether a game installation is healthy,
resumable, or playable.

The first local alpha milestone remains full recipe coverage, the real Tauri bridge, and
one installer-built disposable game. Radar integration follows that milestone and must
pass its own metadata, 7z safety, atomic-upgrade, process-lock, offline, and UI tests
before it can join a public alpha.

# Chriz Easy BG 0.1.0-alpha.15

## Fixed

- Updated the collection to alpha.13 with BG Rebalance v0.3.2. This fixes the
  Tempus Holy Power component failing in installations without Spell Revisions,
  including the Improved Haste resources produced by SCS.
- The recommended setup remains 434 components. No other mod pins or default
  choices changed.

## Existing installations

This app update supplies the corrected collection for new installations. It does
not modify an existing game or save. Completed games do not need an urgent rebuild.

If an earlier installation stopped at BG Rebalance component 401, keep its folder
and export diagnostics. A supervised repair may preserve the completed mods; this
release does not enable an unverified automatic Resume for that failure.

## Alpha notes

Windows x64; clean supported BG:EE + Siege of Dragonspear and BG2:EE sources are
required. The installer obtains mods from their declared sources rather than
redistributing them. Evandra still requires the verified manual-download step.

CEBG updates are authenticated with the app's updater signature. The Windows setup
does not yet have an Authenticode publisher signature, so Windows may still warn.

Build source: [255be75](https://github.com/Chrizhermann/chriz-bg-collection/commit/255be75aa232f3af5ad1e1969ccba313fffcccce).
The signed setup passed real-Tauri download/signature verification and tamper
rejection. Windows Defender's scan reported no threats; that is not a guarantee.

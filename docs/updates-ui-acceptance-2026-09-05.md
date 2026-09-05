# Update-notification visual acceptance

Scope: Christopher requested a clear notification for CEBG itself as well as collection
updates, plus a visual mock check. This is a focused UX check, not a whole-app audit.
Existing CEBG palette/layout were retained; no native updater or game files were changed.

## Changes

- Header **New** badge replaces the small dot. Its hover/keyboard-focus tooltip names
  CEBG app, collection and/or Radar updates. Escape dismisses it. No duplicate native popup.
- App row says **Update CEBG app** and explains that installer/launcher updates do not
  alter installed game files. Versions open collapsed changelogs.
- Collection version is labeled **Included in CEBG**, not incorrectly presented as the
  selected game's installed version. The managed-install list retains individual versions.
- The actual `requires-app` state explains that the new setup comes with the app update.
- After updating the app, a current bundled recipe plus an older managed installation
  still produces a collection notification and **Create updated installation** action.
  That action enters new-install setup directly; it does not call the unsupported separate
  recipe-activation command. Existing games/saves remain unchanged.
- Unavailable/offline/stale version fields and a merely uninstalled Radar do not produce
  false New badges. Authoritative managed-copy update state still counts separately.

## Evidence

The real frontend was mounted in local Vite with browser-only FixtureBackend responses.
No simulated state was added to production entrypoints or the native backend. Tested:

- App-only, bundled-collection-only, app-plus-required-collection, independent future
  next-playthrough collection, Radar-only, current and offline states.
- One correct badge/tooltip per state; corresponding action labels and save-safe notes.
- Hover/focus tooltip; Escape dismissal; keyboard changelog open/close; mock app-update
  callback; current-bundle new-install action without recipe activation; install details,
  Check for updates and Back to launcher.
- 1920x1080, 1366x768, 1160x720 and 768x1024: no horizontal or vertical main overflow.
  375x812 and 320x568 use normal vertical scrolling without horizontal overflow. A narrow
  header wrapping regression was fixed and visually rechecked at 320px.
- Browser console: zero warnings/errors. Typecheck, all 80 frontend tests and production
  Vite build passed. The first tooltip harness incorrectly expected opacity zero after
  Escape; inspection showed the correct `hidden`/display-none state and the corrected
  visibility assertion passed. This was a test expectation, not an application failure.

Screenshots are in ignored `target/updates-ui-20260905/`, especially
`cebg-updates-tooltip.png`, `cebg-updates-collection-only.png`,
`cebg-launcher-update-badge.png` and `cebg-updates-both-320-fixed.png`.

These changes are in source and the web build, **not in the previously installed signed
alpha.8 package**. Include them in the next distinctly versioned signed candidate. Mock
update clicks do not prove real updater apply/restart; that separate acceptance remains.

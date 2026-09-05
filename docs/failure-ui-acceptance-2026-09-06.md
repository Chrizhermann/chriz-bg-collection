# Failure detail and truthful progress: bounded acceptance

Source-only improvement while the modpack owner works on r5's 170/192 failures.
The failed installation, its recipe/receipt and the stream game were not changed.

## Changes

- The failure screen now retains the terminal report's actual reason, including
  when a generic command error arrives afterward or no separate command error
  exists. The fresh-copy warning and diagnostics button remain visible; Retry
  remains unavailable for sealed failures.
- Fresh-copy guidance says to resolve the reported problem before starting again,
  rather than encouraging an immediate repeat of an unchanged broken install.
- For a non-prefix partial mod run, the engine adds the exact run/mod, recorded
  component count and requested component numbers not recorded as installed.
  This is diagnostic context, not a change to the reconciliation verdict or a
  claim that absent log rows explain why a component did not install.
- Context is added only for an intact prior active-entry sequence and an ordered
  subset of the exact requested TP2/language/components, without new uninstall
  markers. Unrelated stack changes retain the original generic diagnostic.
- Empty or all-pending phase displays no longer announce "All phases complete".
  A nonempty set of completed phases is required for that announcement.

## Evidence

- Regressions reproduced before changes: missing specific failure text; false
  completion announcements for empty/pending phase lists. Test-environment setup
  was corrected before verifying the latter's actual failing assertions.
- **91 frontend tests**, TypeScript check and production web build passed.
- **9 focused engine CLI unit tests** passed, including r5-shaped 16 requested /
  14 recorded components with 170/192 missing and unrelated-mod non-attribution.
- Focused engine Clippy with warnings denied passed; workspace formatting and
  `git diff --check` passed. No repeated full workspace test suite was needed.
- Headless Edge rendered a representative partial-mod failure at 1920x1080,
  1366x768, 768x1024, 375x812 and 320x568. No horizontal overflow; desktop/tablet
  did not need vertical scrolling, small screens scrolled normally.
- Keyboard activation of Export diagnostics / Start new installation reached
  the expected fixture callbacks; technical-log expand/collapse and auto-scroll
  toggles worked. Unsafe Retry was absent; no browser runtime errors.
- Local screenshots: `target/failure-screen-1366.png` (visually inspected) and
  `target/failure-screen-320.png`. Harness: `target/failure-screen-visual.mjs`.

This is a focused failure-screen check, not the exhaustive AI UX skill audit,
native installer acceptance, an actual new installation, or a successful modpack
fix. The screenshots use a display fixture, not r5's running UI. Installed signed
alpha.8 is unchanged; include this source slice in the next distinct package.
Safe pause/app-close, reopening reconstructed progress, and full acceptance
remain the next documented recovery work.

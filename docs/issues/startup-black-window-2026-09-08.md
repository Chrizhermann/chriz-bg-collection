# Visible startup while detecting games

Status: implemented in source; not yet packaged or published. App alpha.15 and
collection alpha.13 remain the released versions. No recipe/pin changes.

## Cause and change

Dabbi reported a mostly black window for 30–60 seconds while CEBG detected games.
`initialize()` awaited bootstrap, the installation registry, game discovery,
destination checks and recipe evaluation before the first render. Native discovery
already runs on a background worker; the missing piece was frontend feedback.

- The HTML now contains a styled loading fallback before JavaScript runs.
- Startup paints immediately, then shows distinct game-detection and setup messages.
- A small brass spinner follows the existing CEBG palette and motion preference.
- Existing-install startup still opens the launcher without scanning source games.
- Startup failures replace the spinner with error details and **Try again**.
- No timeouts, validation, source checks or installation behavior were relaxed.

## Acceptance

- Regression tests first reproduced the empty initial render and missing failure UI.
- Frontend suite, TypeScript and Vite production build pass (146 tests).
- Isolated Chromium preview held game discovery pending: readable status/spinner,
  no page or horizontal overflow at 1160×760 and 640×520, reduced-motion animation
  disabled, and normal install screen after releasing discovery.
- Loading screen DOM accessibility check passes; retry and delayed setup covered.
- This is simulated slow-backend/browser evidence, not a new native installer
  package or a timing measurement on the reporter's disks. No full game install
  was needed or started for this frontend-only patch.

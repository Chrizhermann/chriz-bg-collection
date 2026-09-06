# Launcher friction patch — 2026-09-07

Christopher approved continuing the small installer improvements while his alpha.13
test installation runs. This work does not change its process, game files or frozen
recipe. Public-source/signing audit remains a separate handoff. No recipe additions,
code-signing enrollment, mandatory elevation or release publication is part of this
source patch.

## Scope

1. Distinguish the CEBG app shortcut from each managed game's Play shortcut. Preserve
   existing foreign links and support multiple installations with the same name.
   Shortcut failure remains optional: keep Play available and expose the native cause.
2. Remember unfinished setup choices locally across app restarts: customization,
   source paths, install name/location and shortcut preference. Restore preferences,
   not trust: re-inspect sources/location and evaluate the current recipe before a
   new explicit Install action. Never persist review tokens or running install state
   as setup preferences. Existing managed installations keep launcher precedence.
3. Make exact startup errors available on the screen itself, including the affected
   folder and Windows reason. Do not tell users to run everything as administrator.

## Evidence and boundaries

- The generated current-user NSIS setup creates `Chriz Easy BG.lnk` for the app.
  The old default managed-game shortcut used the same name; its ownership check
  correctly refused to replace the app's unrelated link. This is an independently
  reproduced code-path defect, not proof of the cause of Jester's specific report.
- Dalswip reported an immediate folder-creation failure but has not supplied the
  exact path or app version. Alpha.12 already fixes missing destination ancestors;
  genuine denied access and other path restrictions still need the actual error.
- No fresh full mod install is necessary to exercise these changes. Use disposable
  shortcut fixtures and frontend failure/reopen tests; retain tonight's full run as
  the separate real-install acceptance.
- Preferences are discarded with an explanation when their recipe/profile is no
  longer compatible. They never authorize resuming a run; that remains the native
  registry/receipt workflow. A missing saved source must not silently select another.

## Acceptance / release follow-up

Source implementation is complete. Windows shortcut tests: 7 passed, including a
real COM app-link fixture and a reparse fixture that ran without skipping. Existing
native command contracts: 39 passed. `npm run check`: TypeScript, all 139 frontend
tests across 15 files and the production Vite build passed. New error-feedback
regressions failed before their hooks and now pass; draft parsing and controller
remount tests cover preferences, fresh inspections, failure retention and cleanup.

This is not native installed-app close/reopen acceptance. Packaging remains later;
the public app is still alpha.13 and collection alpha.12. No recipe/default change.

During verification Christopher's alpha.13 run stopped at EET's pre-invocation
process check. See [the separate incident](../issues/alpha13-eet-pre-spawn-2026-09-07.md).
That recovery issue takes priority over further cosmetic notice refinements.

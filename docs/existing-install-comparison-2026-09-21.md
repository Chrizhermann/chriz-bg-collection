# Existing-install updates: first implementation slice

September 21, 2026. Implemented in the working tree, not packaged or published.
This advances the [parallel preparation](plans/2026-09-20-existing-install-upgrades-preparation.md)
without claiming that a comparison alone delivers in-place mod upgrades.

**Later the same day:** the [BuffBot Apply/Undo/Restore route](existing-install-upgrades-2026-09-21.md)
was implemented on top of this comparison. The evidence and next-work list below
retain the earlier groundwork checkpoint; use the linked update record for current
scope and acceptance status.

## What works in source

- Updates has an explicit **Check installed mods** action for a registered install.
  Rendering the page does not start a scan. Changing installations discards stale
  results and errors; technical failure details remain available in a disclosure.
- The comparison starts from that installation's frozen recipe, complete saved
  choices and typed settings. Previously disabled options remain disabled. New
  optional/default options stay off. New mandatory choices, missing IDs, changed
  input meaning and changed dependencies require an explicit decision.
- The native command accepts an installation ID, not an arbitrary path. It takes
  the target lock, verifies receipt/session identities, reads completed state and
  checks both active WeiDU logs against recorded history. Failed/incomplete copies,
  modified receipts, manual log changes, unknown history and an older bundled
  collection are not silently treated as updateable installations.
- Verified supervised-recovery artifact replacements are used for the affected
  runs. The original recipe is not rewritten to disguise repairs. The existing
  narrow description patch remains separately recorded, not a whole-mod upgrade.
- Only selected installed runs are compared. Payload, archive layout, component
  set/order, language, arguments, prompts and tool settings matter; a version label
  alone does not establish a change. Changed runs are **unassessed** or require an
  explicit migration. No save-compatibility promise or Apply token is produced.
- The page explicitly says that this is a comparison and no game files changed.
  Existing app-update and targeted-patch flows remain separate.

Main implementation: `engine/src/installed_updates.rs`,
`engine/src/installed_updates/selection.rs`, `engine/src/cli/installed_update.rs`,
`app/src-tauri/src/installed_update_bridge.rs`, and
`app/src/components/installed-update-plan.ts`, plus command/controller wiring.

## Verification and limits

- 12 selection-preservation tests and 8 selected-run comparison tests pass.
- 8 boundary tests pass: ordinary completed fixture, unknown schema, receipt
  tampering, BG1/BG2 log drift, unknown history, incomplete receipt and older target.
  Every boundary test snapshots all fixture files/directories before and after
  comparison and verifies no changes. These use a valid recipe with an empty
  selected plan; real selected-run differences are covered separately above.
- All 194 frontend tests pass, including 15 new comparison/controller tests;
  frontend typecheck passes.
- Native app test targets compile. Engine/app Clippy for library and test targets
  passes with warnings denied. Targeted formatting and diff whitespace checks pass.

This is automated/source evidence only. No app/browser/game was launched; no real
installation, profile or save was modified. No download, full installation,
upgrade/rollback rehearsal, packaging or publication was performed in this slice.
Full recovered-install and already-patched-install boundary fixtures are still
needed; a pure comparison test covers recovery identity, not that complete path.
The narrow patch reader still supports only its existing history contract; unknown
or multiple unsupported histories remain blocked rather than guessed.

## BuffBot: bounded first whole-mod upgrade candidate

The independent source assessment checked owner tags in `C:/src/private/bg-eeex-buffbot`:

| From | To |
|---|---|
| `v1.8.3-alpha`, commit `2c3c7e9de49d95e0118d3396a9f90fd341e7bebf` | `v1.8.4-alpha`, commit `9ed6ba0736bdae365b33d473b5fc2c03c9b247db` |

Pinned archive identities are already in
`manifest/artifacts/buffbot-1.8.3-alpha.toml` and `buffbot-1.8.4-alpha.toml`.
They were not downloaded again in this turn. The production source diff is a small
item-identification guard plus version markers, with no added/removed source files.
Persistence, innate generation, UI/bootstrap and language catalogs are unchanged
between these tags. That supports this candidate; it is not a live acceptance test.

**Why a raw Lua copy is not the whole-mod upgrade:** the owner documents upgrading
both components together. Its installer also handles eight innate text references,
localization, loader configuration and WeiDU uninstall state. The BG-modding
reference confirms that failed reinstall rollback need not restore the old mod,
and that `--safe-exit` cannot be combined with explicit force-uninstall. Snapshot
and restore the transaction rather than assuming WeiDU restores the previous install.

Bound the pilot to exact old components `[1, 0]`, mod language 0, as the final two
active rows; retain the recorded game language and check loader capability/state.
No later mods, manual changes or unsupported history. The existing Combined test
copy has ten later components and is **not eligible** for this exact-tail pilot.
Do not uninstall those later components just to make it qualify.

The next adapter must cover:

- 33 installer-produced override files, selected catalog/strref pair and the
  selected TLK(s); preserve existing string references and verify actual results.
- Conditional root `lua51.dll`, `LuaProvider.dll`, `InfinityLoader.ini` changes;
  complete BuffBot component 0/1 uninstall backups, old/new staged source and logs.
- Exported presets, profile settings and saves: preserve, do not migrate automatically.
- 51 runtime-generated `BFBT*.SPL` files and generated background MOS files.
  `BFBOTBG.MOS` is both shipped and regenerated during normal play: archive-byte
  equality is not a valid universal drift test, and Undo must account for that.

Owner source pointers: `buffbot/setup-buffbot.tp2` (loader, strrefs, copy list),
`buffbot/BfBotInn.lua` (generated innates), `buffbot/BfBotUI.lua` (generated MOS),
`README.md` (both-component upgrade/uninstall policy). No owner files were edited.

## Next work, in order

1. Add durable whole-mod update history alongside the immutable original receipt:
   exact before/after identities, preserved choices, resource snapshots, states for
   interrupted work and explicit restore. Keep old description-patch receipts valid.
2. Implement the exact BuffBot transition with complete preflight/backup/restore,
   not a generic mid-stack reinstall. Recheck conditions under the target lock.
   Application and save compatibility remain separate judgments.
3. Wire an honest supported upgrade action into this comparison. Keep unsupported
   changes visible, and never label a partly updated installation as the full new
   collection. Extend history/adapter fixture tests, including interruption recovery.
4. Arrange the focused disposable upgrade and restore test **with Christopher**.
   No unattended app/game acceptance; no whole-stack reinstall needed for this pilot.
   Then prepare a current package and release only on the authorized release path.

The September 20 signed alpha.19 setup predates this code. Public alpha.18 and its
feed are unchanged. UI alternatives/manual breakpoints remain parked, with their
plans preserved; this batch does not change curation, mod pins, versions or order.

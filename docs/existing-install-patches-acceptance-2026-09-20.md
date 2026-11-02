# Existing-install patches: first Updates implementation

Status: **implemented and tested in source; not publicly packaged or released**.
Public app alpha.18 and collection alpha.16 remain unchanged. This follows the
[user-accepted pilot](hotpatch-pilot-acceptance-2026-09-20.md), not another live-game
experiment. No RC, stream installation, original profile or save was changed in
this implementation turn. No full mod installation was repeated.

## Player flow and scope

Updates now has **Fixes for your current game**. Select a registered, completed
installation and explicitly check it. Rendering Updates does not scan game trees.
The first supported fix is `artisan-campaign-description-links-1.0`: use existing
installation-local KITLIST HELP references for the selected Artisan kit descriptions
in the three campaign class tables. Only recorded English Artisan `chriz-v1.3.0`
components are eligible. Newer/unrecognised versions are not silently accepted.

This does **not** install new kit mechanics, rewrite dialog.tlk, change saved
characters or upgrade the whole mod/collection. Absent selections are not added.
Unsupported inputs have no Apply action; already-correct tables need no execution.
The base collection version and individual applied fix IDs are displayed separately.
Other updates still use the existing separate-new-installation route.

- Affected-file backups are mandatory; a full independent playable-game-folder
  backup is recommended and selected by default. This excludes the separate BG1
  preparation tree and download cache. A profile/save snapshot is optional.
- Backup sizes, free space and location are shown. Unavailable profile-size checks
  disable that snapshot option instead of claiming the folder is empty.
- Apply, Undo and interrupted-patch Restore are separate, guarded actions. The game
  must be closed. CEBG cannot prevent launching an EXE outside its launcher.
- A failed/interrupted patch blocks CEBG launch until restoration succeeds.
  My installs shows recovery guidance and disables Play without calling the folder
  missing. Unexpected external writes or corrupted backup evidence stop automatic
  restoration; user-added files are not deleted to force recovery through.
- Explicit checks add fix/recovery notices to the existing Updates badge without
  removing app, collection or Radar notifications. No heavy startup scan was added.

## Native implementation boundaries

The separately signed `patches/catalog.json` references the existing
`artisan-accepted-update` change ID. Its bytes, signature and public key are bundled
at compile time; there is **no independent online patch-catalog feed yet**. New
definitions currently require an app update. The immutable recipe ledger schema
was not extended and the successful install receipt is never rewritten.

The adapter remains in its owning Artisan repository at commit
`d16d35ba29c68aa18025a1c5323a8973986a97d9`. CEBG downloads only its two hash-pinned
files and verified official WeiDU 249 through the existing artifact cache. Public
raw files use LF; these pins differ from the pilot's local CRLF files, with content
equivalence checked. No third-party mod source is bundled in this repository.

The production transaction uses the existing hidden process runner, installation
lock, closed-game/process checks and exclusive file probes. It rejects unsafe
roots, path links and hardlinked write targets. Before/after hashes protect all
detected TLKs, KITLIST, engine identity, KEY and executable; semantic table checks
allow only planned description cells. Full-tree metadata checks reject undeclared
writes. The original active component sequence must remain unchanged, followed by
one patch row; regenerated WeiDU comments are not component identity changes.

Durable evidence lives under `.chriz/patches/cebg-v1/<transaction>/`; independent
optional backups live in app data under `patch-backups/<install-id>/<transaction>/`.
Prepared, applied, restoring and restored records are separate. An unpublished
preparation cannot have touched the game and does not block launch. Interrupted
Undo retains its own baseline so unrelated files created during normal play survive
restoration. Cleanup is limited to exact known adapter files, not a folder wildcard.

Launcher consistency checks accept only a verified patch suffix in addition to the
immutable base receipt. Unknown/unreceipted extra components remain discrepancies.
Diagnostics exports include allowlisted, redacted patch records and captured logs;
they exclude backup content, mod payloads, executables and saves.

## Verification

- Engine patch tests: **24 passed**; opt-in production-download test separately
  **passed**. The latter downloaded the actual pinned files and WeiDU, backed up a
  disposable fake game and profile, applied through the production runner, checked
  the receipt suffix, repeated without another transaction, and undid the patch.
  Original resource, TLK, receipt and synthetic save bytes were preserved/restored.
- Recovery fixtures cover a missing table, truncated configuration, interrupted
  preparation, interrupted Undo, stale review tokens, changed protected files,
  corrupted backup/history and unknown files under the adapter directory.
- Native unit tests: **15 passed** including signed-catalog and consistency checks.
- Diagnostics suite: **15 passed**, including new bounded patch-evidence export,
  redaction/exclusion and path/size rejection cases.
- Frontend: **179 tests**, typecheck and production web build passed.
- Bounded isolated headless mock: four patch states at 1920x1080, 1366x768,
  768x1024, 375x812 and 320x568; 20 renders without horizontal overflow or page
  errors. Check/backup choices/Apply/Undo/Restore callbacks were exercised. Screenshots
  of desktop and narrow layouts were inspected. This is not native Tauri UI acceptance.

Temporary synthetic game/profile/download/backup trees were removed by the test
fixtures. The original full RC backup is deliberately retained for the user.

## Remaining delivery work

The broader `cargo test -p chriz-bg-app --tests --no-fail-fast` run found a
pre-existing **packaging blocker**: 36 command-contract tests passed and four
failed on the selectable `manifest/` public-alpha recipe. Its
`releases/v0.1.0-alpha.1/acceptance.toml` lacks accepted static-evidence entries for
`srcb-rr-compat-bg2`, `chriz-bg-modpack-pre-continuity-bg2`,
`chriz-bg-modpack-continuity-bg2`, `safana-bg2`,
`chriz-bg-modpack-late-companions-bg2` and `spell-rev-lightning-bg2`; the final three
also lack explicit tail approvals in its known-limitations record. The current
curated profile has existing records citing source commit
`5a5ed2b193fad73c542bb5cd90be79618c0dbb51` and
`tools/tests/test_expanded_release_recipe.py`. Reconcile the actual fallback recipe
and its evidence/versioning before packaging; do not substitute easier fixture data,
invent acceptance or bypass validation. No recipe/campaign selection was changed
to hide these failures. Native unit tests and the three ordinary package-config
checks passed; release-artifact-dependent tests remained opt-in/ignored.

Package/version the next app build, exercise the native Updates actions on a
disposable registered fixture, and publish using the ordinary signed release
workflow. Do not advertise these controls as present in public alpha.18.
The already accepted RC needs no repeat gameplay test for this integration.
Additional mechanics/text adapters and broader save-compatibility coverage remain
separate work. See [the next-work index](plans/2026-09-20-next-work-index.md) before
choosing that next batch with Christopher.

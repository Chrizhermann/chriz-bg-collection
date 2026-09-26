# Update an existing installation: first working route

## Latest status — September 22, after user acceptance

The exact **BuffBot 1.8.3-alpha → 1.8.4-alpha** route passed Apply, Christopher's
existing-save gameplay check, and Undo in the local diagnostic package. His No-SR
test installation is now back on **1.8.3-alpha**. Read-only verification confirmed
the original guarded game files and full WeiDU.log were restored; the original receipt and
dialog.tlk are unchanged. Do not repeat this test or reapply automatically.

This accepts this one supported upgrade, not a general mod updater or publication.
Final combined UI packaging and release work remain pending. Broader upgrades,
collection-wide compatibility categories and more hotpatches were deprioritized
on September 22; they are not release gates. The current
[package acceptance record](package-acceptance-alpha19-2026-09-22.md) and
[handover](handover.md) are authoritative.

The remainder is the chronological implementation and earlier acceptance-preparation
record. Its "pending", "not modified" and overnight-authorization statements
describe those earlier checkpoints, not the current result or permission to control
the user's app. Christopher currently operates the app and game himself.

## What the player gets

In Updates, choose an installation and **Check installed mods**. The existing
comparison retains that installation's choices. A separate BuffBot card can offer
**Update BuffBot now** after explicit review of versions, game folder and backups.
It updates the existing folder; it does not start a new collection installation.

- First supported route: BuffBot **1.8.3-alpha → 1.8.4-alpha**, components 1 and 0.
  The visible fix stops unidentified items revealing their names in BuffBot.
- An affected-file backup is mandatory. A full playable-game-folder backup is
  recommended and selected by default. Copying saves is a separate optional choice.
- Original receipts, collection choices, save files and BuffBot preset settings
  are not rewritten. The base collection version stays the same. Successful
  BuffBot updates are recorded separately and included in future comparisons.
- Undo restores affected game files, not progress saved since the update.
  Interrupted updates offer Restore and block CEBG's Play action until resolved.
  This cannot prevent someone launching the game executable outside CEBG.
- Mod update inspection is explicit, not an automatic download/scan on startup.
  A new whole-mod notification policy is not part of this slice.

This is **not Update All**. Other changed mods remain visible but have no new
automatic installation route. Do not call the entire collection upgraded because
BuffBot was upgraded.

## Deliberate first-release boundaries

Only completed, registered CEBG installations with known frozen choices and a
verified current mod list qualify. Supported old resources, source files, English
catalog and uninstall records must match the reviewed release. BuffBot must be
the final mod, in component order 1 then 0; later/manual mods block this route.
English only, no separate female TLK, one supported EEex layout and an already
configured EEex-owned LuaJIT loader. The helper's uninstall record must not own
loader changes. Unknown backup shapes are unsupported, not silently rewritten.

The first upgrade reinstalls only those two BuffBot components using pinned
WeiDU 249's normal forced-install behavior for already installed components,
not overlapping force-install/force-uninstall switches. The behavior was checked
against [WeiDU 249's component dispatcher](https://github.com/WeiDUorg/weidu/blob/v249.00/src/tpwork.ml).
Both old/new release archives are verified before game writes; all 41
source files are checked against release-tree fingerprints. After execution,
installed resources and component order/version are checked. Every existing TLK
entry, including its metadata and text, and all eight BuffBot string references
must be preserved. Safe byte-layout repacking is allowed; changed/appended text
is not accepted by this adapter. No copying another installation's TLK.

## Backups and recovery

- The installation lock and closed-game/other-writer checks cover mutations.
  A changed review token requires checking again.
- Affected-file backups and original inventories are verified before a durable
  preparation record is published. Only then can source files or WeiDU change
  the game. No blind rerun after a partial reinstall.
- State lives under `.chriz/mod-upgrades/v1/<transaction>/`. Affected backups are
  in that transaction's `affected/`. Optional full game/profile copies are under
  app-data `mod-upgrade-backups/<install-id>/<transaction>/`.
- Prepared, applied, restoring and restored records are linked to the unchanged
  base receipt and preceding transaction. Latest-only Undo checks current owned
  files; automatic Restore refuses unrelated changes or damaged backups.
- Restore supports missing backed-up files, including the TLK, and an interrupted
  restoration. It removes only recorded newly created adapter-owned paths.
  Unknown/player files are never recursively deleted. Backup folders are retained.
- Known runtime-generated BuffBot resources may change while playing without
  invalidating Undo; preset JSON and saves are not restored or overwritten.
- An existing Artisan description fix must be undone before this tail-only
  BuffBot route. A new description fix may follow BuffBot; its verified log link
  is understood. Undo that later fix first. This is a specific supported ordering,
  not a general multi-mod dependency/reinstall algorithm.
- Diagnostics export adds update records and captured logs, with existing path
  and secret redaction. It does not include backup contents, mods or saves.

## Evidence and the remaining acceptance step

Automated tests cover the adapter, transaction success/Undo, interruption and
restore, missing TLK, corrupt backups, manual drift, preserved settings/saves,
history, selection comparison, native command locks and frontend confirmation.
The original evidence claimed the two published ZIPs matched both archive and
tree hashes. The September 22 packaged check disproved the tree-hash claim:
the ZIP hashes were correct, but both expected tree digests were wrong. See the
correction and real-package regression below; no game write caused this failure.

Recorded checks for this implementation:

- Engine library: 124 passed, three pre-existing opt-in live/download checks
  remain ignored; 23 passing tests cover the new adapter/transaction module.
- Installed-update boundary: 12 passed within that library run, including four
  new selected-BuffBot/effective-history cases (not just an empty selected plan).
- Native app library: 19 passed; native command contracts: 40 passed.
- Diagnostics: 16 passed; selection/comparison integration tests: 12 + 8 passed.
- Frontend: 211 passed; typecheck and production asset build passed.
- Engine/native Clippy with warnings denied and diff whitespace checks passed.

The BG-modding guidance informed the TLK preservation, installer-owned metadata
checks and independent restoration design; WeiDU failure alone is not treated as
proof of an intact old installation.

**Still untested:** real WeiDU execution through this new whole-mod route,
packaged frontend/backend operation and loaded-save gameplay after the update.
Synthetic transaction tests do not prove those. Existing description-patch
acceptance is not acceptance of this new feature.

Next, with Christopher present and an explicitly approved disposable completed
CEBG installation: check eligibility, review/apply, reopen CEBG, confirm BuffBot's
new version and unchanged other mods, load a save and check unidentified-item
names/presets, then Undo and verify the previous version. No new full-collection
installation is required merely to run this acceptance, if a qualifying copy exists.
Do not run this interactive acceptance unattended or publish before approval.

### September 22 overnight authorization and preview blocker

Christopher subsequently authorized taking over the test while he sleeps. This
supersedes the earlier user-present/clicking restriction for the specific No-SR
test installation, not the stream install or public release. Windows UI software
installation still requires action-time confirmation.

The existing packaged app was reopened and correctly recognized No-SR. Its
collection comparison worked, but BuffBot preview stopped before any mutation.
Both expected source fingerprints rejected the unmodified official archives.
Correct values, using lowercase ordinal paths, LF delimiters, little-endian u64
lengths and raw bytes (41 files each), are:

| BuffBot | Archive SHA-256 | Source-tree SHA-256 |
| --- | --- | --- |
| 1.8.3-alpha | `fe6e0b4cd393a608d8c3df75f26993106f3550e954030a7b66dbfd28956820a7` | `0d3a2210e1787d26493eaa75a5099a7a1d4f6037bff9ee028e8b70a74bc76710` |
| 1.8.4-alpha | `1081abc59511263570603560059ef361f45dd3288933f7faa26166ce0e805ced` | `24091c2b6b42f1eace4b49dcbb8fc0c8237c4b78ecf6649a850b068c4300c490` |

The extracted files and direct ZIP reads independently produce these values.
The new `published_archives_match_reviewed_sources` test reproduced the failure
before correction. All 24 update-module tests pass including this real-archive
check and rejection of changed/extra files. To repeat without downloads, set
`CEBG_BUFFBOT_OLD_ARCHIVE` / `CEBG_BUFFBOT_NEW_ARCHIVE` to the exact local ZIPs and
run `cargo test -p chriz-bg-engine --lib mod_upgrades -- --include-ignored`.

Six native bridge tests also pass for distinct read-only check error wording.
It no longer suggests a new recovery obligation when only inspection failed;
earlier recovery warnings remain applicable. The installed app still has the old
wording/digests until rebuilt and installed. No Apply, Undo or game launch occurred.

The corrected engine's complete `inspect` call against registered No-SR returns
`available`, with 11,898,950,716 full-backup bytes and 1,456,735 save/profile bytes.
The probe held the normal installation lock and used the installed recipe plus
verified cache; it did not execute WeiDU or apply/restore files. This proves
eligibility after the correction, not packaged Apply or loaded-save acceptance.

The corrected signed candidate subsequently built successfully (job
`f78462ce-7d67-4d7d-8fb3-20dc1d3aef79`, exit 0). Its real-package, native, UI,
Clippy and package-signature checks passed; copies are in Downloads. The installed
app was not replaced: the next Windows installation action awaits the computer-use
skill's action-time confirmation. Apply/Undo and gameplay remain untested.

## September 22 acceptance preparation

Resumed after the interval-save fix. A fresh signed alpha.19 package was built
from current source (including the update route and interval-save fix). No app,
game, updater apply, or publication was launched. Background job results:
`target/work/alpha19-update-candidate-sep22/`; job
`1b37d06d-079f-43b1-8044-b36227ffca79`. Final status/log verified: exit 0,
211 frontend tests and four package/signature tests passed. This does not establish
interactive update acceptance.

Read-only checks found a promising existing registered test target:
`C:\Users\chris\Games\ChrizEasyBG-No-SR-Test`, install ID
`install-c9dc8c3e0e0b0a3476e0`. Its completed supervised-recovery receipt matches
the registry SHA-256, and BuffBot 1.8.3 components 1 then 0 are its last two active
WeiDU rows. This is not yet full adapter eligibility: its normal checks for source
bytes, English strings, loader state and closed-game conditions still apply.

The production registry is under
`%LOCALAPPDATA%\Chriz BG Collection\managed-installs`, not the Tauri cache folder.
Do not manually clone/relabel this target: its receipt and registry bind to the
original path. Asked Christopher whether to use this existing TEST installation
with him present and a full backup; await his answer before interactive testing.
Stream/current-playthrough folders remain untouched. If approved, use the Updates
check/review/apply, reopen, load-save and Undo sequence above; no fresh full-stack
installation is needed if this candidate passes eligibility.

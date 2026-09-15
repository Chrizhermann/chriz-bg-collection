# Documents-folder robustness: viewer staging failure

Status: bounded source check and requested product behavior, 2026-09-16.
No implementation, reproduction, user-file changes or release in this pass.

## Second diagnostics received September 16

The follow-up ZIP is app alpha.15 / recipe alpha.13, same managed installation
and acquisition history. Xan succeeds on the third user-level attempt after two
HTTP-500 failures. All 34 acquisition records are now available, both Steam source
games identify as 2.7.3.0, and `stage:bg1` fails on its first recorded attempt
after about 1.16 seconds. No mod invocation has started in this ledger.

The reported path is again the generated Documents child, not the Documents parent.
The source code checks the parent's existence before creating the child. A missing
parent alone is therefore not established as this failure's cause. Current generic
errors also cannot distinguish child creation, revalidation or an error masked by
failed reservation cleanup. Do not promise that known-folder creation will fix it.

Next evidence: Windows Security Protection History for a matching CEBG controlled-
folder-access block (and any separate security product), plus whether the reported
Documents location is available. Controlled folder access remains a hypothesis,
not a diagnosis. If a matching block is confirmed and the user trusts the app,
allow only that executable through the specific folder-access control, restart
CEBG and retry the same installation; do not disable protection globally. Keep
all cache/failed-install evidence. No need for another clean Steam installation.

## Report and what is actually known

A viewer reports `stage:bg1: failed`, with `os error 2` naming
`<Documents>/Chriz Easy BG - <installation suffix>`. CEBG is on C:, Steam games
are on D:. App version and diagnostics have been requested but not received.
Different source/app drives are supported; this path concerns save-profile
preparation, not a mod download or evidence of dirty source games.

Current `engine/src/cli.rs` staging reserves a save identity before staging BG1.
`engine/src/stage.rs` already uses Windows `FOLDERID_Documents`, not an assumed
`%USERPROFILE%/Documents` path, and creates its installation-specific child itself.
The known-folder lookup currently passes flags 0 and `propose_save_identity`
requires the parent to exist. Creation/verification failures surface as generic
path/I/O errors; the report does not distinguish those operations.

Do not claim the viewer's root cause is missing Documents, OneDrive, antivirus,
permissions or an installer race. Failure to resolve the known folder itself would
normally produce a different path/message; this report names the child. The
parent-creation gap is real but is not yet an explanation of this report.

Viewer follow-up, September 16: reports app alpha.15 (not yet verified), an existing
local Documents folder on C:, no OneDrive, and Steam games on D:. Also reports
running elevated and disabling Defender/SmartScreen; advise restoring security
protection, not expanding exclusions. Diagnostics still pending. This makes a
missing parent/OneDrive explanation less likely, not a confirmed cause.

Viewer also reports being unable to choose D: because of a game-location warning.
Obtain the exact selected destination and message: source/destination overlap is
forbidden, not sharing a volume. Selecting D: itself or the Steam library parent
can overlap source games; a separate empty sibling folder should not. Clarify app
installation versus the managed game destination. Do not infer a drive-wide bug.
Viewer could not find an error-message knowledge base; add a small troubleshooting
page with confirmed cases and diagnostics instructions, not guessed fixes.

## Requested product contract / next-patch hardening candidate

- Ordinary redirected Documents, including a different drive or OneDrive-backed
  location, should work without asking players to move folders or turn off sync.
- During explicit installation setup, create missing configured Documents through
  Windows' known-folder creation support, then reserve the CEBG-owned child. Keep
  read-only discovery read-only; do not add filesystem writes to generic lookup.
  Do not invent a fallback profile path that the game will not use.
- Preserve ownership checks, existing saves, and protection against unexpected
  directory links. Classify supported cloud/reparse behavior before relaxing any
  blanket check; OneDrive support is not permission to accept arbitrary junctions.
- If the configured location is offline, unwritable, or blocked by security policy,
  stop with the actual operation, location and useful remedy. Do not assume
  "run as administrator" or recommend disabling protection/sync wholesale.
- Distinguish resolve/create/marker-write/revalidate errors in diagnostics and show
  clear save-folder wording in the UI instead of only "could not complete that check".
- After the cause is corrected, offer the existing safe retry path when available;
  retain choices/cache. Do not prescribe a fresh mod installation without evidence.
- Focused fixtures: missing known-folder parent at install time, relocated parent,
  existing owned child retry, unowned-child collision, denied access/unavailable
  location and disappearing child. Test actual OneDrive behavior separately rather
  than calling a folder named OneDrive sufficient coverage. No full mod build needed.

## Viewer next step

Request app version and diagnostics ZIP first. Whether the reported path exists,
Documents is redirected, or security history records a block helps diagnose the
failure; these are questions, not new user setup requirements. Do not manually
create the hashed child (it needs an ownership marker), delete more cache/game
files, or change security settings based only on this message.

## Primary reference

[Microsoft KNOWN_FOLDER_FLAG documentation](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ne-shlobj_core-known_folder_flag):
without DEFAULT_PATH the current redirected path is used; CREATE requests creation
with the known folder's security provisions. It may still fail if creation is not
allowed. This supports a bounded setup fix, not a guarantee for unavailable storage.

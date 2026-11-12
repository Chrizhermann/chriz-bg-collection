# Reporting a problem or feedback from the app

Chriz Easy BG and Chriz Easy IWD let a player report a problem or send feedback without
leaving the app to look for the right place. Christopher approved the design on 2026-11-10.

## What the player sees

**Feedback** in the app header is always there. The same dialog opens with what the app knows
from four more places:

| Where | Button | What the report holds |
| --- | --- | --- |
| Header, every screen | Feedback | Versions; the player chooses "A problem" or "Feedback or an idea" |
| A stopped installation | Report this problem | Headline, phase, step, reason, error code and details |
| The question about a mod part that could not be installed | Report this problem | The question, its step and its details. It answers nothing; the question waits below the dialog |
| "Installation complete" with mods left out or downloads changed | Report this | The summary sentences |
| Troubleshooting on an installation | Report a game problem | Game log on or off, number of crash reports |

The dialog shows the whole report: the player's own text (optional, up to 1200 characters) and
the details the app adds. From there:

- **Open GitHub issue** opens the new-issue page of the product's distribution repository in
  the browser, with title and text filled in. The player reads it there and submits it.
- **Copy text** puts the report on the clipboard; **Open Discord** opens the invitation to
  ChrizFader's Discord (`#mod-feedback`).
- **Save diagnostics ZIP** is the existing Export diagnostics for the installation the report
  is about, with **Show in folder** afterwards. The player attaches the ZIP by hand.

## Rules the code keeps

- **The app sends nothing.** The interface has no network access (content security policy);
  the bridge only opens an address in the player's browser. There is no service of ours, no
  token and no webhook in the app or the repository.
- **Only fixed addresses.** The GitHub repository is `report_repository` of the flavor
  (`app/src-tauri/src/flavor.rs`), the Discord invitation is `CHAT_INVITATION`
  (`app/src-tauri/src/report.rs`). Title and text enter the GitHub address as percent-encoded
  query values and cannot change where the browser goes.
- **Short addresses.** Windows cuts an address off without an error when it is longer than
  about 2,000 characters, so `issue_address` never builds a longer one. A report that does not
  fit ends with a note; the app then puts the whole text on the clipboard and says so.
- **Redacted details.** Everything the app adds goes through
  `bg_engine::diagnostics::redact_for_sharing` before it is shown: the home folder, user names
  as path segments, the profile folder of any user, the computer name and lines with secrets
  are replaced, as in the diagnostics ZIP. If that fails, only the versions are shared. The
  technical log is never part of the text; it is in the ZIP.
- **Never in the way.** The dialog does not pause or answer anything. An installation keeps
  running behind it, and the dialog keeps its text and focus while the screen behind it
  updates. A question that arrives meanwhile waits below it.

## The diagnostics ZIP

- The Save dialog suggests `<product>-diagnostics-<date>-<time>.zip` (UTC).
- Size, estimated on 2026-11-10 from the files the export selects in two finished lab
  installations: about 3 MB for a full Chriz Easy BG installation and about 0.5 MB for Chriz
  Easy IWD. Both fit as an attachment in Discord (10 MB) and in a GitHub issue (25 MB).
- Besides the home folder, the export replaces the Windows user name where it is a path
  segment (a game on `D:\Games\<name>\…`), the folder after any `Users` segment, and the
  computer name as a whole word.
- An installation without a finished attempt (interrupted before its first receipt, or still
  running) exports what it has recorded so far. `START-HERE.txt` says so and gives the reason.
  A log that cannot be read at that moment holds one line saying so.
- Before a managed installation exists (game not found, a download fails before the
  installation starts) there is still no ZIP; the report text carries the error.
- An app that crashes or is removed by security software leaves nothing to report from; the
  player reports from the next start or by hand.

## Outside this repository

- **Issue forms.** `docs/releases/issue-templates/` is the source of
  `.github/ISSUE_TEMPLATE/` in `Chrizhermann/chriz-easy-bg` and `Chrizhermann/chriz-easy-iwd`,
  for players who come to GitHub without the app. `blank_issues_enabled` stays `true`: the
  app's link opens a blank issue with the report filled in (checked signed in on 2026-11-12
  with `https://github.com/<repository>/issues/new?title=Test&body=Test` for both
  repositories: the filled-in blank issue appears directly, not the template chooser).
- **READMEs.** `docs/releases/public-README.md` names the three ways to report. The
  `chriz-easy-iwd` README gets the same sentence with its next release.
- **Discord channel.** Both flavors name `#mod-feedback` (`chatChannel` in
  `app/src/flavors.ts`) and use the server invitation `CHAT_INVITATION`
  (`app/src-tauri/src/report.rs`). Keep `#mod-feedback` public on ChrizFader's Discord with
  message and file-attachment permissions for members.

## Not built

A service of ours that takes reports without a GitHub or Discord account was considered and
left out: it needs an endpoint, storage, limits against abuse and a privacy notice. It is the
next step only if reports visibly fail at the account requirement.

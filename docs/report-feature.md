# Reporting a problem or feedback from the app

Chriz Easy BG and Chriz Easy IWD let a player report a problem or send feedback without
leaving the app to look for the right place, and without needing a GitHub or Discord account.
Christopher approved the initial dialog on 2026-11-10 and the direct 1-click relay on
2026-11-13.

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

The dialog shows the whole report: the player's own text (up to 1,200 characters) and the
redacted details the app adds. Under **Send it**:

- **Send report** (primary, no account needed) sends the report straight from the app through
  `https://bg.chrizfader.org/api/installer-report` to Chriz's private Discord report inbox:
  - **How to reach you (optional)** lets the player leave a Discord name or email (up to 120
    characters) if they want a reply.
  - **Include the diagnostics ZIP for this installation** appears whenever an installation is
    in view; it defaults to checked for a problem report and unchecked for general feedback,
    unless the player toggles it.
- Under **Or post it yourself** (public / fallback options):
  - **Open GitHub issue** opens the new-issue page of the product's distribution repository in
    the browser, with title and text filled in. The player reads it there and submits it.
  - **Copy text** puts the report on the clipboard; **Open Discord** opens the invitation to
    ChrizFader's Discord (`#mod-feedback`).
  - **Save diagnostics ZIP** exports a local copy of the installation's diagnostics ZIP, with
    **Show in folder** afterwards, so the player can inspect it or attach it on GitHub or
    Discord by hand.

## Rules the code keeps

- **Explicit player action only.** Nothing is sent in the background or when opening the
  dialog. Network submission happens only when the player clicks **Send report**.
- **Locked-down webview, fixed endpoints.** The webview keeps its strict Content Security
  Policy with no internet access; only the Rust native bridge (`send_report` in
  `app/src-tauri/src/bridge.rs` -> `engine/src/report_submit.rs`) talks to the hardcoded HTTPS
  endpoint `REPORT_API_BASE` (`https://bg.chrizfader.org/api/installer-report`). No Discord
  webhook URL or secret ever ships in the client or in this repository.
- **Abuse and spam protection.** Before uploading, `engine/src/report_submit.rs` fetches a
  short-lived HMAC-signed challenge from `POST /api/installer-report/challenge` and solves a
  bounded SHA-256 Proof-of-Work puzzle (`difficulty <= 22`, default 18 bits ~100–300 ms). The
  Cloudflare Worker (`bg-site/report-relay.mjs` in `twitch-setup-chriz`) enforces single-use
  challenges, strict multipart/JSON size limits (7.5 MB max ZIP), central-directory and
  local-header allowlist verification of the diagnostics ZIP, keyed HMAC rate limits (per IP,
  per local cache tag, and global), and optional ASN blocking / shadowbanning.
- **Graceful fallback.** When the relay is offline, unreachable, rate-limited, or unconfigured
  (`503`), `send_report` returns an actionable message inside the dialog pointing the player
  to **Open GitHub issue** or **Copy text** + **Open Discord** right below.
- **Fixed browser fallback addresses.** The GitHub repository is `report_repository` of the
  flavor (`app/src-tauri/src/flavor.rs`), and the Discord invitation is `CHAT_INVITATION`
  (`app/src-tauri/src/report.rs`). Title and text enter the GitHub address as percent-encoded
  query values and cannot change where the browser goes. `issue_address` caps URLs at 2,000
  characters so Windows never truncates them; longer reports copy the full text to the
  clipboard automatically.
- **Redacted details and diagnostics.** Everything the app adds goes through
  `bg_engine::diagnostics::redact_for_sharing` before it is shown and again in `send_report`
  before upload: the home folder, user names as path segments, the profile folder of any user,
  the computer name and lines with secrets are replaced, as in the diagnostics ZIP. The
  optional contact field is trimmed to one line (120 characters) without path redaction so a
  player's chosen handle or email stays intact.
- **Never in the way.** The dialog does not pause or answer anything. An installation keeps
  running behind it, and the dialog keeps its text and focus while the screen behind it
  updates. A question that arrives meanwhile waits below it.

## The diagnostics ZIP

- Both manual export and direct attachment name the archive `<product>-diagnostics-<date>-<time>.zip` (UTC).
- Size, estimated on 2026-11-10 from the files the export selects in two finished lab
  installations: about 3 MB for a full Chriz Easy BG installation and about 0.5 MB for Chriz
  Easy IWD. Both fit comfortably within the direct relay cap (7.5 MB), Discord (10 MB), and
  GitHub issues (25 MB).
- Besides the home folder, the export replaces the Windows user name where it is a path
  segment (a game on `D:\Games\<name>\…`), the folder after any `Users` segment, and the
  computer name as a whole word.
- An installation without a finished attempt (interrupted before its first receipt, or still
  running) exports what it has recorded so far. `START-HERE.txt` says so and gives the reason.
  A log that cannot be read at that moment holds one line saying so.
- Before a managed installation exists (game not found, a download fails before the
  installation starts) there is no ZIP; the report text carries the error.

## Outside this repository

- **Cloudflare Worker relay (`Chrizhermann/twitch-setup-chriz`).** `bg-site/report-relay.mjs`
  handles `POST /api/installer-report/challenge` and `POST /api/installer-report` on
  `bg.chrizfader.org`. In Cloudflare (`chrizfader-site-deploy`), set the encrypted secrets
  `DISCORD_REPORT_WEBHOOK_URL` (webhook of a private `#installer-reports` Forum or Text
  channel on ChrizFader's Discord) and `REPORT_SECRET` (`openssl rand -hex 32`); see
  `bg-site/CLOUDFLARE.md`.
- **Issue forms.** `docs/releases/issue-templates/` is the source of
  `.github/ISSUE_TEMPLATE/` in `Chrizhermann/chriz-easy-bg` and `Chrizhermann/chriz-easy-iwd`,
  for players who come to GitHub without the app. `blank_issues_enabled` stays `true`: the
  app's link opens a blank issue with the report filled in (checked signed in on 2026-11-12
  with `https://github.com/<repository>/issues/new?title=Test&body=Test` for both
  repositories: the filled-in blank issue appears directly, not the template chooser).
- **READMEs.** `docs/releases/public-README.md` names **Feedback** in the app alongside
  Discord and GitHub issues.
- **Public Discord channel.** Both flavors name `#mod-feedback` (`chatChannel` in
  `app/src/flavors.ts`) and use the server invitation `CHAT_INVITATION`
  (`app/src-tauri/src/report.rs`). Keep `#mod-feedback` public on ChrizFader's Discord with
  message and file-attachment permissions for members who want to discuss or post by hand.

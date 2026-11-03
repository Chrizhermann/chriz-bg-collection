# Chriz Easy BG 0.1.0-alpha.21

**Published:** app alpha.21 / collection alpha.20 (unchanged).
[Download and release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.0-alpha.21).
Exact package and test results: [alpha.21 package verification](package-acceptance-alpha21-2026-10-14.md).

This update fixes two installer problems that could leave you stuck. The mod collection is
unchanged from alpha.20, and updating CEBG leaves your current games and saves unchanged.

## Installer

- **Damaged downloads and unpacked mods are rebuilt automatically.** A resumed
  installation could stop at a mod with "could not inspect extraction marker … (os
  error 2)" on every retry. This happened when CEBG's download folder in AppData had
  been deleted by hand, or while uninstalling with "Delete the application data":
  Windows cannot remove files with very long paths that way, so damaged leftovers
  stayed behind. CEBG now checks every reused download and unpacked mod before using
  it. A damaged one is set aside and downloaded or unpacked again, and the repair is
  recorded in the installation's diagnostics. You never need to delete these folders
  yourself. A file that another program is only holding or scanning at that moment is
  never treated as damaged: the step stops instead, and **Retry** works once that
  program is done.
- **Installations whose folder was partly deleted by hand can be removed from My
  installs.** They showed "Build unavailable — folder moved or changed", and removing
  them always failed with "The installation could not be removed". When CEBG can no
  longer confirm that a folder is the installation it built, it now offers **Remove
  from list**: the entry leaves My installs, and nothing in the folder is deleted. The
  dialog says why. Delete the folder yourself afterwards if you no longer need it.
  Deleting an intact installation also works now when CEBG's download folder was
  deleted or is on a drive that is not connected.

## If an installation stopped with the error above

Update CEBG, then press **Retry** on the failed step. Nothing needs to be deleted first.
Starting a new installation also works; it repairs the damaged files the same way.

## Important notes

- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows
  may still display an unknown-publisher warning. Download only from the official
  release; do not disable Defender or add blanket exclusions.
- Need help? See the [troubleshooting and backup guide](https://github.com/Chrizhermann/chriz-bg-collection/blob/codex/installer-v0-real-alpha/docs/troubleshooting.md).

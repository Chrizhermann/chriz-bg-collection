# Chriz Easy BG 0.1.5

**Released:** app 0.1.5 / collection 0.1.2, on 2026-11-13. The exact package and its test
results are in the [package verification record](package-acceptance-0.1.5-2026-11-13.md).

0.1.5 makes folder checks, drives, archive picking, hotpatching and path editing more
flexible across different Windows setups, and gives clearer guidance if Windows Security
quarantines `InfinityLoader.exe`. Updating CEBG leaves your current games and saves unchanged.

## Highlights

- **More flexible path, drive, junction & cloud-folder handling.** Parent folder junctions,
  OneDrive/synced folders, cloud-file placeholders, mapped/network-style paths and
  case/trailing-separator spelling differences no longer trip unnecessary path rejections,
  while direct link targets remain blocked safely. Diagnostic ZIP replacement also works
  cleanly on exFAT and FAT32 drives (`os error 1`).
- **Benign OS metadata files ignored in destination checks.** Folders containing only hidden
  Windows or macOS metadata (`desktop.ini`, `Thumbs.db`, `.DS_Store`, `.__*`, `Icon\r`) are
  treated as empty during setup without creating extra subfolders.
- **Broader manual archive file picker.** When manually picking a mod download, the file
  dialog offers `.zip`, `.iemod`, `.exe`, `.rar`, and `.7z` extensions consistently across all
  formats.
- **Smoother typing in Setup & Install location inputs.** Typing a custom install name or
  folder path no longer loses focus or cursor selection while background validation updates.
- **Tolerant hotpatch runtime checks & clearer antivirus recovery.** In-game hotpatch checks
  tolerate normal runtime/save files (`baldur.lua`, `baldur.ini`, `weidu.log`, `override/`,
  etc.), and if Windows Security quarantines `InfinityLoader.exe` on an EEex install, **My
  installs** gives direct Windows Security restore guidance.

## Known issues

- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.5_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.5), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

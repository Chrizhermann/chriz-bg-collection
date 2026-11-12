# Chriz Easy BG 0.1.3

**Released:** app 0.1.3 / collection 0.1.2, on 2026-11-12. The exact package and its test
results are in the [package verification record](package-acceptance-0.1.3-2026-11-12.md).

0.1.3 fixes two setup blockers reported by GOG and external-drive players — GOG's 2.7.3.0
Baldur's Gate II: Enhanced Edition executable (`BaldurII.exe`) and installing onto exFAT or
FAT32 drives — and brings **Main choices** on the install screen, campaign XP caps with
banked XP inside **Challenge Mode**, and in-app **Feedback** and problem reports. Updating
CEBG leaves your current games and saves unchanged.

## Highlights

- **GOG BG2:EE 2.7.3.0 (`BaldurII.exe`) and flexible game detection.** GOG's 2.7.3.0 Windows
  build of Baldur's Gate II: Enhanced Edition ships `BaldurII.exe` instead of `Baldur.exe`.
  CEBG now recognises role-specific and renamed game executables automatically, reads their
  version cleanly, and creates an independent `Baldur.exe` copy inside the managed game
  folder during staging so EEex, WeiDU mods and BG Radar Overlay work out of the box without
  modifying your GOG installation. Older game builds whose `.bif` archive names differ from
  2.7.3.0 (such as 2.6.6.0) are also accepted with a version warning instead of being
  blocked as missing files.
- **exFAT and FAT32 installation drives.** Creating a new installation, writing receipts or
  exporting a diagnostics bundle onto an exFAT or FAT32 drive no longer stops with
  `Incorrect function. (os error 1)` at `.chriz\recipe\payload.zip`. File publication now
  uses atomic no-overwrite moves instead of NTFS-only hard links.
- **Main choices on the install screen.** **Challenge Mode** now has its own switch directly
  on the install screen between **Your installation** and **Recommended setup**, kept in sync
  with its card in **Customize**.
- **Campaign XP caps with banked XP in Challenge Mode.** Switching on **Challenge Mode** now
  also enables Chriz BG Modpack's campaign XP caps (`161,000` in BG1, `500,000` in SoD,
  `2,950,000` in Shadows of Amn, `8,000,000` in Throne of Bhaal) together with its chapter
  caps. Experience earned past a cap is banked and awarded once the next chapter or campaign
  raises the limit.
- **Feedback and problem reports from the app.** Use **Feedback** in the header or **Report a
  problem** on a stopped or partial installation to preview a prefilled report, open it on
  GitHub or copy it for `#mod-feedback` on Discord. Diagnostics bundles now also redact your
  Windows user name and computer name and can be exported before an attempt finishes.

## Known issues

- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.3_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.3), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

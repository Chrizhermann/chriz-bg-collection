# Chriz Easy BG 0.1.4

**Released:** app 0.1.4 / collection 0.1.2, on 2026-11-13. The exact package and its test
results are in the [package verification record](package-acceptance-0.1.4-2026-11-13.md).

0.1.4 redesigns the installer and launcher around the original Baldur's Gate stone-and-brass
aesthetic and makes every screen faster to scan, navigate and launch. Updating CEBG leaves
your current games and saves unchanged.

## Highlights

- **Candlekeep stone, aged brass & multi-weight typography.** Panels, recessed input wells,
  square-cut plaques, custom checkboxes/radios and the forged citadel-gate medallion replace
  generic card styling and synthetic browser bolding (`Alegreya` and `Alegreya Sans` bundled
  locally in multiple weights).
- **Chapter sidebar & clean sticky header in Customize.** **Customize** now pairs a sticky
  left-hand category index with a calm single-column option list, seals mandatory foundation
  entries distinctly from optional checkboxes, and keeps scrolled headings cleanly behind the
  sticky toolbar.
- **Launch altar in My installs.** Completed installations put **Play Chriz Easy BG** and the
  optional Radar switch front and center at the top of the card, with quick folder/shortcut
  actions directly beneath, collapsible reference drawers, and separated diagnostics/removal
  controls.
- **Live build activity, full-width Updates & compact Feedback.** Running installations show
  their latest step line above the collapsed technical log, **Updates** uses the full window
  width and adds **Check all for this game**, **Browse…** aligns cleanly with **Install
  location**, and **Feedback** fits standard `1160×760` windows without scrolling.
- **Refreshed app icon & dark setup wizard.** Ships a new multi-size desktop/taskbar icon and
  matching dark stone NSIS installer header and sidebar artwork.

## Known issues

- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.4_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.4), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

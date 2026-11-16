# Chriz Easy BG 0.1.6

**Released:** app 0.1.6 / collection 0.1.3, on 2026-11-16. The exact package and its test
results are in the [package verification record](package-acceptance-0.1.6-2026-11-16.md).

0.1.6 brings **Chriz Overhead Text v0.1.0** into Chris's Recommended setup (`467` default
components across `70` runs and `56` mods) so spoken overhead lines stay visible above
**Tab / Reveal Details** nameplates and **`HP Over Head`** labels. Updating CEBG leaves your
current games and saves unchanged; the new mod comes with a new installation.

## Highlights

- **Spoken overhead text stays visible above Tab nameplates and HP Over Head.** In the
  vanilla Infinity Engine, holding or toggling **Tab / Reveal Details** or enabling **`HP
  Over Head`** overwrites the creature's single floating-text slot every frame and hides
  spoken overhead lines (`DisplayStringHead`, `VerbalConstant` subtitles). **Chriz Overhead
  Text v0.1.0** gives nameplates and overhead HP their own slot and stacks spoken lines `4 px`
  cleanly above single-line or multi-line nameplates, scaled with area zoom.
- **Enabled by default, optional in Customize.** Installed by default with EEex in Chris's
  Recommended (`post-eet-end` phase); you can switch it off under **Interface** in
  **Customize**.

## Known issues

- A full multi-hour EET installation of the combined 467-component stack has not been run
  specifically for 0.1.6; `Chriz Overhead Text v0.1.0` installs a single runtime Lua hook file
  (`override/M_CZOH.lua`) without modifying game resources or `UI.MENU`, and was verified
  offline across `BG:EE`, `BG2:EE`, and `IWD:EE` (`2.7.3.0`, `EEex v1.2.0` and `v1.3.1`) as
  well as in a live `BG2:EE 2.7.3.0` + `EEex v1.3.1` lab.
- CEBG's updater signature is not a Windows Authenticode publisher signature. Windows may
  still display an unknown-publisher warning. Download only from the official release; do
  not disable Defender or add blanket exclusions.

## Installing and updating

- **New to CEBG:** download `Chriz.Easy.BG_0.1.6_x64-setup.exe` from the
  [release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.6), run
  it and choose **New installation**.
- **Already using CEBG:** open **Updates** and choose **Update CEBG app**. It updates the
  installer and launcher; your games and saves stay unchanged.
- **The collection change** comes with a new installation: **Create updated installation**
  on the Updates page, or **New installation**. Your current installation keeps its mods.
- Need help? See the [troubleshooting and backup guide](troubleshooting.md).

# Chriz Easy BG 0.1.8

App **0.1.8** bundles collection **0.1.5**. Chris's Recommended includes **470 default
components across 71 install runs and 57 mods**.

## Highlights

- **No more crash after the multiplayer lobby.** Chriz Multiplayer Fix **v0.1.0** stops the
  random crash right after leaving Beamdog's multiplayer lobby, for example on
  **Multiplayer → Local network game → Load**.
- **Copy saves between single player and multiplayer.** Select a save on the Load screen and
  click **Copy to single player** or **Copy to multiplayer** at the bottom left. The copy
  lands in the other mode's save list and the original save stays. Viewers can build the
  party together in multiplayer for a chatter run, and you play on in single player. Both
  are on by default with EEex and can be switched off in **Customize**.
- **Challenge Mode: no travel back into an earlier campaign.** Chriz BG Modpack
  **v0.2.0-alpha.17** adds a Challenge Mode change: once you move on to the next campaign
  (Baldur's Gate, then Siege of Dragonspear, Shadows of Amn and Throne of Bhaal), you can
  no longer travel back to the earlier campaign's areas. Watcher's Keep stays open in
  Shadows of Amn and Throne of Bhaal. It also fixes several Siege of Dragonspear camp
  problems. It is off by default and comes on with the **Challenge Mode** switch.

## Known issues

- A full combined EET installation with these changes and a full playthrough across the
  campaign transitions are still pending.
- The setup carries a Tauri updater signature. Windows may show an unknown-publisher
  warning because the setup has no Windows Authenticode signature.

## Installing and updating

- For a first-time setup, download
  [Chriz.Easy.BG_0.1.8_x64-setup.exe](https://github.com/Chrizhermann/chriz-easy-bg/releases/download/v0.1.8/Chriz.Easy.BG_0.1.8_x64-setup.exe)
  from the [v0.1.8 release page](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.8).
  Run the setup and choose **New installation**.
- To update the app, choose **Updates → Update CEBG app**. Your installed games and saves
  keep working unchanged.
- The multiplayer fix and the copy button come with a new installation: choose
  **Create updated installation** on the Updates page or **New installation**. Chriz Easy
  BG cannot add them to a game you already installed yet. To get them there now, close the
  game, extract the
  [Chriz Multiplayer Fix v0.1.0 release zip](https://github.com/Chrizhermann/chriz-multiplayer-fix/releases/tag/v0.1.0)
  into the game folder, run `setup-chriz_multiplayer_fix.exe` and install both components.
- For help, see the [troubleshooting and backup guide](troubleshooting.md).

# Chriz Easy IWD Windows releases


Build and release this product from a recorded commit on `main` in the source repository.
The other installer has its own release and updater feed; see [BUILDING](BUILDING.md).

Chriz Easy IWD is this app built with `app/src-tauri/tauri.iwd.conf.json`. Its releases live
in the public distribution repository `Chrizhermann/chriz-easy-iwd`, apart from Chriz Easy
BG's; like [CEBG's](cebg-release.md) they hold no mod archives. Christopher's decision of
2026-10-25 gave the app its own update feed ([v1 plan](plans/2026-10-25-chriz-easy-iwd-v1.md));
it ships with the first version whose configuration names that feed.

Publishing a release, creating or moving a tag, and changing the update channel each need
Christopher's explicit OK. The steps below say where they start.

## What a release holds

A versioned release `v<version>` carries:

- the NSIS setup as GitHub publishes it, e.g. `Chriz.Easy.IWD_0.1.7_x64-setup.exe`
  (GitHub turns the spaces of `Chriz Easy IWD_0.1.7_x64-setup.exe` into dots);
- its Tauri updater signature, `<setup>.sig`;
- `latest.json`, the update feed for exactly this setup;
- `SHA256SUMS` over all of the above plus `LICENSE` and `THIRD_PARTY_NOTICES.md`;
- `LICENSE` and `THIRD_PARTY_NOTICES.md`.

Versions 0.1.1 to 0.1.6 have no updater; their releases have no `.sig` and no `latest.json`.

## Update feed

Installed copies read
`https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/updates/latest.json`
(`plugins.updater.endpoints` in `tauri.iwd.conf.json`). The fixed `updates` tag is a
prerelease that carries only `latest.json`, like CEBG's `alpha` channel. It is re-pointed to a
new version only after that version's release assets are published and checked, so a
download can go out before the in-app update does, and the channel can stay on the previous
version if a release needs another look. The app offers only versions newer than its own:
pointing the channel back stops a rollout but does not downgrade anyone who already
updated, which takes a newer fixed version.

The updater runs the new setup in update mode, where Tauri's installer writes the new files
over the old ones and removes nothing. The preinstall hook in `app/src-tauri/windows/hooks.nsh`
therefore empties the folders the app bundles (`recipes/iwd-current`, `hotpatches/iwd`) first:
an older recipe file left next to its successor stops the app ("artifacts … disagree about one
byte-source URL", seen when 0.1.6 was installed silently over 0.1.3). CI installs each setup,
adds such leftovers and checks that an update install removes them
(`tools/check-nsis-update-cleanup.ps1`).

## Signing key

Chriz Easy IWD has its own minisign key; CEBG's key is not used. A setup signed for one app is
therefore never installed as an update of the other (`app/src-tauri/tests/package_config.rs`
and `updater_download_acceptance.rs` check this).

- The public key is `plugins.updater.pubkey` in `tauri.iwd.conf.json`; it is public by design.
- The private key is stored as the `chriz-easy-iwd-updater.key` attachment in Christopher's
  vault entry **Chriz Easy IWD updater signing key**, without a password. The formerly
  documented `%LOCALAPPDATA%\Chriz Easy IWD Developer\signing\` copy was absent during
  the 0.1.8 release. Retrieve a temporary local copy outside every repository, build output
  and diagnostic archive; pass only its path to the signer and remove the temporary copy
  afterwards. Never commit, upload, paste or print its contents. The OneDrive backups skip
  `*.key` files on purpose.
- If the key is lost or exposed, generate a new pair, put the new public key in
  `tauri.iwd.conf.json` and ship it in a new version. Installed copies cannot verify a setup
  signed with the new key, so every player installs that version by hand once, as with the
  first updater version.

## Release steps

The example version is `0.1.7`; use the real one throughout.

1. **Number the release.** As before (see the `release:` commits): app version in
   `tauri.iwd.conf.json`, recipe version in `recipes/iwd-current/release.json` with its release
   ledger, and the hotpatch catalog's new base. Build from the merged commit. The updater
   offers a version only when it is higher than the installed one.

2. **Build the setup.** The build leaves out the updater artifacts, so the signing key is not
   needed there; step 3 signs the finished bytes. From `app/`:

   ```powershell
   npm.cmd ci
   New-Item -ItemType Directory -Path ..\target -Force | Out-Null
   $iwdConfig = (Resolve-Path 'src-tauri/tauri.iwd.conf.json').Path
   $iwdBuildConfig = Join-Path (Resolve-Path ..\target).Path 'iwd-release-build.json'
   '{"bundle":{"createUpdaterArtifacts":false}}' | Set-Content -LiteralPath $iwdBuildConfig -Encoding utf8
   npm.cmd run tauri -- build --target x86_64-pc-windows-msvc --bundles nsis --config $iwdConfig --config $iwdBuildConfig -- --locked
   ```

   The setup is `target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy IWD_0.1.7_x64-setup.exe`.
   Without the second `--config`, the build stops and asks for the private key
   (`createUpdaterArtifacts` is on in `tauri.iwd.conf.json`).

3. **Sign it.** From the repository root. This is the empty-password pattern from
   [BUILDING.md](BUILDING.md#release-identity-and-signing): PowerShell would drop an empty
   password variable and leave the signer waiting at a prompt.

   ```powershell
   $iwdSigner = (Resolve-Path 'app/node_modules/@tauri-apps/cli/tauri.js').Path
   $iwdSigningKey = (Resolve-Path '<temporary key file retrieved from the vault>').Path
   $iwdSetup = (Resolve-Path 'target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy IWD_0.1.7_x64-setup.exe').Path
   $iwdSign = @'
   const { spawnSync } = require('node:child_process');
   const [cli, key, setup] = process.argv.slice(1);
   const child = spawnSync(process.execPath,
     [cli, 'signer', 'sign', '--private-key-path', key, '--password', '', setup],
     { stdio: 'inherit', windowsHide: true });
   if (child.error) { process.stderr.write(child.error.message); process.exit(1); }
   process.exit(child.status === null ? 1 : child.status);
   '@
   & node -e $iwdSign $iwdSigner $iwdSigningKey $iwdSetup
   if ($LASTEXITCODE -ne 0) { throw 'Tauri setup signing failed.' }
   ```

   This writes `<setup>.sig` beside the setup. Do not change the setup bytes afterwards. The
   signature is for the updater only; the setup still has no Windows Authenticode signature.

4. **Check the signature against the IWD key**, with the real updater as installed copies run
   it. `TAURI_CONFIG` builds the tests as Chriz Easy IWD, so they read its key; the second test
   also refuses the setup with one byte changed.

   ```powershell
   $env:TAURI_CONFIG = Get-Content -LiteralPath app/src-tauri/tauri.iwd.conf.json -Raw
   $env:CEBG_SIGNED_SETUP = $iwdSetup
   cargo test --locked -p chriz-bg-app --test package_config -- --ignored --exact built_update_signature_matches_the_bundled_public_key
   $env:CEBG_UPDATER_SETUP = $iwdSetup
   $env:CEBG_UPDATER_VERSION = '0.1.7'
   $env:CEBG_UPDATER_CURRENT_VERSION = '0.1.6'
   cargo test --locked -p chriz-bg-app --test updater_download_acceptance -- --ignored --exact real_tauri_updater_downloads_and_verifies_the_signed_nsis
   Remove-Item Env:TAURI_CONFIG, Env:CEBG_SIGNED_SETUP, Env:CEBG_UPDATER_SETUP, Env:CEBG_UPDATER_VERSION, Env:CEBG_UPDATER_CURRENT_VERSION
   ```

5. **Write the feed and checksums.** `-Notes` is what the app shows under **What's new**: one
   short paragraph in plain words. `-RecipeVersion` defaults to the app version; pass the
   version of `recipes/iwd-current/release.json` if it differs.

   ```powershell
   ./tools/package-cebg-update.ps1 -Flavor iwd -Version '0.1.7' `
       -SetupPath $iwdSetup -SignaturePath "$iwdSetup.sig" `
       -OutputDirectory 'target/iwd-release/0.1.7' `
       -AdditionalAssets LICENSE, THIRD_PARTY_NOTICES.md `
       -Notes 'Chriz Easy IWD 0.1.7: … Existing games remain unchanged.'
   ```

   The script refuses a setup whose name is not this version's Chriz Easy IWD setup, so the
   feed cannot announce one version and serve another. It neither signs nor uploads.

6. **Publish the versioned release** (Christopher's OK first). The release text follows the
   earlier IWD releases. As the `Chrizhermann` account:

   ```powershell
   $env:GH_TOKEN = gh auth token --user Chrizhermann
   $iwdOut = 'target/iwd-release/0.1.7'
   gh release create v0.1.7 -R Chrizhermann/chriz-easy-iwd --title 'Chriz Easy IWD 0.1.7' --notes-file <release notes> `
       $iwdSetup "$iwdSetup.sig" "$iwdOut/latest.json" "$iwdOut/SHA256SUMS" LICENSE THIRD_PARTY_NOTICES.md
   ```

7. **Check what was published**, signed out: download all six assets from
   `https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/v0.1.7/<name>`, compare
   them with `SHA256SUMS`, and confirm that the `url` in the downloaded `latest.json` is the
   published setup.

8. **Point the update channel** (Christopher's OK first). The first time, create the channel:

   ```powershell
   gh release create updates -R Chrizhermann/chriz-easy-iwd --prerelease --title 'Update channel - metadata only' `
       --notes "This is Chriz Easy IWD's update channel: it hosts the update feed (latest.json) that installed copies of Chriz Easy IWD check for new versions. It now serves Chriz Easy IWD 0.1.7. Download the installer from the [v0.1.7 release](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.7). This is updater metadata, not a second installer." `
       "$iwdOut/latest.json"
   ```

   For later versions, replace the file and the version in the text:

   ```powershell
   gh release upload updates "$iwdOut/latest.json" --clobber -R Chrizhermann/chriz-easy-iwd
   gh release edit updates -R Chrizhermann/chriz-easy-iwd --notes "<the same text with the new version>"
   ```

   Then confirm, signed out, that the feed URL above serves the new `latest.json`.

9. **Update the README** of `Chrizhermann/chriz-easy-iwd` for the version, as before. The
   website's IWD download points at the separate IWD release, alongside the BG release.
   Its own website task owns any link update; the two destinations are in `BUILDING.md`.

## The first version with the updater

Players on 0.1.1 to 0.1.6 have no updater, so they install this version by hand once; its setup
also removes recipe files that earlier upgrades left behind. The release notes tell them, for
example:

> **Updates itself from now on.** This is the last version you download by hand: install it
> over your current one; your installations and saves stay unchanged. From then on, open
> **Updates** and choose **Update Chriz Easy IWD app** when a new version is out.

In the README, "Known limitations" loses "Chriz Easy IWD does not update itself yet. …", and
"The app" gains a bullet such as:

> - **App updates.** From 0.1.7 on, Chriz Easy IWD updates itself: open **Updates** and choose
>   **Update Chriz Easy IWD app**. An app update never changes your installations; a hotpatch
>   is applied only when you choose it.

Until then the channel does not exist, and the feed URL answers 404: a 0.1.7 copy then shows
"Could not connect" under Updates. Create the channel (step 8) together with the release.

# Chriz Easy IWD 0.1.14 package acceptance

Chriz Easy IWD 0.1.14 was released on 2026-11-18 at 15:00:03 UTC
(2026-11-19 at 00:00:03 KST) after the release source passed both CI workflows.

## Source and scope

- The release was built from `main` at `62f4ab7bb3f7e64f86d15d3d3d48b3b699b4622f`,
  the merge of PR #153 (head `1f1d8c369352f117b65fcc598aed738069ba2957`).
  [PR #153](https://github.com/Chrizhermann/chriz-bg-collection/pull/153)
- App **0.1.14** bundles `recipes/iwd-current` and the `hotpatches/iwd` catalog at
  **0.1.15** (`iwd-0.1.15`, content digest
  `518171a581f42c3c82e732ae2d67759ae49668258d8d803f24fc9368245e83fe`).
  Chris's Recommended includes **33 mods, 143 default components, 34 install runs,
  and 18 recruitable companions**. [Release notes](release-notes-iwd-0.1.14.md)
- Changes since app 0.1.13
  - **BuffBot v1.16.2** updates the Italian catalog and is available through
    **Updates → BuffBot**. [PR #149](https://github.com/Chrizhermann/chriz-bg-collection/pull/149)
  - **Chriz IWD Party Dialog v0.1.0** entered recipe `iwd-0.1.14`. Living party
    members in the current area can qualify for world and quest dialogue options
    by class, kit or race and take over as speaker. Personal companion conversations
    still address the main character. The component is enabled by default with EEex
    under **Quality of life** in **Customize**.
    [PR #150](https://github.com/Chrizhermann/chriz-bg-collection/pull/150)
  - **BG Rebalance v0.8.12** adds Round Clock (component `361`) in recipe
    `iwd-0.1.15`. Travel, rest and scripted jumps in world time advance the waits
    between eligible actions and Contingency checks. Round Clock is enabled by
    default with EEex under **Spells** in **Customize**.
    [PR #152](https://github.com/Chrizhermann/chriz-bg-collection/pull/152)

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR #153 head `1f1d8c3` | [44299495186](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44299495186) (success) | [44299495119](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44299495119) (success) |
| `main` at `62f4ab7` (release source) | [44304091104](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44304091104) (success) | [44304091230](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44304091230) (success) |

`tools/own_mod_pins.py check --release` finished on the release source at
`2026-11-18T13:06:07.9450603Z` with exit 0 and `OK: no installer is out of step.`
The check confirmed BuffBot v1.16.2, Chriz IWD Party Dialog v0.1.0 and BG Rebalance
v0.8.12 were on their newest releases when the packages were built.

## Package

- `Chriz Easy IWD_0.1.14_x64-setup.exe` (published as
  `Chriz.Easy.IWD_0.1.14_x64-setup.exe`) was built with the IWD flavor from the clean
  detached release source. The setup is **7,142,731 bytes** with SHA-256
  `d9fb4f9d30861b2baf9454fc2cbbf27b212216d233989676a69bd5f57a67dc6b`.
- `Chriz.Easy.IWD_0.1.14_x64-setup.exe.sig` is **428 bytes** with SHA-256
  `20fcc51c8d450ea27016422eff588f1b3cf740ecf2610fc579dd16ed1b57ae8f`.
- `latest.json` is **966 bytes** with SHA-256
  `89ec0261a900cf28659b96adbb795d23f855ba7ba1243dff72799cd445374332`.
  It points to the versioned setup and records `recipe_version` as `0.1.15` and
  `pub_date` as `2026-11-18T13:36:56Z`.
- `SHA256SUMS` is **449 bytes** with SHA-256
  `006e3ccc5eb42274aa58cb76bc2e6a9f92fc0df6b6a35a261cdc2785c24d2b61`.
- `LICENSE` is **1,076 bytes** with SHA-256
  `b414bdb57a4a78aca070c6e299cdcdbc6ed8f46dbe494f50281cc5f02af369ea`.
- `THIRD_PARTY_NOTICES.md` is **892,416 bytes** with SHA-256
  `8621b5aafcb374f28e1852f405719a4d94e4cb90156b4c0df5587f67cf268712`.
- The setup was signed with the existing IWD Minisign updater key (key ID
  `17DED681DA9CDAFE`). The temporary key copy was removed after signing
  (`TEMP_IWD_KEY_REMOVED=True`).
- `package_config` passed `built_update_signature_matches_the_bundled_public_key`
  against the exact signed setup. `updater_download_acceptance` passed
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` from **0.1.13 to
  0.1.14** using the IWD configuration. The updater accepted the signed bytes and
  rejected the altered bytes.
- The release build scan found no `CHRIZ_BG_COLLECTION_TEST_*` strings in
  `chriz-easy-iwd.exe` or the setup (`NO_TEST_HOOK_STRINGS_OK_IWD`, ASCII and UTF-16LE).
- Microsoft Defender scanned an unexcluded copy of the exact setup bytes outside
  `CEBG-Tests` and reported **found no threats**, exit 0. The Defender product was
  `4.18.26100.4`, engine `1.1.26100.4`, and security intelligence `1.463.349.0`.
  Real-time protection and Behavior Monitor were enabled.

## Publication

- Release **v0.1.14** was published at `2026-11-18T15:00:03Z`. All six anonymous
  asset downloads matched the staged files by byte count and SHA-256. The five
  `SHA256SUMS` entries and the feed's signature against the published `.sig` also
  matched. [GitHub release](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.14)
- The `updates` channel first served the 0.1.14 feed at
  `2026-11-18T15:01:26Z`. GitHub's asset metadata and an anonymous download matched
  the versioned `latest.json` SHA-256. The previous **0.1.13** feed was saved before
  upload for rollback (932 bytes, SHA-256
  `b04387d3d8a5e2bc341e4a05124fc98b58a84b76265d158d262acab88afd9e07`).
  [Update feed](https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/updates/latest.json)
- The distribution repository's `README.md` was updated at
  `cf0d42b6eed2bb16ac3b2ef6c64da46c845f64c1`.
  [README commit](https://github.com/Chrizhermann/chriz-easy-iwd/commit/cf0d42b6eed2bb16ac3b2ef6c64da46c845f64c1)
- Website PR #105 published the new download information and was merged at
  `2d6e30e89a72fff726939142880af29489bc42d9`. The live IWD page and production site
  data showed app 0.1.14 and the published setup hash.
  [Website PR #105](https://github.com/Chrizhermann/twitch-setup-chriz/pull/105) ·
  [Chriz Easy IWD](https://bg.chrizfader.org/iwd)

## Not covered

- The combined 34-run WeiDU installation and a full IWD campaign playthrough
  remain pending for app 0.1.14.
- Chriz IWD Party Dialog passed 159 offline checks. An IWD:EE 2.7.3.0 quiet lab
  verified option availability and speaker handover for qualifying party members.
  Personal companion dialogue continued to address the main character.
- Round Clock's paused Stealth click was verified in BG2:EE and IWD:EE 2.7.3.0
  quiet labs.
- Host setup and desktop UI launches were outside package acceptance. The automated
  Tauri updater test covered the download and cryptographic verification path.

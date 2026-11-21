# Chriz Easy IWD 0.1.15 package acceptance

Chriz Easy IWD 0.1.15 was released on 2026-11-21 at 18:50:46 UTC
(2026-11-22 at 03:50:46 KST) after the release source passed both CI workflows.

## Source and scope

- The release was built from `main` at `2da056bf698ae12780cf8c5eb33623e3259caa83`,
  the merge of PR #157 (head `5330b20`, pin commit
  `ec3710e9e4fbba7bdfcd9dda36ba132fb4f8927d`).
  [PR #157](https://github.com/Chrizhermann/chriz-bg-collection/pull/157)
- App **0.1.15** bundles `recipes/iwd-current` and the `hotpatches/iwd` catalog at
  **0.1.16** (`iwd-0.1.16`, content digest
  `ba389b0325f180c53d14c65d2c4638e9a129fdee84ca3627e7e53ca2212b09d4`).
  Chris's Recommended includes **33 mods, 145 default components, 34 install runs,
  and 18 recruitable companions**. [Release notes](release-notes-iwd-0.1.15.md)
- Changes since app 0.1.14
  - **Chriz IWD Encounters v0.3.0** adds two new defaults under **Customize →
    Chriz IWD Encounters**: component **120** (Baldur's Gate caster-level
    Dispel/Remove Magic, L1–4 Minor Spell Deflection bypassed by Breach/Pierce,
    non-stacking Mirror/Reflected/Spirit Armor, AoE Mirror Image bypass, logical
    status immunities, and Universal utility schools) and component **420** (Chapter 2–6
    main campaign overhaul with curated filler thinning, 3-wave Kresselack crypt
    pacing, Terikan/Tarnsen/Ginafae script fixes, upgraded Severed Hand/Giant/Wyrm
    encounters, and five *Drasus*/*Iron Throne*-style squads).
  - Boss updates in components **300**, **310**, and **360** replace floor dispel
    triggers with Belhifet's level-26 Dispel Magic and Yxunomei's level-18 Dispel
    Magic (plus fixing Yxunomei's missing telekinesis delay) and tune Poquelin to
    310 hit points.
  - Optional component **500** (off by default) curates Heart of Winter and Trials of
    the Luremaster encounters, thins hordes, and repairs `AR9602` Xactile while
    keeping full SCS caster spellbooks.

## Automated source checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR #157 head `5330b20` | [44912998726](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44912998726) (success) · [44912995405](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44912995405) (success) | [44912998651](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44912998651) (success) · [44912995495](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44912995495) (success) |
| `main` at `2da056b` (release source) | [44937150871](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44937150871) (success) | [44937150878](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/44937150878) (success) |

`tools/own_mod_pins.py check --release` finished on the release source at
`2026-11-21T18:44:30.1646673+00:00` with exit 0 and `OK: no installer is out of step.`
The check confirmed BuffBot v1.16.2, BG Rebalance v0.8.12, Chriz IWD Encounters
v0.3.0, Chriz Overhead Text v0.1.0, and Chriz IWD Party Dialog v0.1.0 were on their
newest releases when the package was built.

## Package

- `Chriz Easy IWD_0.1.15_x64-setup.exe` (published as
  `Chriz.Easy.IWD_0.1.15_x64-setup.exe`) was built with the IWD flavor from the clean
  detached release source. The setup is **7,144,911 bytes** with SHA-256
  `31b7e10aa3aca4a9a5b76713de6f0d05c7d00757b8cf157cf1398aa479920c79`.
- `Chriz.Easy.IWD_0.1.15_x64-setup.exe.sig` is **428 bytes** with SHA-256
  `c88f7eb3cf71b2f2df61e8e4ce6cdae736c2c7c9679873f3619d89327166524b`.
- `latest.json` is **1,024 bytes** with SHA-256
  `93c506d6be037d8c23b6b82a5b05113c4c08d5e813da2dada6abe842922737ec`.
  It points to the versioned setup and records `recipe_version` as `0.1.16` and
  `pub_date` as `2026-11-21T18:49:38Z`.
- `SHA256SUMS` is **449 bytes** with SHA-256
  `d37e7ab9ab4d15c7c136157f616f6f5a57bfece9ab0221429b6e642b37048dac`.
- `LICENSE` is **1,097 bytes** with SHA-256
  `554232c3c694355ec4d60fd6fd9d4a3cc30b9f03e280bfc5979d96b6d656db19`.
- `THIRD_PARTY_NOTICES.md` is **909,907 bytes** with SHA-256
  `4ea5ae48b829f83074df9574fa7a7c7078ca6d0e2eb885b129cdf2891d062ce8`.
- The setup was signed with the existing IWD Minisign updater key. The temporary key
  copy was removed after signing (`TEMP_IWD_KEY_REMOVED=True`).
- `package_config` passed `built_update_signature_matches_the_bundled_public_key`
  against the exact signed setup. `updater_download_acceptance` passed
  `real_tauri_updater_downloads_and_verifies_the_signed_nsis` from **0.1.14 to
  0.1.15** using the IWD configuration. The updater accepted the signed bytes and
  rejected altered bytes.
- The release build scan found no `CHRIZ_BG_COLLECTION_TEST_*` strings in
  `chriz-easy-iwd.exe` or the setup (`NO_TEST_HOOK_STRINGS_OK_IWD`, ASCII and UTF-16LE).
- Microsoft Defender scanned an unexcluded copy of the exact setup bytes outside
  `CEBG-Tests` and reported **found no threats**, exit 0 (`SCAN_DIR_REMOVED=True`).
  The Defender product was `4.18.26100.4`, engine `1.1.26100.4`, and security
  intelligence `1.463.404.0`. Real-time protection and Behavior Monitor were enabled.

## Publication

- Release **v0.1.15** was published at `2026-11-21T18:50:46Z`. All six anonymous
  asset downloads matched the staged files by byte count and SHA-256, and the feed's
  `url` pointed to the published `v0.1.15` setup (`ANON_V0115_ALL_6_MATCH_OK`).
  [GitHub release](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.15)
- The `updates` channel first served the 0.1.15 feed at `2026-11-21T18:50:54Z`.
  An anonymous download matched the versioned `latest.json` SHA-256. The previous
  **0.1.14** feed was recorded before upload for rollback (966 bytes, SHA-256
  `89ec0261a900cf28659b96adbb795d23f855ba7ba1243dff72799cd445374332`).
  [Update feed](https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/updates/latest.json)
- The distribution repository's `README.md` was updated at
  `057155a7a18d7f8d2a7953fbcdcc8eafa6b5c6bd`.
  [README commit](https://github.com/Chrizhermann/chriz-easy-iwd/commit/057155a7a18d7f8d2a7953fbcdcc8eafa6b5c6bd)
- Website PR #107 published the new download information and was merged at
  `e75725227aabd56279d772ee0f1e59c3d039168f` (`BG site check`
  [44937945718](https://github.com/Chrizhermann/twitch-setup-chriz/actions/runs/44937945718)
  succeeded). The live production JSON at
  `https://bg.chrizfader.org/content/site-content.json` showed app `0.1.15` and the
  published setup hash.
  [Website PR #107](https://github.com/Chrizhermann/twitch-setup-chriz/pull/107) ·
  [Chriz Easy IWD](https://bg.chrizfader.org/iwd)

## Not covered

- The full combined 34-run app-driven WeiDU installation and a full IWD campaign
  playthrough remain pending for app 0.1.15.
- In `chriz-iwd-encounters v0.3.0`, all five offline test suites (`test_templates`,
  `test_banter_guard`, and twice-installed `test_overwrite` scenarios `base`,
  `edited`, `embedded`, `stack`), a quiet IWD:EE 2.7.3.0 verification lab (live
  spell/dispel rules, Terikan, `AR3503` wave wake timing, Chapter 2–6 squads,
  `check_install.py`), and an upgrade of Christopher's stream installation
  (`check_install.py` passed) ran cleanly.
- Host setup and desktop UI launches were outside package acceptance. The automated
  Tauri updater test covered the download and cryptographic verification path.

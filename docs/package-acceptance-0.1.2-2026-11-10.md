# 0.1.2 package verification — November 10

App **0.1.2**, bundled collection **0.1.2**. Build source: `main` at
`8c51eec437f183829a1975c50a696d5fc36f8ce9`, the merge of
PR [127](https://github.com/Chrizhermann/chriz-bg-collection/pull/127). Published on
2026-11-09 at 20:31 UTC (2026-11-10 05:31 KST) after Christopher's OK of 2026-11-09; see
[Publication](#publication). Later documentation does not alter the verified package.

## What this release changes

See the [release notes](release-notes-0.1.2.md). Against 0.1.1:

- **Installer:**
  - The question when a single component fails (#114).
  - Changed and unusable downloads no longer ask or stop; the overwrite rule for the
    installer's own checks (#120).
  - The BuffBot section under Updates (#115).
  - Clean bundled folders on an app update (#112).
- **Collection:** rolled hit points in Chris's Recommended (#113); BuffBot 1.16.1,
  BG Rebalance 0.8.11, Chriz BG Modpack alpha.16, SoD Remix 0.6.17 and the voicepacks 1.2.0
  (#103, #106, #123, #124).
- **Version:** app 0.1.2 in the files the 0.1.1 bump changed; the notices record the new
  lockfile digests (#127).
- **Ledger:** `releases/v0.1.2/ledger.toml` in both profiles, five entries covering the 35
  recorded differences. It supersedes 0.1.1 and needs app 0.1.2.
- **Publication date:** the ledger says `2026-11-10T00:00:00Z`, fixed before packaging
  because the setup bundles it. The release went out on 2026-11-10 Korean time, which is
  about three and a half hours before that date began in UTC. The setup was not rebuilt for
  this; nothing in the app compares the date.

## Exact package

- **Setup:** `Chriz Easy BG_0.1.2_x64-setup.exe`, published as
  `Chriz.Easy.BG_0.1.2_x64-setup.exe`. **6,760,317 bytes**.
  - SHA256: `47e07dd0d2c33c9d7edabb262ea826582e153a2b0b03730b5d742a68c79d7945`.
- **Signature:** 424 bytes.
  - SHA256: `ffd54f5c1e4c2eb1ba0b920da97acc37b13dfacea3d2763ea41f6639317f7007`.
- **Feed** (`latest.json`, 1,152 bytes; it points to the `v0.1.2` release asset):
  - SHA256: `2f0862bbbe9ab35a0d31b86a33301d20ec9ee6056262ad6604b3e9cb0d5385dc`.
  - `SHA256SUMS`: `113ba2fcb0d8c8261241a34f6532531c6ce57a4c9a6f44a7735ed650fd2bb706`. From
    this release on it also lists the license, the notices and the component inventory.
- **No test hook:** the shipped app contains none of the `CHRIZ_BG_COLLECTION_TEST_*`
  names. The setup has no Authenticode signature, as before.
- **Public component inventory** (`component-inventory.json`, 72,277 bytes), written by
  `tools/public_component_credits.py` from the release source:
  - SHA256: `37c8a6af411ad56aabb9b2e0b2a03ed65d68bf317d4e78838040939d91a2fa68`.
  - **466 default components, 69 runs, 55 mod entries** (0.1.1: 468 / 69 / 55). The two
    components that left Chris's Recommended are Higher HP on Level Up (Maximum) and
    Maximum HP Creatures.
- **License and notices:** the committed file bytes, `LICENSE` `b414bdb5…` (1,076 bytes) and
  `THIRD_PARTY_NOTICES.md` `81306de6…` (892,307 bytes).

Not repeated for this release: the comparison of the extracted app with the compiler output
and of the bundled resources with their source bytes, which the 0.1.1 record has.

## Automated checks

| Commit | Windows source checks | Own mod pins |
| --- | --- | --- |
| PR 127 head `c8d0341` | runs 42640983939, 42640978660 | runs 42640984021, 42640978567 |
| `main` at `8c51eec` (build source) | run 42647837365 | run 42647837384 |

The Windows job runs the Rust workspace, the IWD app flavor, the frontend, the Python
tools, formatting, Clippy, the locked third-party notices and unsigned BG and IWD
packaging. The full suites were not run again on the build machine; the build ran there
from a checkout of the build source with no tracked change, at BelowNormal priority.

Tauri again rewrote `app/src-tauri/Cargo.toml` with LF line endings in this CRLF checkout.
Its content equals the committed file, which was restored before signing.

- The signature test against the bundled updater public key passed for this exact setup.
- The real Tauri updater download check passed from **0.1.1 to 0.1.2**: the updater offered
  0.1.2 to a 0.1.1 app, accepted the exact signed bytes and rejected deliberately altered
  bytes. It did not install the update or test the updater restart.
- `tools/package-cebg-update.ps1` wrote `latest.json` and `SHA256SUMS`.

The setup was signed with the existing updater key, passed to the signer by path only. No
key was created, copied or rotated.

## Defender and runtime boundaries

A single-file custom scan of the exact setup bytes returned **no threats**, exit 0, with
security intelligence **1.463.214.0** (engine 1.1.26100.4). Antivirus, real-time,
behavior-monitor and downloaded-file protection remained enabled; no exclusions or security
preferences were changed. Only the setup's single-file scan ran.

No setup or app was launched, and the uninstaller was not run. No stream installation or
saved game was touched.

These are bounded clean-scan results, not a Microsoft guarantee or a Windows Authenticode
publisher certificate.

## Lab install

The candidate lab of 2026-11-08/09 installed the Recommended selection from `c95db24` into
a disposable lab without errors: 183 steps, 466 components (463 in the EET game, 3 in the
BG:EE copy before EET), final verification passed, no question came up and no mod was left
out. Between `c95db24` and the build source, the recipes, the manifest, the engine and the
app sources differ only in `release.json`'s version and the two new ledgers.

- The start test through the quiet lab runner passed on 2026-11-09: the game started,
  loaded a save, reported BuffBot 1.16.1 and a healthy scroll fix, and exited with code 0.
  The log lines about a missing `dialogf.tlk` and two music files are the known ones.
- Not checked in the lab: the failed-component question with a forced failure in this
  candidate, BuffBot's new party table, clones and summons in play, and the BuffBot update
  of a game in progress from the released app.

## Publication

**Own-mods release check.** `python -I tools/own_mod_pins.py check --release` ran on the
build source on 2026-11-09 at 20:13 UTC. Exit 0:

| Mod | Chriz Easy BG | Chriz Easy IWD | Newest | State |
|---|---|---|---|---|
| BuffBot | v1.16.1 | v1.16.1 | v1.16.1 | newest |
| Chriz BG Modpack | v0.2.0-alpha.16 | – | v0.2.0-alpha.16 | newest |
| BG Rebalance | v0.8.11 | – | v0.8.11 | newest |
| SoD Remix | v0.6.17 | – | v0.6.17 | newest |
| EEex Remote Console | v0.2.1 (3d9899076) | v0.2.1 (3d9899076) | v0.2.1 | newest |
| Voicepacks | v1.2.0 | – | v1.2.0 | newest |
| Chriz IWD Encounters | – | v0.2.2 | v0.2.2 | newest |
| Artisan's Kitpack (our fork) | v4.81a-dev.85928f9-chriz.6 | – | v4.81a-dev.85928f9-chriz.6 | newest |
| Bardic Wonders (our fork) | v2.9c-balance.8 | – | v2.9c-balance.8 | newest |
| Spell Revisions (our fork) | v4.21-chriz.7 | – | v4.21-chriz.7 | newest |
| Item Randomiser (our fork) | v8.1.1 | – | v1.0.3 | held |

The Item Randomiser hold is the decision of 2026-10-25: v1.0.x is the Campaign Item
Randomiser, which replaces v8.1.1 only after its integration and a lab test.

**Release.** [v0.1.2](https://github.com/Chrizhermann/chriz-easy-bg/releases/tag/v0.1.2)
was published on 2026-11-09 at 20:31:34 UTC: a normal release, marked Latest, with seven
files (setup, signature, feed, checksums, component inventory, license, notices). An
anonymous download of all seven matched the local files and `SHA256SUMS`; the feed's
signature is the published `.sig`.

**Update channel.** Since 20:32 UTC the `alpha` release's `latest.json` is the 0.1.2 feed.
An anonymous download returned `2f0862bb…`, the same file as the release asset. The 0.1.1
feed stays as the rollback copy.

**Distribution README.** It names no version and needed no change.

**Website.** Its chat was told about 0.1.2 on 2026-11-10.

**Not tried live from a released setup yet:**
- the failed-component question;
- an installation in which a changed or unusable download is handled without a question;
- the BuffBot update and its Undo in a game in progress;
- a GOG, Beamdog or Microsoft Store copy.

The Updates page does not show the ledger's changes in this app version either; the release
notes say so.

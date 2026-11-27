# Chriz Easy IWD 0.1.17 package acceptance

Status: source checks, final package verification, updater-signature acceptance, and the
Defender scan below passed. Version 0.1.17, its update channel, the distribution README,
and the deployed website are public and verified. Release work is complete within the
coverage documented below.

## Source and scope

- Candidate: PR [169](https://github.com/Chrizhermann/chriz-bg-collection/pull/169) (combining
  PR [168](https://github.com/Chrizhermann/chriz-bg-collection/pull/168) commits `5b178fb` and
  `2d264b1` intact with `a9bba6c` for `command_contract.rs`), merged on `main` at
  `16fecba94f0ba216a0b621f59636a6241dfb8bdd`. The final setup was built from that detached
  `origin/main` checkout.
- App and recipe: **0.1.17** and recipe **0.1.22** (`supersedes = "0.1.21"`). The previous
  public app was `0.1.16` (recipe `0.1.17`).
- Changes:
  - **Chriz IWD Encounters v0.3.4** (`6161252`, PR #9): preserves quest death variables on
    custom squad actors in Dragon's Eye Level 2 and the Severed Hand, aligns thinned-area quest
    kill thresholds (`Talonite > 5 / < 6`, `ELVEN_SOLDIER > 11`, `8001_Salamanders > 8`,
    `8008_Salamanders > 4`), fixes triple-stacked AC/THAC0/APR mastery stats on `CIE4xx`/`CIE5xx`
    elites, marks bespoke elite equipment and AI buff potions undroppable, recalibrates the
    Talonite Coven, stretches Kresselack's antechamber waves (`48 s` / `+54 s`), removes every
    remaining Bombardier Beetle (`AR4001`, `AR4002`, `AR9801`), adds six apex encounters, and
    trims additional late-chapter and Watchknight hordes.
  - **Quest-counter hotpatch (`component 425`):** adds `chriz-iwd-encounters-0.3.4-quest-counters`
    (`save_applicability = "current-save"`) for `0.1.15`–`0.1.16` installs (`iwd-0.1.16` through
    `iwd-0.1.21`).
  - **Chriz BG Modpack v0.2.0-alpha.21** (and **Chriz Multiplayer Fix v0.1.0** from recipe
    `0.1.18`): pins `v0.2.0-alpha.21` (keeping component `470` in IWD with EEex and omitting
    BG-only/Challenge Mode `695`–`698`), totaling **148 default components across 36 install runs
    and 35 mods** with **18 recruitable companions**.

## Automated source checks

[Windows CI run 45909761135](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/45909761135)
and [Own mod pins](https://github.com/Chrizhermann/chriz-bg-collection/actions/runs/45909761225)
passed every check on the candidate (`a9bba6c`), alongside local verification:

- `python -I tools/own_mod_pins.py check --release`: `OK: no installer is out of step.`
- `python -I tools/generate-third-party-notices.py --check`: passed.
- `python -m unittest discover -s tools/tests -p "test_*.py"`: 477 passed, 1 skipped (and
  `test_nov27_chriz_iwd_encounters_034_recipe` 3/3 passed after final commit).
- `npm --prefix app run check:iwd-recipe`, `npm --prefix app test`, `npm --prefix app run build -- --mode iwd`: passed.
- `cargo fmt --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`,
  `cargo test --locked -p chriz-bg-engine`, and `cargo test --locked -p chriz-bg-app` (default BG
  and `tauri.iwd.conf.json` IWD): passed.

This is automated source verification, not a full 36-run live IWD campaign playthrough.
Christopher's live stream game (`Chriz Easy IWD Stream 2\game`) was synced and verified on
`Chriz IWD Encounters v0.3.4` (`420, 425, 500`) with `check_install.py` (`all checks passed`).

## Signed package, updater acceptance & Defender

Built with `createUpdaterArtifacts = false` from detached `origin/main` (`16fecba`), verified
to contain zero `CHRIZ_BG_COLLECTION_TEST_*` strings (ASCII and UTF-16LE), signed temporarily
using the retrieved Bitwarden signing key (key scrubbed and deleted immediately afterward),
and verified cleanly:

- `built_update_signature_matches_the_bundled_public_key`: ok.
- `real_tauri_updater_downloads_and_verifies_the_signed_nsis` (`0.1.16` → `0.1.17`): ok.
- Microsoft Defender (`MpCmdRun.exe -Scan -ScanType 3`): found no threats.

## Published artifacts & website

- Release: [v0.1.17](https://github.com/Chrizhermann/chriz-easy-iwd/releases/tag/v0.1.17)
  - `Chriz.Easy.IWD_0.1.17_x64-setup.exe` (`7,146,996` bytes), SHA-256
    `e51d54f2656fbc00833405178f362bbde39fb4bc08340415b78f1ed75ef9ebeb`
  - `Chriz.Easy.IWD_0.1.17_x64-setup.exe.sig`, SHA-256
    `76c0e860bd7ff25499f806213ae273b8402f8126a446417573d24ba1ed5b01c1`
  - `latest.json` (`1,109` bytes), SHA-256
    `dea82b98ec58ebac25ae00c9847425f1984a36ad7469c81a6e9cb3d16bdaa07c`
  - `SHA256SUMS`, `LICENSE`, `THIRD_PARTY_NOTICES.md` — all 6 assets downloaded anonymously
    and matched against `SHA256SUMS`.
- Update channel ([updates/latest.json](https://github.com/Chrizhermann/chriz-easy-iwd/releases/download/updates/latest.json)):
  verified anonymously (`dea82b98ec58ebac25ae00c9847425f1984a36ad7469c81a6e9cb3d16bdaa07c`;
  rollback record previous `0.1.16` `updates/latest.json` SHA-256
  `b4b450b214713daf8e62fd6137c6ef613b53645d9c3fdd5793c2d1f4f6e73d96`).
- Distribution README updated on `Chrizhermann/chriz-easy-iwd` (`7494fcb9de17ae4f70ad0c494d88af6a729d2693`).
- Website `https://bg.chrizfader.org/iwd` and `/iwd-encounters` updated and verified live via
  [PR #118](https://github.com/Chrizhermann/twitch-setup-chriz/pull/118) (`e5fd837`).

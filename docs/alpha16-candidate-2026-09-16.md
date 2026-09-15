# Alpha.16 installer candidate — September 16

App **0.1.0-alpha.16**, bundled collection **0.1.0-alpha.14**.
Recommended selection: **448 components in 50 runs**. This is the authored
curation, not a replay of the historical WeiDU.log. Source/evidence freeze:
`4c9199b`. Christopher will run the installation and playtest himself.

## Included owner releases

| Package | Version |
|---|---|
| Artisan's Kitpack balance fork | chriz-v1.5.0 |
| Bardic Wonders balance fork | v2.9c-balance.5 |
| Spell Revisions and separate SR/RR compatibility | v4.21-chriz.5 |
| chriz-bg-modpack | v0.2.0-alpha.6 |
| chriz-sod-remix | v0.6.10 |
| chriz-bg-rebalance | v0.4.0 |
| BuffBot | v1.8.4-alpha |
| Official Safana in Amn | v05 |

Every changed archive passed the installer's actual download/hash/extraction
contract. Exact URLs, SHA-256, sizes and publish roots are in the generated recipe's
`artifacts/` files. Only the selected mod roots are staged; standalone repair tools
and their executables are not run as extra collection components.

Classic bouncing Lightning80 is default; 81 is exclusive and optional. SR/RR follows
SR and either RR11 or RR12 before SCS. Yeslick188 follows the Alaghor route;
continuity199 immediately precedes EET_end. Safana189 and Imoen620 run after their
final relevant writers. New SoD normal components are included, not repair-only
components. Dragons110/111 and extra bridge sequencers257 remain unchecked.
BuffBot remains last. Existing curation exclusions and vanilla companion opt-outs
are preserved.

## Verification and handoff

- 99 focused engine tests, 15 app contract/customization tests and TypeScript
  checking passed. Focused Python curation/recipe tests passed.
- Public-alpha recipe validation passed without findings. Resolved defaults have
  only the five previously documented compatibility/availability omissions.
- Final validation caught missing evidence records for the six newly split runs;
  the generator now records the actual passed tests and approved tail placement.
  A regression checks every selected run's evidence and tail approval.
- All owning mod release work is finished. No agent full install, gameplay test,
  existing-game modification or save migration was performed during this wrap-up.
- Signed Windows NSIS build succeeded. The unchanged signature-verification
  harness verified these new bytes against the bundled public key (1 test passed).
  No native installer launch or updater apply/restart was performed.

Setup: `target/x86_64-pc-windows-msvc/release/bundle/nsis/Chriz Easy BG_0.1.0-alpha.16_x64-setup.exe`
(5,479,662 bytes).
SHA-256: `55aa5f45ad3a9a08ef2da80bdf25ff55fe0b40f4055108ec21568e746a646d4c`.
Local updater metadata and checksums are prepared in
`target/cebg-release/0.1.0-alpha.16/`, not published.

For the user's test: install with recommended defaults. To test the dragon changes
in the same game, explicitly enable the two optional dragon choices in Customize
(450 selected components). Do not enable mutually exclusive alternatives together.
Prior accepted Artisan/Bardic/SoD playtests are not new gates.

Later gameplay checks, where practical: BG1 Yeslick kit and normal level-up,
Xan/Yeslick transition continuity, Safana first-arrival inventory and later gear
retention, Spellhold Imoen's one-time XP award, Lightning behavior, optional dragons.
No full campaign replay is required before the user starts using this candidate.

## Publication and next work

**Post-build source change:** [incremental download retry](download-retry-acceptance-2026-09-16.md)
now passes 47 focused checks. The setup/checksum recorded above predates that change
and does **not** include it. Rebuild and re-verify the setup, updater payload/signature
and release checksums before publication; do not relabel the existing binary.

The website's simplified guide, differences overview and Upcoming section are live
at https://bg.chrizfader.org/collection (website commit `e39c1638`).
Current public download remains app alpha.15 / collection alpha.13 pending candidate
acceptance/publication. No new updater feed was published by this wrap-up.

Hotpatch implementation is explicitly next, not part of this installer build.
It must be opt-in, component/version/resource-aware, with backups and install-local
text handling. Existing saves may silently retain cached actors, effects, areas
or quest state even when they still load. Do not promise universal migration or
rollback of progression made after a patch.

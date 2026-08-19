# Pin-list for the 2.7.3.0 stack — Phase 0.3 (for Chris's review)

Generated 2026-08-20 from manifest/mod-sources.tsv + live GitHub checks.
Principle (Chris, 2026-08-19): the compilation targets CURRENT versions, not the
reference install's. Every pin below is validated by the Phase 1 test-install
milestone before it ships. Dropped mods (worksheet) are excluded.

**Global pins:** game 2.7.3.0 · WeiDU **249.00** for every mod (battle-tested,
SCS-safe; revisit >=251 only when Linux support lands) · language 0 (en_US)
except AJANTISBG2 (language 1).

## Third-party mods (kept)

| Mod | Installed | Proposed target | Notes |
|---|---|---|---|
| AJANTISBG2 | 21 | v21 | unchanged (installed = latest) |
| ARTISANSKITPACK | 6.0 | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| ARTISANSKITPACK_NPC | — | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| ARTISANSKITPACK_TWEAK | — | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| ASCENSION | 2.1.0 | 2.1.0 | unchanged (installed = latest) |
| AURA_BG1_2_EET | — | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| BARDICWONDERS | — | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| BGGO | v3.5 | v3.6 | upgrade v3.5 -> v3.6 |
| BRANWEN | v8pre | v8 | upgrade v8pre -> v8 |
| BUBB_SPELL_MENU_EXTENDED | v5.1 | v5.2 | v5.1 -> v5.2 (2026-05-11) |
| C0WARLOCK | 3.0 | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| CDTWEAKS | v18 | v18 | unchanged; reference already on v18 |
| CROSSMODBG2 | v30 | v30 | unchanged (installed = latest) |
| EEEX | v0.11.0-alpha | v1.2.0 | REQUIRED >=v1.1.5 for game 2.7.3.0; big jump from 0.11-alpha (stable line, scripts moved to EEex_scripts/) - retest hotkeys/UI modules |
| EEFIXPACK | Alpha 3 | Beta_2 | Alpha_3 -> Beta_2 (2026-05-04); still prerelease - beta risk accepted per design |
| EET | v14.0 | master @ 74e91d72bca5d073fa11c1d088b90d7ff0c7105d (2026-08-06) | v14.1 tag is NOT 2.7-clean; pin this SHA until v14.2 tags |
| EET_END | — | same EET SHA | ships inside EET |
| EET_TWEAKS | 1.12 | v1.12 | unchanged (installed = latest) |
| EVANDRA | v2.2 | v2.2 | manual download |
| FADE | 5.6 | v5.6 | unchanged (installed = latest) |
| HIDDENGAMEPLAYOPTIONS | 5.0 | v5.1 | upgrade 5.0 -> v5.1 |
| IEPBANTERS | v5.9 | v5.9 | unchanged (installed = latest) |
| IWDIFICATION | v11 | v11 | unchanged (installed = latest) |
| PAINA | — | v1.9 | upgrade ? -> v1.9 |
| RANDOMISER | 7 | v7 | unchanged (installed = latest) |
| RR | v4.92 | v4.92 | unchanged (installed = latest) |
| SAFANA | v0.5 | v05 | unchanged (tag is spelled v05); SoD-items carryover fix component still to be built |
| SARAHTOB | v8 | v8 | unchanged (installed = latest) |
| SIRENE_BG2 | — | master SHA at authoring | no tags/releases - resolve+pin current commit SHA when authoring mods/*.toml |
| SPELL_REV | v4.19 | v4.21 | left beta: v4.19.rc5 -> stable v4.21 (2026-08-17) |
| STRATAGEMS | 35.21 | v35.21 | unchanged; MUST run under WeiDU 249 (breaks on 251); unverified-by-author on 2.7, community installs work |
| UB | v28 | v28 | unchanged (installed = latest) |
| XAN | v19 | v19 | unchanged (installed = latest) |
| YESLICKNPC | v5.0 | v5.0 | unchanged (installed = latest) |

## Pending curation ruling

| Mod | Installed | Proposed target | Notes |
|---|---|---|---|
| HQ_SOUNDCLIPS_BG2EE | 1.3 | v1.3 (latest) | awaiting voice-pack ruling in worksheet |

## Chriz-layer repos

Pinned at authoring time to a tagged release (or SHA) of each own repo:
ABETTORHLAREBALANCE, AKCB_BERSERKER, AKCB_SHAPESHIFTER, AURA_BALANCE_PATCH_SPELLS, BG2EE-EET-FIXPACK, CHRIZ-BG-MODPACK, CHRIZ-BG-REBALANCE, CHRIZ-SOD-REMIX, EEEXREMOTE.

## Local hotfixes (22)

No public versions to pin — each needs its home decision (worksheet section 4)
before it can be pinned to a chriz-repo release.

## Review asks for Chris

1. EEex 0.11-alpha -> 1.2.0 is the riskiest jump (your B3Hotkey.lua, UI-scale,
   timer-module configs may need porting) — OK to target 1.2.0?
2. SCS 35.21 on 2.7: author-unverified. Accept with test-install validation?
3. EE Fixpack Beta_2 replaces Alpha_3 — accept beta risk (design said yes)?
4. EET master SHA pin (no tag): OK until v14.2?
5. Commit-zip mods (Artisan Kitpack/NPC/Tweak, Bardic Wonders, ...): pin the
   then-current SHA at manifest authoring, or the SHAs matching your installed
   versions where known (Bardic Wonders cfcd1c4)? Current = newer features,
   installed = closest to what you've tested.
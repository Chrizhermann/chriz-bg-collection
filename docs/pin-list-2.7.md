# Pin-list for the 2.7.3.0 stack — reviewed targets

Originally generated 2026-08-20; synchronized with completed component review on
2026-09-01, with the Randomiser pin refreshed on 2026-09-03 and the first
release-ready official artifact block frozen on 2026-09-04. Immutable
source/release work still open is indexed in
`docs/curation/components/FOLLOW_UPS.md`.
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
| ARTISANSKITPACK | 6.0 | chriz-v1.2.0 fork @ `f623045f` | Released selected fork; a later release is required before Shapeshifter `5110/5111` is available. |
| ARTISANSKITPACK_NPC | — | same chriz-v1.2.0 artifact | One artifact supplies all three TP2s; use the exact same source hash. |
| ARTISANSKITPACK_TWEAK | — | same chriz-v1.2.0 artifact | One artifact supplies all three TP2s; use the exact same source hash. |
| ASCENSION | 2.1.0 | 2.1.0 | Official release ZIP frozen; component `40` remains omitted until its duplicate-provider decision is resolved. |
| AURA_BG1_2_EET | — | reconciled Chris fork, pending release | Reconcile local balance work with seven newer upstream commits beyond reviewed upstream `285dabbc`, then publish an immutable source. |
| BARDICWONDERS | — | v2.9c-balance.2 @ `db0cf815` | Released selected fork; later finite-Abettor-HLA work still needs integration/release. |
| BGGO | v3.5 | v3.6 | Official v3.6 release ZIP frozen. |
| BRANWEN | v8pre | v8 | upgrade v8pre -> v8 |
| BUBB_SPELL_MENU_EXTENDED | v5.1 | v5.2 | Immutable official v5.2 tag archive frozen. |
| C0WARLOCK | 3.0 | commit `a8219922` | Pin the exact post-v4.0 commit and renamed `Artisans_Warlock/Artisans_Warlock.TP2` path. |
| CDTWEAKS | v18 | v18 | unchanged; reference already on v18 |
| CROSSMODBG2 | v30 | v30 | unchanged (installed = latest) |
| EEEX | v0.11.0-alpha | v1.2.0 | Official v1.2.0 release ZIP frozen; clean-stage runtime acceptance remains open. |
| EEFIXPACK | Alpha 3 | Beta_2 | Alpha_3 -> Beta_2 (2026-05-04); still prerelease - beta risk accepted per design |
| EET | v14.0 | master @ 74e91d72bca5d073fa11c1d088b90d7ff0c7105d (2026-08-06) | v14.1 tag is NOT 2.7-clean; pin this SHA until v14.2 tags |
| EET_END | — | same EET SHA | ships inside EET |
| EET_TWEAKS | 1.12 | v1.12 | unchanged (installed = latest) |
| EVANDRA | v2.2 | v2.2 | Blocked: no exact user-supplied archive contract is available yet; never rehost the page-gated archive. |
| FADE | 5.6 | v5.6 | unchanged (installed = latest) |
| HIDDENGAMEPLAYOPTIONS | 5.0 | v5.2 | Official Windows v5.2 release ZIP and menu frozen; execution remains deferred to the late Tweaks/UI ordering slice. |
| HQ_SOUNDCLIPS_BG2EE | 1.3 | v1.3 | Kept and default; installed equals latest reviewed release. |
| IEPBANTERS | v5.9 | v5.9 | unchanged (installed = latest) |
| IWDIFICATION | v11 | v11 | unchanged (installed = latest) |
| PAINA | — | v1.9 | upgrade ? -> v1.9 |
| RANDOMISER | 7 | maintained fork v8.1.1 @ `f4a9dfb` | Immutable release asset pinned; same component ids and documented EEex-v1.2 legacy-BCS fallback. Clean rebuild and new-game live validation remain open. |
| RR | v4.92 | v4.92 | unchanged (installed = latest) |
| SAFANA | v0.5 | v05 | unchanged (tag is spelled v05); SoD-items carryover fix component still to be built |
| SARAHTOB | v8 | v8 | Official v8 IEMOD frozen; custom portraits remain a separate blocked/manual asset decision. |
| SIRENE_BG2 | — | commit `00beda90` | Exact official commit archive frozen; the separate BG1 package remains a curation decision. |
| SPELL_REV | v4.19 | v4.21-chriz.1, pending release | Publish a fetchable immutable fork source containing the protection-refresh/subspell work. |
| STRATAGEMS | 35.21 | v35.21 | unchanged; MUST run under WeiDU 249 (breaks on 251); unverified-by-author on 2.7, community installs work |
| UB | v28 | v28 | Official v28 release ZIP frozen; component `19` remains omitted with Ascension `40` pending the canonical provider decision. |
| XAN | v19 | v19 | unchanged (installed = latest) |
| YESLICKNPC | v5.0 | v5.0 | unchanged (installed = latest) |

## Chriz-layer repos

Pinned at authoring time to a tagged release (or SHA) of each own repo:
ABETTORHLAREBALANCE, AKCB_BERSERKER, AKCB_SHAPESHIFTER, AURA_BALANCE_PATCH_SPELLS, BG2EE-EET-FIXPACK, CHRIZ-BG-MODPACK, CHRIZ-BG-REBALANCE, CHRIZ-SOD-REMIX, EEEXREMOTE.

## Local hotfixes (22)

No public versions to pin — each needs its home decision (worksheet section 4)
before it can be pinned to a chriz-repo release.

## Remaining freeze checks

- Recheck packaged EET and EE Fixpack releases immediately before manifest freeze; do not
  silently follow moving branches.
- Publish or pin every selected Chris-owned fork listed in the central follow-up queue.
- A reviewed target is not runtime acceptance: EEex 1.2, SCS 35.21/WeiDU 249, both EE
  Fixpack runs, and the EET merge still require the staged 2.7 tests.

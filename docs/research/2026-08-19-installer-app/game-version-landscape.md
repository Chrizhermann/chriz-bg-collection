# BG:EE Modding Version Landscape — as of 2026-08-18

## 1) Game patch: 2.7 shipped

- **Patch 2.7 (build 2.7.3.0)** for BG1:EE, BG2:EE and IWD:EE **released June 23, 2026** on PC + mobile ([Beamdog announcement](https://forums.beamdog.com/discussion/90724/2-7-baldurs-gate-icewind-dale-ees-update-new-languages-improved-cloud-support-mobile-modding), [PC Gamer](https://www.pcgamer.com/games/baldurs-gate/beamdog-swaggers-in-says-sorry-im-late-then-surprise-patches-baldurs-gate-1-and-2-enhanced-14-years-after-release/)). First patch since 2.6.6 (2021). No 2.7.4 hotfix found as of today; current build = **2.7.3.0** (confirmed by EEex release notes naming "EE v2.7.3.0").
- Content: QoL-focused — new languages/dynamic language system, Apple Silicon native, cloud saves, **mobile modding** (iOS `home:/` override dir; Android needs full reinstall), fixed an off-by-one error in rebuilt BIF files that affected community tools. Not a deep engine rework.
- **Mod breakage:** the patch **wipes installed mods** (mod strings show as "Invalid" — [G3 rescue thread](https://www.gibberlings3.net/forums/topic/41430-how-to-save-your-game-if-the-27-patch-wiped-your-mods-invalid-texts-are-showing)); modded installs must stay pinned on their patch level. Most WeiDU mods install fine on 2.7; known 2.7-specific fixes were needed in EET (SoD GUI) — see below. **Installer implication: pin the game build (keep an untouched 2.6.6.0 or 2.7.3.0 base copy) and disable auto-update.**

## 2) EET — Enhanced Edition Trilogy

- Repo moved to the G3 org: **[Gibberlings3/EET](https://github.com/Gibberlings3/EET)** (k4thos/EET now redirects there). Maintained by community (Argent77, CamDawg, morpheus562).
- **Latest tagged release: v14.1 (2025-04-07)**; v14.0 (2025-03-29) was the big jump from 13.4 (2021), adding EE Fixpack compatibility.
- **2.7 status: v14.1 predates 2.7 and is NOT 2.7-clean; fixes live only on `master`, untagged.** Master commits: "Fix patch 2.7 compatibility issue in SoD GUI" (2026-07-01), "WeiDU 251 compatibility fix" (2026-08-01), campaign-screen songlist fix (2026-08-06). Per the [G3 thread "When will EET be made compatible for 2.7?"](https://www.gibberlings3.net/forums/topic/41363-when-will-eet-be-made-compatible-for-27/) (June 23–24, 2026), Argent77 patched it within a day of the 2.7 drop. **Installer implication: pin a master commit SHA (≥ 2026-08-06) or wait for v14.2; do not use the v14.1 tarball on a 2.7 game.**

## 3) EEex — ⚠ patch-version-locked

- **[Bubb13/EEex](https://github.com/Bubb13/EEex)** left alpha: **v1.0.0 stable on 2026-05-10**; rapid releases since; **current: v1.2.0 (2026-08-12)**.
- **Patch targeting:** v1.0.0 targets 2.6.6.0. **v1.1.0 (2026-07-23) added support for EE v2.7.3.0 while remaining compatible with 2.6.6.0** — dual-target. v1.1.5 (2026-07-29) fixed InfinityLoader recognition of the **GOG** 2.7 BG2:EE executable. Windows-only as before.
- Breaking/structural changes since alpha: EEex scripts moved from `override/` to **`EEex_scripts/`** (v1.0.0 — prevents Generalized Biffing breakage); spell-state limit lifted 255 → 2^32; FPS uncap feature; new hook `EEex_Opcode_AddDeferredListsResolvedListener()`.
- ⚠ **Old Near Infinity versions can corrupt EEex saves** (EEex ≥ 0.10.3-alpha stores data in a nonstandard location) — pin latest [Argent77/NearInfinity](https://github.com/Argent77/NearInfinity/releases/latest).
- **Installer implication: pin EEex ≥ v1.1.5 for 2.7.3.0 games (≥ v1.1.0 minimum); ≤ v1.0.0 for a frozen 2.6.6.0 stack — verify per-release notes, the lock is on the game binary.**

## 4) SCS / Spell Revisions / cdtweaks

| Mod | Current | Date | Host |
|---|---|---|---|
| **SCS (Stratagems)** | **v35.21** | 2024-11-21 | [Gibberlings3/SwordCoastStratagems](https://github.com/Gibberlings3/SwordCoastStratagems/releases) (archival mirror, releases only) + [G3 downloads](https://www.gibberlings3.net/files/file/914-sword-coast-stratagems/) |
| **Spell Revisions** | **v4.21** | **2026-08-17** | [Gibberlings3/SpellRevisions](https://github.com/Gibberlings3/SpellRevisions/releases) |
| **cdtweaks (Tweaks Anthology)** | **v18** | 2025-08-22 | [Gibberlings3/Tweaks-Anthology](https://github.com/Gibberlings3/Tweaks-Anthology/releases) |

- **SCS**: no release in ~21 months; no 2.7-specific release. ⚠ **Known incompatibility: SCS 35.21 + WeiDU 251.00 → ToB "Smarter" boss components (Illasera, Gromnir, Yaga-Shura, Abazigal, final villain) fail with `finbalth.bcs.BAF` PARSE ERROR** ([G3 thread, 2026-03-23](https://www.gibberlings3.net/forums/topic/41138-is-anyone-installing-scs-after-the-recent-weidu-update)). **Pin WeiDU 249 for SCS or verify a fixed 252.01 nightly.**
- **Spell Revisions**: left its ~6-year v4 beta — first stable **v4.19 "Grown-Up release" 2025-11-25** (fixed opcode-324 CTDs), v4.20 (2025-12-10), v4.21 (2026-08-17, description/strref fixes). Actively maintained.
- **cdtweaks**: v17 (2025-08-07) ended a 3-year gap since v16; v18 fixed two crash-severity v17 bugs and added 4 components. Has EEex-enhanced component variants.

## 5) EE Fixpack — still beta, active

- **[Gibberlings3/EE_Fixpack](https://github.com/Gibberlings3/EE_Fixpack)** — **NOT final-released.** Alpha 1 (2025-04-01), Alpha 2 (2025-04-23), Alpha 3 (2025-10-15), **Beta 1 (2026-05-02), Beta 2 (2026-05-04, current)**. Active commits through 2026-08-14 (DLC-Merger detection fix). [G3 download page](https://www.gibberlings3.net/files/file/1046-enhanced-edition-fixpack/) lists Beta 2; no explicit 2.7 statement.

## Cross-cutting: WeiDU itself is now a compatibility axis

- **WeiDU v251.00 released 2026-03-01** ([WeiDUorg/weidu](https://github.com/WeiDUorg/weidu/releases)) after years on 249 — it **broke SCS 35.21** (above) and required an EET master fix (2026-08-01). 252.01 nightlies running since March 2026. **Installer implication: pin WeiDU version per mod, don't blanket-upgrade the `setup-*.exe` stubs; 249.00 remains the safe default for the frozen 2.6.6 reference stack.**

## Pin-list summary for the installer app

| Component | Pin for a 2.6.6.0 stack (matches reference install) | Pin for a 2.7.3.0 stack |
|---|---|---|
| Game | 2.6.6.0 (block auto-update) | 2.7.3.0 (2026-06-23) |
| EEex | ≤ v1.0.0 (or any v1.1.0+, dual-target) | **≥ v1.1.5** |
| EET | v14.1 OK | **master @ ≥ 2026-08-06**, no tag yet |
| SCS | 35.21 + **WeiDU 249** | 35.21 + WeiDU 249 (unverified on 2.7 by author; community installs work) |
| Spell Revisions | v4.21 | v4.21 |
| cdtweaks | v18 | v18 |
| EE Fixpack | Beta 2 (beta risk) | Beta 2 (beta risk) |
| Near Infinity | latest (EEex save-corruption guard) | latest |

Sources: [Beamdog 2.7 announcement](https://forums.beamdog.com/discussion/90724/2-7-baldurs-gate-icewind-dale-ees-update-new-languages-improved-cloud-support-mobile-modding) · [PC Gamer on 2.7](https://www.pcgamer.com/games/baldurs-gate/beamdog-swaggers-in-says-sorry-im-late-then-surprise-patches-baldurs-gate-1-and-2-enhanced-14-years-after-release/) · [EEex releases](https://github.com/Bubb13/EEex/releases) · [EET repo](https://github.com/Gibberlings3/EET) · [EET 2.7 thread](https://www.gibberlings3.net/forums/topic/41363-when-will-eet-be-made-compatible-for-27/) · [SCS releases](https://github.com/Gibberlings3/SwordCoastStratagems/releases) · [SCS/WeiDU-251 thread](https://www.gibberlings3.net/forums/topic/41138-is-anyone-installing-scs-after-the-recent-weidu-update) · [SpellRevisions releases](https://github.com/Gibberlings3/SpellRevisions/releases) · [Tweaks-Anthology releases](https://github.com/Gibberlings3/Tweaks-Anthology/releases) · [EE_Fixpack repo](https://github.com/Gibberlings3/EE_Fixpack) · [EEFP download page](https://www.gibberlings3.net/files/file/1046-enhanced-edition-fixpack/) · [WeiDU releases](https://github.com/WeiDUorg/weidu/releases) · [2.7 mod-wipe thread](https://www.gibberlings3.net/forums/topic/41430-how-to-save-your-game-if-the-27-patch-wiped-your-mods-invalid-texts-are-showing) (GitHub dates pulled via `gh api`, 2026-08-18)
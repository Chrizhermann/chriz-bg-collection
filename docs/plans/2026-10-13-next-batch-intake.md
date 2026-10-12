# Next batch intake — 13 October 2026

Intake notes for the next combined CEBG update. Everything below is a **local,
unpublished candidate**. No owner release, PR merge, CEBG pin, website change,
game install or save change was made. Public pins wait for each owner's
publication; then record the published URL and the SHA-256 of the published
asset, not of these local files.

Local archives live under `C:\Users\chris\.codex\artifacts\next-batch-20261013\`
(Bardic: `...\bardic-blade-battleluck-20261012\`). They are private build outputs,
not distribution sources.

## Candidates

| Mod | Version | Source (branch @ commit) | Local archive | Automated evidence |
|---|---|---|---|---|
| Chriz BG Modpack | v0.2.0-alpha.11 | `codex/campaign-scroll-integration` @ `1f0e8db` | `modpack\chriz-bg-modpack-v0.2.0-alpha.11.zip`, 10,431,317 bytes, SHA-256 `a1bd5e59d5e44e7b264d697cc3e7aa958af5415777a8e9cd32ae4afede2583a6` | Full suite 2,561 passed, 2 skipped (both need a private installed game); 20 release-archive checks pass against this ZIP; reproducible build |
| SoD Remix | v0.6.13 | `claude/sod-integration-oct13` @ `fd97c3d` | `sod\chriz-sod-remix-v0.6.13.zip`, 3,010,793 bytes, SHA-256 `a99cd7480b1c8eae1feefbdd120afe64185b36c72314bd2c023ae8dbcb295255` | 345 passed, 3 skipped (Skie capture env), 16 failed: the known component 115 fixture failures, identical on untouched v0.6.12; research scripts 41 passed; reproducible build |
| BG Rebalance | v0.8.2 | `codex/foa-compatibility-20261008` @ `c5a9169` (PR7 branch, pushed) | `rebalance\chriz-bg-rebalance-v0.8.2-windows.zip`, 1,757,530 bytes, SHA-256 `1577243caa0e503c1946ce4fa803952c87a6c4ccb1530141665e2b1d933661f1` (tails: `CBR_SR_HARDINESS-v0.8.2-windows.zip` `987fbf02…`, `CBR_SR_ANTIMAGIC-v0.8.2-windows.zip` `bc97f59c…`) | 565 tests OK, 24 skipped; reproducible build; see the Flail section |
| Bardic Wonders | v2.9c-balance.7-rc.1 | owner worktree `bard-spell-blade-followup` (uncommitted) | `Bardic-Wonders-v2.9c-balance.7-rc.1.zip`, SHA-256 `d3425e82452d8862372e716f7ee68ba2d9fa24e4c4c7bb43969955d35d206879` | Owner: 148 tests, 48 installer checks, archive verification; see its `DELIVERY.md` |

Already on this CEBG branch (local, unreleased): `6c61c94` Emotion 301
optional/off; `74e9aae` bard spell-level ceilings 420/421 + Bardic 3010 wired
in; `1f1b633` those ceilings checked by default (Christopher, 13 October).

## Modpack v0.2.0-alpha.11

Player changes: Fade/Evandra first-recruitment XP catch-up (237/238, new);
reader-level scroll casting replaces the fixed-level 430; experimental campaign
XP banking 650 and optional SoA cap 651 (new, off by default).

Recipe work at intake:

- **Artifact limits must rise.** The archive now carries the scroll-policy tool
  `CBMSPolicy.exe` (8,232,364 bytes). Measured: 136 entries, 12,256,356 bytes
  uncompressed, largest entry 8,232,364, depth 4, max ratio 6. The alpha.10
  record allows 128 entries, 4 MiB per entry and 8 MiB total, so use for
  example 256 entries, 16 MiB per entry and 16 MiB total. The SHA-256 pin still
  fixes the exact bytes; keep depth and ratio limits.
- **Defender scan before publication.** `CBMSPolicy.exe` is a PyInstaller
  executable run by the installer; scan the published archive the same way as
  the alpha.19 package.
- **430 must move to the end.** At install time 430 records SHA-256 hashes of
  the scroll items and the spells they cast; startup refuses a stale policy if a
  later component changes them. Today 430 runs in `chriz-bg-modpack-bg2`, before
  `cdtweaks-spell-save-penalties-bg2` (2310–2312) and `spell-rev-lightning-bg2`
  (80/81), which edit spells. Give 430 its own final post-EET-end run after
  every spell/item writer (after BuffBot unless shown unnecessary).
- **430's feature must change with it.** Today it is "Give Use Any Item scrolls
  a fair caster level", mandatory, requiring Tweaks Anthology 2170. The new 430
  no longer uses 2170: it needs Windows EEex (it fails without `EEex.dll`) and
  sets the level for every scroll reader, casters included. Replace the
  requirement with EEex and rewrite the title/description. It becomes
  **default** (was mandatory; Christopher, 13 October).
- **237/238:** add to `chriz-bg-modpack-late-companions-bg2` in TP2 order,
  `[189, 237, 238, 620]`, after EET finalization and the companion writers.
  Both require EET. **Default** whenever Fade/Evandra are selected.
- **650/651:** a new run after the late-companions run (after 237/238/620).
  650 requires EET, Windows EEex and `XPLEVEL.2DA`; 651 requires 650.
  **Ship now as experimental, optional and unchecked.** A "Banked XP" line on
  the character record screen is in progress (`claude/xp-bank-display`); the
  no-bank dual-class rule still waits for Christopher's choice.
- Catalog: add 237, 238, 650, 651; update 430's title to reader-level casting.
- Verify order with `tools/check_native_component_order.py` against the
  published archive, as for earlier pins.

## SoD Remix v0.6.13

Player changes: early BG1 companion recruitment and Skie before the first
rest (198); report Korlasz to Liia after the first night (177); Avernus portal
and a bounded final battle (285); relocated jailbreak/camp-chest loot keeps its
identification state (170/900, no recipe change).

- Artifact fits the v0.6.12 limits (283 entries, 6,357,867 bytes uncompressed,
  largest 1,364,992).
- Catalog: add 177, 198, 285.
- `chriz-sod-remix-bg2` insertions in TP2 order: 177 after 175, 198 after 197,
  285 after 280. Do not add existing-save repairs 176/291/901.
- Dependencies: 177 needs 175 + 180; 198 needs 110 + 197; 285 needs the SoD
  areas. All three are for a new SoD start (285: before first entering BD4700).
- Selection: **on by default**, like Christopher's other SoD changes. Existing
  SoD Remix rows are mostly mandatory; whether these three are mandatory too or
  default (deselectable) is asked. The harder pieces such as 257 (Insane bridge
  sequencers) stay optional and belong to Challenge Mode.

## BG Rebalance v0.8.2 (Flail correction)

PR7 is a compatibility repair of shipped component 302, not a new balance
change. Review found one real regression, now fixed locally: the PR's stricter
Haste check left the +5 Flail immune to real Haste spells. In vanilla these are
`SPRA301`, `SPIN572` and `SPIN828`; with Spell Revisions, `SPIN572`. Released
v0.8.1 removes those immunities. Their companion records (fatigue, glow/sound,
Slow cancel, self-guards) now count as part of Haste. Checked with the real
spells and the captured CEBG/vanilla items. Spells with other effects keep the
immunity, and the installer now says so.

- No catalog or run change; 302 stays a default. Pin update only.
- The fix and v0.8.2 preparation are pushed to PR7 (`c5a9169`, with
  Christopher's permission). Merge and release still need his go.
- Local package fits the v0.8.1 limits (57 entries, 2,469,967 bytes
  uncompressed, largest 1,364,992, depth 4, ratio 16) and its DESIGNATED order
  equals the v0.8.1 pin.

## Bardic Wonders v2.9c-balance.7-rc.1

Blade proficiency limits (two pips in the listed weapons, Two-Weapon Style
maximum three) and Battleluck (five rounds, caster and allies within 30 ft,
refresh not stack). Changes components 1001/2008; no new components.

- Artifact fits the balance.6 limits (838 entries, 33,031,503 bytes, largest
  3,311,752, ratio 431).
- Pin update only. 3010 wiring is already on this branch.
- Owner source is still uncommitted; commit/tag before publication.

## Decisions (Christopher, 13 October 2026)

1. Bard spell-level ceilings (420/421 + Bardic 3010): **default** (done, `1f1b633`).
2. Recruitment XP 237/238: **default** when Fade/Evandra are selected.
3. Banking 650/651: **ship now, experimental, optional and unchecked.** Show the
   bank in the character record next to XP.
4. SoD 177/198/285: **on by default**; mandatory vs default still to confirm.
5. Reader-level scrolls 430: **default** (was mandatory).
6. Challenge Mode: a big choice at the top of the installer that switches all
   members on/off; members stay individually changeable and are marked as part
   of Challenge Mode; off by default (Christopher turns it on for his stream
   installs). See [the group design](2026-10-13-challenge-mode-group.md).

Still open: the no-bank dual-class rule for 650, and the extra Degrodel fight
layer (new Modpack component in progress: let Degrodel leave as in vanilla, or
refuse the helm and fight him with his servants).

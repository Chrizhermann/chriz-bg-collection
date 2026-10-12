# Combined test — October 13 batch

Prepared, not started. The install waits for the disk cleanup and
Christopher's go. No existing game, save or stream install is touched.

## The build

- One fresh SR-enabled EET installation in a new test folder (for example
  `C:\Users\chris\CEBG-Tests\Combined-20261013`, roughly 25 GB), installed with
  the CLI from a frozen development recipe. Unpublished candidates enter as
  hash-pinned local archives through the normal verified manual intake, as in
  the [September combined test](2026-09-08-combined-playtest-and-alpha16.md).
- Selection: Chris's Recommended **plus Challenge Mode on** (his stream setup).
- Candidates (see the [intake notes](2026-10-13-next-batch-intake.md)):
  Modpack v0.2.0-alpha.11, SoD Remix v0.6.13, Rebalance v0.8.2, Bardic
  Wonders balance.7-rc.1. Degrodel 660 and the Banked XP display only if
  Christopher includes them in this batch.
- The recipe intake (new runs/features, 430 moved to the end, raised Modpack
  archive limits) is the same work the next release needs; only the artifact
  sources differ (local archives now, published URLs at release).

## Short player checks

Pick what matters; a short first session is enough. Use a save before each
scenario so both outcomes can be tried.

1. **Start:** the game starts with EEex, no Lua error box; save and reload.
2. **Banked XP:** on a BG1 character the record screen's Class tab shows
   "Banked XP: 0" and the 161,000 limit. Push XP past the cap (console): a
   notice appears, the bank grows, save/reload keeps it.
3. **Scrolls:** a mage and a fighter read the same scroll; the mage casts at
   their level, the fighter at half their level.
4. **Fade/Evandra (SoA):** first recruitment lifts them to the protagonist's XP
   threshold, once.
5. **Bards:** a Blade or Jester levels up with the right spell levels;
   Battleluck lasts five rounds, covers allies within 30 ft, recasting
   refreshes it.
6. **Degrodel** (if included): hand over the helm (normal ending) and refuse
   it (he and his servants fight; journal note; his loot).
7. **Flail of the Ages +5:** Haste still works on the wielder.
8. **SoD:** Skie joins before the first rest; Liia accepts the Korlasz report
   after the first night; the Avernus portal skips the lift; relocated
   jailbreak/camp loot keeps its identified state.
9. **Challenge Mode parts:** Emotion recast replaces the other Emotion;
   Baeloth's spell list is legal; the Insane bridge sequencers appear on
   Insane (dragon melee only if a dragon fight is reached).

Record each result in a short line (pass, problem, not tried). Problems go to
the owning repo, not into a new full reinstall.

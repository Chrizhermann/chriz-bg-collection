# Combined test — October 13 batch

Installed in a disposable lab on 13 October (approved after the disk
cleanup). The first game start waits for Christopher's go. No existing game,
save or stream install is touched.

## The build

- One fresh SR-enabled EET installation, "CEBG Lab 2026-10-13 B", at
  `C:\Users\chris\CEBG-Tests\Combined-20261013-b`, installed with the CLI from a
  frozen local recipe: the pending [recipe intake](2026-10-13-next-batch-intake.md)
  (branch `claude/next-release-intake-20261013`) plus the playtest-tool fix,
  merged on lab branch `claude/lab-20261013`. Sources, recipe and logs are in
  the ignored `target/combined-playtest-20261013/`.
- Selection: Chris's Recommended, **Challenge Mode on** (his stream setup) and
  **banking 650 on** so it can be tested (651 stays off). EET's desktop
  shortcut is off: it would replace the Desktop `EET.lnk`, which points at
  Chriz Easy BG Test 19. Result: 55 runs, 474 components (27 BG1 + 447 BG2).
- Start the game with `InfinityLoader.exe` in the game folder (EEex). Steam's
  `steam_api64.dll` is renamed `.bgse-disabled`; the game has its own profile
  name, so saves stay separate.
- Four unpublished candidates enter as hash-pinned local archives through the
  verified manual intake. Every other mod uses its published pin.

| Candidate | Source | Archive SHA-256 |
|---|---|---|
| Modpack v0.2.0-alpha.11: 237/238, 430, 650/651, 660, Banked XP display, banking start fix | `claude/alpha11-integration` @ `f38d7db` | `71abd5fd…` |
| SoD Remix v0.6.13: 177/198/285 mandatory, relocated loot keeps its identification | `claude/sod-integration-oct13` @ `fd97c3d` | `a99cd748…` |
| BG Rebalance v0.8.2: Flail of the Ages fix (PR7) | `codex/foa-compatibility-20261008` @ `c5a9169` | `1577243c…` |
| Bardic Wonders v2.9c-balance.7-rc.1: Blade, Battleluck | balance.6 `7db9861` plus its recorded uncommitted overlay | `d3425e82…` |

- Checked before installing: the four archives' TP2 component order matches
  the lab recipe, the recipe validates with no findings, and every Challenge
  Mode member resolves on.
- Install result: every run succeeded and the final verification passed.
  Degrodel 660 found the hand-in in the installed dialog (state 7, four
  servants). Scroll casting 430 analysed this real Spell Revisions install:
  470 scrolls, no errors; the standard learning and protection scrolls
  classify as expected, and one third-party scroll (`TTSCRL01`) keeps its
  normal behavior because learning and casting share an ability.
- Two earlier attempts stopped and are not used: `Combined-20261013` (15.9 GB,
  stopped at EET's desktop shortcut; its profile folder
  `OneDrive\Documents\CEBG Lab 2026-10-13 - c2f5148bcdbf` holds only a marker)
  and `Combined-20261013-r2` (empty, name already taken). All can be deleted.
- The installation writes EET's install state into the standard EET profile, as
  every EET install does; it ended in the normal finished state (2). Its Cheats
  and Debug Mode were already on before this test.

## Short player checks

Pick what matters; a short first session is enough. Use a save before each
scenario so both outcomes can be tried.

1. **Start:** the game starts with EEex, no Lua error box; save and reload.
2. **Banked XP:** on a BG1 character the record screen's Class tab shows
   "Banked XP: 0" and the 161,000 limit. No Banked XP line means banking could
   not start; note it (Imoen's and Fade/Evandra's XP keep working). Push XP
   past the cap (console): a notice appears, the bank grows, save/reload keeps
   it.
3. **Dual class without a bank:** dual a Fighter 7 at 80,000 XP in BG1. As in
   the unmodded game, the old class counts at its level threshold (64,000), so
   the new class has 97,000 XP of room before banking starts.
4. **Scrolls:** a mage and a fighter read the same scroll; the mage casts at
   their level, the fighter at half their level.
5. **Fade/Evandra (SoA):** first recruitment lifts them to the protagonist's XP
   threshold, once.
6. **Bards:** a Blade or Jester levels up with the right spell levels. On a
   fresh Blade, listed weapons stop at two pips and Two-Weapon Style at three.
   Battleluck cast twice near an ally and an enemy: only allies within 30 ft
   benefit, the bonus does not double, and it ends five rounds after the last
   cast.
7. **Degrodel** (default): hand over the helm (normal ending) and refuse it (he
   and his servants fight; journal note; his loot).
8. **Flail of the Ages +5:** Haste still works on the wielder.
9. **SoD:** Skie joins before the first rest; Liia accepts the Korlasz report
   after the first night; the Avernus portal skips the lift; relocated
   jailbreak/camp loot keeps its identified state.
10. **Challenge Mode parts:** Emotion recast replaces the other Emotion;
    Baeloth's spell list is legal; the Insane bridge sequencers appear on
    Insane (dragon melee only if a dragon fight is reached).

Record each result in a short line (pass, problem, not tried). Problems go to
the owning repo, not into a new full reinstall.

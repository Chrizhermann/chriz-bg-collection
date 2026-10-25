# IWD:EE fight audit (2026-10-25)

Read-only audit of IWD:EE 2.7.3.0 game data and SCS v35.21 source for the Chriz Easy IWD combat mod. Spoilers throughout. Decisions taken since are in the [v1 plan](../plans/2026-10-25-chriz-easy-iwd-v1.md).

Markers: **[V]** verified in IWD:EE game data I parsed read-only; **[S]** verified in SCS v35.21 source (paths relative to `stratagems\`); **[I]** inference; **[L]** needs a lab check. Nothing was installed, launched or run. Spoilers throughout.

# PART A: Summary

## A1. Key findings
1. **SCS reaches most actors, but bosses mostly get only generic AI.** 84% of the 3,266 actors in my 53 fight groups get some SCS AI, mostly the generic 6020 layer [V/S].
   - **27 of 188 boss-flagged creatures get no SCS AI at all.** Examples: Poquelin in the Ascension Cave (L7), the Idol (L6), Lord Maluradek (T-3), Mebdinga and the wailing virgins (H4), the Cryshal Sentries (F2), Edion Caradoc (H5). The Luremaster (T-8) is effectively in this group too: SCS only strips his teleport script.
   - **95 more get only the generic layer.** Examples: Joril (W2), Maiden Ilmadia (L5), Marketh (L3), the Lizard King (D1).
2. **SCS already has boss-specific work** for Belhifet and Yxunomei (6510), Icasaracht and her soul gem (6540), and Malavon, the Dorn's Deep lich, Presio and Kontik (6030/6040) [S]. Those fights need an encounter layer (minions, phases, pacing), and it must sit on top of SCS's rewritten scripts.
3. **The override-array lead is not a bug** [S]. The writer at `caster_shared\caster_shared.tph:266` uses three keys and the reader at `:361-362` uses two. WeiDU joins array keys with underscores, so both name `override_spellchoice_mage_<RESREF>`. The real defects are key mismatches, now checked against game data:
   - The D3PRES rows match nothing (no D3PRES.CRE [V]), so Presio loses his priest Poison x2.
   - The LSHAM1/LSHAM2 rows match nothing (the CREs are LMSHAM1/2 [V]).
   - The SCS priest list names SDELFPS1, which doesn't exist; the game has SHELFPS1 [V]. The Shadowed Elven Priest therefore gets no AI.
   - The jackal-shaman top CAJSHA1M is never applied, although CAJSHA1M.BCS exists [V].
4. **The base finale is decoded end to end** [V] (B2.1).
5. **Vanilla Belhifet is predictable** [V code, I behaviour]. He melees whoever he saw last and casts one random free spell after each teleport. His teleport script EEBELWA has seven blocks with identical triggers, so only the first can fire: a hop about every 9 s to one of five points; the sixth point is never used.
6. **SCS 6510 swaps two boss models** [S/V]:
   - Belhifet: RAVER (0x7F2E) replaces BELHIFET (0xE256) (`fiend\fiend_shared.tph:355`).
   - Yxunomei: MARILITH (0xE090) replaces MARILITH_IWD (0xE25C).
   - How this looks on stream needs a lab look [L].
7. **IWD:EE already scales group size by difficulty** [V]. One-shot "remove me below difficulty X" scripts (EFDELNOR, EFDELHRD, EFDELINS, EFDELDEF, EFDELEAS) sit on 234 actors and INI critters. EFDELDEF also keeps the actor at every difficulty once Heart of Winter is finished (SPRITE_IS_DEADGEM). Extra actors in our mod can reuse these scripts.
8. **The HoW and TotL finales are already multi-phase** [V]: dragon, then soul gem with summons and a healer court; Luremaster plus spectral heroes with up to six capped replacements. The upgrade value there is pacing and coordination.
9. **SCS ships no separate license** [S]. The readme states David Wallace's copyright, asks for full credit when code is reused and asks that SCS not be mirrored. We can reuse ideas, not files.

## A2. Notable fights
Coverage levels:
- **covered:** the boss has boss-specific SCS work.
- **partly:** the boss gets only generic 6020, or key members get no AI.
- **none:** the boss gets no SCS AI.

Effort: **S** = one creature or small group, one added script layer. **M** = boss plus minions or phases, plot hooks to keep, separate SCS-present and SCS-absent branches. **L** = a large set piece with several SCS-rewritten scripts and difficulty gating.

Full row text (spawn, patterns, risks, evidence) is in `notable-fights.csv` and `notable-fights.json`.

| ID | Area | Fight | Key creatures | SCS today | Potential | Effort |
|---|---|---|---|---|---|---|
| K3 | AR2100 | Orog raid on Kuldahar (after Yxunomei dies) | KAOROG x7, KAOROGE x3, allied defenders | none | Medium | S |
| V2 | AR3001 | Lysan | LYSAN CLERIC 8, 10 yetis | covered | Low | S |
| V4-6 | AR3501-3503 | Kresselack's Tomb | MYTOS, KTSMG2, KTWGTI1, mummies, spectral guards; KRESS is a plot actor | covered | Low | S |
| D1 | AR4001 | Lizard King | LIZKING FTR 8 63 HP, 14 shamans, ~60 lizard men | partly: King generic, guard looks inverted | Medium | M |
| D3 | AR4003 | Presio | PRESIO C/M 12 80 HP, 5 undead lieutenants, 60 cold wights | covered (override rows dead) | Medium | M |
| D4 | AR4004 | Eldathyn sanctuary | High Ritualist/Summoner, Geelo, 4 yuan-ti mages, Marchon's party | covered | Low | S |
| D5 | AR4005 | Yxunomei and her temple | TANARRI C/M 13 91 HP MR 70; 2 high priests, 14 priests, archers, champions | covered (6510, model swap) | Med-High | M |
| S2 | AR5104-5502 | Elf towers | 5 wizards, 4 clerics, priest SDELFPR, officers, 16 archers, Kaylessa | partly: priest gets no AI | Medium | M |
| U2 | AR6002/6003 | Orog barracks | Krilag, Saablic Tan, 5 drow sorcerers, 10 spellswords, 10 orog generals | covered | Medium | M |
| U3 | AR6005/6006 | Lich tomb | LICH2 MAGE 18, 5 greater mummies, 4 bronze sentries | covered | Low | S |
| W1 | AR7001/7002 | Museum | Kerish, Vera, ~40 frost salamanders, invisible ambushers | partly | Low-Med | S |
| W2 | AR7004 | Joril's throne room | JORIL GIANT 18 145 HP, 2 bodyguards, Gorg, Frostbite, 4 wyrms, Kontik, 5 Black Ice Knights, 16 frost giants | partly: all generic except Kontik | High | L |
| L2 | AR8009/8010 | Malavon | MALAVON MAGE 19 MR 86, simulacrum, stalkers | covered | Low | S |
| L3 | AR8005-8007 | Marketh's palace | Marketh THF 15, Fleezum/Flozem, Seth, 29 thieves, 4 Kraken mages | partly | Medium | M |
| L5 | AR8011 | Ilmadia's forge | Ilmadia FTR 12, 9 fire giants, 2 drow priest-mages, 11 elementals | partly | Med-High | M |
| L6 | AR8012 | The Idol | IDOL MAGE 16 150 HP; 5 zombie lords, 6 greater mummies, 48 boneguards | none (boss) | High | M |
| L7 | AR8013 | Poquelin, escapes at 50% | POQUELIN IMP 15 250 HP; 2 bronze sentries; 5 INI waves | none (boss) | High | M |
| F2 | AR1103 | Cryshal-Tirith | Pomab + 5 images; 4 Cryshal Sentries | none | Medium | S |
| F3 | AR1105 | Finale: Belhifet | BELHIF IMP 15 350 HP AC -12; 2 cornugons; 2 iron golems | covered (6510) | Highest | M-L |
| H1 | AR9100/9101 | Shar cult party, Emmerich | Vaarglan MAGE 18, Alpheus, Priestess of Shar, mercs; Emmerich the werewolf | partly | Medium | M |
| H3 | AR9300 | Burial Isle | Skaldar CLR 15, Polar Bear Spirit, spirits, 21 barrow wights, 5 ghost shamans | partly | Medium | M |
| H4 | AR9400 | Mebdinga's barrow | MEBDING SPECTRE 15; ~36 wailing virgins | none | Med-High | S |
| H5 | AR9301 | Edion (optional) | EDION MAGE 24 54 HP | none | Medium | M |
| H10 | AR9604 | HoW finale | ICASA 400 HP; soul gem; priestess, king, underpriestesses, guards | covered (6540/6040) | Medium | M |
| T-2 | AR9704-9708 | Tower trials | Harald, Harpy Queen, Criek, Adran (+djinni), Shelhai | partly | Medium | M |
| T-3 | AR9712 | Lord Maluradek | LORDM 250 HP MR 75; 3 beholders; 8 animated plate | partly: boss none | High | M |
| T-6/7 | AR9800/9801 | Jackal caverns | Rikasha, 3 summoners, 7 shamans, pack leader, 5 infernal harpies | partly | Medium | M |
| T-8 | AR9711 | TotL finale | TRGBARD BARD 20 420 HP; spectral heroes; 7 guard groups; rakshasas | partly: teleport script removed only | Med-High | M |

Probably wasted effort:
- Early fights (E1, K1, K2, V1-V3, T1).
- Bosses SCS already handles specifically (L2, U3, D4).
- Dialogue-only actors: Wylfdene (H2) has no combat scripts; Kresselack has MR 100 and every resistance 100.
- Pure packs (W3, H6-H8, T-1, T-4, T-5).

## A3. Final bosses: facts and options

**Base game: Belhifet (AR1105)** [V]
- **Stats:** BELHIF is IMP 15, 350 HP, AC -12, THAC0 3, 3 APR. His +5 weapon does 1d10+5 with poison and disease. MR 50, fire 100, cold 50.
- **Spell immunity:** his ring BRING gives immunity to spell levels 1-9 (op102 x9) plus 44 named spells. Party spells cannot touch him.
- **Other protection:** his amulet BEHAMU makes him immune to non-magical, +1 and +2 weapons (op120 type 2 plus type 0 with enchantment 1 and 2, the same set as the spell Mantle, SPWI708), so only +3 or better hits him. SCS's pit-fiend macro leaves the amulet alone. A min-HP ring keeps him at 1 HP.
- **What the party can have by then** (base game, without random treasure): about 17 melee weapons of +3 or better as loot, almost all from the later chapters, plus thirteen single +3/+4 weapons at two stores (base prices 5,100-42,000 gold, sold at 150-180 %). +3 ammunition is loot only: Arrows of Piercing +3 from one late group of archers (up to 190, minus what they shoot). There are no +3 bolts, bullets, darts or launchers. Arrows, bolts and bullets +2 can be bought in the later chapters.
- **Arrival:** Poquelin talks. A dialogue branch opens the portal and walks two cornugons out of it.
- **Transformation:** cutscene EEPOQCNG casts an unresisted Dispel Magic on the nearest PC. SPIN183 (opcode 151) turns Poquelin into Belhifet, and the two iron golems turn hostile.
- **Minions:** golems melee and drop a poison cloud every 49 s. Cornugons only melee.
- **Cornugons by difficulty:** they are removed below Hard unless HoW is finished.
- **End:** HP < 10 runs EEBELDTH, then EEBELKIL: 75,000 XP, end movies.
- **SCS 6510 today** [S]:
  - Pit-fiend macro (model RAVER) and a new spell set.
  - EEBELWA stripped; EEBELHI1 rebuilt as SCS's pit-fiend AI using the six vanilla teleport points.
  - The unresisted dispel is removed.
  - Cornugons get SCS's cornugon AI.
- **Options** (combinable; each branches on whether 6510 is installed):
  - **A. Court of Belhifet:** at 75/50/25% HP he hops to the teleport point farthest from the party's melee, and golems and cornugons switch to defending him. Mirrors CH-2 and EX-2.
  - **B. Capped infernal reinforcements:** at thresholds the portal flares again and lets out one or two more cornugons, capped by difficulty and respecting the HoW rule. Mirrors CH-3 and DF-1. Added fiends need their own "die with the boss" script.
  - **C. Smart vanilla Belhifet (only without 6510):** intent-driven spell picks, cooldowns and reactive hops instead of the random post-hop spell. Mirrors SG-2, FF-1, DI-2/3, TG-4.
  - **D. Minion roles and Poquelin's opener:** golems press casters and time their clouds; cornugons peel attackers off Belhifet; Poquelin gets a short prebuff. Mirrors TG-2 and PB-1. Christopher decided on 2026-10-25 to drop the unresisted dispel.

**HoW: Icasaracht (AR9604)** [V]
- **Stats:** 400 HP, AC -6, cold 100, physical 50; min-HP ring.
- **Dragon phase:** cold breath and wing buffet each on random 12/18 s timers, otherwise melee. The Royal High Priestess heals Icasaracht, the gem, the king and the guards below 75%.
- **Gem phase:** at 1 HP the soul gem appears (MAGE 16, 150 HP). It casts from randomly swapped spell scripts and summons sahuagin or umber hulks when it sees no enemy.
- **End:** the gem's death completes HoW and exports the party.
- **SCS 6540/6040 today** [S]: full dragon and gem AI; caster AI for the priestesses.
- **Options:**
  - **H-A:** court roles (king and guards protect the priestess and hunt casters).
  - **H-B:** gem summons paced by gem HP and difficulty, with a cap.
  - **H-C:** dragon timing, only without 6540.

**TotL: the Luremaster (AR9711)** [V]
- **Stats:** BARD 20, 420 HP, AC -6, cold 100, physical 25; broad status immunities; min-HP ring; 30-odd memorized spells.
- **Scripts:**
  - DULURWRP: Dimension Door circuit over 8 fixed points.
  - DULURMST: 95 blocks with protection checks and unlimited Breach and Ruby Ray.
  - DULURTLK: dialogue and hostility; death at HP < 10.
- **On hostility:** spectral copies of each TotL hero the party killed, plus TRSPH1-3, then up to six one-at-a-time replacements. Guard groups and rakshasas also appear.
- **SCS today** [S]: only DULURWRP is removed, so with SCS he no longer hops.
- **Options:**
  - **L-A:** hero waves keyed to his HP and difficulty.
  - **L-B:** a reactive Dimension Door when surrounded, filling the gap SCS leaves.
  - **L-C:** guards defend him and rakshasas prebuff.

## A4. License
`doc\readme-stratagems.html` lines 1988-1995 [S]: copyright 2006-present David Wallace. The author asks you to "give me full credit when you borrow or re-use my code in your own mod". He also asks that SCS, or modified versions of it, not be mirrored elsewhere (paraphrase).
- There is no separate license file.
- **What this means for us:** the public mod reuses patterns only, writes its own BAF, ships no SCS files, `.ssl` code or `dw#` resources, and credits DavidW for the ideas. This fits "bundle the recipe, not the mods" and ADR 0005.

## A5. Top uncertainties
1. Whether vanilla EEBELWA ever gets to run while EEBELHI1's melee block keeps firing [L].
2. The Lizard King idling under SCS [L].
3. How the RAVER and MARILITH models look in IWD:EE [L].
4. Whether each spectral-hero death increments AR9711_HERO [L].
5. Two DLSGEM spawns in AR9604 [L].
6. Heart of Fury interaction: there is no script trigger for it [V].

The full list is in B6.

# PART B: Report detail

## B1. Strongest candidates
Script slots are the ARE actor slots [V]. A free override slot runs first [I], so an added layer can fire at chosen moments and otherwise fall through to SCS's or the vanilla AI.

**L7 Poquelin, Ascension Cave**
- **Vanilla:** IMP 15, 250 HP, AC -8, MR 50, fire 100. He memorizes Fire Storm, Symbol of Pain, Creeping Doom and Heavenly Inferno x5 each.
- **Script:** LDPOQUE1 in the general slot. On detection he closes the stair door, Dimension Doors to [817.551] and opens with a forced Heavenly Inferno.
- **Escape:** below 50% HP he leaves through LDPOQUEB or LDPOQUEC.
- **Area script:** two bronze sentries on hostility, then five INI waves 5 s apart (boneguards, earth elementals, minotaurs, myconids, salamanders; 5 each).
- **SCS today:** none on Poquelin.
- **Sketch** (override slot free):
  - target-aware spell picks;
  - hops when engaged;
  - waves paced by his HP and difficulty;
  - sentries guard him;
  - both escape blocks kept intact.
- **Risks:** the POQUELIN_HOSTILE global drives the waves. M.

**L6 The Idol**
- **Vanilla:** LDIDOL is its only script. Whenever it sees an enemy it force-casts at the nearest one, every ~3 s, with no checks: Finger of Death 10%, Power Word Kill 10%, Sol's Searing Orb 20%, Flesh to Stone 20%, Flame Strike 20%.
- **Plot hook:** its death opens the statue door (AR8012STATUE).
- **SCS today:** none.
- **Sketch:** target-state-aware picks, cooldowns, undead answering the Idol. Chris decides whether the result is fairer or harder.
- **Effort:** M.

**T-3 Lord Maluradek**
- **Vanilla (KELRDMAL):** while the area flags TELEPORT_1-3 are unset, he teleports the nearest PC away to one of three rooms. Otherwise he alternates melee with one random forced spell per round: Symbol of Hopelessness, Dispel Magic, PW Stun, PW Blind, Symbol of Pain, PW Kill, Animate Dead, Horror.
- **SCS today:** none on him or the plates.
- **Sketch** (override slot free): HP-aware power words, cooldowns, animated plates hunting the banished PC, a self-teleport when he's surrounded.
- **Effort:** M.

**W2 Joril**
- **Vanilla:** WTGIANT1 in the default slot; the other slots are free. Most of the court is neutral until dialogue; several actors are difficulty-gated.
- **SCS today:** WTGIANT1 replaced by SCS generic AI; Kontik gets caster AI.
- **Sketch:**
  - bodyguards screen Joril;
  - Kontik buffs the court (first check what SCS's Kontik already does);
  - Black Ice Knights hunt casters;
  - one capped reinforcement call, using the native EFDEL scripts for difficulty.
- **Risks:** actor count.
- **Effort:** L.

**H4 Mebdinga**
- **Vanilla (BAMEMBA):** a forced Mournful Wail or Undying Lament every 5-7 s. After two of those, a 50% Death Knell (not on Easiest). She runs from attackers.
- **SCS today:** none.
- **Sketch:** the class slot is the only free one. Time wails to clumped targets; virgins converge on her attacker.
- **Effort:** S.

**D5 Yxunomei**
- **Vanilla:** the girl form (D5YXUNG) force-casts SPIN182 on sight and becomes TANARRI: 91 HP, AC -4, 5 APR, MR 70, Animate Dead x2, Cloudkill.
- **SCS today:** 6510 rebuilds D5YXUN; priests and mages get caster AI.
- **Sketch:** priests buff and heal her, champions bodyguard, archers focus casters.
- **Effort:** M.

**L5 Ilmadia**
- **Vanilla:** LDILMAD (dialogue, cutscene, item use); drow priest-mages run LDDROWCM.
- **SCS today:** Ilmadia and the giants generic; the drow get the cleric/mage pass only with both 6030 and 6040.
- **Sketch:** priest-mages prebuff the giants, the giants hold a line, Ilmadia directs focus fire.
- **Effort:** M.

**F2 Cryshal-Tirith**
- **Vanilla:** the area script removes Pomab's images one by one as the fake Pomab is hit. The sentries (EEPOMSEN) play dead below 10 HP for 20 s, then rise; they die with the real Pomab.
- **SCS today:** none.
- **Sketch:** sentries body-block for Pomab; the images kite.
- **Effort:** S.

**K3 Kuldahar raid**
- **Vanilla:** spawns once SPRITE_IS_DEADYXUNOMEI is set: 7 Avengers, 3 Marauders, 8 allied defenders, plus a music change.
- **SCS today:** none (SCS has an unused combat guard for their script KUATKORO).
- **Sketch:** raid leaders and pack logic.
- **Effort:** S.

## B2. Final-boss detail

### B2.1 Base finale sequence [V]
1. **Cornugon check on entry:** below Hard, with HoW unfinished and the portal closed, the area script destroys both cornugons. Their own EFDELDEF repeats the check once.
2. **EEPEQTLK:** Poquelin opens dialogue DBELHIFE.
3. **EEPOQCT1:** the portal opens at [603.414] (INNATE_POQUELINS_PORTAL_OPENING). EECRNENT walks the cornugons to [783.503] and [529.555], then dialogue resumes.
4. **Final branch:** Enemy() plus EEPOQCNG: END_PORTAL 10, END_PROGRESS 5, an unresisted Dispel Magic on the nearest PC, SPIN183, golems hostile. Everard (1000 HP) is removed.
5. **End trigger:** HPLT("EEBELHI1", 10) sets SPRITE_IS_DEADEEBELHI1, then EEBELDTH (death animation SPIN171). EEBELKIL kills CORNUGON_1/2 and IRON_GOLEM_1/2 by name, then plays TOWER and CREDITS. Achievement flags BD_ACH_044/004/003/002 are set at the same threshold.
6. **EEBELHI1 (general slot):** on start, Aura of Fear plus a forced Symbol of Pain. Then AttackOneRound on whoever he last saw; walks to his attacker; one random forced spell after each hop (Fireball, Hold Person, Improved Invisibility, Fire Storm, Sol's Searing Orb, Shades, Summon Shadow, Produce Fire); walks to Player1 when idle and no PC is in sight.
7. **EEBELWA (default slot):** a 9 s timer and five reachable points. [917.1098] only appears in unreachable blocks.

**Slot plan:**
- Belhifet comes from BELHIF.CRE via opcode 151, so ARE slots don't apply. His CRE override, class and race slots are empty, and so is default once SCS strips EEBELWA.
- Golems: ARE override, class and specifics are free.
- Cornugons: override holds EFDELDEF; class and race are free.

**Constraints:**
- Keep EEPEQTLK, EEPOQCT1, EEPOQCNG, EEBELDTH and EEBELKIL intact.
- Never block the HP < 10 end.
- Test with and without 6510, with and without HoW finished, on Normal and Insane.

### B2.2 HoW detail [V/S]
- **Court scripts:**
  - The priestess (ILSAHHP1) opens with a forced Dispel Magic. After being attacked she casts Unholy Blight, Rigid Thinking and Hold Person. In the gem phase she stands at [1010.806] facing the gem.
  - The king and guards run plain EFATKBTH/EFATKMEL. Override slots are free on the whole court.
- **Gem:**
  - ILGEMW holds it idle until GEM_ACTIVE.
  - ILGEMS randomly swaps its race slot among seven spell scripts.
  - ILGEMU summons on a 15 s timer: two points above Normal, one otherwise.
- **SCS today:**
  - 6540 rebuilds ILDRAGON from SCS's own dragon script (`dragon\dragon.tpa:281-321`), keeping the GEM_ACTIVE/ILDRGDIE chain.
  - It strips ILGEMN and ILGEMU and compiles ILGEMS (summons remain, on timers).
  - It removes the breath's cold-resistance gate.

### B2.3 TotL detail [V/S]
- **Dialogue:** DLURE has peaceful outcomes (quest states 4/5) or the fight (Master_Quest 6 plus Enemy()).
- **Reinforcements:** TRSPH1-9 carry AR9711_HERO as their death variable (CRE 0x280). The area script spawns TRSPH4-9 one at a time while AR9711_HERO > 0, capped by #spectral_reinforcements.
- **Guards:** spectral guard groups 1-7 (six each) spawn on MAP_GROUP_HOSTILE1-7. Group 8 is two rakshasas and ten invisible stalkers.
- **Cleanup:** the spectral heroes fade out when the Luremaster dies (KELURDED, DULPOOF).
- **SCS today:** strips DULURWRP (`mage\mage_iwd.tpa:35`). It gives spectral Criek, Adran and TRSPH3/5/8 caster AI, and sets EFFOLPTY as general script on three of them (`priest\priest_iwd.tpa:111-113`).

## B3. SCS on IWD:EE

**Installable components** [S]: 100, 1600, 1610, 2000, 2010, 2020, 2030, 2040, 2060, 2070, 2080, 2500, 2510, 2520, 3510, 3540/3541, 4030, 4115, 4140, 4250, 5000, 5900, 6020, 6030, 6040, 6100, 6510, 6540, 6550, 6700.
- 2050 is not installable on IWD:EE.
- There are no 8000+ tactical components for IWD.

**How the coverage works:**
- **5900** initializes every CRE.
- **6020** sorts vanilla generic scripts. "Pure" ones are replaced wholesale; EFATKMEL, the vanilla "attack nearest seen" script [V], is one of them. "Impure" ones (128 scripts) get SCS blocks above and below and are overwritten with the result.
- **6030/6040** rebuild the spellbook and script for listed casters. The cleric/mage pass (Presio, rakshasas, Ilmadia's drow) runs only when both are installed.
- **6510/6540/6550/6700** are creature macros and boss scripts.

**EFFOLPTY** [V/S]:
- Vanilla walks idle creatures that see no PC to Player1.
- 6040 installs its own version (Detect instead of See).
- 6510 adds a teleport-to-Player1 for demonic creatures only (`fiend\fiend.tpa:948`).
- None of this is AI coverage.

**Defects that touch our fights:**
- **Lizard King combat guard looks inverted.** Code confirmed [S]; whether he really idles needs a lab check [L].
- **CAJSHA1M, SDELFPS1 and sdelfch2:** CAJSHA1M's top is never applied, the SDELFPS1 list entry matches nothing, and sdelfch2 is assigned the missing SDELFCS1. Game data [V]: SDELFCS1 and SDELFPS1 are missing; SHELFCS1, SHELFPS1 and CAJSHA1M exist.
- **RCOPPER** is reordered to D4MAGE, which isn't on the mage list; D4MAGE.BCS exists [V].
- **Wisdom bonus slots never granted** (a `spell_leve` typo) [S].
- **groups.ini lines skipped** because they lack "=" [S].
- **AR4005 YAUN* group names:** not a defect. AR4005.INI really uses YAUN1-6 [V].
- **Wholesale overwrites:** EEBELHI1, D5YXUN, CORNUGON, ILDRAGON, ILGEMS, KEBEHLD1, DUBEHLD1, EFFOLPTY, GNTEAM*, lich.cre and others.
- **Model swaps** for Belhifet and Yxunomei.

**Decision (Christopher, 2026-10-25):** these defects are fixed in our own mod after SCS, as our own patches: no SCS files or modified copies, credit to DavidW. No upstream report.

## B4. Design constraints
1. **Integration with SCS:**
   - Install after SCS and detect components with MOD_IS_INSTALLED.
   - Never edit SCS-compiled scripts in place; add layers in free slots or area scripts.
   - Write our own BAF.
2. **Difficulty:**
   - Reuse the EFDEL* scripts for added actors and DifficultyGT/LT (DIFFLEV 1-5 [V]) for waves.
   - Honor SPRITE_IS_DEADGEM.
   - Heart of Fury has no trigger [V], so use percentage thresholds. Exception: the game's own absolute end triggers (Belhifet and the Luremaster at HP < 10, Icasaracht at HP 1).
3. **Plot:** keep death variables, min-HP items, cutscenes and threshold triggers: LDPOQUE1's 50% escape, EEBELDTH, ILDRGDIE, DULURTLK.
4. **Fairness and stream:**
   - No unresisted ReallyForceSpell on PCs.
   - Telegraph phases with existing visuals.
   - Fallback and error notices go to the log only, never in-game (Christopher's rule, confirmed 2026-10-25).
5. **Spell Revisions:** use spell IDS names and test with and without SR.
6. **Matching:** match by resref and death variable, never by actor index or count; SCS's actor-order-dependent rows show the risk.
7. **Low levels:** leave the early areas alone or gate them by difficulty.

## B5. Encounter inventory by act (condensed)
Full stats are in the generated inventory and `fights_detail.md`: class/level, HP, AC, resistances, immunities, spells, items, script slots, spawn source and SCS mapping for every fight group.
- **Easthaven:** E1 AR1201 orc cave (orc shamans with 6040, ogres).
- **Kuldahar Pass:** K1 AR2000 goblin war band; K2 AR2001/2002 goblin cave set piece (2001ATTA) and beetle caves (no SCS).
- **Kuldahar:** K3 raid; cave bears.
- **Vale of Shadows:**
  - V1 AR3000: yeti chieftain and 31 shadows.
  - V2: Lysan.
  - V3 AR3301: Therik and the undead.
  - V4-V6 Kresselack's Tomb: Mytos, Myrkul's Sending, imbued wights, skeletal mages, spectral guards and knights, mummies.
- **Temple of the Forgotten God:** T1, 23 verbeegs, 10 acolytes, Talonite priest.
- **Dragon's Eye:**
  - D1 Lizard King.
  - D2 AR4002: 33 trolls, 4 Talonite priests, Mother Egenia (no SCS), 25 bombardier beetles.
  - D3 Presio, D4 sanctuary, D5 Yxunomei.
- **Severed Hand:**
  - S1 AR5001-5003 garrison: about 200 shadowed orcs, goblins, ogres, worgs, skeletons and souls; 5 orc shamans; burning skeletons (MAGE 5, Fireball x3, no SCS).
  - S2 elf towers.
- **Dorn's Deep:** U1 (63 blue myconids with no SCS, 16 ettins, orogs), U2 barracks, U3 lich, U4 Bandoth (optional, 6030).
- **Wyrm's Tooth:** W1 museum, W2 Joril, W3 glacier packs.
- **Lower Dorn's Deep:**
  - L1: tarnished sentries, Seth (no SCS), thieves, Kelly and archers, svirfneblin allies.
  - L2 Malavon, L3 Marketh.
  - L4: Shikata and 12 salamanders.
  - L5 Ilmadia, L6 Idol, L7 Poquelin.
- **Return to Easthaven:** F1 seven cyclopes (+2), F2 Cryshal-Tirith, F3 finale.
- **HoW:**
  - H1 Lonelywood.
  - H2 barbarian camp: Angaar and Beornen (6020); Jorn (CLR 12, no SCS); Wylfdene (dialogue only).
  - H3 Burial Isle, H4 barrows, H5 Edion.
  - H6-H7: Gloomfrost packs, 26 remorhaz, 29 ice golems.
  - H8 Sea of Moving Ice: Bergclaw (no SCS), bones, Xactile (6040).
  - H9 sahuagin temple.
  - H10 finale.
- **TotL:**
  - T-1 harpies, wyverns, Wyvern Matron.
  - T-2 tower trials, T-3 Maluradek.
  - T-4 Temple of Helm crypt: 178 actors.
  - T-5 dungeons: 4 beholders with 6550; 12 minotaurs with no SCS.
  - T-6/7 jackal caverns.
  - T-8 finale.

## B6. Verification and open points
**Checked by hand this pass:**
- SCS: `caster_shared.tph` 241-276 and 354-365; `fiend\fiend.tpa` 851-948; `fiend\fiend_shared.tph` 345-363; `dragon\dragon.tpa` 279-322 plus `ildragon.ssl` and `ilgems.ssl`; the EFFOLPTY handling in `priest_iwd.tpa`; readme 1988-1995.
- Game data: every missing/present resource named above; ANIMATE.IDS; AR4005.INI; all finale scripts; the EFDEL* scripts; the AR2100 raid block; AR8012/8013; KELRDMAL; BAMEMBA; Belhifet's items.
- Other SCS claims come from the helper agents' static reads.

**Open:**
1. Act labels are by place; some area sub-names are inferred from map notes.
2. EEBELWA reachability in play, and the "first true block wins" reading [I/L].
3. AR9711_HERO increments, and the triggers for guard groups 1-7 (not traced).
4. Two DLSGEM spawns.
5. SCS behaviours never observed: Lizard King idle, potions on per-actor caster clones, the models, the AR9717 beholder text swap.
6. SPIN182's opcodes not listed.
7. Heart of Fury effects.
8. ~~Op120 thresholds on Belhifet's amulet (which enchantment hits him).~~ Resolved on 2026-10-25: +3 or better (see A3).
9. Dialogue texts were read only for their actions.
10. Boss flags are heuristic; allies such as Arundel and Everard appear as rows.
11. The engine's script-slot order used in the slot plans [I].

## B7. Files
The generated data behind this report (per-act fight inventory, creature data, decompiled finale scripts, SCS coverage notes and pattern catalog) is not committed: it is derived from game and SCS files.

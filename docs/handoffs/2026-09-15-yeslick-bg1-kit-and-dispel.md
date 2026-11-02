# Yeslick: BG1 Alaghor coverage and existing-install Dispel

Status: **confirmed read-only diagnosis; next-patch fix requested**, 2026-09-15.
No game/save changes, source-pin changes or publication in this pass.

## Expected versus observed

The curated Yeslick choice is **Fighter/Alaghor of Clangeddin**, not Stormlord.
The intended innate is hostile-only Dispel with explicit `floor(1.5 * level)`
scaling, aligned with selected SCS 3540.

| Target | Class/kit | Dispel evidence |
|---|---|---|
| Stream installation, September 3 RC | YeslickNPC v5.0 component 1 is installed. BG1 YESLIC/YESLIC5 remain unkitted Fighter/Cleric; BG2 LK#YESL has LK_ALAGHOR. | No modpack 410 or historical YESLICK_KELDORN_DISPEL_FIX row. Effective SPIN112 is 730 bytes, one header, projectile 157, opcode 58 parameters 0/0. This is party-affecting and bypasses the dispel-level contest, not the intended fix. |
| Latest stream save inspected, September 15 05:05 local | Party Yeslick, DV `yeslick`, Fighter/Cleric 5/5, raw kit 0x40000000 (unkitted). Still knows SPIN112. | Reads the stream's uncorrected spell resource. No native casting test was performed. |
| Retained Combined installation | Continuity 199 is installed, but does not supply the missing BG1 kit conversion. | 410 is recorded. SPIN112 has 40 headers; every level 1-40 has projectile 177 and opcode 58 parameters `floor(1.5 * level)`/2. All 40 verified by read-only binary inspection. |
| Current public-modpack source / next collection draft | No BG1 Alaghor conversion found. | Semantic 410 is already present in modpack v0.2.0-alpha.5 (`d4b1e687`), and remains mandatory/ready in the default draft. This is not new implementation to duplicate. |

Yeslick v5.0 `yeslicknpc/lib/main_component.tpa:142-154` assigns the selected
kit only to `lk#yesl.cre` and `lk#yes25.cre`. Selection was successful; its BG2
scope was incorrectly assumed to cover BG1. There is no evidence here that a
later mod stripped the kit, or that TLK corruption caused it.

Modpack 199 intentionally preserves returning BG1 actors' class/kit rather than
overwriting them with BG2 template settings. Without an earlier conversion it
would preserve this omission. Do not "fix" continuity by resetting players'
customized builds during transitions.

## Owning-mod work: chriz-bg-modpack

Implement the narrow missing BG1/SoD Alaghor preset in the owning mod, then intake
its released artifact into CEBG. Do not put gameplay implementation in this repo.

- Start from the integrated continuity source `85cbc42551ca256b167cfdb1ffc4bd3c22df85e4`
  or its verified successor, preserving all later owner work. The retained source
  worktree is `modpack-combined-playtest`; an old task checkout may no longer exist.
  Use a separate worktree for implementation. No live-game/save writes.
- Audit actual BG1 recruitment variants and EET aliases; exclude quest/dream or
  other non-companion creatures rather than patching every Yeslick-like filename.
- Resolve LK_ALAGHOR and its CLAB from the target installation; no hardcoded kit
  number or foreign string references. Apply kit grants/passives appropriate to
  priest level without resetting XP, class levels, proficiencies or dialogue.
  Source LK#YK starts its kit innates at priest levels 8/14/16; do not grant them
  early to the current 5/5 character.
- Place the conversion after its kit provider and **before 199/EET_end**. Respect
  Yeslick component 1 versus vanilla component 0 and CEBG's vanilla-companion
  opt-out. Owner assigns a non-conflicting component number; none reserved here.
- TDD: cover audited recruitment variants, no-kit/vanilla opt-out, dynamic kit ID,
  correct level-dependent grants, and preservation through continuity. A brief
  fresh recruit/level-up/save-reload check can accompany the planned companion test.
- Keep existing 410 and its hostile-only 1.5x behavior. Its source preserves
  non-dispel/casting effects and Keldorn's installed SCS scaling. Do not invent a
  second spell implementation or blindly reinstall the whole modpack at the tail.

Collection integration must validate the new component's selection and order.
The existing vanilla Yeslick route suppresses combined 410 (including Keldorn's
part); this is a known customization limitation, not permission to change that
separate policy silently in this fix.

## Existing stream / public games

Separate two operations:

1. **Spell resources:** 410 is a targeted patch candidate where it is absent and
   effective SPIN112/SPCL231 pass its supported-shape and SCS checks. Test on an
   isolated copy, retain unrelated effects, close/restart the game and record the
   patch. Already-correct resources need no rewrite. This pass did not apply it.
2. **Saved Yeslick:** changing override templates cannot repair an already-saved
   actor. A guarded migration must reconcile kit grants/passives at the actual
   priest level, preserve progression and respect intentional player kit changes.
   Do not claim that writing only the kit field fixes existing characters.

An app update alone does neither. Generic automatic saved-character migration is
not a prerequisite for the fresh-install release. TLK handling remains a high-risk,
installation-specific operation; no copying another installation's TLK/resources
with foreign string numbers, and no general hotpatch safety claim.

## Evidence pointers

- [Yeslick curation](../curation/components/YESLICKNPC.md).
- `manifest/collection.toml`: `feature:chriz-bg-modpack:component-410` and its
  Yeslick 1 / SCS 3540 prerequisites; generated recipe agrees.
- Owning source: `chriz-bg-modpack/lib/cbm_yeslick_keldorn_dispel_fix.tpa`,
  `tests/test_spell_tail.py`, `docs/companion-continuity.md`.
- [IESDP opcode 58](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op58):
  mode 0 bypasses the level contest; mode 2 uses the supplied dispel level.
- Saved data and installed resources were read only. Source tests were inspected,
  not rerun; no claim of live-engine acceptance.

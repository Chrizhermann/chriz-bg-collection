# Collection tail fixes — provenance and migration

The reference stack contains 22 single-component local installers classified as
`local-fix` in `manifest/mod-sources.tsv`. All 22 were installed, but they are historical
snapshots rather than supported fresh-stack packages. This file closes their catalog gap
without pretending that the old installers are safe choices in the new UI.

Most are being consolidated into `chriz-bg-modpack`; a few belong in a mod-specific fork
or balance layer. A `mandatory` plan below means **automatic only when its activation
conditions are true**, with no standalone user toggle. `default` remains an optional
preference enabled by default.

## Current migration inventory

| Legacy installer | Intended home | Fresh-stack status | Planned behavior |
|---|---|---|---|
| `NPC_KIT_CHANGES` | Retire monolithic `chriz-bg-modpack` `500` | Blocked: the current component is a FAIL stub; the legacy installer hardcodes kit numbers, bundles ten unrelated NPC changes, duplicates selected native components, and touches special/nonjoinable CREs. | Replace it with semantic per-NPC choices only where no maintained native component exists. Never install `500`. |
| `BARD_SPELL_FIX` | `chriz-bg-modpack` `100` | Blocked: current component is a FAIL stub; the legacy installer couples Aura and Safana and hardcodes physical `SPWI` slots. | Aura should use its native Bard option. Rebuild the Safana half semantically, then make it mandatory under the Safana-Bard preset. |
| `EVANDRA_SORCERER` | Unassigned | Unmigrated. Its effective spell choices match the selected SR reference, but the patch still needs a maintained home and fixture coverage. | `default` when Evandra is selected. |
| `SKIE_SKILL_FIX` | `chriz-bg-modpack` `160` | FAIL stub. | `mandatory` only when Skie is converted to Swashbuckler. |
| `SAFANA_SNARE_FIX` | `chriz-bg-modpack` `150` | FAIL stub. | `mandatory` only under the Safana-Bard/Abettor preset. |
| `MAZZY_PROF_FIX` | `chriz-bg-modpack` `140` | FAIL stub. Current curation now makes Mazzy a Divine Champion through Artisan NPC `3102`, so the earlier no-Mazzy policy is obsolete. | Reimplement semantically and apply automatically with Artisan NPC `3102`. |
| `VICONIA_MULTICLASS` | New Viconia-only `chriz-bg-modpack` component | Explicitly curated as a default, but the maintained modpack has no implementation and its Phase-0 snapshot omitted this installer source. | Reconstruct and fixture-test the class/level/skill conversion semantically; expose it as the default Viconia class choice and do not reuse `500`. |
| `AK_MULTICLASS_PROFS_FIX` | `chriz-bg-modpack` `200` | Likely obsolete: the target Artisan source now enumerates the six formerly missing multiclasses dynamically. Current component is a FAIL stub. | Retire after a fresh-install verification; do not duplicate the upstream fix. |
| `XAN_EK_FIX` | Artisan NPC `20002`; legacy destination `chriz-bg-modpack` `170` | Partly superseded: target Artisan code now covers `XAN_` but still misses `XAN4`, `XAN6`, and `TTXAN`. | Absorb the remaining EET variants into Artisan `20002`; then make the repair mandatory when Xan Fighter/Mage + Eldritch Knight are selected. |
| `EDWIN_AMULET_FIX` | `chriz-bg-modpack` `310` | FAIL stub; a personal balance choice tied to Red Wizard Edwin, which is itself unavailable under the current SR policy. | Exclude for now. |
| `KIVAN_QUEST_FIX` | `chriz-bg-modpack` `130` | FAIL stub. | `mandatory` when the BG1NPC Kivan sea-elf quest and SCS smarter general AI `6000` are both present. |
| `SR_SUBSPELL_FIX` | Spell Revisions `v4.21-chriz.3` component `60`; legacy modpack `180` retired | Migrated into the maintained successor release. | Installed automatically with the default late NPC-spellbook pass; no standalone tail component. |
| `AURA_BALANCE_PATCH` | `Aura_BG1_BG2_EET-Chriz-Balance-Patch` | Its crossbow rebalance is integrated in the maintained Aura balance patch; complete legacy parity still needs verification. | Retire after verifying all six legacy item changes on a fresh target build. |
| `BEARSKIN_MAIL_FIX` | `chriz-bg-modpack` `300` | Likely obsolete: target Artisan source no longer carries the bad usability byte. Current component is a FAIL stub. | Retire after fresh-install verification. |
| `BRANWEN_HAMMER_FIX` | `chriz-bg-modpack` `400` | Still required with current SR, but the destination is a FAIL stub. | Reimplement with a semantic SR spell guard; then `mandatory` when Branwen and SR are selected. |
| `ELEM_PRINCE_CLAB_FIX` | `chriz-bg-modpack` `210` | Obsolete: target Artisan code already excludes SR-hidden `SPPR724`/`SPPR729` from class delivery. Current component is a FAIL stub. | Retire; do not duplicate the upstream fix. |
| `YESLICK_KELDORN_DISPEL_FIX` | `chriz-bg-modpack` `410` | Still required, but the destination is a FAIL stub and the legacy prebuilt spell snapshot is unsafe to reuse. | Rewrite semantically; then `mandatory` with Yeslick + SCS `3540`, and unavailable with `3541` or another Dispel scaling choice. |
| `FADE_FT_FIX` | `chriz-bg-modpack` `110` | Destination is a FAIL stub. | `default` when Fade is selected; mutually exclusive with Fade component `2` (Shadowdancer). |
| `FADE_FT_PATCH` | `chriz-bg-modpack` `120` | Redundant: `FADE_FT_FIX` v2 already contains the same proficiency and amulet changes. Destination is a FAIL stub. | Retire component `120`; never expose or install it separately. |
| `CBM_UAI_SCROLL` | `chriz-bg-modpack` `430` | Superseded by the implemented maintained component. | Use only `430`, automatically when Tweaks Anthology `2170` is selected; never install both versions. |
| `SAFANA_LATE_SPELLS` | Unassigned | Blocked: hardcoded `SPWI` slots and a legacy never-uninstall assumption need a semantic rewrite. | After rewrite, `mandatory` under the Safana-Bard preset. |
| `PRIEST_DELIVERY_FIX` | Artisan/SR compatibility layer | Blocked: the snapshot hardcodes one installed CLAB/HIDESPL state, lacks guards, and the semantic delivery repair remains open. | Exclude until rewritten and fixture-tested; then make it conditional and automatic for the affected Artisan + SR path. |

## Maintained modpack status

The current `chriz-bg-modpack` source implements components `430`, `440`, `450`, and
`600`. Components `100`–`420`, `500`, and `510`–`514` are explicit FAIL stubs and must
remain unavailable even where this inventory records a future decision. Component `500`
must be retired rather than implemented with its current monolithic meaning. The catalog in
`CHRIZ-BG-MODPACK.md` is the actual component selection surface; this file records how
the historical tail installers map into it.

## NPC assignment reconciliation

The 2026-09-01 joint audit separates the old monolith from the requested defaults. No
new component number is assigned until Chris reviews the split.

| NPC outcome | Status and no-duplicate home |
|---|---|
| Khalid → Vanguard | Use Artisan NPC `1101` only; it is already `default`. Do not reproduce it in the modpack. |
| Sirene → Martyr | Retire this legacy branch. Current Sirene curation defaults to the native True Paladin choice. |
| Shar-Teel → Wizard Slayer | Explicit new `default`; create a semantic Shar-Teel-only component for the joinable `SHARTE`, `SHARTE4`, and `SHARTE6` variants. Do not patch the special/nonjoinable `SHARTD`. |
| Kagain → Dwarven Defender | Explicit new `default`; create a semantic Kagain-only component after the selected Artisan Dwarven Defender definition. |
| Sarah → Archer | Historical reference choice still awaiting final confirmation; if retained, create a Sarah-only child of the revised-audio Sarah route. |
| Skie → Swashbuckler | Explicit new `default`; create a Skie-only assignment and make skill redistribution `160` automatic. Patch joinable `SKIE`, `SKIE6`, `BDSKIE`, and `BDSKIED`, but never fighter dream resource `BDSKIEDR`. |
| Imoen → Trickster | Use Artisan NPC `7102` only; it is already `default`. Duplicating it can duplicate an innate and permanent effect. |
| Mazzy → Divine Champion | Use Artisan NPC `3102` only; it is already `default`; add `140` automatically once implemented. |
| Safana → Abettor | Keep the existing composite Safana-Bard preset unavailable until its class, spells, snare, late-spell, and SoD-inventory pieces are all semantic and tested. |
| Aura → Bard Artificer | Retire the custom branch. Native Aura component `6` is the only future route, and current Aura curation does not select it. |
| Faldorn → Avenger | Explicit new `default`; create a semantic Faldorn-only assignment. Artisan source component `5200` is a separate unresolved overhaul choice, not the assignment itself. |
| Dynaheir → Haste in her level-3 spellbook | Explicit new `default`; add `WIZARD_HASTE` semantically after the final Spell Revisions spell mapping but before SR `60` updates joinable NPC spellbooks. Clarify known-only versus one memorized copy before implementation. |
| Garrick → Troubadour | Explicit new `default`. Use Artisan NPC `99001` exactly once and script Bardic Wonders `1008` to answer **No** to its Garrick prompt. |
| Kivan → Archer | Explicit new `default`; create a Kivan-only semantic assignment separate from quest repair `130`. Treat it as kit-only unless loadout/proficiency changes are explicitly curated. |
| Viconia → Cleric/Thief | Explicit new `default`; create a Viconia-only semantic class conversion with dynamic XP/level handling and fixture coverage. |

These NPC CRE changes normally affect only unjoined/uninstantiated versions. Without a
deliberate save migrator, an update containing them is `new_game_only` or
`before_npc_join`, not a repair for an already joined NPC.

The 2026-09-03 private RC has neither the pending maintained replacements nor their
unsafe legacy installers, so its Viconia variants remain single-class Clerics and its
Shar-Teel variants remain unkitted Fighters. This is a known recipe gap, not an accepted
default.

## Newly identified tail work

These requirements are not among the 22 historical local installers:

| Gap | Curated outcome |
|---|---|
| Safana retains SoD-only items in EET | Add a semantic cleanup that is `mandatory` when Safana is selected. |
| IWDification `120` misses Artisan Arcane Trickster | Add the level-7 Evasion grant to `C0ATR.2DA`; make it `mandatory` when both features are selected. |
| EET Tweaks `2042` needs different progression | Replace the upstream component with a maintained collection-tail implementation; expose that replacement as `default` once implemented. |

## Coverage limits still requiring audit

The 22 installer IDs above close the known `local-fix` catalog gap, but they do not prove
that every manually copied file has a maintained owner. The modpack snapshot audit had
WeiDU backup evidence for only 17 of 67 installed mods; attribution for silent writers was
therefore moderate rather than complete.

In particular, eight captured creature-animation files—`6203.ini`, `6213.ini`,
`6502.ini`, `6503.ini`, `6504.ini`, `6512.ini`, `6513.ini`, and `6514.ini`—still have
unknown CIMC/CDMC provenance and no maintained home. Identify their source and decide
whether they are required before declaring the manual-install-folder inventory complete.

Historical positions in `manifest/install-order.tsv` remain evidence of the reference
install only. Fresh ordering must be generated from each migrated component's real
dependencies rather than replaying the old tail sequence.

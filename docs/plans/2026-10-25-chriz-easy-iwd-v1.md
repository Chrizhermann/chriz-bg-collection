# Chriz Easy IWD v1 — plan

Status: active plan, started 2026-10-25. Supersedes the scheduling parts of
[the September direction](2026-09-08-iwdee-collection-direction.md); its recorded mod
decisions still stand unless noted below.

## Why now

Christopher streams an Icewind Dale: Enhanced Edition playthrough in November 2026. On
2026-10-07 he told his Discord there would be an installer so viewers can play along; on
2026-10-21 he hoped for a first version around 2026-10-25, otherwise the week after. On
2026-10-25 he called it urgent. The stream date is not fixed yet.

## Decisions

| Date | Decision | Source |
|---|---|---|
| 2026-09-08/10 | Name **Chriz Easy IWD**; same repository as CEBG; separate collection recipe; same curation procedure as BG. | Codex thread "Assess IWD:EE modlist effort" |
| 2026-09-08/10 | Mods: EE Fixpack, SCS (IWD components), Tweaks Anthology (re-curate components), BuffBot; Artisan's Kitpack and Bardic Wonders if compatible; Spell Revisions omitted on purpose; Frostforged deferred (balance worry); tentative companions Minerva, Karihi, Ina, Cassia, Oak-Maw plus the IWD Crossmod Banter Pack. | same |
| 2026-10-21 | Discord plan: no-reload, EEex, Bubb's Spell Menu, BuffBot, companion mods, SCS (maybe with extra tweaks), Tweaks Anthology, optional quest mods. All companions and quests may be offered in the installer; the stream uses no extra quests or only the best one. | Discord, quoted in the hand-off |
| 2026-10-25 | **Own app** "Chriz Easy IWD" built from this repository (own name, icon, identifier, registry and update feed), not a game choice inside CEBG. | Christopher, this session |
| 2026-10-25 | "No-reload" is a stream rule only; the installer needs nothing for it. | Christopher, this session |
| 2026-10-25 | The fight audit may show everything (names, places, final boss); Christopher designs the big strokes. NPC and quest summaries stay spoiler-free. | Christopher, this session |
| 2026-10-25 | Combat upgrades: plain WeiDU/BAF scripts in a small own mod, one component per fight or fight group, installed after SCS, each change described so it can later move to the script engine. Small creative calls are the agent's; shortlist and boss idea go to Christopher early. | Hand-off of 2026-10-25 |
| 2026-10-25 | Mod-level selection made; see the [worksheet](../curation/iwdee/WORKSHEET.md). | Christopher, selection page |
| 2026-10-25 | Phase 1 priority: the installer works, SCS works on IWD:EE, and some enemy mages and priests prebuff. Encounter upgrades come after that, designed together; the fight shortlist is open for discussion. | Christopher, this session |
| 2026-10-25 | Belhifet: reinforcements scale with difficulty; his blanket spell immunities should change (to discuss). Poquelin's unresisted Dispel Magic at the transformation goes. | Christopher, this session |
| 2026-10-25 | The SCS defects found on IWD:EE are fixed in our own mod after SCS, not reported upstream. | Christopher, this session |

Curation stays Christopher's: the agent delivers neutral inventories; he decides mods,
components and defaults.

## Verified facts (2026-10-25)

- **Game:** IWD:EE 2.7.3.0 in Steam and in a read-only local reference copy;
  identical `icewind.exe` (SHA-256 `3458B17B…B320F1`), Steam app id 321800, no WeiDU.log,
  empty override. `engine.lua` has `engine_mode = 2`. Data includes `PATCH24/26/27` BIFs.
- **EEex v1.3.1** (2026-10-07) supports it: the TP2 admits `iwdee`, picks the version by
  `data/PATCH27.BIF`, and ships `loader/v2.7.3.0/iwdee/` function and profiler databases;
  InfinityLoader's `ExeNames` includes `icewind.exe`. Live boot not yet tested.
- **Bubb's Spell Menu Extended v5.2** declares IWD:EE support (Sept research; recheck pin).
- **BuffBot v1.11.0** still gates both components to `GAME_IS ~bgee bg2ee eet~`. The
  September source review found no other IWD blocker (UI hooks and graphics exist in
  IWD:EE); enablement plus a lab test is needed in the BuffBot repository.
- **SCS v35.21** (2024-11-21) is still the newest release. On IWD:EE it offers: AI
  initialisation, IWD-only "Smarter general AI and better calls for help", Smarter Mages,
  Smarter Priests, Potions for NPCs, Improved fiends and celestials, Smarter dragons,
  Smarter beholders, IWD-only "Revised Icewind Dale Undead", IWD spell/scroll tweaks.
  Most BG tactical encounter packs are gated off for IWD.
- **Lab tooling** (`bg_lab_slots.py`, `lab_quiet_run.py`) only recognises `Baldur.exe`
  and `InfinityLoader.exe` as game processes; it must learn `icewind.exe` before the
  first IWD lab launch, with its tests.

## Architecture

From a read-only scan of the release line (`5106b53`): almost everything in CEBG is
game-independent (acquisition, cache, archive checks, resolve, WeiDU runner, resume,
`engine_name` save identity, InfinityLoader launch). The EET-specific part is one repeated
assumption: there are always two source games. Plan:

1. **Engine standalone mode** (`claude/iwdee-engine`): `Collection.game` (default EET,
   skipped when EET so BG digests stay identical), `GameRole::Iwdee`, Main-only
   validation for standalone collections, optional BG1 in preflight/stage/session/
   receipt/CLI (`serde(default, skip_serializing_if)`, no schema bumps), stage step
   `stage:game` for standalone only, launch via the profile executable when EEex is
   absent, optional collection id in the registry, injectable registry directory, IWD:EE
   2.7.3.0 Steam build profile. Byte-identical round-trip tests for old ledgers, receipts,
   registries and the real BG recipe.
2. **Branded app build** (after 1): a Tauri config overlay for "Chriz Easy IWD" (product
   name, identifier, icon, updater endpoint), build-time application-data/registry
   directory and identifier so the two apps never share a registry, single-source install
   screen, bundled `recipes/iwd-current/` (name to settle), package script parameters.
3. **IWD recipe** (after curation): artifacts and mod pins, runs, features, presets,
   release ledger with its own `recipe_id` (e.g. `chriz-easy-iwd`).
4. **Own combat mod** (new local repository, working name to settle with Christopher):
   one WeiDU component per chosen fight or fight group, after SCS, described for a later
   script-engine port. SCS code is reused only if its licence allows, with credit.
5. **BuffBot IWD:EE enablement** in the BuffBot repository, with an IWD lab test.

Shared-engine changes stay on IWD branches until alpha.22 has landed; then they are merged
against the new base. The CEBG coordinator session was told on 2026-10-25.

## v1 scope

In: the branded installer for IWD:EE 2.7.3.0 (Steam, en_US first, mirroring CEBG's store
support); Christopher's curated mod list from the neutral inventory; the own combat mod
with the shortlist he approves; BuffBot once enabled and lab-tested; a full lab install
plus boot and short play check.

Later: further companions or quests that need author consent; Frostforged; deeper
caster-readiness ports from BG Rebalance; script-engine migration; GOG/other builds that
we cannot fingerprint yet.

## Timeline (realistic)

- Days 1–3: engine standalone mode; inventory and curation with Christopher; fight audit,
  shortlist and boss idea; lab runner learns `icewind.exe`.
- Days 3–6: branded app build; IWD recipe; combat-mod components; BuffBot enablement.
- Days 6–8: full lab install, boot and play checks, fixes; release candidate for
  Christopher's go-ahead (signing, public release and website need his explicit OK).

A first community version is therefore realistic in early November, not on 2026-10-25.
If the app slips, Christopher's own stream copy can be installed earlier through the
engine CLI once step 1 lands.

## Risks and open points

- Rate limits on long agent runs; the engine change touches resume-critical contracts.
- Third-party licences: mods whose readme forbids compilations wait for author consent
  (Christopher contacts authors; nothing is sent without his OK).
- SCS on IWD:EE is described upstream as partial; our combat mod must not depend on
  unverified SCS behaviour.
- The stray `weidu.conf`/`steam_appid.txt` in the reference copy are not in a pristine
  Steam install; profiles must not treat either copy's quirks as universal.

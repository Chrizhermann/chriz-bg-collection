# Curated full recipe: source and changed-order evidence

Bounded reconciliation evidence captured 2026-09-05. The component catalogs and
`manifest/curation-map.toml` remain the selection authority; this note only freezes the
missing runnable sources and the changed or ambiguous order facts needed by the recipe.

## Runnable source contracts

| Artifact | Source contract | Archive contract |
|---|---|---|
| Bardic Wonders `v2.9c-balance.2` | `https://codeload.github.com/Chrizhermann/Bardic-Wonders-Chriz-Balance-Patch/zip/refs/tags/v2.9c-balance.2`; commit `db0cf81504fd3f84e4e74eb8ab30e65499135512`; 5,234,586 bytes; SHA-256 `bfa16cde633d9722ecc9ddb84693e0ff07dfb5fb2ebf1ec6a6b9d0e3a29cb4fb` | ZIP, single wrapper `Bardic-Wonders-Chriz-Balance-Patch-2.9c-balance.2`; publish `BardicWonders`; TP2 `BardicWonders/Setup-BardicWonders.tp2` |
| Branwen `v8` (`v8pre` package/menu name) | `https://github.com/Pocket-Plane-Group/Branwen_for_BGII/releases/download/v8/branwen-v8pre.zip`; 13,926,210 bytes; SHA-256 `f82c93e67f1910f0754364c17d40e7f4ee2c9609a39cd081efbfc69c9faaea17` | ZIP, direct root; publish `Branwen`; TP2 `Branwen/branwen.tp2` |
| Tweaks Anthology `v18` | `https://github.com/Gibberlings3/Tweaks-Anthology/releases/download/v18/the-tweaks-anthology-v18.iemod`; 21,817,521 bytes; SHA-256 `776212ec781ddf14071a9f74df075824fe8430a13302d2bea7d11fd91d179b17` | IEMOD, direct root; publish `cdtweaks`; TP2 `cdtweaks/setup-cdtweaks.tp2` |
| IWDification `v11` | `https://github.com/Gibberlings3/iwdification/releases/download/v11/iwdification-v11.iemod`; 44,166,170 bytes; SHA-256 `4af9d463d3b4afe1e296fe056b6c93e9b6b95e5bcb5541eca81dcbb8ba15db3f` | IEMOD, direct root; publish `iwdification`; TP2 `iwdification/setup-iwdification.tp2` |

All four use language `0`, the collection's WeiDU 249 artifact, `setup-name` invocation,
and no extra command-line arguments. The byte counts, hashes, roots, and TP2 paths above
were verified from fresh downloads. Prefer the plain IEMOD assets over the Windows
self-extractors for Tweaks Anthology and IWDification.

The recorded Bardic source is the maintained `v2.9c-balance.2` fork, not the old upstream
`cfcd1c4` snapshot. A later `v2.9c-balance.3` release exists, but promoting it is a separate
source/curation decision and is not implied here.

## Manual Evandra source

Evandra has no frozen official standalone download: the G3 `v2.2` file is page-gated.
The existing user-supplied `creator-full-private-extras-20260902.zip` is a feasible
acquisition source for this run: 1,263,540,769 bytes, SHA-256
`3b7dbc1994d57842bbddcb93726319fc5599bc5e84758c0cb33bd46ef9ed7c38`, direct root
`evandra`, TP2 `evandra/setup-evandra.tp2`. The bundled TP2 and the read-only archived
`v2.2` TP2 both hash to
`d35217e68ae3c91871af9b19fdb9dc6af64a652d0468bd938006e9d0f01a85d7`.

The archive supplies bytes only. Curation independently selects Evandra component `0`
as mandatory and component `1` (Crossmod Content) as default. Other roots in the private
archive do not authorize runs. The current extraction cache marker is bound to the old
49-root declaration; declaring only Evandra against the same digest requires a fresh cache
root, while immediate reuse requires the existing broad archive declaration.

## Required prompt and order changes

- Randomiser `1100` has a fresh-install compatibility `ACTION_READLN` in `lib/arrays.tpa`
  lines 22-30. `lists/mod_compat.2da` matches the curated Xan `0` and RR `12` components;
  the approved 2026-08-19 installer design already specifies `y` (leave required items in
  place). Recipe alpha.5 omitted the prompt, causing `End_of_file` and rollback of `1100`.
  Alpha.6 restores this output-gated answer only when either owning feature is effective.
  It also restores the catalog's explicit Randomiser-after-SCS placement, before EET_END.
  Fresh copies do not contain prior randomisation state, so the saved-state preservation
  prompt in `lib/random_seed.tpa` is not part of this new-install recipe.
- BG1 NPC v32 declares its Kivan choices `240`/`241` before portraits `160` and
  player-initiated dialogues `200` (`bg1npc.tp2` lines 590/598/633/644). WeiDU
  `--force-install-list` follows this source order, not the numeric order in the catalog.
  The first corrected full run installed all eight selected components and exited zero,
  but recipe alpha.4 expected `200, 240` and correctly failed exact-suffix verification.
  Recipe alpha.5 restores the already documented native order for every offered BG1 NPC
  choice. No selections or safety checks are removed; that failed copy cannot be resumed.
- A follow-up audit used WeiDU 249 `--list-components-json` on all 36 exact staged TP2s
  covering 43 selected runs. There were no missing selected IDs and only two additional
  order mismatches: Artisan main must place `10001` before `10002`, and `1100` before
  `1003` (TP2 lines 974/978/994/998); Randomiser must place `1100` before `9000`,
  `10200`, and `10210` (TP2 lines 124/425/446/455). Alpha.5 corrects these too, including
  optional choices, without changing the set of selected components. The other 40 runs
  already matched their exact native component order.
- Bardic Wonders `1008` prints `[1] Yes` and `[2] No` for its Garrick assignment. The
  curated single-assignment route must answer literal integer **`2`**, not the historical
  replay's `1`. Then install Artisan NPC `99001`: the exact pinned `chriz-v1.3.1`
  source has a hard `MOD_IS_INSTALLED Setup-BardicWonders.tp2 1008` predicate, reads the
  installed `C0TRB` symbol from `KIT.IDS`, and immediately patches every Garrick CRE's kit
  field. The required order is therefore `Bardic 1008 (answer 2/No) -> Artisan NPC 99001`.
- Bardic `3001` is a late dialogue-class-check patch and its source requires `1012` or
  `2009`. The curated route has `1012`; `2009` stays excluded. Bardic `1012` retains its
  EEex prerequisite. Bardic `1006` remains conditionally inactive with Spell Revisions.
- Evandra `1` detects other NPC dialogue resources at install time. In particular it only
  compiles the approved Xan crossmod when `o#xan25.dlg` already exists. Split the run as
  `Evandra 0 -> Xan -> Evandra 1 -> Crossmod Banter Pack`, rather than replaying `[0, 1]`
  before Xan.
- Install Spell Revisions core before IWDification. Install IWDification after selected
  NPC and kit mods and before SCS, as recorded in its curated catalog. IWDification `120`
  remains deferred until the required Arcane Trickster Evasion tail patch exists.
- Split Tweaks Anthology `2312` out of the ordinary Tweaks run. Its v18 source executes
  `COPY_EXISTING_REGEXP GLOB ~^.+\.spl$~` and adjusts every currently visible arcane and
  divine SPL. Running it before IWDification, SCS, or later collection spell layers leaves
  their subsequently created spell resources untouched. Run the ordinary curated Tweaks
  components in their normal pre-SCS slot, then run only `2312` in `post-eet-end` after
  SCS, SoD Remix, BG Rebalance, and BG Modpack spell components, immediately before the
  final Spell Revisions `60` / BuffBot tail. No SCS v35.21 source predicate references
  Tweaks `2312`.
- This late `2312` run does not violate the EET boundary. `EET_END` remains the last
  non-`post-eet-end` run; the engine and production tests explicitly permit reviewed
  additive patch runs after that core anchor and require BuffBot to remain absolute last.
- Public `chriz-bg-modpack` `v0.2.0-alpha.1` component `400` is runnable in the exact
  pinned release artifact. It validates `SPIN113` and changes only opcode-111 charges from
  zero to one. The older local repository checkout still contains a FAIL stub and must not
  substitute for the release. Gate `400` on Branwen plus Spell Revisions and run it late,
  after the final spell shapers and before Tweaks `2312`, Spell Revisions `60`, and BuffBot.

The resulting relevant tail is:

`... -> SCS -> EET_END -> reviewed collection layers -> BG Modpack 400 -> CDTweaks 2312 -> Spell Revisions 60 -> BuffBot`.

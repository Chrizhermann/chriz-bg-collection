# Full creator installation

This is the private, fixed profile for reproducing Christopher's complete current EET
stack. It is deliberately separate from `manifest/`, which remains the smaller public
alpha recipe. The repository contains the recipe and integrity metadata, never the
third-party mod payloads.

## Executable profile

- Recipe root: `recipes/creator-full-current`
- Preset: `creator-full` (`Full collection`)
- Recipe release identity: `0.1.0-alpha.2` / `Full creator setup`
- Resolved plan: 90 ordered runs, 488 components, 31 payload artifacts
- Absolute final run: BuffBot components `1`, then `0`

The recipe was generated from these read-only sources on 2026-09-05:

| Role | Source | Entries | SHA-256 |
|---|---|---:|---|
| BG1 preparation order | `C:\BG-EET-RC-20260903\bg1\WeiDU.log` | 28 | `D6E631A248AE2174E3A40C47F36C3A0DCA3AE1FD6ADC34E3AC731C9D5D58C028` |
| Full BG2/EET order | `C:\Games\Baldur's Gate II Enhanced Edition modded - CBR Ambient Readiness v1.2 Test\WeiDU.log` | 456 | `6C988DE31A47812C692EEFBB7108D3B7A826FDD9CEA3DFC29A546C5A7132C2C0` |
| Private/manual source payloads | `C:\Games\Baldur's Gate II Enhanced Edition modded - CBR Ambient Readiness v1.2 Test` | 49 installers | read-only source tree |

`reference/source-bg1.tsv` and `reference/source-bg2.tsv` are generated evidence within
the recipe. They do not replace or modify `manifest/install-order.tsv`.

## Manual cache

The 49 installers not already covered by pinned public-alpha artifacts are stored only in
this local manual archive:

- Cache root: `C:\CEBG-creator-full-cache`
- Required file: `C:\CEBG-creator-full-cache\manual\creator-full-private-extras-20260902.zip`
- Length: `1,263,540,769` bytes
- SHA-256: `3B7DBC1994D57842BBDDCB93726319FC5599BC5E84758C0CB33BD46EF9ED7C38`

The archive has 14,240 entries and 1,665,040,487 uncompressed bytes. All 50 declared
publication roots and TP2 paths are present. Backup directories and Git metadata are
excluded. Never commit, package, publish, or redistribute this archive.

Regenerate it only from the named read-only reference root:

```powershell
python .\tools\creator_full_recipe.py generate `
  --base-recipe .\manifest `
  --bg1-log 'C:\BG-EET-RC-20260903\bg1\WeiDU.log' `
  --bg2-log "C:\Games\Baldur's Gate II Enhanced Edition modded - CBR Ambient Readiness v1.2 Test\WeiDU.log" `
  --source-root "C:\Games\Baldur's Gate II Enhanced Edition modded - CBR Ambient Readiness v1.2 Test" `
  --output .\recipes\creator-full-current `
  --cache-root 'C:\CEBG-creator-full-cache'
```

The generator refuses to replace an existing recipe directory. Move the prior generated
directory aside or use a new output directory when performing a parity comparison.

## Clean install command

Both clean Steam sources were freshly inspected as eligible on 2026-09-05:

- BG1+SoD: `C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate Enhanced Edition`
- BG2EE: `C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate II Enhanced Edition`

Use a new managed root. Never target a source game or any `C:\Games\...` reference:

```powershell
.\target\release\chriz-bg-install.exe install `
  .\recipes\creator-full-current `
  --name 'CEBG Full 2026-09-05' `
  --preset creator-full `
  --bg1 "C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate Enhanced Edition" `
  --bg2 "C:\Program Files (x86)\Steam\steamapps\common\Baldur's Gate II Enhanced Edition" `
  --managed-root 'C:\Users\chris\Games\CEBG-Full-20260905' `
  --cache 'C:\CEBG-creator-full-cache'
```

## Parity deltas and open acceptance

The source logs contain 484 components. The executable profile contains 488: sixteen
source entries are intentionally omitted and twenty maintained-source entries are added.

- `__EXTRACT.TP2` and `__IDS.TP2` are Project Infinity-style generated pseudo-installers.
  Their source TP2 files no longer exist, their log labels are unreadable, and no behavior
  can be reproduced safely.
- The reference modpack runs (`430`, then `440`/`450`/`600`) are replaced by one current
  v0.2.0-alpha.1 run: `110`, `130`, `140`, `170`, `190`, `192`-`198`, `410`, `430`,
  `440`, and `450`. This is the union of the approved ready public-alpha mandatory/default
  choices and the supported reference choices. It adds thirteen components relative to
  the three supported reference selections.
- Old modpack component `600` is deliberately not replayed: v0.2.0-alpha.1 exposes no
  such component or equivalent. Its exact old behavior disabled UB component 3's repeated
  Spellhold-maze Bodhi hunts in `AR1512`/`AR1513`/`AR1514` and added a timing-safe
  `J#RealBodhi` marker so SCS could not deadlock the `AR1514` ultimatum. Installing the old
  modpack beside the current one would install the same mod twice, so this remains an
  explicit functional gap rather than a doomed WeiDU request.
- Nine raw tail installers are replaced by their maintained modpack equivalents: both
  Fade tails -> `110`; Kivan -> `130`; Mazzy -> `140`; Xan -> `170`; Viconia -> `192`;
  Skie -> `195`; Yeslick/Keldorn -> `410`; and the UAI-scroll tail -> `430`. Current
  component `195` performs both Skie's semantic Swashbuckler assignment and her
  stealth-to-lock-picking transfer, so replaying `SKIE_SKILL_FIX` would duplicate it.
- The numeric-kit `NPC_KIT_CHANGES` monolith is unsafe to replay alongside semantic
  assignments. Khalid, Shar-Teel, Kagain, Sarah, Skie, Imoen, and Mazzy are represented
  by current Artisan/modpack components. Sirene deliberately changes from the retired
  Martyr rewrite to the approved native True Paladin component `5`. The monolith's
  Safana-to-Abettor and Aura-to-Bard assignments remain explicit unmapped outcome
  differences; they are not silently treated as reproduced.
- Two old standalone tail installers are not replayed. `AKCB_SHAPESHIFTER` is redundant
  because every greater-werewolf template in Artisan chriz-v1.3.1 already has
  `personal_space=3`. `SR_SUBSPELL_FIX` is redundant because Spell Revisions
  v4.21-chriz.3 performs the hidden-elemental-subspell migration itself. The standalone
  Abettor HLA semantic rebalance is retained exactly, with both its root-level TP2 and the
  sibling `abettor-hla-rebalance` source root required by its `INCLUDE`.
- The reference already uses EEex `1.2.0` components `0` through `7`; component `8`, the
  LuaJIT component required for that release to launch, is added.
- BuffBot `1.8.3-alpha` components `1` and `0` are appended as the absolute final run, per
  the later collection decision. The full source log predates that final layer.
- Overlapping public-alpha mods use their newer pinned artifacts. Generated mod manifests
  retain source-selected Artisan `30001`, Ascension `40`, and Unfinished Business `3`/`19`,
  while the current modpack uses only component IDs actually present in v0.2.0-alpha.1.
- Bardic Wonders `1008` is scripted to answer **Yes** to its Garrick/Troubadour prompt. The
  answer is not recorded in WeiDU.log, but the reference install's Garrick CRE variants
  all carry the Troubadour kit and no later Artisan `99001` assignment is present.
- Static validation and archive-shape/hash checks have passed. Full engine extraction,
  every WeiDU invocation, final log reconciliation, InfinityLoader boot, game start, and
  save/reload remain live acceptance until the first creator-full installation completes.

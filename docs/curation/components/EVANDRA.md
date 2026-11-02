# EVANDRA — components

Listed at installed version **v2.2**.
2 entries, 2 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Component `0` is the core and is included automatically when Evandra is selected.
  Component `1` requires `0` and remains an optional default.
- Evandra v2.2 is a manual, page-gated download and may not be rehosted. The approved
  temporary alpha flow asks **before copying/installing**: download from the official
  page, choose the downloaded file, or skip Evandra (including crossmod component `1`).
  The Windows file is `evandra-v2.2.exe`, 13,430,253 bytes, SHA-256
  `21724b6d4679d6df6dbcf95a0a4dbe6ee41d5bfdb3ae13907ec89f5d00014861`.
  CEBG verifies its exact contents, caches it, and unpacks the RAR payload without
  executing the self-extractor. The Linux ZIP is not requested. See
  `docs/handoffs/2026-09-06-evandra-public-acquisition.md` for evidence/status.

## Related collection layers

- `EVANDRA_SORCERER` is the historical **intended default** class option.
  September 29 read-only checks confirmed that the public recipe and stream install
  lack this conversion; the installed creatures remain Illusionist Mages.
  It requires component `0`, installs
  afterward, and does not conflict with crossmod component `1`. The legacy patch covers
  `rh#eva` and `rh#ev25`, but intentionally does not alter vampire `rh#evamp`; preserve
  that scope is preserved by maintained Modpack component **224**, implemented
  September 29 and now pinned in Modpack alpha.8. It preserves stored level and XP,
  replaces the wizard spellbook using semantic SR/non-SR choices and installed
  Sorcerer tables, and leaves subsequent level-up choices to the player.
- The reference's custom `rh#eval.bmp` is now maintained as Modpack component
  **225**, independent of the class conversion. Christopher confirmed that he
  created it with ChatGPT and approved redistribution on September 29. Asset
  SHA-256: `b4bde98aec195d47db521d3cc4d72b636ae5a0f391cc7469ea5d48688b112aad`.
  A separately authorized personal repair replaced the stream override, retaining
  the upstream image in a backup; no saved character/class was edited by the agent.
- Both maintained options are default-on, require actual Evandra core
  `0`, and are omitted if Evandra is skipped. Crossmod `1` does not gate either.
  **The verified alpha.8 Modpack pin supplies both.** The generated production
  recipe includes them independently; normal installer and signed recipe packaging
  reject any regression that omits either default. The narrower authored fallback
  still blocks Evandra; this does not affect the generated production recipe. See
  [release handoff](../../handoffs/2026-09-29-evandra-default-restoration.md).
- The separate reported XP catch-up failure remains open. Neither new component
  silently levels Evandra up or repairs an actor already stored in a save.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Evandra NPC |  |  | ✓ | mandatory |
| 1 | Crossmod Content |  |  | ✓ | default |

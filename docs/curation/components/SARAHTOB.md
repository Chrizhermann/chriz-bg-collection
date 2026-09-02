# SARAHTOB — components

Listed at installed version **v8**.
2 entries, 1 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Components `0` and `1` are alternative full installations, not audio add-ons. The
  curated UI excludes original audio and exposes only revised audio, so `1` is included
  automatically whenever Sarah is selected.
- Revised audio ships inside the Sarah package; it does not require another download.

## Related collection layers

- The reference install uses custom `sarahL.bmp`, `sarahM.bmp`, and `sarahS.bmp` files.
  They differ from upstream, while the current `portrait_backups` copies match upstream
  exactly. Capture the actual custom portraits, provenance, and hashes in an approved
  collection asset/manual layer before claiming the portrait override is reproducible.
- The reference also changes Sarah to Archer through the monolithic
  `NPC_KIT_CHANGES`. That installer hardcodes kit numbers and its planned modpack
  component `500` is still a FAIL stub. Keep the conversion unavailable until it is split
  into a semantic Sarah-only choice requiring component `0` or `1`.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Use original audio |  | Sarah NPC Romance Mod for BG2:ToB |  | |
| 1 | Use revised audio from HalmyLyseas |  | Sarah NPC Romance Mod for BG2:ToB | ✓ | mandatory |

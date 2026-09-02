# EVANDRA — components

Listed at installed version **v2.2**.
2 entries, 2 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Component `0` is the core and is included automatically when Evandra is selected.
  Component `1` requires `0` and remains an optional default.
- Evandra v2.2 is a manual, page-gated download and may not be rehosted. The UI/install
  flow must identify the required archive and ask the user to provide it.

## Related collection layers

- `EVANDRA_SORCERER` is the default class option. It requires component `0`, installs
  afterward, and does not conflict with crossmod component `1`. The legacy patch covers
  `rh#eva` and `rh#ev25`, but intentionally does not alter vampire `rh#evamp`; preserve
  or revisit that scope explicitly when migrating it.
- The reference uses a custom `rh#eval.bmp`. The current `portrait_backups` copy matches
  upstream rather than the live custom image. Capture the custom portrait, provenance,
  and hash in an approved collection asset/manual layer before treating this override as
  reproducible.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Evandra NPC |  |  | ✓ | mandatory |
| 1 | Crossmod Content |  |  | ✓ | default |

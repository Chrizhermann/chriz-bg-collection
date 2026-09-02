# FADE — components

Listed at installed version **5.6**.
3 entries, 1 installed. ✓ = installed. Subgroup = choose one.

## UI/dependency notes

- Component `0` is the core and is included automatically when Fade is selected.
- Component `2` requires `0` and BG2:EE/EET. It is optional and mutually exclusive with
  the collection's Fighter/Thief conversion because both replace or patch the same Fade
  creature resources.

## Related class choice

Expose the Fighter/Thief conversion as the default related option once
`chriz-bg-modpack` component `110` is implemented. It requires Fade `0` and forms an
at-most-one group with Shadowdancer `2`; choosing neither leaves Fade as the base Thief.
Do not expose legacy `FADE_FT_PATCH` or planned modpack component `120`: the v2
Fighter/Thief conversion already contains the same proficiency and amulet changes.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Fade: An NPC for Baldur's Gate II: SoA and ToB |  |  | ✓ | mandatory |
| 1 | Fade NPC: reactions to Romantic Encounters (RE may be installed before or after this component) |  |  |  | |
| 2 | Fade EE: give Fade Shadowdancer kit |  |  |  | optional |

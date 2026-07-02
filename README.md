# chriz-bg-collection

**The umbrella.** A manifest-driven recipe to reproduce (and share) a heavily-modded
BG:EE + SoD + BG2:EE **EET** install — 84 mods, 414 WeiDU components — with a recommended
preset and room to configure everything to your liking.

**Principle: bundle the recipe, not the mods.** No third-party mods are redistributed here;
the collection pins versions, sources, install order, and component selections, and (later)
drives the installation. Decision rationale: chriz-bg-rebalance
`docs/plans/2026-07-03-umbrella-analysis.md`.

**Status:** bootstrap. The manifest is captured from the reference install; everything else
is pending (see `docs/handover.md`).

## Layout

```
manifest/
  install-order.tsv   # 414 rows: position, mod folder, tp2, language, component #, name
  mod-sources.tsv     # 84 rows: mod folder → version, download URL, archive path (TODO)
presets/              # component-selection presets (chris-full = the reference install)
tools/                # install driver (future)
docs/handover.md      # agent entry point
```

## The composed stack ("chriz layer" at the tail)

The collection composes independent, individually-usable repos:

| Repo | Role |
|------|------|
| [chriz-bg-modpack](https://github.com/Chrizhermann/chriz-bg-modpack) | Consolidated personal fixes (WeiDU mod) |
| [chriz-bg-rebalance](https://github.com/Chrizhermann/chriz-bg-rebalance) | SCS/SR-adjacent balance adjustments (WeiDU mod) |
| chriz-sod-rebalance | SoD encounter remix + companion rebalance (WeiDU mod, WIP) |
| *-Chriz-Balance-Patch | Per-mod patches (Aura, Bardic Wonders, Artisan's Kitpack) |

Everything else in the manifest is third-party — install it from its own source, and go
thank its authors.

## License

MIT for the recipe/tooling. Third-party mods keep their own licenses and are **not**
included.

# Installer v0 static prototype

Open `index.html` directly. It has no dependencies, network requests, game access, or
functional install action. It demonstrates the framework-neutral visual and interaction
direction for the Recipe Preview described in
`docs/plans/2026-09-01-installer-v0-design.md`.

Included states:

- returning-user home with one installed copy;
- curated Setup controls and an engine-authored incompatibility explanation;
- campaign-ledger Review preview;
- Update Center with a clearly deferred next-playthrough recipe update;
- keyboard focus, reduced-motion handling, responsive layout, and a DOM self-check.

The prototype uses illustrative fixture content. It is not the recipe source of truth and
must not be connected to a real Build action.

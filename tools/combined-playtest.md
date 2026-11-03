# Local combined playtest recipes

The public curated recipe now includes the original combined-playtest content.
Create a separate local recipe while retaining its approved components and
published archive pins:

```powershell
python tools/prepare_combined_playtest_recipe.py --output C:/scratch/combined-recipe
```

The destination must not exist. The public recipe is never modified. The output
has no public release identity and is not installation or gameplay acceptance.

For a deliberate source experiment, opt in to each local override:

```powershell
python tools/prepare_combined_playtest_recipe.py --sources C:/scratch/sources.json --local-source chriz-sod-remix --output C:/scratch/local-sod-recipe
```

An older base missing a required mod or component also requires `--sources` and
uses a local archive only for that missing content. Unused source entries and
their archives are not needed. The source packager retains its historical
September 8 pins; those snapshots cannot satisfy newer SoD 115/257/266 or Modpack
188 and must not replace today's published archives.

Before publishing the output directory, the tool verifies each used local ZIP's
hash, size, paths and every component offered by its recipe runs, including
optional components. Component coverage is checked against literal TP2
`DESIGNATED` declarations (or a leading implicit component zero); comments and
string examples do not count. Dynamic declarations or unsupported numbering
cannot establish coverage and cause a clear failure when needed. This bounded
check does not evaluate WeiDU predicates, included runtime code or gameplay.
Published sources retain the base recipe's existing component catalogues and
archive verification contract; they are not downloaded again by this tool.

Only used local archives appear in `combined-playtest-sources.json`. Supply those
exact ZIPs through the installer's ordinary verified manual-cache flow.

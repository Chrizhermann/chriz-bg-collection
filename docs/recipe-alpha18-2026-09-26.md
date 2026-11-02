# Recipe 0.1.0-alpha.18 — Emotion, Courage / Hope default

Christopher decided on **2026-09-26** to make BG Rebalance component **301** a
recommended default, accepted on the basis of earlier in-game testing. Any issue
will be fixed in a follow-up rather than gating this recipe on further testing.
This source candidate follows the completed alpha.17 / app alpha.19 commit
`d57b70a`; it does not publish a package or update an existing installation.

## Source and selection

- [BG Rebalance v0.6.0](https://github.com/Chrizhermann/chriz-bg-rebalance/releases/tag/v0.6.0),
  release merge commit `4aa6110`.
- Asset `chriz-bg-rebalance-v0.6.0.zip`: **1,724,047 bytes**, SHA-256
  `e5f39cb60ecf90a715715d95848df4582168e55cdcff4fc5455bc768f3063224`.
- Fresh anonymous download and installer authoring verification passed on September
  26. Redirect host: `release-assets.githubusercontent.com`. Direct-root ZIP:
  49 file entries, 2,355,002 uncompressed bytes, largest entry 1,364,992 bytes,
  depth 4, maximum compression ratio rounded up 18. Only the mod directory and
  `setup-chriz-bg-rebalance.tp2` are published into an installation.
- The v0.4.0 artifact is replaced with v0.6.0. All previously selected components
  are unchanged; the intervening additions are 301 and bard progression 420/421.
  Bard progression remains unselected and unavailable in this recipe.
- Offered Rebalance order:
  `[100, 101, 110, 111, 120, 121, 301, 400, 401, 404, 405, 407, 408]`.
  Dragon 110/111 remains optional and unchecked.
- `feature:chriz-bg-rebalance:component-301` is default, ready, parent
  `mod:chriz-bg-rebalance`, with no additional requirements. Switching it off
  removes only 301; excluding the parent omits its whole run.

## Effect and compatibility

Casting Courage or Hope removes the other from each affected target. The latest
spell wins, and recasting the same one refreshes it. In the collection 301 rebuilds
IWDification's own spells in place, after IWDification, SCS 5900 and Spell Revisions.
The later CDTweaks 2312 save-penalty pass preserves the matching saved hostile
removers and leaves Courage/Hope alone. Modpack 430 does not change their
fixed-level scroll casts.

Earlier live checks used standalone mode. IWDification provider mode has installer
and transformer fixture coverage only. A quick Hope/Courage replacement and refresh
check in the next installation is worthwhile and explicitly **not a release gate**.
This recipe is for new installations / the next playthrough; it is not a hotpatch.

## Records and validation boundary

Curated defaults: **450 components / 50 runs**; base profile: **358 components**.
Curation: **1,206 rows**, 307 default decisions, 680 feature rows and 11 omission
rows. Reconciliation: 458 default/mandatory rows, 448 selected catalog rows
(catalog rows and expanded components are different counts).

The alpha.18 ledger supersedes alpha.17, requires app alpha.19 and classifies this
change as recommended for the next playthrough. Historical ledgers and the captured
`manifest/install-order.tsv` are preserved. The older unused 0.3.2 artifact/evidence
remains historical; current-pin assertions now check the selected 0.6.0 artifact.

The generator retains the existing `5a5ed2b193fad73c542bb5cd90be79618c0dbb51`
reference for its historical static evidence. It does not relabel that evidence as
new gameplay acceptance. Current source checks are recorded in the handover.
The September 23 app alpha.19 package still bundles recipe alpha.17 and predates
the September 26 feedback correction. No new installer package or release was
published for this source change.

Final checks: 806 native tests passed (13 existing opt-in ignored), formatting
and Clippy passed; the Python suite ran 105 tests successfully with one existing
opt-in skip. Both profiles pass public-alpha validation. Direct planning confirms
450 curated / 358 base components and an independent 301 opt-out; pre-existing
conditional-omission notices are unchanged. Fresh generation matches all 101
recipe files after line-ending normalization. The semantic digest changed from
`62a03115adcea4a9812af4a760944ad81a68f0a0a06c7b3e1b1b2b2d8cb50088` to
`9f5d6131c8c13887a80b6d82b592c440484d84043f38f0a0f5378ee177a29015`, independently
checked against the committed alpha.17 baseline plus only the 301 run component,
feature and default selection. Native evidence:
`target/work/sep26-alpha18-native`, job `4e708c54-f961-425a-9878-fab6c6706bcd`, exit 0.

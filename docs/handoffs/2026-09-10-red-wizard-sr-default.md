# Red Wizard Edwin: restore the default, not a blanket SR ban

## Decision and scope

Christopher explicitly requested an independent investigation and restoration as
a default if no substantiated incompatibility was found. The independent source
review found no installer blocker, disabled kit ability or source-declared SR ban.
Remove only the collection conflict for Artisan NPC **5102**. It remains optional,
default-checked with SR enabled or disabled, before the late SR NPC-spellbook pass.
Other kit exclusions and source pins are unchanged. Draft app alpha.16 / recipe
alpha.14 now selects **436** components in 44 runs (previously 435).

This is a collection-source change, not a public release, game patch, new install
or native gameplay acceptance. The stream and Combined-20260908 do **not** contain
5102; their files, saves, receipts and frozen recipes remain unchanged. In
particular, do not claim the completed 446-component combined test validated this kit.

## Why the old exclusion was not justified

- The preserved September 2 curation snapshot (`f945974899209921551bb5ee5e9a3ee45378bc51`)
  says "unavailable with SR" while still marking 5102 default/reference-installed.
  It provides no mechanical explanation or source citation. The runtime conflict
  was added by merge `4d666b9691c2dcaf38bc71a50d7d3c4b01d49997`.
- The historical reference TSV records SR main component 0 and Artisan NPC 5102
  together. That proves prior recorded installation, not gameplay compatibility.
- The pinned Artisan `ac718614991e34b4f720807bec5edc96266c6c5e` and reviewed current
  HEAD `b78f247ab35d42a5219ec47ae914c7078435f486` have identical Red Wizard library
  and asset blobs. `ArtisansKitpack_NPC/ArtisansKitpack_NPC.tp2:623` only requires
  BGEE/BG2EE/EET before including `ArtisansKitpack/lib/redwizard.tpa`; no SR gate.
- Binary inspection of `C0REDW`, `C0REDW1` and `C0REDW2` found opcode-346 effects
  using school 2 (Conjuration). SR preserves that school identifier. No SR-specific
  failure of this aura/defense mechanism surfaced in the bounded review.

## Narrower follow-ups, owned by the mod repositories

1. **Artisan starting-spell cleanup:** `ArtisansKitpack/lib/redwizard.tpa:149-150`
   removes fixed vanilla spell IDs, including `SPWI106`. SR repurposes it to
   Conjuration's Obscuring Mist (`spell_rev/lib/d5_spell_school_list.tpa:92`;
   Combined's effective `dvscrlmap.2da` confirms the mapping). Edwin can lose an
   allowed starting spell if present. This is not a permanent learning prohibition.
   Replace fixed IDs with effective prohibited-school checks in the Artisan repo.
2. **SR60 custom-kit cleanup:** `spell_rev/components/fix_arcane_spellbooks.tpa:140`
   reads a single kit byte and compares vanilla specialist identifiers. A custom
   `C0REDWIZ` generally bypasses school-specific cleanup. It does not reset his kit,
   replace his CLAB or remove his kit abilities; generic hidden/bogus cleanup still
   runs. Investigate separately in the SR repo. Reviewed SR files match the
   collection's `v4.21-chriz.3` source and the inspected current source.
3. **EET variants / balance, not SR-specific:** the current Red Wizard CRE list
   omits `EDWIN7_`. Check joinable EET variants separately. The amulet replacement
   and historical baked-slot repair also need their own balance review; do not
   silently add the unreviewed legacy amulet/slot patches.

The independent review was read-only, with no native game test. A focused future
test should use newly recruited Edwin: kit name and grants, school restrictions,
spellbook contents, aura, rest and save/reload. Existing saved Edwin requires a
separate migration assessment, not merely changing a kit number in EEKeeper.

## Collection verification

The regression first failed with SR enabled, reproducing the unsupported exclusion.
It now asserts default selection with SR on/off, explicit opt-out, parent opt-out,
preserved unrelated exclusions, ordering before SR60, and removal of the stale
limitation from the newly generated recipe. Historical authored release evidence
is retained; the current draft ledger explicitly describes the restored default.

Verification passed: 27 focused Python tests, public-alpha CLI recipe validation
with no findings, and real CLI planning. Red Wizard on selects 436 components in
44 runs; opting out selects 435 in 44 runs while retaining Spell Revisions.
These are selection/validation checks, not a fresh installation or gameplay test.

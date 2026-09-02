# Installer v0.1 real-alpha design

**Date:** 2026-09-02

**Status:** Approved for implementation

**Extends:** [Installer v0 campaign builder and update center](2026-09-01-installer-v0-design.md)
**Changes the milestone:** The first public milestone is a real, recoverable installation,
not the fixture-only Recipe Preview described as the earlier v0 stopping point.

## Outcome

The first release is a paired alpha launch:

1. Christopher builds a complete Baldur's Gate 2.7.3 EET installation with the same
   installer and public recipe, launches it through InfinityLoader, and smoke-tests it.
2. `v0.1.0-alpha.1` is published as a Windows x64 installer that ordinary players can
   use with legally owned BGEE+SoD and BG2EE installations.

The app's safety and recovery behavior must be release-quality. The curated recipe may
remain explicitly alpha: unresolved components are unavailable, experimental components
are labeled, and curation can continue in parallel. A green static preview or synthetic
WeiDU run does not satisfy this milestone.

The collection bundles the recipe, not third-party mods. It downloads pinned artifacts
from their approved sources and never redistributes them. Private or nonredistributable
extras may be supplied manually, but the public recommended setup should remain as close
to Christopher's experience as licensing and provenance permit.

## Audience and product promise

This is a guided campaign builder, not a general-purpose mod manager. It is intended for
players who may never have installed a WeiDU mod. The happy path is:

1. Find two clean games.
2. Accept Chris's recommended collection.
3. Choose a safe destination.
4. Let the app download, build, verify, and launch it.

Mandatory and default-on content create the recognizable collection experience. Advanced
choices remain available without making component numbers, WeiDU terminology, or install
order the primary interface.

The central safety promise is:

> Your store installations and existing saves are left alone. The collection is built in
> a separate copy, can stop safely, and can be resumed or rebuilt from its receipt.

## Scope

### Included in v0.1

- Windows x64.
- Steam BGEE+SoD and BG2EE 2.7.3 as the verified storefront/build combination.
- GOG discovery and staging, labeled experimental until a complete clean GOG rehearsal
  passes.
- English (`en_US`) game and recipe output for the first alpha; additional language
  combinations require their own source fingerprints, translations, prompt answers, and
  end-to-end rehearsal.
- Separate BGEE+SoD and BG2EE discovery, validation, and isolated staging.
- One recommended preset plus settled curated options.
- Official/manual acquisition, SHA-256 verification, caching, and safe extraction.
- The full EET phase sequence, including two logical runs from one EE Fixpack artifact.
- Persistent progress, fail-closed execution, crash reconciliation, and resume.
- Per-run WeiDU.log verification, an immutable installation receipt, and diagnostics.
- Launch through InfinityLoader when EEex is selected.
- Independently signed application and recipe update channels.
- Rebuild-only recipe updates for managed installations.

### Explicitly outside v0.1

- Installing into or changing a store/source game.
- In-place component uninstall or stack mutation.
- Executing hot patches against an existing installation.
- Save inspection or claims that an update applies to a particular existing save.
- Epic Games Store, Microsoft Store, legacy Beamdog Client, macOS, or Linux support.
- A general dependency-solving mod manager or arbitrary user-supplied install order.
- Bit-for-bit identity between installations; the reproducibility contract is the
  expected plan, verified WeiDU.log, immutable receipt, and a bootable EET result.

## Architecture

Use a native Rust orchestration engine with a thin Tauri 2 desktop UI.

```text
signed recipe + selection
          |
          v
  validate and resolve  ---> review projection
          |
          v
 acquire and stage copies
          |
          v
 persistent orchestrator ---> event stream ---> Tauri campaign ledger
          |
          v
 WeiDU runner + log diff
          |
          v
 receipt + diagnostics + launch
```

The Rust engine owns all consequential state: source validation, curation semantics,
selection normalization, ordering, downloads, staging, process execution, verification,
resume, receipts, and update classification. The frontend renders engine projections and
events; it does not duplicate dependency or compatibility rules.

The implementation begins with a thin but real safety spine. Recipe authoring and source
freezing proceed alongside the remaining engine work, then meet in a cold-cache release
candidate installation. Project Infinity may inform metadata research, but it is not the
runtime installer.

## Game discovery and source validation

The Find games screen contains independent dropdowns for:

- Baldur's Gate: Enhanced Edition with Siege of Dragonspear.
- Baldur's Gate II: Enhanced Edition.

Each discovered choice shows storefront, game version, path, one overall eligibility
state, and every applicable finding. Findings include:

- `Fresh`
- `Modified`
- `Unsupported version`
- `Missing SoD`
- `Unverified storefront`

The best valid candidates are preselected and each dropdown includes Browse. Steam
libraries are discovered first. GOG registry/install records are also discovered, but
GOG remains visibly experimental until its acceptance run exists.

Version alone does not establish freshness. A storefront/build-specific probe inspects:

- executable product version and selected core-resource fingerprints;
- `chitin.key`, the root and language `dialog.tlk` files, and expected DLC resources;
- `WeiDU.log`, `override`, `setup-*`, mod folders, debug logs, and DLC Merger residue;
- the consistency of the SoD DLC archive and any prior merge markers.

Unknown files or fingerprints produce an explained non-fresh result rather than an
optimistic guess. Several findings may apply at once; for example, a candidate may be both
unsupported and missing SoD. The app never offers a modified source as a normal install
source.

The first alpha supports an English (`en_US`) target only. It validates the English TLK
identity and uses an authored WeiDU language number for every mod; it never assumes the
same numeric language id across installers. A different installed display language is not
treated as supported merely because an English TLK happens to exist.

The selected store paths are read-only inputs. Before any mod process starts, the app
creates separate BG1+SoD and BG2 copies in a user-writable location and repeats the
validation against those copies. It refuses a destination that is equal to, inside, or a
parent of either source. Containment checks use canonical paths and do not trust textual
prefixes. Copies contain independent regular files: the app does not create hardlinks to
source files, and it rejects unreviewed symlinks, junctions, or other reparse points rather
than following them outside the source root. Post-copy identity checks prove expected
content was copied while source file metadata and hashes remain unchanged.

The creator build also carries an exact canonical denylist for the two protected local
reference directories documented by this repository. It does not reject an unrelated
player installation merely because that player chose a conventional `C:\Games` path.

The current machine audit illustrates the rule: Steam BG2EE 2.7.3 is clean, while the
Steam BGEE+SoD directory contains stale DLC Merger 1.7 residue beside a newly updated
2.7.3 DLC archive. The latter must not be used as fresh input. For v0.1 it requires store
repair or clean reacquisition followed by a fresh scan; deleting the known residue is not
enough to prove freshness.

## EET build phases

The recipe models the two game roots and these explicit phases:

1. **BG1+SoD preparation**
   - DLC Merger 2.1 first for Steam or GOG.
   - The 2.7 language fix where applicable.
   - EE Fixpack.
   - Approved pre-EET BG1 content, including relevant BG1 Unfinished Business content.
2. **BG2 preparation**
   - EE Fixpack against the fresh BG2 copy.
3. **EET initialization**
   - EET receives the prepared BG1+SoD copy as its source.
4. **EET-native curated stack**
   - Every selected run follows the frozen recipe order.
5. **Finalization and reviewed tail**
   - EET_end finalizes the installation.
   - Only explicitly inventoried and reviewed post-EET_end exceptions may follow.

The engine must support two logical runs of the same downloaded artifact. That is needed
for EE Fixpack and must not be faked as two unrelated mods.

This phase model follows the official EET installation sequence and the current
[DLC Merger](https://github.com/Argent77/A7-DlcMerger) release. The recipe pins DLC
Merger 2.1 rather than reproducing the stale 1.7 state found locally.

## Curation projection

Catalog decisions have one stable meaning throughout authoring, engine resolution, and
the UI:

| Catalog value | Runtime and UI meaning |
|---|---|
| blank | Excluded and not exposed. |
| `optional` | Visible and unchecked by default. |
| `default` | Visible and checked by default. |
| `mandatory` | Included whenever its parent feature is active; no separate checkbox. |

Ready components follow those rules. Experimental components carry a visible label and
note. An unresolved or incompatible option is disabled with an authored reason when that
helps the player; otherwise it remains hidden. The UI never substitutes a placeholder or
silently omits a promised default.

Compatibility and grouping are authored recipe data evaluated by Rust. For example, SCS
component 4240 remains visible but unavailable while Spell Revisions is active, with the
reason shown beside it. Atomic features such as a reviewed multi-component kit rework are
presented as one player-facing option while retaining their exact component expansion in
the resolved plan and receipt.

Curation and owning-repository fixes continue independently. A completed, pinned change
becomes part of a new immutable recipe release. An unresolved default must be finished or
explicitly accepted as an alpha omission before release; it never simply disappears.

## Acquisition and trust

The installer downloads mods; it does not download the games. The user must already own
and install BGEE+SoD and BG2EE.

Each public artifact has an immutable source URL, expected archive identity, expected TP2
path, version, and SHA-256 digest. Normal acquisition is:

1. Reuse an already verified cache entry when available.
2. Stream to a partial file without replacing a valid cached artifact. Resume a partial
   response only when its URL, expected length, and server identity (`ETag`/`If-Range` or
   equivalent immutable evidence) still match; otherwise restart it.
3. Verify SHA-256 before extraction.
4. Extract into a new staging directory with traversal, link, collision, depth, and size
   limits.
5. Prove the expected installer entry point exists before the staged source is accepted.

Retries reuse safe partial downloads. A changed URL result, missing artifact, mismatched
hash, or suspicious archive stops the build with a persistent explanation.

For page-gated/manual sources such as Evandra, the app opens the official page, identifies
the expected file and digest, and waits for the user to select or drop the archive. The
same verification and extraction rules then apply. Authoring-mode all-zero hashes are not
allowed in a public recipe.

Public release rehearsal starts with an empty cache. It must prove the recipe has no
undeclared dependency on Christopher's archive folders. Source licenses and upstream
redistribution guidance are release gates even when a fork is publicly reachable; where
necessary the recipe fetches official upstream content and applies a narrowly distributed
collection patch instead of mirroring an entire third-party mod. That patch is distributed
only when its own source material and license or explicit permission allow it; narrowness
alone is not permission.

## Preflight

The review/build transition is allowed only after preflight proves:

- both source games are supported, fresh, readable, and mutually consistent;
- the destination is new or an explicitly resumable managed target and is writable;
- available space covers both staged copies, downloads, extraction, and safety margin;
- every required source is present or has a valid acquisition route and hash;
- the recipe digest, normalized selection, and resolved-plan digest are frozen;
- game, InfinityLoader, EEex, WeiDU, and relevant setup processes are not using the
  target;
- root and language `dialog.tlk` files in the staged targets are writable;
- the required WeiDU version is pinned and available.

Preflight findings distinguish fixable user actions from recipe defects. A recipe defect
cannot be overridden from the ordinary UI.

Starting or resuming a build acquires an exclusive lock for that managed target. App or
recipe replacement is deferred while a build is active. Immediately before every
mutating WeiDU invocation, the engine repeats the relevant target-process and TLK
writability checks; passing the initial preflight is not treated as a permanent lock on
external state.

## Execution, verification, and recovery

The orchestrator executes one WeiDU operation at a time. It supplies only recipe-authored
component choices and prompt answers. An unknown prompt pauses for attention; the app
does not guess and does not silently kill a legitimate long-running installation.

Real installs do not use WeiDU `--quick-log`, because it rewrites descriptive information
for the whole stack. Installer and log paths are constrained to normalized, target-local
or dedicated-log locations; absolute and traversing paths are rejected.

After every invocation the engine compares the before and after WeiDU.log snapshots. A
successful run requires all expected additions, no missing components, and no unexplained
removals, replacements, or extras. Exit code zero by itself is not success. Warnings and
the corresponding debug/log evidence remain part of the receipt.

An append-only campaign ledger persists before and after each consequential transition.
Only a verified operation advances. On failure:

- the current step is marked failed and all later steps remain pending;
- the target is visibly incomplete and cannot be launched as complete;
- the player can Retry, Start over in a new copy, or Export diagnostics;
- no component is skipped to make the build continue.

Resume first reconciles filesystem and WeiDU.log evidence. This covers the crash window
where WeiDU completed but the session update did not: an already proven component is
recorded as complete rather than blindly reinstalled. A changed recipe, selection, plan,
source artifact, or source-game identity requires a new build.

The normal Build screen shows friendly phase and mod labels on a campaign-ledger rail.
An expandable technical console exposes exact process output without making it the main
experience.

## Receipt and diagnostics

Every attempt produces a machine-readable receipt, including failures. A successful
receipt records at least:

- install id, target paths, distinct `engine_name`, timestamps, and final outcome;
- application, engine, schema, and recipe versions;
- source game storefronts, versions, and freshness fingerprints;
- recipe digest, normalized selection, resolved plan, and plan digest;
- every artifact's approved URL, version, SHA-256, and acquisition result;
- WeiDU version and per-run invocation identity, timing, result, warnings, and log diff;
- final expected/observed WeiDU.log identity and verification summary;
- launch executable and managed save location.

The diagnostics bundle contains the receipt, session ledger, sanitized configuration,
and relevant logs. It excludes private archives, game content, credentials, and unrelated
personal paths where redaction is practical.

When EEex is selected, Complete and Home launch through `InfinityLoader.exe`, never the
vanilla game executable.

Before the first launch, staging assigns a unique normalized `engine_name`, derives its
actual per-user data/save directory, and refuses a collision with an unmanaged or
different installation. Final verification reads back the configured identity and proves
that the corresponding managed save root is distinct from the source games' save roots.

## Application and recipe updates

Application and recipe releases use independent signed channels from v0.1.

- The Tauri application updater verifies its mandatory release signature before replacing
  the application.
- A recipe release is an immutable, separately signed envelope containing its content
  digest, source hashes, change ledger, and minimum application version.
- Invalid signatures or digests never replace the last trusted application or recipe.
- Offline checks are nonfatal and show the last successful check time.

The application embeds recipe and update trust roots. Signed metadata is monotonically
versioned and includes a key id; a signed key-rotation statement is required before a new
key is trusted. Downgrades and replayed older metadata are refused by default. A running
or resumable build retains its exact trusted recipe snapshot even if a newer recipe is
downloaded.

The mandatory alpha guarantee is Tauri updater signature verification. Windows
Authenticode is a separate distribution-signing mechanism: use it if a suitable
certificate is available, but do not describe an unsigned executable as Authenticode
signed. If the alpha lacks Authenticode, the release guide must state the expected
SmartScreen warning and publish independent SHA-256 values.

The Updates screen distinguishes an application update, a recipe update, and the state of
each managed installation. Every recipe change classifies save applicability independently
from urgency.

Supported applicability values are `current_save`, `before_npc_join`,
`before_area_visit`, `before_event`, `next_playthrough`, `new_game_only`, and `unknown`.
Urgency values are `critical`, `recommended`, `optional`, and `informational`.

These values are authored generic guidance about the change. In v0.1 they never claim
that the app inspected a player's save or proved that a condition is true for that save;
conditional labels must say what the player needs to know or remain `unknown`.

For v0.1, the only recipe-update action for an existing installation is **Build updated
copy**. A working installation and its saves remain untouched. The UI clearly says when
a change is only useful for a future playthrough rather than presenting every release as
urgent.

## Future hot-patch boundary

The recipe schema may record hot-patch eligibility in v0.1, but no in-place executor is
enabled until a separate safety design and live acceptance exist.

Direct `dialog.tlk` changes are rebuild-only by default. Anything that allocates or
changes strings, dialogue, names, descriptions, SAY/TRA content, or order-sensitive
WeiDU state is not a simple override patch. Even an override-only resource may already
be cached in a save, area, store, or creature instance.

A future hot patch therefore needs an explicit semantic allowlist, a closed game, exact
input hashes, backup and rollback, atomic application, post-write verification, receipt
amendment, and an authored save-applicability classification. “It only copies into
override” is not sufficient evidence.

## Player-facing screens

1. **Welcome** — explains owned-game requirements and the separate-copy promise.
2. **Find games** — two dropdowns with storefront/build/freshness status and Browse.
3. **Destination** — proposes a safe writable path and shows space requirements.
4. **Setup** — leads with “Chris's recommended collection”; defaults are on, mandatory
   details are included, and optional choices are secondary.
5. **Review** — summarizes gameplay content, destination, downloads/manual steps,
   warnings, and the five EET phases.
6. **Build** — shows the persistent campaign ledger and expandable technical log.
7. **Complete** — shows verification, receipt, diagnostics and save locations, and Launch.

Returning users see managed installations with Play, Open folder, Diagnostics, and
Updates actions. Player-facing groups use terms such as companions, quests,
rules/abilities, combat/challenge, and convenience/presentation. Raw component numbers
are available only in technical details.

Errors use persistent cards with a cause and recovery action, not transient toasts.
Every workflow is keyboard-complete, has visible focus and WCAG AA contrast, works at
200% zoom, honors reduced motion, and never communicates status by color alone.

The visual direction may retain the campaign-build ledger palette—blackened iron,
charcoal, vellum, brass, arcane teal, and ember—without blocking the release on a final
product name or elaborate branding.

## Public alpha acceptance

`v0.1.0-alpha.1` is ready only when all of the following are demonstrated from the
packaged Windows app under a normal user account:

1. A clean Steam 2.7.3 pair is detected; a deliberately modified source is rejected.
2. A cold cache downloads and verifies every public source, including the documented
   manual flow, without relying on local mod archives.
3. The complete recommended alpha recipe finishes with no undocumented manual step.
4. Forced interruption during download, staging copy, and WeiDU execution resumes without
   stack corruption or duplicated installation.
5. Hash mismatch, unavailable source, suspicious archive, unexpected prompt, insufficient
   disk, and active-game conditions all fail closed.
6. The original store installations remain byte-for-byte unchanged.
7. The final WeiDU.log matches the resolved plan and the immutable receipt is complete.
8. The result launches through InfinityLoader, reaches the main menu, starts the BG1
   campaign, and can save, exit, relaunch, and reload.
9. Targeted smoke checks cover EET merge/finalization, EEex 1.2 and LuaJIT, the pinned SCS
   35.21/WeiDU 249 stack, and selected custom/rebalance/SoD layers whose acceptance gates
   require runtime evidence.
10. A signed recipe update appears in the UI and offers Build updated copy while leaving
    the existing installation untouched.
11. The public release contains the updater-signed installer package, versioned signed
    recipe, source,
    concise getting-started guide, and honest known limitations.

GOG becomes a verified target only after the same end-to-end rehearsal passes on clean
GOG inputs. Until then it remains discoverable but experimental.

Alpha describes incomplete mod coverage and storefront evidence. It does not relax
source isolation, integrity checking, deterministic execution, recovery, or reporting.

## Delivery sequence

```text
curation consolidation + BG1 pre-merge capture
                         |
source, license, version, and hash freeze
                         |
production manifests and recommended preset
                         |
engine invocation + verification -> runner -> real WeiDU harness
                         |
discovery/staging + acquisition -> orchestration -> CLI/receipts
                         |
Tauri screens + signed recipe/app update channels
                         |
Christopher's clean 2.7.3 release-candidate build and runtime smoke
                         |
empty-cache packaged-app rehearsal
                         |
public v0.1.0-alpha.1
```

The engine, app shell, source freeze, and owning-mod work should run in parallel where
their contracts are already settled. They converge at the production recipe and release
candidate; unresolved content must not weaken the installer safety spine.

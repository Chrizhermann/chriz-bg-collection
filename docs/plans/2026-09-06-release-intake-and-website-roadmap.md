# Release intake and website coordination

User decisions: 2026-09-06. Keep this task focused on releasing CEBG;
owning mod tasks retain implementation and detailed design work.

## Approved next recipe

- Progressive utility XP **610**: include in the recommended installation;
  implement as default-checked optional, with its EEex prerequisite.
- Yoshimo **220**: include the Swashbuckler option in the recommended companion
  changes and the vanilla-companion bundle control.
- Hexxat **221/222/223**: expose Shadowdancer, Fighter/Thief and Assassin as
  mutually exclusive choices. Christopher explicitly chose **Shadowdancer (221)**
  as the default. The vanilla-companion bundle disables this conversion too.
  This replaces the older selected Artisan NPC **7104 Invisible Blade**;
  keep it as an unchecked alternative with cross-mod exclusivity, not a second
  conversion applied before Shadowdancer. The matching alpha.9-to-alpha.10
  default delta is **+3**. The authoring checks currently report 344 engine
  plan entries and 431 selected curation rows; these are different count
  surfaces from r5's 430 installed WeiDU rows. Do not predict a new installed
  total of 433 by mixing these counts.
- SoD Remix **v0.6.6**, victory ending **290**: include in the existing default-on
  SoD bundle. Repair-only **291** is not part of a fresh installation.
- Imoen **620** and new dragon work: Christopher confirms more work is needed;
  remain in their own repositories, not the current recipe.
- Full SoD skip: Christopher reports it is nearly ready and wants it included
  if ready. Request the owner's current evidence/release, not the stale failed
  prototype. Separate from ending 290; integrate the final tested package once
  available. Do not claim an unreleased prototype is in the installation.

Fresh SoD-owner checkpoint: skip is **910**, with EET, EET_end0 and Remix
110/140/150/160 prerequisites, no290 dependency. Candidate20260906-r1 is
uncommitted, not the old19e221b HEAD.38 automated tests and earlier prompt/No
checks passed; latestYes-to-BG2, XP/equipment/reload acceptance remains, then
commit/package/release. Imported-Imoen and above500k-XP edge fixtures also
remain unverified. This is progress, not yet an ingestible release.

The bounded recipe task has completed **recipe alpha.10** source integration
and focused verification; see [intake evidence](../recipe-intake-2026-09-06.md).
This is not packaged, installed or published acceptance. Preserve the existing recipe choices except these
approved additions and the superseded 7104 default. The app version is
independently alpha.9. Current r5 and
its receipt remain unchanged while Christopher plays. No new full installation.

## Website owner and content handoff

**Build interactive BG run page** in `twitch-setup-chriz` owns the website-ready
roadmap draft and presentation (task `01a061fe-a68c-74b1-8272-5bfa9251a731`).
Coordination request sent; no new task or public deployment was created here.

The website owner has now completed its dormant draft:
[collection-roadmap.draft.json](C:/Users/chris/.codex/worktrees/a021/twitch-setup-chriz/bg-site/content/collection-roadmap.draft.json).
It contains 16 compact Available/In progress/Planned items, owner-reviewed
boundaries and an internal missing-facts list. Owner reports JSON/schema,
unique-ID, public HTTPS-link and 47 site-test checks passed. The current site
build does not include this draft; it is uncommitted and not deployed. Review
and presentation remain with the website task, not this installer task.

Keep three honest labels: **Available**, **In progress**, **Planned**. Use links
to owner-maintained roadmaps and release pages, not a second detailed plan here.
No release dates or promises inferred from prototypes. The public installer
button waits for its real signed artifact and verified URL.

Owner snapshot requests sent:

- SoD task `019f6539-74ef-7260-93c3-00b63cee296a`: full-skip readiness and existing
  bridge, Ashatiel, campaign-filler and other approved roadmap items. Continue
  current work; return a small snapshot at a convenient checkpoint.
- **Begin script engine Task 1** (`01a042de-9380-7a52-9275-11f87682075e`): concise
  gamer-facing capabilities and implemented/WIP/planned distinctions. No extra
  engine implementation requested for the website.
- **Review open rebalance todos** (`01a07181-c25c-7c53-a3ef-39623e56ea3d`): existing
  dragon and rebalance roadmap, separating released ambient/readiness work from
  future encounter changes. No new design cycle requested.
- Modpack: website owner can use its current component catalog, utility-XP and
  Imoen documents. Released 610/220-223 are not the same as unreleased 620.

## Later CEBG feature: choose a game UI

Offer a simple optional UI selector, keeping the recommended UI as the default.
Before selection/install, check declared game/version support, dependencies,
mutual exclusions and install order, including EET, EEex, BuffBot, Bubb's spell
menu and other UI-changing components. Explain any affected features before
changing the selection. Do not claim all newer UI mods are compatible by age
alone. Candidate research and focused UI acceptance belong to that later task,
not the current alpha release.

## Current exclusivity UX, checked after Christopher asked

`app/src/screens/setup.ts` blocks non-interactive checkbox clicks and displays
the authored incompatibility reason inline. `engine/src/recipe_view.rs` projects
unavailable/effective choices into the install plan; known conflicting choices
are not both installed. Direct clicks do not automatically deselect another
option (`app/src/app.ts::toggleFeature` changes one ID only). Switching a
mutually exclusive alternative currently requires unchecking the active one
first. `Include all compatible` skips conflicts; bundle collateral appears in
`Also adjusted` details. These protections depend on authored metadata, not
automatic discovery of unknown mod incompatibilities.

**Approved post-alpha patch (2026-09-06):** Christopher accepted this improvement
but explicitly deferred it to a patch. Do not expand the initial-alpha scope.
Express same-purpose alternatives such as Hexxat
as a single radio/dropdown group, or a clear Switch-to action that atomically
replaces the previous choice and names related changes. Broader incompatibilities
should stay explicit and must not silently disable a major mod. This is a UX
gap queued for the post-alpha patch, not functionality already implemented or packaged.

## Current installer work remains here

Recipe intake and safe pause/app-close are delegated bounded implementation
slices. Finish package signing/self-update acceptance and public Evandra
acquisition; preserve Radar launcher convenience in the short release plan.
No desktop/browser/game control while Christopher uses the PC. No public
installer/update feed or website deployment implied by these content handoffs.

Safe-pause helper has completed its source slice and focused automated checks:
cooperative pause after a durably completed step, resumable paused state without
a failure receipt, separate force-stop action, and active-build close handling.
Native close-dialog interaction and real long-running WeiDU pause acceptance
remain untested; do not claim a packaged/live pass. No commit or UI launch yet.

Signing checkpoint: the existing key accepts an explicitly empty password.
The previously built alpha.9 NSIS candidate was signed successfully without
printing or changing the private key. Its bundled-public-key verification and
real Tauri loopback alpha.8-to-alpha.9 download/signature/tamper-rejection tests
passed using the existing acceptance harnesses. This candidate predates the
new recipe/lifecycle changes. A bounded background rebuild and signing of the
integrated candidate is now assigned to the lifecycle helper. It must only
package and perform headless signature/download/tamper checks, not launch the
installer or apply an update. The earlier candidate was
not installed or launched, and automatic apply/restart remains untested.

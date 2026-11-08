# Our own mods: one release in every installer

Christopher, 2026-10-28: when we pin a new version of one of our own mods, for example
BuffBot, it goes into both installers as a new release right away. Chriz Easy BG and
Chriz Easy IWD ship the same release of each of our mods, and that release is the mod's
newest one, unless a recorded hold says why not.

These are the rules every agent follows on every pin of one of our mods. CI checks them
(`.github/workflows/own-mod-pins.yml`); the check is `tools/own_mod_pins.py`.

## Which mods are ours

Every mod released from Christopher's GitHub account, forks included (his decision of
2026-10-28). [`manifest/own-mods.toml`](../manifest/own-mods.toml) lists them by
repository. On 2026-10-28 these are: BuffBot, Chriz BG Modpack, BG Rebalance, SoD Remix,
EEex Remote Console, the voicepacks, Chriz IWD Encounters, and our forks of Artisan's
Kitpack, Bardic Wonders, Spell Revisions and the Item Randomiser.

A new mod on his account goes into that list before its first pin. The check fails when
a recipe installs anything from his account that the list does not name.

## Which recipes

| Installer | Recipes it ships | Where |
|---|---|---|
| Chriz Easy BG | `manifest/` and `recipes/curated-full-current/` | `main` |
| Chriz Easy IWD | `recipes/iwd-current/` | `main` |

`recipes/creator-full-current/` is a historical replay that the app refuses to install.
Its old pins stay as they are.

## On every pin of one of our mods

1. **Pin the newest release.** That is the release GitHub marks as Latest. EEex Remote
   Console publishes tags only; there it is the highest version tag. Keep exact pins:
   URL, size and SHA-256 from a fresh download (Christopher, 2026-10-28). Only add-ons
   outside the install order, such as the Radar overlay, take Latest at install time.
2. **Pin it in every installer that ships the mod, in the same step.**
   `tools/prepare_own_mod_pin.py` does the mechanical part (see "Preparing a pin" below).
   - Chriz Easy BG: `manifest/artifacts` and `manifest/mods`, then regenerate
     `recipes/curated-full-current` with `tools/curated_full_recipe.py`. Superseded
     artifact TOMLs and evidence records stay as history; the tests that freeze a pin move
     to the new one.
   - Chriz Easy IWD: `recipes/iwd-current`. Its static test
     (`engine/tests/production_recipe_iwd.rs`) lists every artifact file, so the new
     artifact TOML replaces the superseded one (`superseded_artifacts = "remove"` in the
     registry). The static-test record of the mod's run in
     `recipes/iwd-current/releases/v0.1.0-alpha.1/acceptance.toml` names a commit of the
     pin PR; merge that PR with a merge commit, so that the commit stays on the branch
     ([PR #74](https://github.com/Chrizhermann/chriz-bg-collection/pull/74)).
   - Both recipes live on `main`; prepare and review their pins in one pull request.
   - A mod only one installer ships today stays there. Adding it to the other installer
     is a curation choice, and curation is Christopher's.
3. **If the newest release cannot go in yet, record a hold** in `manifest/own-mods.toml`.
   Only Christopher decides a hold. It names the release it keeps out, the reason and the
   date he decided it; while the installers differ, it also names each installer's pin:

   ```toml
   [[holds]]
   mod = "buffbot"
   release = "v1.12.1"
   reason = "Christopher tests v1.12.1 himself first; then it goes into both installers together."
   decided = "2026-10-28"
   installers = { chriz-easy-bg = "v1.11.0", chriz-easy-iwd = "v1.12.0" }
   ```

   The PR that ends a hold removes it; a hold that no longer matches the pins fails the
   check. A hold covers only the release it names: the next release needs a new decision.
   A commit-pinned mod such as EEex Remote Console names full commits in `installers`.
4. **Keep the evidence and tests as before.** The worked examples are BuffBot v1.12.6,
   prepared with `tools/prepare_own_mod_pin.py`, in
   [PR #75](https://github.com/Chrizhermann/chriz-bg-collection/pull/75) (Chriz Easy BG) and
   [PR #74](https://github.com/Chrizhermann/chriz-bg-collection/pull/74) (Chriz Easy IWD):
   - a fresh anonymous download, with `chriz-bg-author artifact inspect` and
     `artifact verify` for each recipe's artifact TOML;
   - `tools/check_native_component_order.py` on the fresh archive;
   - a record in `manifest/evidence/release-intake-*.toml`;
   - the TP2 comparison against the previous pin;
   - the component catalog and the changelog draft;
   - the tests that freeze the pin, and the plan digests in `engine/tests/feature_groups.rs`.

   New components and changed defaults stay off until Christopher decides.
5. **Run the check before you open the PR**, and quote its table in the PR body.

## Running the check

```powershell
$env:GH_TOKEN = (gh auth token --user Chrizhermann); python -I tools/own_mod_pins.py check --base origin/main
```

It prints one row per mod: each installer's pin, the newest release and the state. It
reads both installers from this working tree. A missing recipe is an incomplete checkout.
`--offline` skips GitHub and checks only
what needs no network.

What turns it red (exit 1):

- Installers pin different releases of one of our mods, and no hold records that split.
- One installer pins two releases of one mod, or one URL with two different checksums.
- A recipe installs something from Christopher's account that `manifest/own-mods.toml`
  does not list, or the app bundles a recipe that no installer lists.
- A hold no longer matches the pins, or every installer already has its release.
- The change pins one of our mods, but not its newest release, and no hold covers it.
- A release check, and one of our mods is not on its newest release without a hold.

Everything else that is behind shows as a warning, so that a new release of one of our
mods does not turn unrelated PRs red (Christopher, 2026-10-28).

## Before every release

Run the release check and keep its output in the release's acceptance record:

```powershell
$env:GH_TOKEN = (gh auth token --user Chrizhermann); python -I tools/own_mod_pins.py check --release
```

CI runs the release check by itself on every pull request that changes the app version or
a recipe's `release.json` version. Each of our mods must be on its newest release in every
installer that ships it, or held with Christopher's reason. A push without a pull request
gets the status check only.

## Preparing a pin

`tools/prepare_own_mod_pin.py` does steps 1, 2 and the mechanical part of step 4 in one
working tree. It needs the engine's author CLI (`cargo build --locked -p chriz-bg-engine
--bin chriz-bg-author`, or `--author <path>`):

```powershell
$env:GH_TOKEN = (gh auth token --user Chrizhermann); python -I tools/prepare_own_mod_pin.py buffbot
```

- It pins the mod's newest release (or `--release <tag>`) in every installer whose
  recipes are in the working tree; `--installer <id>` narrows that. Both products are in
  the same checkout of `main`; ordinarily prepare their pins together.
- It downloads the release fresh, checks its length and SHA-256 against the digest GitHub
  publishes, writes the new artifact TOML from the old one (only the release fields
  change), switches the mods, regenerates the curated recipe, runs `artifact verify` and
  the native order check for each recipe, compares the TP2 components and VERSION, and
  writes the evidence record.
- Its summary lists the checks, the files it changed and what is left for the agent: the
  catalog, the changelog draft, the tests that freeze the old pin, an intake test, the
  plan digests, the static-test records, the pin list and the hold, each named by the
  files in that tree.
- It never commits, pushes or merges, and it never chooses a component. New or renamed
  components are listed for Christopher's decision.
- Exit code 3 means the release needs an agent, with the reason: for example an asset
  name it cannot carry over (each voicepack has its own version), a digest that does not
  match, or an order that no longer fits. It stops before its first edit, or undoes its
  edits, so the tree is as it was. Exit code 1 is an error, 2 a wrong argument.

## The daily pin job

Christopher chose on 2026-10-28: a GitHub Actions job looks for new releases of our mods
once a day, and on demand, and opens draft pull requests for his test. It never merges,
closes or approves anything.

- **Where it runs:** `.github/workflows/own-mod-pin-job.yml` on `main`, because GitHub
  runs scheduled workflows only from the default branch. It checks out `main` and runs
  `tools/own_mod_pin_job.py` there for both installers; no branch-routing variable is needed.
- **What it does:** for each of our mods whose newest release an installer lacks, it runs
  `tools/prepare_own_mod_pin.py` in a fresh worktree of `main`. It pushes
  `own-mod-pin/<installers>/<mod>-<release>`, opens one draft pull request against `main`
  with all affected installers, and starts its checks.
- **Every installer at once, or none:** each installer that is behind gets the release in
  the same run. If one of them needs an agent, the job opens the issue "Pin <mod> <release>
  by hand" with the reason, and no pull request at all.
- **What stops a release** (the job summary gives the reason):
  - a hold for that release;
  - a newest release numbered below the pin, which starts a new line of the mod and is
    Christopher's choice;
  - an installer that pins the release in only some of its recipes;
  - a pull request the job already opened for that release, open or closed; a closed one
    is not opened again;
  - an open pull request that already changes the mod's pin, until it is merged or
    closed. The job's own pull request for an older release does not stop it; the new
    one says which one it supersedes.
- **After it opens a pull request:** an agent completes "Left for the agent" on its branch
  (catalog, changelog draft, the tests that freeze the pin, the hold). Then Christopher
  tests and merges the shared pull request into `main`.
- **By hand:** Actions → Own mod pin job → Run workflow. `mod` limits it to one mod;
  `dry_run` prepares and reports, but pushes, opens and starts nothing. Locally,
  `python -I tools/own_mod_pin_job.py plan --next-release-branch main` only reads.

The job needs the repository setting "Allow GitHub Actions to create and approve pull
requests" (Settings → Actions → General). Only Christopher changes it.

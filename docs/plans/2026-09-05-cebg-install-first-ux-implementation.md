# Chriz Easy BG Install-First UX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers-extended-cc:executing-plans to implement this plan task-by-task.

**Goal:** Turn the current wizard-shaped private alpha into the approved Chriz Easy BG one-click installer and verified-install launcher.

**Architecture:** Keep all filesystem and launch authority in Rust. The frontend receives display DTOs, renders one primary install/launcher state, and submits only semantic selections plus server-revalidated paths and managed-install IDs. Preserve the existing campaign engine and registry internally while removing its terminology from player-facing copy.

**Tech Stack:** Tauri 2, Rust 2021, vanilla TypeScript, Vitest/testing-library, CSS, NSIS, Windows Shell Link API.

---

### Task 1: Freeze the player-facing installation name

**Files:**
- Modify: `engine/src/cli.rs`
- Modify: `engine/src/orchestrator.rs` only if the frozen request boundary needs the name
- Modify: `engine/tests/cli.rs`
- Modify: `app/src-tauri/src/bridge.rs`
- Modify: `app/src-tauri/src/commands.rs`
- Modify: `app/src-tauri/tests/command_contract.rs`
- Modify: `app/src/backend.ts`
- Modify: `app/src/contracts.ts`

**Step 1: Write failing identity tests**

Add tests proving that:

```rust
let first = review_request("Chriz Easy BG");
let renamed = review_request("My Baldur's Gate");
assert_ne!(review(&first), review(&renamed));
assert_eq!(completed_registry_record(&renamed).display_name, "My Baldur's Gate");
```

Also reject blank names, control characters, and Windows filename characters
`< > : " / \\ | ? *`. Permit ordinary Unicode and punctuation used by player names.

**Step 2: Run RED**

Run:

```powershell
cargo test -p chriz-bg-engine --test cli display_name
cargo test -p chriz-bg-app --test command_contract display_name
```

Expected: FAIL because the name is currently hardcoded as `Chriz BG Collection`.

**Step 3: Thread the validated name through the frozen request**

Add `display_name` to `InstallCommandRequest`, its serialized frozen identity, review digest,
save identity, and `ManagedReceiptWriter`. The CLI accepts `--name` with default
`Chriz Easy BG`; Tauri passes the UI name into `freeze_review`. Resume reads the frozen name
and cannot substitute another value.

Do not rename internal application-data or ledger directories in this task; those locations
are compatibility identifiers rather than visible branding.

**Step 4: Run GREEN and commit**

```powershell
cargo test -p chriz-bg-engine --test cli
cargo test -p chriz-bg-app --test command_contract
cargo clippy --workspace --all-targets -- -D warnings
git add engine app
git commit -m "engine: freeze player-facing installation names"
```

### Task 2: Provide safe defaults and human Windows paths

**Files:**
- Modify: `app/src-tauri/src/bridge.rs`
- Modify: `app/src-tauri/src/commands.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/tests/command_contract.rs`
- Modify: `app/src/backend.ts`
- Modify: `app/src/contracts.ts`
- Extend: `app/tests/backend-contract.test.ts`

**Step 1: Write failing projection tests**

Test a pure default constructor with an injected home directory:

```rust
let defaults = installation_defaults(Path::new(r"C:\Users\Chris"), "Chriz Easy BG")?;
assert_eq!(defaults.path, r"C:\Users\Chris\Games\Chriz Easy BG");
```

Test that candidate, destination, managed-install, receipt, and launcher display fields turn
`\\?\C:\Games\Example` into `C:\Games\Example`, without changing the canonical `PathBuf`
used for validation. Include UNC verbatim-prefix coverage.

**Step 2: Run RED**

```powershell
cargo test -p chriz-bg-app --test command_contract installation_defaults
cargo test -p chriz-bg-app --test command_contract display_path
```

Expected: FAIL because no defaults command exists and current projections expose verbatim
Windows prefixes.

**Step 3: Add the narrow native contract**

Add:

```rust
pub struct InstallationDefaultsResponse {
    pub name: String,
    pub path: String,
}
```

The Tauri command obtains the Windows home directory through Tauri's path resolver, joins
`Games/Chriz Easy BG`, and returns it without creating anything. Every mutating operation
still revalidates the submitted location in the engine. Add a display-only Windows path
formatter; never use its result as an authorization token.

**Step 4: Run GREEN and commit**

```powershell
cargo test -p chriz-bg-app --test command_contract
Push-Location app
npm test -- --run tests/backend-contract.test.ts
Pop-Location
git add app
git commit -m "app: suggest safe CEBG installation defaults"
```

### Task 3: Replace sidebar navigation with the install-first home

**Files:**
- Create: `app/src/screens/install.ts`
- Modify: `app/src/app.ts`
- Modify: `app/src/state.ts`
- Modify: `app/src/contracts.ts`
- Modify: `app/src/components/app-shell.ts`
- Modify: `app/src/screens/games.ts` or retire it from the normal path
- Modify: `app/src/screens/destination.ts` or retire it from the normal path
- Modify: `app/src/screens/setup.ts`
- Modify: `app/src/screens/review.ts`
- Modify: `app/src/screens/build.ts`
- Modify: `app/src/components/campaign-ledger.ts`
- Modify: `app/src/screens/updates.ts`
- Modify: `app/src/styles.css`
- Modify: `app/src-tauri/tauri.conf.json`
- Modify: `app/index.html`
- Extend: `app/tests/wizard.test.ts`
- Extend: `app/tests/state.test.ts`
- Extend: `app/tests/accessibility.test.ts`

**Step 1: Write failing simple-path tests**

Cover these visible behaviors:

```typescript
expect(getByText(root, "Chriz Easy BG")).toBeTruthy();
expect(getByText(root, "0.1 Alpha")).toBeTruthy();
expect(queryByText(root, "Campaigns")).toBeNull();
expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
expect(getByLabelText(root, "Install name")).toHaveValue("Chriz Easy BG");
expect(getByLabelText(root, "Install location")).toHaveValue(
  "C:\\Users\\Chris\\Games\\Chriz Easy BG",
);
expect(getByText(root, "Needs attention")).toBeTruthy();
expect(getByRole(root, "button", { name: "Install Chriz Easy BG" })).toBeDisabled();
```

With two eligible sources and a safe location, assert **Ready to install** and an enabled
Install button. Assert that one candidate renders as a labelled **Found source** rather than
a redundant select, and multiple candidates remain selectable. Assert **Details** contains
the full freshness findings only when expanded. Selection-evaluation findings are nonfatal
omission notices and must not disable Install. Assert **Customize** preserves choices and
returns to the primary screen. Assert a double-click cannot start two builds, and a late
destination-inspection result cannot replace the result for a newer source selection.

**Step 2: Run RED**

```powershell
Push-Location app
npm test -- --run tests/wizard.test.ts
Pop-Location
```

Expected: FAIL on the old sidebar, campaign vocabulary, oversized wizard entry, and missing
readiness summary.

**Step 3: Implement the minimum install-first controller**

The primary screen receives the selected sources, destination evaluation, installation
defaults, evaluation, and actions. Its readiness predicate is:

```typescript
const ready = !starting
  && validName
  && bg1?.eligible === true
  && bg2?.eligible === true
  && destination.safe
  && evaluation !== null
  && !evaluationPending;
```

Do not gate readiness on `evaluation.findings.length`: the engine contract defines these as
nonfatal omissions and warnings, including a normal default that becomes unavailable. Reinspect
the location whenever either source changes, protect all asynchronous inspections with revisions,
and guard the freeze/start operation against duplicate activation.

The primary Install action performs the existing native freeze/start sequence directly.
It does not make the user visit a review ledger first. **Customize** is secondary and returns
to the primary install screen. Keep technical review data accessible from build diagnostics,
not in the novice path.

Use these exact visible terms: **Install**, **Installation**, **Install location**,
**My installs**, **Ready to install**, **Needs attention**, and **Play**. Do not expose
`campaign`, `managed copy`, `destination`, or `output` in player-facing copy.

**Step 4: Compact the visual hierarchy**

Replace the sidebar with a quiet header. Use moderate serif headings, Segoe body text,
compact source/status rows, and one visually dominant Install button. Set the default Tauri
window near `1160 x 760`, respecting its current minimums. At desktop widths use two source
columns; at constrained widths use one. Detailed findings use native `<details>` controls.

**Step 5: Run GREEN and commit**

```powershell
Push-Location app
npm run check
Pop-Location
git add app
git commit -m "app: make CEBG installation the primary experience"
```

### Task 4: Open completed installs as a launcher

**Files:**
- Modify: `app/src/app.ts`
- Modify: `app/src/screens/home.ts`
- Modify: `app/src/screens/complete.ts`
- Modify: `app/src/screens/updates.ts`
- Modify: `app/src/contracts.ts`
- Modify: `app/src/backend.ts`
- Modify: `app/src-tauri/src/bridge.rs`
- Modify: `app/src/styles.css`
- Extend: `app/tests/wizard.test.ts`
- Extend: `app/src-tauri/tests/command_contract.rs`

**Step 1: Write failing startup-state tests**

Test the five approved states: no install, resumable install, one ready install, multiple
installs with a remembered ID, and stale-only records. Load the registry before game discovery or
evaluation, so a returning player can still Play when the original source games are unavailable.
A ready install must open on:

```typescript
expect(getByText(root, "Ready to play")).toBeTruthy();
expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
expect(getByRole(root, "button", { name: "Open game folder" })).toBeTruthy();
```

Test that interrupted work gets **Continue installation**, missing records get
**Installation not found**, and update messaging never blocks Play unless launch validation
itself fails. Multiple installs default to the locally remembered managed-install ID only
when that ID still exists; otherwise select the newest available record using native completion
ordering rather than frontend string or ID guesses.

**Step 2: Run RED**

```powershell
Push-Location app
npm test -- --run tests/wizard.test.ts
Pop-Location
```

**Step 3: Implement launcher routing and completion reuse**

Use the registry list returned by Rust. Store only the last selected install ID in local UI
preferences; it grants no filesystem authority. `launch_install` and `open_install_folder`
continue resolving and verifying the registry record in Rust. Reuse the launcher view after
the `campaign_finished` engine event instead of maintaining a separate completion vocabulary.
Refresh the native registry after that event before routing, so the just-completed installation
is the launcher's verified source of truth. Extend the narrow managed-install projection with the
display-only launch path and completion ordering value needed for details and fallback selection.

Show exact launcher and receipt paths only under **Installation details**.

**Step 4: Run GREEN and commit**

```powershell
Push-Location app
npm run check
Pop-Location
git add app
git commit -m "app: reopen completed CEBG installs as a launcher"
```

### Task 5: Create a verified CEBG desktop shortcut

**Files:**
- Create: `app/src-tauri/src/shortcut.rs`
- Modify: `app/src-tauri/src/bridge.rs`
- Modify: `app/src-tauri/src/commands.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src/backend.ts`
- Modify: `app/src/contracts.ts`
- Modify: `app/src/app.ts`
- Modify: `app/src/screens/home.ts`
- Modify: `app/src/screens/install.ts`
- Modify: `app/src/styles.css`
- Extend: `app/src-tauri/tests/command_contract.rs`
- Extend: `app/tests/backend-contract.test.ts`
- Extend: `app/tests/wizard.test.ts`

**Step 1: Write failing shortcut-boundary tests**

Use an injected shortcut writer and prove:

- only an available verified managed-install ID is accepted;
- incomplete, stale, missing, or arbitrary paths are rejected;
- the shortcut name comes from the validated display name;
- the target is the current CEBG application executable, never a frontend-supplied path;
- the UI defaults **Create desktop shortcut** to checked after successful completion;
- repeated creation replaces only the exact CEBG-owned shortcut for that installation.

**Step 2: Run RED**

```powershell
cargo test -p chriz-bg-app --test command_contract desktop_shortcut
Push-Location app
npm test -- --run tests/wizard.test.ts
Pop-Location
```

**Step 3: Implement the Windows shortcut writer**

Use the pinned Windows Shell Link API directly from Rust. Resolve the Desktop known folder
natively and write a `.lnk` to the running CEBG executable. Do not spawn PowerShell, accept a
target path from JavaScript, or point the default shortcut directly at `InfinityLoader.exe`.
The app launcher will show update status and then invoke the registry-verified Play command.

The command accepts only `install_id`. It reopens that record through the existing registry
validation before writing the shortcut. Return the created display path for confirmation.

**Step 4: Run GREEN and commit**

```powershell
cargo test -p chriz-bg-app --test command_contract
cargo clippy -p chriz-bg-app --all-targets -- -D warnings
Push-Location app
npm run check
Pop-Location
git add Cargo.lock app
git commit -m "app: create verified CEBG launcher shortcuts"
```

### Task 6: Verify the desktop UX and package

**Files:**
- Modify only if a test exposes a defect: `app/src/styles.css`
- Modify: `docs/handover.md`

**Step 1: Run automated verification**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
Push-Location app
npm run check
Pop-Location
cargo run -p chriz-bg-engine --bin chriz-bg-install -- validate manifest --profile public-alpha
git diff --check
```

**Step 2: Run targeted responsive verification**

Inspect the native/fixture screen at 1920x1080 and 1366x768 and confirm no primary-screen
page scrollbar. Inspect 768x1024 and 375x812 for one-column layout, scrolling, visible focus,
and reachable actions. Exercise source Details, Customize/return, My installs, Updates,
Continue installation, Play, Open game folder, and shortcut confirmation. Do not launch or
mutate any protected/reference game directory.

**Step 3: Build and smoke the NSIS package**

```powershell
Push-Location app
npm run tauri -- build
Pop-Location
```

Silently install into a fresh directory beneath this worktree's `target`, verify packaged
recipe files, launch CEBG for five seconds, close the exact process, silently uninstall, and
verify the smoke directory is removed. Record installer size and SHA-256.

**Step 4: Update continuity and commit**

Record the new product name, UX behavior, verification evidence, installer hash, and remaining
fresh-BGEE/full-EET blocker in `docs/handover.md`. Do not mark the public alpha released.

```powershell
git add docs/handover.md docs/plans/2026-09-05-cebg-install-first-ux-implementation.md.tasks.json
git commit -m "docs: record CEBG launcher checkpoint"
```

// @vitest-environment jsdom

import { fireEvent, getAllByRole, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend, NativeBackend } from "../src/backend";
import { installedUpdateSection, type InstalledUpdateActions } from "../src/components/installed-update-plan";
import type { InstalledUpdatePlan, ManagedCopyUpdate, UpdateSummary } from "../src/contracts";
import { updatesScreen } from "../src/screens/updates";

const copies: ManagedCopyUpdate[] = [
  { installId: "first", name: "My game", path: "D:\\First", installedRecipeVersion: "1", state: "update-available", detail: "" },
  { installId: "second", name: "My game", path: "E:\\Second", installedRecipeVersion: "1", state: "update-available", detail: "" },
];

function plan(overrides: Partial<InstalledUpdatePlan> = {}): InstalledUpdatePlan {
  return {
    installId: "first", baseRecipeVersion: "1", targetRecipeVersion: "2", state: "comparison-ready",
    detail: "Compared this installation with the collection included in CEBG.", choicesPreserved: true, issues: [],
    runs: [
      { runId: "buffbot-bg2", modName: "BuffBot", installedVersion: "1.8.3", targetVersion: "1.8.4", components: [1, 0], state: "unassessed", detail: "The installed version differs. Upgrade support has not been verified." },
      { runId: "unchanged-bg2", modName: "Unchanged mod", installedVersion: "3", targetVersion: "3", components: [0], state: "unchanged", detail: "Matches the target recipe." },
    ],
    ...overrides,
  };
}

function actions(overrides: Partial<InstalledUpdateActions> = {}): InstalledUpdateActions {
  return { onCheckInstalledUpdate: vi.fn(), onSelectInstalledUpdate: vi.fn(), ...overrides };
}

function summary(): UpdateSummary {
  return {
    checkedAt: null, networkState: "online",
    application: { state: "available", currentVersion: "1", availableVersion: "2", detail: "" },
    recipe: { state: "up-to-date", currentVersion: "2", availableVersion: null, disposition: "up-to-date", detail: "", changes: [] },
    managedCopies: copies,
  };
}

afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); });

describe("installed-mod comparison", () => {
  it("checks only the selected registry ID after an explicit click", () => {
    const callbacks = actions();
    const section = installedUpdateSection(copies, callbacks);
    expect(callbacks.onCheckInstalledUpdate).not.toHaveBeenCalled();
    const options = getAllByRole(section, "option");
    expect(options[0].textContent).toContain("D:\\First");
    expect(options[1].textContent).toContain("E:\\Second");
    fireEvent.change(getByRole(section, "combobox"), { target: { value: "second" } });
    expect(callbacks.onSelectInstalledUpdate).toHaveBeenCalledWith("second");
    expect(callbacks.onCheckInstalledUpdate).not.toHaveBeenCalled();
    fireEvent.click(getByRole(section, "button", { name: "Check installed mods" }));
    expect(callbacks.onCheckInstalledUpdate).toHaveBeenCalledWith("second");
  });

  it("folds the comparison while retaining changed versions, unchanged entries and every detail", () => {
    const section = installedUpdateSection(copies, actions({ installedUpdatePlan: plan() }));
    const comparison = section.querySelector<HTMLDetailsElement>(".installed-mod-comparison");
    expect(comparison?.open).toBe(false);
    expect(comparison?.querySelector("summary")?.textContent).toBe("Mod comparison details");
    expect(section.querySelector(".installed-update-result")?.closest("details")).toBe(comparison);
    expect(section.textContent).toContain("Comparison only — no game files changed");
    expect(section.textContent).toContain("Base collection: 1 · Compared with: 2");
    expect(section.textContent).toContain("Your existing choices are preserved in this comparison");
    expect(section.textContent).toContain("1.8.3 → 1.8.4");
    expect(section.textContent).toContain("Update support not assessed");
    const unchanged = section.querySelector<HTMLDetailsElement>(".installed-update-unchanged");
    expect(unchanged?.open).toBe(false);
    expect(unchanged?.textContent).toContain("Unchanged mod");
    expect(section.querySelectorAll(".installed-update-changes > article")).toHaveLength(1);
    const details = section.querySelector<HTMLDetailsElement>(".installed-update-changes details");
    expect(details?.open).toBe(false);
    expect(details?.textContent).toContain("Components: 1, 0");
    expect(queryByRole(section, "button", { name: /apply|update this installation/i })).toBeNull();
  });

  it("puts the supported BuffBot action before the collapsed comparison without changing either result", () => {
    const section = installedUpdateSection(copies, actions({
      installedUpdatePlan: plan({ state: "blocked", choicesPreserved: false, issues: [
        { rule: "new-required-option", featureId: "new-option", message: "Review a new collection requirement." },
      ] }),
      modUpgrade: {
        installId: "first", upgradeId: "buffbot-upgrade", title: "BuffBot", state: "available", detail: "Only BuffBot changes.",
        installedVersion: "1.8.3-alpha", targetVersion: "1.8.4-alpha", baseRecipeVersion: "1", reviewToken: "review",
        canUndo: false, canRestore: false, fullBackupBytes: "10", saveBackupBytes: "2", availableBytes: "100",
        backupPath: "D:\\Backups", saveCompatibility: "Existing saves remain compatible.", history: [],
      },
      onApplyModUpgrade: vi.fn(),
    }));
    const comparison = section.querySelector<HTMLDetailsElement>(".installed-mod-comparison")!;
    const update = section.querySelector(".mod-upgrade-result")!;
    expect(comparison.open).toBe(false);
    expect(update.compareDocumentPosition(comparison) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    const review = getByRole(section, "button", { name: "Review BuffBot update" }) as HTMLButtonElement;
    expect(review.closest("details")).toBeNull();
    expect(review.disabled).toBe(false);
    expect(comparison.textContent).toContain("Comparison needs attention");
    expect(comparison.textContent).toContain("Review a new collection requirement.");
    expect(update.textContent).not.toContain("Review a new collection requirement.");
  });

  it("explains migration and choice issues without claiming an update is available", () => {
    const section = installedUpdateSection(copies, actions({ installedUpdatePlan: plan({
      state: "blocked", choicesPreserved: false,
      issues: [{ rule: "new_mandatory_feature", featureId: "mandatory-new", message: "The new required feature needs an explicit choice before this setup can be preserved." }],
      runs: [{ ...plan().runs[0], state: "requires-migration", targetVersion: null }],
    }) }));
    expect(section.textContent).toContain("Comparison needs attention");
    expect(section.textContent).toContain("Your choices need review");
    expect(section.textContent).toContain("new required feature needs an explicit choice");
    expect(section.textContent).toContain("Target version unavailable");
    expect(section.textContent).not.toContain("Update available");
    expect(queryByRole(section, "button", { name: /apply|update this installation/i })).toBeNull();
  });

  it("does not call a blocked or empty comparison up to date", () => {
    const blocked = installedUpdateSection(copies, actions({ installedUpdatePlan: plan({ state: "blocked", choicesPreserved: false, runs: [] }) }));
    expect(blocked.textContent).not.toContain("No installed-mod differences");
    const current = installedUpdateSection(copies, actions({ installedUpdatePlan: plan({ runs: [plan().runs[1]] }) }));
    expect(current.textContent).toContain("No installed-mod differences were found");
  });

  it("folds a long choice-issue list while retaining every explanation", () => {
    const issues = Array.from({ length: 8 }, (_, index) => ({ rule: "choice", featureId: `choice-${index}`, message: `Choice ${index} needs review.` }));
    const section = installedUpdateSection(copies, actions({ installedUpdatePlan: plan({ state: "blocked", choicesPreserved: false, issues }) }));
    const more = Array.from(section.querySelectorAll("details")).find(details => details.querySelector("summary")?.textContent === "More choices to review (5)");
    expect(more?.open).toBe(false);
    expect(more?.querySelectorAll("li")).toHaveLength(5);
    for (const issue of issues) expect(section.textContent).toContain(issue.message);
  });

  it("does not attach a missing installation's error to its fallback selection", () => {
    const section = installedUpdateSection(copies, actions({ installedUpdateInstallId: "removed", installedUpdateError: "Removed-only error" }));
    expect(section.textContent).not.toContain("Removed-only error");
  });

  it("hides a previous installation's result and error after changing selection", () => {
    const section = installedUpdateSection(copies, actions({ installedUpdatePlan: plan(), installedUpdateError: "First-only error", installedUpdateErrorDetail: "First-only diagnostic", installedUpdateInstallId: "first" }));
    fireEvent.change(getByRole(section, "combobox"), { target: { value: "second" } });
    expect(section.textContent).not.toContain("BuffBot");
    expect(section.textContent).not.toContain("First-only error");
    expect(section.textContent).not.toContain("First-only diagnostic");
    expect(section.textContent).toContain("Not compared yet");
  });

  it("keeps selection available during a comparison, and blocks checks for busy or missing games", () => {
    const busy = installedUpdateSection(copies, actions({ installedUpdateBusy: true }));
    expect((getByRole(busy, "button", { name: "Checking installed mods…" }) as HTMLButtonElement).disabled).toBe(true);
    expect((getByRole(busy, "combobox") as HTMLSelectElement).disabled).toBe(false);
    expect(busy.getAttribute("aria-busy")).toBe("true");
    const stale = installedUpdateSection([{ ...copies[0], state: "stale" }], actions());
    expect((getByRole(stale, "button", { name: "Check installed mods" }) as HTMLButtonElement).disabled).toBe(true);
    expect(stale.textContent).toContain("folder is unavailable");
    const empty = installedUpdateSection([], actions());
    expect(empty.textContent).toContain("Complete an installation first");
  });

  it("renders backend and installation text as text, not HTML", () => {
    const marker = '<img src=x onerror="alert(1)">';
    const section = installedUpdateSection([{ ...copies[0], name: marker }], actions({ installedUpdatePlan: plan({
      detail: marker, issues: [{ rule: marker, featureId: marker, message: marker }],
      runs: [{ ...plan().runs[0], modName: marker, detail: marker }],
    }), installedUpdateError: marker, installedUpdateErrorDetail: marker }));
    expect(section.querySelector("img")).toBeNull();
    expect(section.textContent).toContain(marker);
  });

  it("keeps app updates, game fixes and creating a separate installation visible", () => {
    const page = updatesScreen(summary(), {
      ...actions({ installedUpdateBusy: true }), onCheckPatches: vi.fn(),
      checkAgain: vi.fn(), installApplication: vi.fn(), buildUpdatedCopy: vi.fn(),
    });
    expect(getByRole(page, "button", { name: "Update CEBG app" })).toBeTruthy();
    expect(getByRole(page, "button", { name: "Create updated installation" })).toBeTruthy();
    expect(getByRole(page, "heading", { name: "Fixes for your current game" })).toBeTruthy();
    expect((getByRole(page, "button", { name: "Check this installation" }) as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("installed comparison app state and transport", () => {
  async function mounted() {
    const backend = new FixtureBackend();
    vi.spyOn(backend, "getUpdates").mockResolvedValue(summary());
    const root = document.createElement("div");
    document.body.append(root);
    const inspect = vi.spyOn(backend, "inspectInstalledUpdate");
    const app = await mountApp(root, backend);
    await app.navigate("updates");
    return { root, backend, inspect };
  }

  it("does not inspect on startup or navigation, and ignores an old result after selection changes", async () => {
    const { root, inspect } = await mounted();
    expect(inspect).not.toHaveBeenCalled();
    let finishFirst!: (value: InstalledUpdatePlan) => void;
    inspect.mockImplementationOnce(() => new Promise(resolve => { finishFirst = resolve; }));
    inspect.mockResolvedValueOnce(plan({ installId: "second", runs: [{ ...plan().runs[0], modName: "Second mod" }] }));
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect(root.textContent).toContain("Second mod"));
    finishFirst(plan());
    await new Promise(resolve => setTimeout(resolve, 0));
    expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).value).toBe("second");
    expect(root.textContent).toContain("Second mod");
    expect(root.querySelector(".installed-update-result")?.textContent).not.toContain("BuffBot");
    expect(inspect.mock.calls).toEqual([["first"], ["second"]]);
  });

  it("ignores an obsolete error and retains the active comparison's busy state", async () => {
    const { root, inspect } = await mounted();
    let rejectFirst!: (error: Error) => void;
    let finishSecond!: (value: InstalledUpdatePlan) => void;
    inspect.mockImplementationOnce(() => new Promise((_resolve, reject) => { rejectFirst = reject; }));
    inspect.mockImplementationOnce(() => new Promise(resolve => { finishSecond = resolve; }));
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    rejectFirst(new Error("Obsolete first failure"));
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(root.textContent).not.toContain("Obsolete first failure");
    expect((getByRole(root, "button", { name: "Checking installed mods…" }) as HTMLButtonElement).disabled).toBe(true);
    finishSecond(plan({ installId: "second" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Check installed mods" })).toBeTruthy());
  });

  it("shows actionable errors and allows a fresh check", async () => {
    const { root, inspect } = await mounted();
    inspect.mockRejectedValueOnce(new BackendCommandError({ code: "unsupported", message: "The install record could not be verified.", recovery_action: "Export diagnostics for this installation.", technical_detail: "Long internal details" }));
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain("Export diagnostics"));
    expect(getByRole(root, "alert").textContent).not.toContain("Long internal details");
    const diagnostic = Array.from(root.querySelectorAll("details")).find(details => details.querySelector("summary")?.textContent === "Comparison error details");
    expect(diagnostic?.open).toBe(false);
    expect(diagnostic?.textContent).toContain("Long internal details");
    inspect.mockResolvedValueOnce(plan());
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect(root.textContent).toContain("BuffBot"));
    expect(queryByRole(root, "alert")).toBeNull();
    expect(root.textContent).not.toContain("Long internal details");
  });

  it("rejects a response belonging to a different installation", async () => {
    const { root, inspect } = await mounted();
    inspect.mockResolvedValueOnce(plan({ installId: "second" }));
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain("different installation"));
    expect(root.querySelector(".installed-update-result")).toBeNull();
  });

  it("passes only the registry identity through the native read-only command", async () => {
    const invoke = vi.fn(async () => plan());
    const backend = new NativeBackend(invoke);
    await expect(backend.inspectInstalledUpdate("first")).resolves.toEqual(plan());
    expect(invoke).toHaveBeenCalledExactlyOnceWith("inspect_installed_update", { installId: "first" });
  });
});

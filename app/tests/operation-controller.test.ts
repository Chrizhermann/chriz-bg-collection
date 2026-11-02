// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { ModUpgradePreview, OperationProgress, UpdateSummary } from "../src/contracts";

const pendingKey = "cebg.pending-app-update.v1";
const oldApp = "0.1.0-alpha.18", newApp = "0.1.0-alpha.19";
function preview(overrides: Partial<ModUpgradePreview> = {}): ModUpgradePreview {
  return { installId: "chosen", upgradeId: "buffbot-upgrade", title: "BuffBot", state: "available",
    detail: "Only BuffBot will change.", baseRecipeVersion: "1", installedVersion: "1.8.3-alpha", targetVersion: "1.8.4-alpha",
    reviewToken: "review", canUndo: false, canRestore: false, fullBackupBytes: "100", saveBackupBytes: "5", availableBytes: "1000",
    backupPath: "D:\\Backups", saveCompatibility: "Existing saves are kept.", history: [], ...overrides };
}
function summary(version = oldApp): UpdateSummary {
  return { checkedAt: null, networkState: "online",
    application: { state: version === newApp ? "up-to-date" : "available", currentVersion: version, availableVersion: version === newApp ? null : newApp, detail: "" },
    recipe: { state: "up-to-date", currentVersion: "1", availableVersion: null, disposition: "up-to-date", detail: "", changes: [] },
    managedCopies: [{ installId: "chosen", name: "My test game", path: "D:\\Test", installedRecipeVersion: "1", state: "update-available", detail: "" }] };
}
function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
function feedback(root: HTMLElement): HTMLElement { return root.querySelector<HTMLElement>(".operation-feedback")!; }

async function mounted(version = oldApp) {
  const backend = new FixtureBackend({ managedInstallations: [{ id: "chosen", name: "My test game", path: "D:\\Test", status: "Ready to play", receiptPath: null, launchPath: null, completedAtMillis: null, available: true, resumable: false }] });
  vi.spyOn(backend, "getStatus").mockResolvedValue({ mode: "fixture", applicationVersion: version, engineVersion: "0.1.0", recipeVersion: "1", startupInstallId: "chosen" });
  vi.spyOn(backend, "getUpdates").mockResolvedValue(summary(version));
  vi.spyOn(backend, "inspectInstalledUpdate").mockResolvedValue({ installId: "chosen", baseRecipeVersion: "1", targetRecipeVersion: "1", state: "comparison-ready", detail: "Compared", choicesPreserved: true, issues: [], runs: [] });
  const inspect = vi.spyOn(backend, "inspectModUpgrade").mockResolvedValue(preview());
  const apply = vi.spyOn(backend, "applyModUpgrade");
  const restore = vi.spyOn(backend, "restoreModUpgrade");
  const root = document.createElement("div"); document.body.append(root);
  const app = await mountApp(root, backend);
  await app.navigate("updates");
  return { root, backend, inspect, apply, restore };
}
async function startApply(root: HTMLElement) {
  fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
  await waitFor(() => expect(getByRole(root, "button", { name: "Review BuffBot update" })).toBeTruthy());
  fireEvent.click(getByRole(root, "button", { name: "Review BuffBot update" }));
  fireEvent.click(getByRole(root, "button", { name: "Update BuffBot now" }));
}

afterEach(() => { document.body.replaceChildren(); localStorage.clear(); vi.restoreAllMocks(); });

describe("operation feedback controller", () => {
  it("keeps byte completion working until Apply returns, then ignores late progress", async () => {
    const { root, apply, inspect } = await mounted();
    const pending = deferred<ModUpgradePreview>();
    let report: ((progress: OperationProgress) => void) | undefined;
    apply.mockImplementation(async (_id, _token, _full, _saves, onProgress?: (progress: OperationProgress) => void) => {
      report = onProgress;
      return pending.promise;
    });
    await startApply(root);
    await waitFor(() => expect(apply).toHaveBeenCalledOnce());
    expect(feedback(root).classList.contains("operation-working")).toBe(true);
    expect(feedback(root).textContent).toContain("Updating BuffBot");
    report!({ stage: "backup-game", completed: "25", total: "100", unit: "bytes" });
    expect(getByRole(feedback(root), "progressbar").getAttribute("value")).toBe("25");
    report!({ stage: "backup-game", completed: "100", total: "100", unit: "bytes" });
    expect(feedback(root).classList.contains("operation-success")).toBe(false);
    expect(queryByRole(root, "button", { name: "Dismiss" })).toBeNull();
    const applied = preview({ state: "applied", installedVersion: "1.8.4-alpha", reviewToken: null, canUndo: true });
    inspect.mockResolvedValue(applied);
    pending.resolve(applied);
    await waitFor(() => expect(feedback(root).classList.contains("operation-success")).toBe(true));
    expect(feedback(root).textContent).toContain("BuffBot update complete");
    expect(feedback(root).textContent).toContain("BuffBot 1.8.4-alpha is installed");
    report!({ stage: "backup-game", completed: "1", total: "100", unit: "bytes" });
    expect(feedback(root).classList.contains("operation-success")).toBe(true);
    expect(queryByRole(feedback(root), "progressbar")).toBeNull();
    fireEvent.click(getByRole(feedback(root), "button", { name: "Dismiss" }));
    expect(feedback(root)).toBeNull();
  });

  it.each(["throws", "needs-recovery"] as const)("never shows success when Apply %s", async outcome => {
    const { root, apply, inspect } = await mounted();
    const recovery = preview({ state: "needs-recovery", reviewToken: null, canRestore: true, detail: "Restore before playing." });
    apply.mockImplementation(async () => {
      inspect.mockResolvedValue(recovery);
      if (outcome === "throws") throw new BackendCommandError({ code: "interrupted", message: "Update interrupted.", recovery_action: "Restore its backup.", technical_detail: "fixture" });
      return recovery;
    });
    await startApply(root);
    await waitFor(() => expect(feedback(root).classList.contains("operation-error")).toBe(true));
    expect(feedback(root).textContent).not.toContain("update complete");
    expect(feedback(root).textContent).toContain("Keep the game closed");
    expect(getByRole(root, "button", { name: "Restore BuffBot backup" })).toBeTruthy();
    expect(queryByRole(root, "button", { name: "Review BuffBot update" })).toBeNull();
  });

  it("names the restored old version after Undo without claiming saved progress was restored", async () => {
    const { root, inspect, restore } = await mounted();
    inspect.mockResolvedValue(preview({ state: "applied", installedVersion: "1.8.4-alpha", reviewToken: null, canUndo: true }));
    restore.mockImplementation(async () => {
      const restored = preview({ history: [{ transactionId: "one", fromVersion: "1.8.3-alpha", toVersion: "1.8.4-alpha", state: "restored" }] });
      inspect.mockResolvedValue(restored);
      return restored;
    });
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Undo BuffBot update" })).toBeTruthy());
    fireEvent.click(getByRole(root, "button", { name: "Undo BuffBot update" }));
    fireEvent.click(getByRole(root, "button", { name: "Confirm undo" }));
    await waitFor(() => expect(feedback(root).classList.contains("operation-success")).toBe(true));
    expect(feedback(root).textContent).toContain("BuffBot update undone");
    expect(feedback(root).textContent).toContain("BuffBot 1.8.3-alpha is installed");
    expect(feedback(root).textContent).toContain("saves are unchanged");
    expect(feedback(root).textContent).not.toContain("1.8.4-alpha");
    expect(restore).toHaveBeenCalledExactlyOnceWith("chosen", true, expect.any(Function));
  });

  it("shows app success only after restarting into the requested version", async () => {
    const { root, backend } = await mounted();
    const install = vi.spyOn(backend, "installAppUpdate").mockResolvedValue();
    fireEvent.click(getByRole(root, "button", { name: "Update CEBG app" }));
    await waitFor(() => expect(feedback(root)?.textContent).toContain("Restart CEBG to finish"));
    expect(feedback(root).textContent).not.toContain("CEBG update complete");
    expect(JSON.parse(localStorage.getItem(pendingKey)!)).toMatchObject({ from: oldApp, to: newApp });
    expect(install).toHaveBeenCalledExactlyOnceWith(newApp, expect.any(Function));
    root.remove();
    const stillOld = await mounted(oldApp);
    expect(feedback(stillOld.root)).toBeNull();
    stillOld.root.remove();
    const restarted = await mounted(newApp);
    expect(feedback(restarted.root).textContent).toContain("CEBG update complete");
    expect(feedback(restarted.root).textContent).toContain(newApp);
    fireEvent.click(getByRole(feedback(restarted.root), "button", { name: "Dismiss" }));
    expect(localStorage.getItem(pendingKey)).toBeNull();
  });

  it("clears pending app success after an updater failure", async () => {
    const { root, backend } = await mounted();
    vi.spyOn(backend, "installAppUpdate").mockRejectedValue(new Error("Download could not finish"));
    fireEvent.click(getByRole(root, "button", { name: "Update CEBG app" }));
    await waitFor(() => expect(feedback(root)?.textContent).toContain("CEBG update could not finish"));
    expect(feedback(root).classList.contains("operation-error")).toBe(true);
    expect(localStorage.getItem(pendingKey)).toBeNull();
  });
});

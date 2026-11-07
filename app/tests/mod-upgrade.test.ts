// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend, NativeBackend } from "../src/backend";
import { installedUpdateSection } from "../src/components/installed-update-plan";
import { modUpgradeResult, type ModUpgradeActions } from "../src/components/mod-upgrade";
import type { InstalledUpdatePlan, ManagedCopyUpdate, ModUpgradePreview, UpdateSummary } from "../src/contracts";
import { updatesScreen } from "../src/screens/updates";

const copies: ManagedCopyUpdate[] = [
  { installId: "first", name: "My game", path: "D:\\First", installedRecipeVersion: "1", state: "update-available", detail: "" },
  { installId: "second", name: "My other game", path: "E:\\Second", installedRecipeVersion: "1", state: "update-available", detail: "" },
];
function preview(overrides: Partial<ModUpgradePreview> = {}): ModUpgradePreview {
  return { installId: "first", upgradeId: "buffbot-upgrade", title: "BuffBot", state: "retired", detail: "The legacy update is retired.",
    baseRecipeVersion: "1", installedVersion: "1.8.3-alpha", targetVersion: "1.8.4-alpha", reviewToken: null, canUndo: false, canRestore: false,
    fullBackupBytes: "1024", saveBackupBytes: "128", availableBytes: "8192", backupPath: "D:\\Backups",
    saveCompatibility: "Existing references and settings are retained.", history: [], ...overrides };
}
function callbacks(overrides: Partial<ModUpgradeActions> = {}): ModUpgradeActions {
  return { modUpgrade: preview(), onUndoModUpgrade: vi.fn(), onRestoreModUpgrade: vi.fn(), ...overrides };
}
function summary(): UpdateSummary {
  return { checkedAt: null, networkState: "online", application: { state: "available", currentVersion: "1", availableVersion: "2", detail: "" },
    recipe: { state: "up-to-date", currentVersion: "2", availableVersion: null, disposition: "up-to-date", detail: "", changes: [] }, managedCopies: copies };
}
function plan(installId = "first"): InstalledUpdatePlan {
  return { installId, baseRecipeVersion: "1", targetRecipeVersion: "2", state: "comparison-ready", detail: "Compared", choicesPreserved: true, issues: [], runs: [] };
}
function button(root: HTMLElement, name: string): HTMLButtonElement { return getByRole(root, "button", { name }) as HTMLButtonElement; }
afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); });

describe("historical BuffBot update recovery UI", () => {

  it("keeps recovery, blocked-update and unavailable-Undo reasons outside disclosures", () => {
    for (const state of ["needs-recovery", "unsupported", "applied"] as const) {
      const detail = `Important ${state} reason; export diagnostics before continuing.`;
      const section = modUpgradeResult(copies[0], callbacks({ modUpgrade: preview({ state, detail, canUndo: false, canRestore: false }) }));
      const visibleReason = Array.from(section.children).find(child => child.textContent === detail);
      expect(visibleReason?.tagName).toBe("P");
      expect(visibleReason?.closest("details")).toBeNull();
      expect(queryByRole(section, "button", { name: "Review BuffBot update" })).toBeNull();
      if (state !== "applied") expect(section.textContent).not.toContain("Works with your current saves");
    }
  });

  it("separates Undo and interrupted Restore and requires confirmation for each", () => {
    const undoActions = callbacks({ modUpgrade: preview({ state: "applied", canUndo: true, reviewToken: null }) });
    const undo = modUpgradeResult(copies[0], undoActions);
    document.body.append(undo);
    fireEvent.click(button(undo, "Undo BuffBot update"));
    expect(undoActions.onUndoModUpgrade).not.toHaveBeenCalled();
    expect(undo.textContent).toContain("Saved games and progress made since updating are not restored");
    fireEvent.click(button(undo, "Confirm undo"));
    expect(undoActions.onUndoModUpgrade).toHaveBeenCalledOnce();
    expect(queryByRole(undo, "button", { name: "Restore BuffBot backup" })).toBeNull();
    const restoreActions = callbacks({ modUpgrade: preview({ state: "needs-recovery", canRestore: true, reviewToken: null }) });
    const restore = modUpgradeResult(copies[0], restoreActions);
    document.body.append(restore);
    fireEvent.click(button(restore, "Restore BuffBot backup"));
    expect(restoreActions.onRestoreModUpgrade).not.toHaveBeenCalled();
    fireEvent.click(button(restore, "Restore backup now"));
    expect(restoreActions.onRestoreModUpgrade).toHaveBeenCalledOnce();
    expect(queryByRole(restore, "button", { name: "Undo BuffBot update" })).toBeNull();
  });

  it("keeps history and error details folded and renders all backend strings as plain text", () => {
    const marker = '<img src=x onerror="alert(1)">';
    const section = modUpgradeResult(copies[0], callbacks({ modUpgrade: preview({ detail: marker,
      history: [{ transactionId: marker, fromVersion: marker, toVersion: "new", state: "restored" }] }), modUpgradeError: "Unable to update", modUpgradeErrorDetail: marker }));
    expect(section.querySelector("img")).toBeNull();
    expect(section.textContent).toContain(marker);
    expect(section.textContent).toContain(`Restored BuffBot ${marker}`);
    expect(Array.from(section.querySelectorAll("details")).every(details => !details.open)).toBe(true);
  });

  it("describes every update-history state without treating unfinished or unknown work as success", () => {
    const section = modUpgradeResult(copies[0], callbacks({ modUpgrade: preview({ history: [
      { transactionId: "restored", fromVersion: "1.8.3-alpha", toVersion: "1.8.4-alpha", state: "restored" },
      { transactionId: "applied", fromVersion: "1.8.3-alpha", toVersion: "1.8.4-alpha", state: "applied" },
      { transactionId: "prepared", fromVersion: "1.8.3-alpha", toVersion: "1.8.4-alpha", state: "prepared" },
      { transactionId: "restoring", fromVersion: "1.8.3-alpha", toVersion: "1.8.4-alpha", state: "restoring" },
      { transactionId: "unknown", fromVersion: "1.8.3-alpha", toVersion: "1.8.4-alpha", state: "unrecognized" },
    ] }) }));
    expect(section.textContent).toContain("Restored BuffBot 1.8.3-alpha");
    expect(section.textContent).toContain("Updated BuffBot to 1.8.4-alpha (from 1.8.3-alpha)");
    expect(section.textContent).toContain("BuffBot update to 1.8.4-alpha was not finished");
    expect(section.textContent).toContain("Restoration of BuffBot 1.8.3-alpha was not finished");
    expect(section.textContent).toContain("BuffBot update needs attention (status: unrecognized)");
    expect(section.textContent).not.toContain("→");
  });

  it("never shows a previous installation's update or error under another selection", () => {
    const section = installedUpdateSection(copies, { ...callbacks({ modUpgradeError: "First error" }), installedUpdateInstallId: "first", onCheckInstalledUpdate: vi.fn() });
    fireEvent.change(getByRole(section, "combobox"), { target: { value: "second" } });
    expect(section.textContent).not.toContain("First error");
    expect(section.querySelector(".mod-upgrade-result")).toBeNull();
  });

  it("blocks other updates, fixes and installation selection during a mod mutation", () => {
    const page = updatesScreen(summary(), { ...callbacks({ modUpgradeBusy: true }), onCheckInstalledUpdate: vi.fn(), onCheckPatches: vi.fn(), checkAgain: vi.fn(), installApplication: vi.fn(), buildUpdatedCopy: vi.fn() });
    for (const name of ["Update CEBG app", "Create updated installation", "Check this installation", "Check installed mods"]) expect(button(page, name).disabled).toBe(true);
    expect((getByRole(page, "combobox", { name: "Game installation" }) as HTMLSelectElement).disabled).toBe(true);
  });
});

describe("BuffBot update controller and transport", () => {
  async function mounted() {
    const backend = new FixtureBackend();
    vi.spyOn(backend, "getUpdates").mockResolvedValue(summary());
    const compare = vi.spyOn(backend, "inspectInstalledUpdate").mockImplementation(async id => plan(id));
    const inspect = vi.spyOn(backend, "inspectModUpgrade").mockImplementation(async id => preview({ installId: id }));
    const restore = vi.spyOn(backend, "restoreModUpgrade").mockResolvedValue(preview());
    const root = document.createElement("div"); document.body.append(root);
    const app = await mountApp(root, backend); await app.navigate("updates");
    return { root, backend, app, compare, inspect, restore };
  }

  it("checks upgrade recovery even when comparison fails, without applying anything", async () => {
    const { root, compare, inspect, restore } = await mounted();
    expect(inspect).not.toHaveBeenCalled();
    compare.mockRejectedValueOnce(new Error("Collection comparison unavailable"));
    inspect.mockResolvedValueOnce(preview({ state: "needs-recovery", reviewToken: null, canRestore: true }));
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(button(root, "Restore BuffBot backup").disabled).toBe(false));
    expect(root.textContent).toContain("Collection comparison unavailable");
    expect(restore).not.toHaveBeenCalled();
  });

  it("discards stale upgrade responses after changing installation", async () => {
    const { root, inspect } = await mounted();
    let finishFirst!: (value: ModUpgradePreview) => void;
    inspect.mockImplementationOnce(() => new Promise(resolve => { finishFirst = resolve; }));
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(inspect).toHaveBeenCalledOnce());
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(button(root, "Check installed mods").disabled).toBe(false));
    finishFirst(preview({ detail: "Obsolete first preview" }));
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(root.textContent).not.toContain("Obsolete first preview");
    expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).value).toBe("second");
  });

  it("refreshes failed restoration and preserves useful error details", async () => {
    const { root, inspect, restore } = await mounted();
    inspect.mockResolvedValueOnce(preview({ state: "needs-recovery", canRestore: true }));
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(button(root, "Restore BuffBot backup").disabled).toBe(false));
    restore.mockRejectedValueOnce(new BackendCommandError({ code: "interrupted", message: "Update interrupted.", recovery_action: "Restore its backup.", technical_detail: "Exact operation diagnostic" }));
    inspect.mockResolvedValueOnce(preview({ state: "needs-recovery", canRestore: true, reviewToken: null }));
    fireEvent.click(button(root, "Restore BuffBot backup")); fireEvent.click(button(root, "Restore backup now"));
    await waitFor(() => expect(button(root, "Restore BuffBot backup").disabled).toBe(false));
    expect(root.textContent).toContain("Restore its backup");
    const errorDetails = Array.from(root.querySelectorAll<HTMLDetailsElement>(".mod-upgrade-result details"))
      .find(details => details.querySelector("summary")?.textContent === "BuffBot update error details");
    expect(errorDetails?.open).toBe(false);
    expect(errorDetails?.textContent).toContain("Exact operation diagnostic");
  });

  it("blocks navigation and duplicate mutation while an update is running", async () => {
    const { root, app, inspect, restore } = await mounted();
    let finish!: (value: ModUpgradePreview) => void;
    restore.mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    inspect.mockResolvedValueOnce(preview({ state: "needs-recovery", canRestore: true }));
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(button(root, "Restore BuffBot backup").disabled).toBe(false));
    fireEvent.click(button(root, "Restore BuffBot backup")); fireEvent.click(button(root, "Restore backup now"));
    await app.navigate("home");
    expect(getByRole(root, "heading", { name: "Updates" })).toBeTruthy();
    expect(button(root, "Check installed mods").disabled).toBe(true);
    expect(button(root, "Update CEBG app").disabled).toBe(true);
    expect(restore).toHaveBeenCalledOnce();
    finish(preview({ state: "retired", reviewToken: null }));
    await waitFor(() => expect(button(root, "Check installed mods").disabled).toBe(false));
  });

  it("passes exact native arguments and keeps Undo distinct from Restore", async () => {
    const invoke = vi.fn(async () => preview());
    const backend = new NativeBackend(invoke);
    await backend.inspectModUpgrade("first");
    await backend.restoreModUpgrade("first", true);
    await backend.restoreModUpgrade("first", false);
    expect(invoke.mock.calls).toEqual([
      ["inspect_mod_upgrade", { installId: "first" }],
      ["restore_mod_upgrade", { installId: "first", undo: true }],
      ["restore_mod_upgrade", { installId: "first", undo: false }],
    ]);
  });

  it("routes confirmed Undo and Restore separately and refreshes each result", async () => {
    for (const undo of [true, false]) {
      const { root, inspect, restore } = await mounted();
      inspect.mockResolvedValueOnce(preview({ state: undo ? "applied" : "needs-recovery", reviewToken: null, canUndo: undo, canRestore: !undo }));
      fireEvent.click(button(root, "Check installed mods"));
      const reviewName = undo ? "Undo BuffBot update" : "Restore BuffBot backup";
      await waitFor(() => expect(button(root, reviewName).disabled).toBe(false));
      fireEvent.click(button(root, reviewName));
      expect(restore).not.toHaveBeenCalled();
      fireEvent.click(button(root, undo ? "Confirm undo" : "Restore backup now"));
      await waitFor(() => expect(button(root, "Check installed mods").disabled).toBe(false));
      expect(restore).toHaveBeenCalledExactlyOnceWith("first", undo, expect.any(Function));
      expect(inspect).toHaveBeenCalledTimes(2);
      root.remove();
    }
  });

  it("rejects an upgrade preview for the wrong installation", async () => {
    const { root, inspect, restore } = await mounted();
    inspect.mockResolvedValueOnce(preview({ installId: "second" }));
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(root.textContent).toContain("BuffBot update belongs to another installation"));
    expect(queryByRole(root, "button", { name: "Review BuffBot update" })).toBeNull();
    expect(restore).not.toHaveBeenCalled();
  });

  it("ignores an obsolete upgrade error without clearing another check's busy state", async () => {
    const { root, inspect } = await mounted();
    let rejectFirst!: (error: Error) => void;
    let finishSecond!: (value: ModUpgradePreview) => void;
    inspect.mockImplementationOnce(() => new Promise((_resolve, reject) => { rejectFirst = reject; }));
    inspect.mockImplementationOnce(() => new Promise(resolve => { finishSecond = resolve; }));
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(inspect).toHaveBeenCalledTimes(1));
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    fireEvent.click(button(root, "Check installed mods"));
    await waitFor(() => expect(inspect).toHaveBeenCalledTimes(2));
    rejectFirst(new Error("Obsolete first upgrade error"));
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(root.textContent).not.toContain("Obsolete first upgrade error");
    expect(button(root, "Checking installed mods…").disabled).toBe(true);
    finishSecond(preview({ installId: "second" }));
    await waitFor(() => expect(button(root, "Check installed mods").disabled).toBe(false));
  });
});

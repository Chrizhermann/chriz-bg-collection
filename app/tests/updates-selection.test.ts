// @vitest-environment jsdom

import { fireEvent, getAllByRole, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type { ManagedCopyUpdate, ManagedInstallation, OperationProgress, PatchPreview, UpdateSummary } from "../src/contracts";
import { updatesScreen, type UpdateActions } from "../src/screens/updates";

const copies: ManagedCopyUpdate[] = [
  { installId: "first", name: "My game", path: "D:\\First", installedRecipeVersion: "1", state: "up-to-date", detail: "" },
  { installId: "second", name: "My game", path: "E:\\Second", installedRecipeVersion: "0", state: "update-available", detail: "" },
];
function summary(): UpdateSummary {
  return {
    checkedAt: null, networkState: "online", managedCopies: copies,
    application: { state: "available", currentVersion: "1", availableVersion: "2", detail: "" },
    recipe: { state: "up-to-date", currentVersion: "1", availableVersion: null, disposition: "up-to-date", detail: "", changes: [] },
    radar: { state: "not-installed", currentVersion: null, availableVersion: "2", detail: "" },
  };
}
function actions(overrides: Partial<UpdateActions> = {}): UpdateActions {
  return { onCheckInstalledUpdate: vi.fn(), onSelectInstalledUpdate: vi.fn(), onCheckPatches: vi.fn(),
    checkAgain: vi.fn(), installApplication: vi.fn(), buildUpdatedCopy: vi.fn(), installRadar: vi.fn(), ...overrides };
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); });

describe("one selected game in Updates", () => {
  it("puts one compact selector above every update section with short unique names", () => {
    const callbacks = actions({ installedUpdateInstallId: "second" });
    const page = updatesScreen(summary(), callbacks);
    const selector = getByRole(page, "combobox", { name: "Game installation" }) as HTMLSelectElement;
    expect(getAllByRole(page, "combobox")).toHaveLength(1);
    expect(selector.value).toBe("second");
    expect(page.children[1].className).toBe("update-selection");
    expect(Array.from(selector.options, option => option.textContent)).toEqual(["My game — First", "My game — Second"]);
    const details = page.querySelector<HTMLDetailsElement>(".update-selection-details")!;
    expect(details.open).toBe(false);
    expect(details.textContent).toContain("E:\\Second");
    expect(details.textContent).toContain("ID: second");
    expect(page.querySelector(".update-selection-status")?.textContent).toBe("Collection 0");
    fireEvent.click(getByRole(page, "button", { name: "Check installed mods" }));
    fireEvent.click(getByRole(page, "button", { name: "Check this installation" }));
    expect(callbacks.onCheckInstalledUpdate).toHaveBeenCalledWith("second");
    expect(callbacks.onCheckPatches).toHaveBeenCalledWith("second");
  });

  it("names incomplete records by folder, keeps unavailable entries selectable, and disables their actions", () => {
    const unavailable = { ...copies[0], name: "Chriz Easy BG — incomplete installation", state: "stale" as const };
    const page = updatesScreen({ ...summary(), managedCopies: [unavailable, copies[1]] }, actions({ installedUpdateInstallId: "first" }));
    expect(getAllByRole(page, "option")[0].textContent).toBe("First");
    expect((getByRole(page, "combobox") as HTMLSelectElement).disabled).toBe(false);
    for (const label of ["Check installed mods", "Check this installation"]) {
      expect((getByRole(page, "button", { name: label }) as HTMLButtonElement).disabled).toBe(true);
    }
    expect(queryByRole(page, "button", { name: "Install overlay" })).toBeNull();
    expect(page.textContent).toContain("Needs attention");
    expect(page.textContent).not.toContain("This folder is unavailable");
  });

  it("does not offer a newer separate installation just because a different game is old", () => {
    const current = updatesScreen(summary(), actions({ installedUpdateInstallId: "first" }));
    expect(queryByRole(current, "button", { name: "Create updated installation" })).toBeNull();
    const old = updatesScreen(summary(), actions({ installedUpdateInstallId: "second" }));
    expect(getByRole(old, "button", { name: "Create updated installation" })).toBeTruthy();
  });

  it("allows superseding a comparison but locks selection during patches, mod updates or app/overlay installs", () => {
    const comparison = updatesScreen(summary(), actions({ installedUpdateBusy: true }));
    expect((getByRole(comparison, "combobox") as HTMLSelectElement).disabled).toBe(false);
    for (const settings of [{ patchBusy: true }, { modUpgradeBusy: true }]) {
      const page = updatesScreen(summary(), actions(settings));
      expect((getByRole(page, "combobox") as HTMLSelectElement).disabled).toBe(true);
    }
    const installing = updatesScreen(summary(), actions(), false, true);
    expect((getByRole(installing, "combobox") as HTMLSelectElement).disabled).toBe(true);
  });

  it("keeps app updates available when there are no game installations", () => {
    const page = updatesScreen({ ...summary(), managedCopies: [] }, actions());
    expect(queryByRole(page, "combobox")).toBeNull();
    expect(getByRole(page, "button", { name: "Update CEBG app" })).toBeTruthy();
    expect(queryByRole(page, "button", { name: "Install overlay" })).toBeNull();
  });

  it("keeps even identical folder names distinguishable without displaying technical IDs", () => {
    const page = updatesScreen({ ...summary(), managedCopies: copies.map(copy => ({ ...copy, path: "D:\\Same" })) }, actions());
    const labels = getAllByRole(page, "option").map(option => option.textContent);
    expect(new Set(labels).size).toBe(2);
    expect(labels).toEqual(["My game — Same", "My game — Same (2)"]);
  });
});

describe("shared selection controller", () => {
  async function mounted() {
    const installs: ManagedInstallation[] = copies.map(copy => ({
      id: copy.installId, name: copy.name, path: copy.path, recipeVersion: copy.installedRecipeVersion,
      available: true, resumable: false, status: "Ready to play", receiptPath: null,
      launchPath: `${copy.path}\\InfinityLoader.exe`, completedAtMillis: 1, radarVersion: null,
    }));
    const backend = new FixtureBackend({ managedInstallations: installs });
    vi.spyOn(backend, "getUpdates").mockResolvedValue(summary());
    const compare = vi.spyOn(backend, "inspectInstalledUpdate");
    const patch = vi.spyOn(backend, "inspectInstallPatches");
    const apply = vi.spyOn(backend, "applyInstallPatch");
    const radar = vi.spyOn(backend, "installRadar");
    const root = document.createElement("div");
    document.body.append(root);
    const app = await mountApp(root, backend);
    await app.navigate("updates");
    return { root, compare, patch, apply, radar };
  }

  it("targets comparison, fixes and Radar to the same selection and discards old Apply controls", async () => {
    const { root, compare, patch, apply, radar } = await mounted();
    fireEvent.click(getByRole(root, "button", { name: "Check this installation" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Apply this fix" })).toBeTruthy());
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    expect(queryByRole(root, "button", { name: "Apply this fix" })).toBeNull();
    expect(apply).not.toHaveBeenCalled();
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect((getByRole(root, "button", { name: "Check installed mods" }) as HTMLButtonElement).disabled).toBe(false));
    expect(compare).toHaveBeenCalledWith("second");
    fireEvent.click(getByRole(root, "button", { name: "Check this installation" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Apply this fix" })).toBeTruthy());
    expect(patch.mock.calls).toEqual([["first"], ["second"]]);
    fireEvent.click(getByRole(root, "button", { name: "Apply this fix" }));
    await waitFor(() => expect(apply).toHaveBeenCalledWith("second", expect.any(String), true, false, expect.any(Function)));
    await waitFor(() => expect(getByRole(root, "button", { name: "Undo this fix" })).toBeTruthy());
    fireEvent.click(getByRole(root, "button", { name: "Install overlay" }));
    await waitFor(() => expect(radar).toHaveBeenCalledExactlyOnceWith("second"));
  });

  it("does not allow a target switch while a fix check is running", async () => {
    const { root, patch } = await mounted();
    let finish!: (preview: PatchPreview) => void;
    patch.mockImplementationOnce(() => new Promise(resolve => { finish = resolve; }));
    fireEvent.click(getByRole(root, "button", { name: "Check this installation" }));
    const selector = getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement;
    expect(selector.disabled).toBe(true);
    fireEvent.change(selector, { target: { value: "second" } });
    finish({ installId: "first", patchId: "test", title: "Already checked", state: "already-fixed", detail: "",
      baseRecipeVersion: "1", appliedPatchIds: [], reviewToken: null, canUndo: false, canRestore: false,
      fullBackupBytes: "0", saveBackupBytes: "0", availableBytes: "100", backupPath: "" });
    await waitFor(() => expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).value).toBe("first"));
    expect(patch).toHaveBeenCalledExactlyOnceWith("first");
  });

  it.each(["success", "error"] as const)("shows default-selection fix progress and %s feedback only for its installation", async outcome => {
    const { root, compare, patch, apply } = await mounted();
    const pending = deferred<PatchPreview>();
    let report: ((progress: OperationProgress) => void) | undefined;
    apply.mockImplementationOnce(async (_id, _token, _full, _saves, onProgress?: (progress: OperationProgress) => void) => {
      report = onProgress;
      return pending.promise;
    });
    expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).value).toBe("first");
    fireEvent.click(getByRole(root, "button", { name: "Check this installation" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Apply this fix" })).toBeTruthy());
    const available = await patch.mock.results[0].value;
    fireEvent.click(getByRole(root, "button", { name: "Apply this fix" }));
    await waitFor(() => expect(apply).toHaveBeenCalledExactlyOnceWith("first", available.reviewToken, true, false, expect.any(Function)));
    expect(compare).not.toHaveBeenCalled();
    report!({ stage: "backup-game", completed: "25", total: "100", unit: "bytes" });
    const working = getByRole(root, "region", { name: "Applying game fix" });
    expect(working.classList.contains("operation-working")).toBe(true);
    expect(getByRole(working, "progressbar", { name: "Backing up your game…" }).getAttribute("value")).toBe("25");
    if (outcome === "success") pending.resolve({ ...available, state: "applied", reviewToken: null, canUndo: true });
    else pending.reject(new Error("Fixture fix could not finish"));
    const title = outcome === "success" ? "Game fix applied" : "Game fix could not finish";
    await waitFor(() => {
      expect(getByRole(root, "region", { name: title }).classList.contains(`operation-${outcome}`)).toBe(true);
      expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).disabled).toBe(false);
    });
    expect(queryByRole(getByRole(root, "region", { name: title }), "progressbar")).toBeNull();
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    expect(root.querySelector(".operation-feedback")).toBeNull();
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "first" } });
    expect(getByRole(root, "region", { name: title })).toBeTruthy();
  });

  it.each(["success", "error"] as const)("shows default-selection Radar progress and %s feedback only for its installation", async outcome => {
    const { root, compare, patch, radar } = await mounted();
    const pending = deferred<void>();
    radar.mockImplementationOnce(() => pending.promise);
    expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).value).toBe("first");
    fireEvent.click(getByRole(root, "button", { name: "Install overlay" }));
    await waitFor(() => expect(radar).toHaveBeenCalledExactlyOnceWith("first"));
    expect(compare).not.toHaveBeenCalled();
    expect(patch).not.toHaveBeenCalled();
    const working = getByRole(root, "region", { name: "Updating BG Radar Overlay" });
    expect(working.classList.contains("operation-working")).toBe(true);
    expect(getByRole(working, "progressbar").hasAttribute("value")).toBe(false);
    if (outcome === "success") pending.resolve();
    else pending.reject(new Error("Fixture overlay could not finish"));
    const title = outcome === "success" ? "BG Radar Overlay update complete" : "BG Radar Overlay update could not finish";
    await waitFor(() => {
      expect(getByRole(root, "region", { name: title }).classList.contains(`operation-${outcome}`)).toBe(true);
      expect((getByRole(root, "combobox", { name: "Game installation" }) as HTMLSelectElement).disabled).toBe(false);
    });
    expect(queryByRole(getByRole(root, "region", { name: title }), "progressbar")).toBeNull();
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "second" } });
    expect(root.querySelector(".operation-feedback")).toBeNull();
    fireEvent.change(getByRole(root, "combobox", { name: "Game installation" }), { target: { value: "first" } });
    expect(getByRole(root, "region", { name: title })).toBeTruthy();
  });
});

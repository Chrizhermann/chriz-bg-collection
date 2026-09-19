// @vitest-environment jsdom

import { fireEvent, getAllByRole, getByRole, queryByRole, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NativeBackend, FixtureBackend } from "../src/backend";
import type { ManagedCopyUpdate, PatchPreview, UpdateSummary } from "../src/contracts";
import { appendPatchAvailability, installPatchesSection, type InstallPatchActions } from "../src/components/install-patches";
import { updatesScreen } from "../src/screens/updates";
import { createAppShell } from "../src/components/app-shell";
import { updateUpdatesControl } from "../src/components/update-notification";
import { mountApp } from "../src/app";

const copies: ManagedCopyUpdate[] = [
  { installId: "first", name: "Chriz Easy BG", path: "D:\\First", installedRecipeVersion: "0.1.0-alpha.1", state: "update-available", detail: "" },
  { installId: "second", name: "Chriz Easy BG", path: "E:\\Second", installedRecipeVersion: "0.1.0-alpha.2", state: "update-available", detail: "" },
];

afterEach(() => document.body.replaceChildren());

function preview(overrides: Partial<PatchPreview> = {}): PatchPreview {
  return {
    installId: "first", patchId: "artisan-links-1", title: "Artisan kit-description links",
    state: "available", detail: "Three kit descriptions need corrected links.",
    baseRecipeVersion: "0.1.0-alpha.1", appliedPatchIds: [], reviewToken: "bound-review",
    canUndo: false, canRestore: false, fullBackupBytes: "12884901888", saveBackupBytes: "104857600",
    availableBytes: "107374182400", backupPath: "C:\\Users\\Player\\Games\\CEBG-Backups\\first",
    ...overrides,
  };
}

function actions(overrides: Partial<InstallPatchActions> = {}): InstallPatchActions {
  return { onCheckPatches: vi.fn(), onApplyPatch: vi.fn(), onUndoPatch: vi.fn(), onRestorePatch: vi.fn(), ...overrides };
}

describe("current-game fixes", () => {
  it("does not scan on rendering, and distinguishes identical names by path and id", () => {
    const callbacks = actions();
    const section = installPatchesSection(copies, callbacks);
    expect(callbacks.onCheckPatches).not.toHaveBeenCalled();
    const options = getAllByRole(section, "option");
    expect(options[0].textContent).toContain("D:\\First");
    expect(options[1].textContent).toContain("E:\\Second");
    expect((options[1] as HTMLOptionElement).value).toBe("second");
    fireEvent.change(getByRole(section, "combobox"), { target: { value: "second" } });
    expect(callbacks.onCheckPatches).not.toHaveBeenCalled();
    fireEvent.click(getByRole(section, "button", { name: "Check this installation" }));
    expect(callbacks.onCheckPatches).toHaveBeenCalledWith("second");
  });

  it("defaults to a full backup, keeps saves optional, and discloses the exact narrow scope", () => {
    const callbacks = actions({ patches: preview() });
    const section = installPatchesSection(copies, callbacks);
    expect((getByRole(section, "checkbox", { name: /Back up the entire playable game folder/ }) as HTMLInputElement).checked).toBe(true);
    expect((getByRole(section, "checkbox", { name: /Copy saved games/ }) as HTMLInputElement).checked).toBe(false);
    expect(section.textContent).toContain("Can be applied to your current run");
    expect(section.textContent).toContain("Affected-file backup is always included");
    expect(section.textContent).toContain("does not update kit abilities");
    expect(section.textContent).toContain("dialog.tlk");
    expect(section.textContent).toContain("12 GiB");
    expect(section.textContent).toContain("100 GiB");
    expect(section.textContent).toContain(preview().backupPath);
    expect(section.textContent).toContain("Base collection: 0.1.0-alpha.1");
    fireEvent.click(getByRole(section, "button", { name: "Apply this fix" }));
    expect(callbacks.onApplyPatch).toHaveBeenCalledWith(true, false);
  });

  it("passes explicit backup choices and updates the space estimate", () => {
    const callbacks = actions({ patches: preview() });
    const section = installPatchesSection(copies, callbacks);
    document.body.append(section);
    fireEvent.click(getByRole(section, "checkbox", { name: /Back up the entire playable game folder/ }));
    fireEvent.click(getByRole(section, "checkbox", { name: /Copy saved games/ }));
    expect(section.querySelector(".patch-space")?.textContent).toContain("100 MiB");
    fireEvent.click(getByRole(section, "button", { name: "Apply this fix" }));
    expect(callbacks.onApplyPatch).toHaveBeenCalledWith(false, true);
  });

  it("hides a previous installation's actions after selection changes", () => {
    const callbacks = actions({ patches: preview(), patchInstallId: "first" });
    const section = installPatchesSection(copies, callbacks);
    fireEvent.change(getByRole(section, "combobox"), { target: { value: "second" } });
    expect(queryByRole(section, "button", { name: "Apply this fix" })).toBeNull();
    expect(section.textContent).toContain("E:\\Second");
    expect(callbacks.onCheckPatches).not.toHaveBeenCalled();
  });

  it.each(["already-fixed", "not-selected", "unsupported", "needs-recovery"] as const)("does not offer Apply for %s", (state) => {
    const section = installPatchesSection(copies, actions({ patches: preview({ state }) }));
    expect(queryByRole(section, "button", { name: "Apply this fix" })).toBeNull();
  });

  it("requires the native review token before enabling Apply", () => {
    const section = installPatchesSection(copies, actions({ patches: preview({ reviewToken: null }) }));
    expect((getByRole(section, "button", { name: "Apply this fix" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("disables an unavailable save snapshot without representing it as zero bytes", () => {
    const callbacks = actions({ patches: preview({ saveBackupBytes: "unavailable" }) });
    const section = installPatchesSection(copies, callbacks);
    const snapshot = getByRole(section, "checkbox", { name: /Copy saved games/ }) as HTMLInputElement;
    expect(snapshot.disabled).toBe(true);
    expect(snapshot.checked).toBe(false);
    expect(snapshot.closest("label")?.textContent).toContain("Not available");
    expect(section.textContent).toContain("Make a manual backup");
    fireEvent.click(getByRole(section, "button", { name: "Apply this fix" }));
    expect(callbacks.onApplyPatch).toHaveBeenCalledWith(true, false);
  });

  it("offers only guarded undo or recovery and warns about saved progress", () => {
    const undo = actions({ patches: preview({ state: "applied", canUndo: true, appliedPatchIds: ["artisan-links-1"] }) });
    const section = installPatchesSection(copies, undo);
    expect(section.textContent).toContain("artisan-links-1");
    expect(section.textContent).toContain("cannot reverse progress saved after a patch");
    fireEvent.click(getByRole(section, "button", { name: "Undo this fix" }));
    expect(undo.onUndoPatch).toHaveBeenCalledOnce();
    const recovery = actions({ patches: preview({ state: "needs-recovery", canRestore: true }) });
    const repair = installPatchesSection(copies, recovery);
    expect(queryByRole(repair, "button", { name: "Apply this fix" })).toBeNull();
    fireEvent.click(getByRole(repair, "button", { name: "Restore backup" }));
    expect(recovery.onRestorePatch).toHaveBeenCalledOnce();
    expect(queryByRole(installPatchesSection(copies, actions({ patches: preview({ state: "applied", canUndo: false }) })), "button", { name: "Undo this fix" })).toBeNull();
  });

  it("locks every action and input while work is in progress and exposes errors", () => {
    const section = installPatchesSection(copies, actions({ patches: preview(), patchBusy: true, patchError: "Close this game's running process." }));
    for (const control of section.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>("button, input, select")) expect(control.disabled).toBe(true);
    expect(getByRole(section, "status").textContent).toContain("Checking, backing up, applying or restoring");
    expect(getByRole(section, "alert").textContent).toContain("Close this game's running process.");
    expect(section.getAttribute("aria-busy")).toBe("true");
  });

  it("cannot check missing installations and explains an empty list", () => {
    const stale = installPatchesSection([{ ...copies[0], state: "stale" }], actions());
    expect((getByRole(stale, "button", { name: "Check this installation" }) as HTMLButtonElement).disabled).toBe(true);
    const empty = installPatchesSection([], actions());
    expect(empty.textContent).toContain("Complete an installation first");
    expect(queryByRole(empty, "button", { name: "Apply this fix" })).toBeNull();
  });

  it("can disable patch controls for an app update without claiming a patch is in progress", () => {
    const section = installPatchesSection(copies, actions({ patches: preview() }), true);
    for (const control of section.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLButtonElement>("button, input, select")) expect(control.disabled).toBe(true);
    expect(queryByRole(section, "status")).toBeNull();
    expect(section.getAttribute("aria-busy")).toBe("false");
  });

  it("integrates in Updates without allowing app updates during patch work", () => {
    const summary: UpdateSummary = {
      checkedAt: null, networkState: "online",
      application: { state: "available", currentVersion: "1", availableVersion: "2", detail: "" },
      recipe: { state: "up-to-date", currentVersion: "1", availableVersion: null, detail: "", disposition: "up-to-date", changes: [] },
      managedCopies: copies,
    };
    const page = updatesScreen(summary, { checkAgain: vi.fn(), installApplication: vi.fn(), buildUpdatedCopy: vi.fn(), ...actions({ patchBusy: true }) });
    expect(getByRole(page, "heading", { name: "Fixes for your current game" })).toBeTruthy();
    expect((getByRole(page, "button", { name: "Update CEBG app" }) as HTMLButtonElement).disabled).toBe(true);
    expect((getByRole(page, "button", { name: "Check for updates" }) as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("patch update notifications", () => {
  const summary: UpdateSummary = {
    checkedAt: null, networkState: "online",
    application: { state: "available", currentVersion: "1", availableVersion: "2", detail: "" },
    recipe: { state: "available", currentVersion: "1", availableVersion: "2", detail: "", disposition: "deferred-for-next-playthrough", changes: [] },
    managedCopies: copies,
    radar: { state: "available", currentVersion: "1", availableVersion: "2", detail: "" },
  };

  it("adds checked patch availability without dropping other update channels", () => {
    const shell = createAppShell("home", document.createElement("div"), () => undefined);
    const button = getByRole(shell, "button", { name: "Updates" }) as HTMLButtonElement;
    updateUpdatesControl(button, summary);
    appendPatchAvailability(button, [preview()]);
    expect(shell.querySelectorAll(".update-badge")).toHaveLength(1);
    expect(button.dataset.updateAvailable).toBe("true");
    expect(button.dataset.patchAvailable).toBe("true");
    const tooltip = getByRole(shell, "tooltip").textContent;
    for (const channel of ["CEBG app", "collection", "BG Radar Overlay", "Current-game fixes available"]) expect(tooltip).toContain(channel);
  });

  it("keeps recovery conspicuous and clears it after a successful restore or recheck", () => {
    const shell = createAppShell("home", document.createElement("div"), () => undefined);
    const button = getByRole(shell, "button", { name: "Updates" }) as HTMLButtonElement;
    const current: UpdateSummary = { ...summary, application: { ...summary.application, state: "up-to-date", availableVersion: null }, recipe: { ...summary.recipe, state: "up-to-date", availableVersion: null }, managedCopies: [], radar: undefined };
    updateUpdatesControl(button, current);
    appendPatchAvailability(button, [preview({ state: "needs-recovery" })]);
    expect(button.querySelector(".update-badge")?.textContent).toBe("Attention");
    expect(getByRole(shell, "tooltip").textContent).toContain("Patch recovery needed");
    expect(button.dataset.patchRecovery).toBe("true");
    updateUpdatesControl(button, current);
    appendPatchAvailability(button, [preview({ state: "applied" })]);
    expect(button.querySelector(".update-badge")).toBeNull();
    expect(button.dataset.updateAvailable).toBe("false");
    expect(button.dataset.patchRecovery).toBe("false");
  });

  it("only inspects after an explicit click and keeps the badge across normal update checks", async () => {
    const backend = new FixtureBackend();
    const inspect = vi.spyOn(backend, "inspectInstallPatches");
    const root = document.createElement("div");
    document.body.append(root);
    const app = await mountApp(root, backend);
    expect(inspect).not.toHaveBeenCalled();
    await app.navigate("updates");
    expect(inspect).not.toHaveBeenCalled();
    fireEvent.click(getByRole(root, "button", { name: "Check this installation" }));
    await waitFor(() => expect(getByRole(root, "tooltip").textContent).toContain("Current-game fixes available"));
    expect(inspect).toHaveBeenCalledTimes(1);
    fireEvent.click(getByRole(root, "button", { name: "Check for updates" }));
    await waitFor(() => expect((getByRole(root, "button", { name: "Check for updates" }) as HTMLButtonElement).disabled).toBe(false));
    expect(inspect).toHaveBeenCalledTimes(1);
    expect(getByRole(root, "tooltip").textContent).toContain("Current-game fixes available");
    fireEvent.click(getByRole(root, "button", { name: "Apply this fix" }));
    await waitFor(() => expect(root.querySelector("[data-patch-state='applied']")).not.toBeNull());
    expect(getByRole(root, "tooltip").textContent).not.toContain("Current-game fixes available");
  });
});

describe("patch backend boundary", () => {
  it("uses registry IDs and the preview token, without accepting filesystem authority", async () => {
    const result = preview();
    const invoke = vi.fn(async () => result);
    const backend = new NativeBackend(invoke);
    await expect(backend.inspectInstallPatches("first")).resolves.toEqual(result);
    expect(invoke).toHaveBeenLastCalledWith("inspect_install_patches", { installId: "first" });
    await expect(backend.applyInstallPatch("first", "bound-review", true, false)).resolves.toEqual(result);
    expect(invoke).toHaveBeenLastCalledWith("apply_install_patch", { installId: "first", reviewToken: "bound-review", fullBackup: true, saveBackup: false });
    await backend.undoInstallPatch("first");
    expect(invoke).toHaveBeenLastCalledWith("undo_install_patch", { installId: "first" });
    await backend.restoreInstallPatch("first");
    expect(invoke).toHaveBeenLastCalledWith("restore_install_patch", { installId: "first" });
  });

  it("the preview fixture can apply and undo without touching a real installation", async () => {
    const backend = new FixtureBackend();
    const result = await backend.inspectInstallPatches("fixture-install");
    expect(result.state).toBe("available");
    await expect(backend.applyInstallPatch("other", result.reviewToken!, true, false)).rejects.toThrow();
    await expect(backend.applyInstallPatch("fixture-install", result.reviewToken!, true, false)).resolves.toMatchObject({ state: "applied", canUndo: true });
    await expect(backend.inspectInstallPatches("fixture-install")).resolves.toMatchObject({ state: "applied" });
    await expect(backend.undoInstallPatch("fixture-install")).resolves.toMatchObject({ state: "available", canUndo: false });
  });
});

// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { InstallationRemovalPreview, InstallationRemovalResult, ManagedInstallation } from "../src/contracts";
import { mountApp } from "../src/app";
import { buildScreen } from "../src/screens/build";
import { installationRemovalDialog, installationRemovalNotice } from "../src/components/installation-removal";

const installation: ManagedInstallation = {
  id: "chosen", name: "My test installation", path: "D:\\CEBG\\My test installation", status: "Ready",
  receiptPath: null, launchPath: "D:\\CEBG\\My test installation\\game\\InfinityLoader.exe",
  completedAtMillis: 1, available: true, resumable: false,
};
const profile = "C:\\Users\\Fixture\\Documents\\Chriz Easy BG-test";

class RemovalBackend extends FixtureBackend {
  entries = [installation];
  previews: string[] = [];
  removals: { id: string; token: string }[] = [];
  action: "delete" | "forget" | "unlist" = "delete";
  inUse = false;
  /** Whether the profile holds nothing but CEBG's marker, and whether it then stays after all. */
  unusedProfile = false;
  unusedProfileKept = false;
  override listManagedInstallations() { return Promise.resolve([...this.entries]); }
  override previewInstallationRemoval(id: string): Promise<InstallationRemovalPreview> {
    this.previews.push(id);
    return Promise.resolve({
      installId: id, displayName: installation.name, managedRoot: installation.path, action: this.action,
      reason: this.action === "unlist" ? "Its CEBG install records are missing or damaged." : null,
      preservedSavePath: this.unusedProfile ? null : profile, unusedSavePath: this.unusedProfile ? profile : null,
      confirmationToken: "exact-preview-token",
    });
  }
  override removeInstallation(id: string, token: string): Promise<InstallationRemovalResult> {
    this.removals.push({ id, token });
    if (this.inUse) return Promise.reject(new BackendCommandError({
      code: "installation_in_use", message: "This game is still running.", recovery_action: "Close the game and try again.", technical_detail: "fixture process",
    }));
    this.entries = this.entries.filter((entry) => entry.id !== id);
    return Promise.resolve({ unusedSavePathKept: this.unusedProfileKept ? profile : null });
  }
}

describe("explicit managed-install removal", () => {
  afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); });

  it.each([
    ["12884901888", "12 GiB"],
    ["1610612736", "1.5 GiB"],
    ["0", "0 B"],
  ])("shows %s bytes as an approximate folder size", (approximateSizeBytes, formatted) => {
    const dialog = installationRemovalDialog({
      installId: installation.id, displayName: installation.name, managedRoot: installation.path,
      action: "delete", preservedSavePath: null, confirmationToken: "token", approximateSizeBytes,
    }, false, null, () => {}, () => {});
    expect(dialog.textContent).toContain(`Approximate folder size: ${formatted}`);
    expect(dialog.textContent).toContain("Disk space freed may differ");
    expect(dialog.textContent).toContain("This cannot be undone");
  });

  it.each([undefined, null, "invalid", "-1"])("keeps deletion available when size is %s", (approximateSizeBytes) => {
    const dialog = installationRemovalDialog({
      installId: installation.id, displayName: installation.name, managedRoot: installation.path,
      action: "delete", preservedSavePath: null, confirmationToken: "token", approximateSizeBytes,
    }, false, null, () => {}, () => {});
    expect(dialog.textContent).toContain("Approximate folder size: unavailable");
    expect((getByRole(dialog, "button", { name: "Delete permanently" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("shows no size estimate when forgetting a missing folder", () => {
    const dialog = installationRemovalDialog({
      installId: installation.id, displayName: installation.name, managedRoot: installation.path,
      action: "forget", preservedSavePath: null, confirmationToken: "token", approximateSizeBytes: "12884901888",
    }, false, null, () => {}, () => {});
    expect(dialog.textContent).not.toContain("folder size");
    expect(dialog.textContent).not.toContain("Disk space");
    expect(dialog.textContent).toContain("no files will be deleted");
  });

  it("previews the exact selected folder, preserves saves outside it, and cancellation performs no removal", async () => {
    const backend = new RemovalBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Delete / remove installation…" }));
    const dialog = getByRole(root, "alertdialog");
    expect(backend.previews).toEqual(["chosen"]);
    expect(getByText(dialog, installation.path)).toBeTruthy();
    expect(dialog.textContent).toContain("including any files or saves you manually placed there");
    expect(dialog.textContent).toContain("Saved-game profile kept");
    expect(backend.removals).toEqual([]);
    await user.click(getByRole(dialog, "button", { name: "Cancel" }));
    expect(queryByRole(root, "alertdialog")).toBeNull();
    expect(backend.removals).toEqual([]);
    expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
  });

  it("removes only the confirmed entry and clears stale launcher state", async () => {
    const backend = new RemovalBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    window.localStorage.setItem("cebg.last-install-id", "chosen");
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Delete / remove installation…" }));
    await user.click(getByRole(root, "button", { name: "Delete permanently" }));
    await waitFor(() => expect(backend.removals).toEqual([{ id: "chosen", token: "exact-preview-token" }]));
    expect(queryByRole(root, "alertdialog")).toBeNull();
    expect(queryByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeNull();
    expect(root.textContent).toContain("No installations yet");
    expect(window.localStorage.getItem("cebg.last-install-id")).toBeNull();
  });

  it("keeps an in-use installation and requires a fresh preview and explicit confirmation before retry", async () => {
    const backend = new RemovalBackend();
    backend.inUse = true;
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Delete / remove installation…" }));
    await user.click(getByRole(root, "button", { name: "Delete permanently" }));
    const dialog = getByRole(root, "alertdialog");
    expect(getByRole(dialog, "alert").textContent).toContain("Close the game and try again");
    expect(backend.entries).toHaveLength(1);
    backend.inUse = false;
    await user.click(getByRole(dialog, "button", { name: "Review removal again" }));
    expect(backend.previews).toEqual(["chosen", "chosen"]);
    expect(backend.removals).toHaveLength(1);
    expect(backend.entries).toHaveLength(1);
    await user.click(getByRole(root, "button", { name: "Delete permanently" }));
    expect(backend.removals).toHaveLength(2);
    expect(backend.entries).toHaveLength(0);
  });

  it("offers entry-only forgetting for a missing folder", async () => {
    const backend = new RemovalBackend();
    backend.action = "forget";
    backend.entries = [{ ...installation, available: false, launchPath: null }];
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Delete / remove installation…" }));
    expect(getByRole(root, "alertdialog").textContent).toContain("no files will be deleted");
    expect(queryByRole(root, "button", { name: "Delete permanently" })).toBeNull();
    await user.click(getByRole(root, "button", { name: "Forget installation" }));
    expect(backend.removals[0]?.id).toBe("chosen");
  });

  it("names an unused saved-game profile that deletion removes too", () => {
    const dialog = installationRemovalDialog({
      installId: installation.id, displayName: installation.name, managedRoot: installation.path,
      action: "delete", preservedSavePath: null, unusedSavePath: profile, confirmationToken: "token",
    }, false, null, () => {}, () => {});
    expect(dialog.textContent).toContain("Saved-game profile deleted too, since the game never used it:");
    expect(getByText(dialog, profile)).toBeTruthy();
    expect(dialog.textContent).not.toContain("Saved-game profile kept");
  });

  it("says that forgetting a missing folder deletes only its unused profile", () => {
    const dialog = installationRemovalDialog({
      installId: installation.id, displayName: installation.name, managedRoot: installation.path,
      action: "forget", preservedSavePath: null, unusedSavePath: profile, confirmationToken: "token",
    }, false, null, () => {}, () => {});
    expect(dialog.textContent).toContain("only its unused saved-game profile below is deleted");
    expect(dialog.textContent).not.toContain("no files will be deleted");
    expect(getByText(dialog, profile)).toBeTruthy();
  });

  it.each([
    ["delete", false, null, "Installation deleted. Saved games outside its folder were kept."],
    ["delete", true, null, "Installation deleted, along with its unused saved-game profile."],
    ["delete", true, profile, `Installation deleted. Its unused saved-game profile could not be deleted; it stays at ${profile}.`],
    ["forget", false, null, "Installation entry removed. No files were deleted."],
    ["forget", true, null, "Installation entry removed, along with its unused saved-game profile."],
    ["forget", true, profile, `Installation entry removed. Its unused saved-game profile could not be deleted; it stays at ${profile}.`],
    ["unlist", false, null, "Installation removed from My installs. Its folder was not changed."],
  ] as const)("reports a completed %s (unused profile: %s, kept: %s)", (action, unused, kept, notice) => {
    const preview: InstallationRemovalPreview = {
      installId: installation.id, displayName: installation.name, managedRoot: installation.path, action,
      preservedSavePath: unused ? null : profile, unusedSavePath: unused ? profile : null, confirmationToken: "token",
    };
    expect(installationRemovalNotice(preview, { unusedSavePathKept: kept })).toBe(notice);
  });

  it("reports an unused profile that stays after the installation was deleted", async () => {
    const backend = new RemovalBackend();
    backend.unusedProfile = true;
    backend.unusedProfileKept = true;
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Delete / remove installation…" }));
    expect(getByRole(root, "alertdialog").textContent).toContain("since the game never used it");
    await user.click(getByRole(root, "button", { name: "Delete permanently" }));
    await waitFor(() => expect(backend.removals).toEqual([{ id: "chosen", token: "exact-preview-token" }]));
    expect(queryByRole(root, "alertdialog")).toBeNull();
    expect(root.textContent).toContain(`Its unused saved-game profile could not be deleted; it stays at ${profile}.`);
  });

  it("explains entry-only removal of a folder that cannot be proven to be the installation", () => {
    const dialog = installationRemovalDialog({
      installId: installation.id, displayName: installation.name, managedRoot: installation.path,
      action: "unlist", reason: "Its CEBG install records are missing or damaged.", preservedSavePath: null,
      confirmationToken: "token", approximateSizeBytes: "12884901888",
    }, false, null, () => {}, () => {});
    expect(getByRole(dialog, "heading").textContent).toBe("Remove this installation from My installs?");
    expect(dialog.textContent).toContain("it will not delete anything in it");
    expect(dialog.textContent).toContain("Its CEBG install records are missing or damaged.");
    expect(dialog.textContent).toContain(installation.path);
    expect(dialog.textContent).not.toContain("folder size");
    expect(dialog.textContent).not.toContain("cannot be undone");
    expect(queryByRole(dialog, "button", { name: "Delete permanently" })).toBeNull();
    expect(getByRole(dialog, "button", { name: "Remove from list" })).toBeTruthy();
  });

  it("removes an installation it cannot prove from the list and says its folder was kept", async () => {
    const backend = new RemovalBackend();
    backend.action = "unlist";
    backend.entries = [{ ...installation, available: false, launchPath: null, status: "Build unavailable — folder moved or changed" }];
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Delete / remove installation…" }));
    expect(getByRole(root, "alertdialog").textContent).toContain("Its CEBG install records are missing or damaged.");
    await user.click(getByRole(root, "button", { name: "Remove from list" }));
    await waitFor(() => expect(backend.removals).toEqual([{ id: "chosen", token: "exact-preview-token" }]));
    expect(queryByRole(root, "alertdialog")).toBeNull();
    expect(root.textContent).toContain("Installation removed from My installs. Its folder was not changed.");
  });

  it("keeps deletion available for sealed failures but disables it during active work", () => {
    for (const state of ["failed", "paused", "running", "attention", "waiting-manual"] as const) {
      const root = buildScreen({ state, headline: "Test", detail: "Test", phases: [], logTail: [], manualArchiveName: null, freshCopyRequired: state === "failed" }, {
        backToSetup: () => {}, advance: () => {}, retry: () => {}, supplyManual: () => {}, openManualSource: () => {}, pause: () => {}, stopNow: () => {},
        diagnostics: () => {}, diagnosticsAvailable: true, fixture: false, retryAvailable: false,
        logState: { open: false, paused: false }, updateLogState: () => {}, remove: () => {},
      });
      expect((getByRole(root, "button", { name: "Delete installation…" }) as HTMLButtonElement).disabled).toBe(!["failed", "paused"].includes(state));
    }
  });
});

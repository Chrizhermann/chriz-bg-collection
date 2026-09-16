// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { InstallationRemovalPreview, ManagedInstallation } from "../src/contracts";
import { mountApp } from "../src/app";
import { buildScreen } from "../src/screens/build";

const installation: ManagedInstallation = {
  id: "chosen", name: "My test installation", path: "D:\\CEBG\\My test installation", status: "Ready",
  receiptPath: null, launchPath: "D:\\CEBG\\My test installation\\game\\InfinityLoader.exe",
  completedAtMillis: 1, available: true, resumable: false,
};

class RemovalBackend extends FixtureBackend {
  entries = [installation];
  previews: string[] = [];
  removals: { id: string; token: string }[] = [];
  action: "delete" | "forget" = "delete";
  inUse = false;
  override listManagedInstallations() { return Promise.resolve([...this.entries]); }
  override previewInstallationRemoval(id: string): Promise<InstallationRemovalPreview> {
    this.previews.push(id);
    return Promise.resolve({
      installId: id, displayName: installation.name, managedRoot: installation.path, action: this.action,
      preservedSavePath: "C:\\Users\\Fixture\\Documents\\Chriz Easy BG-test", confirmationToken: "exact-preview-token",
    });
  }
  override removeInstallation(id: string, token: string): Promise<void> {
    this.removals.push({ id, token });
    if (this.inUse) return Promise.reject(new BackendCommandError({
      code: "installation_in_use", message: "This game is still running.", recovery_action: "Close the game and try again.", technical_detail: "fixture process",
    }));
    this.entries = this.entries.filter((entry) => entry.id !== id);
    return Promise.resolve();
  }
}

describe("explicit managed-install removal", () => {
  afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); });

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

// @vitest-environment jsdom

import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { BuildSnapshot, CommandErrorPayload, RunEventEnvelope } from "../src/contracts";
import { buildScreen, type BuildActions } from "../src/screens/build";

const folderFailure: CommandErrorPayload = {
  code: "unsafe_target",
  message: "The install folder could not be used.",
  recovery_action: "Return to setup and choose another empty install folder.",
  technical_detail: "C:\\Restricted\\My game: could not create folder: Access is denied. (os error 5)",
};

const unsupportedInstaller: CommandErrorPayload = {
  code: "installer_launch_unsupported",
  message: "This CEBG release cannot start one of the selected mod installers.",
  recovery_action: "Install a corrected CEBG release, then retry this installation in the same folder. Keep the existing installation files.",
  technical_detail: "explicit-relative-TP2 invocation is unavailable until the signed recipe schema records separately reviewed compatibility evidence; use setup-name mode",
};

const buildActions: BuildActions = {
  backToSetup: () => {}, advance: () => {}, retry: () => {}, supplyManual: () => {},
  openManualSource: () => {}, pause: () => {}, stopNow: () => {}, diagnostics: () => {},
  diagnosticsAvailable: true, fixture: false, retryAvailable: true,
  logState: { open: false, paused: false }, updateLogState: () => {},
};

async function openApp(backend: FixtureBackend) {
  const root = document.createElement("div");
  document.body.append(root);
  await mountApp(root, backend);
  return root;
}

describe("actionable installation errors", () => {
  afterEach(() => {
    document.body.replaceChildren();
    window.localStorage.clear();
  });

  it("keeps the installer correction readable and its raw failure only in collapsed Error details", async () => {
    const snapshot: BuildSnapshot = {
      state: "failed", headline: "Installation stopped", detail: unsupportedInstaller.message,
      recoveryAction: unsupportedInstaller.recovery_action,
      failureReason: unsupportedInstaller.technical_detail, commandError: unsupportedInstaller,
      phases: [], logTail: [], manualArchiveName: null,
    };
    const root = buildScreen(snapshot, buildActions);
    document.body.append(root);

    const card = root.querySelector<HTMLElement>(".status-card")!;
    expect(card.querySelector(".failure-reason")).toBeNull();
    const details = card.querySelector<HTMLDetailsElement>(".error-details")!;
    expect(details.open).toBe(false);
    expect(details.textContent).toContain(unsupportedInstaller.technical_detail);
    const visibleCopy = card.cloneNode(true) as HTMLElement;
    visibleCopy.querySelector("details")?.remove();
    expect(visibleCopy.textContent).toContain(unsupportedInstaller.message);
    expect(visibleCopy.textContent).toContain(unsupportedInstaller.recovery_action);
    expect(visibleCopy.textContent).not.toContain("explicit-relative-TP2");
    expect(getByRole(card, "button", { name: "Retry failed step" })).toBeTruthy();
    await userEvent.setup().click(getByText(card, "Error details"));
    expect(details.open).toBe(true);
  });

  it.each([false, true])("keeps raw failure evidence when no structured detail is available (fresh copy: %s)", (freshCopyRequired) => {
    const reason = "Expected component 0 is missing from the completed WeiDU log.";
    const root = buildScreen({
      state: "failed", headline: "This installation needs attention", detail: "Keep the installation folder unchanged.",
      failureReason: reason, freshCopyRequired,
      phases: [], logTail: [], manualArchiveName: null,
    }, buildActions);
    expect(root.querySelector(".failure-reason")?.textContent).toBe(reason);
    expect(root.querySelector(".error-details")).toBeNull();
    expect(getByRole(root, "button", { name: freshCopyRequired ? "Start new installation" : "Retry failed step" })).toBeTruthy();
  });

  it.each([false, true])("preserves a distinct failure reason alongside generic structured diagnostics (fresh copy: %s)", (freshCopyRequired) => {
    const reason = "Expected component 0 is missing from the completed WeiDU log.";
    const root = buildScreen({
      state: "failed", headline: "This installation needs attention", detail: "Keep the installation folder unchanged.",
      failureReason: reason, freshCopyRequired,
      commandError: { code: "campaign_failed", message: "The installation could not finish.", recovery_action: "Export diagnostics.", technical_detail: "Generic worker error" },
      phases: [], logTail: [], manualArchiveName: null,
    }, buildActions);
    expect(root.querySelector<HTMLDetailsElement>(".error-details")?.open).toBe(false);
    expect(root.querySelector(".error-details")?.textContent).toContain("Generic worker error");
    if (freshCopyRequired) {
      expect(root.querySelector(".failure-reason")?.textContent).toBe(reason);
      expect(queryByRole(root, "button", { name: "Retry failed step" })).toBeNull();
    } else {
      expect(root.querySelector(".failure-reason")).toBeNull();
      expect(root.querySelector(".error-details")?.textContent).toContain(reason);
    }
  });

  it("shows the exact folder failure before a run exists, with setup controls still available", async () => {
    class RejectedStartBackend extends FixtureBackend {
      override startBuild(): Promise<{ runId: string }> {
        return Promise.reject(new BackendCommandError(folderFailure));
      }
    }
    const root = await openApp(new RejectedStartBackend());
    await userEvent.setup().click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain(folderFailure.technical_detail));
    const details = getByRole(root, "alert").querySelector("details");
    expect(details?.open).toBe(true);
    expect(getByRole(root, "button", { name: "Customize" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(root.textContent).not.toContain("Run as administrator");
  });

  it("shows a worker's folder failure without requiring a diagnostic export or offering an unsafe retry", async () => {
    class FailedWorkerBackend extends FixtureBackend {
      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }
      override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
        queueMicrotask(() => {
          onEvent({ runId: "folder-run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "folder-install", resumed: false } });
          onEvent({ runId: "folder-run", sequenceAsString: "2", event: { type: "error", step_id: null, message: folderFailure.message } });
        });
        return Promise.resolve({ runId: "folder-run" });
      }
      override getRunSnapshot(runId: string) {
        return Promise.resolve({ runId, status: "failed" as const, events: [], report: null, error: folderFailure });
      }
    }
    const root = await openApp(new FailedWorkerBackend());
    await userEvent.setup().click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(root.querySelector(".status-card .error-details")?.textContent).toContain(folderFailure.technical_detail));
    expect(root.querySelector<HTMLDetailsElement>(".status-card .error-details")?.open).toBe(true);
    expect(getByRole(root, "button", { name: "Back to setup" })).toBeTruthy();
    expect(queryByRole(root, "button", { name: "Retry failed step" })).toBeNull();
  });

  it("keeps shortcut failure nonfatal and exposes its actual reason as text, not markup", async () => {
    class ShortcutFailureBackend extends FixtureBackend {
      constructor() {
        super({ managedInstallations: [{
          id: "ready", name: "Chriz Easy BG", path: "D:\\Ready", status: "Ready to play",
          receiptPath: "D:\\Ready\\receipt.json", launchPath: "D:\\Ready\\InfinityLoader.exe",
          completedAtMillis: 1, available: true, resumable: false,
        }] });
      }
      override createDesktopShortcut(): Promise<{ path: string }> {
        return Promise.reject(new BackendCommandError({
          code: "desktop_shortcut_failed",
          message: "The desktop shortcut could not be created.",
          recovery_action: "Retry the shortcut from My installs.",
          technical_detail: "Desktop\\<example>.lnk: Access is denied. (os error 5)",
        }));
      }
    }
    const root = await openApp(new ShortcutFailureBackend());
    await userEvent.setup().click(getByRole(root, "button", { name: "Create desktop shortcut" }));
    await waitFor(() => expect(getByText(root, "Your game is ready to play. Only the optional desktop shortcut failed.")).toBeTruthy());
    expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Retry desktop shortcut" })).toBeTruthy();
    const details = root.querySelector<HTMLDetailsElement>(".shortcut-feedback .error-details");
    expect(details?.open).toBe(false);
    expect(details?.textContent).toContain("Desktop\\<example>.lnk: Access is denied. (os error 5)");
    expect(details?.querySelector("example")).toBeNull();
    expect(root.querySelector(".shortcut-feedback")?.classList.contains("danger")).toBe(false);
  });
});

// @vitest-environment jsdom

import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { CommandErrorPayload, RunEventEnvelope } from "../src/contracts";

const folderFailure: CommandErrorPayload = {
  code: "unsafe_target",
  message: "The install folder could not be used.",
  recovery_action: "Return to setup and choose another empty install folder.",
  technical_detail: "C:\\Restricted\\My game: could not create folder: Access is denied. (os error 5)",
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

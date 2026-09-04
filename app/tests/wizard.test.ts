// @vitest-environment jsdom

import { getByLabelText, getByRole, getByText, queryByText, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { BackendCommandError, FixtureBackend, NativeBackend, type EventChannelFactory, type InvokeCommand } from "../src/backend";
import type { GameCandidate, GameRole, ManagedInstallation, RunEventEnvelope } from "../src/contracts";
import { mountApp } from "../src/app";
import { technicalLog } from "../src/components/technical-log";

function managedInstallation(overrides: Partial<ManagedInstallation> = {}): ManagedInstallation {
  return {
    id: "ready",
    name: "Chriz Easy BG",
    path: "D:\\Installations\\Chriz Easy BG",
    status: "Ready to play",
    receiptPath: "D:\\Installations\\Chriz Easy BG\\install-receipt.json",
    launchPath: "D:\\Installations\\Chriz Easy BG\\InfinityLoader.exe",
    completedAtMillis: 1_788_451_200_000,
    available: true,
    resumable: false,
    recipeVersion: "0.1.0-alpha.1",
    ...overrides,
  };
}

class ShortcutCompletionBackend extends FixtureBackend {
  registryReads = 0;
  shortcutCalls: string[] = [];
  failNextShortcut = false;

  override getStatus() {
    return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
  }

  override listManagedInstallations() {
    this.registryReads += 1;
    return Promise.resolve(this.registryReads === 1 ? [] : [managedInstallation({ id: "completed" })]);
  }

  override startBuild(_reviewToken: string, onEvent: (event: RunEventEnvelope) => void) {
    queueMicrotask(() => {
      onEvent({ runId: "shortcut-run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "completed", resumed: false } });
      onEvent({ runId: "shortcut-run", sequenceAsString: "2", event: { type: "campaign_finished", install_id: "completed" } });
    });
    return Promise.resolve({ runId: "shortcut-run" });
  }

  override createDesktopShortcut(installId: string) {
    this.shortcutCalls.push(installId);
    if (this.failNextShortcut) {
      this.failNextShortcut = false;
      return Promise.reject(new Error("Desktop access denied"));
    }
    return Promise.resolve({ path: "C:\\Users\\Chris\\Desktop\\Chriz Easy BG.lnk" });
  }
}

describe("Chriz Easy BG application flow", () => {
  it("tries Radar once after completion and keeps Play available if the addon fails", async () => {
    class AddonFailureBackend extends ShortcutCompletionBackend {
      radarCalls: string[] = [];
      override installRadar(installId: string): Promise<void> {
        this.radarCalls.push(installId);
        return Promise.reject(new Error("Release download is offline"));
      }
    }
    const backend = new AddonFailureBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByText(root, "BG Radar Overlay couldn't be added. Your game is ready; retry the overlay from Updates.")).toBeTruthy());
    expect(backend.radarCalls).toEqual(["completed"]);
    expect((getByRole(root, "button", { name: "Play Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);
    expect(backend.shortcutCalls).toEqual(["completed"]);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    document.body.replaceChildren();
    window.localStorage.clear();
  });

  it("customizes the recommended setup and completes the fixture installation", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy();

    const bg1 = getByLabelText(root, "Baldur's Gate source") as HTMLSelectElement;
    const bg2 = getByLabelText(root, "Baldur's Gate II source") as HTMLSelectElement;
    const originalBg2 = bg2.value;
    await user.selectOptions(bg1, "bg1-modified");
    expect(bg2.value).toBe(originalBg2);
    expect(getByText(root, "Files differ from a clean store installation.")).toBeTruthy();
    await user.selectOptions(bg1, "bg1-fresh");
    await user.click(getByRole(root, "button", { name: "Customize" }));
    expect(getByRole(root, "heading", { level: 1, name: "Customize your installation" })).toBeTruthy();
    const mandatory = getByRole(root, "checkbox", { name: /Curated foundation/ }) as HTMLInputElement;
    const recommended = getByRole(root, "checkbox", { name: /Recommended rules balance/ }) as HTMLInputElement;
    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement;
    const blocked = getByRole(root, "checkbox", { name: /Experimental quest restoration/ });
    expect(mandatory.checked).toBe(true);
    expect(mandatory.getAttribute("aria-disabled")).toBe("true");
    expect(recommended.checked).toBe(true);
    expect(recommended.hasAttribute("aria-disabled")).toBe(false);
    expect(optional.checked).toBe(false);
    expect(optional.hasAttribute("aria-disabled")).toBe(false);
    expect(blocked.getAttribute("aria-disabled")).toBe("true");
    expect(getByText(root, "Deferred until its installer can be reproduced safely.")).toBeTruthy();
    expect(getByText(root, "Experimental quest restoration remains visible but is omitted.")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Done" }));
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    expect(getByRole(root, "heading", { level: 1, name: "Installation progress" })).toBeTruthy();
    expect(root.querySelectorAll("[data-ledger-phase]")).toHaveLength(5);
    expect(getByText(root, "Manual archive needed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "I added the archive" }));
    expect(getByText(root, "Your attention is needed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue build" }));
    expect(getByText(root, "A fixture step failed")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Export diagnostics" }));
    expect(getByText(root, /Diagnostics exported to/)).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Retry failed step" }));
    expect(getByText(root, "Build in progress")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Finish fixture build" }));

    expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
  });

  it("surfaces every non-fresh source reason", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());

    const cases = [
      ["bg1-modified", "Files differ from a clean store installation."],
      ["bg1-old", "The installed game version is not supported."],
      ["bg1-nosod", "Siege of Dragonspear is required for this EET recipe."],
    ] as const;
    for (const [id, message] of cases) {
      await user.selectOptions(getByLabelText(root, "Baldur's Gate source"), id);
      expect(getByText(root, message)).toBeTruthy();
    }
    await user.selectOptions(getByLabelText(root, "Baldur's Gate II source"), "bg2-store");
    expect(getByText(root, "This storefront layout has not been verified yet.")).toBeTruthy();
  });

  it("bounds the selectable technical tail and allows auto-scroll to be paused", async () => {
    let logState = { paused: false, open: false };
    const log = technicalLog(
      Array.from({ length: 250 }, (_, index) => `line-${index}`),
      logState,
      (state) => { logState = state; },
    );
    document.body.append(log);
    const output = log.querySelector("pre");
    expect(output?.tabIndex).toBe(0);
    expect(output?.textContent?.split("\n")).toHaveLength(200);
    expect(output?.textContent).not.toContain("line-0\n");
    const pause = getByRole(log, "button", { name: "Pause auto-scroll" });
    await userEvent.setup().click(pause);
    expect(pause.getAttribute("aria-pressed")).toBe("true");
    expect(pause.textContent).toBe("Resume auto-scroll");
    expect(logState.paused).toBe(true);
    const restored = technicalLog(["later line"], logState);
    expect(getByRole(restored, "button", { name: "Resume auto-scroll" })).toBeTruthy();
  });

  it("shows returning managed installations and the separate Updates screen", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend({ managedInstallations: [managedInstallation()] }));

    expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy();
    expect(getByRole(root, "heading", { level: 2, name: "Chriz Easy BG" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Updates" }));
    expect(getByRole(root, "heading", { level: 1, name: "Updates" })).toBeTruthy();
    expect(getByText(root, "CEBG app")).toBeTruthy();
    expect(getByRole(root, "button", { name: "Back" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Check for updates" })).toBeTruthy();
    expect(queryByText(root, "Update now")).toBeNull();
  });

  it("creates or refreshes a desktop shortcut from an existing launcher", async () => {
    class ExistingInstallBackend extends FixtureBackend {
      shortcuts: string[] = [];

      constructor() {
        super({ managedInstallations: [managedInstallation()] });
      }

      override createDesktopShortcut(installId: string) {
        this.shortcuts.push(installId);
        return Promise.resolve({ path: "C:\\Users\\Chris\\Desktop\\Chriz Easy BG.lnk" });
      }
    }
    const backend = new ExistingInstallBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);

    await user.click(getByRole(root, "button", { name: "Create desktop shortcut" }));
    expect(backend.shortcuts).toEqual(["ready"]);
    expect(getByText(root, "Shortcut created on your desktop.")).toBeTruthy();
    expect(getByRole(root, "button", { name: "Recreate desktop shortcut" })).toBeTruthy();
  });

  it("defaults the completion shortcut on and creates it only after the install is available", async () => {
    const backend = new ShortcutCompletionBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);

    const shortcut = getByRole(root, "checkbox", { name: "Create desktop shortcut when finished" }) as HTMLInputElement;
    expect(shortcut.checked).toBe(true);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    await waitFor(() => expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy());
    expect(backend.registryReads).toBe(3);
    expect(backend.shortcutCalls).toEqual(["completed"]);
    expect(getByText(root, "Shortcut created on your desktop.")).toBeTruthy();
    expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
  });

  it("does not create the completion shortcut when the default is unchecked", async () => {
    const backend = new ShortcutCompletionBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);

    await user.click(getByRole(root, "checkbox", { name: "Create desktop shortcut when finished" }));
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    await waitFor(() => expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy());
    expect(backend.shortcutCalls).toEqual([]);
    expect(getByRole(root, "button", { name: "Create desktop shortcut" })).toBeTruthy();
  });

  it("keeps Play available when shortcut creation fails and lets the player retry", async () => {
    const backend = new ShortcutCompletionBackend();
    backend.failNextShortcut = true;
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);

    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByText(root, "The desktop shortcut wasn't created.")).toBeTruthy());

    expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Retry desktop shortcut" }));
    await waitFor(() => expect(getByText(root, "Shortcut created on your desktop.")).toBeTruthy());
    expect(backend.shortcutCalls).toEqual(["completed", "completed"]);
  });

  it("keeps app, recipe, and managed-copy updates separate and rebuild-only", async () => {
    class UpdateCenterBackend extends FixtureBackend {
      installedAppVersions: string[] = [];
      activatedRecipeVersions: string[] = [];

      override getUpdates() {
        return Promise.resolve({
          checkedAt: "2026-09-04T12:00:00Z",
          networkState: "online" as const,
          application: {
            state: "available" as const,
            currentVersion: "0.1.0-alpha.1",
            availableVersion: "0.1.0-alpha.2",
            detail: "A signed application update is ready.",
          },
          recipe: {
            state: "available" as const,
            currentVersion: "0.1.0-alpha.1",
            availableVersion: "0.1.0-alpha.2",
            disposition: "deferred-for-next-playthrough" as const,
            detail: "Useful for your next playthrough; your current campaign stays unchanged.",
            changes: [{
              title: "Viconia class correction",
              summary: "Uses the corrected class when she joins a newly built campaign.",
              saveApplicability: "before-npc-join" as const,
              urgency: "recommended" as const,
              conditionNote: "This guidance is authored for games where Viconia has not joined; the installer did not inspect your save.",
            }],
          },
          managedCopies: [
            {
              installId: "fixture-install",
              name: "Chriz EET — Stream test",
              path: "D:\\Fixture Campaigns\\Chriz EET Stream Test",
              installedRecipeVersion: "0.1.0-alpha.1",
              state: "update-available" as const,
              detail: "Build an updated copy to use recipe 0.1.0-alpha.2.",
            },
            {
              installId: "moved-install",
              name: "Moved campaign",
              path: "D:\\Fixture Campaigns\\Moved",
              installedRecipeVersion: "0.1.0-alpha.1",
              state: "stale" as const,
              detail: "The registered folder moved or changed; no update action is available.",
            },
          ],
        });
      }

      override installAppUpdate(version: string) {
        this.installedAppVersions.push(version);
        return Promise.resolve();
      }

      override activateRecipeUpdate(version: string) {
        this.activatedRecipeVersions.push(version);
        return Promise.resolve();
      }
    }

    const backend = new UpdateCenterBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Updates" }));

    expect(getByRole(root, "button", { name: "CEBG app 0.1.0-alpha.2 changelog" })).toBeTruthy();
    const collectionChangelog = getByRole(root, "button", { name: "Collection 0.1.0-alpha.2 changelog" });
    expect(collectionChangelog.getAttribute("aria-expanded")).toBe("false");
    await user.click(collectionChangelog);
    expect(collectionChangelog.getAttribute("aria-expanded")).toBe("true");
    expect(getByText(root, /installer did not inspect your save/)).toBeTruthy();
    expect(getByText(root, "Moved campaign")).toBeTruthy();
    expect(queryByText(root, "Patch campaign")).toBeNull();
    expect(queryByText(root, "Update installation")).toBeNull();
    expect(queryByText(root, "Uninstall mod")).toBeNull();

    await user.click(getByRole(root, "button", { name: "Install application update" }));
    expect(backend.installedAppVersions).toEqual(["0.1.0-alpha.2"]);
    await user.click(getByRole(root, "button", { name: "Create updated installation" }));
    expect(backend.activatedRecipeVersions).toEqual(["0.1.0-alpha.2"]);
    expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy();
  });

  it("shows offline and rejected update checks without destructive actions", async () => {
    class OfflineUpdateBackend extends FixtureBackend {
      override getUpdates() {
        return Promise.resolve({
          checkedAt: "2026-09-03T08:30:00Z",
          networkState: "offline" as const,
          application: {
            state: "offline" as const,
            currentVersion: "0.1.0-alpha.1",
            availableVersion: null,
            detail: "Could not reach the application channel. Last checked 2026-09-03 08:30 UTC.",
          },
          recipe: {
            state: "invalid" as const,
            currentVersion: "0.1.0-alpha.1",
            availableVersion: null,
            disposition: "unknown-applicability" as const,
            detail: "The downloaded recipe signature was invalid. The trusted recipe was kept.",
            changes: [],
          },
          managedCopies: [],
        });
      }
    }

    const root = document.createElement("div");
    document.body.append(root);
    const handle = await mountApp(root, new OfflineUpdateBackend());
    await handle.navigate("updates");

    expect(getByText(root, /Last checked 2026-09-03 08:30 UTC/)).toBeTruthy();
    expect(getByText(root, /signature was invalid.*trusted recipe was kept/i)).toBeTruthy();
    expect(queryByText(root, "Install application update")).toBeNull();
    expect(queryByText(root, "Create updated installation")).toBeNull();
  });

  it("loads a ready installation before discovery and keeps Play independent from updates", async () => {
    class ReadyBackend extends FixtureBackend {
      calls: string[] = [];
      launched: string[] = [];
      opened: string[] = [];

      override getStatus() {
        this.calls.push("status");
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override listManagedInstallations() {
        this.calls.push("registry");
        return Promise.resolve([managedInstallation()]);
      }

      override discoverGames(): Promise<never> {
        this.calls.push("discovery");
        return Promise.reject(new Error("The old source games are offline."));
      }

      override getUpdates(): Promise<never> {
        this.calls.push("updates");
        return Promise.reject(new Error("Updates are offline."));
      }

      override launchInstall(installId: string) {
        this.launched.push(installId);
        return Promise.resolve();
      }

      override openInstallFolder(installId: string) {
        this.opened.push(installId);
        return Promise.resolve();
      }
    }

    const backend = new ReadyBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);

    expect(backend.calls).toEqual(["status", "registry", "updates"]);
    expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Open game folder" })).toBeTruthy();
    const details = getByText(root, "Installation details").closest("details") as HTMLDetailsElement;
    expect(details.open).toBe(false);

    await user.click(getByRole(root, "button", { name: "Play Chriz Easy BG" }));
    await user.click(getByRole(root, "button", { name: "Open game folder" }));
    expect(backend.launched).toEqual(["ready"]);
    expect(backend.opened).toEqual(["ready"]);
    expect(backend.calls.filter((call) => call === "updates")).toHaveLength(1);
    expect(backend.calls).not.toContain("discovery");
  });

  it("opens a resumable-only registry on Continue installation", async () => {
    class ResumableBackend extends FixtureBackend {
      resumed: string[] = [];

      constructor() {
        super({ managedInstallations: [managedInstallation({
          id: "interrupted",
          name: "Interrupted install",
          status: "Build interrupted — ready to resume",
          receiptPath: null,
          launchPath: null,
          completedAtMillis: null,
          available: false,
          resumable: true,
        })] });
      }

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override resumeBuild(installId: string, _onEvent: (event: RunEventEnvelope) => void) {
        this.resumed.push(installId);
        return Promise.resolve({ runId: "resume-run" });
      }
    }

    const backend = new ResumableBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);

    expect(getByRole(root, "button", { name: "Continue installation" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Continue installation" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(backend.resumed).toEqual(["interrupted"]);
    expect(getByRole(root, "heading", { level: 1, name: "Installation progress" })).toBeTruthy();
  });

  it("shows a stale-only registry as missing with a path to a new installation", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    await mountApp(root, new FixtureBackend({ managedInstallations: [managedInstallation({
      id: "missing",
      name: "Moved install",
      status: "Folder unavailable",
      receiptPath: null,
      launchPath: null,
      completedAtMillis: null,
      available: false,
      resumable: false,
    })] }));

    expect(getByRole(root, "heading", { level: 1, name: "Installation not found" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "New installation" })).toBeTruthy();
  });

  it("remembers a valid selected installation and otherwise falls back to the newest ready one", async () => {
    const older = managedInstallation({ id: "older", name: "Older install", completedAtMillis: 100 });
    const newer = managedInstallation({ id: "newer", name: "Newest install", completedAtMillis: 200 });
    window.localStorage.setItem("cebg.last-install-id", "older");
    const firstRoot = document.createElement("div");
    document.body.append(firstRoot);
    const user = userEvent.setup();
    await mountApp(firstRoot, new FixtureBackend({ managedInstallations: [newer, older] }));

    expect(getByRole(firstRoot, "heading", { level: 2, name: "Older install" })).toBeTruthy();
    expect((getByLabelText(firstRoot, "Switch install") as HTMLSelectElement).value).toBe("older");
    await user.selectOptions(getByLabelText(firstRoot, "Switch install"), "newer");
    expect(getByRole(firstRoot, "heading", { level: 2, name: "Newest install" })).toBeTruthy();
    expect(window.localStorage.getItem("cebg.last-install-id")).toBe("newer");

    window.localStorage.setItem("cebg.last-install-id", "no-longer-present");
    const secondRoot = document.createElement("div");
    document.body.append(secondRoot);
    await mountApp(secondRoot, new FixtureBackend({ managedInstallations: [older, newer] }));

    expect(getByRole(secondRoot, "heading", { level: 2, name: "Newest install" })).toBeTruthy();
    expect((getByLabelText(secondRoot, "Switch install") as HTMLSelectElement).value).toBe("newer");
  });

  it("uses only a valid startup shortcut hint before the remembered install", async () => {
    const older = managedInstallation({ id: "older", name: "Shortcut install", completedAtMillis: 100 });
    const newer = managedInstallation({ id: "newer", name: "Remembered install", completedAtMillis: 200 });
    window.localStorage.setItem("cebg.last-install-id", "newer");
    class StartupBackend extends FixtureBackend {
      constructor(private readonly startupInstallId: string) {
        super({ managedInstallations: [newer, older] });
      }

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: this.startupInstallId });
      }
    }
    const firstRoot = document.createElement("div");
    document.body.append(firstRoot);
    await mountApp(firstRoot, new StartupBackend("older"));
    expect(getByRole(firstRoot, "heading", { level: 2, name: "Shortcut install" })).toBeTruthy();

    const secondRoot = document.createElement("div");
    document.body.append(secondRoot);
    await mountApp(secondRoot, new StartupBackend("not-in-the-registry"));
    expect(getByRole(secondRoot, "heading", { level: 2, name: "Remembered install" })).toBeTruthy();
  });

  it("still opens the launcher when local storage is unavailable", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("storage blocked"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("storage blocked"); });
    const root = document.createElement("div");
    document.body.append(root);

    await mountApp(root, new FixtureBackend({ managedInstallations: [managedInstallation()] }));

    expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy();
  });

  it("keeps a completed build visible when its launcher registry refresh fails", async () => {
    class RefreshFailureBackend extends FixtureBackend {
      #registryReads = 0;

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override listManagedInstallations() {
        this.#registryReads += 1;
        if (this.#registryReads > 1) return Promise.reject(new Error("registry unavailable"));
        return Promise.resolve([managedInstallation({
          id: "interrupted",
          status: "Build interrupted — ready to resume",
          receiptPath: null,
          launchPath: null,
          completedAtMillis: null,
          available: false,
          resumable: true,
        })]);
      }

      override resumeBuild(installId: string, onEvent: (event: RunEventEnvelope) => void) {
        queueMicrotask(() => {
          onEvent({ runId: "resume-run", sequenceAsString: "1", event: { type: "campaign_started", install_id: installId, resumed: true } });
          onEvent({ runId: "resume-run", sequenceAsString: "2", event: { type: "campaign_finished", install_id: installId } });
        });
        return Promise.resolve({ runId: "resume-run" });
      }
    }

    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new RefreshFailureBackend());
    await user.click(getByRole(root, "button", { name: "Continue installation" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByRole(root, "heading", { level: 1, name: "Installation progress" })).toBeTruthy();
    expect(getByRole(root, "alert").textContent).toContain("launcher record could not be refreshed");
  });

  it("does not let a late evaluation overwrite a newer selection", async () => {
    const backend = new FixtureBackend({ evaluationDelays: [0, 80, 0] });
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const handle = await mountApp(root, backend);
    await handle.navigate("setup");

    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ });
    await user.click(optional);
    await user.click(optional);
    await new Promise((resolve) => window.setTimeout(resolve, 100));

    expect((optional as HTMLInputElement).checked).toBe(false);
    expect(getByText(root, "2 choices included")).toBeTruthy();
  });

  it("uses native folder choices and immediately shows their validated results", async () => {
    class FolderChoiceBackend extends FixtureBackend {
      override chooseGameFolder(role: GameRole): Promise<GameCandidate | null> {
        return Promise.resolve({
          id: `chosen-${role}`,
          label: role === "bgee_sod" ? "BG:EE + SoD — chosen clean folder" : "BGII:EE — chosen clean folder",
          path: role === "bgee_sod" ? "C:\\Chosen BGEE" : "C:\\Chosen BG2EE",
          storefront: "steam",
          build: "2.7.3.0",
          freshness: "fresh",
          eligible: true,
          findings: ["Clean supported installation."],
        });
      }

      override chooseDestinationFolder(): Promise<{
        readonly path: string;
        readonly safe: boolean;
        readonly title: string;
        readonly detail: string;
      } | null> {
        return Promise.resolve({
          path: "D:\\Chosen Campaign",
          safe: true,
          title: "Safe separate destination",
          detail: "The source games and their saves will remain untouched.",
        });
      }
    }

    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FolderChoiceBackend());

    await user.click(getByRole(root, "button", { name: "Change Baldur's Gate source" }));
    expect(getByText(root, "C:\\Chosen BGEE")).toBeTruthy();
    expect((getByLabelText(root, "Baldur's Gate source") as HTMLSelectElement).value).toBe("chosen-bgee_sod");

    await user.click(getByRole(root, "button", { name: "Change install location" }));
    expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toBe("D:\\Chosen Campaign");
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("uses the native discovery, destination, review, and event path without fixture controls", async () => {
    const calls: string[] = [];
    let emitNative: ((event: unknown) => void) | undefined;
    let registryReads = 0;
    const channelFactory: EventChannelFactory = (onMessage) => {
      emitNative = onMessage;
      return { nativeChannel: true };
    };
    const selection = { platform: "windows", features: {}, inputs: {} };
    const evaluation = {
      view: { categories: ["Foundation"], controls: [] },
      normalized_selection: selection,
      findings: [],
      plan: { phases: [{ id: "main", title: "Build main", detail: "Install the reviewed recipe." }] },
      selected_choice_count: 0,
    };
    const invoke: InvokeCommand = async (command, args) => {
      calls.push(command);
      switch (command) {
        case "bootstrap": return { mode: "native", engine_version: "0.1.0", recipe_version: null, startup_install_id: null };
        case "installation_defaults": return { name: "Chriz Easy BG", path: "D:\\Native Campaign" };
        case "list_managed_installations":
          registryReads += 1;
          return registryReads === 1 ? [] : [{
            id: "native-install",
            name: "Chriz Easy BG",
            path: "D:\\Native Campaign",
            status: "Ready to play",
            receipt_path: "D:\\Native Campaign\\install-receipt.json",
            launch_path: "D:\\Native Campaign\\InfinityLoader.exe",
            completed_at_millis: 1_788_451_200_000,
            available: true,
            resumable: false,
            recipe_version: "0.1.0-alpha.1",
          }];
        case "discover_games": return {
          bg1_candidates: [{ id: "native-bg1", label: "BG1 clean", path: "C:\\BG1", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: [] }],
          bg2_candidates: [{ id: "native-bg2", label: "BG2 clean", path: "C:\\BG2", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: [] }],
          selected_bg1_id: "native-bg1",
          selected_bg2_id: "native-bg2",
        };
        case "evaluate_build": return evaluation;
        case "inspect_destination":
          expect(args).toEqual({ path: "D:\\Native Campaign", bg1CandidateId: "native-bg1", bg2CandidateId: "native-bg2" });
          return { path: "D:\\Native Campaign", safe: true, title: "Ready", detail: "Isolated." };
        case "freeze_review": return { review_token: "native-review", digest: "11".repeat(32), display_name: "Chriz Easy BG", destination: "D:\\Native Campaign", game_labels: ["BG1 clean", "BG2 clean"], evaluation };
        case "start_build": return { run_id: "native-run" };
        case "launch_install":
        case "open_install_folder":
          expect(args).toEqual({ installId: "native-install" });
          return null;
        default: throw new Error(`Unexpected native command ${command}`);
      }
    };
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();

    await mountApp(root, new NativeBackend(invoke, channelFactory));

    expect(calls.slice(0, 2)).toEqual(["bootstrap", "list_managed_installations"]);
    expect(calls.indexOf("discover_games")).toBeGreaterThan(calls.indexOf("list_managed_installations"));
    expect(calls).toEqual(expect.arrayContaining(["inspect_destination", "list_managed_installations", "evaluate_build"]));
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    expect(getByRole(root, "heading", { level: 1, name: "Installation progress" })).toBeTruthy();
    expect(queryByText(root, "Finish fixture build")).toBeNull();
    expect(getByRole(root, "button", { name: "Cancel build" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "My installs" }));
    expect(getByRole(root, "heading", { level: 1, name: "Installation progress" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Cancel build" })).toBeTruthy();
    (emitNative as (event: RunEventEnvelope | unknown) => void)({
      run_id: "native-run",
      sequence_as_string: "1",
      event: { type: "campaign_finished", install_id: "native-install" },
    });
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(registryReads).toBe(2);
    expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Play Chriz Easy BG" }));
    await user.click(getByRole(root, "button", { name: "Open game folder" }));
    expect(calls.slice(-2)).toEqual(["launch_install", "open_install_folder"]);
  });

  it("bounds log-only renders during native event bursts and renders manual action immediately", async () => {
    class BurstBackend extends FixtureBackend {
      listener: ((event: RunEventEnvelope) => void) | null = null;

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override startBuild(_reviewToken: string, onEvent: (event: RunEventEnvelope) => void) {
        this.listener = onEvent;
        return Promise.resolve({ runId: "run-burst" });
      }
    }

    const backend = new BurstBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    const emit = backend.listener as (event: RunEventEnvelope) => void;
    const replaceChildren = vi.spyOn(root, "replaceChildren");

    for (let sequence = 1; sequence <= 250; sequence += 1) {
      emit({
        runId: "run-burst",
        sequenceAsString: String(sequence),
        event: { type: "step_progress", id: "acquire:eet", done: sequence * 65_536, total: 64_000_000 },
      });
    }
    expect(replaceChildren).not.toHaveBeenCalled();

    await user.click(getByText(root, "Technical log"));
    replaceChildren.mockClear();
    for (let sequence = 251; sequence <= 500; sequence += 1) {
      emit({
        runId: "run-burst",
        sequenceAsString: String(sequence),
        event: { type: "step_progress", id: "acquire:eet", done: sequence * 65_536, total: 64_000_000 },
      });
    }
    expect(replaceChildren).not.toHaveBeenCalled();
    await new Promise((resolve) => window.setTimeout(resolve, 120));
    expect(replaceChildren).toHaveBeenCalledTimes(1);
    expect(root.querySelector("pre")?.textContent).toContain("500: acquire:eet");

    replaceChildren.mockClear();
    emit({
      runId: "run-burst",
      sequenceAsString: "501",
      event: { type: "step_progress", id: "acquire:bggo", done: 65_536, total: 1_000_000 },
    });
    emit({
      runId: "run-burst",
      sequenceAsString: "502",
      event: {
        type: "manual_download_needed",
        mod_id: "evandra",
        page: "https://example.invalid/evandra",
        expected_sha256: "22".repeat(32),
        drop_dir: "installer-owned cache",
      },
    });
    expect(getByText(root, "Manual archive needed")).toBeTruthy();
    expect(replaceChildren).toHaveBeenCalledTimes(1);
    await new Promise((resolve) => window.setTimeout(resolve, 120));
    expect(replaceChildren).toHaveBeenCalledTimes(1);
  });

  it("selects a requested manual archive and resumes the failed native build", async () => {
    class ManualArchiveBackend extends FixtureBackend {
      openedSources: string[] = [];
      suppliedArtifacts: string[] = [];
      resumedInstalls: string[] = [];

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override openManualSource(artifactId: string) {
        this.openedSources.push(artifactId);
        return Promise.resolve();
      }

      override startBuild(_reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        queueMicrotask(() => {
          onEvent({ runId: "run-manual", sequenceAsString: "1", event: { type: "campaign_started", install_id: "install-manual", resumed: false } });
          onEvent({ runId: "run-manual", sequenceAsString: "2", event: { type: "manual_download_needed", mod_id: "manual-fixture", page: "https://example.invalid/manual-fixture", expected_sha256: "22".repeat(32), drop_dir: "installer-owned cache" } });
          onEvent({ runId: "run-manual", sequenceAsString: "3", event: { type: "step_finished", id: "acquire:manual-fixture", outcome: "failed" } });
          onEvent({ runId: "run-manual", sequenceAsString: "4", event: { type: "error", step_id: null, message: "The required manual archive is not available yet." } });
        });
        return Promise.resolve({ runId: "run-manual" });
      }

      override getRunSnapshot(runId: string) {
        return Promise.resolve({
          runId,
          status: "failed",
          events: [],
          report: {
            install_id: "install-manual",
            managed_root: "D:\\Manual Campaign",
            plan_sha256: "11".repeat(32),
            status: { status: "failed", step_id: "acquire:manual-fixture", reason: "manual archive missing" },
          },
          error: null,
        });
      }

      override supplyManualArchive(artifactId: string) {
        this.suppliedArtifacts.push(artifactId);
        return Promise.resolve({ artifactId, filename: "manual-fixture.zip", sha256: "22".repeat(32), length: 4_096 });
      }

      override resumeBuild(installId: string, onEvent: Parameters<FixtureBackend["resumeBuild"]>[1]) {
        this.resumedInstalls.push(installId);
        queueMicrotask(() => {
          onEvent({ runId: "run-manual-resumed", sequenceAsString: "1", event: { type: "campaign_started", install_id: installId, resumed: true } });
        });
        return Promise.resolve({ runId: "run-manual-resumed" });
      }
    }

    const backend = new ManualArchiveBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByText(root, "Manual archive needed")).toBeTruthy();
    expect(getByText(root, /https:\/\/example\.invalid\/manual-fixture/)).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Open download page" }));
    expect(backend.openedSources).toEqual(["manual-fixture"]);
    await user.click(getByRole(root, "button", { name: "Choose downloaded archive" }));

    expect(backend.suppliedArtifacts).toEqual(["manual-fixture"]);
    expect(backend.resumedInstalls).toEqual(["install-manual"]);
    expect(getByText(root, "Resuming installation")).toBeTruthy();
  });

  it("surfaces a rejected destination and does not retain the previous safe state", async () => {
    class RejectingDestinationBackend extends FixtureBackend {
      #calls = 0;

      override inspectDestination(path: string, bg1CandidateId: string, bg2CandidateId: string) {
        this.#calls += 1;
        if (this.#calls === 1) return super.inspectDestination(path, bg1CandidateId, bg2CandidateId);
        return Promise.reject(new BackendCommandError({
          code: "destination_unsafe",
          message: "That folder is occupied.",
          recovery_action: "Choose a new empty folder.",
          technical_detail: "fixture rejection",
        }));
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new RejectingDestinationBackend());

    const destination = getByLabelText(root, "Install location") as HTMLInputElement;
    await user.clear(destination);
    await user.type(destination, "D:\\Occupied");
    destination.dispatchEvent(new Event("change", { bubbles: true }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByRole(root, "status").textContent).toContain("Choose a new empty folder.");
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("does not start when the server-frozen review differs from what was displayed", async () => {
    class ChangedReviewBackend extends FixtureBackend {
      started = false;

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override async freezeReview(displayName: string, selection: Parameters<FixtureBackend["freezeReview"]>[1], destination: string, bg1CandidateId: string, bg2CandidateId: string) {
        const review = await super.freezeReview(displayName, selection, destination, bg1CandidateId, bg2CandidateId);
        return { ...review, evaluation: { ...review.evaluation, selectedChoiceCount: review.evaluation.selectedChoiceCount + 1 } };
      }

      override startBuild(reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        this.started = true;
        return super.startBuild(reviewToken, onEvent);
      }
    }
    const backend = new ChangedReviewBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(backend.started).toBe(false);
    expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(getByRole(root, "alert").textContent).toContain("Read the refreshed Review");
  });

  it("lets the player leave an unrecoverable failed build after its terminal snapshot", async () => {
    class UnrecoverableBuildBackend extends FixtureBackend {
      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override startBuild(_reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        queueMicrotask(() => {
          onEvent({ runId: "run-broken", sequenceAsString: "1", event: { type: "campaign_started", install_id: "install-broken", resumed: false } });
          onEvent({ runId: "run-broken", sequenceAsString: "2", event: { type: "error", step_id: null, message: "The worker ended before it could create a durable report." } });
        });
        return Promise.resolve({ runId: "run-broken" });
      }

      override getRunSnapshot(runId: string) {
        return Promise.resolve({
          runId,
          status: "failed" as const,
          events: [],
          report: null,
          error: {
            code: "build_worker_failed",
            message: "The worker ended unexpectedly.",
            recovery_action: "Return to Campaigns and start with a new destination.",
            technical_detail: "fixture worker panic",
          },
        });
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new UnrecoverableBuildBackend());
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByText(root, "The build stopped safely")).toBeTruthy();
    expect(queryByText(root, "Retry failed step")).toBeNull();
    await user.click(getByRole(root, "button", { name: "My installs" }));
    expect(getByRole(root, "heading", { level: 1, name: "My installs" })).toBeTruthy();
  });

  it("preserves a failed native snapshot and Retry when resume is rejected", async () => {
    class RejectingResumeBackend extends FixtureBackend {
      diagnosticInstalls: string[] = [];

      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }

      override startBuild(_reviewToken: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        queueMicrotask(() => {
          onEvent({ runId: "run-failed", sequenceAsString: "1", event: { type: "campaign_started", install_id: "install-failed", resumed: false } });
          onEvent({ runId: "run-failed", sequenceAsString: "2", event: { type: "error", step_id: null, message: "The build stopped safely for this test." } });
        });
        return Promise.resolve({ runId: "run-failed" });
      }

      override getRunSnapshot(runId: string) {
        return Promise.resolve({
          runId,
          status: "failed",
          events: [],
          report: {
            install_id: "install-failed",
            managed_root: "D:\\Failed Campaign",
            plan_sha256: "11".repeat(32),
            status: { status: "failed", step_id: "install:test", reason: "fixture failure" },
          },
          error: null,
        });
      }

      override resumeBuild() {
        return Promise.reject(new BackendCommandError({
          code: "build_already_running",
          message: "The build cannot resume yet.",
          recovery_action: "Keep the failed copy and try Retry again.",
          technical_detail: "fixture rejection",
        }));
      }

      override exportDiagnostics(installId: string) {
        this.diagnosticInstalls.push(installId);
        return Promise.resolve({ path: "D:\\Diagnostics\\install-failed.zip" });
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    const backend = new RejectingResumeBackend();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    const retry = getByRole(root, "button", { name: "Retry failed step" });

    await user.click(getByRole(root, "button", { name: "Export diagnostics" }));
    expect(backend.diagnosticInstalls).toEqual(["install-failed"]);
    expect(getByText(root, /Diagnostics exported to D:\\Diagnostics\\install-failed\.zip/)).toBeTruthy();

    await user.click(retry);
    await new Promise((resolve) => window.setTimeout(resolve, 0));

    expect(getByText(root, "The build stopped safely")).toBeTruthy();
    expect(getByRole(root, "button", { name: "Retry failed step" })).toBeTruthy();
    expect(getByRole(root, "alert").textContent).toContain("Keep the failed copy");
  });
});

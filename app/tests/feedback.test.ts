// @vitest-environment jsdom

import { getByRole, getByText, queryByText, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend, NativeBackend } from "../src/backend";
import type { ManagedInstallation, NormalizedSelection } from "../src/contracts";

describe("player feedback regressions", () => {
  it("checks updates once after native startup without blocking the installer", async () => {
    class StartupBackend extends FixtureBackend {
      checks = 0;
      override async getStatus() { return { ...await super.getStatus(), mode: "native" as const }; }
      override async getUpdates() { this.checks += 1; return super.getUpdates(); }
    }
    const backend = new StartupBackend();
    const root = document.createElement("div");
    document.body.append(root);
    await mountApp(root, backend);
    await waitFor(() => expect(backend.checks).toBe(1));
    expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
  });
  it("switches mod setups using fresh defaults and keeps the install ready", async () => {
    class ProfilesBackend extends FixtureBackend {
      profile = "public-alpha";
      selections: NormalizedSelection[] = [];
      override async getStatus() {
        return { ...await super.getStatus(), selectedProfile: this.profile, profiles: [
          { id: "public-alpha", label: "Public alpha", description: "Released collection" },
          { id: "creator-full-current", label: "Chriz's full setup", description: "Full stream setup" },
        ] };
      }
      override selectProfile(profileId: string) { this.profile = profileId; return this.getStatus(); }
      override evaluateBuild(selection: NormalizedSelection) {
        this.selections.push(selection);
        return super.evaluateBuild(selection);
      }
    }
    const backend = new ProfilesBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Customize" }));
    await user.click(getByRole(root, "checkbox", { name: "Companion conversations" }));
    await user.click(getByRole(root, "button", { name: "Done" }));
    await user.selectOptions(getByRole(root, "combobox", { name: "Mod setup" }), "creator-full-current");
    await waitFor(() => expect(backend.profile).toBe("creator-full-current"));
    expect(backend.selections.at(-1)?.features).toEqual({});
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);
  });

  afterEach(() => document.body.replaceChildren());

  it("returns from Updates and My installs without resetting the install form", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());
    const name = getByRole(root, "textbox", { name: "Install name" }) as HTMLInputElement;
    name.value = "Tonight";
    name.dispatchEvent(new Event("change", { bubbles: true }));
    await waitFor(() => expect((getByRole(root, "textbox", { name: "Install name" }) as HTMLInputElement).value).toBe("Tonight"));
    for (const label of ["Updates", "My installs"]) {
      await user.click(getByRole(root, "button", { name: label }));
      await user.click(getByRole(root, "button", { name: "Back" }));
      expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
      expect((getByRole(root, "textbox", { name: "Install name" }) as HTMLInputElement).value).toBe("Tonight");
    }
  });

  it("shows compact update versions without internal release configuration", async () => {
    class AlphaBackend extends FixtureBackend {
      checks = 0;
      override async getUpdates() {
        this.checks += 1;
        const result = await super.getUpdates();
        return {
          ...result, networkState: "unconfigured" as const,
          application: { ...result.application, state: "unavailable" as const, detail: "The application update key and endpoint must be supplied at release-time." },
          recipe: { ...result.recipe, state: "unavailable" as const, detail: "No signed channel is configured; the bundled trusted recipe was kept." },
        };
      }
    }
    const backend = new AlphaBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    await user.click(getByRole(root, "button", { name: "Updates" }));
    expect(getByText(root, "CEBG app")).toBeTruthy();
    expect(getByText(root, "Collection")).toBeTruthy();
    expect(root.textContent).toContain("0.1.0-alpha.1");
    expect(root.textContent).not.toMatch(/key and endpoint|signed channel|Your current game stays safe/);
    await user.click(getByRole(root, "button", { name: "Check for updates" }));
    expect(backend.checks).toBe(2);
  });

  it("shows the real terminal error and returns an early failure to setup", async () => {
    class FailedBackend extends FixtureBackend {
      override getStatus() {
        return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: null, startupInstallId: null });
      }
      override startBuild(_token: string, onEvent: Parameters<FixtureBackend["startBuild"]>[1]) {
        queueMicrotask(() => onEvent({ runId: "early", sequenceAsString: "4", event: {
          type: "error", step_id: null, message: "The installer could not complete that check.",
        } }));
        return Promise.resolve({ runId: "early" });
      }
      override getRunSnapshot(runId: string) {
        return Promise.resolve({ runId, status: "failed", events: [], report: null, error: {
          code: "campaign_error", message: "The installation could not finish.",
          recovery_action: "Return to setup and try again.",
          technical_detail: "Artifact identity dlcmerger-2.1 was rejected.",
        } });
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FailedBackend());
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(root.querySelector("pre")?.textContent).toContain("Artifact identity dlcmerger-2.1 was rejected."));
    expect(getByText(root, "Return to setup and try again.")).toBeTruthy();
    expect(queryByText(root, "Retry failed step")).toBeNull();
    await user.click(getByRole(root, "button", { name: "Back to setup" }));
    expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
  });

  it("keeps customization search and selection when a choice is changed", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new FixtureBackend());
    await user.click(getByRole(root, "button", { name: "Customize" }));
    await user.type(getByRole(root, "searchbox", { name: "Find a mod or option" }), "Companion");
    await user.click(getByRole(root, "checkbox", { name: "Companion conversations" }));
    await waitFor(() => expect((getByRole(root, "checkbox", { name: "Companion conversations" }) as HTMLInputElement).checked).toBe(true));
    expect((getByRole(root, "searchbox") as HTMLInputElement).value).toBe("Companion");
    await user.selectOptions(getByRole(root, "combobox", { name: "Category" }), "Foundation");
    expect(getByText(root, "No matching choices. Try another search or category.").hidden).toBe(false);
    await user.click(getByRole(root, "button", { name: "Done" }));
    expect(getByRole(root, "heading", { name: "Install Chriz Easy BG" })).toBeTruthy();
  });

  it("shows changed mods without blocking Play and installs Radar for the selected game", async () => {
    const installation: ManagedInstallation = {
      id: "stream", name: "Stream game", path: "D:\\CEBG", status: "Ready to play", receiptPath: null,
      launchPath: "D:\\CEBG\\InfinityLoader.exe", completedAtMillis: 1, available: true, resumable: false,
      recipeVersion: "0.1.0-alpha.1", radarVersion: null,
      consistency: { state: "changed", detail: "WeiDU.log changed after setup.", componentCount: 250, modCount: 80 },
    };
    class RadarBackend extends FixtureBackend {
      installed: string[] = [];
      constructor() { super({ managedInstallations: [installation] }); }
      override async getUpdates() {
        return { ...await super.getUpdates(), radar: { state: "not-installed" as const, currentVersion: null,
          availableVersion: "1.2.3", detail: "Available", releaseNotes: "Improved overlay controls." } };
      }
      override installRadar(installId: string) {
        this.installed.push(installId);
        return Promise.resolve();
      }
    }
    const backend = new RadarBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    expect(getByText(root, "Mods changed since installation")).toBeTruthy();
    expect((getByRole(root, "button", { name: "Play Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);
    await user.click(getByRole(root, "button", { name: "Updates" }));
    await user.click(getByRole(root, "button", { name: "BG Radar Overlay 1.2.3 changelog" }));
    expect(getByText(root, "Improved overlay controls.").closest<HTMLElement>(".update-changelog")?.hidden).toBe(false);
    await user.click(getByRole(root, "button", { name: "Install overlay" }));
    await waitFor(() => expect(backend.installed).toEqual(["stream"]));
  });

  it("maps native application version, mod checks and Radar fields", async () => {
    const calls: unknown[] = [];
    const backend = new NativeBackend(async (command, args) => {
      if (command === "bootstrap") return { mode: "native", application_version: "0.1.0-alpha.2", engine_version: "0.1.0", recipe_version: "0.1.0-alpha.1", startup_install_id: null };
      if (command === "list_managed_installations") return [{ id: "game", name: "Game", path: "D:\\Game", status: "Ready", receipt_path: null,
        launch_path: "D:\\Game\\InfinityLoader.exe", completed_at_millis: 1, available: true, resumable: false, recipe_version: "1",
        radar_version: "1.2.3", consistency: { state: "matches", detail: "All recorded components match.", component_count: 250, mod_count: 80 } }];
      if (command === "install_radar") { calls.push(args); return null; }
      throw new Error(command);
    });
    expect((await backend.getStatus()).applicationVersion).toBe("0.1.0-alpha.2");
    expect((await backend.listManagedInstallations())[0]).toMatchObject({ radarVersion: "1.2.3", consistency: { state: "matches", componentCount: 250, modCount: 80 } });
    await backend.installRadar("game");
    expect(calls).toEqual([{ installId: "game" }]);
  });
});

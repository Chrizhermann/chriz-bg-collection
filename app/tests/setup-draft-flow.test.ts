// @vitest-environment jsdom

import { fireEvent, getByLabelText, getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { BackendStatus, GameRole, ManagedInstallation, NormalizedSelection, RunEventEnvelope } from "../src/contracts";
import { saveSetupDraft, SETUP_DRAFT_STORAGE_KEY, type SetupDraft } from "../src/setup-draft";

function savedDraft(): SetupDraft {
  return {
    version: 1, recipeVersion: "recipe-12", profileId: "full",
    installationName: "My adventure", destinationPath: "D:\\My adventure", destinationAutomatic: false,
    sourcePaths: { bg1: "D:\\Saved BG1", bg2: "E:\\Saved BG2" },
    selection: { platform: "windows", features: { "companion-conversations": true }, inputs: {} },
    createDesktopShortcut: false,
  };
}

function completedInstall(path = "D:\\My adventure"): ManagedInstallation {
  return {
    id: "ready", name: "Ready game", path, status: "Ready to play", receiptPath: `${path}\\receipt.json`,
    launchPath: `${path}\\InfinityLoader.exe`, available: true, resumable: false,
    completedAtMillis: 10, recipeVersion: "recipe-12", radarVersion: "1",
  };
}

class DraftBackend extends FixtureBackend {
  profile = "recommended";
  recipeVersion = "recipe-12";
  installations: ManagedInstallation[] = [];
  discoveryCalls = 0;
  inspectedSources: Array<{ role: GameRole; path: string }> = [];
  inspectedDestinations: Array<{ path: string; bg1: string; bg2: string }> = [];
  starts: string[] = [];
  reviews: Array<{ selection: NormalizedSelection; destination: string; bg1: string; bg2: string }> = [];
  sourceUnavailable = false;
  destinationOccupied = false;
  rejectStart = false;
  rejectChoices = false;
  listener: ((event: RunEventEnvelope) => void) | null = null;

  override getStatus(): Promise<BackendStatus> {
    return Promise.resolve({
      mode: "native", engineVersion: "0.1.0", recipeVersion: this.recipeVersion, startupInstallId: null,
      selectedProfile: this.profile,
      profiles: [
        { id: "recommended", label: "Recommended", description: "Recommended choices" },
        { id: "full", label: "Full", description: "Full choices" },
      ],
    });
  }

  override selectProfile(profileId: string) { this.profile = profileId; return this.getStatus(); }
  override listManagedInstallations() { return Promise.resolve(this.installations); }
  override discoverGames() { this.discoveryCalls += 1; return super.discoverGames(); }

  override async inspectGamePath(role: GameRole, path: string) {
    this.inspectedSources.push({ role, path });
    if (this.sourceUnavailable && role === "bgee_sod") throw new Error("Source drive is disconnected");
    return { ...await super.inspectGamePath(role, path), id: `rechecked-${role}`, eligible: true, findings: [] };
  }

  override async inspectDestination(path: string, bg1: string, bg2: string) {
    this.inspectedDestinations.push({ path, bg1, bg2 });
    return { ...await super.inspectDestination(path, bg1, bg2), safe: !this.destinationOccupied };
  }

  override evaluateBuild(selection: NormalizedSelection) {
    if (this.rejectChoices && Object.keys(selection.features).length > 0) return Promise.reject(new Error("Unknown saved choice"));
    return super.evaluateBuild(selection);
  }

  override async freezeReview(name: string, selection: NormalizedSelection, destination: string, bg1: string, bg2: string) {
    this.reviews.push({ selection, destination, bg1, bg2 });
    return { ...await super.freezeReview(name, selection, destination, bg1, bg2), reviewToken: `new-review-${this.reviews.length}` };
  }

  override startBuild(token: string, onEvent: (event: RunEventEnvelope) => void) {
    this.starts.push(token);
    this.listener = onEvent;
    if (this.rejectStart) return Promise.reject(new BackendCommandError({
      code: "unsafe_target", message: "This location could not be created.",
      recovery_action: "Choose another folder.", technical_detail: "Access denied while claiming destination",
    }));
    return Promise.resolve({ runId: "draft-run" });
  }

  override getRunSnapshot(runId: string) {
    return Promise.resolve({ runId, status: "failed", events: [], report: null, error: {
      code: "unsafe_target", message: "This location could not be created.",
      recovery_action: "Choose another folder.", technical_detail: "Access denied while claiming destination",
    } });
  }

  emit(event: RunEventEnvelope["event"]) {
    this.listener?.({ runId: "draft-run", sequenceAsString: "1", event });
  }
}

async function open(backend = new DraftBackend()) {
  const root = document.createElement("div");
  document.body.append(root);
  const app = await mountApp(root, backend);
  return { root, app, backend, user: userEvent.setup() };
}

afterEach(() => {
  vi.restoreAllMocks();
  document.body.replaceChildren();
  window.localStorage.clear();
});

describe("setup drafts across application restarts", () => {
  it("remembers edited choices, manually selected sources, name, location, profile, and shortcut preference", async () => {
    const first = await open();
    await first.user.click(getByRole(first.root, "button", { name: "Change Baldur's Gate source" }));
    await first.user.selectOptions(getByRole(first.root, "combobox", { name: "Mod setup" }), "full");
    await first.user.click(getByRole(first.root, "button", { name: "Customize" }));
    await first.user.click(getByRole(first.root, "checkbox", { name: "Companion conversations" }));
    await first.user.click(getByRole(first.root, "button", { name: "Done" }));
    fireEvent.change(getByLabelText(first.root, "Install name"), { target: { value: "My adventure" } });
    await waitFor(() => expect((getByLabelText(first.root, "Install location") as HTMLInputElement).value).toContain("My adventure"));
    fireEvent.change(getByLabelText(first.root, "Install location"), { target: { value: "D:\\My adventure" } });
    await waitFor(() => expect((getByRole(first.root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false));
    await first.user.click(getByRole(first.root, "checkbox", { name: "Create desktop shortcut when finished" }));
    first.root.remove();

    const second = await open();
    expect((getByLabelText(second.root, "Install name") as HTMLInputElement).value).toBe("My adventure");
    expect((getByLabelText(second.root, "Install location") as HTMLInputElement).value).toBe("D:\\My adventure");
    expect((getByRole(second.root, "combobox", { name: "Mod setup" }) as HTMLSelectElement).value).toBe("full");
    expect((getByRole(second.root, "checkbox", { name: "Create desktop shortcut when finished" }) as HTMLInputElement).checked).toBe(false);
    expect(second.backend.inspectedSources).toEqual([
      { role: "bgee_sod", path: "C:\\Fixture\\Browsed BGEE" },
      { role: "bg2ee", path: "C:\\Fixture\\BG2EE" },
    ]);
    expect(second.backend.inspectedDestinations.at(-1)).toEqual({ path: "D:\\My adventure", bg1: "rechecked-bgee_sod", bg2: "rechecked-bg2ee" });
    expect(second.backend.starts).toEqual([]);
    expect(second.backend.reviews).toEqual([]);
    await second.user.click(getByRole(second.root, "button", { name: "Customize" }));
    expect((getByRole(second.root, "checkbox", { name: "Companion conversations" }) as HTMLInputElement).checked).toBe(true);
  });

  it("restores the automatic location behavior when the saved name changes", async () => {
    saveSetupDraft({ ...savedDraft(), destinationAutomatic: true });
    const { root } = await open();
    fireEvent.change(getByLabelText(root, "Install name"), { target: { value: "Next adventure" } });
    await waitFor(() => expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toBe("C:\\Users\\Chris\\Games\\Next adventure"));
  });

  it("validates a saved alternate profile against that profile's recipe version", async () => {
    saveSetupDraft({ ...savedDraft(), recipeVersion: "full-recipe-4" });
    class AlternateRecipeBackend extends DraftBackend {
      override async getStatus() {
        return { ...await super.getStatus(), recipeVersion: this.profile === "full" ? "full-recipe-4" : "recommended-recipe-12" };
      }
    }
    const { root, backend } = await open(new AlternateRecipeBackend());
    expect(backend.profile).toBe("full");
    expect((getByLabelText(root, "Install name") as HTMLInputElement).value).toBe("My adventure");
    expect(backend.starts).toEqual([]);
  });

  it("does not substitute a discovered source when the saved source cannot be checked", async () => {
    saveSetupDraft(savedDraft());
    const backend = new DraftBackend();
    backend.sourceUnavailable = true;
    const { root, user } = await open(backend);
    const source = getByRole(root, "combobox", { name: "Baldur's Gate source" }) as HTMLSelectElement;
    expect(source.value).toBe("");
    expect(getByRole(source, "option", { name: "Choose source" })).toBeTruthy();
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(true);
    expect(backend.inspectedDestinations).toEqual([]);
    expect(root.textContent).toContain("D:\\Saved BG1");
    await user.selectOptions(source, "bg1-fresh");
    await waitFor(() => expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false));
  });

  it("checks the restored destination again and leaves an occupied location blocked", async () => {
    saveSetupDraft(savedDraft());
    const backend = new DraftBackend();
    backend.destinationOccupied = true;
    const { root } = await open(backend);
    expect(backend.inspectedDestinations.at(-1)?.path).toBe("D:\\My adventure");
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(true);
    expect(backend.starts).toEqual([]);
  });

  it.each(["direct rejection", "background failure"])("preserves edited setup after %s and creates a fresh review only after another click", async (failure) => {
    saveSetupDraft(savedDraft());
    const backend = new DraftBackend();
    backend.rejectStart = failure === "direct rejection";
    const first = await open(backend);
    await first.user.click(getByRole(first.root, "button", { name: "Install Chriz Easy BG" }));
    if (failure === "background failure") {
      backend.emit({ type: "error", step_id: null, message: "Destination creation failed" });
      await waitFor(() => expect(getByText(first.root, "This location could not be created.")).toBeTruthy());
    } else {
      await waitFor(() => expect(getByRole(first.root, "alert")).toBeTruthy());
      backend.rejectStart = false;
      await first.user.click(getByRole(first.root, "button", { name: "Install Chriz Easy BG" }));
      expect(backend.starts).toEqual(["new-review-1", "new-review-2"]);
    }
    first.root.remove();
    const second = await open();
    expect((getByLabelText(second.root, "Install name") as HTMLInputElement).value).toBe("My adventure");
    expect(second.backend.starts).toEqual([]);
    expect(second.backend.reviews).toEqual([]);
    await second.user.click(getByRole(second.root, "button", { name: "Install Chriz Easy BG" }));
    expect(second.backend.starts).toEqual(["new-review-1"]);
    expect(second.backend.reviews[0].selection.features["companion-conversations"]).toBe(true);
    expect(second.backend.reviews[0].destination).toBe("D:\\My adventure");
  });

  it("keeps the managed launcher first and restores draft preferences only on New installation", async () => {
    saveSetupDraft(savedDraft());
    const backend = new DraftBackend();
    backend.installations = [completedInstall("D:\\Other completed game")];
    const { root, user } = await open(backend);
    expect(getByRole(root, "heading", { name: "Ready to play" })).toBeTruthy();
    expect(backend.discoveryCalls).toBe(0);
    expect(backend.inspectedSources).toEqual([]);
    await user.click(getByRole(root, "button", { name: "New installation" }));
    expect((getByLabelText(root, "Install name") as HTMLInputElement).value).toBe("My adventure");
    expect(backend.starts).toEqual([]);
  });

  it("does not apply incompatible or engine-rejected saved choices silently", async () => {
    saveSetupDraft(savedDraft());
    const incompatible = new DraftBackend();
    incompatible.recipeVersion = "recipe-13";
    const first = await open(incompatible);
    expect(first.root.textContent).toContain("saved setup belongs to a different collection version");
    expect((getByLabelText(first.root, "Install name") as HTMLInputElement).value).toBe("Chriz Easy BG");
    expect(incompatible.starts).toEqual([]);
    first.root.remove();

    saveSetupDraft(savedDraft());
    const rejected = new DraftBackend();
    rejected.rejectChoices = true;
    const second = await open(rejected);
    expect(second.root.textContent).toContain("saved choices could not be checked");
    expect((getByLabelText(second.root, "Install name") as HTMLInputElement).value).toBe("My adventure");
    expect(rejected.starts).toEqual([]);
  });

  it("keeps storage errors nonfatal during startup and edits", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("Storage unavailable"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("Storage unavailable"); });
    const { root } = await open();
    fireEvent.change(getByLabelText(root, "Install name"), { target: { value: "Still editable" } });
    await waitFor(() => expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toContain("Still editable"));
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("clears the matching draft after confirmed success", async () => {
    saveSetupDraft(savedDraft());
    const { backend, root, user } = await open();
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    expect(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)).not.toBeNull();
    backend.installations = [completedInstall()];
    backend.emit({ type: "campaign_finished", install_id: "ready" });
    await waitFor(() => expect(getByRole(root, "heading", { name: "Ready to play" })).toBeTruthy());
    expect(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)).toBeNull();
  });
});

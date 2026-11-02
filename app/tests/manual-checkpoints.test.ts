// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, getByLabelText, getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { mountApp } from "../src/app";
import { FixtureBackend, NativeBackend, type InvokeCommand } from "../src/backend";
import type { ManualBreakpoint, ManualCheckpointOption, ManualCheckpointStatus, ManagedInstallation, NormalizedSelection, RunEventEnvelope } from "../src/contracts";
import { invalidBreakpoints, manualBreakpointEditor, manualCheckpointPanel } from "../src/components/manual-checkpoints";
import { installationReadiness } from "../src/state";
import { loadSetupDraft, saveSetupDraft, SETUP_DRAFT_STORAGE_KEY } from "../src/setup-draft";
import { updatesScreen } from "../src/screens/updates";
import { setupScreen } from "../src/screens/setup";

const options: ManualCheckpointOption[] = [
  { runId: "bg1-batch", modId: "bg1npc", modLabel: "BG1 NPC Project", component: 0, componentLabel: "Dialogue", phase: "bg1-preparation", target: "bg1", unavailableReason: null },
  { runId: "eeex-batch", modId: "eeex", modLabel: "EEex", component: 1, componentLabel: "Core", phase: "main", target: "bg2", unavailableReason: null },
  { runId: "eeex-batch", modId: "eeex", modLabel: "EEex", component: 8, componentLabel: "LuaJIT", phase: "main", target: "bg2", unavailableReason: null },
  { runId: "indivisible", modId: "example", modLabel: "Grouped installer", component: 0, componentLabel: "First component", phase: "main", target: "bg2", unavailableReason: "This installer must finish its component group before pausing." },
];
const status: ManualCheckpointStatus = {
  checkpointId: "manual-1", target: "bg2", targetPath: "D:\\Managed Game\\BG2", after: options[1], next: options[2],
  additions: [], diagnostics: [], reviewToken: "", customized: false,
};
const addition = { tp2: "example/setup-example.tp2", language: 0, component: 12, description: "Example option: v1" };

class CheckpointBackend extends FixtureBackend {
  paused = true;
  pending: ManualCheckpointStatus | null = status;
  reviewed: ManualCheckpointStatus = { ...status, reviewToken: "review-1", additions: [addition] };
  readError = false;
  checkError = false;
  acceptError = false;
  statusLabel = "Paused for your mods";
  reads: string[] = [];
  checks: string[] = [];
  accepts: unknown[][] = [];
  resumes: string[] = [];
  folders: string[] = [];
  frozen: readonly ManualBreakpoint[] = [];
  emit?: (event: RunEventEnvelope) => void;

  override getStatus() { return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: "recipe-1", startupInstallId: null }); }
  override listManagedInstallations(): Promise<ManagedInstallation[]> {
    return Promise.resolve(this.paused ? [{ id: "manual-install", name: "Adventure", path: "D:\\Managed Game", status: this.statusLabel,
      available: false, resumable: true, receiptPath: null, launchPath: null, completedAtMillis: null }] : []);
  }
  override async evaluateBuild(selection: NormalizedSelection) {
    return { ...await super.evaluateBuild(selection), manualCheckpointOptions: options };
  }
  override async freezeReview(name: string, selection: NormalizedSelection, destination: string, bg1: string, bg2: string, breakpoints?: readonly ManualBreakpoint[]) {
    this.frozen = breakpoints ?? [];
    return super.freezeReview(name, selection, destination, bg1, bg2);
  }
  override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
    this.emit = onEvent;
    return Promise.resolve({ runId: "manual-run" });
  }
  async getManualCheckpoint(id: string) {
    this.reads.push(id);
    if (this.readError) throw new Error("Checkpoint could not be read");
    return this.pending;
  }
  async checkManualCheckpoint(id: string) {
    this.checks.push(id);
    if (this.checkError) throw new Error("Managed installation history changed; restore the recorded entries");
    return this.reviewed;
  }
  async acceptManualCheckpoint(id: string, token: string, acceptAdditions: boolean) {
    this.accepts.push([id, token, acceptAdditions]);
    if (this.acceptError) throw new Error("Files changed after review. Check changes again.");
    this.pending = null;
  }
  async openManualCheckpointFolder(id: string) { this.folders.push(id); }
  override resumeBuild(id: string, onEvent: (event: RunEventEnvelope) => void) {
    this.resumes.push(id);
    onEvent({ runId: "resumed", sequenceAsString: "1", event: { type: "campaign_started", install_id: id, resumed: true } });
    return Promise.resolve({ runId: "resumed" });
  }
}

afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); vi.restoreAllMocks(); });

describe("manual component boundaries", () => {
  it("starts collapsed and empty, searches selected components, and adds exact within-batch anchors", async () => {
    const change = vi.fn();
    const editor = manualBreakpointEditor({ options, selected: [], phaseTitles: { main: "Curated mods" }, onOpen: () => {}, onChange: change });
    document.body.append(editor);
    expect((editor as HTMLDetailsElement).open).toBe(false);
    expect(change).not.toHaveBeenCalled();
    (editor as HTMLDetailsElement).open = true;
    fireEvent.input(getByLabelText(editor, "Find a selected component"), { target: { value: "EEex" } });
    const select = getByLabelText(editor, "Pause after") as HTMLSelectElement;
    expect(select.options).toHaveLength(2);
    select.value = "1";
    fireEvent.change(select);
    getByRole(editor, "button", { name: "Add pause" }).click();
    expect(change).toHaveBeenCalledWith([{ runId: "eeex-batch", component: 1 }]);
  });

  it("explains indivisible boundaries and preserves invalid anchors until explicitly removed", () => {
    const change = vi.fn();
    const missing = { runId: "removed-mod", component: 20 };
    const editor = manualBreakpointEditor({ options, selected: [missing], phaseTitles: {}, open: true, onOpen: () => {}, onChange: change });
    document.body.append(editor);
    expect(getByRole(editor, "alert").textContent).toContain("no longer matches");
    fireEvent.input(getByLabelText(editor, "Find a selected component"), { target: { value: "Grouped" } });
    expect((getByRole(editor, "button", { name: "Add pause" }) as HTMLButtonElement).disabled).toBe(true);
    expect(getByRole(editor, "status").textContent).toContain("finish its component group");
    expect(change).not.toHaveBeenCalled();
    getByRole(editor, "button", { name: /Remove pause after removed-mod/ }).click();
    expect(change).toHaveBeenCalledWith([]);
    expect(invalidBreakpoints([missing], options)).toEqual([missing]);
  });

  it("blocks installation for stale selected boundaries", async () => {
    const backend = new CheckpointBackend();
    const games = await backend.discoverGames();
    const evaluation = await backend.evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    const readiness = { starting: false, name: "New game", bg1: games.bg1Candidates.find(game => game.eligible), bg2: games.bg2Candidates.find(game => game.eligible),
      destination: { path: "D:\\New", safe: true, title: "Ready", detail: "" }, evaluation, evaluationPending: false };
    expect(installationReadiness({ ...readiness, manualBreakpoints: [{ runId: "eeex-batch", component: 1 }] })).toBe(true);
    expect(installationReadiness({ ...readiness, manualBreakpoints: [{ runId: "gone", component: 1 }] })).toBe(false);
    expect(installationReadiness({ ...readiness, manualBreakpoints: [{ runId: "indivisible", component: 0 }] })).toBe(false);
  });

  it("retains multiple anchors across draft reload without storing review authority", async () => {
    const backend = new CheckpointBackend();
    const draft = { version: 1 as const, recipeVersion: "recipe-1", profileId: null, installationName: "New game", destinationPath: "D:\\New", destinationAutomatic: true,
      sourcePaths: { bg1: null, bg2: null }, selection: { platform: "windows", features: {}, inputs: {} }, createDesktopShortcut: false,
      manualBreakpoints: [{ runId: "bg1-batch", component: 0 }, { runId: "eeex-batch", component: 1 }] };
    saveSetupDraft({ ...draft, reviewToken: "secret" } as typeof draft);
    expect(loadSetupDraft(await backend.getStatus()).draft?.manualBreakpoints).toEqual(draft.manualBreakpoints);
    expect(window.localStorage.getItem(SETUP_DRAFT_STORAGE_KEY)).not.toContain("secret");
  });

  it("explains a blocked Done action even when the advanced editor is collapsed", async () => {
    const backend = new CheckpointBackend();
    const evaluation = await backend.evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    const screen = setupScreen(evaluation, vi.fn(), vi.fn(), vi.fn(), { search: "", category: "" }, {
      change: vi.fn(), reset: vi.fn(), manualBreakpoints: { options, selected: [{ runId: "removed", component: 10 }], onChange: vi.fn() },
    });
    expect((getByRole(screen, "button", { name: "Done" }) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.querySelector("#manual-pause-correction")?.textContent).toContain("Open Advanced: Add your own mods");
    expect((screen.querySelector(".manual-breakpoints") as HTMLDetailsElement).open).toBe(false);
  });
});

describe("manual checkpoint flow", () => {
  async function mounted(backend = new CheckpointBackend()) {
    const root = document.createElement("div"); document.body.append(root);
    await mountApp(root, backend);
    return { root, backend, user: userEvent.setup() };
  }

  it("reopens a durable pause by reading only, showing the exact folder and boundaries", async () => {
    const { root, backend, user } = await mounted();
    expect(getByText(root, "D:\\Managed Game\\BG2")).toBeTruthy();
    expect(getByText(root, "EEex: Core (1)")).toBeTruthy();
    expect(getByText(root, "EEex: LuaJIT (8) · BG2")).toBeTruthy();
    expect(backend.reads).toEqual(["manual-install"]);
    expect(backend.checks).toEqual([]); expect(backend.accepts).toEqual([]); expect(backend.resumes).toEqual([]);
    expect(queryByRole(root, "button", { name: "Resume installation" })).toBeNull();
    // A first manual pause has no terminal receipt to export yet.
    expect(queryByRole(root, "button", { name: "Export diagnostics" })).toBeNull();
    await user.click(getByRole(root, "button", { name: "Open game folder" }));
    expect(backend.folders).toEqual(["manual-install"]);
  });

  it("requires a separate acceptance of displayed additions before resuming", async () => {
    const { root, backend, user } = await mounted();
    await user.click(getByRole(root, "button", { name: "Check changes and continue" }));
    expect(getByText(root, "Example option: v1")).toBeTruthy();
    expect(document.activeElement?.id).toBe("manual-checkpoint-review");
    expect(backend.accepts).toEqual([]); expect(backend.resumes).toEqual([]);
    expect(root.textContent).toContain("disables game hotpatches and whole-mod updates");
    await user.click(getByRole(root, "button", { name: "Accept additions and continue" }));
    expect(backend.accepts).toEqual([["manual-install", "review-1", true]]);
    expect(backend.resumes).toEqual(["manual-install"]);
    expect(queryByRole(root, "button", { name: "Accept additions and continue" })).toBeNull();
  });

  it("checks an unchanged request and still waits for explicit continuation without additions", async () => {
    const backend = new CheckpointBackend(); backend.reviewed = { ...status, reviewToken: "empty-review" };
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Continue unchanged" }));
    expect(root.textContent).toContain("No manual additions found");
    expect(backend.accepts).toEqual([]); expect(backend.resumes).toEqual([]);
    await user.click(getByRole(root, "button", { name: "Continue unchanged" }));
    expect(backend.accepts).toEqual([["manual-install", "empty-review", false]]);
    expect(backend.resumes).toEqual(["manual-install"]);
  });

  it("accepts and resumes once while a checkpoint continuation is starting", async () => {
    let acknowledge!: (result: { runId: string }) => void;
    class DelayedCheckpointBackend extends CheckpointBackend {
      override resumeBuild(id: string, onEvent: (event: RunEventEnvelope) => void) {
        this.resumes.push(id);
        this.emit = onEvent;
        return new Promise<{ runId: string }>(resolve => { acknowledge = resolve; });
      }
    }
    const backend = new DelayedCheckpointBackend();
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Check changes and continue" }));
    const accept = getByRole(root, "button", { name: "Accept additions and continue" });
    accept.click(); accept.click();
    await waitFor(() => expect(backend.resumes).toEqual(["manual-install"]));
    expect(backend.accepts).toEqual([["manual-install", "review-1", true]]);
    expect((getByRole(root, "button", { name: "Pause after current mod" }) as HTMLButtonElement).disabled).toBe(true);
    backend.emit!({ runId: "resumed", sequenceAsString: "1", event: { type: "campaign_started", install_id: "manual-install", resumed: true } });
    backend.emit!({ runId: "resumed", sequenceAsString: "2", event: { type: "step_started", id: "install:bggo", label: "Install bggo" } });
    acknowledge({ runId: "resumed" });
    await waitFor(() => expect((getByRole(root, "button", { name: "Pause after current mod" }) as HTMLButtonElement).disabled).toBe(false));
    expect(getByText(root, "Install bggo")).toBeTruthy();
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("does not accept unexpected additions through Continue unchanged", async () => {
    const { root, backend, user } = await mounted();
    await user.click(getByRole(root, "button", { name: "Continue unchanged" }));
    expect(root.textContent).toContain("Manual additions were found");
    expect(backend.accepts).toEqual([]); expect(backend.resumes).toEqual([]);
    expect(queryByRole(root, "button", { name: "Continue unchanged" })).toBeNull();
  });

  it("keeps failures paused and consumes a failed acceptance token", async () => {
    const { root, backend, user } = await mounted();
    backend.checkError = true;
    await user.click(getByRole(root, "button", { name: "Check changes and continue" }));
    expect(root.textContent).toContain("Managed installation history changed");
    expect(backend.resumes).toEqual([]);
    backend.checkError = false;
    await user.click(getByRole(root, "button", { name: "Check changes and continue" }));
    backend.acceptError = true;
    await user.click(getByRole(root, "button", { name: "Accept additions and continue" }));
    expect(root.textContent).toContain("Files changed after review");
    expect(document.activeElement?.id).toBe("manual-checkpoint-error");
    expect(queryByRole(root, "button", { name: "Accept additions and continue" })).toBeNull();
    expect(queryByRole(root, "button", { name: "Resume installation" })).toBeNull();
    expect(backend.resumes).toEqual([]);
  });

  it("does not fall through to ordinary resume after a failed checkpoint read", async () => {
    const backend = new CheckpointBackend(); backend.readError = true;
    const { root, user } = await mounted(backend);
    expect(getByRole(root, "alert").textContent).toContain("Checkpoint could not be read");
    expect(queryByRole(root, "button", { name: "Resume installation" })).toBeNull();
    backend.readError = false;
    await user.click(getByRole(root, "button", { name: "Reload checkpoint" }));
    expect(getByText(root, "D:\\Managed Game\\BG2")).toBeTruthy();
    expect(backend.resumes).toEqual([]);
  });

  it("preserves ordinary explicit resume when no manual checkpoint exists", async () => {
    const backend = new CheckpointBackend(); backend.statusLabel = "Paused"; backend.pending = null;
    const { root, user } = await mounted(backend);
    expect(backend.reads).toEqual([]); expect(backend.resumes).toEqual([]);
    await user.click(getByRole(root, "button", { name: "Continue installation" }));
    expect(backend.reads).toEqual(["manual-install"]);
    expect(backend.checks).toEqual([]); expect(backend.accepts).toEqual([]);
    expect(backend.resumes).toEqual(["manual-install"]);
  });

  it("freezes chosen anchors and handles the manual pause event without reviewing or resuming", async () => {
    const backend = new CheckpointBackend(); backend.paused = false;
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Customize" }));
    await user.click(getByText(root, "Advanced: Add your own mods"));
    fireEvent.input(getByLabelText(root, "Find a selected component"), { target: { value: "EEex" } });
    await user.click(getByRole(root, "button", { name: "Add pause" }));
    await user.click(getByRole(root, "button", { name: "Done" }));
    expect(root.textContent).toContain("1 manual-mod pause selected");
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    expect(backend.frozen).toEqual([{ runId: "eeex-batch", component: 1 }]);
    backend.emit!({ runId: "manual-run", sequenceAsString: "2", event: { type: "campaign_paused", install_id: "manual-install", after_step_id: "checkpoint:manual-1" } });
    await waitFor(() => expect(getByText(root, "D:\\Managed Game\\BG2")).toBeTruthy());
    expect(backend.checks).toEqual([]); expect(backend.resumes).toEqual([]);
  });
});

describe("manual checkpoint contracts", () => {
  it("maps native options and sends explicit boundary, check and acceptance arguments", async () => {
    const selection = { platform: "windows", features: {}, inputs: {} };
    const evaluation = { view: { categories: [], controls: [] }, normalized_selection: selection, findings: [], plan: { phases: [] }, selected_choice_count: 0, manual_checkpoint_options: options };
    const invoke = vi.fn<InvokeCommand>(async command => {
      if (command === "evaluate_build") return evaluation;
      if (command === "freeze_review") return { review_token: "frozen", digest: "hash", display_name: "New", destination: "D:\\New", game_labels: [], evaluation };
      if (command === "get_manual_checkpoint" || command === "check_manual_checkpoint") return status;
      return undefined;
    });
    const backend = new NativeBackend(invoke);
    expect((await backend.evaluateBuild(selection)).manualCheckpointOptions).toEqual(options);
    await backend.freezeReview("New", selection, "D:\\New", "bg1", "bg2", [{ runId: "eeex-batch", component: 1 }]);
    expect(invoke).toHaveBeenLastCalledWith("freeze_review", expect.objectContaining({ manualBreakpoints: [{ runId: "eeex-batch", component: 1 }] }));
    await backend.getManualCheckpoint("install"); expect(invoke).toHaveBeenLastCalledWith("get_manual_checkpoint", { installId: "install" });
    await backend.checkManualCheckpoint("install"); expect(invoke).toHaveBeenLastCalledWith("check_manual_checkpoint", { installId: "install" });
    await backend.acceptManualCheckpoint("install", "review", false); expect(invoke).toHaveBeenLastCalledWith("accept_manual_checkpoint", { installId: "install", reviewToken: "review", acceptAdditions: false });
    await backend.openManualCheckpointFolder("install"); expect(invoke).toHaveBeenLastCalledWith("open_manual_checkpoint_folder", { installId: "install" });
  });

  it("suppresses game mutation controls for customized copies and preserves app updates", async () => {
    const updates = await new FixtureBackend().getUpdates();
    const appUpdate = vi.fn();
    const screen = updatesScreen({ ...updates, application: { ...updates.application, state: "available", availableVersion: "next" },
      managedCopies: [{ installId: "custom", name: "Custom", path: "D:\\Custom", installedRecipeVersion: "old", state: "up-to-date", detail: "" }] }, {
      customized: true, checkAgain: () => {}, installApplication: appUpdate, buildUpdatedCopy: () => {}, onCheckInstalledUpdate: vi.fn(), onCheckPatches: vi.fn(),
    });
    expect(screen.textContent).toContain("Game hotpatches and whole-mod updates are disabled");
    expect(queryByRole(screen, "button", { name: /Check installed mods|Check available fixes/ })).toBeNull();
    getByRole(screen, "button", { name: "Update CEBG app" }).click();
    expect(appUpdate).toHaveBeenCalledWith("next");
  });

  it("disables repeat actions while a checkpoint request is running", () => {
    const panel = manualCheckpointPanel({ status: { ...status, reviewToken: "token", additions: [addition] }, busy: true,
      check: vi.fn(), reload: vi.fn(), accept: vi.fn(), continueUnchanged: vi.fn(), openFolder: vi.fn() });
    expect([...panel.querySelectorAll("button")].every(button => button.disabled)).toBe(true);
    expect(panel.getAttribute("aria-busy")).toBe("true");
  });

  it("keeps raw diagnostics collapsed and explains the backup responsibility", () => {
    const panel = manualCheckpointPanel({ status: { ...status, diagnostics: ["Local TP2 SHA-256: example"] }, busy: false,
      check: vi.fn(), reload: vi.fn(), accept: vi.fn(), continueUnchanged: vi.fn(), openFolder: vi.fn() });
    const details = panel.querySelector(".manual-checkpoint-details") as HTMLDetailsElement;
    expect(details.open).toBe(false);
    expect(details.querySelector("summary")?.textContent).toBe("Technical details");
    expect(details.textContent).toContain("Local TP2 SHA-256");
    expect(panel.textContent).toContain("back up this managed game folder");
    expect(panel.textContent).toContain("does not automatically create a full checkpoint backup");
  });
});

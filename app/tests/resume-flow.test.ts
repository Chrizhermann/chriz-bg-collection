// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type { EngineEvent, ManualCheckpointStatus, RunEventEnvelope, RunSnapshot, StartBuildResponse } from "../src/contracts";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

class ResumeBackend extends FixtureBackend {
  checkpoint = Promise.resolve<ManualCheckpointStatus | null>(null);
  requests: { onEvent: (event: RunEventEnvelope) => void; result: ReturnType<typeof deferred<StartBuildResponse>> }[] = [];
  override getStatus() {
    return Promise.resolve({ mode: "native" as const, engineVersion: "test", recipeVersion: null, startupInstallId: null });
  }
  override listManagedInstallations() {
    return Promise.resolve([{ id: "existing", name: "Test install", path: "D:\\Test",
      status: "Build interrupted — ready to resume", available: false, resumable: true,
      receiptPath: null, launchPath: null, completedAtMillis: null }]);
  }
  getManualCheckpoint() { return this.checkpoint; }
  override resumeBuild(_installId: string, onEvent: (event: RunEventEnvelope) => void) {
    const result = deferred<StartBuildResponse>();
    this.requests.push({ onEvent, result });
    return result.promise;
  }
  override getRunSnapshot(runId: string): Promise<RunSnapshot> {
    return Promise.resolve({ runId, status: "failed", events: [], error: null,
      report: { install_id: "existing", managed_root: "D:\\Test", plan_sha256: "11".repeat(32),
        status: { status: "failed" as const, step_id: "install:infinity", reason: "Test failure" } } });
  }
  emit(index: number, event: EngineEvent) {
    this.requests[index].onEvent({ runId: `run-${index}`, sequenceAsString: "1", event });
  }
  accept(index = 0) { this.requests[index].result.resolve({ runId: `run-${index}` }); }
}

async function launch(backend = new ResumeBackend()) {
  const root = document.createElement("div");
  document.body.append(root);
  const app = await mountApp(root, backend);
  const resume = getByRole(root, "button", { name: "Continue installation" });
  resume.click();
  await waitFor(() => expect(backend.requests).toHaveLength(1));
  return { root, backend, resume, app };
}

afterEach(() => {
  document.body.replaceChildren();
  window.localStorage.clear();
  vi.restoreAllMocks();
});

describe("resuming an existing installation", () => {
  it("sends one request even when the old Resume action fires again before acknowledgement", async () => {
    const { root, backend, resume } = await launch();
    resume.click();
    resume.click();
    expect(backend.requests).toHaveLength(1);
    expect(getByText(root, "Preparing to continue…")).toBeTruthy();
    expect((getByRole(root, "button", { name: "Pause after current mod" }) as HTMLButtonElement).disabled).toBe(true);
    expect((getByRole(root, "button", { name: "Stop now (may need repair)" }) as HTMLButtonElement).disabled).toBe(true);

    backend.emit(0, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(0, { type: "step_started", id: "install:bggo", label: "Install bggo" });
    backend.accept();
    await waitFor(() => expect((getByRole(root, "button", { name: "Pause after current mod" }) as HTMLButtonElement).disabled).toBe(false));
    resume.click();
    expect(backend.requests).toHaveLength(1);
    expect(getByText(root, "Install bggo")).toBeTruthy();
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("locks resume before checking manual checkpoints and prevents navigation during launch", async () => {
    const backend = new ResumeBackend();
    const checkpoint = deferred<ManualCheckpointStatus | null>();
    backend.checkpoint = checkpoint.promise;
    const root = document.createElement("div");
    document.body.append(root);
    const app = await mountApp(root, backend);
    const resume = getByRole(root, "button", { name: "Continue installation" });
    resume.click(); resume.click();
    expect((getByRole(root, "button", { name: "Resume installation" }) as HTMLButtonElement).disabled).toBe(true);
    await app.navigate("home");
    expect(getByRole(root, "heading", { name: "Installation progress" })).toBeTruthy();
    expect(backend.requests).toHaveLength(0);
    checkpoint.resolve(null);
    await waitFor(() => expect(backend.requests).toHaveLength(1));
    backend.accept();
  });

  it("shows phases from resumed events without a new-install evaluation", async () => {
    const { root, backend } = await launch();
    backend.emit(0, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(0, { type: "phase_started", name: "Install the curated collection" });
    expect(getByText(root, "Install your mods")).toBeTruthy();
    expect(root.textContent).not.toContain("Installation progress is not available yet");
    expect(root.textContent).not.toContain("Prepare Baldur's Gate");
    backend.accept();
  });

  it("clears finished-step progress and keeps the new step when acknowledgement arrives late", async () => {
    const { root, backend } = await launch();
    backend.emit(0, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(0, { type: "step_started", id: "install:infinity", label: "Install infinity-ui" });
    backend.emit(0, { type: "step_progress", id: "install:infinity", done: 1, total: 1 });
    await waitFor(() => expect(root.textContent).toContain("100% of this step"));
    backend.emit(0, { type: "step_finished", id: "install:infinity", outcome: "succeeded" });
    expect(root.querySelector("progress")?.hasAttribute("value")).toBe(false);
    backend.emit(0, { type: "step_progress", id: "install:infinity", done: 1, total: 1 });
    expect(root.querySelector("progress")?.hasAttribute("value")).toBe(false);
    backend.emit(0, { type: "step_started", id: "install:bggo", label: "Install bggo" });
    backend.emit(0, { type: "step_progress", id: "install:infinity", done: 1, total: 1 });
    backend.accept();
    await waitFor(() => expect((getByRole(root, "button", { name: "Pause after current mod" }) as HTMLButtonElement).disabled).toBe(false));
    expect(getByText(root, "Install bggo")).toBeTruthy();
    expect(root.textContent).not.toContain("100% of this step");
  });

  it("preserves a real rejected-resume error and allows an explicit retry", async () => {
    const { root, backend } = await launch();
    backend.requests[0].result.reject(new BackendCommandError({ code: "build_already_running",
      message: "Another installer build is already running.", recovery_action: "Wait for that installation.", technical_detail: "test guard" }));
    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain("Another installer"));
    getByRole(root, "button", { name: "Resume installation" }).click();
    await waitFor(() => expect(backend.requests).toHaveLength(2));
    backend.emit(1, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.accept(1);
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("keeps an early failure retryable and ignores events from the previous attempt", async () => {
    const { root, backend } = await launch();
    backend.emit(0, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(0, { type: "error", step_id: null, message: "Test failure" });
    await waitFor(() => expect(getByRole(root, "button", { name: "Retry failed step" })).toBeTruthy());
    backend.accept();
    await waitFor(() => expect((getByRole(root, "button", { name: "Retry failed step" }) as HTMLButtonElement).disabled).toBe(false));
    const retry = getByRole(root, "button", { name: "Retry failed step" });
    retry.click(); retry.click();
    expect(backend.requests).toHaveLength(2);
    backend.emit(1, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(1, { type: "step_started", id: "install:bggo", label: "Install bggo" });
    backend.emit(0, { type: "error", step_id: null, message: "Late failure from old attempt" });
    backend.emit(0, { type: "step_started", id: "install:infinity", label: "Old step" });
    backend.accept(1);
    expect(getByText(root, "Install bggo")).toBeTruthy();
    expect(root.textContent).not.toContain("Late failure");
    expect(root.textContent).not.toContain("Old step");
    expect(root.querySelector(".failure-reason")).toBeNull();
    expect(queryByRole(root, "button", { name: "Retry failed step" })).toBeNull();
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("shows native artifact download progress without mistaking a cache hit for install progress", async () => {
    const { root, backend } = await launch();
    backend.emit(0, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(0, { type: "step_started", id: "acquire:mod-x", label: "Download mod-x" });
    backend.emit(0, { type: "step_progress", id: "mod-x", done: 50, total: 100 });
    await waitFor(() => expect(root.textContent).toContain("50% of this step"));
    backend.emit(0, { type: "step_finished", id: "acquire:mod-x", outcome: "succeeded" });
    backend.emit(0, { type: "step_started", id: "install:infinity", label: "Install infinity-ui" });
    backend.emit(0, { type: "step_progress", id: "weidu-249", done: 100, total: 100 });
    backend.accept();
    expect(root.querySelector("progress")?.hasAttribute("value")).toBe(false);
    expect(root.textContent).not.toContain("100% of this step");
  });

  it.each([false, true])("does not let an old pause check overwrite a new attempt (rejected: %s)", async rejected => {
    const backend = new ResumeBackend();
    const pause = deferred<Awaited<ReturnType<ResumeBackend["getRunSnapshot"]>>>();
    const snapshot = vi.spyOn(backend, "getRunSnapshot").mockReturnValue(pause.promise);
    const { root } = await launch(backend);
    backend.emit(0, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.accept();
    await waitFor(() => expect((getByRole(root, "button", { name: "Pause after current mod" }) as HTMLButtonElement).disabled).toBe(false));
    getByRole(root, "button", { name: "Pause after current mod" }).click();
    await waitFor(() => expect(snapshot).toHaveBeenCalledOnce());
    backend.emit(0, { type: "campaign_paused", install_id: "existing", after_step_id: "install:infinity" });
    getByRole(root, "button", { name: "Resume installation" }).click();
    await waitFor(() => expect(backend.requests).toHaveLength(2));
    backend.emit(1, { type: "campaign_started", install_id: "existing", resumed: true });
    backend.emit(1, { type: "step_started", id: "install:bggo", label: "Install bggo" });
    backend.accept(1);
    if (rejected) pause.reject(new Error("Old pause check failed"));
    else pause.resolve({ runId: "run-0", status: "paused", events: [], error: null, report: null });
    await new Promise(resolve => window.setTimeout(resolve, 0));
    expect(getByText(root, "Install bggo")).toBeTruthy();
    expect(queryByRole(root, "button", { name: "Resume installation" })).toBeNull();
    expect(queryByRole(root, "alert")).toBeNull();
  });
});

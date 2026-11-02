// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from "vitest";
import { fireEvent, getByRole, getByText, queryByRole, queryByText, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend, NativeBackend, type InvokeCommand } from "../src/backend";
import type { DecisionAnswer, ManagedInstallation, PendingDecision, RunEventEnvelope, RunSnapshot } from "../src/contracts";
import { downloadDecisionDialog, focusDownloadDecision } from "../src/components/download-decision";

// The texts as the engine renders them; the app only shows them.
const AFTER_NO = "You chose to stop. Resume this installation to check the download again.";
const changedOptional: PendingDecision = {
  installId: "waiting-install", stepId: "acquire:snowfall", decisionId: "decision-1", token: "token-1",
  kind: "changed_download", artifactId: "snowfall",
  message: "A newer version of Snowfall was found. The reviewed version is no longer available at this link. Install it anyway?"
    + " If you leave it out, the mods that depend on it are left out too: Frostbite."
    + " Without it, these are installed without the parts that need it: Tweaks Anthology.",
  details: ["File: snowfall-v2.zip", "Size: 99 bytes", "SHA-256: 1111", "From: downloads.example.test", "Reviewed version: v1 (98 bytes, SHA-256 0000)"],
  answers: [
    { answer: "yes", label: "Install it" },
    { answer: "leave_out", label: "Leave it out and continue" },
    { answer: "no", label: "Stop the installation" },
  ],
  focus: "yes", afterNo: AFTER_NO, answered: null,
};
const changedRequired: PendingDecision = {
  ...changedOptional,
  message: "A newer version of Snowfall was found. The reviewed version is no longer available at this link. Install it anyway?",
  answers: [{ answer: "yes", label: "Install it" }, { answer: "no", label: "Stop the installation" }],
};
const unusable: PendingDecision = {
  ...changedOptional, kind: "unusable_download",
  message: "Snowfall can't be installed (its official link is gone). It and the mods that depend on it (Frostbite) will be left out."
    + " Installed without the parts that need it: Tweaks Anthology. Continue the installation?",
  details: ["The official link answered 410 Gone."],
  answers: [{ answer: "yes", label: "Continue without them" }, { answer: "no", label: "Stop the installation" }],
};
const unreviewedTool: PendingDecision = {
  ...changedOptional, kind: "unreviewed_tool", stepId: "acquire:weidu", artifactId: "weidu",
  message: "A newer version of WeiDU was found. The reviewed version is no longer available at this link. Install it anyway?"
    + " It came from mirror.example.test, which the collection did not review. If you're not sure, stop the installation.",
  answers: [{ answer: "yes", label: "Install it" }, { answer: "no", label: "Stop the installation" }],
  focus: "no",
};

function buttonLabels(container: HTMLElement): string[] {
  return [...container.querySelectorAll(".decision-dialog button")].map(button => button.textContent ?? "");
}

function shown(decision: PendingDecision, overrides: { busy?: boolean; error?: string } = {}) {
  const answer = vi.fn<(answer: DecisionAnswer) => void>();
  const root = document.createElement("div");
  document.body.append(root);
  const model = { decision, busy: overrides.busy ?? false, error: overrides.error };
  root.append(downloadDecisionDialog({ ...model, answer }));
  focusDownloadDecision(root, model);
  return { root, answer };
}

afterEach(() => { document.body.replaceChildren(); window.localStorage.clear(); vi.restoreAllMocks(); });

describe("download question dialog", () => {
  it("(a) offers install, leave out and stop for an optional mod, with the leave-out sentences and focus on Install it", async () => {
    const { root, answer } = shown(changedOptional);
    const dialog = getByRole(root, "dialog", { name: "A download needs your answer" });
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(getByText(dialog, changedOptional.message)).toBeTruthy();
    expect(dialog.textContent).toContain("If you leave it out, the mods that depend on it are left out too: Frostbite.");
    expect(dialog.textContent).toContain("Without it, these are installed without the parts that need it: Tweaks Anthology.");
    expect(buttonLabels(root)).toEqual(["Install it", "Leave it out and continue", "Stop the installation"]);
    expect(document.activeElement?.textContent).toBe("Install it");
    const details = dialog.querySelector("details") as HTMLDetailsElement;
    expect(details.open).toBe(false);
    expect(details.querySelector("summary")?.textContent).toBe("Details");
    expect([...details.querySelectorAll("li")].map(item => item.textContent)).toEqual(changedOptional.details);
    await userEvent.setup().click(getByRole(dialog, "button", { name: "Leave it out and continue" }));
    expect(answer).toHaveBeenCalledWith("leave_out");
  });

  it("(a) offers no leave-out for a mod the collection cannot do without", () => {
    const { root } = shown(changedRequired);
    expect(buttonLabels(root)).toEqual(["Install it", "Stop the installation"]);
    expect(root.textContent).not.toContain("leave it out");
    expect(document.activeElement?.textContent).toBe("Install it");
  });

  it("(b) continues without the unusable mod and its dependents, focus on Continue", async () => {
    const { root, answer } = shown(unusable);
    expect(getByText(root, unusable.message)).toBeTruthy();
    expect(buttonLabels(root)).toEqual(["Continue without them", "Stop the installation"]);
    expect(document.activeElement?.textContent).toBe("Continue without them");
    await userEvent.setup().click(getByRole(root, "button", { name: "Stop the installation" }));
    expect(answer).toHaveBeenCalledWith("no");
  });

  it("(c) names the unreviewed host, advises stopping and focuses Stop", () => {
    const { root } = shown(unreviewedTool);
    expect(root.textContent).toContain("It came from mirror.example.test, which the collection did not review. If you're not sure, stop the installation.");
    expect(buttonLabels(root)).toEqual(["Install it", "Stop the installation"]);
    expect(document.activeElement?.textContent).toBe("Stop the installation");
  });

  it("disables every answer while one is saved and shows a failure as an alert", async () => {
    const busy = shown(changedOptional, { busy: true });
    const busyDialog = getByRole(busy.root, "dialog");
    expect(busyDialog.getAttribute("aria-busy")).toBe("true");
    expect([...busyDialog.querySelectorAll("button")].every(button => button.disabled)).toBe(true);
    expect(getByRole(busyDialog, "status").textContent).toBe("Saving your answer…");
    expect(document.activeElement).toBe(busyDialog);
    busy.root.remove();
    const failed = shown(changedOptional, { error: "The question changed since it was shown. Check it again before answering." });
    expect(getByRole(failed.root, "alert").textContent).toContain("The question changed");
    expect(document.activeElement?.id).toBe("download-decision-error");
  });

  it("keeps the focus inside the dialog and cannot be dismissed with Escape", () => {
    const { root, answer } = shown(changedRequired);
    const dialog = getByRole(root, "dialog");
    const stop = getByRole(dialog, "button", { name: "Stop the installation" });
    stop.focus();
    fireEvent.keyDown(stop, { key: "Tab" });
    expect(document.activeElement?.textContent).toBe("Details");
    fireEvent.keyDown(document.activeElement!, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(stop);
    fireEvent.keyDown(stop, { key: "Escape" });
    expect(root.querySelector(".decision-dialog")).not.toBeNull();
    expect(answer).not.toHaveBeenCalled();
  });
});

class DecisionBackend extends FixtureBackend {
  installations: ManagedInstallation[] = [];
  pending: PendingDecision | null = changedOptional;
  readError: BackendCommandError | null = null;
  answerError: BackendCommandError | null = null;
  snapshots: string[] = ["decision_required"];
  snapshotReport: RunSnapshot["report"] = null;
  reads: string[] = [];
  answers: unknown[][] = [];
  resumes: string[] = [];
  emit?: (event: RunEventEnvelope) => void;
  onResume?: (id: string, onEvent: (event: RunEventEnvelope) => void) => void;

  override getStatus() { return Promise.resolve({ mode: "native" as const, engineVersion: "0.1.0", recipeVersion: "recipe-1", startupInstallId: null }); }
  override listManagedInstallations(): Promise<ManagedInstallation[]> { return Promise.resolve(this.installations); }
  override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
    this.emit = onEvent;
    return Promise.resolve({ runId: "first-run" });
  }
  override getRunSnapshot(runId: string): Promise<RunSnapshot> {
    const status = this.snapshots.length > 1 ? this.snapshots.shift()! : this.snapshots[0]!;
    return Promise.resolve({ runId, status, events: [], report: this.snapshotReport, error: null });
  }
  async getPendingDecision(id: string): Promise<PendingDecision | null> {
    this.reads.push(id);
    if (this.readError) throw this.readError;
    return this.pending;
  }
  async answerDecision(id: string, decisionId: string, token: string, answer: DecisionAnswer): Promise<void> {
    this.answers.push([id, decisionId, token, answer]);
    if (this.answerError) throw this.answerError;
    this.pending = this.pending && { ...this.pending, answered: answer };
  }
  override resumeBuild(id: string, onEvent: (event: RunEventEnvelope) => void) {
    this.resumes.push(id);
    this.emit = onEvent;
    onEvent({ runId: "resumed", sequenceAsString: "1", event: { type: "campaign_started", install_id: id, resumed: true } });
    this.onResume?.(id, onEvent);
    return Promise.resolve({ runId: "resumed" });
  }
}

const waiting: ManagedInstallation = {
  id: "waiting-install", name: "Adventure", path: "D:\\Managed Game", status: "Waiting for your answer",
  available: false, resumable: true, receiptPath: null, launchPath: null, completedAtMillis: null,
};

async function mounted(backend: DecisionBackend) {
  const root = document.createElement("div");
  document.body.append(root);
  await mountApp(root, backend);
  return { root, user: userEvent.setup() };
}

function decisionRequired(backend: DecisionBackend, runId = "first-run", sequence = "2"): void {
  backend.emit!({ runId, sequenceAsString: sequence, event: { type: "decision_required", install_id: "waiting-install", step_id: "acquire:snowfall", decision_id: "decision-1" } });
}

describe("download question flow", () => {
  it("shows the question instead of a failure once the run has let go of the installation", async () => {
    const backend = new DecisionBackend();
    backend.snapshots = ["running", "running", "decision_required"];
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    backend.emit!({ runId: "first-run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "waiting-install", resumed: false } });
    decisionRequired(backend);
    // The card waits; the question is read only after the run ended and released its lock.
    expect(getByText(root, "Waiting for your answer")).toBeTruthy();
    expect(backend.reads).toEqual([]);
    const dialog = await waitFor(() => getByRole(root, "dialog", { name: "A download needs your answer" }));
    expect(backend.reads).toEqual(["waiting-install"]);
    expect(backend.snapshots).toEqual(["decision_required"]);
    expect(getByText(dialog, changedOptional.message)).toBeTruthy();
    expect((root.querySelector(".app-shell") as HTMLElement).inert).toBe(true);
    expect(document.activeElement?.textContent).toBe("Install it");
    expect(queryByText(root, "The build stopped safely")).toBeNull();
    expect(queryByRole(root, "button", { name: "Retry failed step" })).toBeNull();
    expect(backend.answers).toEqual([]);
    expect(backend.resumes).toEqual([]);
  });

  it("stores the answer with its question's token, then resumes the installation", async () => {
    const backend = new DecisionBackend();
    backend.onResume = (_id, onEvent) => onEvent({ runId: "resumed", sequenceAsString: "2", event: { type: "step_started", id: "acquire:snowfall", label: "Download Snowfall" } });
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    decisionRequired(backend);
    const dialog = await waitFor(() => getByRole(root, "dialog"));
    await user.click(getByRole(dialog, "button", { name: "Install it" }));
    expect(backend.answers).toEqual([["waiting-install", "decision-1", "token-1", "yes"]]);
    await waitFor(() => expect(backend.resumes).toEqual(["waiting-install"]));
    expect(root.querySelector(".decision-dialog")).toBeNull();
    expect((root.querySelector(".app-shell") as HTMLElement).inert).toBeFalsy();
    await waitFor(() => expect(getByText(root, "Download Snowfall")).toBeTruthy());
    await user.click(getByText(root, "Technical log"));
    expect(root.querySelector("pre")?.textContent).toContain('acquire:snowfall: you answered "Install it"');
  });

  it("keeps the question open when the answer cannot be saved, and never resumes", async () => {
    const backend = new DecisionBackend();
    backend.answerError = new BackendCommandError({ code: "decision_stale", message: "The question changed since it was shown.", recovery_action: "Check it again before answering.", technical_detail: "token mismatch" });
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    decisionRequired(backend);
    await user.click(await waitFor(() => getByRole(getByRole(root, "dialog"), "button", { name: "Install it" })));
    const dialog = getByRole(root, "dialog");
    await waitFor(() => expect(getByRole(dialog, "alert").textContent).toBe("The question changed since it was shown. Check it again before answering."));
    expect(document.activeElement?.id).toBe("download-decision-error");
    expect(backend.resumes).toEqual([]);
    backend.answerError = null;
    await user.click(getByRole(getByRole(root, "dialog"), "button", { name: "Install it" }));
    await waitFor(() => expect(backend.resumes).toEqual(["waiting-install"]));
  });

  it("after Stop, resumes once so the step records the choice, and shows the stop as the player's choice", async () => {
    const backend = new DecisionBackend();
    backend.onResume = (id, onEvent) => {
      backend.snapshots = ["failed"];
      backend.snapshotReport = { install_id: id, managed_root: "D:\\Managed Game", plan_sha256: "plan",
        status: { status: "failed", step_id: "acquire:snowfall", reason: "You chose to stop: A newer version of Snowfall was found." } };
      onEvent({ runId: "resumed", sequenceAsString: "2", event: { type: "error", step_id: null, message: "A required download is not available." } });
    };
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    decisionRequired(backend);
    await user.click(await waitFor(() => getByRole(getByRole(root, "dialog"), "button", { name: "Stop the installation" })));
    expect(backend.answers).toEqual([["waiting-install", "decision-1", "token-1", "no"]]);
    await waitFor(() => expect(getByText(root, "You stopped the installation")).toBeTruthy());
    expect(getByText(root, AFTER_NO)).toBeTruthy();
    expect(queryByRole(root, "button", { name: "Retry failed step" })).toBeNull();
    expect(root.querySelector(".status-card.danger")).toBeNull();
    // A resume asks the link again: the engine stops for a new question.
    backend.pending = { ...changedOptional, decisionId: "decision-2", token: "token-2" };
    backend.snapshots = ["decision_required"];
    backend.onResume = (_id, onEvent) => onEvent({ runId: "resumed", sequenceAsString: "2", event: { type: "decision_required", install_id: "waiting-install", step_id: "acquire:snowfall", decision_id: "decision-2" } });
    await user.click(getByRole(root, "button", { name: "Resume installation" }));
    await waitFor(() => expect(getByRole(root, "dialog")).toBeTruthy());
    expect(backend.resumes).toEqual(["waiting-install", "waiting-install"]);
    expect(queryByText(root, "You stopped the installation")).toBeNull();
  });

  it("reopens a waiting installation on the question without resuming it", async () => {
    const backend = new DecisionBackend();
    backend.installations = [waiting];
    const { root } = await mounted(backend);
    const dialog = await waitFor(() => getByRole(root, "dialog"));
    expect(getByText(root, "Waiting for your answer")).toBeTruthy();
    expect(getByText(dialog, changedOptional.message)).toBeTruthy();
    expect(backend.reads).toEqual(["waiting-install"]);
    expect(backend.resumes).toEqual([]);
  });

  it("closes the question with Decide later and keeps the installation waiting for it", async () => {
    const backend = new DecisionBackend();
    backend.installations = [waiting];
    const { root, user } = await mounted(backend);
    await waitFor(() => getByRole(root, "dialog"));
    expect(buttonLabels(root)).toEqual(["Install it", "Leave it out and continue", "Stop the installation", "Decide later"]);
    expect(document.activeElement?.textContent).toBe("Install it");
    fireEvent.keyDown(document.activeElement!, { key: "Escape" });
    // Let any closing work finish before checking that Escape changed nothing.
    await new Promise(resolve => window.setTimeout(resolve, 50));
    expect(root.querySelector(".decision-dialog")).not.toBeNull();
    expect(queryByRole(root, "button", { name: "Answer the question" })).toBeNull();
    await user.click(getByRole(getByRole(root, "dialog"), "button", { name: "Decide later" }));
    expect(root.querySelector(".decision-dialog")).toBeNull();
    expect((root.querySelector(".app-shell") as HTMLElement).inert).toBeFalsy();
    expect(getByText(root, "Waiting for your answer")).toBeTruthy();
    expect(backend.answers).toEqual([]);
    expect(backend.resumes).toEqual([]);
    await user.click(getByRole(root, "button", { name: "Answer the question" }));
    await waitFor(() => expect(getByRole(root, "dialog")).toBeTruthy());
    expect(document.activeElement?.textContent).toBe("Install it");
    expect(backend.reads).toEqual(["waiting-install", "waiting-install"]);
    expect(backend.resumes).toEqual([]);
  });

  it("offers Answer the question on the installation card, which opens the question", async () => {
    const backend = new DecisionBackend();
    const ready: ManagedInstallation = { id: "ready-install", name: "Finished game", path: "D:\\Finished", status: "Ready",
      available: true, resumable: false, receiptPath: "D:\\Finished\\install-receipt.json", launchPath: "D:\\Finished\\InfinityLoader.exe", completedAtMillis: 1 };
    backend.installations = [ready, waiting];
    const { root, user } = await mounted(backend);
    expect(backend.reads).toEqual([]);
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "waiting-install");
    expect(getByText(root, "Waiting for your answer")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Answer the question" }));
    const dialog = await waitFor(() => getByRole(root, "dialog"));
    expect(getByText(dialog, changedOptional.message)).toBeTruthy();
    expect(backend.resumes).toEqual([]);
  });

  it("resumes directly when the answer is already stored", async () => {
    const backend = new DecisionBackend();
    const ready: ManagedInstallation = { id: "ready-install", name: "Finished game", path: "D:\\Finished", status: "Ready",
      available: true, resumable: false, receiptPath: "D:\\Finished\\install-receipt.json", launchPath: "D:\\Finished\\InfinityLoader.exe", completedAtMillis: 1 };
    backend.installations = [ready, waiting];
    backend.pending = { ...changedOptional, answered: "yes" };
    const { root, user } = await mounted(backend);
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "waiting-install");
    await user.click(getByRole(root, "button", { name: "Answer the question" }));
    await waitFor(() => expect(backend.resumes).toEqual(["waiting-install"]));
    expect(root.querySelector(".decision-dialog")).toBeNull();
    expect(backend.answers).toEqual([]);
  });

  it("keeps waiting when the question cannot be read, and reads it again on request", async () => {
    const backend = new DecisionBackend();
    backend.installations = [waiting];
    backend.readError = new BackendCommandError({ code: "decision_busy", message: "The installation is busy.", recovery_action: "Try again in a moment.", technical_detail: "lock held" });
    const { root, user } = await mounted(backend);
    await waitFor(() => expect(getByRole(root, "alert").textContent).toBe("The installation is busy. Try again in a moment."));
    expect(root.querySelector(".decision-dialog")).toBeNull();
    expect(queryByRole(root, "button", { name: "Resume installation" })).toBeNull();
    backend.readError = null;
    await user.click(getByRole(root, "button", { name: "Show the question" }));
    await waitFor(() => expect(getByRole(root, "dialog")).toBeTruthy());
    expect(backend.resumes).toEqual([]);
  });

  it("shows the question when a pause request ends at a question instead", async () => {
    const backend = new DecisionBackend();
    backend.snapshots = ["pausing", "decision_required"];
    const { root, user } = await mounted(backend);
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    backend.emit!({ runId: "first-run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "waiting-install", resumed: false } });
    await user.click(getByRole(root, "button", { name: "Pause after current mod" }));
    await waitFor(() => expect(getByRole(root, "dialog")).toBeTruthy());
    expect(getByText(root, "Waiting for your answer")).toBeTruthy();
  });
});

describe("download question commands", () => {
  it("reads and answers a question through the native bridge", async () => {
    const invoke = vi.fn<InvokeCommand>(async command => command === "get_pending_decision" ? changedOptional : undefined);
    const backend = new NativeBackend(invoke);
    expect(await backend.getPendingDecision("install")).toEqual(changedOptional);
    expect(invoke).toHaveBeenLastCalledWith("get_pending_decision", { installId: "install" });
    await backend.answerDecision("install", "decision-1", "token-1", "leave_out");
    expect(invoke).toHaveBeenLastCalledWith("answer_decision", { installId: "install", decisionId: "decision-1", token: "token-1", answer: "leave_out" });
  });
});

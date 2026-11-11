// @vitest-environment jsdom
import { fireEvent, getByLabelText, getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { afterEach, describe, expect, it, vi } from "vitest";

import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend, NativeBackend, type InvokeCommand } from "../src/backend";
import type { DecisionAnswer, GameLogStatus, ManagedInstallation, PendingDecision, RunEventEnvelope, RunSnapshot } from "../src/contracts";
import { FLAVORS } from "../src/flavors";
import bridgeFlavor from "../src-tauri/src/flavor.rs?raw";
import { environmentFacts, failedComponentSubject, failedInstallationSubject, reportBody, reportTitle, REPORT_TEXT_LIMIT } from "../src/report";

function installation(overrides: Partial<ManagedInstallation> = {}): ManagedInstallation {
  return { id: "ready", name: "Adventure", path: "D:\\Games\\Adventure", status: "Ready to play", receiptPath: null,
    launchPath: "D:\\Games\\Adventure\\game\\InfinityLoader.exe", completedAtMillis: 1, available: true, resumable: false,
    recipeVersion: "0.1.2", ...overrides };
}

/** Records what the report dialog asks the backend to do. */
class ReportBackend extends FixtureBackend {
  redactions: string[] = [];
  redactionError: Error | null = null;
  issues: { title: string; body: string }[] = [];
  shortened = false;
  issueError: BackendCommandError | null = null;
  chats = 0;
  exports: string[] = [];
  exportResult: { path: string } | null = { path: "D:\\Exports\\chriz-easy-bg-diagnostics.zip" };
  shown = 0;

  override getStatus() {
    return Promise.resolve({ mode: "native" as const, applicationVersion: "0.1.3", engineVersion: "0.1.3", recipeVersion: "0.1.3",
      startupInstallId: null, profiles: [{ id: "recommended", label: "Recommended setup", description: "" }], selectedProfile: "recommended" });
  }
  override redactReportText(text: string): Promise<string> {
    this.redactions.push(text);
    if (this.redactionError) return Promise.reject(this.redactionError);
    return Promise.resolve(text.replaceAll("C:\\Users\\Max", "<redacted-home>"));
  }
  override openReportIssue(title: string, body: string) {
    this.issues.push({ title, body });
    if (this.issueError) return Promise.reject(this.issueError);
    return Promise.resolve({ shortened: this.shortened });
  }
  override openReportChat(): Promise<void> {
    this.chats += 1;
    return Promise.resolve();
  }
  override exportDiagnostics(installId: string) {
    this.exports.push(installId);
    return Promise.resolve(this.exportResult);
  }
  override showDiagnosticsExport(): Promise<void> {
    this.shown += 1;
    return Promise.resolve();
  }
}

async function mounted<T extends FixtureBackend>(backend: T) {
  const root = document.createElement("div");
  document.body.append(root);
  await mountApp(root, backend);
  return { root, backend, user: userEvent.setup() };
}

const dialog = (root: HTMLElement) => getByRole(root, "dialog", { name: /Report/ });
const details = (root: HTMLElement) => root.querySelector<HTMLElement>("#report-details")!.textContent ?? "";
const statusText = (root: HTMLElement) => root.querySelector<HTMLElement>("#report-status")!.textContent ?? "";

async function openFromHeader(root: HTMLElement, user: ReturnType<typeof userEvent.setup>) {
  await user.click(getByRole(root, "button", { name: "Feedback" }));
  await waitFor(() => expect(details(root)).toContain("App: Chriz Easy BG"));
  return dialog(root);
}

function stubClipboard(writeText: (text: string) => Promise<void>): void {
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
}

afterEach(() => {
  vi.restoreAllMocks();
  Reflect.deleteProperty(navigator, "clipboard");
  window.localStorage.clear();
  document.body.replaceChildren();
});

describe("report text", () => {
  it("names the kind and the subject in the title and keeps it short", () => {
    expect(reportTitle("problem", "Installation stopped at \"Install mods\"", "ignored")).toBe("[Problem] Installation stopped at \"Install mods\"");
    expect(reportTitle("feedback", "", "\n  Please add a dark theme\nmore words")).toBe("[Feedback] Please add a dark theme");
    expect(reportTitle("problem", "", "")).toBe("[Problem] Problem report");
    expect(reportTitle("feedback", "", "x".repeat(300)).length).toBe(100);
  });

  it("puts the player's words first and asks for them when there are none", () => {
    expect(reportBody("problem", " It broke. ", "App: Chriz Easy BG 0.1.3", true)).toBe(
      "It broke.\n\nDetails from Chriz Easy BG:\nApp: Chriz Easy BG 0.1.3\n\nDiagnostics ZIP: attach it to this report if you saved one.");
    expect(reportBody("feedback", "", "App: Chriz Easy BG 0.1.3", false)).toBe(
      "(Write your feedback or idea here.)\n\nDetails from Chriz Easy BG:\nApp: Chriz Easy BG 0.1.3");
  });

  it("lists versions, store and setup without any path", () => {
    const facts = environmentFacts(
      { mode: "native", applicationVersion: "0.1.3", engineVersion: "0.1.0", recipeVersion: "0.1.3", startupInstallId: null,
        profiles: [{ id: "recommended", label: "Recommended setup", description: "" }], selectedProfile: "recommended" },
      [{ slot: "bg2", candidate: { id: "c", label: "Baldur's Gate II", path: "C:\\Users\\Max\\Games\\BG2", storefront: "gog", build: "2.6.6.0",
        freshness: "fresh", eligible: true, tested: false, findings: [] } }],
      installation({ customized: true }),
    );
    expect(facts).toEqual([
      "App: Chriz Easy BG 0.1.3",
      "Collection: 0.1.3 (Recommended setup)",
      "Game: Baldur's Gate II 2.6.6.0, GOG, less-tested copy",
      "Installation: collection 0.1.2, Ready to play, with manual mods",
    ]);
    expect(facts.join("\n")).not.toContain("Max");
  });

  it("describes a stopped installation by its phase, step, reason and error", () => {
    const subject = failedInstallationSubject({
      state: "failed", headline: "Installation failed", detail: "", logTail: ["C:\\Users\\Max\\log line"], manualArchiveName: null,
      phases: [{ id: "a", title: "Copy your games", detail: "", state: "done" }, { id: "b", title: "Install mods", detail: "", state: "failed" }],
      failureReason: "WeiDU stopped\nwith exit code 1",
      commandError: { code: "campaign_failed", message: "The installation could not finish.", recovery_action: "Retry.", technical_detail: "x".repeat(900) },
    }, "install:tweaks-bg2", "install-1");
    expect(subject.summary).toBe("Installation stopped at \"Install mods\"");
    expect(subject.facts.slice(0, 5)).toEqual([
      "What happened: Installation failed",
      "Phase: Install mods",
      "Step: install:tweaks-bg2",
      "Reason: WeiDU stopped with exit code 1",
      "Error: campaign_failed: The installation could not finish.",
    ]);
    // Long details are cut; the log itself is only in the diagnostics ZIP.
    expect(subject.facts[5]!.length).toBeLessThanOrEqual("Error details: ".length + 400);
    expect(subject.facts.join("\n")).not.toContain("log line");
    expect(subject.installId).toBe("install-1");
  });

  it("takes the first sentence of a failed part's question as its subject", () => {
    const subject = failedComponentSubject({
      installId: "i", stepId: "install:tweaks-bg2", decisionId: "d", token: "t", kind: "failed_component", artifactId: "tweaks",
      message: "Rolled HP (Tweaks Anthology) could not be installed. Leave it out and continue?",
      details: ["Run tweaks-bg2, component 3000", "The installer said: cannot convert a variable"],
      answers: [], focus: "leave_out", afterNo: "", answered: null,
    });
    expect(subject.summary).toBe("Rolled HP (Tweaks Anthology) could not be installed");
    expect(subject.facts).toEqual([
      "What happened: Rolled HP (Tweaks Anthology) could not be installed. Leave it out and continue?",
      "Step: install:tweaks-bg2",
      "Detail: Run tweaks-bg2, component 3000",
      "Detail: The installer said: cannot convert a variable",
    ]);
  });
});

describe("report dialog", () => {
  it("is always in the header and opens a GitHub issue with the player's words and the app's details", async () => {
    const { root, backend, user } = await mounted(new ReportBackend({ managedInstallations: [installation()] }));
    const opened = await openFromHeader(root, user);
    expect(opened.getAttribute("aria-modal")).toBe("true");
    expect((root.querySelector(".app-shell") as HTMLElement).inert).toBe(true);
    expect(getByText(opened, /CEBG sends nothing by itself/)).toBeTruthy();
    expect(getByText(opened, /Opens github\.com\/Chrizhermann\/chriz-easy-bg in your browser/)).toBeTruthy();
    // A report without a subject lets the player say what it is; a problem is preselected.
    expect((getByRole(opened, "radio", { name: "A problem" }) as HTMLInputElement).checked).toBe(true);
    expect(document.activeElement).toBe(getByRole(opened, "radio", { name: "A problem" }));

    await user.click(getByRole(opened, "radio", { name: "Feedback or an idea" }));
    const text = getByLabelText(opened, "Your feedback or idea") as HTMLTextAreaElement;
    expect(text.maxLength).toBe(REPORT_TEXT_LIMIT);
    await user.type(text, "Please add a search for mods.");
    await user.click(getByRole(opened, "button", { name: "Open GitHub issue" }));

    await waitFor(() => expect(statusText(root)).toBe("Your browser opens the GitHub issue page. Check the text there, then submit it."));
    expect(backend.issues).toEqual([{
      title: "[Feedback] Please add a search for mods.",
      body: "Please add a search for mods.\n\nDetails from Chriz Easy BG:\nApp: Chriz Easy BG 0.1.3\nCollection: 0.1.3 (Recommended setup)\n"
        + "Installation: collection 0.1.2, Ready to play\n\nDiagnostics ZIP: attach it to this report if you saved one.",
    }]);
    expect((await axe.run(root, { rules: { "color-contrast": { enabled: false } } })).violations).toEqual([]);
  });

  it("shows the details only after the backend removed private paths, and drops free-form facts when it cannot", async () => {
    class FailedBackend extends ReportBackend {
      override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
        queueMicrotask(() => {
          onEvent({ runId: "run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "install-failed", resumed: false } });
          onEvent({ runId: "run", sequenceAsString: "2", event: { type: "error", step_id: null, message: "The build stopped." } });
        });
        return Promise.resolve({ runId: "run" });
      }
      override getRunSnapshot(runId: string): Promise<RunSnapshot> {
        return Promise.resolve({ runId, status: "failed", events: [], error: null, report: {
          install_id: "install-failed", managed_root: "D:\\Failed", plan_sha256: "11".repeat(32),
          status: { status: "failed", step_id: "install:test", reason: "could not read C:\\Users\\Max\\Documents\\save" },
        } });
      }
    }
    const { root, backend, user } = await mounted(new FailedBackend());
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByRole(root, "button", { name: "Retry failed step" })).toBeTruthy());
    await user.click(getByRole(root, "button", { name: "Report this problem" }));
    const opened = dialog(root);
    // A report from a failure is a problem: no choice is offered.
    expect(queryByRole(opened, "radio")).toBeNull();
    await waitFor(() => expect(details(root)).toContain("Reason: could not read <redacted-home>\\Documents\\save"));
    expect(details(root)).not.toContain("Max");
    expect(backend.redactions).toHaveLength(1);
    expect(backend.redactions[0]).toContain("C:\\Users\\Max");

    await user.click(getByRole(opened, "button", { name: "Close" }));
    expect(queryByRole(root, "dialog")).toBeNull();
    expect((root.querySelector(".app-shell") as HTMLElement).inert).not.toBe(true);
    expect(document.activeElement).toBe(getByRole(root, "button", { name: "Report this problem" }));

    backend.redactionError = new Error("offline");
    await user.click(getByRole(root, "button", { name: "Report this problem" }));
    await waitFor(() => expect(details(root)).toContain("App: Chriz Easy BG 0.1.3"));
    expect(details(root)).toContain("What happened: the details could not be prepared; please describe it in your own words.");
    expect(details(root)).not.toContain("Max");
    expect(details(root)).not.toContain("Reason:");
  });

  it("keeps the text being written and its focus while the app renders behind it", async () => {
    class RunningBackend extends ReportBackend {
      emit?: (event: RunEventEnvelope) => void;
      override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
        this.emit = onEvent;
        return Promise.resolve({ runId: "run" });
      }
    }
    const { root, backend, user } = await mounted(new RunningBackend());
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    backend.emit!({ runId: "run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "running", resumed: false } });
    const opened = await openFromHeader(root, user);
    const text = getByLabelText(opened, "What happened, and what did you expect?") as HTMLTextAreaElement;
    await user.type(text, "The progress bar");
    const shell = root.querySelector(".app-shell");
    backend.emit!({ runId: "run", sequenceAsString: "2", event: { type: "step_started", install_id: "running", step_id: "install:tweaks", title: "Tweaks" } as never });
    await waitFor(() => expect(root.querySelector(".app-shell")).not.toBe(shell));

    expect(dialog(root)).toBe(opened);
    expect(text.value).toBe("The progress bar");
    expect(document.activeElement).toBe(text);
    expect((root.querySelector(".app-shell") as HTMLElement).inert).toBe(true);
    await user.type(text, " jumps.");
    expect(text.value).toBe("The progress bar jumps.");
  });

  it("copies the whole report when it did not fit into the link", async () => {
    const copied: string[] = [];
    stubClipboard((text) => { copied.push(text); return Promise.resolve(); });
    const { root, backend, user } = await mounted(new ReportBackend());
    backend.shortened = true;
    const opened = await openFromHeader(root, user);
    await user.type(getByLabelText(opened, "What happened, and what did you expect?"), "Long story");
    stubClipboard((text) => { copied.push(text); return Promise.resolve(); });
    await user.click(getByRole(opened, "button", { name: "Open GitHub issue" }));
    await waitFor(() => expect(statusText(root)).toContain("the whole text is on your clipboard"));
    expect(copied).toEqual([`[Problem] Long story\n\n${backend.issues[0]!.body}`]);
  });

  it("copies the report for Discord and shows it for copying by hand when the clipboard refuses", async () => {
    const { root, backend, user } = await mounted(new ReportBackend());
    const opened = await openFromHeader(root, user);
    expect(getByText(opened, /paste it in ChrizFader's Discord, channel #mod-feedback\./)).toBeTruthy();
    await user.type(getByLabelText(opened, "What happened, and what did you expect?"), "It crashed");
    const copied: string[] = [];
    stubClipboard((text) => { copied.push(text); return Promise.resolve(); });
    await user.click(getByRole(opened, "button", { name: "Copy text" }));
    await waitFor(() => expect(statusText(root)).toBe("Report text copied. Paste it where you want to send it."));
    expect(copied[0]).toContain("[Problem] It crashed\n\nIt crashed\n\nDetails from Chriz Easy BG:\nApp: Chriz Easy BG 0.1.3");
    expect((root.querySelector("#report-manual-copy") as HTMLTextAreaElement).hidden).toBe(true);

    await user.click(getByRole(opened, "button", { name: "Open Discord" }));
    await waitFor(() => expect(statusText(root)).toBe("Your browser opens the invitation to ChrizFader's Discord."));
    expect(backend.chats).toBe(1);

    stubClipboard(() => Promise.reject(new Error("denied")));
    await user.click(getByRole(opened, "button", { name: "Copy text" }));
    await waitFor(() => expect(statusText(root)).toBe("The text couldn't be copied for you. It is selected below: press Ctrl+C."));
    const manual = root.querySelector("#report-manual-copy") as HTMLTextAreaElement;
    expect(manual.hidden).toBe(false);
    expect(manual.value).toBe(copied[0]);
    expect(document.activeElement).toBe(manual);
  });

  it("saves the diagnostics ZIP of the installation and shows its folder", async () => {
    const { root, backend, user } = await mounted(new ReportBackend({ managedInstallations: [installation()] }));
    const opened = await openFromHeader(root, user);
    expect(queryByRole(opened, "button", { name: "Show in folder" })).toBeNull();
    await user.click(getByRole(opened, "button", { name: "Save diagnostics ZIP" }));
    await waitFor(() => expect(statusText(root)).toBe("Diagnostics ZIP saved. Attach it to your report yourself."));
    expect(backend.exports).toEqual(["ready"]);
    expect(getByText(opened, "Saved to D:\\Exports\\chriz-easy-bg-diagnostics.zip")).toBeTruthy();
    await user.click(getByRole(opened, "button", { name: "Show in folder" }));
    expect(backend.shown).toBe(1);

    // A cancelled Save dialog changes nothing.
    backend.exportResult = null;
    await user.click(getByRole(opened, "button", { name: "Save another copy" }));
    await waitFor(() => expect(backend.exports).toHaveLength(2));
    expect(getByText(opened, "Saved to D:\\Exports\\chriz-easy-bg-diagnostics.zip")).toBeTruthy();
  });

  it("offers no diagnostics where no installation is in view, and shows a failed action in the dialog", async () => {
    const { root, backend, user } = await mounted(new ReportBackend());
    const opened = await openFromHeader(root, user);
    expect((opened.querySelector(".report-diagnostics") as HTMLElement).hidden).toBe(true);
    backend.issueError = new BackendCommandError({ code: "report_open_failed", message: "The page could not be opened.",
      recovery_action: "Use Copy text instead.", technical_detail: "no browser" });
    await user.click(getByRole(opened, "button", { name: "Open GitHub issue" }));
    await waitFor(() => expect(statusText(root)).toBe("The page could not be opened. Use Copy text instead."));
    expect(queryByRole(root, "alert")).toBeNull();
    // Escape closes the dialog and returns to the button that opened it.
    fireEvent.keyDown(opened, { key: "Escape" });
    expect(queryByRole(root, "dialog")).toBeNull();
    expect(document.activeElement).toBe(getByRole(root, "button", { name: "Feedback" }));
  });

  it("reports a part that could not be installed without answering its question", async () => {
    const failedComponent: PendingDecision = {
      installId: "waiting-install", stepId: "install:tweaks-bg2", decisionId: "decision-1", token: "token-1",
      kind: "failed_component", artifactId: "tweaks",
      message: "Rolled HP (Tweaks Anthology) could not be installed. Leave it out and continue?",
      details: ["Run tweaks-bg2, component 3000", "The installer said: cannot convert a variable"],
      answers: [{ answer: "leave_out", label: "Leave it out and continue" }, { answer: "retry", label: "Try again" }, { answer: "no", label: "Stop the installation" }],
      focus: "leave_out", afterNo: "You chose to stop.", answered: null,
    };
    class WaitingBackend extends ReportBackend {
      answers: DecisionAnswer[] = [];
      override listManagedInstallations(): Promise<ManagedInstallation[]> {
        return Promise.resolve([installation({ id: "waiting-install", status: "Waiting for your answer", available: false, resumable: true, launchPath: null })]);
      }
      getPendingDecision(): Promise<PendingDecision | null> { return Promise.resolve(failedComponent); }
      answerDecision(_id: string, _decision: string, _token: string, answer: DecisionAnswer): Promise<void> {
        this.answers.push(answer);
        return Promise.resolve();
      }
    }
    const { root, backend, user } = await mounted(new WaitingBackend());
    const question = await waitFor(() => getByRole(root, "dialog", { name: "A mod needs your answer" }));
    expect(document.activeElement?.textContent).toBe("Leave it out and continue");
    expect(getByText(question, "Reporting does not answer the question.")).toBeTruthy();
    await user.click(getByRole(question, "button", { name: "Report this problem" }));

    const opened = getByRole(root, "dialog", { name: "Report this problem" });
    await waitFor(() => expect(details(root)).toContain("Detail: The installer said: cannot convert a variable"));
    // The question waits below the report dialog and cannot be answered by accident.
    expect((root.querySelector(".decision-dialog")!.parentElement as HTMLElement).inert).toBe(true);
    expect(root.lastElementChild).toBe(opened.parentElement);
    await user.click(getByRole(opened, "button", { name: "Open GitHub issue" }));
    await waitFor(() => expect(backend.issues).toHaveLength(1));
    expect(backend.issues[0]!.title).toBe("[Problem] Rolled HP (Tweaks Anthology) could not be installed");

    await user.click(getByRole(opened, "button", { name: "Close" }));
    expect(backend.answers).toEqual([]);
    const again = getByRole(root, "dialog", { name: "A mod needs your answer" });
    expect((again.parentElement as HTMLElement).inert).not.toBe(true);
    expect(document.activeElement).toBe(getByRole(again, "button", { name: "Report this problem" }));
    await user.click(getByRole(again, "button", { name: "Leave it out and continue" }));
    await waitFor(() => expect(backend.answers).toEqual(["leave_out"]));
  });

  it("offers no report in a question about a download", async () => {
    const { downloadDecisionDialog } = await import("../src/components/download-decision");
    const shown = downloadDecisionDialog({
      decision: { installId: "i", stepId: "acquire:x", decisionId: "d", token: "t", kind: "changed_download", artifactId: "x",
        message: "A newer version was found. Install it anyway?", details: [], answers: [{ answer: "yes", label: "Install it" }],
        focus: "yes", afterNo: "", answered: null },
      busy: false, answer: () => {}, report: () => {},
    });
    expect(queryByRole(shown, "button", { name: "Report this problem" })).toBeNull();
  });

  it("reports what a finished installation left out", async () => {
    class LeftOutBackend extends ReportBackend {
      reads = 0;
      override listManagedInstallations() {
        this.reads += 1;
        return Promise.resolve(this.reads === 1 ? [] : [installation({ id: "completed" })]);
      }
      override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
        queueMicrotask(() => {
          onEvent({ runId: "run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "completed", resumed: false } });
          onEvent({ runId: "run", sequenceAsString: "2", event: { type: "campaign_finished", install_id: "completed" } });
        });
        return Promise.resolve({ runId: "run" });
      }
      installationSummary(): Promise<readonly string[]> {
        return Promise.resolve(["Left out: Snowytoes. Its download could not be used.",
          "If something does not work as expected, please report it so the collection can be fixed."]);
      }
    }
    const { root, backend, user } = await mounted(new LeftOutBackend());
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByText(root, /use Report this below, or Feedback at the top at any time\./)).toBeTruthy());
    await user.click(getByRole(root, "button", { name: "Report this" }));
    await waitFor(() => expect(details(root)).toContain("Installation summary: Left out: Snowytoes. Its download could not be used."));
    expect(details(root)).not.toContain("please report it");
    await user.click(getByRole(dialog(root), "button", { name: "Open GitHub issue" }));
    await waitFor(() => expect(backend.issues[0]?.title).toBe("[Problem] Installation finished with mods left out or changed"));

    // The offer goes with its message.
    await user.click(getByRole(dialog(root), "button", { name: "Close" }));
    await user.click(getByRole(root, "button", { name: "Dismiss" }));
    expect(queryByRole(root, "button", { name: "Report this" })).toBeNull();
  });

  it("has no report offer on a completion without left-out mods", async () => {
    class CleanBackend extends ReportBackend {
      reads = 0;
      override listManagedInstallations() {
        this.reads += 1;
        return Promise.resolve(this.reads === 1 ? [] : [installation({ id: "completed" })]);
      }
      override startBuild(_token: string, onEvent: (event: RunEventEnvelope) => void) {
        queueMicrotask(() => {
          onEvent({ runId: "run", sequenceAsString: "1", event: { type: "campaign_started", install_id: "completed", resumed: false } });
          onEvent({ runId: "run", sequenceAsString: "2", event: { type: "campaign_finished", install_id: "completed" } });
        });
        return Promise.resolve({ runId: "run" });
      }
      installationSummary(): Promise<readonly string[]> { return Promise.resolve([]); }
    }
    const { root, user } = await mounted(new CleanBackend());
    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByText(root, "Installation complete")).toBeTruthy());
    expect(queryByRole(root, "button", { name: "Report this" })).toBeNull();
  });

  it("reports a game problem from Troubleshooting with the game log state", async () => {
    class CrashBackend extends ReportBackend {
      override gameLogStatus(installId: string): Promise<GameLogStatus> {
        const file = (name: string) => ({ path: `D:\\Games\\Adventure\\game\\${name}`, state: "missing" as const, sizeBytes: null, modifiedAtMillis: null, detail: null });
        return Promise.resolve({ installId, setting: "on", detail: null, log: file("InfinityLoader.log"), previousLog: file("InfinityLoader-previous.log"),
          crashFolder: "D:\\Games\\Adventure\\game\\InfinityLoader_Crash", crashReports: 2 });
      }
    }
    const { root, backend, user } = await mounted(new CrashBackend({ managedInstallations: [installation()] }));
    await user.click(getByText(root, "Troubleshooting"));
    await user.click(await waitFor(() => getByRole(root, "button", { name: "Report a game problem" })));
    await waitFor(() => expect(details(root)).toContain("Crash reports in the crash folder: 2"));
    expect(details(root)).toContain("Game log: on");
    expect(details(root)).toContain("Installation: collection 0.1.2, Ready to play");
    await user.type(getByLabelText(dialog(root), "What happened, and what did you expect?"), "Crash in the Copper Coronet");
    await user.click(getByRole(dialog(root), "button", { name: "Open GitHub issue" }));
    await waitFor(() => expect(backend.issues[0]?.title).toBe("[Problem] Crash in the Copper Coronet"));
  });

  it("shows the exported ZIP's folder from the diagnostics message", async () => {
    const { root, backend, user } = await mounted(new ReportBackend({ managedInstallations: [installation()] }));
    await user.click(getByRole(root, "button", { name: "Export diagnostics" }));
    await waitFor(() => expect(getByText(root, /Diagnostics exported to D:\\Exports/)).toBeTruthy());
    await user.click(getByRole(root, "button", { name: "Show in folder" }));
    expect(backend.shown).toBe(1);
  });
});

describe("report commands", () => {
  it("passes the report to the native commands", async () => {
    const calls: [string, Record<string, unknown> | undefined][] = [];
    const invoke: InvokeCommand = (command, args) => {
      calls.push([command, args]);
      if (command === "open_report_issue") return Promise.resolve({ shortened: true });
      if (command === "redact_report_text") return Promise.resolve("clean");
      return Promise.resolve(null);
    };
    const backend = new NativeBackend(invoke);
    expect(await backend.redactReportText("raw")).toBe("clean");
    expect(await backend.openReportIssue("[Problem] x", "body")).toEqual({ shortened: true });
    await backend.openReportChat();
    await backend.showDiagnosticsExport();
    expect(calls).toEqual([
      ["redact_report_text", { text: "raw" }],
      ["open_report_issue", { title: "[Problem] x", body: "body" }],
      ["open_report_chat", undefined],
      ["show_diagnostics_export", undefined],
    ]);
  });

  it("names the same GitHub repository as the bridge opens", () => {
    for (const flavor of Object.values(FLAVORS)) {
      const repository = flavor.report.issuesLabel.replace(/^github\.com\//u, "");
      expect(flavor.report.issuesLabel).toMatch(/^github\.com\/Chrizhermann\/chriz-easy-(bg|iwd)$/u);
      expect(flavor.report.chatChannel).toBe("#mod-feedback");
      expect(bridgeFlavor).toContain(`report_repository: "${repository}"`);
    }
  });
});

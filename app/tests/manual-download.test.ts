// @vitest-environment jsdom

import {
  getByRole,
  getByText,
  queryByRole,
  queryByText,
  waitFor,
} from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend } from "../src/backend";
import type {
  ManualArchiveSupply,
  ManualDownloadRequirement,
  NormalizedSelection,
} from "../src/contracts";

type SupplyOutcome = "valid" | "cancel" | "wrong" | "missing";

class ManualDownloadBackend extends FixtureBackend {
  readonly inspectedSelections: NormalizedSelection[] = [];
  readonly evaluatedSelections: NormalizedSelection[] = [];
  readonly openedArtifacts: string[] = [];
  readonly suppliedArtifacts: string[] = [];
  freezeCalls = 0;
  startCalls = 0;
  ready: boolean;
  supplyOutcome: SupplyOutcome = "valid";
  supplyGate: Promise<void> | null = null;
  defaultEvandra: boolean;
  detail = "Download the official Evandra release, then choose it here.";

  constructor(options: { ready?: boolean; selected?: boolean } = {}) {
    super();
    this.ready = options.ready ?? false;
    this.defaultEvandra = options.selected ?? true;
  }

  override listManagedInstallations() {
    return Promise.resolve([]);
  }

  override async evaluateBuild(selection: NormalizedSelection) {
    this.evaluatedSelections.push(selection);
    const evaluation = await super.evaluateBuild(selection);
    const evandraSelected = selection.features["mod:evandra"] ?? this.defaultEvandra;
    return {
      ...evaluation,
      normalizedSelection: {
        ...evaluation.normalizedSelection,
        features: { ...evaluation.normalizedSelection.features, "mod:evandra": evandraSelected },
      },
      findings: evandraSelected
        ? evaluation.findings
        : [...evaluation.findings, { rule: "manual-mod-skipped", featureId: "mod:evandra", message: "Evandra and related compatibility content will be left out." }],
      selectedChoiceCount: evaluation.selectedChoiceCount + (evandraSelected ? 1 : 0),
    };
  }

  override inspectManualDownloads(selection: NormalizedSelection): Promise<ManualDownloadRequirement[]> {
    this.inspectedSelections.push(selection);
    if (selection.features["mod:evandra"] === false) return Promise.resolve([]);
    return Promise.resolve([{
      artifactId: "evandra-2.2-windows",
      modIds: ["evandra"],
      title: "Evandra",
      filename: "evandra-v2.2.exe",
      length: 5_420_000,
      ready: this.ready,
      detail: this.detail,
    }]);
  }

  override async supplyManualArchive(artifactId: string): Promise<ManualArchiveSupply | null> {
    this.suppliedArtifacts.push(artifactId);
    if (this.supplyGate !== null) await this.supplyGate;
    if (this.supplyOutcome === "cancel") return null;
    if (this.supplyOutcome === "wrong") {
      throw new BackendCommandError({
        code: "manual_archive_mismatch",
        message: "That file is not the official Evandra download.",
        recovery_action: "Choose evandra-v2.2.exe from the official download.",
        technical_detail: "fixture mismatch",
      });
    }
    if (this.supplyOutcome === "missing") {
      this.detail = "The selected file is no longer available. Choose it again.";
    } else {
      this.ready = true;
    }
    return {
      artifactId,
      filename: "evandra-v2.2.exe",
      sha256: "11".repeat(32),
      length: 5_420_000,
    };
  }

  override openManualSource(artifactId: string): Promise<void> {
    this.openedArtifacts.push(artifactId);
    return Promise.resolve();
  }

  override async freezeReview(...args: Parameters<FixtureBackend["freezeReview"]>) {
    this.freezeCalls += 1;
    return super.freezeReview(...args);
  }

  override startBuild() {
    this.startCalls += 1;
    return Promise.resolve({ runId: "manual-download-run" });
  }
}

async function render(backend: ManualDownloadBackend) {
  const root = document.createElement("div");
  document.body.append(root);
  const app = await mountApp(root, backend);
  return { app, root, user: userEvent.setup() };
}

async function revealRequirement(root: HTMLElement, user: ReturnType<typeof userEvent.setup>) {
  await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
  await waitFor(() => expect(getByRole(root, "heading", { name: "Finish the download before installation" })).toBeTruthy());
}

describe("upfront manual downloads", () => {
  afterEach(() => document.body.replaceChildren());

  it("does not freeze or start before the selected manual archive is ready", async () => {
    const backend = new ManualDownloadBackend();
    const { root, user } = await render(backend);

    await revealRequirement(root, user);

    expect(getByText(root, "evandra-v2.2.exe (5.4 MB)")).toBeTruthy();
    expect(getByRole(root, "button", { name: "Choose downloaded file" })).toBeTruthy();
    expect(backend.freezeCalls).toBe(0);
    expect(backend.startCalls).toBe(0);
    expect(backend.openedArtifacts).toEqual([]);
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(true);

    await user.click(getByRole(root, "button", { name: "Download Evandra" }));
    expect(backend.openedArtifacts).toEqual(["evandra-2.2-windows"]);
  });

  it("shows checking, marks a valid file ready, and waits for another explicit Install", async () => {
    let releaseSupply!: () => void;
    const backend = new ManualDownloadBackend();
    backend.supplyGate = new Promise<void>((resolve) => { releaseSupply = resolve; });
    const { root, user } = await render(backend);
    await revealRequirement(root, user);

    await user.click(getByRole(root, "button", { name: "Choose downloaded file" }));
    expect(getByText(root, "Checking evandra-v2.2.exe…")).toBeTruthy();
    expect(backend.freezeCalls).toBe(0);
    releaseSupply();

    await waitFor(() => expect(getByText(root, "Verified and ready")).toBeTruthy());
    expect(backend.freezeCalls).toBe(0);
    expect(backend.startCalls).toBe(0);
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);

    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByRole(root, "heading", { name: "Installation progress" })).toBeTruthy());
    expect(backend.freezeCalls).toBe(1);
    expect(backend.startCalls).toBe(1);
  });

  it("keeps the requirement intact when the file picker is cancelled", async () => {
    const backend = new ManualDownloadBackend();
    backend.supplyOutcome = "cancel";
    const { root, user } = await render(backend);
    await revealRequirement(root, user);

    await user.click(getByRole(root, "button", { name: "Choose downloaded file" }));
    await waitFor(() => expect(backend.suppliedArtifacts).toHaveLength(1));

    expect(getByText(root, "File required")).toBeTruthy();
    expect(queryByRole(root, "alert")).toBeNull();
    expect(backend.freezeCalls).toBe(0);
  });

  it("shows the backend recovery action for a wrong file", async () => {
    const backend = new ManualDownloadBackend();
    backend.supplyOutcome = "wrong";
    const { root, user } = await render(backend);
    await revealRequirement(root, user);

    await user.click(getByRole(root, "button", { name: "Choose downloaded file" }));

    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain("not the official Evandra download"));
    expect(getByRole(root, "alert").textContent).toContain("Choose evandra-v2.2.exe");
    expect(backend.freezeCalls).toBe(0);
  });

  it("keeps a missing selected file actionable after reinspection", async () => {
    const backend = new ManualDownloadBackend();
    backend.supplyOutcome = "missing";
    const { root, user } = await render(backend);
    await revealRequirement(root, user);

    await user.click(getByRole(root, "button", { name: "Choose downloaded file" }));

    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain("no longer available"));
    expect(getByText(root, "File required")).toBeTruthy();
    expect(backend.freezeCalls).toBe(0);
  });

  it("skips every owning mod, re-evaluates omissions, and still requires Install", async () => {
    const backend = new ManualDownloadBackend();
    const { root, user } = await render(backend);
    await revealRequirement(root, user);

    await user.click(getByRole(root, "button", { name: "Skip Evandra" }));

    await waitFor(() => expect(queryByRole(root, "heading", { name: "Finish the download before installation" })).toBeNull());
    expect(getByText(root, "Evandra and related compatibility content will be left out.")).toBeTruthy();
    expect(backend.inspectedSelections.at(-1)?.features["mod:evandra"]).toBe(false);
    expect(backend.freezeCalls).toBe(0);
    expect(backend.startCalls).toBe(0);

    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));
    await waitFor(() => expect(getByRole(root, "heading", { name: "Installation progress" })).toBeTruthy());
    expect(backend.freezeCalls).toBe(1);
    expect(backend.startCalls).toBe(1);
  });

  it("does not prompt when Evandra is unselected", async () => {
    const backend = new ManualDownloadBackend({ selected: false });
    const { root, user } = await render(backend);

    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    await waitFor(() => expect(getByRole(root, "heading", { name: "Installation progress" })).toBeTruthy());
    expect(queryByText(root, "Finish the download before installation")).toBeNull();
    expect(backend.freezeCalls).toBe(1);
    expect(backend.startCalls).toBe(1);
  });

  it("continues immediately when cached bytes are already verified", async () => {
    const backend = new ManualDownloadBackend({ ready: true });
    const { root, user } = await render(backend);

    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    await waitFor(() => expect(getByRole(root, "heading", { name: "Installation progress" })).toBeTruthy());
    expect(queryByText(root, "Finish the download before installation")).toBeNull();
    expect(backend.freezeCalls).toBe(1);
    expect(backend.startCalls).toBe(1);
  });

  it("routes the legacy Review action through the same gate", async () => {
    const backend = new ManualDownloadBackend();
    const { app, root, user } = await render(backend);
    await app.navigate("review");

    await user.click(getByRole(root, "button", { name: "Install Chriz Easy BG" }));

    await waitFor(() => expect(getByRole(root, "heading", { name: "Finish the download before installation" })).toBeTruthy());
    expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(backend.freezeCalls).toBe(0);
    expect(backend.startCalls).toBe(0);
  });
});

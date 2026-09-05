// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { getByRole } from "@testing-library/dom";
import { campaignLedger } from "../src/components/campaign-ledger";
import { buildScreen } from "../src/screens/build";

describe("installation progress announcements", () => {
  it("does not announce completion when a resumed run has no projected phases", () => {
    const ledger = campaignLedger([], true);
    expect(ledger.textContent).not.toContain("All phases complete");
    expect(ledger.querySelector('[role="status"]')?.textContent).toBe("Installation progress is not available yet.");
  });

  it("does not announce completion when all phases are still pending", () => {
    const ledger = campaignLedger([{ id: "main", title: "Mods", detail: "Install mods", state: "pending" }], true);
    expect(ledger.querySelector('[role="status"]')?.textContent).toBe("Waiting to start installation steps.");
  });

  it("announces completion only for a nonempty set of completed phases", () => {
    const ledger = campaignLedger([{ id: "main", title: "Mods", detail: "Install mods", state: "done" }], true);
    expect(ledger.querySelector('[role="status"]')?.textContent).toBe("All phases complete");
  });

  it("separates cooperative pause from the explicitly unsafe force stop", () => {
    const pause = vi.fn();
    const stopNow = vi.fn();
    const screen = buildScreen({ state: "running", headline: "Running", detail: "A mod is active", phases: [], logTail: [], manualArchiveName: null }, {
      backToSetup: () => {}, advance: () => {}, retry: () => {}, supplyManual: () => {}, openManualSource: () => {},
      pause, stopNow, diagnostics: () => {}, diagnosticsAvailable: false, fixture: false, retryAvailable: false,
      logState: { paused: false, open: false }, updateLogState: () => {},
    });

    getByRole(screen, "button", { name: "Pause after current mod" }).click();
    getByRole(screen, "button", { name: "Stop now (may need repair)" }).click();
    expect(pause).toHaveBeenCalledOnce();
    expect(stopNow).toHaveBeenCalledOnce();
  });
});

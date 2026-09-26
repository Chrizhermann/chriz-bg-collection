// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole } from "@testing-library/dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { operationFeedback, progressDisplay } from "../src/components/operation-feedback";
import type { OperationProgress } from "../src/contracts";
import {
  clearAppUpdate, completedAppUpdate, decodeOperationProgress,
  progressLabel, progressPercent, rememberAppUpdate,
} from "../src/operation-feedback";

const pendingKey = "cebg.pending-app-update.v1";
const oldVersion = "0.1.0-alpha.18";
const newVersion = "0.1.0-alpha.19";
function bytes(completed: string | null, total: string | null): OperationProgress {
  return { stage: "backup-game", completed, total, unit: "bytes" };
}

beforeEach(() => {
  localStorage.clear();
  vi.spyOn(Date, "now").mockReturnValue(Date.UTC(2026, 8, 22, 12));
});
afterEach(() => {
  document.body.replaceChildren();
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("honest operation progress", () => {
  it.each([
    ["0", "100", 0], ["25", "100", 25], ["1", "3", 33], ["100", "100", 100],
    ["18446744073709551614", "18446744073709551615", 99],
    ["18446744073709551615", "18446744073709551615", 100],
  ])("uses exact bounded counts %s / %s", (completed, total, percent) => {
    expect(progressPercent(bytes(completed, total))).toBe(percent);
  });

  it.each([
    [null, null], ["0", "0"], ["1", "0"], ["101", "100"], [null, "100"],
    ["1", null], ["", "100"], ["1", ""], ["-1", "100"], ["1", "-100"],
    ["0.5", "100"], ["1e2", "100"], ["Infinity", "100"], ["NaN", "100"],
    [" 1", "100"], ["1 ", "100"], ["0x10", "100"],
    ["123456789012345678901", "123456789012345678901"],
  ])("keeps unknown or invalid counts indeterminate: %s / %s", (completed, total) => {
    expect(progressPercent(bytes(completed, total))).toBeNull();
    const display = progressDisplay(bytes(completed, total));
    expect(getByRole(display, "progressbar").hasAttribute("value")).toBe(false);
    expect(display.querySelector(".operation-spinner")).not.toBeNull();
    expect(display.querySelector(".operation-count")).toBeNull();
  });

  it("shows step progress without calling a fully copied backup a completed update", () => {
    const display = progressDisplay(bytes("100", "100"));
    const bar = getByRole(display, "progressbar", { name: "Backing up your game…" });
    expect(bar.getAttribute("value")).toBe("100");
    expect(bar.getAttribute("max")).toBe("100");
    expect(display.textContent).toContain("100% of this step");
    expect(display.textContent).not.toContain("Update complete");
    expect(display.querySelector(".operation-spinner")).toBeNull();
    expect(getByRole(display, "status").textContent).toBe("Backing up your game…");
  });

  it("uses a spinner while preparing or when stage totals are unavailable", () => {
    expect(progressPercent()).toBeNull();
    expect(progressLabel()).toBe("Preparing…");
    const display = progressDisplay();
    expect(getByRole(display, "progressbar", { name: "Preparing…" }).hasAttribute("value")).toBe(false);
    expect(display.querySelector(".operation-spinner")?.getAttribute("aria-hidden")).toBe("true");
    expect(progressLabel({ stage: "unknown internal value" })).toBe("Working…");
  });

  it("accepts only known progress stages and does not reinterpret numeric counts", () => {
    for (const value of [null, undefined, "backup-game", 1, [], {}, { stage: "not-a-stage" }]) {
      expect(decodeOperationProgress(value)).toBeNull();
    }
    const progress = decodeOperationProgress({ stage: "backup-saves", completed: 5, total: 10, unit: "bytes" });
    expect(progress).toMatchObject({ stage: "backup-saves", completed: null, total: null });
    expect(progressPercent(progress ?? undefined)).toBeNull();
    expect(decodeOperationProgress(bytes("5", "10"))).toEqual(bytes("5", "10"));
  });
});

describe("visible operation result", () => {
  it("names a successful result and provides a working dismissal action", () => {
    const dismiss = vi.fn();
    const panel = operationFeedback({
      title: "BuffBot update complete", detail: "My test game now uses BuffBot 1.8.4-alpha.", state: "success",
    }, dismiss);
    document.body.append(panel);
    expect(getByRole(panel, "status").textContent).toBe("BuffBot update complete");
    expect(panel.getAttribute("aria-label")).toBe("BuffBot update complete");
    expect(panel.textContent).toContain("My test game now uses BuffBot 1.8.4-alpha.");
    expect(queryByRole(panel, "progressbar")).toBeNull();
    fireEvent.click(getByRole(panel, "button", { name: "Dismiss" }));
    expect(dismiss).toHaveBeenCalledOnce();
  });

  it("does not dismiss an in-progress operation or present it as success", () => {
    const panel = operationFeedback({ title: "Updating BuffBot…", detail: "Keep the game closed.", state: "working", progress: bytes("2", "8") }, vi.fn());
    expect(queryByRole(panel, "button", { name: "Dismiss" })).toBeNull();
    expect(getByRole(panel, "progressbar").getAttribute("value")).toBe("25");
    expect(panel.classList.contains("operation-success")).toBe(false);
  });

  it("announces failures as errors, not successful completion", () => {
    const panel = operationFeedback({ title: "Update could not finish", detail: "Restore before playing.", state: "error" }, vi.fn());
    expect(getByRole(panel, "alert").textContent).toBe("Update could not finish");
    expect(panel.classList.contains("operation-error")).toBe(true);
    expect(panel.classList.contains("operation-success")).toBe(false);
    expect(queryByRole(panel, "progressbar")).toBeNull();
  });

  it("renders titles, details and stage labels as plain text", () => {
    const injected = '<img src=x onerror="window.injected=true">';
    const panel = operationFeedback({ title: injected, detail: `<script>${injected}</script>`, state: "success" }, vi.fn());
    const display = progressDisplay(bytes("1", "2"), injected);
    expect(panel.textContent).toContain(injected);
    expect(panel.querySelector("img, script")).toBeNull();
    expect(display.querySelector("img, script")).toBeNull();
    expect(getByRole(display, "progressbar").getAttribute("aria-label")).toBe(injected);
  });
});

describe("app-update confirmation across restart", () => {
  it("announces only a requested different version that is actually running", () => {
    expect(completedAppUpdate(newVersion)).toBeNull();
    rememberAppUpdate(oldVersion, newVersion);
    expect(completedAppUpdate(oldVersion)).toBeNull();
    expect(completedAppUpdate("0.1.0-alpha.20")).toBeNull();
    expect(completedAppUpdate(newVersion)).toMatchObject({ title: "CEBG update complete", state: "success" });
    expect(completedAppUpdate(newVersion)?.detail).toContain(newVersion);
    expect(completedAppUpdate(newVersion)?.detail).toContain("game installations are unchanged");
  });

  it("does not call an unchanged requested version an update", () => {
    rememberAppUpdate(newVersion, newVersion);
    expect(completedAppUpdate(newVersion)).toBeNull();
  });

  it("forgets the completed notification when explicitly cleared", () => {
    rememberAppUpdate(oldVersion, newVersion);
    expect(completedAppUpdate(newVersion)).not.toBeNull();
    clearAppUpdate();
    expect(localStorage.getItem(pendingKey)).toBeNull();
    expect(completedAppUpdate(newVersion)).toBeNull();
  });

  it.each([
    "not JSON", "null", "[]", "{}", "3",
    JSON.stringify({ from: oldVersion, to: newVersion }),
    JSON.stringify({ from: 18, to: newVersion, at: Date.UTC(2026, 8, 22, 12) }),
    JSON.stringify({ from: oldVersion, to: 19, at: Date.UTC(2026, 8, 22, 12) }),
    JSON.stringify({ from: oldVersion, to: newVersion, at: "now" }),
    JSON.stringify({ from: "", to: newVersion, at: Date.UTC(2026, 8, 22, 12) }),
    JSON.stringify({ from: "   ", to: newVersion, at: Date.UTC(2026, 8, 22, 12) }),
  ])("ignores malformed pending record %s", raw => {
    localStorage.setItem(pendingKey, raw);
    expect(completedAppUpdate(newVersion)).toBeNull();
  });

  it("ignores expired or future-dated records", () => {
    for (const at of [Date.now() - 31 * 86_400_000, Date.now() + 1, -1]) {
      localStorage.setItem(pendingKey, JSON.stringify({ from: oldVersion, to: newVersion, at }));
      expect(completedAppUpdate(newVersion)).toBeNull();
    }
  });

  it("does not report success when storing update intent fails", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("storage blocked"); });
    expect(() => rememberAppUpdate(oldVersion, newVersion)).not.toThrow();
    expect(completedAppUpdate(newVersion)).toBeNull();
  });

  it("keeps optional storage errors from affecting the app operation", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("storage blocked"); });
    vi.spyOn(Storage.prototype, "removeItem").mockImplementation(() => { throw new Error("storage blocked"); });
    expect(completedAppUpdate(newVersion)).toBeNull();
    expect(() => clearAppUpdate()).not.toThrow();
  });
});

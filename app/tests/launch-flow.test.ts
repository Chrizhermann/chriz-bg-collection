// @vitest-environment jsdom
import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { FixtureBackend, NativeBackend } from "../src/backend";
import type { LaunchInstallResult, ManagedInstallation } from "../src/contracts";

function installation(id = "one", overrides: Partial<ManagedInstallation> = {}): ManagedInstallation {
  return { id, name: `Game ${id}`, path: `D:\\${id}`, status: "Ready to play", receiptPath: null,
    launchPath: `D:\\${id}\\game\\InfinityLoader.exe`, completedAtMillis: 1,
    available: true, resumable: false, radarVersion: "2.5.2.0", ...overrides };
}

function result(installId: string, state: LaunchInstallResult["radar"]["state"], detail: string | null = null,
  technicalDetail: string | null = null): LaunchInstallResult {
  return { installId, radar: { state, detail, technicalDetail } };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

class LaunchBackend extends FixtureBackend {
  readonly requests: { installId: string; startRadar: boolean }[] = [];
  launchResult: Promise<LaunchInstallResult> | null = null;
  constructor(installations = [installation()]) { super({ managedInstallations: installations }); }
  override launchInstall(installId: string, startRadar: boolean): Promise<LaunchInstallResult> {
    this.requests.push({ installId, startRadar });
    return this.launchResult ?? Promise.resolve(result(installId, startRadar ? "started" : "not-requested"));
  }
}

async function mount(backend = new LaunchBackend()) {
  const root = document.createElement("div");
  document.body.append(root);
  const app = await mountApp(root, backend);
  return { root, backend, app, user: userEvent.setup() };
}

const radarChoice = (root: HTMLElement) => getByRole(root, "checkbox", { name: "Start Radar with the game" }) as HTMLInputElement;
const play = (root: HTMLElement) => getByRole(root, "button", { name: "Play Chriz Easy BG" }) as HTMLButtonElement;

afterEach(() => {
  vi.restoreAllMocks();
  window.localStorage.clear();
  document.body.replaceChildren();
});

describe("optional Radar launch", () => {
  it("starts without Radar by default and describes the optional administrator prompt", async () => {
    const { root, backend, user } = await mount();
    expect(radarChoice(root).checked).toBe(false);
    expect(radarChoice(root).disabled).toBe(false);
    expect(radarChoice(root).getAttribute("aria-describedby")).toBe("radar-launch-help");
    expect(getByText(root, "Windows may ask for administrator approval.")).toBeTruthy();
    await user.click(play(root));
    expect(backend.requests).toEqual([{ installId: "one", startRadar: false }]);
    expect(getByRole(root, "status").textContent).toBe("Game started.");
    expect(root.textContent).toContain("Select Start Radar with the game");
    expect(root.textContent).toContain("BG Radar Overlay folder inside the game folder");
  });

  it("remembers an explicit choice per installation and after reopening", async () => {
    const backend = new LaunchBackend([installation(), installation("two")]);
    const { root, user } = await mount(backend);
    await user.click(radarChoice(root));
    await user.click(play(root));
    expect(backend.requests).toEqual([{ installId: "one", startRadar: true }]);
    expect(getByRole(root, "status").textContent).toBe("Game and Radar started.");
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "two");
    expect(radarChoice(root).checked).toBe(false);
    expect(queryByRole(root, "status")).toBeNull();
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "one");
    expect(radarChoice(root).checked).toBe(true);
    root.remove();
    const reopened = await mount(backend);
    expect(radarChoice(reopened.root).checked).toBe(true);
    await reopened.user.click(radarChoice(reopened.root));
    reopened.root.remove();
    expect(radarChoice((await mount(backend)).root).checked).toBe(false);
  });

  it("retains the in-session choice if browser preference storage is denied", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("Denied"); });
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("Denied"); });
    const { root, backend, user, app } = await mount();
    expect(radarChoice(root).checked).toBe(false);
    await user.click(radarChoice(root));
    await app.navigate("home");
    expect(radarChoice(root).checked).toBe(true);
    await user.click(play(root));
    expect(backend.requests).toEqual([{ installId: "one", startRadar: true }]);
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("supports keyboard opt-in and has no DOM accessibility violations with a Radar warning", async () => {
    const backend = new LaunchBackend();
    backend.launchResult = Promise.resolve(result("one", "failed", "Continue playing without Radar.", "Launch error"));
    const { root, user } = await mount(backend);
    radarChoice(root).focus();
    await user.keyboard(" ");
    expect(radarChoice(root).checked).toBe(true);
    expect(document.activeElement).toBe(radarChoice(root));
    play(root).focus();
    await user.keyboard("{Enter}");
    expect(backend.requests).toEqual([{ installId: "one", startRadar: true }]);
    expect(getByRole(root, "alert").textContent).toContain("Game started, but Radar couldn't start.");
    // jsdom lacks the canvas API required by axe's visual contrast rule.
    expect((await axe.run(root, { rules: { "color-contrast": { enabled: false } } })).violations).toEqual([]);
  });

  it("accepts only a stored explicit true value", async () => {
    window.localStorage.setItem("cebg.start-radar.one", "yes");
    expect(radarChoice((await mount()).root).checked).toBe(false);
  });

  it.each(["cancelled", "failed"] as const)("keeps %s warnings scoped to the launched installation and preserves details", async (state) => {
    const backend = new LaunchBackend([installation(), installation("two")]);
    const launch = deferred<LaunchInstallResult>();
    backend.launchResult = launch.promise;
    const { root, user, app } = await mount(backend);
    await user.click(radarChoice(root));
    await user.click(play(root));
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "two");
    launch.resolve(result("one", state, "You can continue playing without Radar.", "Native diagnostic text"));
    await waitFor(() => expect(play(root).disabled).toBe(false));
    expect(queryByRole(root, "alert")).toBeNull();
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "one");
    const warning = getByRole(root, "alert");
    expect(warning.textContent).toContain(state === "cancelled"
      ? "Game started, but Radar wasn't started because administrator approval was cancelled."
      : "Game started, but Radar couldn't start.");
    expect(warning.textContent).toContain("You can continue playing without Radar.");
    expect(warning.querySelector<HTMLDetailsElement>("details")?.open).toBe(false);
    expect(warning.textContent).toContain("Native diagnostic text");
    await app.navigate("home");
    expect(getByRole(root, "alert").textContent).toContain("Game started, but Radar");
    expect(play(root).disabled).toBe(false);
  });

  it("blocks repeated Play requests until launch completes, including detached old buttons", async () => {
    const backend = new LaunchBackend();
    const launch = deferred<LaunchInstallResult>();
    backend.launchResult = launch.promise;
    const { root, user } = await mount(backend);
    await user.click(radarChoice(root));
    const oldPlay = play(root);
    oldPlay.click(); oldPlay.click(); oldPlay.click();
    expect(backend.requests).toEqual([{ installId: "one", startRadar: true }]);
    expect((getByRole(root, "button", { name: "Starting game…" }) as HTMLButtonElement).disabled).toBe(true);
    expect(radarChoice(root).disabled).toBe(true);
    launch.resolve(result("one", "started"));
    await waitFor(() => expect(play(root).disabled).toBe(false));
    expect(radarChoice(root).disabled).toBe(false);
  });

  it("restores Play and reports a game-launch error without claiming the game started", async () => {
    const backend = new LaunchBackend();
    const launch = deferred<LaunchInstallResult>();
    backend.launchResult = launch.promise;
    const { root, user } = await mount(backend);
    await user.click(play(root));
    launch.reject(new Error("Game launcher is unavailable."));
    await waitFor(() => expect(getByRole(root, "alert").textContent).toContain("Game launcher is unavailable."));
    expect(play(root).disabled).toBe(false);
    expect(root.textContent).not.toContain("Game started");
  });

  it.each([
    { radarVersion: null },
    { patchRecoveryNeeded: true },
  ])("does not permit Radar selection when unavailable: %j", async (overrides) => {
    const { root, backend, user } = await mount(new LaunchBackend([installation("one", overrides)]));
    expect(radarChoice(root).disabled).toBe(true);
    await user.click(radarChoice(root));
    expect(radarChoice(root).checked).toBe(false);
    await user.click(play(root));
    expect(backend.requests).toEqual(overrides.patchRecoveryNeeded ? [] : [{ installId: "one", startRadar: false }]);
  });

  it("does not offer launch options for incomplete installations", async () => {
    const { root } = await mount(new LaunchBackend([installation("one", { available: false, resumable: true })]));
    expect(queryByRole(root, "checkbox", { name: "Start Radar with the game" })).toBeNull();
    expect(queryByRole(root, "button", { name: "Play Chriz Easy BG" })).toBeNull();
  });

  it.each([false, true])("passes the explicit %s choice through the native boundary and returns Radar outcome", async (startRadar) => {
    const outcome = result("one", startRadar ? "failed" : "not-requested", "Player-facing detail", "Windows error");
    const invoke = vi.fn(async () => outcome);
    await expect(new NativeBackend(invoke).launchInstall("one", startRadar)).resolves.toEqual(outcome);
    expect(invoke).toHaveBeenCalledWith("launch_install", { installId: "one", startRadar });
  });
});

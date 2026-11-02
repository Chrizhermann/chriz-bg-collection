// @vitest-environment jsdom
import { getByRole, getByText, queryByRole, waitFor } from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import axe from "axe-core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { mountApp } from "../src/app";
import { BackendCommandError, FixtureBackend, NativeBackend } from "../src/backend";
import type { GameLogStatus, ManagedInstallation } from "../src/contracts";

function installation(id = "one", overrides: Partial<ManagedInstallation> = {}): ManagedInstallation {
  return { id, name: `Game ${id}`, path: `D:\\${id}`, status: "Ready to play", receiptPath: null,
    launchPath: `D:\\${id}\\game\\InfinityLoader.exe`, completedAtMillis: 1,
    available: true, resumable: false, ...overrides };
}

function status(installId: string, overrides: Partial<GameLogStatus> = {}): GameLogStatus {
  const game = `D:\\${installId}\\game`;
  const file = (name: string) => ({ path: `${game}\\${name}`, state: "missing" as const, sizeBytes: null,
    modifiedAtMillis: null, detail: null });
  return { installId, setting: "off", detail: null, log: file("InfinityLoader.log"),
    previousLog: file("InfinityLoader-previous.log"), crashFolder: `${game}\\InfinityLoader_Crash`, crashReports: 0,
    ...overrides };
}

async function mount(backend = new FixtureBackend({ managedInstallations: [installation()] })) {
  const root = document.createElement("div");
  document.body.append(root);
  await mountApp(root, backend);
  return { root, backend, user: userEvent.setup() };
}

const section = (root: HTMLElement) => root.querySelector<HTMLDetailsElement>("details.troubleshooting")!;
const gameLog = (root: HTMLElement) =>
  getByRole(root, "checkbox", { name: "Write a game log for crash reports" }) as HTMLInputElement;
const help = (root: HTMLElement) => root.querySelector<HTMLElement>("#game-log-help")!.textContent;

async function openTroubleshooting(root: HTMLElement, user: ReturnType<typeof userEvent.setup>) {
  await user.click(getByText(root, "Troubleshooting"));
  await waitFor(() => expect(gameLog(root).disabled).toBe(false));
}

afterEach(() => {
  vi.restoreAllMocks();
  window.localStorage.clear();
  document.body.replaceChildren();
});

describe("game log for crash reports", () => {
  it("stays out of the normal flow and reads nothing until Troubleshooting is opened", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    const statusCheck = vi.spyOn(backend, "gameLogStatus");
    const { root, user } = await mount(backend);
    expect(section(root).open).toBe(false);
    expect(queryByRole(root, "checkbox", { name: "Write a game log for crash reports" })).toBeNull();
    expect(statusCheck).not.toHaveBeenCalled();

    await openTroubleshooting(root, user);
    expect(statusCheck).toHaveBeenCalledWith("one");
    expect(gameLog(root).checked).toBe(false);
    expect(gameLog(root).getAttribute("aria-describedby")).toBe("game-log-help");
    expect(help(root)).toContain("Takes effect the next time the game starts.");
    expect(document.activeElement).toBe(getByText(root, "Troubleshooting"));
    expect(root.textContent).toContain("D:\\one\\game\\InfinityLoader.log");
    expect(root.textContent).toContain("D:\\one\\game\\InfinityLoader-previous.log");
    expect(root.textContent).toContain("D:\\one\\game\\InfinityLoader_Crash");
    expect(root.textContent).toContain("No crash reports yet.");
  });

  it("turns the log on and off for the next start and keeps the section open", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    const change = vi.spyOn(backend, "setGameLog");
    const { root, user } = await mount(backend);
    await openTroubleshooting(root, user);

    await user.click(gameLog(root));
    await waitFor(() => expect(gameLog(root).checked).toBe(true));
    expect(change).toHaveBeenLastCalledWith("one", true);
    expect(section(root).open).toBe(true);
    expect(gameLog(root).disabled).toBe(false);
    expect(document.activeElement).toBe(gameLog(root));

    await user.click(gameLog(root));
    await waitFor(() => expect(gameLog(root).checked).toBe(false));
    expect(change).toHaveBeenLastCalledWith("one", false);
    expect(change).toHaveBeenCalledTimes(2);
  });

  it("lists the evidence of a crash and says which files to send", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    vi.spyOn(backend, "gameLogStatus").mockResolvedValue(status("one", {
      setting: "on",
      log: { path: "D:\\one\\game\\InfinityLoader.log", state: "present", sizeBytes: "1677721", modifiedAtMillis: 1_788_451_200_000, detail: null },
      previousLog: { path: "D:\\one\\game\\InfinityLoader-previous.log", state: "not-a-file", sizeBytes: null, modifiedAtMillis: null, detail: null },
      crashReports: 2,
    }));
    const { root, user } = await mount(backend);
    await openTroubleshooting(root, user);
    expect(gameLog(root).checked).toBe(true);
    expect(root.textContent).toContain("1.6 MiB, last written");
    expect(root.textContent).toContain("Something other than a file is in its place.");
    expect(root.textContent).toContain("2 crash reports.");
    expect(root.textContent).toContain("that crash's .log and _small.dmp files");
    expect(root.textContent).toContain("send one only when asked");
  });

  it("explains a log set outside CEBG and an installation whose loader settings cannot change", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    const check = vi.spyOn(backend, "gameLogStatus").mockResolvedValue(status("one", { setting: "other", detail: "logs\\mine.log" }));
    const { root, user } = await mount(backend);
    await openTroubleshooting(root, user);
    expect(gameLog(root).checked).toBe(true);
    expect(help(root)).toContain("The game writes its log to logs\\mine.log, as set outside CEBG.");
    expect(help(root)).toContain("Turning this off stops that log.");

    check.mockResolvedValue(status("one", { setting: "unavailable", detail: "InfinityLoader.ini has no [General] section." }));
    await user.click(getByRole(root, "button", { name: "Check again" }));
    await waitFor(() => expect(gameLog(root).checked).toBe(false));
    expect(gameLog(root).disabled).toBe(true);
    expect(help(root)).toBe("The game log can't be changed for this installation. InfinityLoader.ini has no [General] section.");
    expect(document.activeElement).toBe(getByRole(root, "button", { name: "Check again" }));
  });

  it("reports a refused change and keeps showing the actual setting", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    vi.spyOn(backend, "setGameLog").mockRejectedValue(new BackendCommandError({
      code: "game_log_failed",
      message: "The game log setting wasn't changed.",
      recovery_action: "Close the game and wait for any update of this installation to finish, then try again.",
      technical_detail: "Close the game and other programs from this installation first; still running: D:\\one\\game\\Baldur.exe",
    }));
    const { root, user } = await mount(backend);
    await openTroubleshooting(root, user);
    await user.click(gameLog(root));
    const alert = await waitFor(() => getByRole(root, "alert"));
    expect(alert.textContent).toContain("The game log setting wasn't changed.");
    expect(alert.textContent).toContain("Close the game and wait for any update");
    expect(alert.querySelector<HTMLDetailsElement>("details")?.open).toBe(false);
    expect(alert.textContent).toContain("still running: D:\\one\\game\\Baldur.exe");
    expect(gameLog(root).checked).toBe(false);
    expect(gameLog(root).disabled).toBe(false);
  });

  it("waits for patch recovery before changing the log", async () => {
    const { root, user } = await mount(new FixtureBackend({ managedInstallations: [installation("one", { patchRecoveryNeeded: true })] }));
    await user.click(getByText(root, "Troubleshooting"));
    await waitFor(() => expect(help(root)).toBe("Finish patch recovery in Updates before changing the game log."));
    expect(gameLog(root).disabled).toBe(true);
  });

  it("closes when another installation is chosen and reads that one only when opened", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation(), installation("two")] });
    const statusCheck = vi.spyOn(backend, "gameLogStatus");
    const { root, user } = await mount(backend);
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "one");
    await openTroubleshooting(root, user);
    await user.selectOptions(getByRole(root, "combobox", { name: "Switch install" }), "two");
    expect(section(root).open).toBe(false);
    expect(statusCheck.mock.calls).toEqual([["one"]]);
    await openTroubleshooting(root, user);
    expect(statusCheck.mock.calls).toEqual([["one"], ["two"]]);
    expect(root.textContent).toContain("D:\\two\\game\\InfinityLoader.log");
  });

  it("never lets a slow check of one installation hold up or move focus in another", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation(), installation("two")] });
    let finishFirst: (result: GameLogStatus) => void = () => undefined;
    const statusCheck = vi.spyOn(backend, "gameLogStatus")
      .mockImplementationOnce(() => new Promise<GameLogStatus>((resolve) => { finishFirst = resolve; }));
    const { root, user } = await mount(backend);
    const switcher = () => getByRole(root, "combobox", { name: "Switch install" });
    await user.selectOptions(switcher(), "one");
    await user.click(getByText(root, "Troubleshooting"));
    expect(help(root)).toBe("Checking the game log…");

    await user.selectOptions(switcher(), "two");
    await openTroubleshooting(root, user);
    expect(statusCheck.mock.calls).toEqual([["one"], ["two"]]);
    const refresh = getByRole(root, "button", { name: "Check again" });
    refresh.focus();
    finishFirst(status("one", { setting: "on" }));
    await waitFor(() => expect(refresh.isConnected).toBe(false));
    expect(document.activeElement).toBe(getByRole(root, "button", { name: "Check again" }));
    expect(gameLog(root).checked).toBe(false);
    expect(gameLog(root).disabled).toBe(false);
    expect(root.textContent).toContain("D:\\two\\game\\InfinityLoader.log");
  });

  it("warns after Play when the last session's log couldn't be kept, without blocking the game", async () => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    vi.spyOn(backend, "launchInstall").mockResolvedValue({
      installId: "one",
      radar: { state: "not-requested", detail: null, technicalDetail: null },
      gameSettings: { state: "unchanged", technicalDetail: null },
      gameLog: { state: "failed", technicalDetail: "D:\\one\\game\\InfinityLoader-previous.log: Access is denied. (os error 5)" },
    });
    const { root, user } = await mount(backend);
    await user.click(getByRole(root, "button", { name: "Play Chriz Easy BG" }));
    const alert = await waitFor(() => getByRole(root, "alert"));
    expect(getByRole(root, "status").textContent).toBe("Game started.");
    expect(alert.textContent).toContain("The log of your last game session couldn't be kept.");
    expect(alert.textContent).toContain("Access is denied");
    // jsdom lacks the canvas API required by axe's visual contrast rule.
    expect((await axe.run(root, { rules: { "color-contrast": { enabled: false } } })).violations).toEqual([]);
  });

  it.each(["not-enabled", "nothing-to-keep", "kept"] as const)("shows no game-log notice after Play when it is %s", async (state) => {
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    vi.spyOn(backend, "launchInstall").mockResolvedValue({
      installId: "one",
      radar: { state: "not-requested", detail: null, technicalDetail: null },
      gameSettings: { state: "unchanged", technicalDetail: null },
      gameLog: { state, technicalDetail: null },
    });
    const { root, user } = await mount(backend);
    await user.click(getByRole(root, "button", { name: "Play Chriz Easy BG" }));
    await waitFor(() => expect(getByRole(root, "status").textContent).toBe("Game started."));
    expect(queryByRole(root, "alert")).toBeNull();
  });

  it("has no DOM accessibility violations with the section open", async () => {
    const { root, user } = await mount();
    await openTroubleshooting(root, user);
    // jsdom lacks the canvas API required by axe's visual contrast rule.
    expect((await axe.run(root, { rules: { "color-contrast": { enabled: false } } })).violations).toEqual([]);
  });

  it("passes the installation and the choice through the native boundary", async () => {
    const result = status("one", { setting: "on" });
    const invoke = vi.fn(async () => result);
    const backend = new NativeBackend(invoke);
    await expect(backend.gameLogStatus("one")).resolves.toEqual(result);
    expect(invoke).toHaveBeenLastCalledWith("game_log_status", { installId: "one" });
    await expect(backend.setGameLog("one", true)).resolves.toEqual(result);
    expect(invoke).toHaveBeenLastCalledWith("set_game_log", { installId: "one", enabled: true });
  });
});

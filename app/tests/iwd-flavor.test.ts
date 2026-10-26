// @vitest-environment jsdom

import { fireEvent, getByRole, queryByRole, queryByText, waitFor } from "@testing-library/dom";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { ManagedInstallation, UpdateSummary } from "../src/contracts";
import { FLAVORS } from "../src/flavors";

/** Baldur's Gate words that must never appear on a screen of the IWD build. */
const BG_WORDS = /Baldur|BG1|BG2|CEBG|Dragonspear|EET|Radar|BuffBot/;

/** Loads the frontend as `vite --mode iwd` builds it. */
async function iwd() {
  vi.stubEnv("MODE", "iwd");
  vi.resetModules();
  const [app, backend, flavor, state, games, review, loading, removal] = await Promise.all([
    import("../src/app"),
    import("../src/backend"),
    import("../src/flavor"),
    import("../src/state"),
    import("../src/screens/games"),
    import("../src/screens/review"),
    import("../src/screens/loading"),
    import("../src/components/installation-removal"),
  ]);
  expect(flavor.flavor.id).toBe("iwd");
  return { ...app, ...backend, ...flavor, ...state, ...games, ...review, ...loading, ...removal };
}

/** Loads the default (BG) build, regardless of earlier stubs. */
async function bg() {
  vi.unstubAllEnvs();
  vi.resetModules();
  const [app, backend, flavor] = await Promise.all([import("../src/app"), import("../src/backend"), import("../src/flavor")]);
  expect(flavor.flavor.id).toBe("bg");
  return { ...app, ...backend, ...flavor };
}

function rootElement(): HTMLElement {
  const root = document.createElement("div");
  document.body.append(root);
  return root;
}

/** Visible text plus every attribute a player can perceive through assistive technology. */
function perceivable(node: Element): string {
  const parts = [node.textContent ?? ""];
  for (const element of node.querySelectorAll("*")) {
    for (const name of ["aria-label", "title", "placeholder", "alt"]) {
      const value = element.getAttribute(name);
      if (value) parts.push(value);
    }
    if (element instanceof HTMLInputElement) parts.push(element.value);
  }
  return parts.join("\n");
}

function expectNoBgWords(node: Element, screen: string): void {
  const text = perceivable(node);
  const match = BG_WORDS.exec(text);
  expect(match === null ? null : `${screen}: "${match[0]}" in …${text.slice(Math.max(0, match.index - 60), match.index + 60)}…`).toBeNull();
}

function installation(overrides: Partial<ManagedInstallation> = {}): ManagedInstallation {
  return {
    id: "iwd-install", name: "Chriz Easy IWD", path: "D:\\Games\\Chriz Easy IWD", status: "Ready to play",
    receiptPath: "D:\\Games\\Chriz Easy IWD\\install-receipt.json", launchPath: "D:\\Games\\Chriz Easy IWD\\InfinityLoader.exe",
    completedAtMillis: 1, available: true, resumable: false, recipeVersion: "2026.10",
    // A backend that still reports an overlay must not bring Radar into this build.
    radarVersion: "1.2.0",
    consistency: {
      state: "matches", detail: "All recorded mods are present.", componentCount: 2, modCount: 2,
      components: [
        { target: "BG2", tp2: "EEex/EEex.tp2", component: 0, title: "EEex", version: "1.3.1", status: "installed" },
        { target: "BG2", tp2: "buffbot/setup-buffbot.tp2", component: 0, title: "Quick buffs", version: "1.0", status: "installed" },
      ],
    },
    ...overrides,
  };
}

/** An update summary that offers every BG-only extra; the IWD build must show none of them. */
function updatesWithBgExtras(): UpdateSummary {
  return {
    checkedAt: null,
    networkState: "unconfigured",
    application: { state: "available", currentVersion: "0.1.0-alpha.1", availableVersion: "0.1.0-alpha.2", detail: "Available." },
    recipe: { state: "unavailable", currentVersion: "bundled", availableVersion: null, disposition: "unknown-applicability", detail: "No channel.", changes: [] },
    managedCopies: [{ installId: "iwd-install", name: "Chriz Easy IWD", path: "D:\\Games\\Chriz Easy IWD", installedRecipeVersion: "2026.10", state: "unknown", detail: "No channel." }],
    radar: { state: "available", currentVersion: "1.2.0", availableVersion: "1.3.0", detail: "Overlay update." },
  };
}

/** Steps the fixture build through every state to completion, recording each screen. */
async function finishFixtureBuild(root: HTMLElement, record: (step: string) => void = () => undefined): Promise<void> {
  await waitFor(() => expect(getByRole(root, "heading", { name: "Installation progress" })).toBeTruthy());
  for (const action of ["I added the archive", "Keep waiting", "Retry failed step", "Finish fixture build"]) {
    record(action);
    fireEvent.click(getByRole(root, "button", { name: action }));
    if (action !== "Finish fixture build") await waitFor(() => expect(queryByRole(root, "button", { name: action })).toBeNull());
  }
  await waitFor(() => expect(getByRole(root, "heading", { level: 1, name: "Ready to play" })).toBeTruthy());
}

afterEach(() => {
  document.body.replaceChildren();
  try { window.localStorage.clear(); } catch { /* optional */ }
  vi.restoreAllMocks();
  vi.unstubAllEnvs();
  vi.resetModules();
});

describe("Chriz Easy IWD build flavor", () => {
  it("switches to one Icewind Dale source in slot bg2", async () => {
    const { flavor, sourceSlots } = await iwd();
    expect(flavor.productName).toBe("Chriz Easy IWD");
    expect(flavor.sources).toEqual([{ slot: "bg2", role: "iwdee", label: "Icewind Dale: Enhanced Edition", name: "Icewind Dale: Enhanced Edition", reviewLabel: "Game source" }]);
    expect(sourceSlots).toEqual(["bg2"]);
    expect(Object.values(flavor.features).every(enabled => !enabled)).toBe(true);
  });

  it("shows exactly one source card and installs with an empty bg1 candidate", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const backend = new FixtureBackend();
    const discovery = await backend.discoverGames();
    expect(discovery.bg1Candidates).toEqual([]);
    expect(discovery.selectedBg1Id).toBe("");
    const freeze = vi.spyOn(backend, "freezeReview");
    const destination = vi.spyOn(backend, "inspectDestination");
    const choose = vi.spyOn(backend, "chooseGameFolder");
    const root = rootElement();
    await mountApp(root, backend);

    expect(getByRole(root, "heading", { name: "Install Chriz Easy IWD" })).toBeTruthy();
    const cards = root.querySelectorAll<HTMLElement>(".source-card");
    expect(cards).toHaveLength(1);
    expect(cards[0].dataset.source).toBe("bg2");
    expect(cards[0].querySelector(".source-title")?.textContent).toBe("Icewind Dale: Enhanced Edition");
    expect(destination).toHaveBeenCalled();
    for (const call of destination.mock.calls) expect(call.slice(1)).toEqual(["", "iwd-fresh"]);

    fireEvent.click(getByRole(root, "button", { name: "Change Icewind Dale: Enhanced Edition source" }));
    await waitFor(() => expect(choose).toHaveBeenCalledWith("iwdee"));
    // The browsed fixture folder is not clean; choose the found copy again.
    const select = await waitFor(() => getByRole(root, "combobox", { name: "Icewind Dale: Enhanced Edition source" }));
    expect(root.querySelectorAll(".source-card")).toHaveLength(1);
    fireEvent.change(select, { target: { value: "iwd-fresh" } });
    await waitFor(() => expect((getByRole(root, "button", { name: "Install Chriz Easy IWD" }) as HTMLButtonElement).disabled).toBe(false));

    fireEvent.click(getByRole(root, "button", { name: "Install Chriz Easy IWD" }));
    await waitFor(() => expect(freeze).toHaveBeenCalledTimes(1));
    const [name, , path, bg1, bg2] = freeze.mock.calls[0];
    expect([name, path, bg1, bg2]).toEqual(["Chriz Easy IWD", "C:\\Users\\Player\\Games\\Chriz Easy IWD", "", "iwd-fresh"]);
  });

  it("restores a saved IWD source with the iwdee role and keeps the draft's bg1 empty", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const { SETUP_DRAFT_STORAGE_KEY, loadSetupDraft } = await import("../src/setup-draft");
    const backend = new FixtureBackend();
    vi.spyOn(backend, "getStatus").mockResolvedValue({ ...(await backend.getStatus()), mode: "native" });
    vi.spyOn(backend, "listManagedInstallations").mockResolvedValue([]);
    const inspect = vi.spyOn(backend, "inspectGamePath");
    window.localStorage.setItem(SETUP_DRAFT_STORAGE_KEY, JSON.stringify({
      version: 1, recipeVersion: "2026.09-fixture", profileId: null, installationName: "Frost run",
      destinationPath: "D:\\Frost run", destinationAutomatic: false, sourcePaths: { bg1: null, bg2: "C:\\Fixture\\IWDEE" },
      selection: { platform: "windows", features: {}, inputs: {} }, createDesktopShortcut: true,
    }));
    const root = rootElement();
    await mountApp(root, backend);
    expect(inspect.mock.calls).toEqual([["iwdee", "C:\\Fixture\\IWDEE"]]);
    const saved = loadSetupDraft(await backend.getStatus().then(status => ({ ...status, mode: "native" as const })));
    expect(saved.draft?.sourcePaths).toEqual({ bg1: null, bg2: "C:\\Fixture\\IWDEE" });
  });

  it("offers no Radar, game fix, BuffBot or manual-mod pause controls", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const backend = new FixtureBackend({ managedInstallations: [installation()] });
    vi.spyOn(backend, "getUpdates").mockResolvedValue(updatesWithBgExtras());
    const upgrade = vi.spyOn(backend, "inspectModUpgrade");
    const patches = vi.spyOn(backend, "inspectInstallPatches");
    const radar = vi.spyOn(backend, "installRadar");
    const launch = vi.spyOn(backend, "launchInstall");
    const evaluate = backend.evaluateBuild.bind(backend);
    vi.spyOn(backend, "evaluateBuild").mockImplementation(async (selection) => ({
      ...(await evaluate(selection)),
      manualCheckpointOptions: [{ runId: "eeex-main", component: 0, modId: "eeex", modLabel: "EEex", componentLabel: "Core", phase: "main", target: "bg2", unavailableReason: null }],
    }));
    const root = rootElement();
    const app = await mountApp(root, backend);

    expect(root.querySelector("#start-radar")).toBeNull();
    fireEvent.click(getByRole(root, "button", { name: "Play Chriz Easy IWD" }));
    await waitFor(() => expect(launch).toHaveBeenCalledWith("iwd-install", false));
    const tips = root.querySelector(".first-play-guide");
    expect(tips?.querySelectorAll("li")).toHaveLength(1);
    expect(queryByText(root, /F11/)).toBeNull();

    await app.navigate("updates");
    expect(root.querySelector(".install-patches:not(.installed-update-plan)")).toBeNull();
    expect(queryByRole(root, "button", { name: "Check this installation" })).toBeNull();
    expect(queryByRole(root, "button", { name: /overlay/i })).toBeNull();
    expect(queryByRole(root, "button", { name: /app$/ })).toBeNull();
    fireEvent.click(getByRole(root, "button", { name: "Check installed mods" }));
    await waitFor(() => expect((getByRole(root, "button", { name: "Check installed mods" }) as HTMLButtonElement).disabled).toBe(false));
    expect(upgrade).not.toHaveBeenCalled();
    expect(patches).not.toHaveBeenCalled();

    await app.navigate("home");
    fireEvent.click(getByRole(root, "button", { name: "New installation" }));
    await waitFor(() => expect(getByRole(root, "heading", { name: "Install Chriz Easy IWD" })).toBeTruthy());
    await app.navigate("setup");
    expect(queryByText(root, /Add your own mods/)).toBeNull();
    expect(root.querySelector(".manual-breakpoints")).toBeNull();
    expect(radar).not.toHaveBeenCalled();
  });

  it("does not add Radar after an installation completes", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const backend = new FixtureBackend();
    const radar = vi.spyOn(backend, "installRadar");
    const root = rootElement();
    const app = await mountApp(root, backend);
    fireEvent.click(getByRole(root, "button", { name: "Install Chriz Easy IWD" }));
    await finishFixtureBuild(root);
    expect(radar).not.toHaveBeenCalled();
    expect(queryByText(root, /overlay/i)).toBeNull();
    void app;
  });

  it("shows no Baldur's Gate words on any screen it can reach", async () => {
    const m = await iwd();
    const screens: Array<[string, Element]> = [];
    const snapshot = (name: string, root: Element): void => { screens.push([name, root.cloneNode(true) as Element]); };

    for (const stage of ["startup", "games", "setup"] as const) snapshot(`loading ${stage}`, m.loadingScreen(stage));
    snapshot("startup failure", m.startupFailureScreen(new Error("boom"), () => undefined));

    // First run: install, customize, build through every fixture state, then the launcher.
    const backend = new m.FixtureBackend();
    const root = rootElement();
    const app = await m.mountApp(root, backend);
    snapshot("install", root);
    await app.navigate("setup");
    snapshot("setup", root);
    await app.navigate("welcome");
    fireEvent.click(getByRole(root, "button", { name: "Install Chriz Easy IWD" }));
    await finishFixtureBuild(root, step => {
      root.querySelectorAll("details").forEach(details => { details.open = true; });
      snapshot(`build before ${step}`, root);
    });
    snapshot("home after install", root);
    document.body.replaceChildren();

    // A returning player: launcher with details, troubleshooting, Updates and removal.
    const returning = new m.FixtureBackend({ managedInstallations: [installation(), installation({ id: "second", name: "Second run", path: "D:\\Games\\Second run", radarVersion: null })] });
    vi.spyOn(returning, "getUpdates").mockResolvedValue(updatesWithBgExtras());
    const second = rootElement();
    const launcher = await m.mountApp(second, returning);
    second.querySelectorAll("details").forEach(details => { details.open = true; });
    snapshot("home", second);
    fireEvent.click(getByRole(second, "button", { name: "Play Chriz Easy IWD" }));
    await waitFor(() => expect(second.querySelector(".shortcut-feedback")).not.toBeNull());
    snapshot("home after play", second);
    const troubleshooting = second.querySelector<HTMLDetailsElement>("details.troubleshooting")!;
    troubleshooting.open = true;
    troubleshooting.dispatchEvent(new Event("toggle"));
    await waitFor(() => expect(second.querySelector("#game-log")).not.toBeNull());
    snapshot("troubleshooting", second);
    fireEvent.click(getByRole(second, "button", { name: "Delete / remove installation…" }));
    await waitFor(() => expect(second.querySelector(".removal-dialog")).not.toBeNull());
    snapshot("removal dialog", second);
    fireEvent.click(getByRole(second, "button", { name: "Cancel" }));
    await launcher.navigate("updates");
    second.querySelectorAll("details").forEach(details => { details.open = true; });
    snapshot("updates", second);
    fireEvent.click(getByRole(second, "button", { name: "Check installed mods" }));
    await waitFor(() => expect(second.querySelector(".installed-mod-comparison")).not.toBeNull());
    snapshot("updates compared", second);
    snapshot("updates badge", second.querySelector(".update-nav-item")!);

    // Screens built from the same parts outside the main flow.
    const discovery = await returning.discoverGames();
    snapshot("games", m.gamesScreen(discovery, discovery.selectedBg1Id, discovery.selectedBg2Id, () => undefined, () => undefined, () => undefined, () => undefined));
    const evaluation = await returning.evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    const destination = await returning.inspectDestination("D:\\Games\\Chriz Easy IWD", "", discovery.selectedBg2Id);
    snapshot("review", m.reviewScreen(evaluation, destination, discovery.bg2Candidates, () => undefined, () => undefined));
    snapshot("removal unlist", m.installationRemovalDialog({ installId: "x", displayName: "Chriz Easy IWD", managedRoot: "D:\\x", action: "unlist", reason: null, preservedSavePath: null, confirmationToken: "t" }, false, null, () => undefined, () => undefined));

    expect(screens.length).toBeGreaterThan(15);
    for (const [name, node] of screens) expectNoBgWords(node, name);
  });

  it("names the installation, folder and review rows after Icewind Dale", async () => {
    const m = await iwd();
    expect(m.initialState().installationName).toBe("Chriz Easy IWD");
    expect(m.initialState().selectedBg1Id).toBe("");
    const backend = new m.FixtureBackend();
    await expect(backend.getInstallationDefaults()).resolves.toEqual({ name: "Chriz Easy IWD", path: "C:\\Users\\Player\\Games\\Chriz Easy IWD" });
    const discovery = await backend.discoverGames();
    const evaluation = await backend.evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    const review = m.reviewScreen(evaluation, await backend.inspectDestination("D:\\x", "", "iwd-fresh"), discovery.bg2Candidates, () => undefined, () => undefined);
    expect([...review.querySelectorAll("dt")].map(term => term.textContent)).toEqual(["Game source", "Install location", "Choices"]);
    expect(review.querySelector(".ledger li strong")?.textContent).toBe("Copy your game");
  });
});

describe("startup with a backend for another collection", () => {
  it("fails in the IWD build when the backend serves the Baldur's Gate collection", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const backend = new FixtureBackend();
    const { collection: _ignored, ...bgStatus } = await backend.getStatus();
    vi.spyOn(backend, "getStatus").mockResolvedValue(bgStatus);
    const discover = vi.spyOn(backend, "discoverGames");
    const root = rootElement();
    await expect(mountApp(root, backend)).rejects.toThrow("Chriz Easy IWD can't use the game collection it was started with.");
    const alert = getByRole(root, "alert");
    expect(alert.textContent).toContain("Chriz Easy IWD couldn't finish getting ready");
    expect(alert.textContent).toContain("Reinstall Chriz Easy IWD");
    expect(discover).not.toHaveBeenCalled();
    expect(queryByRole(root, "button", { name: "Install Chriz Easy IWD" })).toBeNull();
  });

  it("fails in the IWD build when the backend serves another standalone collection", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const backend = new FixtureBackend();
    const status = await backend.getStatus();
    vi.spyOn(backend, "getStatus").mockResolvedValue({ ...status, collection: { ...status.collection!, flavor: "pst" } });
    const root = rootElement();
    await expect(mountApp(root, backend)).rejects.toThrow("can't use the game collection");
    expect(getByRole(root, "alert").textContent).toContain("backend serves the pst collection");
  });

  it("fails in the IWD build when the collection reports other game sources", async () => {
    const { mountApp, FixtureBackend } = await iwd();
    const backend = new FixtureBackend();
    const status = await backend.getStatus();
    vi.spyOn(backend, "getStatus").mockResolvedValue({ ...status, collection: { ...status.collection!, sources: [{ slot: "bg1", role: "iwdee", label: "Icewind Dale: Enhanced Edition" }] } });
    const root = rootElement();
    await expect(mountApp(root, backend)).rejects.toThrow("can't use the game collection");
  });

  it("fails in the BG build when the backend serves a standalone collection", async () => {
    const { mountApp, FixtureBackend } = await bg();
    const backend = new FixtureBackend();
    vi.spyOn(backend, "getStatus").mockResolvedValue({
      ...(await backend.getStatus()),
      collection: { flavor: "iwd", game: "iwdee", productName: "Chriz Easy IWD", sources: [{ slot: "bg2", role: "iwdee", label: "Icewind Dale: Enhanced Edition" }] },
    });
    const root = rootElement();
    await expect(mountApp(root, backend)).rejects.toThrow("Chriz Easy BG can't use the game collection it was started with.");
    expect(getByRole(root, "alert").textContent).toContain("CEBG couldn't finish getting ready");
  });

  it("reads the collection from the native bootstrap and select_profile payloads", async () => {
    const { NativeBackend } = await iwd();
    const wire = {
      mode: "native", engine_version: "0.1.0", recipe_version: "2026.10", startup_install_id: null,
      collection: { flavor: "iwd", game: "iwdee", product_name: "Chriz Easy IWD", sources: [{ slot: "bg2", role: "iwdee", label: "Icewind Dale: Enhanced Edition" }] },
    };
    const backend = new NativeBackend(async () => wire);
    const expected = { flavor: "iwd", game: "iwdee", productName: "Chriz Easy IWD", sources: [{ slot: "bg2", role: "iwdee", label: "Icewind Dale: Enhanced Edition" }] };
    expect((await backend.getStatus()).collection).toEqual(expected);
    expect((await backend.selectProfile("chris-recommended")).collection).toEqual(expected);
    expect("collection" in await new NativeBackend(async () => ({ ...wire, collection: null })).getStatus()).toBe(false);
  });
});

describe("installation readiness by source slot", () => {
  it("needs only the slots a collection installs from", async () => {
    const { installationReadiness } = await iwd();
    const candidate = { id: "iwd", label: "IWD:EE", path: "C:\\IWD", storefront: "steam" as const, build: "2.7.3.0", freshness: "fresh" as const, eligible: true, findings: [] };
    const base = {
      starting: false, name: "Chriz Easy IWD", bg1: undefined, bg2: candidate,
      destination: { path: "D:\\IWD", safe: true, title: "Ready", detail: "" },
      evaluation: { view: { categories: [], controls: [] }, normalizedSelection: { platform: "windows", features: {}, inputs: {} }, findings: [], plan: { phases: [] }, selectedChoiceCount: 0 },
      evaluationPending: false,
    };
    expect(installationReadiness({ ...base, requiredSlots: ["bg2"] })).toBe(true);
    expect(installationReadiness({ ...base, requiredSlots: ["bg2"], bg2: { ...candidate, eligible: false } })).toBe(false);
    expect(installationReadiness({ ...base, requiredSlots: ["bg2"], bg2: undefined })).toBe(false);
    // Without a slot list both games stay required, as the BG app needs them.
    expect(installationReadiness(base)).toBe(false);
    expect(installationReadiness({ ...base, requiredSlots: [] })).toBe(false);
  });
});

describe("Baldur's Gate flavor", () => {
  it("is the default build and keeps today's words exactly", async () => {
    const { flavor } = await bg();
    expect(flavor.id).toBe("bg");
    for (const value of [flavor, FLAVORS.bg]) expect(value).toEqual({
      id: "bg",
      productName: "Chriz Easy BG",
      shortName: "CEBG",
      gameName: "Baldur's Gate",
      sources: [
        { slot: "bg1", role: "bgee_sod", label: "Baldur's Gate + Siege of Dragonspear", name: "Baldur's Gate", reviewLabel: "BG1 source" },
        { slot: "bg2", role: "bg2ee", label: "Baldur's Gate II", name: "Baldur's Gate II", reviewLabel: "BG2 source" },
      ],
      features: { radar: true, installPatches: true, modUpgrades: true, manualPauses: true, appUpdates: true, buffBotTip: true, radarTip: true },
      text: {
        lookingForGames: "Looking for your Baldur's Gate games…",
        preparingSetupDetail: "Checking your game folders and mod choices. Your original games stay unchanged.",
        gamesIntro: "Choose your original Baldur's Gate games. CEBG will check that they are ready to use.",
        installIntro: "Your games, with Chriz's recommended mods. Ready to install in a separate folder.",
        installSafetyNote: "Your original games stay unchanged. CEBG installs into this separate folder.",
        destinationPending: "The installer will verify that it is separate from both clean source games.",
        sourcesMissingTitle: "Choose both game sources",
        sourcesMissingDetail: "Use Change source for each missing game before CEBG checks this install location.",
        findingGamesAndSetup: "CEBG is finding your games and recommended setup.",
        findingOriginalGames: "CEBG is finding your original games.",
        removalKeepsSources: "Your original game sources and shared downloads stay untouched. Saves stored outside this folder are kept.",
        gameSettingsFailed: "Lefreut's interfaces need this update to offer the trilogy campaigns. Close the game, then use Play again to retry.",
        copyGamesPhase: "Copy your games",
      },
    });
  });

  it("keeps both BG sources and the BG fixture data set", async () => {
    const { FixtureBackend, sourceSlots } = await bg();
    expect(sourceSlots).toEqual(["bg1", "bg2"]);
    const backend = new FixtureBackend();
    expect((await backend.getStatus()).collection).toBeUndefined();
    const discovery = await backend.discoverGames();
    expect(discovery.selectedBg1Id).toBe("bg1-fresh");
    expect(discovery.bg1Candidates.length).toBeGreaterThan(0);
  });
});

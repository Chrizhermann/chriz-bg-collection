// @vitest-environment jsdom

import {
  fireEvent,
  getAllByRole,
  getAllByText,
  getByLabelText,
  getByRole,
  getByText,
  queryByRole,
  queryByText,
  waitFor,
} from "@testing-library/dom";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";

import { mountApp } from "../src/app";
import { FixtureBackend } from "../src/backend";
import type {
  DestinationEvaluation,
  GameDiscovery,
  SelectionEvaluation,
} from "../src/contracts";
import {
  automaticInstallationPath,
  installationReadiness,
  validateInstallationName,
} from "../src/state";

function cleanDiscovery(): GameDiscovery {
  return {
    selectedBg1Id: "bg1",
    selectedBg2Id: "bg2",
    bg1Candidates: [{
      id: "bg1",
      label: "BG:EE + SoD — Steam — clean",
      path: "C:\\Games\\Baldur's Gate Enhanced Edition",
      storefront: "steam",
      build: "2.7.3.0",
      freshness: "fresh",
      eligible: true,
      findings: ["Clean supported installation."],
    }],
    bg2Candidates: [{
      id: "bg2",
      label: "BGII:EE — GOG — clean",
      path: "D:\\Games\\Baldur's Gate II Enhanced Edition",
      storefront: "gog",
      build: "2.7.3.0",
      freshness: "fresh",
      eligible: true,
      findings: ["Clean supported installation."],
    }],
  };
}

class InstallBackend extends FixtureBackend {
  readonly freezeCalls: Array<{ name: string; path: string; bg1: string; bg2: string }> = [];
  startCalls = 0;

  override discoverGames() {
    return Promise.resolve(cleanDiscovery());
  }

  override listManagedInstallations() {
    return Promise.resolve([]);
  }

  override async freezeReview(
    displayName: string,
    selection: Parameters<FixtureBackend["freezeReview"]>[1],
    destination: string,
    bg1CandidateId: string,
    bg2CandidateId: string,
  ) {
    this.freezeCalls.push({ name: displayName, path: destination, bg1: bg1CandidateId, bg2: bg2CandidateId });
    return super.freezeReview(displayName, selection, destination, bg1CandidateId, bg2CandidateId);
  }

  override startBuild() {
    this.startCalls += 1;
    return Promise.resolve({ runId: "install-run" });
  }
}

describe("CEBG install-first experience", () => {
  afterEach(() => document.body.replaceChildren());

  it("opens on the one-screen installation with quiet product navigation", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    await mountApp(root, new InstallBackend());

    expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(getAllByText(root, "Chriz Easy BG").length).toBeGreaterThan(0);
    expect(getByText(root, "0.1 Alpha")).toBeTruthy();
    expect(getByRole(root, "button", { name: "My installs" })).toBeTruthy();
    expect(getByRole(root, "button", { name: "Updates" })).toBeTruthy();
    expect(root.querySelector("aside")).toBeNull();
    expect(root.textContent?.toLowerCase()).not.toContain("campaign");

    expect(getAllByRole(root, "heading", { level: 2, name: "Found source" })).toHaveLength(2);
    expect(queryByRole(root, "combobox", { name: "Baldur's Gate source" })).toBeNull();
    expect(getByText(root, "C:\\Games\\Baldur's Gate Enhanced Edition")).toBeTruthy();
    expect(getAllByText(root, "Ready")).toHaveLength(2);
    const details = getAllByRole(root, "group");
    expect(details.filter((item) => item.tagName === "DETAILS").every((item) => !(item as HTMLDetailsElement).open)).toBe(true);

    expect((getByLabelText(root, "Install name") as HTMLInputElement).value).toBe("Chriz Easy BG");
    expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toBe("C:\\Users\\Chris\\Games\\Chriz Easy BG");
    expect(getByText(root, /original games stay unchanged/i)).toBeTruthy();
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("keeps multiple detected sources selectable and their findings collapsed", async () => {
    class MultipleBackend extends InstallBackend {
      override async discoverGames() {
        const discovery = cleanDiscovery();
        return {
          ...discovery,
          bg1Candidates: [
            ...discovery.bg1Candidates,
            { ...discovery.bg1Candidates[0]!, id: "dirty", label: "BG:EE + SoD — modified", path: "C:\\Other BGEE", eligible: false, freshness: "modified" as const, findings: ["Mods are installed."] },
          ],
        };
      }
    }
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new MultipleBackend());

    const select = getByRole(root, "combobox", { name: "Baldur's Gate source" });
    expect(select).toBeTruthy();
    expect(root.querySelector<HTMLDetailsElement>("[data-source='bg1'] details")?.open).toBe(false);
    await user.selectOptions(select, "dirty");
    await waitFor(() => expect(getByRole(root, "heading", { level: 2, name: "Needs attention" })).toBeTruthy());
    const sourceDetails = root.querySelector<HTMLDetailsElement>("[data-source='bg1'] details");
    expect(sourceDetails?.open).toBe(false);
    sourceDetails!.open = true;
    expect(getByText(root, "Mods are installed.")).toBeTruthy();
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("treats recipe findings as notices rather than installation blockers", () => {
    const destination: DestinationEvaluation = { path: "D:\\CEBG", safe: true, title: "Ready", detail: "Safe." };
    const evaluation: SelectionEvaluation = {
      view: { categories: [], controls: [] },
      normalizedSelection: { platform: "windows", features: {}, inputs: {} },
      findings: [{ rule: "notice", featureId: "optional", message: "An optional component is omitted." }],
      plan: { phases: [] },
      selectedChoiceCount: 0,
    };
    const [bg1, bg2] = [cleanDiscovery().bg1Candidates[0]!, cleanDiscovery().bg2Candidates[0]!];

    expect(installationReadiness({ starting: false, name: "Chriz Easy BG", bg1, bg2, destination, evaluation, evaluationPending: false })).toBe(true);
    expect(installationReadiness({ starting: false, name: "Chriz Easy BG", bg1, bg2, destination, evaluation, evaluationPending: true })).toBe(false);
    expect(installationReadiness({ starting: false, name: "Chriz Easy BG", bg1, bg2, destination: { ...destination, safe: false }, evaluation, evaluationPending: false })).toBe(false);
    expect(installationReadiness({ starting: true, name: "Chriz Easy BG", bg1, bg2, destination, evaluation, evaluationPending: false })).toBe(false);
    expect(installationReadiness({ starting: false, name: "Bad|Name", bg1, bg2, destination, evaluation, evaluationPending: false })).toBe(false);
  });

  it("preserves choices through Customize and returns to the installation", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new InstallBackend());

    await user.click(getByRole(root, "button", { name: "Customize" }));
    expect(getByRole(root, "heading", { level: 1, name: "Customize your installation" })).toBeTruthy();
    const optional = getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement;
    await user.click(optional);
    await waitFor(() => expect((getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement).checked).toBe(true));
    await user.click(getByRole(root, "button", { name: "Done" }));

    expect(getByRole(root, "heading", { level: 1, name: "Install Chriz Easy BG" })).toBeTruthy();
    expect(getByText(root, "3 choices included")).toBeTruthy();
    await user.click(getByRole(root, "button", { name: "Customize" }));
    expect((getByRole(root, "checkbox", { name: /Companion conversations/ }) as HTMLInputElement).checked).toBe(true);
  });

  it("freezes and starts exactly once under double activation", async () => {
    let releaseFreeze!: () => void;
    const freezeGate = new Promise<void>((resolve) => { releaseFreeze = resolve; });
    class SlowFreezeBackend extends InstallBackend {
      override async freezeReview(...args: Parameters<InstallBackend["freezeReview"]>) {
        await freezeGate;
        return super.freezeReview(...args);
      }
    }
    const backend = new SlowFreezeBackend();
    const root = document.createElement("div");
    document.body.append(root);
    await mountApp(root, backend);
    const install = getByRole(root, "button", { name: "Install Chriz Easy BG" });

    fireEvent.click(install);
    fireEvent.click(install);
    expect((getByRole(root, "button", { name: "Starting installation…" }) as HTMLButtonElement).disabled).toBe(true);
    releaseFreeze();
    await waitFor(() => expect(getByRole(root, "heading", { level: 1, name: "Installation progress" })).toBeTruthy());
    expect(backend.freezeCalls).toHaveLength(1);
    expect(backend.freezeCalls[0]).toMatchObject({ name: "Chriz Easy BG", path: "C:\\Users\\Chris\\Games\\Chriz Easy BG", bg1: "bg1", bg2: "bg2" });
    expect(backend.startCalls).toBe(1);
  });

  it("keeps the newest source inspection when async checks resolve out of order", async () => {
    const resolvers = new Map<string, (value: DestinationEvaluation) => void>();
    class RacingBackend extends InstallBackend {
      override async discoverGames() {
        const discovery = cleanDiscovery();
        return { ...discovery, bg1Candidates: [...discovery.bg1Candidates, { ...discovery.bg1Candidates[0]!, id: "second", label: "BG:EE + SoD — second clean", path: "C:\\Second BGEE" }] };
      }

      override inspectDestination(path: string, bg1: string) {
        if (bg1 === "bg1") return super.inspectDestination(path, bg1, "bg2");
        return new Promise<DestinationEvaluation>((resolve) => resolvers.set(bg1, resolve));
      }
    }
    const backend = new RacingBackend();
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, backend);
    const source = getByRole(root, "combobox", { name: "Baldur's Gate source" });

    await user.selectOptions(source, "second");
    await user.selectOptions(getByRole(root, "combobox", { name: "Baldur's Gate source" }), "bg1");
    resolvers.get("second")?.({ path: "C:\\Users\\Chris\\Games\\Chriz Easy BG", safe: false, title: "Stale rejection", detail: "This result is stale." });

    await waitFor(() => expect(getByText(root, "Ready to install")).toBeTruthy());
    expect(queryByText(root, "Stale rejection")).toBeNull();
    expect((getByRole(root, "button", { name: "Install Chriz Easy BG" }) as HTMLButtonElement).disabled).toBe(false);
  });

  it("couples a valid name to the automatic folder until the location is edited", async () => {
    const root = document.createElement("div");
    document.body.append(root);
    const user = userEvent.setup();
    await mountApp(root, new InstallBackend());
    const name = getByLabelText(root, "Install name") as HTMLInputElement;

    await user.clear(name);
    await user.type(name, "My BG");
    fireEvent.change(name);
    await waitFor(() => expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toBe("C:\\Users\\Chris\\Games\\My BG"));

    const location = getByLabelText(root, "Install location") as HTMLInputElement;
    await user.clear(location);
    await user.type(location, "D:\\Games\\Stream BG");
    fireEvent.change(location);
    await waitFor(() => expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toBe("D:\\Games\\Stream BG"));

    const renamed = getByLabelText(root, "Install name") as HTMLInputElement;
    await user.clear(renamed);
    await user.type(renamed, "Renamed BG");
    fireEvent.change(renamed);
    expect((getByLabelText(root, "Install location") as HTMLInputElement).value).toBe("D:\\Games\\Stream BG");

    expect(validateInstallationName("Bad<Name")).toMatch(/cannot contain/i);
    expect(validateInstallationName("\u0001name")).toMatch(/control/i);
    expect(validateInstallationName("   ")).toMatch(/enter/i);
    expect(validateInstallationName(".")).toMatch(/folder/i);
    expect(validateInstallationName("Ordinary — 이름")).toBeNull();
    expect(automaticInstallationPath("C:\\Users\\Chris\\Games\\Chriz Easy BG", "Ordinary — 이름")).toBe("C:\\Users\\Chris\\Games\\Ordinary — 이름");
  });
});

import { describe, expect, it } from "vitest";
import { updateBuildPhase } from "../src/build-progress";
import type { BuildPhase } from "../src/contracts";

describe("native installation phase progress", () => {
  it.each([
    ["Prepare Baldur's Gate: Enhanced Edition", "bg1-preparation"],
    ["Prepare Baldur's Gate II: Enhanced Edition", "bg2-preparation"],
    ["Build the EET campaign", "eet-initialization"],
    ["Install the curated collection", "main"],
    ["Finalize EET", "eet-finalization"],
    ["Apply reviewed final compatibility fixes", "post-eet-end"],
  ])("shows the observed phase on a cold resume: %s", (name, id) => {
    expect(updateBuildPhase([], name)).toEqual([{ id, title: name, detail: "", state: "current" }]);
  });

  it("advances the existing projected phases without replacing their descriptions", () => {
    const phases: readonly BuildPhase[] = [
      { id: "bg1-preparation", title: "BG1", detail: "Prepare BG1", state: "current" },
      { id: "main", title: "Mods", detail: "Install mods", state: "pending" },
      { id: "eet-finalization", title: "Finish", detail: "Finish EET", state: "pending" },
    ];
    expect(updateBuildPhase(phases, "Install the curated collection")).toEqual([
      { ...phases[0], state: "done" },
      { ...phases[1], state: "current" },
      { ...phases[2], state: "pending" },
    ]);
    expect(phases.map(phase => phase.state)).toEqual(["current", "pending", "pending"]);
  });

  it("appends later observed phases without inventing skipped earlier phases", () => {
    const first = updateBuildPhase([], "Install the curated collection");
    const next = updateBuildPhase(first, "Finalize EET");
    expect(next).toEqual([
      { ...first[0], state: "done" },
      { id: "eet-finalization", title: "Finalize EET", detail: "", state: "current" },
    ]);
  });

  it("does not duplicate a repeated phase event", () => {
    const phases = updateBuildPhase([], "Install the curated collection");
    expect(updateBuildPhase(phases, "Install the curated collection")).toEqual(phases);
  });

  it("labels an unrecognized native phase truthfully", () => {
    expect(updateBuildPhase([], "Prepare additional game data")).toEqual([
      { id: "native:Prepare additional game data", title: "Prepare additional game data", detail: "", state: "current" },
    ]);
  });

  it("treats names inherited by ordinary objects as unknown phases", () => {
    expect(updateBuildPhase([], "constructor")).toEqual([
      { id: "native:constructor", title: "constructor", detail: "", state: "current" },
    ]);
  });

  it("does not call pending work complete when an unknown phase is observed or repeated", () => {
    const phases: readonly BuildPhase[] = [
      { id: "main", title: "Mods", detail: "Install mods", state: "current" },
      { id: "eet-finalization", title: "Finish", detail: "Finish EET", state: "pending" },
    ];
    const next = updateBuildPhase(phases, "Prepare additional game data");
    expect(next).toEqual([
      { ...phases[0], state: "done" },
      phases[1],
      { id: "native:Prepare additional game data", title: "Prepare additional game data", detail: "", state: "current" },
    ]);
    expect(updateBuildPhase(next, "Prepare additional game data")).toEqual(next);
  });
});

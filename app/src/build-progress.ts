import type { BuildPhase } from "./contracts";

const nativePhaseIds: Readonly<Record<string, string>> = {
  "Prepare Baldur's Gate: Enhanced Edition": "bg1-preparation",
  "Prepare Baldur's Gate II: Enhanced Edition": "bg2-preparation",
  "Build the EET campaign": "eet-initialization",
  "Install the curated collection": "main",
  "Finalize EET": "eet-finalization",
  "Apply reviewed final compatibility fixes": "post-eet-end",
};

/** Project the phase actually reported by the worker, including a cold resume. */
export function updateBuildPhase(phases: readonly BuildPhase[], name: string): readonly BuildPhase[] {
  const knownId = Object.hasOwn(nativePhaseIds, name) ? nativePhaseIds[name] : undefined;
  const id = knownId ?? `native:${name}`;
  const matchedIndex = phases.findIndex(phase => phase.id === id);
  if (knownId !== undefined && matchedIndex >= 0) {
    return phases.map((phase, index) => ({
      ...phase,
      state: index < matchedIndex ? "done" : index === matchedIndex ? "current" : "pending",
    }));
  }

  // Reopened installs do not have a projected recipe. Keep only observed phases;
  // do not invent skipped phases or mark unobserved pending work complete.
  const observed = phases.map((phase): BuildPhase => ({
    ...phase,
    state: phase.id === id ? "current" : phase.state === "current" ? "done" : phase.state,
  }));
  if (matchedIndex >= 0) return observed;
  return [...observed, { id, title: name, detail: "", state: "current" }];
}

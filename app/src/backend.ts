import type {
  BackendStatus,
  BuildSnapshot,
  DestinationEvaluation,
  FixtureOptions,
  FrozenReview,
  GameDiscovery,
  ManagedInstallation,
  NormalizedSelection,
  PhaseSummary,
  SelectionEvaluation,
  UpdateSummary,
} from "./contracts";

export interface Backend {
  // This is the UI adapter boundary, not the eventual Tauri wire shape. Task 23
  // may map snake_case command payloads without leaking transport casing here.
  getStatus(): Promise<BackendStatus>;
  discoverGames(): Promise<GameDiscovery>;
  inspectDestination(path: string): Promise<DestinationEvaluation>;
  evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation>;
  freezeReview(
    selection: NormalizedSelection,
    destination: string,
    gameLabels: readonly string[],
  ): Promise<FrozenReview>;
  getBuildSnapshot(): Promise<BuildSnapshot>;
  advanceBuild(): Promise<BuildSnapshot>;
  retryBuild(): Promise<BuildSnapshot>;
  exportDiagnostics(): Promise<{ readonly path: string }>;
  listManagedInstallations(): Promise<readonly ManagedInstallation[]>;
  getUpdates(): Promise<UpdateSummary>;
}

const phases: readonly PhaseSummary[] = [
  { id: "preparation", title: "Prepare clean game copies", detail: "Verify sources and create a separate workspace." },
  { id: "bg1", title: "Build the BG1 campaign", detail: "Apply the pinned pre-merge recipe." },
  { id: "merge", title: "Merge with EET", detail: "Create the continuous campaign world." },
  { id: "main", title: "Build the main campaign", detail: "Install curated content and rules in order." },
  { id: "final", title: "Finalization and reviewed tail", detail: "Finalize EET and apply the audited post-merge tail." },
];

const baseControls = [
  { id: "core-fixes", title: "Curated foundation", description: "The required fixes and EET foundation owned by the recipe.", category: "Foundation", decision: "mandatory", readiness: "ready", parent: null, selected: true, interactive: false, unavailableReason: null, inputs: [] },
  { id: "recommended-rules", title: "Recommended rules balance", description: "The collection author's tested default rules profile.", category: "Rules", decision: "default", readiness: "ready", parent: null, selected: true, interactive: true, unavailableReason: null, inputs: [] },
  { id: "companion-conversations", title: "Companion conversations", description: "An optional layer of additional party interactions.", category: "Story", decision: "optional", readiness: "experimental", parent: null, selected: false, interactive: true, unavailableReason: null, inputs: [] },
  { id: "blocked-restoration", title: "Experimental quest restoration", description: "A visible recipe choice that is not ready for this release.", category: "Story", decision: "optional", readiness: "blocked", parent: null, selected: false, interactive: false, unavailableReason: "Deferred until its installer can be reproduced safely.", inputs: [] },
] as const;

function wait(milliseconds: number): Promise<void> {
  return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds));
}

export class FixtureBackend implements Backend {
  readonly #options: FixtureOptions;
  #evaluationCall = 0;
  #buildIndex = 0;

  constructor(options: FixtureOptions = {}) {
    this.#options = options;
  }

  getStatus(): Promise<BackendStatus> {
    return Promise.resolve({ mode: "fixture", engineVersion: "0.1.0", recipeVersion: "2026.09-fixture" });
  }

  discoverGames(): Promise<GameDiscovery> {
    return Promise.resolve({
      selectedBg1Id: "bg1-fresh",
      selectedBg2Id: "bg2-fresh",
      bg1Candidates: [
        { id: "bg1-fresh", label: "BG:EE + SoD — clean", path: this.#options.textOverrides?.gamePath ?? "C:\\Fixture\\BGEE", freshness: "fresh", eligible: true, findings: ["Clean supported installation."] },
        { id: "bg1-modified", label: "BG:EE — modified", path: "C:\\Fixture\\BGEE Modified", freshness: "modified", eligible: false, findings: ["Files differ from a clean store installation."] },
        { id: "bg1-old", label: "BG:EE — old version", path: "C:\\Fixture\\BGEE Old", freshness: "unsupported-version", eligible: false, findings: ["The installed game version is not supported."] },
        { id: "bg1-nosod", label: "BG:EE — SoD missing", path: "C:\\Fixture\\BGEE No SoD", freshness: "missing-sod", eligible: false, findings: ["Siege of Dragonspear is required for this EET recipe."] },
      ],
      bg2Candidates: [
        { id: "bg2-fresh", label: "BGII:EE — clean", path: "C:\\Fixture\\BG2EE", freshness: "fresh", eligible: true, findings: ["Clean supported installation."] },
        { id: "bg2-store", label: "BGII:EE — verify storefront", path: "C:\\Fixture\\BG2EE Other", freshness: "unverified-storefront", eligible: false, findings: ["This storefront layout has not been verified yet."] },
      ],
    });
  }

  inspectDestination(path: string): Promise<DestinationEvaluation> {
    const normalized = path.trim();
    const safe = normalized.length > 3 && !normalized.toLowerCase().includes("steamapps");
    return Promise.resolve({
      path: normalized,
      safe,
      title: safe ? "Safe separate destination" : "Choose a separate destination",
      detail: safe ? "The source games and their saves will remain untouched." : "The collection cannot be built inside a store-managed game folder.",
      requiredSpace: "62 GB",
      availableSpace: "184 GB",
    });
  }

  async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    const call = this.#evaluationCall++;
    const delay = this.#options.evaluationDelays?.[call] ?? 0;
    if (delay > 0) await wait(delay);
    const controls = baseControls.map((control) => ({
      ...control,
      title: control.id === "companion-conversations" ? this.#options.textOverrides?.featureTitle ?? control.title : control.title,
      unavailableReason: control.id === "blocked-restoration" ? this.#options.textOverrides?.unavailableReason ?? control.unavailableReason : control.unavailableReason,
      selected: selection.features[control.id] ?? control.selected,
    }));
    return {
      view: { categories: ["Foundation", "Rules", "Story"], controls },
      normalizedSelection: { platform: selection.platform, features: Object.fromEntries(controls.map((control) => [control.id, control.selected])), inputs: selection.inputs },
      findings: [{ rule: "blocked-choice-omitted", featureId: "blocked-restoration", message: "Experimental quest restoration remains visible but is omitted." }],
      plan: { phases },
      selectedChoiceCount: controls.filter((control) => control.selected).length,
    };
  }

  async freezeReview(selection: NormalizedSelection, destination: string, gameLabels: readonly string[]): Promise<FrozenReview> {
    const evaluation = await this.evaluateBuild(selection);
    return { digest: "fixture-review-8d6d75", destination, gameLabels, evaluation };
  }

  getBuildSnapshot(): Promise<BuildSnapshot> {
    return Promise.resolve(this.#snapshot());
  }

  advanceBuild(): Promise<BuildSnapshot> {
    this.#buildIndex = Math.min(this.#buildIndex + 1, 4);
    return Promise.resolve(this.#snapshot());
  }

  retryBuild(): Promise<BuildSnapshot> {
    this.#buildIndex = 3;
    return Promise.resolve(this.#snapshot());
  }

  exportDiagnostics(): Promise<{ readonly path: string }> {
    return Promise.resolve({ path: "C:\\Fixture\\diagnostics\\build-report.zip" });
  }

  listManagedInstallations(): Promise<readonly ManagedInstallation[]> {
    return Promise.resolve([{ id: "fixture-install", name: this.#options.textOverrides?.campaignName ?? "Chriz EET — Stream test", path: "D:\\Fixture Campaigns\\Chriz EET Stream Test", status: "Ready to play", receiptPath: "D:\\Fixture Campaigns\\Chriz EET Stream Test\\install-receipt.json" }]);
  }

  getUpdates(): Promise<UpdateSummary> {
    return Promise.resolve({ app: "0.1.0-alpha.1 fixture", recipe: "2026.09 fixture", message: "Fixture data only. Signed update checks are implemented in a later task." });
  }

  #snapshot(): BuildSnapshot {
    const states = ["waiting-manual", "attention", "failed", "running", "complete"] as const;
    const headlines = { "waiting-manual": "Manual archive needed", attention: "Your attention is needed", failed: "A fixture step failed", running: "Build in progress", complete: "Build verified" } as const;
    const details = {
      "waiting-manual": "Add the named archive to the staging folder, then continue.",
      attention: "Review the fixture installer prompt before continuing.",
      failed: "Nothing was changed outside the fixture. Retry or export diagnostics.",
      running: "The fixture ledger is advancing through the reviewed plan.",
      complete: "The campaign copy and immutable receipt are ready.",
    } as const;
    const state = states[this.#buildIndex] ?? "complete";
    const currentIndex = state === "complete" ? phases.length : Math.min(this.#buildIndex, phases.length - 1);
    return {
      state,
      headline: headlines[state],
      detail: details[state],
      phases: phases.map((phase, index) => ({
        ...phase,
        state: state === "failed" && index === currentIndex ? "failed" : index < currentIndex ? "done" : index === currentIndex ? "current" : "pending",
      })),
      logTail: ["fixture: validated immutable review fixture-review-8d6d75", this.#options.textOverrides?.logLine ?? "fixture: no game files were touched"],
      manualArchiveName: state === "waiting-manual" ? "example-manual-mod.zip" : null,
    };
  }
}

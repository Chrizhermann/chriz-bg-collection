import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  BackendStatus,
  BuildSnapshot,
  CommandErrorPayload,
  DestinationEvaluation,
  FeatureControl,
  FixtureOptions,
  FrozenReview,
  Freshness,
  GameCandidate,
  GameDiscovery,
  GameRole,
  InstallationDefaults,
  ManualArchiveSupply,
  ManualDownloadRequirement,
  ManagedInstallation,
  NormalizedSelection,
  PhaseSummary,
  RunEventEnvelope,
  RunSnapshot,
  SelectionEvaluation,
  StartBuildResponse,
  Storefront,
  UpdateSummary,
} from "./contracts";

export interface Backend {
  // This is the UI adapter boundary, not the eventual Tauri wire shape. Task 23
  // may map snake_case command payloads without leaking transport casing here.
  getStatus(): Promise<BackendStatus>;
  selectProfile(profileId: string): Promise<BackendStatus>;
  getInstallationDefaults(): Promise<InstallationDefaults>;
  discoverGames(): Promise<GameDiscovery>;
  chooseGameFolder(role: GameRole): Promise<GameCandidate | null>;
  inspectGamePath(role: GameRole, path: string): Promise<GameCandidate>;
  chooseDestinationFolder(bg1CandidateId: string, bg2CandidateId: string): Promise<DestinationEvaluation | null>;
  inspectDestination(path: string, bg1CandidateId: string, bg2CandidateId: string): Promise<DestinationEvaluation>;
  evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation>;
  inspectManualDownloads(selection: NormalizedSelection): Promise<ManualDownloadRequirement[]>;
  freezeReview(
    displayName: string,
    selection: NormalizedSelection,
    destination: string,
    bg1CandidateId: string,
    bg2CandidateId: string,
  ): Promise<FrozenReview>;
  startBuild(reviewToken: string, onEvent: (event: RunEventEnvelope) => void): Promise<StartBuildResponse>;
  resumeBuild(installId: string, onEvent: (event: RunEventEnvelope) => void): Promise<StartBuildResponse>;
  supplyManualArchive(artifactId: string): Promise<ManualArchiveSupply | null>;
  openManualSource(artifactId: string): Promise<void>;
  getRunSnapshot(runId: string): Promise<RunSnapshot>;
  continueWaiting(runId: string): Promise<void>;
  pauseRun(runId: string): Promise<void>;
  cancelRun(runId: string): Promise<void>;
  getBuildSnapshot(): Promise<BuildSnapshot>;
  advanceBuild(): Promise<BuildSnapshot>;
  retryBuild(): Promise<BuildSnapshot>;
  exportDiagnostics(installId: string): Promise<{ readonly path: string } | null>;
  listManagedInstallations(): Promise<ManagedInstallation[]>;
  launchInstall(installId: string): Promise<void>;
  openInstallFolder(installId: string): Promise<void>;
  createDesktopShortcut(installId: string): Promise<{ readonly path: string }>;
  installRadar(installId: string): Promise<void>;
  getUpdates(): Promise<UpdateSummary>;
  installAppUpdate(version: string): Promise<void>;
  activateRecipeUpdate(version: string): Promise<void>;
}

export type InvokeCommand = (
  command: string,
  args?: Record<string, unknown>,
) => Promise<unknown>;

export type EventChannelFactory = (onMessage: (event: unknown) => void) => unknown;

type BootstrapWire = {
  readonly mode: "native";
  readonly application_version?: string;
  readonly engine_version: string;
  readonly recipe_version: string | null;
  readonly startup_install_id: string | null;
  readonly profiles?: BackendStatus["profiles"];
  readonly selected_profile?: string;
};

type GameCandidateWire = {
  readonly id: string;
  readonly label: string;
  readonly path: string;
  readonly storefront: Storefront;
  readonly build: string | null;
  readonly freshness: Freshness;
  readonly eligible: boolean;
  readonly findings: readonly string[];
};

type GameDiscoveryWire = {
  readonly bg1_candidates: readonly GameCandidateWire[];
  readonly bg2_candidates: readonly GameCandidateWire[];
  readonly selected_bg1_id: string;
  readonly selected_bg2_id: string;
};

type FeatureControlWire = Omit<FeatureControl, "unavailableReason"> & {
  readonly unavailable_reason: string | null;
};

type SelectionEvaluationWire = {
  readonly view: {
    readonly categories: readonly string[];
    readonly controls: readonly FeatureControlWire[];
  };
  readonly normalized_selection: NormalizedSelection;
  readonly findings: readonly {
    readonly rule: string;
    readonly feature_id: string;
    readonly message: string;
  }[];
  readonly plan: {
    readonly phases: readonly PhaseSummary[];
  };
  readonly selected_choice_count: number;
};

type FrozenReviewWire = {
  readonly review_token: string;
  readonly digest: string;
  readonly display_name: string;
  readonly destination: string;
  readonly game_labels: readonly string[];
  readonly evaluation: SelectionEvaluationWire;
};

type RunEventEnvelopeWire = {
  readonly run_id: string;
  readonly sequence_as_string: string;
  readonly event: RunEventEnvelope["event"];
};

type StartBuildWire = { readonly run_id: string };

type ManualArchiveSupplyWire = {
  readonly artifact_id: string;
  readonly filename: string;
  readonly sha256: string;
  readonly length: number;
};

type ManualDownloadRequirementWire = {
  readonly artifact_id: string;
  readonly mod_ids: readonly string[];
  readonly title: string;
  readonly filename: string;
  readonly length: number;
  readonly ready: boolean;
  readonly detail: string | null;
};

type ManagedInstallationWire = {
  readonly id: string;
  readonly name: string;
  readonly path: string;
  readonly status: string;
  readonly receipt_path: string | null;
  readonly launch_path: string | null;
  readonly completed_at_millis: number | null;
  readonly available: boolean;
  readonly resumable: boolean;
  readonly recipe_version?: string | null;
  readonly radar_version?: string | null;
  readonly consistency?: {
    readonly state: "matches" | "changed" | "unavailable";
    readonly detail: string;
    readonly component_count: number;
    readonly mod_count: number;
  } | null;
};

type UpdateSummaryWire = {
  readonly checked_at: string | null;
  readonly network_state: UpdateSummary["networkState"];
  readonly application: {
    readonly state: UpdateSummary["application"]["state"];
    readonly current_version: string;
    readonly available_version: string | null;
    readonly detail: string;
    readonly release_notes?: string | null;
  };
  readonly recipe: {
    readonly state: UpdateSummary["recipe"]["state"];
    readonly current_version: string;
    readonly available_version: string | null;
    readonly disposition: UpdateSummary["recipe"]["disposition"];
    readonly detail: string;
    readonly changes: readonly {
      readonly title: string;
      readonly summary: string;
      readonly save_applicability: UpdateSummary["recipe"]["changes"][number]["saveApplicability"];
      readonly urgency: UpdateSummary["recipe"]["changes"][number]["urgency"];
      readonly condition_note: string | null;
    }[];
  };
  readonly managed_copies: readonly {
    readonly install_id: string;
    readonly name: string;
    readonly path: string;
    readonly installed_recipe_version: string | null;
    readonly state: UpdateSummary["managedCopies"][number]["state"];
    readonly detail: string;
  }[];
  readonly radar?: {
    readonly state: "up-to-date" | "available" | "not-installed" | "offline";
    readonly current_version: string | null;
    readonly available_version: string | null;
    readonly detail: string;
    readonly release_notes?: string | null;
  } | null;
};

type RunSnapshotWire = {
  readonly run_id: string;
  readonly status: string;
  readonly events: readonly RunEventEnvelopeWire[];
  readonly report: RunSnapshot["report"];
  readonly error: CommandErrorPayload | null;
};

const invokeNative: InvokeCommand = (
  command: string,
  args?: Record<string, unknown>,
): Promise<unknown> => invoke<unknown>(command, args);

const createNativeEventChannel: EventChannelFactory = (onMessage) =>
  new Channel<unknown>(onMessage);

function isCommandErrorPayload(value: unknown): value is CommandErrorPayload {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Partial<CommandErrorPayload>;
  return typeof candidate.code === "string"
    && typeof candidate.message === "string"
    && typeof candidate.recovery_action === "string"
    && typeof candidate.technical_detail === "string";
}

export class BackendCommandError extends Error {
  readonly code: string;
  readonly recoveryAction: string;
  readonly technicalDetail: string;

  constructor(payload: CommandErrorPayload) {
    super(payload.message);
    this.name = "BackendCommandError";
    this.code = payload.code;
    this.recoveryAction = payload.recovery_action;
    this.technicalDetail = payload.technical_detail;
  }
}

function normalizeCommandError(error: unknown): BackendCommandError {
  if (error instanceof BackendCommandError) return error;
  if (isCommandErrorPayload(error)) return new BackendCommandError(error);
  const detail = error instanceof Error ? error.message : String(error);
  return new BackendCommandError({
    code: "native_command_failed",
    message: "The installer could not complete that native check.",
    recovery_action: "Retry the check. If it still fails, keep the technical detail for diagnosis.",
    technical_detail: detail,
  });
}

function projectCandidate(candidate: GameCandidateWire): GameCandidate {
  return candidate;
}

function projectEvaluation(evaluation: SelectionEvaluationWire): SelectionEvaluation {
  return {
    view: {
      categories: evaluation.view.categories,
      controls: evaluation.view.controls.map((control) => {
        const { unavailable_reason: unavailableReason, ...rest } = control;
        return { ...rest, unavailableReason };
      }),
    },
    normalizedSelection: evaluation.normalized_selection,
    findings: evaluation.findings.map((finding) => ({
      rule: finding.rule,
      featureId: finding.feature_id,
      message: finding.message,
    })),
    plan: evaluation.plan,
    selectedChoiceCount: evaluation.selected_choice_count,
  };
}

function projectRunEvent(event: RunEventEnvelopeWire): RunEventEnvelope {
  return {
    runId: event.run_id,
    sequenceAsString: event.sequence_as_string,
    event: event.event,
  };
}

export class NativeBackend implements Backend {
  readonly #invoke: InvokeCommand;
  readonly #eventChannel: EventChannelFactory;

  constructor(
    invokeCommand: InvokeCommand = invokeNative,
    eventChannel: EventChannelFactory = createNativeEventChannel,
  ) {
    this.#invoke = invokeCommand;
    this.#eventChannel = eventChannel;
  }

  async getStatus(): Promise<BackendStatus> {
    const status = await this.#command<BootstrapWire>("bootstrap");
    return {
      mode: status.mode,
      applicationVersion: status.application_version,
      engineVersion: status.engine_version,
      recipeVersion: status.recipe_version,
      startupInstallId: status.startup_install_id,
      profiles: status.profiles,
      selectedProfile: status.selected_profile,
    };
  }

  async selectProfile(profileId: string): Promise<BackendStatus> {
    const status = await this.#command<BootstrapWire>("select_profile", { profileId });
    return {
      mode: status.mode, applicationVersion: status.application_version, engineVersion: status.engine_version,
      recipeVersion: status.recipe_version, startupInstallId: status.startup_install_id,
      profiles: status.profiles, selectedProfile: status.selected_profile,
    };
  }

  getInstallationDefaults(): Promise<InstallationDefaults> {
    return this.#command("installation_defaults");
  }

  async discoverGames(): Promise<GameDiscovery> {
    const discovery = await this.#command<GameDiscoveryWire>("discover_games");
    return {
      bg1Candidates: discovery.bg1_candidates.map(projectCandidate),
      bg2Candidates: discovery.bg2_candidates.map(projectCandidate),
      selectedBg1Id: discovery.selected_bg1_id,
      selectedBg2Id: discovery.selected_bg2_id,
    };
  }

  async chooseGameFolder(role: GameRole): Promise<GameCandidate | null> {
    const candidate = await this.#command<GameCandidateWire | null>("choose_game_folder", { role });
    return candidate === null ? null : projectCandidate(candidate);
  }

  async inspectGamePath(role: GameRole, path: string): Promise<GameCandidate> {
    return projectCandidate(await this.#command<GameCandidateWire>("inspect_game_path", { role, path }));
  }

  chooseDestinationFolder(bg1CandidateId: string, bg2CandidateId: string): Promise<DestinationEvaluation | null> {
    return this.#command("choose_destination_folder", { bg1CandidateId, bg2CandidateId });
  }

  inspectDestination(path: string, bg1CandidateId: string, bg2CandidateId: string): Promise<DestinationEvaluation> {
    return this.#command("inspect_destination", { path, bg1CandidateId, bg2CandidateId });
  }

  async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    const evaluation = await this.#command<SelectionEvaluationWire>("evaluate_build", { selection });
    return projectEvaluation(evaluation);
  }

  async inspectManualDownloads(selection: NormalizedSelection): Promise<ManualDownloadRequirement[]> {
    const requirements = await this.#command<readonly ManualDownloadRequirementWire[]>("inspect_manual_downloads", { selection });
    return requirements.map(requirement => ({
      artifactId: requirement.artifact_id,
      modIds: requirement.mod_ids,
      title: requirement.title,
      filename: requirement.filename,
      length: requirement.length,
      ready: requirement.ready,
      detail: requirement.detail,
    }));
  }

  async freezeReview(
    displayName: string,
    selection: NormalizedSelection,
    destination: string,
    bg1CandidateId: string,
    bg2CandidateId: string,
  ): Promise<FrozenReview> {
    const review = await this.#command<FrozenReviewWire>("freeze_review", {
      displayName,
      selection,
      destination,
      bg1CandidateId,
      bg2CandidateId,
    });
    return {
      reviewToken: review.review_token,
      digest: review.digest,
      displayName: review.display_name,
      destination: review.destination,
      gameLabels: review.game_labels,
      evaluation: projectEvaluation(review.evaluation),
    };
  }

  async startBuild(reviewToken: string, onEvent: (event: RunEventEnvelope) => void): Promise<StartBuildResponse> {
    const onEventChannel = this.#eventChannel((event) => onEvent(projectRunEvent(event as RunEventEnvelopeWire)));
    const started = await this.#command<StartBuildWire>("start_build", { reviewToken, onEvent: onEventChannel });
    return { runId: started.run_id };
  }

  async resumeBuild(installId: string, onEvent: (event: RunEventEnvelope) => void): Promise<StartBuildResponse> {
    const onEventChannel = this.#eventChannel((event) => onEvent(projectRunEvent(event as RunEventEnvelopeWire)));
    const started = await this.#command<StartBuildWire>("resume_build", { installId, onEvent: onEventChannel });
    return { runId: started.run_id };
  }

  async supplyManualArchive(artifactId: string): Promise<ManualArchiveSupply | null> {
    const supplied = await this.#command<ManualArchiveSupplyWire | null>("supply_manual_archive", { artifactId });
    return supplied === null ? null : {
      artifactId: supplied.artifact_id,
      filename: supplied.filename,
      sha256: supplied.sha256,
      length: supplied.length,
    };
  }

  async openManualSource(artifactId: string): Promise<void> {
    await this.#command("open_manual_source", { artifactId });
  }

  async getRunSnapshot(runId: string): Promise<RunSnapshot> {
    const snapshot = await this.#command<RunSnapshotWire>("get_run_snapshot", { runId });
    return {
      runId: snapshot.run_id,
      status: snapshot.status,
      events: snapshot.events.map(projectRunEvent),
      report: snapshot.report,
      error: snapshot.error,
    };
  }

  async continueWaiting(runId: string): Promise<void> {
    await this.#command("continue_waiting", { runId });
  }

  async pauseRun(runId: string): Promise<void> {
    await this.#command("pause_run", { runId });
  }

  async cancelRun(runId: string): Promise<void> {
    await this.#command("cancel_run", { runId });
  }

  getBuildSnapshot(): Promise<BuildSnapshot> {
    return this.#unavailable("Build status");
  }

  advanceBuild(): Promise<BuildSnapshot> {
    return this.#unavailable("Build execution");
  }

  retryBuild(): Promise<BuildSnapshot> {
    return this.#unavailable("Build retry");
  }

  exportDiagnostics(installId: string): Promise<{ readonly path: string } | null> {
    return this.#command("export_diagnostics", { installId });
  }

  async listManagedInstallations(): Promise<ManagedInstallation[]> {
    const installations = await this.#command<ManagedInstallationWire[]>("list_managed_installations");
    return installations.map(({
      receipt_path: receiptPath,
      launch_path: launchPath,
      completed_at_millis: completedAtMillis,
      recipe_version: recipeVersion,
      radar_version: radarVersion,
      consistency,
      ...installation
    }) => ({
      ...installation,
      receiptPath,
      launchPath,
      completedAtMillis,
      recipeVersion,
      radarVersion,
      consistency: consistency == null ? undefined : {
        state: consistency.state,
        detail: consistency.detail,
        componentCount: consistency.component_count,
        modCount: consistency.mod_count,
      },
    }));
  }

  async launchInstall(installId: string): Promise<void> {
    await this.#command("launch_install", { installId });
  }

  async openInstallFolder(installId: string): Promise<void> {
    await this.#command("open_install_folder", { installId });
  }

  createDesktopShortcut(installId: string): Promise<{ readonly path: string }> {
    return this.#command("create_desktop_shortcut", { installId });
  }

  async installRadar(installId: string): Promise<void> {
    await this.#command("install_radar", { installId });
  }

  async getUpdates(): Promise<UpdateSummary> {
    const update = await this.#command<UpdateSummaryWire>("check_updates");
    return {
      checkedAt: update.checked_at,
      networkState: update.network_state,
      application: {
        state: update.application.state,
        currentVersion: update.application.current_version,
        availableVersion: update.application.available_version,
        detail: update.application.detail,
        releaseNotes: update.application.release_notes ?? undefined,
      },
      recipe: {
        state: update.recipe.state,
        currentVersion: update.recipe.current_version,
        availableVersion: update.recipe.available_version,
        disposition: update.recipe.disposition,
        detail: update.recipe.detail,
        changes: update.recipe.changes.map((change) => ({
          title: change.title,
          summary: change.summary,
          saveApplicability: change.save_applicability,
          urgency: change.urgency,
          conditionNote: change.condition_note,
        })),
      },
      managedCopies: update.managed_copies.map((copy) => ({
        installId: copy.install_id,
        name: copy.name,
        path: copy.path,
        installedRecipeVersion: copy.installed_recipe_version,
        state: copy.state,
        detail: copy.detail,
      })),
      radar: update.radar == null ? undefined : {
        state: update.radar.state,
        currentVersion: update.radar.current_version,
        availableVersion: update.radar.available_version,
        detail: update.radar.detail,
        releaseNotes: update.radar.release_notes ?? undefined,
      },
    };
  }

  async installAppUpdate(version: string): Promise<void> {
    await this.#command("install_app_update", { version });
  }

  async activateRecipeUpdate(version: string): Promise<void> {
    await this.#command("activate_recipe_update", { version });
  }

  async #command<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await this.#invoke(command, args) as T;
    } catch (error: unknown) {
      throw normalizeCommandError(error);
    }
  }

  #unavailable<T>(operation: string): Promise<T> {
    return Promise.reject(new BackendCommandError({
      code: "command_not_available",
      message: `${operation} is not connected to the native engine yet.`,
      recovery_action: "Use only the connected read-only setup checks in this build.",
      technical_detail: "Deferred Task 23 command surface.",
    }));
  }
}

const phases: readonly PhaseSummary[] = [
  { id: "preparation", title: "Prepare clean game copies", detail: "Verify sources and create a separate workspace." },
  { id: "bg1", title: "Prepare Baldur's Gate", detail: "Apply the pinned pre-merge setup." },
  { id: "merge", title: "Merge with EET", detail: "Create the continuous game world." },
  { id: "main", title: "Install the collection", detail: "Install curated content and rules in order." },
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
    return Promise.resolve({ mode: "fixture", applicationVersion: "0.1.0-alpha.1", engineVersion: "0.1.0", recipeVersion: "2026.09-fixture", startupInstallId: null });
  }

  discoverGames(): Promise<GameDiscovery> {
    return Promise.resolve({
      selectedBg1Id: "bg1-fresh",
      selectedBg2Id: "bg2-fresh",
      bg1Candidates: [
        { id: "bg1-fresh", label: "BG:EE + SoD — clean", path: this.#options.textOverrides?.gamePath ?? "C:\\Fixture\\BGEE", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: ["Clean supported installation."] },
        { id: "bg1-modified", label: "BG:EE — modified", path: "C:\\Fixture\\BGEE Modified", storefront: "steam", build: "2.7.3.0", freshness: "modified", eligible: false, findings: ["Files differ from a clean store installation."] },
        { id: "bg1-old", label: "BG:EE — old version", path: "C:\\Fixture\\BGEE Old", storefront: "steam", build: "2.6.6.0", freshness: "unsupported-version", eligible: false, findings: ["The installed game version is not supported."] },
        { id: "bg1-nosod", label: "BG:EE — SoD missing", path: "C:\\Fixture\\BGEE No SoD", storefront: "steam", build: "2.7.3.0", freshness: "missing-sod", eligible: false, findings: ["Siege of Dragonspear is required for this EET recipe."] },
      ],
      bg2Candidates: [
        { id: "bg2-fresh", label: "BGII:EE — clean", path: "C:\\Fixture\\BG2EE", storefront: "steam", build: "2.7.3.0", freshness: "fresh", eligible: true, findings: ["Clean supported installation."] },
        { id: "bg2-store", label: "BGII:EE — verify storefront", path: "C:\\Fixture\\BG2EE Other", storefront: "gog", build: "2.7.3.0", freshness: "unverified-storefront", eligible: false, findings: ["This storefront layout has not been verified yet."] },
      ],
    });
  }

  selectProfile(_profileId: string): Promise<BackendStatus> {
    return this.getStatus();
  }

  getInstallationDefaults(): Promise<InstallationDefaults> {
    return Promise.resolve({
      name: "Chriz Easy BG",
      path: "C:\\Users\\Chris\\Games\\Chriz Easy BG",
    });
  }

  async chooseGameFolder(role: GameRole): Promise<GameCandidate | null> {
    const path = role === "bgee_sod" ? "C:\\Fixture\\Browsed BGEE" : "C:\\Fixture\\Browsed BG2EE";
    return this.inspectGamePath(role, path);
  }

  async inspectGamePath(role: GameRole, path: string): Promise<GameCandidate> {
    const discovery = await this.discoverGames();
    const candidates = role === "bgee_sod" ? discovery.bg1Candidates : discovery.bg2Candidates;
    return candidates.find((candidate) => candidate.path === path) ?? {
      id: `fixture-browsed-${role}`,
      label: role === "bgee_sod" ? "BG:EE + SoD — browsed fixture" : "BGII:EE — browsed fixture",
      path,
      storefront: "steam",
      build: null,
      freshness: "modified",
      eligible: false,
      findings: ["Fixture browsing never accepts an unmodeled source as clean."],
    };
  }

  chooseDestinationFolder(bg1CandidateId: string, bg2CandidateId: string): Promise<DestinationEvaluation | null> {
    return this.inspectDestination("D:\\Fixture Campaigns\\Browsed Chriz EET Alpha", bg1CandidateId, bg2CandidateId);
  }

  inspectDestination(path: string, _bg1CandidateId: string, _bg2CandidateId: string): Promise<DestinationEvaluation> {
    const normalized = path.trim();
    const safe = normalized.length > 3 && !normalized.toLowerCase().includes("steamapps");
    return Promise.resolve({
      path: normalized,
      safe,
      title: safe ? "Ready to install" : "Choose a separate install location",
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

  async freezeReview(
    displayName: string,
    selection: NormalizedSelection,
    destination: string,
    bg1CandidateId: string,
    bg2CandidateId: string,
  ): Promise<FrozenReview> {
    const evaluation = await this.evaluateBuild(selection);
    const discovery = await this.discoverGames();
    const gameLabels = [
      discovery.bg1Candidates.find((candidate) => candidate.id === bg1CandidateId)?.label ?? "Not selected",
      discovery.bg2Candidates.find((candidate) => candidate.id === bg2CandidateId)?.label ?? "Not selected",
    ];
    return { reviewToken: "fixture-review-token", digest: "fixture-review-8d6d75", displayName, destination, gameLabels, evaluation };
  }

  inspectManualDownloads(_selection: NormalizedSelection): Promise<ManualDownloadRequirement[]> {
    return Promise.resolve([]);
  }

  startBuild(_reviewToken: string, _onEvent: (event: RunEventEnvelope) => void): Promise<StartBuildResponse> {
    return Promise.resolve({ runId: "fixture-run" });
  }

  resumeBuild(_installId: string, _onEvent: (event: RunEventEnvelope) => void): Promise<StartBuildResponse> {
    this.#buildIndex = 3;
    return Promise.resolve({ runId: "fixture-run-resumed" });
  }

  supplyManualArchive(artifactId: string): Promise<ManualArchiveSupply | null> {
    return Promise.resolve({
      artifactId,
      filename: `${artifactId}.zip`,
      sha256: "11".repeat(32),
      length: 1_234,
    });
  }

  openManualSource(_artifactId: string): Promise<void> {
    return Promise.resolve();
  }

  getRunSnapshot(runId: string): Promise<RunSnapshot> {
    return Promise.resolve({ runId, status: "running", events: [], report: null, error: null });
  }

  continueWaiting(_runId: string): Promise<void> {
    return Promise.resolve();
  }

  pauseRun(_runId: string): Promise<void> {
    return Promise.resolve();
  }

  cancelRun(_runId: string): Promise<void> {
    return Promise.resolve();
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

  exportDiagnostics(_installId: string): Promise<{ readonly path: string } | null> {
    return Promise.resolve({ path: "C:\\Fixture\\diagnostics\\build-report.zip" });
  }

  listManagedInstallations(): Promise<ManagedInstallation[]> {
    if (this.#options.managedInstallations !== undefined) {
      return Promise.resolve([...this.#options.managedInstallations]);
    }
    if (this.#buildIndex < 4) return Promise.resolve([]);
    return Promise.resolve([{
      id: "fixture-install",
      name: this.#options.textOverrides?.campaignName ?? "Chriz Easy BG",
      path: "D:\\Fixture Installations\\Chriz Easy BG",
      status: "Ready to play",
      receiptPath: "D:\\Fixture Installations\\Chriz Easy BG\\install-receipt.json",
      launchPath: "D:\\Fixture Installations\\Chriz Easy BG\\InfinityLoader.exe",
      completedAtMillis: 1_788_451_200_000,
      available: true,
      resumable: false,
    }]);
  }

  launchInstall(_installId: string): Promise<void> {
    return Promise.resolve();
  }

  openInstallFolder(_installId: string): Promise<void> {
    return Promise.resolve();
  }

  createDesktopShortcut(_installId: string): Promise<{ readonly path: string }> {
    return Promise.resolve({ path: "C:\\Users\\Chris\\Desktop\\Chriz Easy BG.lnk" });
  }

  installRadar(_installId: string): Promise<void> {
    return Promise.resolve();
  }

  getUpdates(): Promise<UpdateSummary> {
    return Promise.resolve({
      checkedAt: "2026-09-04T12:00:00Z",
      networkState: "online",
      application: { state: "up-to-date", currentVersion: "0.1.0-alpha.1", availableVersion: null, detail: "The fixture application is current." },
      recipe: { state: "up-to-date", currentVersion: "0.1.0-alpha.1", availableVersion: null, disposition: "up-to-date", detail: "The fixture recipe is current.", changes: [] },
      managedCopies: [{ installId: "fixture-install", name: "Chriz Easy BG", path: "D:\\Fixture Installations\\Chriz Easy BG", installedRecipeVersion: "0.1.0-alpha.1", state: "up-to-date", detail: "This installation uses the current recipe." }],
    });
  }

  installAppUpdate(_version: string): Promise<void> {
    return Promise.resolve();
  }

  activateRecipeUpdate(_version: string): Promise<void> {
    return Promise.resolve();
  }

  #snapshot(): BuildSnapshot {
    const states = ["waiting-manual", "attention", "failed", "running", "complete"] as const;
    const headlines = { "waiting-manual": "Manual archive needed", attention: "Your attention is needed", failed: "A fixture step failed", running: "Build in progress", complete: "Build verified" } as const;
    const details = {
      "waiting-manual": "Add the named archive to the staging folder, then continue.",
      attention: "Review the fixture installer prompt before continuing.",
      failed: "Nothing was changed outside the fixture. Retry or export diagnostics.",
      running: "The fixture ledger is advancing through the reviewed plan.",
      complete: "The installation and its record are ready.",
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

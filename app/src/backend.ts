import { Channel, invoke } from "@tauri-apps/api/core";
import { decodeOperationProgress } from "./operation-feedback";
import { flavor } from "./flavor";

import type {
  BackendStatus,
  BuildSnapshot,
  CommandErrorPayload,
  DestinationEvaluation,
  FeatureControl,
  FeatureGroup,
  FeatureGroupVariant,
  FixtureOptions,
  FrozenReview,
  Freshness,
  GameCandidate,
  GameDiscovery,
  GameLogFile,
  GameLogStatus,
  GameRole,
  HotpatchPreview,
  InstallationDefaults,
  InstallationRemovalPreview,
  LaunchInstallResult,
  InstalledComponent,
  InstalledUpdatePlan,
  ManualArchiveSupply,
  ManualDownloadRequirement,
  ManualBreakpoint,
  ManualCheckpointOption,
  ManualCheckpointStatus,
  ManagedInstallation,
  ModUpgradePreview,
  OperationProgress,
  NormalizedSelection,
  PatchPreview,
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
    manualBreakpoints?: readonly ManualBreakpoint[],
  ): Promise<FrozenReview>;
  getManualCheckpoint?(installId: string): Promise<ManualCheckpointStatus | null>;
  checkManualCheckpoint?(installId: string): Promise<ManualCheckpointStatus>;
  acceptManualCheckpoint?(installId: string, reviewToken: string, acceptAdditions: boolean): Promise<void>;
  openManualCheckpointFolder?(installId: string): Promise<void>;
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
  renameInstallation?(installId: string, name: string): Promise<{ readonly name: string }>;
  previewInstallationRemoval(installId: string): Promise<InstallationRemovalPreview>;
  removeInstallation(installId: string, confirmationToken: string): Promise<void>;
  launchInstall(installId: string, startRadar: boolean): Promise<LaunchInstallResult>;
  /** Reads only: whether the game writes its log for crash reports, and where the evidence is. */
  gameLogStatus?(installId: string): Promise<GameLogStatus>;
  /** Turns the game log on or off for the next game start. */
  setGameLog?(installId: string, enabled: boolean): Promise<GameLogStatus>;
  openInstallFolder(installId: string): Promise<void>;
  createDesktopShortcut(installId: string): Promise<{ readonly path: string }>;
  installRadar(installId: string): Promise<void>;
  getUpdates(): Promise<UpdateSummary>;
  installAppUpdate(version: string, onProgress?: (progress: OperationProgress) => void): Promise<void>;
  activateRecipeUpdate(version: string): Promise<void>;
  inspectInstalledUpdate?(installId: string): Promise<InstalledUpdatePlan>;
  inspectModUpgrade?(installId: string): Promise<ModUpgradePreview>;
  applyModUpgrade?(installId: string, reviewToken: string, fullBackup: boolean, backupSaves: boolean, onProgress?: (progress: OperationProgress) => void): Promise<ModUpgradePreview>;
  restoreModUpgrade?(installId: string, undo: boolean, onProgress?: (progress: OperationProgress) => void): Promise<ModUpgradePreview>;
  inspectInstallPatches?(installId: string): Promise<PatchPreview>;
  applyInstallPatch?(installId: string, reviewToken: string, fullBackup: boolean, saveBackup: boolean, onProgress?: (progress: OperationProgress) => void): Promise<PatchPreview>;
  undoInstallPatch?(installId: string, onProgress?: (progress: OperationProgress) => void): Promise<PatchPreview>;
  restoreInstallPatch?(installId: string, onProgress?: (progress: OperationProgress) => void): Promise<PatchPreview>;
  inspectHotpatch?(installId: string): Promise<HotpatchPreview>;
  applyHotpatch?(installId: string, reviewToken: string, fullBackup: boolean, saveBackup: boolean, onProgress?: (progress: OperationProgress) => void): Promise<HotpatchPreview>;
  /** Undo the latest hotpatch (`undo`), or finish restoring one that did not finish. */
  restoreHotpatch?(installId: string, undo: boolean, onProgress?: (progress: OperationProgress) => void): Promise<HotpatchPreview>;
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
  readonly collection?: {
    readonly flavor: string;
    readonly game: string;
    readonly product_name: string;
    readonly sources: readonly { readonly slot: string; readonly role: string; readonly label: string }[];
  } | null;
};

function projectStatus(status: BootstrapWire): BackendStatus {
  return {
    mode: status.mode,
    applicationVersion: status.application_version,
    engineVersion: status.engine_version,
    recipeVersion: status.recipe_version,
    startupInstallId: status.startup_install_id,
    profiles: status.profiles,
    selectedProfile: status.selected_profile,
    ...(status.collection == null ? {} : { collection: {
      flavor: status.collection.flavor,
      game: status.collection.game,
      productName: status.collection.product_name,
      sources: status.collection.sources,
    } }),
  };
}

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

type FeatureControlWire = Omit<FeatureControl, "unavailableReason" | "sourceLabel" | "groupLabel" | "choiceGroup" | "choiceAvailable" | "requiresAny"> & {
  readonly unavailable_reason: string | null;
  readonly source_label?: string | null;
  readonly group_label?: string | null;
  readonly choice_group?: string | null;
  readonly choice_available?: boolean | null;
  readonly requires_any?: readonly string[];
};

type FeatureGroupWire = {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly members: readonly string[];
  readonly variants?: readonly {
    readonly choice_group: string;
    readonly option: string;
    readonly label: string;
    readonly options: readonly string[];
    readonly recipe_default: readonly string[];
  }[];
};

type FeatureGroupStateWire = {
  readonly id: string;
  readonly state: FeatureGroup["state"];
  readonly selected_members: readonly string[];
  readonly available_members: readonly string[];
  readonly unavailable_members: readonly { readonly feature_id: string; readonly reason: string }[];
  readonly variants?: readonly {
    readonly choice_group: string;
    readonly option: string;
    readonly selected: boolean;
    readonly available: boolean;
    readonly unavailable_reason?: string | null;
  }[];
  readonly disabled_reason?: string | null;
};

type SelectionEvaluationWire = {
  readonly manual_checkpoint_options?: readonly ManualCheckpointOption[];
  readonly groups?: readonly FeatureGroupStateWire[];
  readonly view: {
    readonly categories: readonly string[];
    readonly controls: readonly FeatureControlWire[];
    readonly groups?: readonly FeatureGroupWire[];
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
  readonly customized?: boolean;
  readonly id: string;
  readonly name: string;
  readonly path: string;
  readonly status: string;
  readonly receipt_path: string | null;
  readonly launch_path: string | null;
  readonly completed_at_millis: number | null;
  readonly available: boolean;
  readonly resumable: boolean;
  readonly patch_recovery_needed?: boolean;
  readonly recipe_version?: string | null;
  readonly radar_version?: string | null;
  readonly consistency?: {
    readonly state: "matches" | "changed" | "unavailable";
    readonly detail: string;
    readonly component_count: number;
    readonly mod_count: number;
    readonly components?: readonly InstalledComponent[];
  } | null;
};

type UpdateSummaryWire = {
  readonly lookup_trace_enabled?: boolean;
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

function projectFeatureGroups(evaluation: SelectionEvaluationWire): FeatureGroup[] {
  const states = new Map((evaluation.groups ?? []).map(state => [state.id, state]));
  return (evaluation.view.groups ?? []).flatMap((group) => {
    const state = states.get(group.id);
    if (!state) return [];
    // A group whose variant states don't match its authored variants is left out like a group
    // without state, rather than shown half-switched.
    const authoredVariants = group.variants ?? [];
    const stateVariants = state.variants ?? [];
    if (stateVariants.length !== authoredVariants.length) return [];
    const variantStates = new Map(stateVariants.map(variant => [variant.choice_group, variant]));
    const variants: FeatureGroupVariant[] = [];
    for (const variant of authoredVariants) {
      const variantState = variantStates.get(variant.choice_group);
      if (!variantState || variantState.option !== variant.option) return [];
      variants.push({
        choiceGroup: variant.choice_group,
        option: variant.option,
        label: variant.label,
        options: variant.options,
        recipeDefault: variant.recipe_default,
        selected: variantState.selected,
        available: variantState.available,
        unavailableReason: variantState.unavailable_reason ?? null,
      });
    }
    return [{
      id: group.id,
      title: group.title,
      description: group.description,
      members: group.members,
      state: state.state,
      selectedMembers: state.selected_members,
      availableMembers: state.available_members,
      unavailableMembers: state.unavailable_members.map(member => ({ featureId: member.feature_id, reason: member.reason })),
      disabledReason: state.disabled_reason ?? null,
      ...(variants.length === 0 ? {} : { variants }),
    }];
  });
}

function projectEvaluation(evaluation: SelectionEvaluationWire): SelectionEvaluation {
  const featureGroups = projectFeatureGroups(evaluation);
  return {
    ...(evaluation.manual_checkpoint_options === undefined ? {} : { manualCheckpointOptions: evaluation.manual_checkpoint_options }),
    ...(featureGroups.length === 0 ? {} : { featureGroups }),
    view: {
      categories: evaluation.view.categories,
      controls: evaluation.view.controls.map((control) => {
        const { unavailable_reason: unavailableReason, source_label: sourceLabel, group_label: groupLabel,
          choice_group: choiceGroup, choice_available: choiceAvailable, requires_any: requiresAny, ...rest } = control;
        return { ...rest, unavailableReason, sourceLabel, groupLabel, choiceGroup, choiceAvailable, requiresAny };
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
    return projectStatus(await this.#command<BootstrapWire>("bootstrap"));
  }

  async selectProfile(profileId: string): Promise<BackendStatus> {
    return projectStatus(await this.#command<BootstrapWire>("select_profile", { profileId }));
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
    manualBreakpoints?: readonly ManualBreakpoint[],
  ): Promise<FrozenReview> {
    const review = await this.#command<FrozenReviewWire>("freeze_review", {
      displayName,
      selection,
      destination,
      bg1CandidateId,
      bg2CandidateId,
      ...(manualBreakpoints?.length ? { manualBreakpoints } : {}),
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

  getManualCheckpoint(installId: string): Promise<ManualCheckpointStatus | null> {
    return this.#command("get_manual_checkpoint", { installId });
  }

  checkManualCheckpoint(installId: string): Promise<ManualCheckpointStatus> {
    return this.#command("check_manual_checkpoint", { installId });
  }

  acceptManualCheckpoint(installId: string, reviewToken: string, acceptAdditions: boolean): Promise<void> {
    return this.#command("accept_manual_checkpoint", { installId, reviewToken, acceptAdditions });
  }

  openManualCheckpointFolder(installId: string): Promise<void> {
    return this.#command("open_manual_checkpoint_folder", { installId });
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
      patch_recovery_needed: patchRecoveryNeeded,
      consistency,
      ...installation
    }) => ({
      ...installation,
      receiptPath,
      launchPath,
      completedAtMillis,
      recipeVersion,
      radarVersion,
      patchRecoveryNeeded,
      consistency: consistency == null ? undefined : {
        state: consistency.state,
        detail: consistency.detail,
        componentCount: consistency.component_count,
        modCount: consistency.mod_count,
        components: consistency.components,
      },
    }));
  }

  launchInstall(installId: string, startRadar: boolean): Promise<LaunchInstallResult> {
    return this.#command("launch_install", { installId, startRadar });
  }

  gameLogStatus(installId: string): Promise<GameLogStatus> {
    return this.#command("game_log_status", { installId });
  }

  setGameLog(installId: string, enabled: boolean): Promise<GameLogStatus> {
    return this.#command("set_game_log", { installId, enabled });
  }

  previewInstallationRemoval(installId: string): Promise<InstallationRemovalPreview> {
    return this.#command("preview_installation_removal", { installId });
  }

  async removeInstallation(installId: string, confirmationToken: string): Promise<void> {
    await this.#command("remove_installation", { installId, confirmationToken });
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
    if (update.lookup_trace_enabled) {
      // Local diagnostic builds only. A logging failure must not block Updates.
      void this.#command("record_install_lookup_client", { report: {
        checked_at: update.checked_at,
        copies: update.managed_copies.map(copy => ({
          install_id: copy.install_id, name: copy.name, path: copy.path, state: copy.state,
        })),
      } }).catch(() => undefined);
    }
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

  async installAppUpdate(version: string, onProgress?: (progress: OperationProgress) => void): Promise<void> {
    await this.#commandWithProgress("install_app_update", { version }, onProgress);
  }

  renameInstallation(installId: string, name: string): Promise<{ readonly name: string }> {
    return this.#command("rename_installation", { installId, name });
  }

  async activateRecipeUpdate(version: string): Promise<void> {
    await this.#command("activate_recipe_update", { version });
  }

  inspectInstallPatches(installId: string): Promise<PatchPreview> {
    return this.#command("inspect_install_patches", { installId });
  }

  inspectInstalledUpdate(installId: string): Promise<InstalledUpdatePlan> {
    return this.#command("inspect_installed_update", { installId });
  }

  inspectModUpgrade(installId: string): Promise<ModUpgradePreview> {
    return this.#command("inspect_mod_upgrade", { installId });
  }

  applyModUpgrade(installId: string, reviewToken: string, fullBackup: boolean, backupSaves: boolean, onProgress?: (progress: OperationProgress) => void): Promise<ModUpgradePreview> {
    return this.#commandWithProgress("apply_mod_upgrade", { installId, reviewToken, fullBackup, backupSaves }, onProgress);
  }

  restoreModUpgrade(installId: string, undo: boolean, onProgress?: (progress: OperationProgress) => void): Promise<ModUpgradePreview> {
    return this.#commandWithProgress("restore_mod_upgrade", { installId, undo }, onProgress);
  }

  applyInstallPatch(installId: string, reviewToken: string, fullBackup: boolean, saveBackup: boolean, onProgress?: (progress: OperationProgress) => void): Promise<PatchPreview> {
    return this.#commandWithProgress("apply_install_patch", { installId, reviewToken, fullBackup, saveBackup }, onProgress);
  }

  undoInstallPatch(installId: string, onProgress?: (progress: OperationProgress) => void): Promise<PatchPreview> {
    return this.#commandWithProgress("undo_install_patch", { installId }, onProgress);
  }

  restoreInstallPatch(installId: string, onProgress?: (progress: OperationProgress) => void): Promise<PatchPreview> {
    return this.#commandWithProgress("restore_install_patch", { installId }, onProgress);
  }

  inspectHotpatch(installId: string): Promise<HotpatchPreview> {
    return this.#command("inspect_hotpatch", { installId });
  }

  applyHotpatch(installId: string, reviewToken: string, fullBackup: boolean, saveBackup: boolean, onProgress?: (progress: OperationProgress) => void): Promise<HotpatchPreview> {
    return this.#commandWithProgress("apply_hotpatch", { installId, reviewToken, fullBackup, saveBackup }, onProgress);
  }

  restoreHotpatch(installId: string, undo: boolean, onProgress?: (progress: OperationProgress) => void): Promise<HotpatchPreview> {
    return this.#commandWithProgress("restore_hotpatch", { installId, undo }, onProgress);
  }

  async #commandWithProgress<T>(command: string, args: Record<string, unknown>, onProgress?: (progress: OperationProgress) => void): Promise<T> {
    if (!onProgress) return this.#command(command, args);
    let active = true;
    const channel = this.#eventChannel((event) => {
      const progress = decodeOperationProgress(event);
      if (active && progress) onProgress(progress);
    });
    try { return await this.#command(command, { ...args, onProgress: channel }); }
    finally { active = false; }
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

/** The Icewind Dale fixture: one clean game copy and a small recipe in the IWD collection's shape. */
const iwdPhases: readonly PhaseSummary[] = [
  { id: "preparation", title: "Prepare a clean game copy", detail: "Verify the source and create a separate workspace." },
  { id: "main", title: "Install the collection", detail: "Install curated content and rules in order." },
];

const iwdControl = (
  id: string, title: string, description: string, category: string,
  decision: FeatureControl["decision"], selected: boolean,
): FeatureControl => ({
  id, title, description, category, decision, readiness: "ready", parent: null, selected,
  interactive: decision !== "mandatory", unavailableReason: null, inputs: [],
});

const iwdControls: readonly FeatureControl[] = [
  iwdControl("mod:eefixpack", "Enhanced Edition Fixpack", "Fixes for the game itself, installed before every other mod.", "core", "mandatory", true),
  iwdControl("mod:eeex", "EEex", "Extends the game engine for the mods and tweaks that need it.", "core", "mandatory", true),
  iwdControl("mod:cdtweaks", "The Tweaks Anthology", "Quality-of-life and rules tweaks from the collection's selection.", "convenience", "default", true),
  iwdControl("mod:bubb-spell-menu", "Bubb's Spell Menu Extended", "A searchable menu for spells and special abilities.", "convenience", "default", true),
  iwdControl("mod:stratagems", "Sword Coast Stratagems", "Smarter enemy tactics and spellcasting.", "difficulty", "default", true),
  iwdControl("mod:minervanpc", "Minerva NPC", "An additional companion who can join your party.", "npcs", "optional", false),
  iwdControl("mod:karihinpc", "Karihi NPC", "An additional companion who can join your party.", "npcs", "optional", false),
];

const iwdCandidate: GameCandidate = {
  id: "iwd-fresh", label: "IWD:EE — clean", path: "C:\\Fixture\\IWDEE", storefront: "steam", build: "2.7.3.0",
  freshness: "fresh", eligible: true, findings: ["Clean supported installation."],
};

function wait(milliseconds: number): Promise<void> {
  return new Promise((resolve) => globalThis.setTimeout(resolve, milliseconds));
}

export class FixtureBackend implements Backend {
  readonly #patches = new Map<string, PatchPreview>();
  readonly #hotpatches = new Map<string, HotpatchPreview>();
  readonly #modUpgrades = new Map<string, ModUpgradePreview>();
  /** Installations whose game log is on. */
  readonly #gameLogs = new Set<string>();
  #patchReviewSequence = 0;
  readonly #removedInstallations = new Set<string>();
  readonly #removalTokens = new Map<string, string>();
  readonly #options: FixtureOptions;
  #evaluationCall = 0;
  #buildIndex = 0;

  constructor(options: FixtureOptions = {}) {
    this.#options = options;
  }

  getStatus(): Promise<BackendStatus> {
    return Promise.resolve({ mode: "fixture", applicationVersion: "0.1.0-alpha.1", engineVersion: "0.1.0", recipeVersion: "2026.09-fixture", startupInstallId: null,
      ...(flavor.id === "iwd" ? { collection: {
        flavor: "iwd", game: "iwdee", productName: flavor.productName,
        sources: flavor.sources.map(source => ({ slot: source.slot, role: source.role, label: source.label })),
      } } : {}) });
  }

  discoverGames(): Promise<GameDiscovery> {
    if (flavor.id === "iwd") {
      return Promise.resolve({
        selectedBg1Id: "",
        selectedBg2Id: iwdCandidate.id,
        bg1Candidates: [],
        bg2Candidates: [{ ...iwdCandidate, path: this.#options.textOverrides?.gamePath ?? iwdCandidate.path }],
      });
    }
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
      name: flavor.productName,
      path: `C:\\Users\\Player\\Games\\${flavor.productName}`,
    });
  }

  async chooseGameFolder(role: GameRole): Promise<GameCandidate | null> {
    const path = role === "bgee_sod" ? "C:\\Fixture\\Browsed BGEE" : role === "iwdee" ? "C:\\Fixture\\Browsed IWDEE" : "C:\\Fixture\\Browsed BG2EE";
    return this.inspectGamePath(role, path);
  }

  async inspectGamePath(role: GameRole, path: string): Promise<GameCandidate> {
    const discovery = await this.discoverGames();
    const candidates = role === "bgee_sod" ? discovery.bg1Candidates : discovery.bg2Candidates;
    return candidates.find((candidate) => candidate.path === path) ?? {
      id: `fixture-browsed-${role}`,
      label: role === "bgee_sod" ? "BG:EE + SoD — browsed fixture" : role === "iwdee" ? "IWD:EE — browsed fixture" : "BGII:EE — browsed fixture",
      path,
      storefront: "steam",
      build: null,
      freshness: "modified",
      eligible: false,
      findings: ["Fixture browsing never accepts an unmodeled source as clean."],
    };
  }

  chooseDestinationFolder(bg1CandidateId: string, bg2CandidateId: string): Promise<DestinationEvaluation | null> {
    const browsed = flavor.id === "iwd" ? `D:\\Fixture Campaigns\\Browsed ${flavor.productName}` : "D:\\Fixture Campaigns\\Browsed Chriz EET Alpha";
    return this.inspectDestination(browsed, bg1CandidateId, bg2CandidateId);
  }

  inspectDestination(path: string, _bg1CandidateId: string, _bg2CandidateId: string): Promise<DestinationEvaluation> {
    const normalized = path.trim();
    const safe = normalized.length > 3 && !normalized.toLowerCase().includes("steamapps");
    return Promise.resolve({
      path: normalized,
      safe,
      title: safe ? "Ready to install" : "Choose a separate install location",
      detail: safe
        ? flavor.id === "iwd" ? "The source game and its saves will remain untouched." : "The source games and their saves will remain untouched."
        : "The collection cannot be built inside a store-managed game folder.",
      requiredSpace: flavor.id === "iwd" ? "9 GB" : "62 GB",
      availableSpace: "184 GB",
    });
  }

  async evaluateBuild(selection: NormalizedSelection): Promise<SelectionEvaluation> {
    const call = this.#evaluationCall++;
    const delay = this.#options.evaluationDelays?.[call] ?? 0;
    if (delay > 0) await wait(delay);
    if (flavor.id === "iwd") {
      const controls = iwdControls.map((control) => ({ ...control, selected: control.decision === "mandatory" || (selection.features[control.id] ?? control.selected) }));
      return {
        view: { categories: ["core", "convenience", "difficulty", "npcs"], controls },
        normalizedSelection: { platform: selection.platform, features: Object.fromEntries(controls.map((control) => [control.id, control.selected])), inputs: selection.inputs },
        findings: [],
        plan: { phases: iwdPhases },
        selectedChoiceCount: controls.filter((control) => control.selected).length,
      };
    }
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
    _manualBreakpoints?: readonly ManualBreakpoint[],
  ): Promise<FrozenReview> {
    const evaluation = await this.evaluateBuild(selection);
    const discovery = await this.discoverGames();
    const gameLabels = flavor.sources.map(source => (source.slot === "bg1"
      ? discovery.bg1Candidates.find((candidate) => candidate.id === bg1CandidateId)
      : discovery.bg2Candidates.find((candidate) => candidate.id === bg2CandidateId))?.label ?? "Not selected");
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
      return Promise.resolve(this.#options.managedInstallations.filter((install) => !this.#removedInstallations.has(install.id)));
    }
    if (this.#buildIndex < 4 || this.#removedInstallations.has("fixture-install")) return Promise.resolve([]);
    return Promise.resolve([{
      id: "fixture-install",
      name: this.#options.textOverrides?.campaignName ?? flavor.productName,
      path: `D:\\Fixture Installations\\${flavor.productName}`,
      status: "Ready to play",
      receiptPath: `D:\\Fixture Installations\\${flavor.productName}\\install-receipt.json`,
      launchPath: `D:\\Fixture Installations\\${flavor.productName}\\InfinityLoader.exe`,
      completedAtMillis: 1_788_451_200_000,
      available: true,
      resumable: false,
    }]);
  }

  launchInstall(installId: string, startRadar: boolean): Promise<LaunchInstallResult> {
    return Promise.resolve({ installId, radar: {
      state: startRadar ? "started" : "not-requested", detail: null, technicalDetail: null,
    }, gameSettings: { state: "unchanged", technicalDetail: null },
    gameLog: { state: this.#gameLogs.has(installId) ? "kept" : "not-enabled", technicalDetail: null } });
  }

  async gameLogStatus(installId: string): Promise<GameLogStatus> {
    const installation = (await this.listManagedInstallations()).find((copy) => copy.id === installId && copy.available);
    if (!installation) throw new Error("Unknown fixture installation.");
    const game = `${installation.path}\\game`;
    const enabled = this.#gameLogs.has(installId);
    const file = (name: string, present: boolean): GameLogFile => ({
      path: `${game}\\${name}`, state: present ? "present" : "missing", sizeBytes: present ? "1048576" : null,
      modifiedAtMillis: present ? 1_788_451_200_000 : null, detail: null,
    });
    return { installId, setting: enabled ? "on" : "off", detail: null, log: file("InfinityLoader.log", enabled),
      previousLog: file("InfinityLoader-previous.log", false), crashFolder: `${game}\\InfinityLoader_Crash`, crashReports: 0 };
  }

  async setGameLog(installId: string, enabled: boolean): Promise<GameLogStatus> {
    await this.gameLogStatus(installId);
    if (enabled) this.#gameLogs.add(installId);
    else this.#gameLogs.delete(installId);
    return this.gameLogStatus(installId);
  }

  async previewInstallationRemoval(installId: string): Promise<InstallationRemovalPreview> {
    const install = (await this.listManagedInstallations()).find((entry) => entry.id === installId);
    if (!install) throw new Error("Unknown installation.");
    const token = `fixture-remove-${install.id}-${this.#removalTokens.size}`;
    this.#removalTokens.set(token, install.id);
    return { installId, displayName: install.name, managedRoot: install.path,
      action: install.status.toLowerCase().includes("folder moved") || install.status.toLowerCase().includes("not found") ? "forget" : "delete",
      preservedSavePath: `C:\\Users\\Player\\Documents\\${flavor.productName}`, confirmationToken: token };
  }

  async removeInstallation(installId: string, confirmationToken: string): Promise<void> {
    const expected = this.#removalTokens.get(confirmationToken);
    this.#removalTokens.delete(confirmationToken);
    if (expected !== installId) throw new Error("Removal confirmation does not match.");
    this.#removedInstallations.add(installId);
  }

  openInstallFolder(_installId: string): Promise<void> {
    return Promise.resolve();
  }

  createDesktopShortcut(_installId: string): Promise<{ readonly path: string }> {
    return Promise.resolve({ path: `C:\\Users\\Player\\Desktop\\${flavor.productName}.lnk` });
  }

  installRadar(_installId: string): Promise<void> {
    return Promise.resolve();
  }

  getUpdates(): Promise<UpdateSummary> {
    if (flavor.id === "iwd") return this.#unconfiguredUpdates();
    return Promise.resolve({
      checkedAt: "2026-09-04T12:00:00Z",
      networkState: "online",
      application: { state: "up-to-date", currentVersion: "0.1.0-alpha.1", availableVersion: null, detail: "The fixture application is current." },
      recipe: { state: "up-to-date", currentVersion: "0.1.0-alpha.1", availableVersion: null, disposition: "up-to-date", detail: "The fixture recipe is current.", changes: [] },
      managedCopies: [{ installId: "fixture-install", name: "Chriz Easy BG", path: "D:\\Fixture Installations\\Chriz Easy BG", installedRecipeVersion: "0.1.0-alpha.1", state: "up-to-date", detail: "This installation uses the current recipe." }],
    });
  }

  /** The standalone collection has no update channel; mirrors the native unconfigured update center. */
  async #unconfiguredUpdates(): Promise<UpdateSummary> {
    const copies = await this.listManagedInstallations();
    return {
      checkedAt: null,
      networkState: "unconfigured",
      application: { state: "unavailable", currentVersion: "0.1.0-alpha.1", availableVersion: null, detail: "The application update key and endpoint must be supplied at release-time." },
      recipe: { state: "unavailable", currentVersion: "bundled", availableVersion: null, disposition: "unknown-applicability", detail: "No signed channel is configured; the bundled trusted recipe was kept.", changes: [] },
      managedCopies: copies.map(copy => ({
        installId: copy.id, name: copy.name, path: copy.path, installedRecipeVersion: copy.recipeVersion ?? null,
        state: !copy.available && !copy.resumable ? "stale" : "unknown",
        detail: !copy.available && !copy.resumable
          ? "The registered folder moved or changed; no update action is available."
          : "No signed recipe channel is configured, so update applicability is unknown.",
      })),
    };
  }

  installAppUpdate(_version: string): Promise<void> {
    return Promise.resolve();
  }

  activateRecipeUpdate(_version: string): Promise<void> {
    return Promise.resolve();
  }

  async inspectInstallPatches(installId: string): Promise<PatchPreview> {
    const copies = (await this.getUpdates()).managedCopies;
    const copy = copies.find((entry) => entry.installId === installId);
    if (!copy) throw new Error("Unknown fixture installation.");
    const current = this.#patches.get(installId);
    if (current) return current;
    const patch: PatchPreview = {
      installId, patchId: "artisan-campaign-description-links-1.0", title: "Artisan kit-description links",
      state: "available", detail: "Preview only: repairs character-creation description links without changing abilities or text.",
      baseRecipeVersion: copy.installedRecipeVersion ?? "Unknown", appliedPatchIds: [],
      reviewToken: `fixture-patch-${installId}-${++this.#patchReviewSequence}`, canUndo: false, canRestore: false,
      fullBackupBytes: "12884901888", saveBackupBytes: "104857600", availableBytes: "107374182400",
      backupPath: "C:\\Fixture\\CEBG-Backups",
    };
    this.#patches.set(installId, patch);
    return patch;
  }

  async inspectInstalledUpdate(installId: string): Promise<InstalledUpdatePlan> {
    const updates = await this.getUpdates();
    const copy = updates.managedCopies.find((entry) => entry.installId === installId);
    if (!copy) throw new Error("Unknown fixture installation.");
    return {
      installId, baseRecipeVersion: copy.installedRecipeVersion ?? "Unknown",
      targetRecipeVersion: updates.recipe.currentVersion, state: "comparison-ready",
      detail: "Preview fixture only. No real installation was inspected.",
      choicesPreserved: true, issues: [], runs: [],
    };
  }

  async applyInstallPatch(installId: string, reviewToken: string, _fullBackup: boolean, _saveBackup: boolean): Promise<PatchPreview> {
    const patch = this.#patches.get(installId);
    if (!patch || patch.state !== "available" || patch.reviewToken !== reviewToken) throw new Error("Check this fixture installation again before applying.");
    const applied: PatchPreview = { ...patch, state: "applied", detail: "Preview only: the description-link fix was applied.", reviewToken: null, canUndo: true, appliedPatchIds: [patch.patchId] };
    this.#patches.set(installId, applied);
    return applied;
  }

  async inspectModUpgrade(installId: string): Promise<ModUpgradePreview> {
    const current = this.#modUpgrades.get(installId);
    if (current) return current;
    const copy = (await this.getUpdates()).managedCopies.find(entry => entry.installId === installId);
    if (!copy) throw new Error("Unknown fixture installation.");
    const preview: ModUpgradePreview = {
      installId, upgradeId: "buffbot-1.8.3-to-1.8.4", title: "BuffBot",
      state: "available", detail: "Preview fixture only. No real installation was inspected.",
      baseRecipeVersion: copy.installedRecipeVersion ?? "Unknown", installedVersion: "1.8.3-alpha", targetVersion: "1.8.4-alpha",
      reviewToken: `fixture-mod-upgrade-${installId}`, canUndo: false, canRestore: false,
      fullBackupBytes: "12884901888", saveBackupBytes: "104857600", availableBytes: "107374182400",
      backupPath: "C:\\Fixture\\CEBG-Backups", saveCompatibility: "Preview only. No saved games are changed.", history: [],
    };
    this.#modUpgrades.set(installId, preview);
    return preview;
  }

  async applyModUpgrade(installId: string, reviewToken: string, _fullBackup: boolean, _backupSaves: boolean): Promise<ModUpgradePreview> {
    const preview = this.#modUpgrades.get(installId);
    if (!preview || preview.state !== "available" || preview.reviewToken !== reviewToken) throw new Error("Check this fixture installation again before updating.");
    const applied: ModUpgradePreview = { ...preview, state: "applied", installedVersion: preview.targetVersion,
      detail: "Preview only: BuffBot was updated. No game files changed.", reviewToken: null, canUndo: true,
      history: [...preview.history, { transactionId: "fixture-upgrade", fromVersion: preview.installedVersion ?? "Unknown", toVersion: preview.targetVersion, state: "applied" }],
    };
    this.#modUpgrades.set(installId, applied);
    return applied;
  }

  async restoreModUpgrade(installId: string, undo: boolean): Promise<ModUpgradePreview> {
    const preview = this.#modUpgrades.get(installId);
    if (!preview || !(undo ? preview.canUndo : preview.canRestore)) throw new Error("No fixture update can be restored.");
    this.#modUpgrades.delete(installId);
    const restored = await this.inspectModUpgrade(installId);
    const result = { ...restored, history: preview.history.map(entry => ({ ...entry, state: "restored" })) };
    this.#modUpgrades.set(installId, result);
    return result;
  }

  async undoInstallPatch(installId: string): Promise<PatchPreview> {
    const patch = this.#patches.get(installId);
    if (!patch || patch.state !== "applied" || !patch.canUndo) throw new Error("No fixture fix can be undone.");
    this.#patches.delete(installId);
    return this.inspectInstallPatches(installId);
  }

  async restoreInstallPatch(_installId: string): Promise<PatchPreview> {
    throw new Error("No interrupted fixture patch needs recovery.");
  }

  async inspectHotpatch(installId: string): Promise<HotpatchPreview> {
    const current = this.#hotpatches.get(installId);
    if (current) return current;
    const copy = (await this.getUpdates()).managedCopies.find(entry => entry.installId === installId);
    if (!copy) throw new Error("Unknown fixture installation.");
    const preview: HotpatchPreview = {
      installId, state: "available", detail: "Preview only: Belhifet fights with new tactics.",
      baseRecipeVersion: copy.installedRecipeVersion ?? "Unknown",
      hotpatch: {
        id: "fixture-belhifet", title: "Belhifet fight", summary: "Preview only: Belhifet fights with new tactics.",
        changes: [{ title: "Belhifet", summary: "New tactics and a fairer fight.", saveApplicability: "before-area-visit", conditionNote: "Games that have not reached the final fight get it." }],
      },
      applied: [], refusals: [], notes: ["Your own choices in Tweaks Anthology do not affect hotpatches."],
      reviewToken: `fixture-hotpatch-${installId}`, canUndo: false, undoTitle: null, canRestore: false,
      fullBackupBytes: "4294967296", saveBackupBytes: "52428800", availableBytes: "107374182400",
      backupPath: "C:\Fixture\Hotpatch-Backups",
    };
    this.#hotpatches.set(installId, preview);
    return preview;
  }

  async applyHotpatch(installId: string, reviewToken: string, _fullBackup: boolean, _saveBackup: boolean): Promise<HotpatchPreview> {
    const preview = this.#hotpatches.get(installId);
    if (!preview || preview.state !== "available" || preview.reviewToken !== reviewToken || !preview.hotpatch) throw new Error("Check this fixture installation again before applying.");
    const applied: HotpatchPreview = {
      ...preview, state: "up-to-date", detail: "Every hotpatch for your collection version is installed.", hotpatch: null,
      applied: [{ id: preview.hotpatch.id, title: preview.hotpatch.title }], reviewToken: null, canUndo: true, undoTitle: preview.hotpatch.title,
    };
    this.#hotpatches.set(installId, applied);
    return applied;
  }

  async restoreHotpatch(installId: string, undo: boolean): Promise<HotpatchPreview> {
    const preview = this.#hotpatches.get(installId);
    if (!undo || !preview?.canUndo) throw new Error("No fixture hotpatch can be undone or restored.");
    this.#hotpatches.delete(installId);
    return this.inspectHotpatch(installId);
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
    const planPhases = flavor.id === "iwd" ? iwdPhases : phases;
    const currentIndex = state === "complete" ? planPhases.length : Math.min(this.#buildIndex, planPhases.length - 1);
    return {
      state,
      headline: headlines[state],
      detail: details[state],
      phases: planPhases.map((phase, index) => ({
        ...phase,
        state: state === "failed" && index === currentIndex ? "failed" : index < currentIndex ? "done" : index === currentIndex ? "current" : "pending",
      })),
      logTail: ["fixture: validated immutable review fixture-review-8d6d75", this.#options.textOverrides?.logLine ?? "fixture: no game files were touched"],
      manualArchiveName: state === "waiting-manual" ? "example-manual-mod.zip" : null,
    };
  }
}

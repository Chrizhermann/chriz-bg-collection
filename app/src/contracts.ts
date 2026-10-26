export type Route =
  | "home"
  | "updates"
  | "welcome"
  | "games"
  | "destination"
  | "setup"
  | "review"
  | "build"
  | "complete";

export interface BackendStatus {
  readonly mode: "fixture" | "native";
  readonly applicationVersion?: string;
  readonly engineVersion: string;
  readonly recipeVersion: string | null;
  readonly startupInstallId: string | null;
  readonly profiles?: readonly { readonly id: string; readonly label: string; readonly description: string }[];
  readonly selectedProfile?: string;
  /** The standalone collection this backend serves; absent for the Baldur's Gate collection. */
  readonly collection?: CollectionIdentity;
}

/** A standalone collection's identity, as `bootstrap` and `select_profile` report it. */
export interface CollectionIdentity {
  readonly flavor: string;
  readonly game: string;
  readonly productName: string;
  readonly sources: readonly { readonly slot: string; readonly role: string; readonly label: string }[];
}

export interface InstallationDefaults {
  readonly name: string;
  readonly path: string;
}

export interface CommandErrorPayload {
  readonly code: string;
  readonly message: string;
  readonly recovery_action: string;
  readonly technical_detail: string;
}

export type GameRole = "bgee_sod" | "bg2ee" | "iwdee";
export type Storefront = "steam" | "gog";

export type Freshness =
  | "fresh"
  | "modified"
  | "unsupported-version"
  | "missing-sod"
  | "unverified-storefront"
  | "unknown-fingerprint"
  | "unsupported-locale";

export interface GameCandidate {
  readonly id: string;
  readonly label: string;
  readonly path: string;
  readonly storefront: Storefront;
  readonly build: string | null;
  readonly freshness: Freshness;
  readonly eligible: boolean;
  readonly findings: readonly string[];
}

export interface GameDiscovery {
  readonly bg1Candidates: readonly GameCandidate[];
  readonly bg2Candidates: readonly GameCandidate[];
  readonly selectedBg1Id: string;
  readonly selectedBg2Id: string;
}

export interface DestinationEvaluation {
  readonly path: string;
  readonly safe: boolean;
  readonly title: string;
  readonly detail: string;
  readonly requiredSpace?: string;
  readonly availableSpace?: string;
}

export type FeatureDecision = "excluded" | "optional" | "default" | "mandatory";

export interface InputOption {
  readonly id: string;
  readonly title: string;
  readonly answer: string;
}

export type InputSpec =
  | { readonly kind: "boolean"; readonly id: string; readonly default: boolean }
  | { readonly kind: "choice"; readonly id: string; readonly default: string; readonly options: readonly InputOption[] }
  | { readonly kind: "integer"; readonly id: string; readonly default: number; readonly min: number; readonly max: number };

export type InputValue =
  | { readonly kind: "boolean"; readonly value: boolean }
  | { readonly kind: "choice"; readonly value: string }
  | { readonly kind: "integer"; readonly value: number };

export interface InputControl {
  readonly spec: InputSpec;
  readonly value: InputValue;
  readonly interactive: boolean;
}

export interface FeatureControl {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly category: string;
  readonly sourceLabel?: string | null;
  readonly groupLabel?: string | null;
  readonly choiceGroup?: string | null;
  readonly choiceAvailable?: boolean | null;
  readonly decision: FeatureDecision;
  readonly readiness: "ready" | "experimental" | "blocked";
  readonly parent: string | null;
  readonly requires?: readonly string[];
  readonly requiresAny?: readonly string[];
  readonly conflicts?: readonly string[];
  readonly selected: boolean;
  readonly interactive: boolean;
  readonly unavailableReason: string | null;
  readonly inputs: readonly InputControl[];
}

export interface RecipeView {
  readonly categories: readonly string[];
  readonly controls: readonly FeatureControl[];
}

export interface NormalizedSelection {
  readonly platform: string;
  readonly features: Readonly<Record<string, boolean>>;
  readonly inputs: Readonly<Record<string, Readonly<Record<string, InputValue>>>>;
}

export interface SelectionFinding {
  readonly rule: string;
  readonly featureId: string;
  readonly message: string;
}

export interface PhaseSummary {
  readonly id: string;
  readonly title: string;
  readonly detail: string;
}

export type FeatureGroupState = "on" | "off" | "mixed";

export interface UnavailableGroupMember {
  readonly featureId: string;
  readonly reason: string;
}

/**
 * A choice-group alternative the group picks while it is on. Switching the group off restores
 * `recipeDefault`; `selected` and `available` are derived by the engine like member state.
 */
export interface FeatureGroupVariant {
  readonly choiceGroup: string;
  readonly option: string;
  readonly label: string;
  readonly options: readonly string[];
  readonly recipeDefault: readonly string[];
  readonly selected: boolean;
  readonly available: boolean;
  readonly unavailableReason: string | null;
}

/** An authored feature group plus its state, derived by the engine from member selections. */
export interface FeatureGroup {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly members: readonly string[];
  readonly state: FeatureGroupState;
  readonly selectedMembers: readonly string[];
  readonly availableMembers: readonly string[];
  readonly unavailableMembers: readonly UnavailableGroupMember[];
  readonly disabledReason: string | null;
  /** Omitted when the group has no variants. */
  readonly variants?: readonly FeatureGroupVariant[];
}

export interface SelectionEvaluation {
  readonly manualCheckpointOptions?: readonly ManualCheckpointOption[];
  readonly featureGroups?: readonly FeatureGroup[];
  readonly view: RecipeView;
  readonly normalizedSelection: NormalizedSelection;
  readonly findings: readonly SelectionFinding[];
  readonly plan: {
    readonly phases: readonly PhaseSummary[];
  };
  readonly selectedChoiceCount: number;
}

export interface ManualBreakpoint {
  readonly runId: string;
  readonly component: number;
}

export interface ManualCheckpointOption extends ManualBreakpoint {
  readonly modId: string;
  readonly modLabel: string;
  readonly componentLabel: string;
  readonly phase: string;
  readonly target: "bg1" | "bg2";
  readonly unavailableReason: string | null;
}

export interface ManualCheckpointStatus {
  readonly checkpointId: string;
  readonly target: "bg1" | "bg2";
  readonly targetPath: string;
  readonly after: ManualCheckpointOption;
  readonly next: ManualCheckpointOption | null;
  readonly additions: readonly { readonly tp2: string; readonly language: number; readonly component: number; readonly description: string | null }[];
  readonly diagnostics: readonly string[];
  readonly reviewToken: string;
  readonly customized: boolean;
}

export interface FrozenReview {
  readonly reviewToken: string;
  readonly digest: string;
  readonly displayName: string;
  readonly destination: string;
  readonly gameLabels: readonly string[];
  readonly evaluation: SelectionEvaluation;
}

export type EngineEvent =
  | { readonly type: "campaign_started"; readonly install_id: string; readonly resumed: boolean }
  | { readonly type: "phase_started"; readonly name: string }
  | { readonly type: "step_started"; readonly id: string; readonly label: string }
  | { readonly type: "step_progress"; readonly id: string; readonly done: number; readonly total: number }
  | { readonly type: "console_line"; readonly step_id: string; readonly stream: "stdout" | "stderr"; readonly line: string }
  | { readonly type: "attention_required"; readonly step_id: string; readonly reason: string; readonly last_output: string }
  | { readonly type: "step_finished"; readonly id: string; readonly outcome: "succeeded" | "failed" | "skipped" }
  | { readonly type: "campaign_finished"; readonly install_id: string }
  | { readonly type: "campaign_paused"; readonly install_id: string; readonly after_step_id: string }
  | { readonly type: "manual_download_needed"; readonly mod_id: string; readonly page: string; readonly expected_sha256: string; readonly drop_dir: string }
  | { readonly type: "error"; readonly step_id: string | null; readonly message: string };

export interface RunEventEnvelope {
  readonly runId: string;
  readonly sequenceAsString: string;
  readonly event: EngineEvent;
}

export interface StartBuildResponse {
  readonly runId: string;
}

export interface ManualArchiveSupply {
  readonly artifactId: string;
  readonly filename: string;
  readonly sha256: string;
  readonly length: number;
}

export interface ManualDownloadRequirement {
  readonly artifactId: string;
  readonly modIds: readonly string[];
  readonly title: string;
  readonly filename: string;
  readonly length: number;
  readonly ready: boolean;
  readonly detail: string | null;
}

export interface CampaignReport {
  readonly install_id: string;
  readonly managed_root: string;
  readonly plan_sha256: string;
  readonly status: { readonly status: string; readonly step_id?: string; readonly reason?: string };
}

export interface RunSnapshot {
  readonly runId: string;
  readonly status: string;
  readonly events: readonly RunEventEnvelope[];
  readonly report: CampaignReport | null;
  readonly error: CommandErrorPayload | null;
}

export type BuildState =
  | "waiting-manual"
  | "attention"
  | "failed"
  | "running"
  | "pausing"
  | "paused"
  | "complete";

export type PhaseState = "done" | "current" | "pending" | "failed";

export interface BuildPhase extends PhaseSummary {
  readonly state: PhaseState;
}

export interface OperationProgress {
  readonly stage: string;
  readonly completed?: string | null;
  readonly total?: string | null;
  readonly unit?: "bytes" | "items" | null;
}

export interface BuildSnapshot {
  readonly progress?: OperationProgress;
  readonly state: BuildState;
  readonly headline: string;
  readonly detail: string;
  readonly phases: readonly BuildPhase[];
  readonly logTail: readonly string[];
  readonly manualArchiveName: string | null;
  readonly recoveryAction?: string;
  readonly failureReason?: string;
  readonly commandError?: CommandErrorPayload;
  readonly freshCopyRequired?: boolean;
}

export interface ManagedInstallation {
  readonly customized?: boolean;
  readonly id: string;
  readonly name: string;
  readonly path: string;
  readonly status: string;
  readonly receiptPath: string | null;
  readonly launchPath: string | null;
  readonly completedAtMillis: number | null;
  readonly available: boolean;
  readonly resumable: boolean;
  readonly patchRecoveryNeeded?: boolean;
  readonly recipeVersion?: string | null;
  readonly radarVersion?: string | null;
  readonly consistency?: {
    readonly state: "matches" | "changed" | "unavailable";
    readonly detail: string;
    readonly componentCount: number;
    readonly modCount: number;
    readonly components?: readonly InstalledComponent[];
  };
}

export interface LaunchInstallResult {
  readonly installId: string;
  readonly radar: {
    readonly state: "not-requested" | "started" | "cancelled" | "failed";
    readonly detail: string | null;
    readonly technicalDetail: string | null;
  };
  /** Installer-owned profile settings checked before the launch; never blocks playing. */
  readonly gameSettings: {
    readonly state: "unchanged" | "updated" | "failed";
    readonly technicalDetail: string | null;
  };
  /** The last session's game log, kept before the game starts a new one; never blocks playing. */
  readonly gameLog: {
    readonly state: "not-enabled" | "nothing-to-keep" | "kept" | "failed";
    readonly technicalDetail: string | null;
  };
}

/** One file of the game log's evidence, as last seen. */
export interface GameLogFile {
  readonly path: string;
  readonly state: "missing" | "present" | "not-a-file" | "unreadable";
  /** Decimal byte counts remain strings across the native boundary. */
  readonly sizeBytes: string | null;
  readonly modifiedAtMillis: number | null;
  readonly detail: string | null;
}

/** Whether the game writes its own log for crash reports, and where that evidence is. */
export interface GameLogStatus {
  readonly installId: string;
  /** `other` names a log file set outside CEBG in `detail`; `unavailable` says why in `detail`. */
  readonly setting: "off" | "on" | "other" | "unavailable";
  readonly detail: string | null;
  readonly log: GameLogFile;
  readonly previousLog: GameLogFile;
  readonly crashFolder: string;
  /** Crashes with a report in the crash folder; null when it cannot be looked into. */
  readonly crashReports: number | null;
}

export interface InstallationRemovalPreview {
  readonly installId: string;
  readonly displayName: string;
  readonly managedRoot: string;
  /** `unlist` removes only the entry: the folder exists but cannot be proven to be this installation. */
  readonly action: "delete" | "forget" | "unlist";
  /** Why the folder will not be deleted, only for `unlist`. */
  readonly reason?: string | null;
  /** Advisory logical file bytes; absent when the bounded scan cannot finish. */
  readonly approximateSizeBytes?: string | null;
  readonly preservedSavePath: string | null;
  readonly confirmationToken: string;
}

export interface InstalledComponent {
  readonly target: "BG1" | "BG2";
  readonly tp2: string;
  readonly component: number;
  readonly title: string | null;
  readonly version: string | null;
  readonly status: "installed" | "missing" | "extra";
}

export type AppUpdateState = "up-to-date" | "available" | "offline" | "invalid" | "unavailable";
export type RecipeUpdateState = "up-to-date" | "available" | "requires-app" | "offline" | "invalid" | "replayed" | "unavailable";
export type ManagedCopyUpdateState = "up-to-date" | "update-available" | "unknown" | "stale";

export interface ApplicationUpdate {
  readonly state: AppUpdateState;
  readonly currentVersion: string;
  readonly availableVersion: string | null;
  readonly detail: string;
  readonly releaseNotes?: string;
}

export interface RecipeUpdateChange {
  readonly title: string;
  readonly summary: string;
  readonly saveApplicability: "current-save" | "before-npc-join" | "before-area-visit" | "before-event" | "next-playthrough" | "new-game-only" | "unknown";
  readonly urgency: "critical" | "recommended" | "optional" | "informational";
  readonly conditionNote: string | null;
}

export interface RecipeUpdate {
  readonly state: RecipeUpdateState;
  readonly currentVersion: string;
  readonly availableVersion: string | null;
  readonly disposition: "up-to-date" | "deferred-for-next-playthrough" | "may-affect-current-playthrough" | "unknown-applicability" | "app-update-required";
  readonly detail: string;
  readonly changes: readonly RecipeUpdateChange[];
}

export interface ManagedCopyUpdate {
  readonly installId: string;
  readonly name: string;
  readonly path: string;
  readonly installedRecipeVersion: string | null;
  readonly state: ManagedCopyUpdateState;
  readonly detail: string;
}

/** Read-only comparison; no state in this contract authorizes applying an update. */
export interface InstalledUpdatePlan {
  readonly installId: string;
  readonly baseRecipeVersion: string;
  readonly targetRecipeVersion: string;
  readonly state: "comparison-ready" | "blocked";
  readonly detail: string;
  readonly choicesPreserved: boolean;
  readonly issues: readonly {
    readonly rule: string;
    readonly featureId: string | null;
    readonly message: string;
  }[];
  readonly runs: readonly {
    readonly runId: string;
    readonly modName: string;
    readonly installedVersion: string;
    readonly targetVersion: string | null;
    readonly components: readonly number[];
    readonly state: "unchanged" | "unassessed" | "requires-migration";
    readonly detail: string;
  }[];
}

/** One explicitly supported whole-mod transition, not an entire collection upgrade. */
export interface ModUpgradePreview {
  readonly installId: string;
  readonly upgradeId: string;
  readonly title: string;
  readonly state: "available" | "already-current" | "not-selected" | "unsupported" | "applied" | "needs-recovery";
  readonly detail: string;
  readonly baseRecipeVersion: string;
  readonly installedVersion: string | null;
  readonly targetVersion: string;
  readonly reviewToken: string | null;
  readonly canUndo: boolean;
  readonly canRestore: boolean;
  readonly fullBackupBytes: string;
  readonly saveBackupBytes: string;
  readonly availableBytes: string;
  readonly backupPath: string;
  readonly saveCompatibility: string;
  readonly history: readonly {
    readonly transactionId: string;
    readonly fromVersion: string;
    readonly toVersion: string;
    readonly state: string;
  }[];
}

/** One independently verified fix, not a whole-mod or collection upgrade. */
export interface PatchPreview {
  readonly installId: string;
  readonly patchId: string;
  readonly title: string;
  readonly state: "available" | "already-fixed" | "not-selected" | "unsupported" | "applied" | "needs-recovery";
  readonly detail: string;
  readonly baseRecipeVersion: string;
  readonly appliedPatchIds: readonly string[];
  readonly reviewToken: string | null;
  readonly canUndo: boolean;
  readonly canRestore: boolean;
  /** Decimal byte counts remain strings across the native boundary. */
  readonly fullBackupBytes: string;
  readonly saveBackupBytes: string;
  readonly availableBytes: string;
  readonly backupPath: string;
}

export interface UpdateSummary {
  readonly checkedAt: string | null;
  readonly networkState: "online" | "offline" | "unconfigured";
  readonly application: ApplicationUpdate;
  readonly recipe: RecipeUpdate;
  readonly managedCopies: readonly ManagedCopyUpdate[];
  readonly radar?: {
    readonly state: "up-to-date" | "available" | "not-installed" | "offline";
    readonly currentVersion: string | null;
    readonly availableVersion: string | null;
    readonly detail: string;
    readonly releaseNotes?: string;
  };
}

export interface FixtureOptions {
  readonly evaluationDelays?: readonly number[];
  readonly managedInstallations?: readonly ManagedInstallation[];
  readonly textOverrides?: {
    readonly campaignName?: string;
    readonly logLine?: string;
    readonly gamePath?: string;
    readonly featureTitle?: string;
    readonly unavailableReason?: string;
  };
}

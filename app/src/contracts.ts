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

export type GameRole = "bgee_sod" | "bg2ee";
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
  readonly decision: FeatureDecision;
  readonly readiness: "ready" | "experimental" | "blocked";
  readonly parent: string | null;
  readonly requires?: readonly string[];
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

export interface SelectionEvaluation {
  readonly view: RecipeView;
  readonly normalizedSelection: NormalizedSelection;
  readonly findings: readonly SelectionFinding[];
  readonly plan: {
    readonly phases: readonly PhaseSummary[];
  };
  readonly selectedChoiceCount: number;
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
  | "complete";

export type PhaseState = "done" | "current" | "pending" | "failed";

export interface BuildPhase extends PhaseSummary {
  readonly state: PhaseState;
}

export interface BuildSnapshot {
  readonly state: BuildState;
  readonly headline: string;
  readonly detail: string;
  readonly phases: readonly BuildPhase[];
  readonly logTail: readonly string[];
  readonly manualArchiveName: string | null;
  readonly recoveryAction?: string;
  readonly failureReason?: string;
  readonly freshCopyRequired?: boolean;
}

export interface ManagedInstallation {
  readonly id: string;
  readonly name: string;
  readonly path: string;
  readonly status: string;
  readonly receiptPath: string | null;
  readonly launchPath: string | null;
  readonly completedAtMillis: number | null;
  readonly available: boolean;
  readonly resumable: boolean;
  readonly recipeVersion?: string | null;
  readonly radarVersion?: string | null;
  readonly consistency?: {
    readonly state: "matches" | "changed" | "unavailable";
    readonly detail: string;
    readonly componentCount: number;
    readonly modCount: number;
  };
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

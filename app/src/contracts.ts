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
  readonly engineVersion: string;
  readonly recipeVersion: string | null;
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
  readonly requiredSpace: string;
  readonly availableSpace: string;
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
  readonly digest: string;
  readonly destination: string;
  readonly gameLabels: readonly string[];
  readonly evaluation: SelectionEvaluation;
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
}

export interface ManagedInstallation {
  readonly id: string;
  readonly name: string;
  readonly path: string;
  readonly status: string;
  readonly receiptPath: string;
}

export interface UpdateSummary {
  readonly app: string;
  readonly recipe: string;
  readonly message: string;
}

export interface FixtureOptions {
  readonly evaluationDelays?: readonly number[];
  readonly textOverrides?: {
    readonly campaignName?: string;
    readonly logLine?: string;
    readonly gamePath?: string;
    readonly featureTitle?: string;
    readonly unavailableReason?: string;
  };
}

import type {
  BuildSnapshot,
  DestinationEvaluation,
  FrozenReview,
  GameCandidate,
  NormalizedSelection,
  Route,
  SelectionEvaluation,
} from "./contracts";

export interface AppState {
  readonly route: Route;
  readonly history: readonly Route[];
  readonly selectedBg1Id: string;
  readonly selectedBg2Id: string;
  readonly installationName: string;
  readonly destinationPath: string;
  readonly destinationAutomatic: boolean;
  readonly selection: NormalizedSelection;
  readonly evaluation: SelectionEvaluation | null;
  readonly evaluationRevision: number;
  readonly evaluationPending: boolean;
  readonly frozenReview: FrozenReview | null;
  readonly build: BuildSnapshot | null;
}

export type AppAction =
  | { readonly type: "navigate"; readonly route: Route; readonly remember?: boolean }
  | { readonly type: "back" }
  | { readonly type: "select-game"; readonly game: "bg1" | "bg2"; readonly id: string }
  | { readonly type: "set-installation-defaults"; readonly name: string; readonly path: string }
  | { readonly type: "set-installation-name"; readonly name: string }
  | { readonly type: "set-destination"; readonly path: string; readonly automatic?: boolean }
  | { readonly type: "set-feature"; readonly id: string; readonly selected: boolean }
  | { readonly type: "evaluation-requested"; readonly revision: number }
  | { readonly type: "evaluation-resolved"; readonly revision: number; readonly evaluation: SelectionEvaluation }
  | { readonly type: "review-frozen"; readonly review: FrozenReview }
  | { readonly type: "review-cleared" }
  | { readonly type: "build-updated"; readonly build: BuildSnapshot }
  | { readonly type: "build-cleared" };

export function initialState(): AppState {
  return {
    route: "welcome",
    history: [],
    selectedBg1Id: "bg1-fresh",
    selectedBg2Id: "bg2-fresh",
    installationName: "Chriz Easy BG",
    destinationPath: "",
    destinationAutomatic: true,
    selection: { platform: "windows", features: {}, inputs: {} },
    evaluation: null,
    evaluationRevision: 0,
    evaluationPending: false,
    frozenReview: null,
    build: null,
  };
}

export function reduce(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "navigate":
      if (action.route === "build" && state.frozenReview === null && state.build === null) return state;
      return { ...state, route: action.route, history: action.remember === false ? state.history : [...state.history, state.route] };
    case "back": {
      const route = state.history.at(-1);
      return route === undefined ? state : { ...state, route, history: state.history.slice(0, -1) };
    }
    case "select-game":
      return action.game === "bg1" ? { ...state, selectedBg1Id: action.id } : { ...state, selectedBg2Id: action.id };
    case "set-installation-defaults":
      return { ...state, installationName: action.name, destinationPath: action.path, destinationAutomatic: true };
    case "set-installation-name":
      return { ...state, installationName: action.name };
    case "set-destination":
      return { ...state, destinationPath: action.path, destinationAutomatic: action.automatic ?? state.destinationAutomatic };
    case "set-feature":
      return { ...state, selection: { ...state.selection, features: { ...state.selection.features, [action.id]: action.selected } } };
    case "evaluation-requested":
      return { ...state, evaluationRevision: action.revision, evaluationPending: true };
    case "evaluation-resolved":
      return action.revision === state.evaluationRevision
        ? { ...state, evaluation: action.evaluation, selection: action.evaluation.normalizedSelection, evaluationPending: false }
        : state;
    case "review-frozen":
      return { ...state, frozenReview: action.review };
    case "review-cleared":
      return { ...state, frozenReview: null };
    case "build-updated":
      return { ...state, build: action.build };
    case "build-cleared":
      return { ...state, build: null };
  }
}

export function validateInstallationName(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed.length === 0) return "Enter an install name.";
  if (/[\u0000-\u001f\u007f]/u.test(name)) return "The install name cannot contain control characters.";
  if (/[<>:"/\\|?*]/u.test(name)) return "The install name cannot contain < > : \" / \\ | ? or *.";
  if (trimmed === "." || trimmed === "..") return "Choose a name that can be used as a folder.";
  return null;
}

export function automaticInstallationPath(defaultPath: string, name: string): string {
  const withoutTrailingSeparators = defaultPath.replace(/[\\/]+$/u, "");
  const separatorIndex = Math.max(withoutTrailingSeparators.lastIndexOf("\\"), withoutTrailingSeparators.lastIndexOf("/"));
  if (separatorIndex < 0) return name;
  return `${withoutTrailingSeparators.slice(0, separatorIndex + 1)}${name}`;
}

export interface InstallationReadinessInput {
  readonly starting: boolean;
  readonly name: string;
  readonly bg1: GameCandidate | undefined;
  readonly bg2: GameCandidate | undefined;
  readonly destination: DestinationEvaluation;
  readonly evaluation: SelectionEvaluation | null;
  readonly evaluationPending: boolean;
}

export function installationReadiness(input: InstallationReadinessInput): boolean {
  return !input.starting
    && validateInstallationName(input.name) === null
    && input.bg1?.eligible === true
    && input.bg2?.eligible === true
    && input.destination.safe
    && input.evaluation !== null
    && !input.evaluationPending;
}

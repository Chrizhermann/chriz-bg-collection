import type { BuildSnapshot, FrozenReview, NormalizedSelection, Route, SelectionEvaluation } from "./contracts";

export interface AppState {
  readonly route: Route;
  readonly history: readonly Route[];
  readonly selectedBg1Id: string;
  readonly selectedBg2Id: string;
  readonly destinationPath: string;
  readonly selection: NormalizedSelection;
  readonly evaluation: SelectionEvaluation | null;
  readonly evaluationRevision: number;
  readonly frozenReview: FrozenReview | null;
  readonly build: BuildSnapshot | null;
}

export type AppAction =
  | { readonly type: "navigate"; readonly route: Route; readonly remember?: boolean }
  | { readonly type: "back" }
  | { readonly type: "select-game"; readonly game: "bg1" | "bg2"; readonly id: string }
  | { readonly type: "set-destination"; readonly path: string }
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
    destinationPath: "D:\\Fixture Campaigns\\Chriz EET Alpha",
    selection: { platform: "windows", features: {}, inputs: {} },
    evaluation: null,
    evaluationRevision: 0,
    frozenReview: null,
    build: null,
  };
}

export function reduce(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "navigate":
      if (action.route === "build" && state.frozenReview === null) return state;
      return { ...state, route: action.route, history: action.remember === false ? state.history : [...state.history, state.route] };
    case "back": {
      const route = state.history.at(-1);
      return route === undefined ? state : { ...state, route, history: state.history.slice(0, -1) };
    }
    case "select-game":
      return action.game === "bg1" ? { ...state, selectedBg1Id: action.id } : { ...state, selectedBg2Id: action.id };
    case "set-destination":
      return { ...state, destinationPath: action.path };
    case "set-feature":
      return { ...state, selection: { ...state.selection, features: { ...state.selection.features, [action.id]: action.selected } } };
    case "evaluation-requested":
      return { ...state, evaluationRevision: action.revision };
    case "evaluation-resolved":
      return action.revision === state.evaluationRevision
        ? { ...state, evaluation: action.evaluation, selection: action.evaluation.normalizedSelection }
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

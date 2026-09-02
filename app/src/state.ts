export interface FrozenReview {
  readonly digest: string;
}

export type AppState =
  | { readonly route: "welcome"; readonly frozenReview: FrozenReview | null }
  | { readonly route: "build"; readonly frozenReview: FrozenReview };

export type AppAction = { readonly type: "continue" };

export function initialState(): AppState {
  return { route: "welcome", frozenReview: null };
}

export function reduce(state: AppState, action: AppAction): AppState {
  if (
    action.type === "continue" &&
    state.route === "welcome" &&
    state.frozenReview !== null
  ) {
    return { route: "build", frozenReview: state.frozenReview };
  }

  return state;
}

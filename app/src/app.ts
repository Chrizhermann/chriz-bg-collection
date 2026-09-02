import type { Backend } from "./backend";
import { createAppShell } from "./components/app-shell";
import type { TechnicalLogState } from "./components/technical-log";
import type { DestinationEvaluation, GameCandidate, GameDiscovery, ManagedInstallation, Route, UpdateSummary } from "./contracts";
import { buildScreen } from "./screens/build";
import { completeScreen } from "./screens/complete";
import { destinationScreen } from "./screens/destination";
import { gamesScreen } from "./screens/games";
import { homeScreen } from "./screens/home";
import { reviewScreen } from "./screens/review";
import { setupScreen } from "./screens/setup";
import { updatesScreen } from "./screens/updates";
import { welcomeScreen } from "./screens/welcome";
import { initialState, reduce, type AppAction, type AppState } from "./state";

export interface AppHandle {
  navigate(route: Route): Promise<void>;
}

class AppController implements AppHandle {
  #state: AppState = initialState();
  #discovery!: GameDiscovery;
  #destination!: DestinationEvaluation;
  #installations: readonly ManagedInstallation[] = [];
  #updates!: UpdateSummary;
  #revision = 0;
  #logState: TechnicalLogState = { paused: false, open: false };

  constructor(private readonly root: HTMLElement, private readonly backend: Backend) {}

  async initialize(): Promise<void> {
    [this.#discovery, this.#destination, this.#installations, this.#updates] = await Promise.all([
      this.backend.discoverGames(),
      this.backend.inspectDestination(this.#state.destinationPath),
      this.backend.listManagedInstallations(),
      this.backend.getUpdates(),
    ]);
    this.#dispatch({ type: "select-game", game: "bg1", id: this.#discovery.selectedBg1Id });
    this.#dispatch({ type: "select-game", game: "bg2", id: this.#discovery.selectedBg2Id });
    await this.#evaluate(false);
    this.#render();
  }

  async navigate(route: Route): Promise<void> {
    if (route === "build" && this.#state.frozenReview === null) await this.#freezeReview(false);
    if (route === "build") this.#dispatch({ type: "build-updated", build: await this.backend.getBuildSnapshot() });
    this.#dispatch({ type: "navigate", route });
    this.#render();
  }

  #dispatch(action: AppAction): void {
    this.#state = reduce(this.#state, action);
  }

  #back(): void {
    this.#dispatch({ type: "back" });
    this.#render();
  }

  async #evaluate(render = true, focusTargetId?: string): Promise<void> {
    const revision = ++this.#revision;
    this.#dispatch({ type: "evaluation-requested", revision });
    const evaluation = await this.backend.evaluateBuild(this.#state.selection);
    this.#dispatch({ type: "evaluation-resolved", revision, evaluation });
    if (render && revision === this.#state.evaluationRevision) this.#render(focusTargetId);
  }

  async #toggleFeature(id: string, selected: boolean): Promise<void> {
    this.#dispatch({ type: "set-feature", id, selected });
    await this.#evaluate(true, `feature-${id}`);
  }

  async #inspectDestination(path: string): Promise<void> {
    this.#destination = await this.backend.inspectDestination(path);
    this.#dispatch({ type: "set-destination", path: this.#destination.path });
    this.#render();
  }

  #selectedGames(): readonly GameCandidate[] {
    const bg1 = this.#discovery.bg1Candidates.find((candidate) => candidate.id === this.#state.selectedBg1Id);
    const bg2 = this.#discovery.bg2Candidates.find((candidate) => candidate.id === this.#state.selectedBg2Id);
    return [bg1, bg2].filter((candidate): candidate is GameCandidate => candidate !== undefined);
  }

  async #freezeReview(render = true): Promise<void> {
    const gameLabels = this.#selectedGames().map((candidate) => candidate.label);
    const review = await this.backend.freezeReview(this.#state.selection, this.#destination.path, gameLabels);
    this.#dispatch({ type: "review-frozen", review });
    if (render) await this.navigate("build");
  }

  async #advanceBuild(): Promise<void> {
    const build = await this.backend.advanceBuild();
    this.#dispatch({ type: "build-updated", build });
    if (build.state === "complete") {
      this.#dispatch({ type: "navigate", route: "complete" });
    }
    this.#render();
  }

  async #retryBuild(): Promise<void> {
    this.#dispatch({ type: "build-updated", build: await this.backend.retryBuild() });
    this.#render();
  }

  async #exportDiagnostics(): Promise<void> {
    const result = await this.backend.exportDiagnostics();
    const announcement = document.createElement("p");
    announcement.className = "diagnostics-result";
    announcement.textContent = `Diagnostics exported to ${result.path}`;
    announcement.tabIndex = -1;
    this.root.querySelector("main")?.append(announcement);
    announcement.focus();
  }

  #render(focusTargetId?: string): void {
    const evaluation = this.#state.evaluation;
    if (evaluation === null) return;
    const navigate = (route: Route): Promise<void> => this.navigate(route);
    let content: HTMLElement;
    switch (this.#state.route) {
      case "home":
        content = homeScreen(this.#installations, () => void navigate("welcome"));
        break;
      case "updates":
        content = updatesScreen(this.#updates);
        break;
      case "welcome":
        content = welcomeScreen(() => navigate("games"));
        break;
      case "games":
        content = gamesScreen(
          this.#discovery,
          this.#state.selectedBg1Id,
          this.#state.selectedBg2Id,
          (game, id) => this.#dispatch({ type: "select-game", game, id }),
          () => this.#back(),
          () => navigate("destination"),
        );
        break;
      case "destination":
        content = destinationScreen(this.#destination, (path) => this.#inspectDestination(path), () => this.#back(), () => void navigate("setup"));
        break;
      case "setup":
        content = setupScreen(evaluation, (id, selected) => this.#toggleFeature(id, selected), () => this.#back(), () => void navigate("review"));
        break;
      case "review":
        content = reviewScreen(evaluation, this.#destination, this.#selectedGames(), () => this.#back(), () => this.#freezeReview());
        break;
      case "build":
        content = buildScreen(this.#state.build ?? { state: "running", headline: "Build in progress", detail: "Loading fixture snapshot.", phases: evaluation.plan.phases.map((phase, index) => ({ ...phase, state: index === 0 ? "current" : "pending" })), logTail: [], manualArchiveName: null }, {
          advance: () => this.#advanceBuild(),
          retry: () => this.#retryBuild(),
          diagnostics: () => this.#exportDiagnostics(),
          logState: this.#logState,
          updateLogState: (state) => { this.#logState = state; },
        });
        break;
      case "complete":
        content = completeScreen(this.#state.frozenReview, () => void navigate("home"));
        break;
    }
    this.root.replaceChildren(createAppShell(this.#state.route, content, navigate));
    (focusTargetId ? this.root.querySelector<HTMLElement>(`#${focusTargetId}`) : this.root.querySelector<HTMLElement>("h1"))?.focus();
  }
}

export async function mountApp(root: HTMLElement, backend: Backend): Promise<AppHandle> {
  const controller = new AppController(root, backend);
  await controller.initialize();
  return controller;
}

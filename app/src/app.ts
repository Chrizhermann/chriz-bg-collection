import { BackendCommandError, type Backend } from "./backend";
import { createAppShell } from "./components/app-shell";
import type { TechnicalLogState } from "./components/technical-log";
import type { BackendStatus, BuildSnapshot, DestinationEvaluation, GameCandidate, GameDiscovery, ManagedInstallation, Route, RunEventEnvelope, UpdateSummary } from "./contracts";
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
import { statusCard } from "./components/status-card";

export interface AppHandle {
  navigate(route: Route): Promise<void>;
}

class AppController implements AppHandle {
  #state: AppState = initialState();
  #status!: BackendStatus;
  #discovery!: GameDiscovery;
  #destination: DestinationEvaluation = {
    path: "",
    safe: false,
    title: "Choose a new campaign folder",
    detail: "The installer will verify that it is separate from both clean source games.",
  };
  #installations: readonly ManagedInstallation[] = [];
  #updates: UpdateSummary = {
    app: "Not checked",
    recipe: "Not checked",
    message: "Signed update checks are not connected in this private alpha.",
  };
  #revision = 0;
  #runId: string | null = null;
  #installId: string | null = null;
  #retryAvailable = false;
  #commandError: BackendCommandError | null = null;
  #logState: TechnicalLogState = { paused: false, open: false };

  constructor(private readonly root: HTMLElement, private readonly backend: Backend) {}

  async initialize(): Promise<void> {
    this.#status = await this.backend.getStatus();
    this.#discovery = await this.backend.discoverGames();
    this.#dispatch({ type: "select-game", game: "bg1", id: this.#discovery.selectedBg1Id });
    this.#dispatch({ type: "select-game", game: "bg2", id: this.#discovery.selectedBg2Id });
    if (this.#status.mode === "fixture") {
      [this.#destination, this.#installations, this.#updates] = await Promise.all([
        this.backend.inspectDestination(
          this.#state.destinationPath,
          this.#state.selectedBg1Id,
          this.#state.selectedBg2Id,
        ),
        this.backend.listManagedInstallations(),
        this.backend.getUpdates(),
      ]);
      this.#installId = this.#installations[0]?.id ?? null;
    } else {
      this.#dispatch({ type: "set-destination", path: "" });
      this.#installations = await this.backend.listManagedInstallations();
    }
    await this.#evaluate(false);
    this.#render();
  }

  async navigate(route: Route): Promise<void> {
    const buildNeedsControls = this.#state.build !== null
      && (["running", "attention", "waiting-manual"].includes(this.#state.build.state)
        || (this.#state.build.state === "failed" && this.#retryAvailable));
    if (this.#status.mode === "native"
      && route !== "build"
      && buildNeedsControls) {
      route = "build";
    }
    if (route === "build" && this.#state.frozenReview === null) await this.#freezeReview(false);
    if (route === "build" && this.#status.mode === "fixture") {
      this.#dispatch({ type: "build-updated", build: await this.backend.getBuildSnapshot() });
    }
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
    this.#destination = {
      path,
      safe: false,
      title: "Checking this destination",
      detail: "The folder has not completed native safety inspection yet.",
    };
    this.#dispatch({ type: "set-destination", path });
    this.#render();
    this.#destination = await this.backend.inspectDestination(
      path,
      this.#state.selectedBg1Id,
      this.#state.selectedBg2Id,
    );
    this.#dispatch({ type: "set-destination", path: this.#destination.path });
    this.#render();
  }

  #selectedGames(): readonly GameCandidate[] {
    const bg1 = this.#discovery.bg1Candidates.find((candidate) => candidate.id === this.#state.selectedBg1Id);
    const bg2 = this.#discovery.bg2Candidates.find((candidate) => candidate.id === this.#state.selectedBg2Id);
    return [bg1, bg2].filter((candidate): candidate is GameCandidate => candidate !== undefined);
  }

  async #freezeReview(render = true): Promise<void> {
    try {
      const displayedEvaluation = this.#state.evaluation;
      const review = await this.backend.freezeReview(
        this.#state.selection,
        this.#destination.path,
        this.#state.selectedBg1Id,
        this.#state.selectedBg2Id,
      );
      if (this.#status.mode === "native"
        && JSON.stringify(review.evaluation) !== JSON.stringify(displayedEvaluation)) {
        const revision = ++this.#revision;
        this.#dispatch({ type: "evaluation-requested", revision });
        this.#dispatch({ type: "evaluation-resolved", revision, evaluation: review.evaluation });
        this.#dispatch({ type: "review-cleared" });
        throw new BackendCommandError({
          code: "review_display_changed",
          message: "The selected options changed while Review was open.",
          recovery_action: "Read the refreshed Review, then freeze it again.",
          technical_detail: "The server-frozen evaluation did not equal the evaluation displayed before Start.",
        });
      }
      this.#dispatch({ type: "review-frozen", review });
      if (this.#status.mode === "native") {
        this.#retryAvailable = false;
        this.#dispatch({ type: "build-updated", build: this.#initialNativeBuild() });
        const started = await this.backend.startBuild(review.reviewToken, (event) => this.#handleRunEvent(event));
        this.#runId = started.runId;
      }
      if (render && this.#state.build?.state !== "complete") await this.navigate("build");
    } catch (error: unknown) {
      this.#dispatch({ type: "review-cleared" });
      this.#dispatch({ type: "build-cleared" });
      throw error;
    }
  }

  async #advanceBuild(): Promise<void> {
    if (this.#status.mode === "native") {
      if (this.#runId === null) return;
      await this.backend.continueWaiting(this.#runId);
      const build = this.#state.build;
      if (build !== null) {
        this.#dispatch({ type: "build-updated", build: { ...build, state: "running", headline: "Build in progress", detail: "WeiDU is continuing under installer supervision." } });
      }
      this.#render();
      return;
    }
    const build = await this.backend.advanceBuild();
    this.#dispatch({ type: "build-updated", build });
    if (build.state === "complete") {
      this.#dispatch({ type: "navigate", route: "complete" });
    }
    this.#render();
  }

  async #retryBuild(): Promise<void> {
    if (this.#status.mode === "native") {
      if (this.#installId === null) return;
      const started = await this.backend.resumeBuild(this.#installId, (event) => this.#handleRunEvent(event));
      this.#runId = started.runId;
      this.#retryAvailable = false;
      if (this.#state.build?.state === "failed" || this.#state.build?.state === "waiting-manual") {
        this.#dispatch({ type: "build-updated", build: this.#initialNativeBuild() });
      }
      this.#render();
      return;
    }
    this.#dispatch({ type: "build-updated", build: await this.backend.retryBuild() });
    this.#render();
  }

  async #supplyManualArchive(): Promise<void> {
    const artifactId = this.#state.build?.manualArchiveName;
    if (artifactId === null || artifactId === undefined) return;
    const supplied = await this.backend.supplyManualArchive(artifactId);
    if (supplied === null) return;
    await this.#retryBuild();
  }

  async #openManualSource(): Promise<void> {
    const artifactId = this.#state.build?.manualArchiveName;
    if (artifactId === null || artifactId === undefined) return;
    await this.backend.openManualSource(artifactId);
  }

  async #chooseGameFolder(game: "bg1" | "bg2"): Promise<void> {
    const role = game === "bg1" ? "bgee_sod" : "bg2ee";
    const candidate = await this.backend.chooseGameFolder(role);
    if (candidate === null) return;
    const key = game === "bg1" ? "bg1Candidates" : "bg2Candidates";
    const candidates = this.#discovery[key].filter((entry) => entry.id !== candidate.id);
    this.#discovery = { ...this.#discovery, [key]: [...candidates, candidate] };
    this.#dispatch({ type: "select-game", game, id: candidate.id });
    this.#render();
  }

  async #chooseDestinationFolder(): Promise<void> {
    const evaluation = await this.backend.chooseDestinationFolder(
      this.#state.selectedBg1Id,
      this.#state.selectedBg2Id,
    );
    if (evaluation === null) return;
    this.#destination = evaluation;
    this.#dispatch({ type: "set-destination", path: evaluation.path });
    this.#render();
  }

  async #cancelBuild(): Promise<void> {
    if (this.#runId !== null) await this.backend.cancelRun(this.#runId);
  }

  #safely(operation: () => Promise<void>): void {
    this.#commandError = null;
    void operation().catch((error: unknown) => {
      this.#commandError = error instanceof BackendCommandError
        ? error
        : new BackendCommandError({
          code: "ui_operation_failed",
          message: error instanceof Error ? error.message : "The installer could not complete that action.",
          recovery_action: "Review the current screen and try again.",
          technical_detail: String(error),
        });
      this.#render();
    });
  }

  #initialNativeBuild(): BuildSnapshot {
    const phases = this.#state.evaluation?.plan.phases ?? [];
    return {
      state: "running",
      headline: "Build in progress",
      detail: "The reviewed installation is starting in a separate managed copy.",
      phases: phases.map((phase, index) => ({ ...phase, state: index === 0 ? "current" : "pending" })),
      logTail: [],
      manualArchiveName: null,
    };
  }

  #handleRunEvent(envelope: RunEventEnvelope): void {
    this.#runId = envelope.runId;
    const current = this.#state.build ?? this.#initialNativeBuild();
    let next: BuildSnapshot = current;
    const event = envelope.event;
    const log = (line: string): readonly string[] => [...current.logTail, line].slice(-200);
    switch (event.type) {
      case "campaign_started":
        this.#installId = event.install_id;
        next = { ...current, state: "running", headline: event.resumed ? "Resuming build" : "Build in progress", detail: "The engine is executing the exact frozen recipe.", logTail: log(`${envelope.sequenceAsString}: campaign ${event.install_id} started`), manualArchiveName: null };
        break;
      case "phase_started": {
        const phaseIds: Readonly<Record<string, string>> = {
          "Prepare Baldur's Gate: Enhanced Edition": "bg1-preparation",
          "Prepare Baldur's Gate II: Enhanced Edition": "bg2-preparation",
          "Build the EET campaign": "eet-initialization",
          "Install the curated collection": "main",
          "Finalize EET": "eet-finalization",
          "Apply reviewed final compatibility fixes": "post-eet-end",
        };
        const matchedIndex = current.phases.findIndex((phase) => phase.id === phaseIds[event.name]);
        const currentIndex = matchedIndex >= 0
          ? matchedIndex
          : Math.max(0, current.phases.findIndex((phase) => phase.state === "pending"));
        next = { ...current, phases: current.phases.map((phase, index) => ({ ...phase, state: index < currentIndex ? "done" : index === currentIndex ? "current" : "pending" })), logTail: log(`${envelope.sequenceAsString}: ${event.name}`) };
        break;
      }
      case "step_started":
        next = { ...current, logTail: log(`${envelope.sequenceAsString}: ${event.label}`) };
        break;
      case "step_progress":
        next = { ...current, logTail: log(`${envelope.sequenceAsString}: ${event.id} ${event.done}/${event.total}`) };
        break;
      case "console_line":
        next = { ...current, logTail: log(event.line) };
        break;
      case "attention_required":
        next = { ...current, state: "attention", headline: "Your attention is needed", detail: event.reason, logTail: log(event.last_output) };
        break;
      case "step_finished":
        next = event.outcome === "failed" && current.manualArchiveName !== null
          ? { ...current, state: "waiting-manual", headline: "Manual archive needed", logTail: log(`${event.id}: failed`) }
          : event.outcome === "failed"
          ? { ...current, state: "running", headline: "Finalizing failure evidence", detail: `The engine is closing ${event.id} safely before Retry becomes available.`, logTail: log(`${event.id}: failed`) }
          : { ...current, logTail: log(`${event.id}: ${event.outcome}`) };
        break;
      case "manual_download_needed":
        next = { ...current, state: "waiting-manual", headline: "Manual archive needed", detail: `Download ${event.mod_id} from ${event.page}. Then choose the downloaded archive; the installer will verify it before resuming.`, manualArchiveName: event.mod_id, logTail: log(`Expected SHA-256: ${event.expected_sha256}`) };
        break;
      case "error":
        if (event.step_id === null) {
          next = current.manualArchiveName !== null
            ? { ...current, state: "waiting-manual", headline: "Manual archive needed", logTail: log(event.message) }
            : { ...current, state: "failed", headline: "The build stopped safely", detail: event.message, logTail: log(event.message) };
          if (this.#status.mode === "native") void this.#refreshRetryAvailability(envelope.runId);
        } else {
          next = { ...current, state: "running", headline: "Finalizing failure evidence", detail: event.message, logTail: log(event.message) };
        }
        break;
      case "campaign_finished":
        this.#installId = event.install_id;
        next = { ...current, state: "complete", headline: "Build verified", detail: "The campaign copy and its durable receipt are complete.", phases: current.phases.map((phase) => ({ ...phase, state: "done" })), logTail: log(`${envelope.sequenceAsString}: campaign complete`) };
        break;
    }
    this.#dispatch({ type: "build-updated", build: next });
    if (event.type === "campaign_finished") this.#dispatch({ type: "navigate", route: "complete" });
    this.#render();
  }

  async #refreshRetryAvailability(runId: string): Promise<void> {
    try {
      const snapshot = await this.backend.getRunSnapshot(runId);
      this.#retryAvailable = snapshot.status === "failed" && snapshot.report !== null;
      this.#render();
    } catch (error: unknown) {
      this.#commandError = error instanceof BackendCommandError ? error : null;
      this.#render();
    }
  }

  async #exportDiagnostics(): Promise<void> {
    if (this.#installId === null) return;
    const result = await this.backend.exportDiagnostics(this.#installId);
    if (result === null) return;
    const announcement = document.createElement("p");
    announcement.className = "diagnostics-result";
    announcement.textContent = `Diagnostics exported to ${result.path}`;
    announcement.tabIndex = -1;
    this.root.querySelector("main")?.append(announcement);
    announcement.focus();
  }

  async #launchInstall(installId: string): Promise<void> {
    await this.backend.launchInstall(installId);
  }

  async #openInstallFolder(installId: string): Promise<void> {
    await this.backend.openInstallFolder(installId);
  }

  async #resumeManagedInstall(installId: string): Promise<void> {
    this.#installId = installId;
    this.#retryAvailable = false;
    this.#dispatch({ type: "build-updated", build: this.#initialNativeBuild() });
    this.#dispatch({ type: "navigate", route: "build" });
    this.#render();
    await this.#retryBuild();
  }

  #render(focusTargetId?: string): void {
    const evaluation = this.#state.evaluation;
    if (evaluation === null) return;
    const navigate = (route: Route): Promise<void> => this.navigate(route);
    const safely = (operation: () => Promise<void>): void => this.#safely(operation);
    let content: HTMLElement;
    switch (this.#state.route) {
      case "home":
        content = homeScreen(this.#installations, {
          begin: () => safely(() => navigate("welcome")),
          launch: (installId) => safely(() => this.#launchInstall(installId)),
          openFolder: (installId) => safely(() => this.#openInstallFolder(installId)),
          resume: (installId) => safely(() => this.#resumeManagedInstall(installId)),
        });
        break;
      case "updates":
        content = updatesScreen(this.#updates);
        break;
      case "welcome":
        content = welcomeScreen(() => safely(() => navigate("games")));
        break;
      case "games":
        content = gamesScreen(
          this.#discovery,
          this.#state.selectedBg1Id,
          this.#state.selectedBg2Id,
          (game, id) => this.#dispatch({ type: "select-game", game, id }),
          (game) => safely(() => this.#chooseGameFolder(game)),
          () => this.#back(),
          () => safely(() => navigate("destination")),
        );
        break;
      case "destination":
        content = destinationScreen(
          this.#destination,
          (path) => safely(() => this.#inspectDestination(path)),
          () => safely(() => this.#chooseDestinationFolder()),
          () => this.#back(),
          () => safely(() => navigate("setup")),
        );
        break;
      case "setup":
        content = setupScreen(evaluation, (id, selected) => safely(() => this.#toggleFeature(id, selected)), () => this.#back(), () => safely(() => navigate("review")));
        break;
      case "review":
        content = reviewScreen(evaluation, this.#destination, this.#selectedGames(), () => this.#back(), () => safely(() => this.#freezeReview()));
        break;
      case "build":
        content = buildScreen(this.#state.build ?? { state: "running", headline: "Build in progress", detail: "Loading fixture snapshot.", phases: evaluation.plan.phases.map((phase, index) => ({ ...phase, state: index === 0 ? "current" : "pending" })), logTail: [], manualArchiveName: null }, {
          advance: () => safely(() => this.#advanceBuild()),
          retry: () => safely(() => this.#retryBuild()),
          supplyManual: () => safely(() => this.#supplyManualArchive()),
          openManualSource: () => safely(() => this.#openManualSource()),
          cancel: () => safely(() => this.#cancelBuild()),
          diagnostics: () => safely(() => this.#exportDiagnostics()),
          diagnosticsAvailable: this.#installId !== null,
          fixture: this.#status.mode === "fixture",
          retryAvailable: this.#status.mode === "fixture" || this.#retryAvailable,
          logState: this.#logState,
          updateLogState: (state) => { this.#logState = state; },
        });
        break;
      case "complete": {
        const installId = this.#installId;
        content = completeScreen(this.#state.frozenReview, {
          home: () => safely(() => navigate("home")),
          launch: installId === null ? null : () => safely(() => this.#launchInstall(installId)),
          openFolder: installId === null ? null : () => safely(() => this.#openInstallFolder(installId)),
        });
        break;
      }
    }
    if (this.#commandError !== null) {
      const alert = statusCard(this.#commandError.message, this.#commandError.recoveryAction, "danger");
      alert.setAttribute("role", "alert");
      content.append(alert);
    }
    this.root.replaceChildren(createAppShell(
      this.#state.route,
      content,
      (route) => safely(() => navigate(route)),
      this.#status.mode,
    ));
    (focusTargetId ? this.root.querySelector<HTMLElement>(`#${focusTargetId}`) : this.root.querySelector<HTMLElement>("h1"))?.focus();
  }
}

export async function mountApp(root: HTMLElement, backend: Backend): Promise<AppHandle> {
  const controller = new AppController(root, backend);
  await controller.initialize();
  return controller;
}

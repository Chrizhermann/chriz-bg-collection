import { BackendCommandError, type Backend } from "./backend";
import { createAppShell } from "./components/app-shell";
import type { TechnicalLogState } from "./components/technical-log";
import type { BackendStatus, BuildSnapshot, DestinationEvaluation, GameCandidate, GameDiscovery, ManagedInstallation, Route, RunEventEnvelope, UpdateSummary } from "./contracts";
import { buildScreen } from "./screens/build";
import { destinationScreen } from "./screens/destination";
import { gamesScreen } from "./screens/games";
import { homeScreen, type AddonFeedback, type ShortcutFeedback } from "./screens/home";
import { reviewScreen } from "./screens/review";
import { installScreen } from "./screens/install";
import { setupScreen, type SetupViewState } from "./screens/setup";
import { updatesScreen } from "./screens/updates";
import {
  automaticInstallationPath,
  initialState,
  installationReadiness,
  reduce,
  validateInstallationName,
  type AppAction,
  type AppState,
} from "./state";
import { statusCard } from "./components/status-card";

export interface AppHandle {
  navigate(route: Route): Promise<void>;
}

const LAST_INSTALL_STORAGE_KEY = "cebg.last-install-id";
const LOG_RENDER_INTERVAL_MS = 100;

function readRememberedInstallId(): string | null {
  try {
    return window.localStorage.getItem(LAST_INSTALL_STORAGE_KEY);
  } catch {
    return null;
  }
}

function rememberInstallId(installId: string): void {
  try {
    window.localStorage.setItem(LAST_INSTALL_STORAGE_KEY, installId);
  } catch {
    // Local preferences are optional and never grant filesystem authority.
  }
}

function preferredInstallation(
  installations: readonly ManagedInstallation[],
  startupInstallId: string | null,
  rememberedId: string | null,
): ManagedInstallation | null {
  const available = installations.filter((installation) => installation.available);
  const resumable = installations.filter((installation) => installation.resumable);
  const candidates = available.length > 0 ? available : resumable.length > 0 ? resumable : installations;
  const startup = candidates.find((installation) => installation.id === startupInstallId);
  if (startup !== undefined) return startup;
  const remembered = candidates.find((installation) => installation.id === rememberedId);
  if (remembered !== undefined) return remembered;
  return candidates.reduce<ManagedInstallation | null>((newest, installation) => {
    if (newest === null) return installation;
    return (installation.completedAtMillis ?? Number.NEGATIVE_INFINITY)
      > (newest.completedAtMillis ?? Number.NEGATIVE_INFINITY)
      ? installation
      : newest;
  }, null);
}

class AppController implements AppHandle {
  #state: AppState = initialState();
  #status!: BackendStatus;
  #discovery: GameDiscovery | null = null;
  #destination: DestinationEvaluation = {
    path: "",
    safe: false,
    title: "Choose an install location",
    detail: "The installer will verify that it is separate from both clean source games.",
  };
  #defaultInstallationPath = "";
  #installations: readonly ManagedInstallation[] = [];
  #installPreparation: Promise<void> | null = null;
  #updates: UpdateSummary = {
    checkedAt: null,
    networkState: "unconfigured",
    application: { state: "unavailable", currentVersion: "0.1.0-alpha.1", availableVersion: null, detail: "Application updates have not been checked." },
    recipe: { state: "unavailable", currentVersion: "0.1.0-alpha.1", availableVersion: null, disposition: "unknown-applicability", detail: "Recipe updates have not been checked.", changes: [] },
    managedCopies: [],
  };
  #revision = 0;
  #destinationRevision = 0;
  #identityRevision = 0;
  #starting = false;
  #runId: string | null = null;
  #installId: string | null = null;
  #retryAvailable = false;
  #commandError: BackendCommandError | null = null;
  #logState: TechnicalLogState = { paused: false, open: false };
  #createDesktopShortcutAfterInstall = true;
  #shortcutFeedback: ShortcutFeedback | null = null;
  #checkingUpdates = false;
  #installingUpdate = false;
  #setupView: SetupViewState = { search: "", category: "" };
  #changingProfile = false;
  #radarAttempts = new Set<string>();
  #addonFeedback: AddonFeedback | null = null;
  #logRenderTimer: number | null = null;

  constructor(private readonly root: HTMLElement, private readonly backend: Backend) {}

  async initialize(): Promise<void> {
    const [status, installations] = await Promise.all([
      this.backend.getStatus(),
      this.backend.listManagedInstallations(),
    ]);
    this.#status = status;
    this.#updates = { ...this.#updates,
      application: { ...this.#updates.application, currentVersion: status.applicationVersion ?? status.engineVersion },
      recipe: { ...this.#updates.recipe, currentVersion: status.recipeVersion ?? "bundled" },
    };
    this.#installations = installations;
    const selected = preferredInstallation(installations, status.startupInstallId, readRememberedInstallId());
    this.#installId = selected?.id ?? null;
    if (installations.length > 0) {
      this.#dispatch({ type: "navigate", route: "home", remember: false });
    } else {
      await this.#prepareInstallFlow();
    }
    this.#render();
    if (status.mode === "native") void this.#checkUpdates(false).catch(() => {
      // Offline startup must not block setup or Play; Updates offers an explicit retry.
    });
  }

  async #prepareInstallFlow(): Promise<void> {
    if (this.#discovery !== null && this.#state.evaluation !== null) return;
    if (this.#installPreparation === null) {
      this.#installPreparation = (async () => {
        const [discovery, defaults] = await Promise.all([
          this.backend.discoverGames(),
          this.backend.getInstallationDefaults(),
        ]);
        this.#discovery = discovery;
        this.#defaultInstallationPath = defaults.path;
        this.#dispatch({ type: "select-game", game: "bg1", id: discovery.selectedBg1Id });
        this.#dispatch({ type: "select-game", game: "bg2", id: discovery.selectedBg2Id });
        this.#dispatch({ type: "set-installation-defaults", name: defaults.name, path: defaults.path });
        await Promise.all([
          this.#inspectDestination(defaults.path, true, false),
          this.#evaluate(false),
        ]);
      })();
    }
    try {
      await this.#installPreparation;
    } catch (error) {
      this.#installPreparation = null;
      throw error;
    }
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
    if (route === "welcome") await this.#prepareInstallFlow();
    if (route === "complete") route = "home";
    if (route === this.#state.route) return;
    if (route === "updates") {
      this.#dispatch({ type: "navigate", route });
      await this.#checkUpdates();
      return;
    }
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
    if (!this.#beginIdentityEdit()) return;
    this.#dispatch({ type: "set-feature", id, selected });
    await this.#evaluate(true, `feature-${id}`);
  }

  async #selectProfile(profileId: string): Promise<void> {
    if (profileId === this.#status.selectedProfile || this.#changingProfile || !this.#beginIdentityEdit()) return;
    this.#changingProfile = true;
    this.#render();
    try {
      this.#status = await this.backend.selectProfile(profileId);
      this.#dispatch({ type: "reset-selection" });
      this.#setupView = { search: "", category: "" };
      await this.#evaluate(false);
    } finally {
      this.#changingProfile = false;
      this.#render();
    }
  }

  async #backFromSecondary(): Promise<void> {
    const route = this.#state.history.at(-1) ?? "welcome";
    if (route === "welcome") await this.#prepareInstallFlow();
    if (this.#state.history.length > 0) this.#dispatch({ type: "back" });
    else this.#dispatch({ type: "navigate", route, remember: false });
    this.#render();
  }

  async #checkUpdates(render = true): Promise<void> {
    if (this.#checkingUpdates) return;
    this.#checkingUpdates = true;
    if (render) this.#render();
    try {
      this.#updates = await this.backend.getUpdates();
    } finally {
      this.#checkingUpdates = false;
      if (render) this.#render();
      this.#updateBadge();
    }
  }

  #updateBadge(): void {
    const available = this.#updates.application.state === "available" || this.#updates.recipe.state === "available"
      || this.#updates.radar?.state === "available";
    const button = this.root.querySelector<HTMLButtonElement>('[data-action="updates"]');
    button?.setAttribute("data-update-available", String(available));
    if (button) button.title = available ? "An update is available" : "Check versions and updates";
  }

  async #backToSetup(): Promise<void> {
    this.#dispatch({ type: "build-cleared" });
    this.#dispatch({ type: "review-cleared" });
    this.#runId = null;
    this.#installId = null;
    this.#commandError = null;
    await this.#prepareInstallFlow();
    await this.#inspectDestination(this.#state.destinationPath);
    await this.navigate("welcome");
  }

  async #inspectDestination(path: string, automatic = this.#state.destinationAutomatic, render = true): Promise<void> {
    const revision = ++this.#destinationRevision;
    this.#dispatch({ type: "set-destination", path, automatic });
    if (this.#selectedGames().length !== 2) {
      this.#destination = {
        path,
        safe: false,
        title: "Choose both game sources",
        detail: "Use Change source for each missing game before CEBG checks this install location.",
      };
      if (render) this.#render();
      return;
    }
    this.#destination = {
      path,
      safe: false,
      title: "Checking this location",
      detail: "The folder has not completed native safety inspection yet.",
    };
    if (render) this.#render();
    let inspected: DestinationEvaluation;
    try {
      inspected = await this.backend.inspectDestination(
        path,
        this.#state.selectedBg1Id,
        this.#state.selectedBg2Id,
      );
    } catch (error: unknown) {
      if (revision !== this.#destinationRevision) return;
      this.#destination = error instanceof BackendCommandError
        ? { path, safe: false, title: error.message, detail: error.recoveryAction }
        : { path, safe: false, title: "This install location could not be checked", detail: "Choose another folder or try this location again." };
      this.#dispatch({ type: "set-destination", path, automatic });
      if (render) this.#render();
      return;
    }
    if (revision !== this.#destinationRevision) return;
    this.#destination = inspected;
    this.#dispatch({ type: "set-destination", path: inspected.path, automatic });
    if (render) this.#render();
  }

  #selectedGames(): readonly GameCandidate[] {
    const bg1 = this.#discovery?.bg1Candidates.find((candidate) => candidate.id === this.#state.selectedBg1Id);
    const bg2 = this.#discovery?.bg2Candidates.find((candidate) => candidate.id === this.#state.selectedBg2Id);
    return [bg1, bg2].filter((candidate): candidate is GameCandidate => candidate !== undefined);
  }

  async #freezeReview(render = true, identityRevision = this.#identityRevision): Promise<void> {
    try {
      const displayedEvaluation = this.#state.evaluation;
      const review = await this.backend.freezeReview(
        this.#state.installationName,
        this.#state.selection,
        this.#destination.path,
        this.#state.selectedBg1Id,
        this.#state.selectedBg2Id,
      );
      if (identityRevision !== this.#identityRevision) {
        throw new BackendCommandError({
          code: "installation_identity_changed",
          message: "Installation details changed while starting.",
          recovery_action: "Review the current installation details, then choose Install Chriz Easy BG again.",
          technical_detail: "An installation identity control changed after freezeReview began and before startBuild.",
        });
      }
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
      this.#retryAvailable = false;
      this.#dispatch({ type: "build-updated", build: this.#initialNativeBuild() });
      const started = await this.backend.startBuild(review.reviewToken, (event) => this.#handleRunEvent(event));
      this.#runId = started.runId;
      if (this.#status.mode === "fixture" && this.#installId === null) this.#installId = "fixture-install";
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
        this.#dispatch({ type: "build-updated", build: { ...build, state: "running", headline: "Installation in progress", detail: "Continuing your installation…" } });
      }
      this.#render();
      return;
    }
    const build = await this.backend.advanceBuild();
    this.#dispatch({ type: "build-updated", build });
    if (build.state === "complete") {
      await this.#finishInstallation(this.#installId ?? "fixture-install");
      return;
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
    if (this.#discovery === null) return;
    if (!this.#beginIdentityEdit()) return;
    const role = game === "bg1" ? "bgee_sod" : "bg2ee";
    const candidate = await this.backend.chooseGameFolder(role);
    if (candidate === null) return;
    if (this.#starting) {
      ++this.#identityRevision;
      return;
    }
    const key = game === "bg1" ? "bg1Candidates" : "bg2Candidates";
    const candidates = this.#discovery[key].filter((entry) => entry.id !== candidate.id);
    this.#discovery = { ...this.#discovery, [key]: [...candidates, candidate] };
    this.#dispatch({ type: "select-game", game, id: candidate.id });
    this.#dispatch({ type: "review-cleared" });
    await this.#inspectDestination(this.#state.destinationPath);
  }

  async #selectGame(game: "bg1" | "bg2", id: string): Promise<void> {
    if (!this.#beginIdentityEdit()) return;
    this.#dispatch({ type: "select-game", game, id });
    this.#dispatch({ type: "review-cleared" });
    await this.#inspectDestination(this.#state.destinationPath);
  }

  async #changeInstallationName(name: string): Promise<void> {
    if (!this.#beginIdentityEdit()) return;
    this.#dispatch({ type: "set-installation-name", name });
    this.#dispatch({ type: "review-cleared" });
    if (this.#state.destinationAutomatic && validateInstallationName(name) === null) {
      await this.#inspectDestination(automaticInstallationPath(this.#defaultInstallationPath, name), true);
      return;
    }
    this.#render();
  }

  async #changeInstallLocation(path: string): Promise<void> {
    if (!this.#beginIdentityEdit()) return;
    this.#dispatch({ type: "review-cleared" });
    await this.#inspectDestination(path, false);
  }

  async #chooseDestinationFolder(): Promise<void> {
    if (!this.#beginIdentityEdit()) return;
    const evaluation = await this.backend.chooseDestinationFolder(
      this.#state.selectedBg1Id,
      this.#state.selectedBg2Id,
    );
    if (evaluation === null) return;
    if (this.#starting) {
      ++this.#identityRevision;
      return;
    }
    ++this.#destinationRevision;
    this.#destination = evaluation;
    this.#dispatch({ type: "set-destination", path: evaluation.path, automatic: false });
    this.#dispatch({ type: "review-cleared" });
    this.#render();
  }

  async #customizeInstallation(): Promise<void> {
    if (!this.#beginIdentityEdit()) return;
    await this.navigate("setup");
  }

  #beginIdentityEdit(): boolean {
    ++this.#identityRevision;
    return !this.#starting;
  }

  #canInstall(): boolean {
    if (this.#changingProfile) return false;
    const bg1 = this.#discovery?.bg1Candidates.find((candidate) => candidate.id === this.#state.selectedBg1Id);
    const bg2 = this.#discovery?.bg2Candidates.find((candidate) => candidate.id === this.#state.selectedBg2Id);
    return installationReadiness({
      starting: this.#starting,
      name: this.#state.installationName,
      bg1,
      bg2,
      destination: this.#destination,
      evaluation: this.#state.evaluation,
      evaluationPending: this.#state.evaluationPending,
    });
  }

  async #startInstallation(): Promise<void> {
    if (!this.#canInstall()) return;
    const identityRevision = this.#identityRevision;
    this.#starting = true;
    this.#render();
    try {
      await this.#freezeReview(true, identityRevision);
    } finally {
      this.#starting = false;
      if (this.#state.route === "welcome") this.#render();
    }
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
      headline: "Installation in progress",
      detail: "Preparing your game folder…",
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
        next = { ...current, state: "running", headline: event.resumed ? "Resuming installation" : "Installation in progress", detail: "Setting up your selected mods…", logTail: log(`${envelope.sequenceAsString}: installation ${event.install_id} started`), manualArchiveName: null };
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
        next = { ...current, detail: event.label, logTail: log(`${envelope.sequenceAsString}: ${event.label}`) };
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
          ? { ...current, state: "running", headline: "Stopping installation", detail: "Saving progress before retry becomes available…", logTail: log(`${event.id}: failed`) }
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
          next = { ...current, state: "running", headline: "Stopping installation", detail: event.message, logTail: log(event.message) };
        }
        break;
      case "campaign_finished":
        this.#installId = event.install_id;
        next = { ...current, state: "complete", headline: "Installation verified", detail: "The game folder and its installation record are complete.", phases: current.phases.map((phase) => ({ ...phase, state: "done" })), logTail: log(`${envelope.sequenceAsString}: installation complete`) };
        break;
    }
    this.#dispatch({ type: "build-updated", build: next });
    if (event.type === "campaign_finished") {
      this.#render();
      void this.#finishInstallation(event.install_id);
      return;
    }
    if (event.type === "step_progress" || event.type === "console_line") {
      this.#scheduleLogRender();
      return;
    }
    this.#render();
  }

  #scheduleLogRender(): void {
    if (!this.#logState.open || this.#logRenderTimer !== null) return;
    this.#logRenderTimer = window.setTimeout(() => {
      this.#logRenderTimer = null;
      this.#render();
    }, LOG_RENDER_INTERVAL_MS);
  }

  async #finishInstallation(installId: string): Promise<void> {
    try {
      const installations = await this.backend.listManagedInstallations();
      this.#installations = installations;
      const completed = installations.find((installation) => installation.id === installId);
      const selected = completed ?? preferredInstallation(installations, null, readRememberedInstallId());
      this.#installId = selected?.id ?? null;
      if (selected !== null) rememberInstallId(selected.id);
      this.#dispatch({ type: "navigate", route: "home", remember: false });
      if (completed?.available === true) await this.#installCompletionRadar(completed);
      if (completed?.available === true && this.#createDesktopShortcutAfterInstall) {
        this.#render();
        await this.#createDesktopShortcut(completed.id);
        return;
      }
    } catch (error: unknown) {
      this.#commandError = error instanceof BackendCommandError
        ? error
        : new BackendCommandError({
          code: "installation_registry_refresh_failed",
          message: "The installation finished, but its launcher record could not be refreshed.",
          recovery_action: "Open My installs again after restarting CEBG.",
          technical_detail: String(error),
        });
    }
    this.#render();
  }

  async #installCompletionRadar(installation: ManagedInstallation): Promise<void> {
    if (installation.radarVersion || this.#radarAttempts.has(installation.id)) return;
    this.#radarAttempts.add(installation.id);
    this.#addonFeedback = { installId: installation.id, state: "installing" };
    this.#render();
    try {
      await this.backend.installRadar(installation.id);
      this.#installations = await this.backend.listManagedInstallations();
      this.#addonFeedback = null;
    } catch {
      this.#addonFeedback = { installId: installation.id, state: "failed" };
    }
    this.#render();
  }

  async #refreshRetryAvailability(runId: string): Promise<void> {
    try {
      const snapshot = await this.backend.getRunSnapshot(runId);
      if (runId !== this.#runId) return;
      const freshCopyRequired = snapshot.status === "failed"
        && snapshot.report?.status.status === "fresh_copy_required";
      this.#retryAvailable = snapshot.status === "failed"
        && snapshot.report !== null
        && !freshCopyRequired;
      if (this.#state.build !== null && (freshCopyRequired || snapshot.error !== null)) {
        const current = this.#state.build;
        const diagnostic = snapshot.error === null
          ? null
          : `${snapshot.error.code}: ${snapshot.error.technical_detail}`;
        this.#dispatch({ type: "build-updated", build: {
          ...current,
          ...(freshCopyRequired ? {
            state: "failed" as const,
            headline: "A new installation is needed",
            detail: "This copy cannot be resumed safely. Its failure evidence has been preserved.",
            recoveryAction: "Return to setup and choose a new empty folder for a fresh installation.",
            manualArchiveName: null,
            freshCopyRequired: true,
          } : current.state === "waiting-manual" || snapshot.error === null ? {} : {
            detail: snapshot.error.message,
            recoveryAction: snapshot.error.recovery_action,
          }),
          logTail: diagnostic === null || current.logTail.includes(diagnostic)
            ? current.logTail
            : [...current.logTail, diagnostic].slice(-200),
        } });
      }
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

  async #createDesktopShortcut(installId: string): Promise<void> {
    if (!this.#installations.some((installation) => installation.id === installId && installation.available)) return;
    try {
      const result = await this.backend.createDesktopShortcut(installId);
      this.#shortcutFeedback = { installId, state: "created", path: result.path };
    } catch {
      this.#shortcutFeedback = { installId, state: "failed" };
    }
    this.#render();
  }

  async #installAppUpdate(version: string): Promise<void> {
    if (this.#installingUpdate) return;
    this.#installingUpdate = true;
    this.#render();
    try {
      await this.backend.installAppUpdate(version);
    } finally {
      this.#installingUpdate = false;
      this.#render();
    }
  }

  async #installRadar(): Promise<void> {
    const installation = this.#installations.find((item) => item.id === this.#installId && item.available);
    if (installation === undefined || this.#installingUpdate) return;
    this.#installingUpdate = true;
    this.#render();
    try {
      await this.backend.installRadar(installation.id);
      this.#installations = await this.backend.listManagedInstallations();
      if (this.#addonFeedback?.installId === installation.id) this.#addonFeedback = null;
      await this.#checkUpdates();
    } finally {
      this.#installingUpdate = false;
      this.#render();
    }
  }

  async #buildUpdatedCopy(version: string): Promise<void> {
    await this.backend.activateRecipeUpdate(version);
    await this.#beginNewInstallation();
  }

  async #beginNewInstallation(): Promise<void> {
    this.#dispatch({ type: "review-cleared" });
    this.#dispatch({ type: "build-cleared" });
    this.#createDesktopShortcutAfterInstall = true;
    this.#shortcutFeedback = null;
    this.#addonFeedback = null;
    await this.#prepareInstallFlow();
    this.#dispatch({ type: "navigate", route: "welcome" });
    this.#render();
  }

  #selectManagedInstallation(installId: string): void {
    if (!this.#installations.some((installation) => installation.id === installId)) return;
    this.#installId = installId;
    rememberInstallId(installId);
    this.#render();
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
    if (this.#logRenderTimer !== null) {
      window.clearTimeout(this.#logRenderTimer);
      this.#logRenderTimer = null;
    }
    const previousRoute = this.root.querySelector<HTMLElement>(".app-shell")?.dataset.route;
    const sameRoute = previousRoute === this.#state.route;
    const scrollTop = sameRoute ? this.root.querySelector("main")?.scrollTop ?? 0 : 0;
    const evaluation = this.#state.evaluation;
    const navigate = (route: Route): Promise<void> => this.navigate(route);
    const safely = (operation: () => Promise<void>): void => this.#safely(operation);
    const renderHome = (): HTMLElement => homeScreen(
      this.#installations,
      this.#installations.find((installation) => installation.id === this.#installId) ?? null,
      {
        begin: () => safely(() => this.#beginNewInstallation()),
        launch: (installId) => safely(() => this.#launchInstall(installId)),
        openFolder: (installId) => safely(() => this.#openInstallFolder(installId)),
        resume: (installId) => safely(() => this.#resumeManagedInstall(installId)),
        select: (installId) => this.#selectManagedInstallation(installId),
        createShortcut: (installId) => this.#createDesktopShortcut(installId),
      },
      this.#shortcutFeedback,
      this.#addonFeedback,
    );
    let content: HTMLElement;
    switch (this.#state.route) {
      case "home":
        content = renderHome();
        break;
      case "updates":
        content = updatesScreen(this.#updates, {
          checkAgain: () => safely(() => this.#checkUpdates()),
          installApplication: (version) => safely(() => this.#installAppUpdate(version)),
          buildUpdatedCopy: (version) => safely(() => this.#buildUpdatedCopy(version)),
          installRadar: this.#installations.some((item) => item.id === this.#installId && item.available) ? () => safely(() => this.#installRadar()) : undefined,
          installationName: this.#installations.find((item) => item.id === this.#installId && item.available)?.name,
          radarVersion: this.#installations.find((item) => item.id === this.#installId && item.available)?.radarVersion,
        }, this.#checkingUpdates, this.#installingUpdate);
        break;
      case "welcome":
        if (this.#discovery === null || evaluation === null) {
          content = statusCard("Preparing your installation", "CEBG is finding your games and recommended setup.", "ok");
          break;
        }
        content = installScreen({
          discovery: this.#discovery,
          selectedBg1Id: this.#state.selectedBg1Id,
          selectedBg2Id: this.#state.selectedBg2Id,
          installationName: this.#state.installationName,
          destination: this.#destination,
          evaluation,
          evaluationPending: this.#state.evaluationPending || this.#changingProfile,
          starting: this.#starting,
          createDesktopShortcut: this.#createDesktopShortcutAfterInstall,
          profiles: this.#status.profiles,
          selectedProfile: this.#status.selectedProfile,
          changingProfile: this.#changingProfile,
        }, {
          selectSource: (game, id) => safely(() => this.#selectGame(game, id)),
          browseSource: (game) => safely(() => this.#chooseGameFolder(game)),
          changeName: (name) => safely(() => this.#changeInstallationName(name)),
          changeLocation: (path) => safely(() => this.#changeInstallLocation(path)),
          browseLocation: () => safely(() => this.#chooseDestinationFolder()),
          customize: () => safely(() => this.#customizeInstallation()),
          selectProfile: (profileId) => safely(() => this.#selectProfile(profileId)),
          install: () => safely(() => this.#startInstallation()),
          changeDesktopShortcut: (selected) => {
            if (!this.#starting) this.#createDesktopShortcutAfterInstall = selected;
          },
        });
        break;
      case "games":
        if (this.#discovery === null) {
          content = statusCard("Preparing game sources", "CEBG is finding your original games.", "ok");
          break;
        }
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
        if (evaluation === null) {
          content = statusCard("Preparing setup", "CEBG is loading the recommended choices.", "ok");
          break;
        }
        content = setupScreen(evaluation, (id, selected) => safely(() => this.#toggleFeature(id, selected)), () => safely(() => navigate("welcome")), () => safely(() => navigate("welcome")), this.#setupView);
        break;
      case "review":
        if (evaluation === null) {
          content = statusCard("Preparing review", "CEBG is checking the selected setup.", "ok");
          break;
        }
        content = reviewScreen(evaluation, this.#destination, this.#selectedGames(), () => this.#back(), () => safely(() => this.#freezeReview()));
        break;
      case "build":
        content = buildScreen(this.#state.build ?? { state: "running", headline: "Build in progress", detail: "Loading fixture snapshot.", phases: (evaluation?.plan.phases ?? []).map((phase, index) => ({ ...phase, state: index === 0 ? "current" : "pending" })), logTail: [], manualArchiveName: null }, {
          backToSetup: () => safely(() => this.#backToSetup()),
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
        content = renderHome();
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
      ["home", "updates"].includes(this.#state.route) ? () => safely(() => this.#backFromSecondary()) : undefined,
      this.#status.applicationVersion ?? this.#status.engineVersion,
    ));
    this.#updateBadge();
    const main = this.root.querySelector("main");
    if (main !== null) main.scrollTop = scrollTop;
    if (focusTargetId || !sameRoute) {
      (focusTargetId ? this.root.querySelector<HTMLElement>(`#${focusTargetId}`) : this.root.querySelector<HTMLElement>("h1"))?.focus({ preventScroll: true });
    }
  }
}

export async function mountApp(root: HTMLElement, backend: Backend): Promise<AppHandle> {
  const controller = new AppController(root, backend);
  await controller.initialize();
  return controller;
}

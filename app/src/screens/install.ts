import type {
  BackendStatus,
  DestinationEvaluation,
  FeatureGroup,
  GameCandidate,
  GameDiscovery,
  ManualDownloadRequirement,
  ManualBreakpoint,
  SelectionEvaluation,
} from "../contracts";
import { actionButton, element, screenIntro } from "../components/app-shell";
import { installationReadiness, sourceStatus, sourceVersionNote } from "../state";
import { invalidBreakpoints } from "../components/manual-checkpoints";
import { mainChoicesCard } from "../components/main-choices";
import { flavor, sourceSlots, type SourceDefinition } from "../flavor";

export interface InstallScreenModel {
  readonly discovery: GameDiscovery;
  readonly selectedBg1Id: string;
  readonly selectedBg2Id: string;
  readonly installationName: string;
  /** "Will be saved as “…”." when the name is saved in another form; never blocks. */
  readonly installationNameNote?: string | null;
  readonly destination: DestinationEvaluation;
  readonly evaluation: SelectionEvaluation | null;
  readonly evaluationPending: boolean;
  readonly starting: boolean;
  readonly createDesktopShortcut: boolean;
  readonly profiles?: BackendStatus["profiles"];
  readonly selectedProfile?: string;
  readonly changingProfile?: boolean;
  readonly manualDownloads: readonly ManualDownloadRequirement[];
  readonly manualDownloadGateVisible: boolean;
  readonly manualDownloadCheckingArtifactId: string | null;
  readonly manualDownloadError: { readonly artifactId: string; readonly message: string } | null;
  readonly manualBreakpoints?: readonly ManualBreakpoint[];
}

export interface InstallScreenActions {
  readonly selectSource: (game: "bg1" | "bg2", id: string) => void | Promise<void>;
  readonly browseSource: (game: "bg1" | "bg2") => void | Promise<void>;
  readonly changeName: (name: string, liveTyping?: boolean) => void | Promise<void>;
  readonly changeLocation: (path: string, liveTyping?: boolean) => void | Promise<void>;
  readonly browseLocation: () => void | Promise<void>;
  readonly customize: () => void | Promise<void>;
  /** Switches a main choice: all member changes of one feature group as one batch. */
  readonly changeFeatures?: (changes: Record<string, boolean>, focusId: string) => void | Promise<void>;
  /** Opens Customize at the feature group's card. */
  readonly showGroupDetails?: (group: FeatureGroup) => void | Promise<void>;
  readonly install: () => void | Promise<void>;
  readonly changeDesktopShortcut: (selected: boolean) => void | Promise<void>;
  readonly selectProfile?: (profileId: string) => void | Promise<void>;
  readonly openManualSource: (artifactId: string) => void | Promise<void>;
  readonly chooseManualArchive: (artifactId: string) => void | Promise<void>;
  readonly skipManualDownload: (requirement: ManualDownloadRequirement) => void | Promise<void>;
}

function formatFileSize(length: number): string {
  if (length < 1_000_000) return `${Math.max(1, Math.round(length / 1_000))} KB`;
  return `${(length / 1_000_000).toFixed(1)} MB`;
}

function manualDownloadPanel(model: InstallScreenModel, actions: InstallScreenActions): HTMLElement {
  const panel = element("section", "manual-download-panel");
  panel.setAttribute("aria-labelledby", "manual-download-title");
  const missingCount = model.manualDownloads.filter((requirement) => !requirement.ready).length;
  panel.append(element("p", "manual-download-kicker", missingCount === 0
    ? "Official file verified"
    : missingCount === 1 ? "One official file needed" : `${missingCount} official files needed`));
  const heading = element("h2", undefined, "Finish the download before installation");
  heading.id = "manual-download-title";
  panel.append(
    heading,
    element("p", "manual-download-intro", `${flavor.shortName} cannot redistribute this mod. Download its file from the mod's page, then choose that file here. A newer version than the one named below works too.`),
  );

  for (const requirement of model.manualDownloads) {
    const checking = model.manualDownloadCheckingArtifactId === requirement.artifactId;
    const row = element("article", `manual-download-row ${requirement.ready ? "is-ready" : "needs-file"}`);
    row.dataset.artifactId = requirement.artifactId;
    const copy = element("div", "manual-download-copy");
    copy.append(
      element("h3", undefined, requirement.ready ? `${requirement.title} is ready` : requirement.title),
      element("p", undefined, requirement.detail ?? (requirement.ready
        ? "The official file passed its identity check."
        : "Choose the file you downloaded. The name below is the version the collection was tested with.")),
      element("p", "manual-download-file", `${requirement.filename} (${formatFileSize(requirement.length)})`),
    );
    const status = element("p", `manual-download-status ${requirement.ready ? "is-ready" : "needs-file"}`, requirement.ready ? "Verified and ready" : "File required");
    status.setAttribute("role", "status");
    row.append(copy, status);

    if (!requirement.ready) {
      if (checking) {
        const checkingStatus = element("p", "manual-download-checking", `Checking ${requirement.filename}…`);
        checkingStatus.setAttribute("role", "status");
        row.append(checkingStatus);
      }
      if (model.manualDownloadError?.artifactId === requirement.artifactId) {
        const error = element("p", "manual-download-error", model.manualDownloadError.message);
        error.setAttribute("role", "alert");
        row.append(error);
      }
      const controls = element("div", "manual-download-actions");
      const download = actionButton(`Download ${requirement.title}`, () => actions.openManualSource(requirement.artifactId), "quiet");
      const choose = actionButton("Choose downloaded file", () => actions.chooseManualArchive(requirement.artifactId));
      const skip = actionButton(`Skip ${requirement.title}`, () => actions.skipManualDownload(requirement), "quiet");
      download.disabled = model.starting || checking;
      choose.disabled = model.starting || checking;
      skip.disabled = model.starting || checking;
      controls.append(download, choose, skip);
      row.append(controls);
    }
    panel.append(row);
  }
  return panel;
}

function selectedCandidate(candidates: readonly GameCandidate[], selectedId: string): GameCandidate | undefined {
  return candidates.find((candidate) => candidate.id === selectedId);
}

function sourceCard(
  source: SourceDefinition,
  candidates: readonly GameCandidate[],
  selectedId: string,
  starting: boolean,
  actions: InstallScreenActions,
): HTMLElement {
  const game = source.slot;
  const candidate = selectedCandidate(candidates, selectedId);
  const usable = sourceStatus(candidate);
  const compact = candidate !== undefined && candidates.length === 1 && usable === "ready";
  const card = element("article", `source-card card${compact ? " is-compact" : ""}`);
  card.dataset.source = game;
  const copy = element("div", "source-card-heading");
  copy.append(element("p", "source-title", source.label));
  copy.append(element("h2", undefined, candidate === undefined ? "Source needed" : candidates.length === 1 ? "Found source" : "Choose source"));
  const status = element(
    "span",
    `source-status ${usable === "ready" ? "ok" : "warning"}`,
    usable === "ready" ? "Ready" : usable === "less-tested" ? "Less tested" : "Needs attention",
  );
  card.append(copy, status);

  if (candidates.length > 1 || (candidate === undefined && candidates.length > 0)) {
    const label = element("label", "visually-hidden", `${source.name} source`);
    const select = element("select");
    select.id = `source-${game}`;
    label.htmlFor = select.id;
    if (candidate === undefined) {
      const placeholder = element("option", undefined, "Choose source");
      placeholder.value = "";
      placeholder.selected = true;
      select.append(placeholder);
    }
    for (const optionCandidate of candidates) {
      const option = element("option", undefined, optionCandidate.label);
      option.value = optionCandidate.id;
      option.selected = optionCandidate.id === candidate?.id;
      select.append(option);
    }
    select.disabled = starting;
    select.addEventListener("change", () => void actions.selectSource(game, select.value));
    card.append(label, select);
  } else {
    card.append(element("p", "source-label", candidate?.label ?? "No supported source found"));
  }

  card.append(element("p", "path", candidate?.path ?? "No installation detected"));
  if (usable === "less-tested") {
    card.append(element("p", "source-note", "You can install from this copy. It differs from the tested setup; Details says how."));
  }
  const details = element("details", "source-details");
  details.append(element("summary", undefined, "Details"));
  if (candidate === undefined) {
    details.append(element("p", "finding warning", `Choose the game folder so ${flavor.shortName} can inspect it.`));
  } else if (candidate.findings.length === 0) {
    details.append(element("p", "finding", "No additional findings."));
  } else {
    for (const finding of candidate.findings) details.append(element("p", `finding ${usable === "ready" ? "ok" : "warning"}`, finding));
  }
  const changeSource = actionButton("Change source", () => actions.browseSource(game), "quiet compact");
  changeSource.setAttribute("aria-label", `Change ${source.name} source`);
  changeSource.disabled = starting;
  card.append(details, changeSource);
  return card;
}

function field(labelText: string, id: string, value: string): { wrapper: HTMLElement; input: HTMLInputElement } {
  const wrapper = element("div", "install-field");
  const label = element("label", undefined, labelText);
  label.htmlFor = id;
  const input = element("input");
  input.id = id;
  input.type = "text";
  input.value = value;
  wrapper.append(label, input);
  return { wrapper, input };
}

export function installScreen(model: InstallScreenModel, actions: InstallScreenActions): HTMLElement {
  const page = element("div", "screen-stack install-screen");
  page.append(screenIntro("", `Install ${flavor.productName}`, flavor.text.installIntro));

  const sources = element("section", "source-grid install-sources");
  sources.setAttribute("aria-label", flavor.sources.length === 1 ? "Found game source" : "Found game sources");
  sources.append(...flavor.sources.map(source => source.slot === "bg1"
    ? sourceCard(source, model.discovery.bg1Candidates, model.selectedBg1Id, model.starting, actions)
    : sourceCard(source, model.discovery.bg2Candidates, model.selectedBg2Id, model.starting, actions)));
  const versionNote = sourceVersionNote(
    selectedCandidate(model.discovery.bg1Candidates, model.selectedBg1Id),
    selectedCandidate(model.discovery.bg2Candidates, model.selectedBg2Id),
  );
  if (versionNote !== null) {
    const note = element("p", "source-note source-version-note", versionNote);
    note.setAttribute("role", "status");
    sources.append(note);
  }

  const settings = element("section", "install-settings card");
  settings.append(element("h2", undefined, "Your installation"));
  const fields = element("div", "install-field-grid");
  const name = field("Install name", "install-name", model.installationName);
  const location = field("Install location", "install-location", model.destination.path);
  name.input.disabled = model.starting;
  location.input.disabled = model.starting;
  const nameNote = model.installationNameNote ?? null;
  const nameHelp = element("p", "field-help", nameNote ?? `This name also labels your installation in ${flavor.shortName}.`);
  nameHelp.id = "install-name-help";
  nameHelp.setAttribute("aria-live", "polite");
  name.input.setAttribute("aria-describedby", nameHelp.id);
  name.wrapper.append(nameHelp);
  let nameTimer: number | null = null;
  name.input.addEventListener("input", () => {
    if (nameTimer !== null) window.clearTimeout(nameTimer);
    nameTimer = window.setTimeout(() => {
      nameTimer = null;
      if (name.input.isConnected) void actions.changeName(name.input.value, true);
    }, 250);
  });
  name.input.addEventListener("change", () => {
    if (nameTimer !== null) {
      window.clearTimeout(nameTimer);
      nameTimer = null;
    }
    void actions.changeName(name.input.value, false);
  });
  const locationControls = element("div", "location-controls");
  const changeLocation = actionButton("Browse…", actions.browseLocation, "quiet");
  changeLocation.setAttribute("aria-label", "Change install location");
  changeLocation.disabled = model.starting;
  location.input.remove();
  locationControls.append(location.input, changeLocation);
  location.wrapper.append(locationControls);
  const locationHelp = element("p", "field-help", `Enter a new folder path or browse. ${flavor.shortName} creates missing folders when you install.`);
  locationHelp.id = "install-location-help";
  location.input.setAttribute("aria-describedby", locationHelp.id);
  location.wrapper.append(locationHelp);
  if (model.destination.safe && model.destination.note) {
    // A usable folder CEBG still has something to say about: it installs into a new folder
    // inside it, or Steam or Windows may change files there.
    const locationNote = element("p", "field-help", model.destination.note);
    locationNote.id = "install-location-note";
    locationNote.setAttribute("role", "status");
    location.input.setAttribute("aria-describedby", `${locationHelp.id} ${locationNote.id}`);
    location.wrapper.append(locationNote);
  }
  if (!model.destination.safe) {
    const locationFinding = element("div", "location-finding");
    locationFinding.id = "install-location-finding";
    locationFinding.setAttribute("role", "status");
    locationFinding.append(
      element("strong", undefined, model.destination.title),
      element("p", undefined, model.destination.detail),
    );
    location.input.setAttribute("aria-describedby", `${locationHelp.id} ${locationFinding.id}`);
    location.wrapper.append(locationFinding);
  }
  let locationTimer: number | null = null;
  location.input.addEventListener("input", () => {
    if (locationTimer !== null) window.clearTimeout(locationTimer);
    locationTimer = window.setTimeout(() => {
      locationTimer = null;
      if (location.input.isConnected) void actions.changeLocation(location.input.value, true);
    }, 250);
  });
  location.input.addEventListener("change", () => {
    if (locationTimer !== null) {
      window.clearTimeout(locationTimer);
      locationTimer = null;
    }
    void actions.changeLocation(location.input.value, false);
  });
  fields.append(name.wrapper, location.wrapper);
  settings.append(fields, element("p", "safety-note", flavor.text.installSafetyNote));

  const recipe = element("section", "recommended-setup card");
  const recipeCopy = element("div");
  const recipeSummary = element("p", "choice-summary", model.evaluation === null ? "Checking the recommended choices…" : `${model.evaluation.selectedChoiceCount} choices included`);
  recipeCopy.append(element("h2", undefined, "Recommended setup"), recipeSummary);
  const customize = actionButton("Customize", actions.customize, "quiet");
  customize.disabled = model.starting;
  recipe.append(recipeCopy);
  if ((model.profiles?.length ?? 0) > 1 && actions.selectProfile !== undefined) {
    recipe.classList.add("has-profiles");
    const label = element("label", "visually-hidden", "Mod setup");
    label.htmlFor = "install-profile";
    const select = element("select", "profile-select");
    select.id = label.htmlFor;
    select.disabled = model.starting || model.changingProfile === true;
    for (const profile of model.profiles ?? []) {
      const option = element("option", undefined, profile.label);
      option.value = profile.id;
      option.selected = profile.id === model.selectedProfile;
      option.title = profile.description;
      select.append(option);
    }
    select.addEventListener("change", () => void actions.selectProfile?.(select.value));
    recipe.append(label, select);
  }
  recipe.append(customize);
  if (model.manualBreakpoints?.length) {
    const invalid = invalidBreakpoints(model.manualBreakpoints, model.evaluation?.manualCheckpointOptions ?? []);
    const note = element("p", invalid.length ? "unavailable-reason" : "muted", invalid.length
      ? "A manual-mod pause no longer matches this setup. Open Customize and correct it before installing."
      : `${model.manualBreakpoints.length} manual-mod ${model.manualBreakpoints.length === 1 ? "pause" : "pauses"} selected. Installation will stop at these points for your additions.`);
    if (invalid.length) note.setAttribute("role", "alert");
    recipe.append(note);
  }
  if (model.evaluation !== null && model.evaluation.findings.length > 0) {
    const notices = element("details", "recipe-notices");
    notices.append(element("summary", undefined, `${model.evaluation.findings.length} setup ${model.evaluation.findings.length === 1 ? "note" : "notes"}`));
    const list = element("ul");
    for (const finding of model.evaluation.findings) list.append(element("li", undefined, finding.message));
    notices.append(list);
    recipe.append(notices);
  }

  const bg1 = selectedCandidate(model.discovery.bg1Candidates, model.selectedBg1Id);
  const bg2 = selectedCandidate(model.discovery.bg2Candidates, model.selectedBg2Id);
  const selectionReady = installationReadiness({
    starting: model.starting,
    bg1,
    bg2,
    destination: model.destination,
    evaluation: model.evaluation,
    evaluationPending: model.evaluationPending,
    manualBreakpoints: model.manualBreakpoints,
    requiredSlots: sourceSlots,
  });
  const manualDownloadPending = model.manualDownloadGateVisible
    && (model.manualDownloadCheckingArtifactId !== null || model.manualDownloads.some((requirement) => !requirement.ready));
  const ready = selectionReady && !manualDownloadPending;
  const finish = element("section", `install-ready ${ready ? "is-ready" : "needs-attention"}`);
  const readiness = element("div");
  const readinessTitle = element("h2", undefined, model.starting ? "Starting installation" : model.evaluationPending ? "Checking your choices" : ready ? "Ready to install" : "Needs attention");
  const readinessDetail = element("p", undefined, model.starting ? "Preparing your selected setup…" : model.evaluationPending ? "This will only take a moment." : ready ? "Everything required for this installation is ready." : "Resolve the highlighted item before installing.");
  readiness.append(readinessTitle, readinessDetail);
  const install = actionButton(model.starting ? "Starting installation…" : `Install ${flavor.productName}`, actions.install);
  install.disabled = !ready;
  const finishActions = element("div", "install-finish-actions");
  const shortcutChoice = element("label", "shortcut-choice");
  const shortcut = element("input");
  shortcut.type = "checkbox";
  shortcut.checked = model.createDesktopShortcut;
  shortcut.disabled = model.starting;
  shortcut.addEventListener("change", () => void actions.changeDesktopShortcut(shortcut.checked));
  shortcutChoice.append(shortcut, element("span", undefined, "Create desktop shortcut when finished"));
  finishActions.append(shortcutChoice, install);
  finish.append(readiness, finishActions);

  page.append(sources, settings);
  // Main choices (Challenge Mode) sit above the setup they change. A recipe without feature
  // groups, like Chriz Easy IWD's, shows no such card.
  const groups = model.evaluation?.featureGroups ?? [];
  const { changeFeatures, showGroupDetails } = actions;
  if (groups.length > 0 && changeFeatures !== undefined && showGroupDetails !== undefined) {
    page.append(mainChoicesCard(groups, {
      change: changeFeatures,
      details: showGroupDetails,
      // The engine checks the changed choices before this screen is drawn again. Until then
      // the old result must not look installable.
      requested: () => {
        install.disabled = true;
        finish.className = "install-ready needs-attention";
        readinessTitle.textContent = "Checking your choices";
        readinessDetail.textContent = "This will only take a moment.";
        recipeSummary.textContent = "Checking your choices…";
      },
    }, model.starting || model.changingProfile === true));
  }
  page.append(recipe);
  if (model.manualDownloadGateVisible && model.manualDownloads.length > 0) {
    page.append(manualDownloadPanel(model, actions));
  }
  page.append(finish);
  return page;
}

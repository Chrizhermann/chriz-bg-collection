import type {
  BackendStatus,
  DestinationEvaluation,
  GameCandidate,
  GameDiscovery,
  ManualDownloadRequirement,
  SelectionEvaluation,
} from "../contracts";
import { actionButton, element, screenIntro } from "../components/app-shell";
import { installationReadiness, validateInstallationName } from "../state";

export interface InstallScreenModel {
  readonly discovery: GameDiscovery;
  readonly selectedBg1Id: string;
  readonly selectedBg2Id: string;
  readonly installationName: string;
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
}

export interface InstallScreenActions {
  readonly selectSource: (game: "bg1" | "bg2", id: string) => void | Promise<void>;
  readonly browseSource: (game: "bg1" | "bg2") => void | Promise<void>;
  readonly changeName: (name: string) => void | Promise<void>;
  readonly changeLocation: (path: string) => void | Promise<void>;
  readonly browseLocation: () => void | Promise<void>;
  readonly customize: () => void | Promise<void>;
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
    element("p", "manual-download-intro", "CEBG cannot redistribute this mod. Download its official file, then choose that file here so CEBG can verify it."),
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
        : "Choose the exact official file shown below.")),
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
  game: "bg1" | "bg2",
  title: string,
  candidates: readonly GameCandidate[],
  selectedId: string,
  starting: boolean,
  actions: InstallScreenActions,
): HTMLElement {
  const card = element("article", "source-card card");
  card.dataset.source = game;
  const candidate = selectedCandidate(candidates, selectedId);
  const copy = element("div", "source-card-heading");
  copy.append(element("p", "source-title", title));
  copy.append(element("h2", undefined, candidate === undefined ? "Source needed" : candidates.length === 1 ? "Found source" : "Choose source"));
  const status = element("span", `source-status ${candidate?.eligible === true ? "ok" : "warning"}`, candidate?.eligible === true ? "Ready" : "Needs attention");
  card.append(copy, status);

  if (candidates.length > 1 || (candidate === undefined && candidates.length > 0)) {
    const label = element("label", "visually-hidden", game === "bg1" ? "Baldur's Gate source" : "Baldur's Gate II source");
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
  const details = element("details", "source-details");
  details.append(element("summary", undefined, "Details"));
  if (candidate === undefined) {
    details.append(element("p", "finding warning", "Choose the game folder so CEBG can inspect it."));
  } else if (candidate.findings.length === 0) {
    details.append(element("p", "finding", "No additional findings."));
  } else {
    for (const finding of candidate.findings) details.append(element("p", `finding ${candidate.eligible ? "ok" : "warning"}`, finding));
  }
  const changeSource = actionButton("Change source", () => actions.browseSource(game), "quiet compact");
  changeSource.setAttribute("aria-label", game === "bg1" ? "Change Baldur's Gate source" : "Change Baldur's Gate II source");
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
  page.append(screenIntro("", "Install Chriz Easy BG", "Your games, with Chriz's recommended mods. Ready to install in a separate folder."));

  const sources = element("section", "source-grid install-sources");
  sources.setAttribute("aria-label", "Found game sources");
  sources.append(
    sourceCard("bg1", "Baldur's Gate + Siege of Dragonspear", model.discovery.bg1Candidates, model.selectedBg1Id, model.starting, actions),
    sourceCard("bg2", "Baldur's Gate II", model.discovery.bg2Candidates, model.selectedBg2Id, model.starting, actions),
  );

  const settings = element("section", "install-settings card");
  settings.append(element("h2", undefined, "Your installation"));
  const fields = element("div", "install-field-grid");
  const name = field("Install name", "install-name", model.installationName);
  const location = field("Install location", "install-location", model.destination.path);
  name.input.disabled = model.starting;
  location.input.disabled = model.starting;
  const nameError = validateInstallationName(model.installationName);
  const nameHelp = element("p", nameError === null ? "field-help" : "field-error", nameError ?? "This name also labels your installation in CEBG.");
  nameHelp.id = "install-name-help";
  name.input.setAttribute("aria-describedby", nameHelp.id);
  if (nameError !== null) name.input.setAttribute("aria-invalid", "true");
  name.wrapper.append(nameHelp);
  name.input.addEventListener("change", () => void actions.changeName(name.input.value));
  const locationControls = element("div", "location-controls");
  const changeLocation = actionButton("Browse…", actions.browseLocation, "quiet");
  changeLocation.setAttribute("aria-label", "Change install location");
  changeLocation.disabled = model.starting;
  const locationHelp = element("p", "field-help", "Enter a new folder path or browse. CEBG creates missing folders when you install.");
  locationHelp.id = "install-location-help";
  location.input.setAttribute("aria-describedby", locationHelp.id);
  location.wrapper.append(locationHelp);
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
  locationControls.append(location.wrapper, changeLocation);
  location.input.addEventListener("change", () => void actions.changeLocation(location.input.value));
  fields.append(name.wrapper, locationControls);
  settings.append(fields, element("p", "safety-note", "Your original games stay unchanged. CEBG installs into this separate folder."));

  const recipe = element("section", "recommended-setup card");
  const recipeCopy = element("div");
  recipeCopy.append(
    element("h2", undefined, "Recommended setup"),
    element("p", "choice-summary", model.evaluation === null ? "Checking the recommended choices…" : `${model.evaluation.selectedChoiceCount} choices included`),
  );
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
    name: model.installationName,
    bg1,
    bg2,
    destination: model.destination,
    evaluation: model.evaluation,
    evaluationPending: model.evaluationPending,
  });
  const manualDownloadPending = model.manualDownloadGateVisible
    && (model.manualDownloadCheckingArtifactId !== null || model.manualDownloads.some((requirement) => !requirement.ready));
  const ready = selectionReady && !manualDownloadPending;
  const finish = element("section", `install-ready ${ready ? "is-ready" : "needs-attention"}`);
  const readiness = element("div");
  readiness.append(
    element("h2", undefined, model.starting ? "Starting installation" : model.evaluationPending ? "Checking your choices" : ready ? "Ready to install" : "Needs attention"),
    element("p", undefined, model.starting ? "Preparing your selected setup…" : model.evaluationPending ? "This will only take a moment." : ready ? "Everything required for this installation is ready." : "Resolve the highlighted item before installing."),
  );
  const install = actionButton(model.starting ? "Starting installation…" : "Install Chriz Easy BG", actions.install);
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

  page.append(sources, settings, recipe);
  if (model.manualDownloadGateVisible && model.manualDownloads.length > 0) {
    page.append(manualDownloadPanel(model, actions));
  }
  page.append(finish);
  return page;
}

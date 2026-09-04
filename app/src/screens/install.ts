import type {
  DestinationEvaluation,
  GameCandidate,
  GameDiscovery,
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
}

export interface InstallScreenActions {
  readonly selectSource: (game: "bg1" | "bg2", id: string) => void | Promise<void>;
  readonly browseSource: (game: "bg1" | "bg2") => void | Promise<void>;
  readonly changeName: (name: string) => void | Promise<void>;
  readonly changeLocation: (path: string) => void | Promise<void>;
  readonly browseLocation: () => void | Promise<void>;
  readonly customize: () => void | Promise<void>;
  readonly install: () => void | Promise<void>;
}

function selectedCandidate(candidates: readonly GameCandidate[], selectedId: string): GameCandidate | undefined {
  return candidates.find((candidate) => candidate.id === selectedId) ?? candidates[0];
}

function sourceCard(
  game: "bg1" | "bg2",
  title: string,
  candidates: readonly GameCandidate[],
  selectedId: string,
  actions: InstallScreenActions,
): HTMLElement {
  const card = element("article", "source-card card");
  card.dataset.source = game;
  const candidate = selectedCandidate(candidates, selectedId);
  const copy = element("div", "source-card-heading");
  copy.append(element("p", "source-title", title));
  copy.append(element("h2", undefined, candidates.length <= 1 ? "Found source" : "Choose source"));
  const status = element("span", `source-status ${candidate?.eligible === true ? "ok" : "warning"}`, candidate?.eligible === true ? "Ready" : "Needs attention");
  card.append(copy, status);

  if (candidates.length > 1) {
    const label = element("label", "visually-hidden", game === "bg1" ? "Baldur's Gate source" : "Baldur's Gate II source");
    const select = element("select");
    select.id = `source-${game}`;
    label.htmlFor = select.id;
    for (const optionCandidate of candidates) {
      const option = element("option", undefined, optionCandidate.label);
      option.value = optionCandidate.id;
      option.selected = optionCandidate.id === candidate?.id;
      select.append(option);
    }
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
  page.append(screenIntro("Everything in one place", "Install Chriz Easy BG", "CEBG finds your games, uses the recommended setup, and builds a separate installation for you."));

  const sources = element("section", "source-grid install-sources");
  sources.setAttribute("aria-label", "Found game sources");
  sources.append(
    sourceCard("bg1", "Baldur's Gate + Siege of Dragonspear", model.discovery.bg1Candidates, model.selectedBg1Id, actions),
    sourceCard("bg2", "Baldur's Gate II", model.discovery.bg2Candidates, model.selectedBg2Id, actions),
  );

  const settings = element("section", "install-settings card");
  settings.append(element("h2", undefined, "Your installation"));
  const fields = element("div", "install-field-grid");
  const name = field("Install name", "install-name", model.installationName);
  const location = field("Install location", "install-location", model.destination.path);
  const nameError = validateInstallationName(model.installationName);
  const nameHelp = element("p", nameError === null ? "field-help" : "field-error", nameError ?? "This name also labels your installation in CEBG.");
  nameHelp.id = "install-name-help";
  name.input.setAttribute("aria-describedby", nameHelp.id);
  if (nameError !== null) name.input.setAttribute("aria-invalid", "true");
  name.wrapper.append(nameHelp);
  name.input.addEventListener("change", () => void actions.changeName(name.input.value));
  const locationControls = element("div", "location-controls");
  const changeLocation = actionButton("Change", actions.browseLocation, "quiet");
  changeLocation.setAttribute("aria-label", "Change install location");
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
  recipe.append(recipeCopy, actionButton("Customize", actions.customize, "quiet"));
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
  const ready = installationReadiness({
    starting: model.starting,
    name: model.installationName,
    bg1,
    bg2,
    destination: model.destination,
    evaluation: model.evaluation,
    evaluationPending: model.evaluationPending,
  });
  const finish = element("section", `install-ready ${ready ? "is-ready" : "needs-attention"}`);
  const readiness = element("div");
  readiness.append(
    element("h2", undefined, ready ? "Ready to install" : "Needs attention"),
    element("p", undefined, ready ? "Everything required for this installation is ready." : "Resolve the highlighted item before installing."),
  );
  const install = actionButton(model.starting ? "Starting installation…" : "Install Chriz Easy BG", actions.install);
  install.disabled = !ready;
  finish.append(readiness, install);

  page.append(sources, settings, recipe, finish);
  return page;
}

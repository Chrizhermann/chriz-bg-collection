import type { ManagedInstallation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export interface HomeActions {
  readonly begin: () => void | Promise<void>;
  readonly launch: (installId: string) => void | Promise<void>;
  readonly openFolder: (installId: string) => void | Promise<void>;
  readonly resume: (installId: string) => void | Promise<void>;
  readonly select: (installId: string) => void | Promise<void>;
}

function installationPicker(
  installations: readonly ManagedInstallation[],
  selectedId: string,
  selectInstallation: (installId: string) => void | Promise<void>,
): HTMLElement | null {
  if (installations.length < 2) return null;
  const field = element("div", "install-switcher");
  const label = element("label", undefined, "Switch install");
  const select = element("select");
  for (const installation of installations) {
    const option = element("option", undefined, installation.name);
    option.value = installation.id;
    option.selected = installation.id === selectedId;
    select.append(option);
  }
  select.addEventListener("change", () => void selectInstallation(select.value));
  label.append(select);
  field.append(label);
  return field;
}

function installationDetails(installation: ManagedInstallation): HTMLDetailsElement {
  const details = element("details", "installation-details");
  details.append(element("summary", undefined, "Installation details"));
  const rows = element("dl", "installation-detail-list");
  const appendRow = (label: string, value: string | null): void => {
    if (value !== null) rows.append(element("dt", undefined, label), element("dd", "path", value));
  };
  appendRow("Install location", installation.path);
  appendRow("Game launcher", installation.launchPath);
  appendRow("Installation receipt", installation.receiptPath);
  appendRow("Recipe version", installation.recipeVersion ?? null);
  details.append(rows);
  return details;
}

export function homeScreen(
  installations: readonly ManagedInstallation[],
  selected: ManagedInstallation | null,
  actions: HomeActions,
): HTMLElement {
  const page = element("div", "screen-stack launcher-screen");
  if (selected === null) {
    page.append(screenIntro("Your installations", "No installations yet", "Create Chriz Easy BG when you are ready to play."));
    page.append(screenActions(null, actionButton("New installation", actions.begin)));
    return page;
  }

  if (selected.available) {
    page.append(screenIntro("Your game is ready", "Ready to play", "Continue your adventure or open the game folder."));
  } else if (selected.resumable) {
    page.append(screenIntro("CEBG can continue", "Continue your installation", "Your previous progress is saved and ready to resume."));
  } else {
    page.append(screenIntro("CEBG remembers this install", "Installation not found", "The registered game folder moved or is no longer available."));
  }

  const card = element("section", "card launcher-card");
  card.append(
    element("p", `badge ${selected.available ? "ok" : selected.resumable ? "warning" : "danger"}`, selected.status),
    element("h2", undefined, selected.name),
  );
  const picker = installationPicker(installations, selected.id, actions.select);
  if (picker !== null) card.append(picker);
  const controls = element("div", "inline-actions launcher-actions");
  if (selected.available) {
    controls.append(
      actionButton("Play Chriz Easy BG", () => actions.launch(selected.id)),
      actionButton("Open game folder", () => actions.openFolder(selected.id), "quiet"),
    );
  } else if (selected.resumable) {
    controls.append(actionButton("Continue installation", () => actions.resume(selected.id)));
  }
  card.append(controls, installationDetails(selected));
  page.append(card, screenActions(null, actionButton("New installation", actions.begin, "quiet")));
  return page;
}

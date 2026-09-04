import type { ManagedInstallation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export interface HomeActions {
  readonly begin: () => void | Promise<void>;
  readonly launch: (installId: string) => void | Promise<void>;
  readonly openFolder: (installId: string) => void | Promise<void>;
  readonly resume: (installId: string) => void | Promise<void>;
  readonly select: (installId: string) => void | Promise<void>;
  readonly createShortcut: (installId: string) => void | Promise<void>;
}

export interface ShortcutFeedback {
  readonly installId: string;
  readonly state: "created" | "failed";
  readonly path?: string;
}

export interface AddonFeedback {
  readonly installId: string;
  readonly state: "installing" | "failed";
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
  appendRow("Collection version", installation.recipeVersion ?? null);
  appendRow("BG Radar Overlay", installation.radarVersion ?? null);
  appendRow("Mod check", installation.consistency?.detail ?? null);
  details.append(rows);
  return details;
}

export function homeScreen(
  installations: readonly ManagedInstallation[],
  selected: ManagedInstallation | null,
  actions: HomeActions,
  shortcutFeedback: ShortcutFeedback | null = null,
  addonFeedback: AddonFeedback | null = null,
): HTMLElement {
  const page = element("div", "screen-stack launcher-screen");
  if (selected === null) {
    page.append(screenIntro("", "My installs", "No installations yet. Set up Chriz Easy BG to start playing."));
    page.append(screenActions(null, actionButton("New installation", actions.begin)));
    return page;
  }

  if (selected.available) {
    page.append(screenIntro("", "Ready to play", "Continue your adventure."));
  } else if (selected.resumable) {
    page.append(screenIntro("", "Continue your installation", "Your previous progress is saved and ready to resume."));
  } else {
    page.append(screenIntro("", "Installation not found", "The game folder moved or is no longer available. Reconnect the drive if it is stored elsewhere."));
  }

  const card = element("section", "card launcher-card");
  card.append(
    element("p", `badge ${selected.available ? "ok" : selected.resumable ? "warning" : "danger"}`, selected.status),
    element("h2", undefined, selected.name),
  );
  const metadata = element("div", "installation-metadata");
  if (selected.recipeVersion) metadata.append(element("span", undefined, `Collection ${selected.recipeVersion}`));
  if (selected.consistency !== undefined) {
    const check = selected.consistency;
    const summary = element("span", `consistency-summary ${check.state}`, check.state === "matches"
      ? `${check.modCount} mods · ${check.componentCount} components checked`
      : check.state === "changed" ? "Mods changed since installation" : "Mod check unavailable");
    summary.dataset.consistency = check.state;
    metadata.append(summary);
  }
  if (metadata.childElementCount > 0) card.append(metadata);
  const picker = installationPicker(installations, selected.id, actions.select);
  if (picker !== null) card.append(picker);
  const controls = element("div", "inline-actions launcher-actions");
  let feedback: HTMLElement | null = null;
  if (selected.available) {
    const currentFeedback = shortcutFeedback?.installId === selected.id ? shortcutFeedback : null;
    controls.append(
      actionButton("Play Chriz Easy BG", () => actions.launch(selected.id)),
      actionButton("Open game folder", () => actions.openFolder(selected.id), "quiet"),
      actionButton(
        currentFeedback?.state === "created"
          ? "Recreate desktop shortcut"
          : currentFeedback?.state === "failed"
            ? "Retry desktop shortcut"
            : "Create desktop shortcut",
        () => actions.createShortcut(selected.id),
        "quiet",
      ),
    );
    if (currentFeedback !== null) {
      feedback = element("div", `shortcut-feedback ${currentFeedback.state === "created" ? "ok" : "danger"}`);
      feedback.setAttribute("role", "status");
      feedback.append(element("strong", undefined, currentFeedback.state === "created" ? "Shortcut created on your desktop." : "The desktop shortcut wasn't created."));
      if (currentFeedback.path !== undefined) feedback.append(element("p", "path", currentFeedback.path));
    }
  } else if (selected.resumable) {
    controls.append(actionButton("Continue installation", () => actions.resume(selected.id)));
  }
  card.append(controls);
  if (addonFeedback?.installId === selected.id) {
    const note = element("p", "addon-feedback", addonFeedback.state === "installing"
      ? "Adding BG Radar Overlay… Your game is ready to play."
      : "BG Radar Overlay couldn't be added. Your game is ready; retry the overlay from Updates.");
    note.setAttribute("role", "status");
    card.append(note);
  }
  if (feedback !== null) card.append(feedback);
  card.append(installationDetails(selected));
  page.append(card, screenActions(null, actionButton("New installation", actions.begin, "quiet")));
  return page;
}

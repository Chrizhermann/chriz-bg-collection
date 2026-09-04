import type { ManagedInstallation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export interface HomeActions {
  readonly begin: () => void | Promise<void>;
  readonly launch: (installId: string) => void | Promise<void>;
  readonly openFolder: (installId: string) => void | Promise<void>;
  readonly resume: (installId: string) => void | Promise<void>;
}

export function homeScreen(installations: readonly ManagedInstallation[], actions: HomeActions): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Your campaigns", "Your campaigns", "Managed copies remain separate from store games and from one another."));
  const list = element("div", "campaign-list");
  installations.forEach((installation) => {
    const card = element("article", "card campaign-card");
    card.append(element("p", "badge ok", installation.status), element("h2", undefined, installation.name), element("p", "path", installation.path));
    if (installation.receiptPath !== null) {
      card.append(element("p", undefined, `Receipt: ${installation.receiptPath}`));
    }
    if (installation.available) {
      const controls = element("div", "inline-actions");
      controls.append(
        actionButton("Play", () => actions.launch(installation.id)),
        actionButton("Open folder", () => actions.openFolder(installation.id), "quiet"),
      );
      card.append(controls);
    } else if (installation.resumable) {
      card.append(actionButton("Resume build", () => actions.resume(installation.id)));
    }
    list.append(card);
  });
  page.append(list, screenActions(null, actionButton("Build a new campaign", actions.begin)));
  return page;
}

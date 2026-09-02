import type { ManagedInstallation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export function homeScreen(installations: readonly ManagedInstallation[], begin: () => void): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Your campaigns", "Your campaigns", "Managed copies remain separate from store games and from one another."));
  const list = element("div", "campaign-list");
  installations.forEach((installation) => {
    const card = element("article", "card campaign-card");
    card.append(element("p", "badge ok", installation.status), element("h2", undefined, installation.name), element("p", "path", installation.path), element("p", undefined, `Receipt: ${installation.receiptPath}`));
    list.append(card);
  });
  page.append(list, screenActions(null, actionButton("Build a new campaign", begin)));
  return page;
}

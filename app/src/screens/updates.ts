import type { UpdateSummary } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";

export interface UpdateActions {
  readonly checkAgain: () => void | Promise<void>;
  readonly installApplication: (version: string) => void | Promise<void>;
  readonly buildUpdatedCopy: (version: string) => void | Promise<void>;
}

export function updatesScreen(updates: UpdateSummary, actions: UpdateActions): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Keep CEBG current", "Updates", "The app and curated setup update separately, so each change stays clear."));
  const appTitle = updates.application.availableVersion === null ? "Application" : `Application ${updates.application.availableVersion}`;
  const appCard = statusCard(appTitle, updates.application.detail, updates.application.state === "invalid" ? "danger" : updates.application.state === "available" ? "warning" : "ok");
  if (updates.application.state === "available" && updates.application.availableVersion !== null) {
    appCard.append(actionButton("Install application update", () => actions.installApplication(updates.application.availableVersion!)));
  }
  const recipeTitle = updates.recipe.availableVersion === null ? "Recipe" : `Recipe ${updates.recipe.availableVersion}`;
  const recipeCard = statusCard(recipeTitle, updates.recipe.detail, ["invalid", "replayed"].includes(updates.recipe.state) ? "danger" : updates.recipe.state === "available" ? "warning" : "neutral");
  for (const change of updates.recipe.changes) {
    const item = element("div", "update-change");
    item.append(element("h3", undefined, change.title), element("p", undefined, change.summary));
    if (change.conditionNote !== null) item.append(element("p", "fixture-note", change.conditionNote));
    recipeCard.append(item);
  }
  if (updates.recipe.state === "available" && updates.recipe.availableVersion !== null) {
    recipeCard.append(actionButton("Create updated installation", () => actions.buildUpdatedCopy(updates.recipe.availableVersion!)));
  }
  page.append(appCard, recipeCard);
  if (updates.managedCopies.length > 0) {
    const copies = element("section", "screen-stack");
    copies.append(element("h2", undefined, "Your installations"));
    for (const copy of updates.managedCopies) {
      copies.append(statusCard(copy.name, copy.detail, copy.state === "stale" ? "danger" : copy.state === "update-available" ? "warning" : "neutral"));
    }
    page.append(copies);
  }
  page.append(
    statusCard("Your current game stays safe.", "When an update cannot be applied safely, CEBG creates a separate updated installation. Your current saves stay unchanged.", "warning"),
    element("p", "fixture-note", updates.checkedAt === null ? "No successful update check has been recorded." : `Last checked ${updates.checkedAt}`),
    screenActions(null, actionButton("Check again", actions.checkAgain, "quiet")),
  );
  return page;
}

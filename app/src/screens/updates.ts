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
  page.append(screenIntro("Separate release tracks", "Updates", "Application, recipe, and campaign-copy versions have distinct lifecycles."));
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
    recipeCard.append(actionButton("Build updated copy", () => actions.buildUpdatedCopy(updates.recipe.availableVersion!)));
  }
  page.append(appCard, recipeCard);
  if (updates.managedCopies.length > 0) {
    const copies = element("section", "screen-stack");
    copies.append(element("h2", undefined, "Managed campaign copies"));
    for (const copy of updates.managedCopies) {
      copies.append(statusCard(copy.name, copy.detail, copy.state === "stale" ? "danger" : copy.state === "update-available" ? "warning" : "neutral"));
    }
    page.append(copies);
  }
  page.append(
    statusCard("Existing campaigns are never patched in place.", "Recipe updates build a separate managed copy. The working campaign and its saves stay unchanged.", "warning"),
    element("p", "fixture-note", updates.checkedAt === null ? "No successful update check has been recorded." : `Last checked ${updates.checkedAt}`),
    screenActions(null, actionButton("Check again", actions.checkAgain, "quiet")),
  );
  return page;
}

import type { UpdateSummary } from "../contracts";
import { element, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";

export function updatesScreen(updates: UpdateSummary): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Separate release tracks", "Updates", "Application, recipe, and campaign-copy versions have distinct lifecycles."));
  page.append(
    statusCard("Application", updates.app, "ok"),
    statusCard("Recipe", updates.recipe, "neutral"),
    statusCard("Existing campaigns are never patched in place.", "A future safe live patch path must be explicitly classified and reviewed; this private alpha performs no update action.", "warning"),
    element("p", "fixture-note", updates.message),
  );
  return page;
}

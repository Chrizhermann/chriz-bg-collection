import type { FrozenReview } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";

export function completeScreen(review: FrozenReview | null, home: () => void): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Build verified", "Campaign complete", "The fixture campaign is ready for live installer wiring in the next engineering task."));
  page.append(statusCard("Immutable install receipt", review ? `Review ${review.digest} is bound to ${review.destination}.` : "Fixture receipt preview." , "ok"));
  page.append(screenActions(null, actionButton("View campaigns", home)));
  return page;
}

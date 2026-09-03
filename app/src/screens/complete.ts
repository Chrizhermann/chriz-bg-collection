import type { FrozenReview } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { statusCard } from "../components/status-card";

export function completeScreen(review: FrozenReview | null, home: () => void): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Build verified", "Campaign complete", "The reviewed managed campaign and its durable evidence are ready."));
  page.append(statusCard("Immutable install receipt", review ? `Review ${review.digest} is bound to ${review.destination}.` : "The campaign receipt is available from its managed copy." , "ok"));
  page.append(screenActions(null, actionButton("View campaigns", home)));
  return page;
}

import type { FeatureGroup } from "../contracts";
import { actionButton, element } from "./app-shell";
import { featureGroupCheckbox, featureGroupStateText, type FeatureGroupChange } from "./feature-group-card";

export function mainChoiceControlId(group: FeatureGroup): string {
  return `main-choice-${group.id.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
}

export interface MainChoiceActions {
  /** Sends the group's member changes as one batch, like the group's card in Customize. */
  readonly change: FeatureGroupChange;
  /** Opens Customize at the group's card, where every single change is listed. */
  readonly details: (group: FeatureGroup) => void | Promise<void>;
  /** Runs when a switch is used, before the batch is sent. */
  readonly requested?: () => void;
}

/**
 * The recipe's feature groups (Challenge Mode) as main choices on the install screen: one
 * switch per group. The state shown is the engine's derived group state, the same one the
 * group's card in Customize shows, so the two never disagree.
 */
export function mainChoicesCard(
  groups: readonly FeatureGroup[],
  actions: MainChoiceActions,
  disabled: boolean,
): HTMLElement {
  const card = element("section", "main-choices card");
  card.setAttribute("aria-labelledby", "main-choices-title");
  const heading = element("h2", undefined, "Main choices");
  heading.id = "main-choices-title";
  card.append(heading);
  for (const group of groups) {
    const id = mainChoiceControlId(group);
    const row = element("div", `main-choice is-${group.state}`);
    row.dataset.group = group.id;
    const checkbox = featureGroupCheckbox(group, id, actions.change, actions.requested);
    checkbox.disabled = disabled;
    const label = element("label", "main-choice-title", group.title);
    label.htmlFor = id;
    const description = element("p", "main-choice-description", group.description);
    description.id = `${id}-description`;
    const state = element("p", group.disabledReason ? "main-choice-state unavailable-reason" : "main-choice-state", featureGroupStateText(group));
    state.id = `${id}-state`;
    checkbox.setAttribute("aria-describedby", `${description.id} ${state.id}`);
    const details = actionButton("Details", () => actions.details(group), "quiet compact");
    details.setAttribute("aria-label", `${group.title} details`);
    details.disabled = disabled;
    row.append(checkbox, label, details, description, state);
    card.append(row);
  }
  return card;
}

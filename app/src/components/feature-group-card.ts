import type { FeatureGroup, SelectionEvaluation } from "../contracts";
import { featureGroupChanges } from "../customization";
import { element } from "./app-shell";

export function featureGroupControlId(group: FeatureGroup): string {
  return `group-${group.id.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
}

function stateText(group: FeatureGroup): string {
  if (group.disabledReason) return group.disabledReason;
  const available = group.availableMembers.length;
  const selected = group.selectedMembers.length;
  if (group.state === "on") return `On: all ${available} available ${available === 1 ? "change is" : "changes are"} included.`;
  if (group.state === "mixed") return `Partly on: ${selected} of ${available} available changes are included. Select it to include them all.`;
  return `Off: no ${group.title} change is included.`;
}

/**
 * Prominent tri-state switch for one authored feature group. Its state is derived by the
 * engine from the member features; a click sends all member changes as one batch.
 */
export function featureGroupCard(
  evaluation: SelectionEvaluation,
  group: FeatureGroup,
  change: (changes: Record<string, boolean>, focusId: string) => void | Promise<void>,
): HTMLElement {
  const id = featureGroupControlId(group);
  const card = element("section", "feature-group-card");
  card.dataset.group = group.id;
  card.setAttribute("aria-labelledby", `${id}-label`);

  const header = element("div", "feature-group-header");
  const checkbox = element("input");
  checkbox.type = "checkbox";
  checkbox.id = id;
  checkbox.checked = group.state === "on";
  checkbox.indeterminate = group.state === "mixed";
  checkbox.setAttribute("aria-checked", group.state === "mixed" ? "mixed" : String(group.state === "on"));
  const label = element("label", "feature-group-title", group.title);
  label.id = `${id}-label`;
  label.htmlFor = id;
  header.append(checkbox, label);

  const description = element("p", "feature-group-description", group.description);
  description.id = `${id}-description`;
  const state = element("p", group.disabledReason ? "feature-group-state unavailable-reason" : "feature-group-state", stateText(group));
  state.id = `${id}-state`;
  checkbox.setAttribute("aria-describedby", `${description.id} ${state.id}`);

  const controls = new Map(evaluation.view.controls.map(control => [control.id, control]));
  const unavailable = new Map(group.unavailableMembers.map(member => [member.featureId, member.reason]));
  const list = element("ul", "feature-group-members");
  list.setAttribute("aria-label", `${group.title} changes`);
  for (const memberId of group.members) {
    const control = controls.get(memberId);
    const item = element("li", "feature-group-member");
    item.dataset.feature = memberId;
    const title = element("span", "feature-group-member-title", control?.title ?? memberId);
    const reason = unavailable.get(memberId);
    const selected = group.selectedMembers.includes(memberId);
    const status = element("span", `feature-group-member-state ${reason ? "is-unavailable" : selected ? "is-included" : ""}`.trim(),
      reason ? "Unavailable" : selected ? "Included" : "Not included");
    item.append(title, status);
    if (control?.sourceLabel) item.append(element("span", "feature-group-member-source", control.sourceLabel));
    if (reason) item.append(element("p", "unavailable-reason", reason));
    list.append(item);
  }
  const note = element("p", "feature-group-note",
    `Each change can also be switched on or off by itself under its own mod in “Advanced options by category”, where it is marked “${group.title}”.`);

  if (group.disabledReason) {
    checkbox.setAttribute("aria-disabled", "true");
    checkbox.addEventListener("click", (event) => {
      event.preventDefault();
    });
  } else {
    // The checkbox's own state is the requested target: a click on "off" or "mixed" checks it
    // and a click on "on" unchecks it. Reading it at click time (not the rendered group state)
    // keeps rapid clicks made before the next evaluation re-renders the card in step with the
    // last action.
    checkbox.addEventListener("change", () => {
      checkbox.setAttribute("aria-checked", String(checkbox.checked));
      void change(featureGroupChanges(group, checkbox.checked), checkbox.id);
    });
  }

  card.append(header, description, state, list, note);
  return card;
}

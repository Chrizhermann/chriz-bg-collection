import type { SelectionEvaluation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export function setupScreen(
  evaluation: SelectionEvaluation,
  onToggle: (id: string, selected: boolean) => void | Promise<void>,
  back: () => void,
  next: () => void,
): HTMLElement {
  const page = element("div", "screen-stack");
  page.append(screenIntro("Optional choices", "Customize your installation", "The recommended setup is already selected. Change only what you want, then return to the installation screen."));
  const summary = element("p", "choice-summary", `${evaluation.selectedChoiceCount} choices included`);
  page.append(summary);
  evaluation.view.categories.forEach((category) => {
    const fieldset = element("fieldset", "choice-group card");
    fieldset.append(element("legend", undefined, category));
    evaluation.view.controls.filter((control) => control.category === category).forEach((control) => {
      const row = element("div", `control-row ${control.interactive ? "" : "is-unavailable"}`.trim());
      const input = element("input");
      input.type = "checkbox";
      input.id = `feature-${control.id}`;
      input.checked = control.selected;
      const descriptionId = `${control.id}-description`;
      input.setAttribute("aria-describedby", control.unavailableReason ? `${descriptionId} ${control.id}-reason` : descriptionId);
      if (!control.interactive) input.setAttribute("aria-disabled", "true");
      input.addEventListener("click", (event) => {
        if (!control.interactive) {
          event.preventDefault();
          input.checked = control.selected;
          return;
        }
        void onToggle(control.id, input.checked);
      });
      const copy = element("div");
      const label = element("label", undefined, control.title);
      label.htmlFor = input.id;
      const badge = element("span", `badge ${control.decision}`, control.decision);
      const description = element("p", undefined, control.description);
      description.id = descriptionId;
      copy.append(label, badge);
      if (control.readiness !== "ready") copy.append(element("span", `badge ${control.readiness}`, control.readiness));
      copy.append(description);
      if (control.unavailableReason) {
        const reason = element("p", "unavailable-reason", control.unavailableReason);
        reason.id = `${control.id}-reason`;
        reason.tabIndex = 0;
        copy.append(reason);
      }
      row.append(input, copy);
      fieldset.append(row);
    });
    page.append(fieldset);
  });
  if (evaluation.findings.length > 0) {
    const notices = element("section", "card plan-notices");
    notices.setAttribute("aria-labelledby", "setup-notices-title");
    const heading = element("h2", undefined, "Plan notices");
    heading.id = "setup-notices-title";
    const list = element("ul");
    evaluation.findings.forEach((finding) => list.append(element("li", undefined, finding.message)));
    notices.append(heading, list);
    page.append(notices);
  }
  page.append(screenActions(back, actionButton("Done", next)));
  return page;
}

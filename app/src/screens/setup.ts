import type { SelectionEvaluation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export interface SetupViewState {
  search: string;
  category: string;
}

const decisionLabels = { mandatory: "Always included", default: "Recommended", optional: "Optional", excluded: "Not included" } as const;

export function setupScreen(
  evaluation: SelectionEvaluation,
  onToggle: (id: string, selected: boolean) => void | Promise<void>,
  back: () => void,
  next: () => void,
  viewState: SetupViewState = { search: "", category: "" },
): HTMLElement {
  const page = element("div", "screen-stack setup-screen");
  const header = element("div", "setup-toolbar");
  const top = element("div", "setup-heading");
  top.append(screenIntro("", "Customize your installation", "Recommended choices are selected. Change anything you like."), screenActions(back, actionButton("Done", next)));
  const filters = element("div", "setup-filters");
  const searchLabel = element("label", undefined, "Find a mod or option");
  const search = element("input");
  search.type = "search";
  search.id = "setup-search";
  search.placeholder = "Search choices…";
  search.value = viewState.search;
  searchLabel.htmlFor = search.id;
  searchLabel.append(search);
  const categoryLabel = element("label", undefined, "Category");
  const categorySelect = element("select");
  categorySelect.id = "setup-category";
  categoryLabel.htmlFor = categorySelect.id;
  const allCategories = element("option", undefined, "All categories");
  allCategories.value = "";
  categorySelect.append(allCategories);
  evaluation.view.categories.forEach((category) => {
    const option = element("option", undefined, category);
    option.value = category;
    categorySelect.append(option);
  });
  categorySelect.value = viewState.category;
  categoryLabel.append(categorySelect);
  const summary = element("p", "choice-summary", `${evaluation.selectedChoiceCount} choices included`);
  summary.setAttribute("role", "status");
  filters.append(searchLabel, categoryLabel, summary);
  header.append(top, filters);
  page.append(header);
  const groups = element("div", "setup-groups");
  const filterable: { group: HTMLElement; category: string; rows: { element: HTMLElement; text: string }[] }[] = [];
  evaluation.view.categories.forEach((category) => {
    const fieldset = element("fieldset", "choice-group card");
    fieldset.append(element("legend", undefined, category));
    const rows: { element: HTMLElement; text: string }[] = [];
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
      const badge = element("span", `badge ${control.decision}`, decisionLabels[control.decision]);
      const description = element("p", undefined, control.description);
      description.id = descriptionId;
      copy.append(label, badge);
      if (control.readiness !== "ready") copy.append(element("span", `badge ${control.readiness}`, control.readiness === "blocked" ? "Unavailable" : "Experimental"));
      copy.append(description);
      if (control.unavailableReason) {
        const reason = element("p", "unavailable-reason", control.unavailableReason);
        reason.id = `${control.id}-reason`;
        reason.tabIndex = 0;
        copy.append(reason);
      }
      row.append(input, copy);
      fieldset.append(row);
      rows.push({ element: row, text: `${control.title} ${control.description}`.toLocaleLowerCase() });
    });
    groups.append(fieldset);
    filterable.push({ group: fieldset, category, rows });
  });
  const empty = element("p", "setup-empty", "No matching choices. Try another search or category.");
  const filter = (): void => {
    viewState.search = search.value;
    viewState.category = categorySelect.value;
    const query = viewState.search.trim().toLocaleLowerCase();
    let visible = 0;
    for (const group of filterable) {
      let groupVisible = 0;
      for (const row of group.rows) {
        row.element.hidden = (viewState.category !== "" && viewState.category !== group.category) || !row.text.includes(query);
        if (!row.element.hidden) groupVisible += 1;
      }
      group.group.hidden = groupVisible === 0;
      visible += groupVisible;
    }
    empty.hidden = visible > 0;
    summary.textContent = `${evaluation.selectedChoiceCount} choices included${query || viewState.category ? ` · ${visible} shown` : ""}`;
  };
  search.addEventListener("input", filter);
  categorySelect.addEventListener("change", filter);
  filter();
  page.append(groups, empty);
  if (evaluation.findings.length > 0) {
    const notices = element("details", "recipe-notices setup-notices");
    const heading = element("summary", undefined, `${evaluation.findings.length} setup ${evaluation.findings.length === 1 ? "note" : "notes"}`);
    const list = element("ul");
    evaluation.findings.forEach((finding) => list.append(element("li", undefined, finding.message)));
    notices.append(heading, list);
    page.append(notices);
  }
  return page;
}

import type { SelectionEvaluation } from "../contracts";
import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";
import { bulkCategoryChanges, bundleChanges, bundleSelected, commonBundles, exclusiveChoiceChanges, exclusiveChoiceGroups, type ExclusiveChoiceGroup } from "../customization";

export interface SetupViewState {
  search: string;
  category: string;
  advancedOpen?: boolean;
  bundleMemory?: Record<string, Readonly<Record<string, boolean>>>;
  adjustedChoices?: readonly string[];
}

export interface SetupBatchActions {
  change(changes: Record<string, boolean>, focusId: string): void | Promise<void>;
  reset(): void | Promise<void>;
}

const decisionLabels = { mandatory: "Always included", default: "Recommended", optional: "Optional", excluded: "Not included" } as const;

function choiceGroupControlId(group: ExclusiveChoiceGroup): string {
  return `choice-group-${group.id.replace(/[^a-zA-Z0-9_-]/g, "-")}`;
}

function choiceOptionLabel(group: ExclusiveChoiceGroup, title: string): string {
  const prefix = `${group.label}: `;
  return title.startsWith(prefix) ? title.slice(prefix.length) : title;
}

export function setupScreen(
  evaluation: SelectionEvaluation,
  onToggle: (id: string, selected: boolean) => void | Promise<void>,
  back: () => void,
  next: () => void,
  viewState: SetupViewState = { search: "", category: "" },
  batch?: SetupBatchActions,
): HTMLElement {
  const page = element("div", "screen-stack setup-screen");
  const header = element("div", "setup-toolbar");
  const top = element("div", "setup-heading");
  top.append(screenIntro("", "Customize your installation", "Chriz’s setup is ready to install. Change just the things you want."), screenActions(back, actionButton("Done", next)));
  header.append(top);
  page.append(header);
  const bundles = batch ? commonBundles(evaluation) : [];
  if (batch && bundles.length > 0) {
    const common = element("section", "common-choices");
    const title = element("h2", undefined, "Common changes");
    common.setAttribute("aria-label", "Common changes");
    const choices = element("div", "common-choice-grid");
    for (const bundle of bundles) {
      const row = element("label", "common-choice");
      const checkbox = element("input");
      checkbox.type = "checkbox";
      checkbox.id = `bundle-${bundle.id}`;
      checkbox.checked = bundleSelected(evaluation, bundle);
      checkbox.setAttribute("aria-label", bundle.title);
      checkbox.setAttribute("aria-describedby", `${checkbox.id}-description`);
      const copy = element("span");
      copy.append(element("strong", undefined, bundle.title));
      const description = element("span", "common-choice-description", bundle.description);
      description.id = `${checkbox.id}-description`;
      copy.append(description);
      row.append(checkbox, copy);
      checkbox.addEventListener("change", () => {
        const turningAway = bundle.id === "original-companions" ? checkbox.checked : !checkbox.checked;
        viewState.bundleMemory ??= {};
        if (turningAway) viewState.bundleMemory[bundle.id] = Object.fromEntries(bundle.ids.map(id => [id, evaluation.normalizedSelection.features[id] ?? false]));
        void batch.change(bundleChanges(evaluation, bundle, checkbox.checked, viewState.bundleMemory[bundle.id]), checkbox.id);
      });
      choices.append(row);
    }
    const reset = actionButton("Reset to Chriz’s setup", () => { viewState.bundleMemory = {}; void batch.reset(); }, "quiet");
    reset.classList.add("button-quiet");
    common.append(title, choices, reset);
    page.append(common);
  }
  if (viewState.adjustedChoices?.length) {
    const effects = element("details", "setup-effects");
    effects.append(element("summary", undefined, `Also adjusted: ${viewState.adjustedChoices.length} related choices`));
    const list = element("ul");
    viewState.adjustedChoices.forEach(text => list.append(element("li", undefined, text)));
    effects.append(list);
    page.append(effects);
  }
  const advanced = element("details", "setup-advanced");
  advanced.open = viewState.advancedOpen ?? bundles.length === 0;
  advanced.addEventListener("toggle", () => { viewState.advancedOpen = advanced.open; });
  advanced.append(element("summary", undefined, "Advanced options by category"));
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
  advanced.append(filters);
  const groups = element("div", "setup-groups");
  const filterable: { group: HTMLElement; category: string; rows: { element: HTMLElement; text: string }[] }[] = [];
  const exclusiveGroups = batch ? exclusiveChoiceGroups(evaluation) : [];
  const exclusiveByControl = new Map(exclusiveGroups.flatMap(group => group.controls.map(control => [control.id, group] as const)));
  evaluation.view.categories.forEach((category) => {
    const fieldset = element("fieldset", "choice-group card");
    fieldset.append(element("legend", undefined, category));
    if (batch) {
      const buttons = element("div", "category-actions");
      for (const [selected, label] of [[true, "Include all compatible"], [false, "Exclude all optional"]] as const) {
        const id = `category-${category}-${selected ? "include" : "exclude"}`;
        const changes = bulkCategoryChanges(evaluation, category, selected);
        const button = actionButton(label, () => { void batch.change(changes, id); }, "quiet");
        button.id = id;
        button.disabled = Object.keys(changes).length === 0;
        buttons.append(button);
      }
      fieldset.append(buttons, element("p", "category-hint", "Required fixes stay included. Unavailable choices and conflicting alternatives are left out."));
    }
    const rows: { element: HTMLElement; text: string }[] = [];
    const renderedExclusiveGroups = new Set<string>();
    evaluation.view.controls.filter((control) => control.category === category).forEach((control) => {
      const exclusive = exclusiveByControl.get(control.id);
      if (exclusive) {
        if (renderedExclusiveGroups.has(exclusive.id)) return;
        renderedExclusiveGroups.add(exclusive.id);
        const row = element("div", "exclusive-choice-row");
        const selectId = choiceGroupControlId(exclusive);
        const label = element("label", "exclusive-choice-label", exclusive.label);
        label.htmlFor = selectId;
        const select = element("select", "exclusive-choice-select");
        select.id = selectId;
        const selectedControl = exclusive.controls.find(option => option.selected);
        if (exclusive.allowNone) {
          const none = element("option", undefined, "None / unchanged");
          none.value = "";
          select.append(none);
        } else if (!selectedControl) {
          const placeholder = element("option", undefined, "Choose an option");
          placeholder.value = "";
          placeholder.disabled = true;
          select.append(placeholder);
        }
        for (const optionControl of exclusive.controls) {
          const available = optionControl.selected || optionControl.choiceAvailable === true;
          const optionLabel = `${choiceOptionLabel(exclusive, optionControl.title)}${optionControl.decision === "default" ? " (Recommended)" : ""}`;
          const option = element("option", undefined, `${optionLabel}${available ? "" : " (Unavailable)"}`);
          option.value = optionControl.id;
          option.disabled = !available;
          select.append(option);
        }
        select.value = selectedControl?.id ?? "";
        const detail = element("p", "exclusive-choice-description", selectedControl?.description ?? "No option selected.");
        const sources = [...new Set(exclusive.controls.map(option => option.sourceLabel).filter((source): source is string => Boolean(source)))];
        const source = element("p", "exclusive-choice-source", `${sources.length === 1 ? "Source" : "Sources"}: ${sources.join(", ")}`);
        source.hidden = sources.length === 0;
        const unavailable = exclusive.controls.filter(option => !option.selected && option.choiceAvailable !== true && option.unavailableReason);
        const reasons = element("ul", "exclusive-choice-reasons");
        for (const option of unavailable) reasons.append(element("li", undefined, `${option.title}: ${option.unavailableReason}`));
        reasons.hidden = unavailable.length === 0;
        select.setAttribute("aria-describedby", `${selectId}-description${unavailable.length ? ` ${selectId}-reasons` : ""}`);
        detail.id = `${selectId}-description`;
        reasons.id = `${selectId}-reasons`;
        select.addEventListener("change", () => {
          const selected = exclusive.controls.find(option => option.id === select.value);
          detail.textContent = selected?.description ?? "No option selected.";
          if (select.value === "" && !exclusive.allowNone) return;
          void batch!.change(exclusiveChoiceChanges(exclusive, select.value), select.id);
        });
        row.append(label, select, detail, source, reasons);
        fieldset.append(row);
        rows.push({
          element: row,
          text: [exclusive.label, ...exclusive.controls.flatMap(option => [option.title, option.description, option.sourceLabel ?? ""])].join(" ").toLocaleLowerCase(),
        });
        return;
      }
      const row = element("div", `control-row ${control.interactive ? "" : "is-unavailable"}`.trim());
      const input = element("input");
      input.type = "checkbox";
      input.id = `feature-${control.id}`;
      input.checked = control.selected;
      const descriptionId = `${control.id}-description`;
      const hasDistinctDescription = control.description.trim().toLocaleLowerCase() !== control.title.trim().toLocaleLowerCase();
      const describedBy = [hasDistinctDescription ? descriptionId : "", control.unavailableReason ? `${control.id}-reason` : ""].filter(Boolean).join(" ");
      if (describedBy) input.setAttribute("aria-describedby", describedBy);
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
      if (hasDistinctDescription) copy.append(description);
      if (control.unavailableReason) {
        const reason = element("p", "unavailable-reason", control.unavailableReason);
        reason.id = `${control.id}-reason`;
        reason.tabIndex = 0;
        copy.append(reason);
      }
      row.append(input, copy);
      fieldset.append(row);
      if (control.sourceLabel || control.groupLabel) {
        copy.append(element("p", "control-source", [control.groupLabel ? `Group: ${control.groupLabel}` : "", control.sourceLabel ? `Source: ${control.sourceLabel}` : ""].filter(Boolean).join(". ")));
      }
      rows.push({ element: row, text: `${control.title} ${control.description} ${control.sourceLabel ?? ""} ${control.groupLabel ?? ""}`.toLocaleLowerCase() });
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
  advanced.append(groups, empty);
  page.append(advanced);
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

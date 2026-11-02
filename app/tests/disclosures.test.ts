// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { fireEvent, getByRole, getByText } from "@testing-library/dom";
import css from "../src/styles.css?raw";
import { technicalLog, type TechnicalLogState } from "../src/components/technical-log";
import { setupScreen, type SetupViewState } from "../src/screens/setup";
import { FixtureBackend } from "../src/backend";

describe("stable, compact disclosures", () => {
  afterEach(() => document.body.replaceChildren());

  it("opens and closes the technical log synchronously and survives a render before native toggle dispatch", () => {
    let state: TechnicalLogState = { open: false, paused: false };
    const root = document.createElement("div");
    document.body.append(root);
    const render = () => {
      const log = technicalLog(["Current log line"], state, (next) => { state = next; });
      root.replaceChildren(log);
      return log as HTMLDetailsElement;
    };
    const old = render();
    fireEvent.click(getByText(old, "Technical log"));
    expect(state.open).toBe(true);
    const replacement = render();
    expect(replacement.open).toBe(true);
    old.open = false;
    old.dispatchEvent(new Event("toggle"));
    expect(state.open).toBe(true);
    fireEvent.click(getByText(replacement, "Technical log"));
    expect(state.open).toBe(false);
    expect(render().open).toBe(false);
  });

  it("preserves independent category collapse through choices and search with framed human-readable headings", async () => {
    const raw = await new FixtureBackend().evaluateBuild({ platform: "windows", features: {}, inputs: {} });
    const evaluation = { ...raw, view: {
      categories: ["core", "bg1-content"],
      controls: raw.view.controls.map((control, index) => ({ ...control, category: index === 0 ? "core" : "bg1-content" })),
    } };
    const view: SetupViewState = { search: "", category: "", advancedOpen: true };
    const root = document.createElement("div");
    document.body.append(root);
    const render = () => { root.replaceChildren(setupScreen(evaluation, () => {}, () => {}, () => {}, view)); };
    render();
    fireEvent.click(getByText(root, "Essential setup", { selector: ".category-title" }));
    expect(view.categoryOpen?.core).toBe(false);
    expect(root.querySelector<HTMLDetailsElement>('[data-category="bg1-content"]')?.open).toBe(true);
    render();
    const core = root.querySelector<HTMLDetailsElement>('[data-category="core"]')!;
    expect(core.open).toBe(false);
    expect(core.querySelector("summary.category-heading")).toBeTruthy();
    expect(root.querySelector("legend")).toBeNull();
    const search = getByRole(root, "searchbox") as HTMLInputElement;
    fireEvent.input(search, { target: { value: "conversation" } });
    expect(core.hidden).toBe(true);
    fireEvent.input(search, { target: { value: "" } });
    expect(core.hidden).toBe(false);
    expect(core.open).toBe(false);
    fireEvent.click(getByText(root, "Baldur’s Gate: content", { selector: ".category-title" }));
    render();
    expect(root.querySelector<HTMLDetailsElement>('[data-category="bg1-content"]')?.open).toBe(false);
  });

  it("lets the grid size the scrolling area instead of giving it another full-height box", () => {
    const screen = css.match(/\.screen\s*\{([^}]+)\}/)?.[1] ?? "";
    expect(screen).toContain("min-height: 0");
    expect(screen).toContain("overflow-y: auto");
    expect(screen).not.toMatch(/(?:^|\s)height:\s*100%/);
    expect(css).not.toContain("overflow-y: scroll");
  });
});

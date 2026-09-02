import type { Route } from "../contracts";

export function element<K extends keyof HTMLElementTagNameMap>(tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

export function actionButton(label: string, action: () => void | Promise<void>, variant = "primary"): HTMLButtonElement {
  const control = element("button", `button ${variant}`, label);
  control.type = "button";
  control.addEventListener("click", () => void action());
  return control;
}

export function screenIntro(eyebrow: string, title: string, lede: string): HTMLElement {
  const header = element("header", "screen-intro");
  header.append(element("p", "eyebrow", eyebrow));
  const heading = element("h1", undefined, title);
  heading.tabIndex = -1;
  header.append(heading, element("p", "lede", lede));
  return header;
}

export function screenActions(back: (() => void) | null, next?: HTMLButtonElement): HTMLElement {
  const actions = element("footer", "screen-actions");
  if (back) actions.append(actionButton("Back", back, "quiet"));
  if (next) actions.append(next);
  return actions;
}

export function createAppShell(route: Route, content: HTMLElement, navigate: (route: Route) => void | Promise<void>): HTMLElement {
  const shell = element("div", "app-shell");
  const sidebar = element("aside", "sidebar");
  const brand = element("div", "brand");
  const mark = element("span", "brand-mark", "C");
  mark.setAttribute("aria-hidden", "true");
  const copy = element("div", "brand-copy");
  copy.append(element("strong", undefined, "Campaign Builder"), element("small", undefined, "Fixture alpha"));
  brand.append(mark, copy);
  const nav = element("nav", "nav-list");
  nav.setAttribute("aria-label", "Primary");
  for (const item of [
    { route: "home" as const, label: "Campaigns" },
    { route: "welcome" as const, label: "New campaign" },
    { route: "updates" as const, label: "Updates" },
  ]) {
    const button = actionButton(item.label, () => navigate(item.route), "nav-button");
    if (route === item.route || (item.route === "welcome" && !["home", "updates"].includes(route))) button.setAttribute("aria-current", "page");
    nav.append(button);
  }
  sidebar.append(brand, nav);
  const workspace = element("div", "workspace");
  const topbar = element("header", "topbar");
  topbar.append(element("p", "topbar-kicker", "Local fixture mode"), element("p", undefined, "No game files are read or written"));
  const main = element("main", "screen");
  main.append(content);
  workspace.append(topbar, main);
  shell.append(sidebar, workspace);
  return shell;
}

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

export function createAppShell(
  route: Route,
  content: HTMLElement,
  navigate: (route: Route) => void | Promise<void>,
  mode: "fixture" | "native" = "fixture",
): HTMLElement {
  const shell = element("div", "app-shell");
  shell.dataset.mode = mode;
  const header = element("header", "app-header");
  const brand = element("div", "brand");
  const mark = element("span", "brand-mark", "C");
  mark.setAttribute("aria-hidden", "true");
  const copy = element("div", "brand-copy");
  copy.append(element("strong", undefined, "Chriz Easy BG"), element("small", undefined, "0.1 Alpha"));
  brand.append(mark, copy);
  const nav = element("nav", "header-actions");
  nav.setAttribute("aria-label", "CEBG");
  for (const item of [
    { route: "home" as const, label: "My installs" },
    { route: "updates" as const, label: "Updates" },
  ]) {
    const button = actionButton(item.label, () => navigate(item.route), "header-button");
    if (route === item.route) button.setAttribute("aria-current", "page");
    nav.append(button);
  }
  header.append(brand, nav);
  const main = element("main", "screen");
  main.append(content);
  shell.append(header, main);
  return shell;
}

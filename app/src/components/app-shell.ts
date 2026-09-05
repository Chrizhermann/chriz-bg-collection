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
  if (eyebrow) header.append(element("p", "eyebrow", eyebrow));
  const heading = element("h1", undefined, title);
  heading.tabIndex = -1;
  header.append(heading);
  if (lede) header.append(element("p", "lede", lede));
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
  back?: () => void | Promise<void>,
  version = "0.1.0-alpha.1",
): HTMLElement {
  const shell = element("div", "app-shell");
  shell.dataset.mode = mode;
  shell.dataset.route = route;
  const header = element("header", "app-header");
  const brand = element("div", "brand");
  const mark = element("span", "brand-mark", "C");
  mark.setAttribute("aria-hidden", "true");
  const copy = element("div", "brand-copy");
  copy.append(element("strong", undefined, "Chriz Easy BG"), element("small", undefined, version.replace(/-alpha\.?/i, " Alpha ")));
  brand.append(mark, copy);
  const nav = element("nav", "header-actions");
  nav.setAttribute("aria-label", "CEBG");
  for (const item of [
    { route: "home" as const, label: "My installs" },
    { route: "updates" as const, label: "Updates" },
  ]) {
    const button = actionButton(item.label, () => navigate(item.route), "header-button");
    if (item.route === "updates") {
      button.dataset.action = "updates";
      button.setAttribute("aria-label", "Updates");
      const container = element("span", "update-nav-item");
      container.append(button);
      if (route === item.route) button.setAttribute("aria-current", "page");
      nav.append(container);
      continue;
    }
    if (route === item.route) button.setAttribute("aria-current", "page");
    nav.append(button);
  }
  header.append(brand, nav);
  const main = element("main", "screen");
  if (back !== undefined) {
    const navigation = element("div", "screen-navigation");
    navigation.append(actionButton("Back", back, "quiet compact"));
    main.append(navigation);
  }
  main.append(content);
  shell.append(header, main);
  return shell;
}

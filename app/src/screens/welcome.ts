import { actionButton, element, screenActions, screenIntro } from "../components/app-shell";

export function welcomeScreen(begin: () => void | Promise<void>): HTMLElement {
  const page = element("div", "screen-stack welcome-screen");
  page.append(screenIntro("A guided EET campaign", "Welcome", "Build a reviewed collection from the Baldur's Gate games you already own."));
  const promise = element("section", "safety-promise card");
  promise.append(
    element("h2", undefined, "Your originals stay untouched"),
    element("p", undefined, "The campaign is built in a separate copy, with its own progress, saves, and immutable receipt."),
  );
  page.append(promise, element("p", "requirement", "You need clean supported copies of BG:EE with Siege of Dragonspear and BGII:EE. Detection is read-only."));
  page.append(screenActions(null, actionButton("Begin setup", begin)));
  return page;
}

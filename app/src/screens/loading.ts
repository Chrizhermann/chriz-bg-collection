import { BackendCommandError } from "../backend";
import { actionButton, element } from "../components/app-shell";
import { errorDetails } from "../components/error-details";
import { flavor } from "../flavor";

const messages = {
  startup: [`Getting ${flavor.productName} ready…`, "Checking your setup and saved installations."],
  games: [flavor.text.lookingForGames, "Checking your installed games. This can take a minute or longer on slower drives."],
  setup: ["Preparing your recommended setup…", flavor.text.preparingSetupDetail],
} as const;

function startupLayout(): HTMLElement {
  const main = element("main", "startup-screen");
  const brand = element("div", "brand startup-brand");
  const mark = element("span", "brand-mark", "C");
  mark.setAttribute("aria-hidden", "true");
  brand.append(mark, element("h1", undefined, flavor.productName));
  main.append(brand);
  return main;
}

export function loadingScreen(stage: keyof typeof messages): HTMLElement {
  const main = startupLayout();
  const status = element("section", "startup-status");
  status.setAttribute("role", "status");
  status.setAttribute("aria-live", "polite");
  status.setAttribute("aria-atomic", "true");
  const spinner = element("span", "loading-spinner");
  spinner.setAttribute("aria-hidden", "true");
  const [title, detail] = messages[stage];
  status.append(spinner, element("h2", undefined, title), element("p", undefined, detail));
  main.append(status);
  return main;
}

export function startupFailureScreen(error: unknown, retry: () => void): HTMLElement {
  const main = startupLayout();
  const alert = element("section", "startup-status");
  alert.setAttribute("role", "alert");
  const message = error instanceof Error ? error.message : `${flavor.shortName} could not finish its startup checks.`;
  alert.append(element("h2", undefined, `${flavor.shortName} couldn't finish getting ready`), element("p", undefined, message));
  alert.append(element("p", undefined, error instanceof BackendCommandError
    ? error.recoveryAction : "Check that your game drives are connected, then try again."));
  if (error instanceof BackendCommandError && error.technicalDetail.trim()) {
    alert.append(errorDetails(error.technicalDetail));
  }
  alert.append(actionButton("Try again", retry));
  main.append(alert);
  return main;
}

import type { Backend } from "./backend";
import { initialState } from "./state";

export async function mountApp(root: HTMLElement, backend: Backend): Promise<void> {
  const state = initialState();
  const page = document.createElement("main");
  page.className = "welcome-shell";
  page.setAttribute("aria-labelledby", "welcome-title");

  const product = document.createElement("p");
  product.className = "product-name";
  product.textContent = "Chriz's BG Collection";

  const heading = document.createElement("h1");
  heading.id = "welcome-title";
  heading.textContent = state.route === "welcome" ? "Welcome" : "Build";

  const introduction = document.createElement("p");
  introduction.className = "introduction";
  introduction.textContent =
    "Build a guided EET campaign from the Baldur's Gate games you already own.";

  const promise = document.createElement("div");
  promise.className = "safety-promise";

  const promiseHeading = document.createElement("h2");
  promiseHeading.textContent = "Your originals stay untouched";

  const promiseBody = document.createElement("p");
  promiseBody.textContent =
    "The collection is built in a separate copy, with its own progress and saves.";

  const requirement = document.createElement("p");
  requirement.className = "requirement";
  requirement.textContent =
    "You will need Baldur's Gate: Enhanced Edition with Siege of Dragonspear and Baldur's Gate II: Enhanced Edition.";

  promise.append(promiseHeading, promiseBody);
  page.append(product, heading, introduction, promise, requirement);

  if (import.meta.env.DEV) {
    const status = await backend.getStatus();
    const statusLine = document.createElement("p");
    statusLine.className = "developer-status";
    statusLine.textContent = `Developer backend: ${status.mode}; engine ${status.engineVersion}; recipe ${status.recipeVersion ?? "not loaded"}`;
    page.append(statusLine);
  }

  root.replaceChildren(page);
}

import "./styles.css";

import { mountApp } from "./app";
import { FixtureBackend } from "./backend";

const root = document.querySelector<HTMLElement>("#app");

if (root === null) {
  throw new Error("Installer root element is missing");
}

void mountApp(root, new FixtureBackend()).catch((error: unknown) => {
  root.textContent =
    error instanceof Error
      ? `The installer shell could not start: ${error.message}`
      : "The installer shell could not start.";
});

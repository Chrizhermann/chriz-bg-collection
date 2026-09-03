import "./styles.css";

import { mountApp } from "./app";
import { BackendCommandError, NativeBackend } from "./backend";

const root = document.querySelector<HTMLElement>("#app");

if (root === null) {
  throw new Error("Installer root element is missing");
}

void mountApp(root, new NativeBackend()).catch((error: unknown) => {
  const message = error instanceof Error ? error.message : "The installer shell could not start.";
  const recovery = error instanceof BackendCommandError ? ` ${error.recoveryAction}` : "";
  root.textContent = `The installer shell could not start: ${message}${recovery}`;
});

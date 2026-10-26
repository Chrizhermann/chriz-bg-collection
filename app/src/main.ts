import { isTauri } from "@tauri-apps/api/core";
import { mountApp } from "./app";
import { FixtureBackend, NativeBackend } from "./backend";

const root = document.querySelector<HTMLElement>("#app");

if (root === null) {
  throw new Error("Installer root element is missing");
}

// Only the dev server opened in a plain browser previews the fixture; every build and every
// Tauri window talks to the native backend.
const backend = import.meta.env.DEV && !isTauri() ? new FixtureBackend() : new NativeBackend();

void mountApp(root, backend).catch(() => {
  // mountApp replaces the loading state with an error and a Try again action.
});

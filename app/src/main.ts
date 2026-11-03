import { mountApp } from "./app";
import { NativeBackend } from "./backend";

const root = document.querySelector<HTMLElement>("#app");

if (root === null) {
  throw new Error("Installer root element is missing");
}

void mountApp(root, new NativeBackend()).catch(() => {
  // mountApp replaces the loading state with an error and a Try again action.
});

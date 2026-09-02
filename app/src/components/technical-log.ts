import { actionButton, element } from "./app-shell";

const MAX_VISIBLE_LINES = 200;

export interface TechnicalLogState {
  readonly paused: boolean;
  readonly open: boolean;
}

export function technicalLog(
  lines: readonly string[],
  state: TechnicalLogState = { paused: false, open: false },
  onStateChange: (state: TechnicalLogState) => void = () => undefined,
): HTMLElement {
  const details = element("details", "technical-log");
  details.open = state.open;
  const summary = element("summary", undefined, "Technical log");
  const toolbar = element("div", "log-toolbar");
  let paused = state.paused;
  const pause = actionButton("Pause auto-scroll", () => {
    paused = !paused;
    pause.textContent = paused ? "Resume auto-scroll" : "Pause auto-scroll";
    pause.setAttribute("aria-pressed", String(paused));
    onStateChange({ paused, open: details.open });
  }, "quiet");
  pause.textContent = paused ? "Resume auto-scroll" : "Pause auto-scroll";
  pause.setAttribute("aria-pressed", String(paused));
  toolbar.append(pause, element("span", undefined, "Full history is retained by the backend."));
  const output = element("pre");
  output.tabIndex = 0;
  output.textContent = lines.slice(-MAX_VISIBLE_LINES).join("\n");
  details.append(summary, toolbar, output);
  details.addEventListener("toggle", () => onStateChange({ paused, open: details.open }));
  if (!paused) queueMicrotask(() => { output.scrollTop = output.scrollHeight; });
  return details;
}

import { element } from "./app-shell";

export type StatusTone = "ok" | "warning" | "danger" | "neutral";

export function statusCard(title: string, detail: string, tone: StatusTone = "neutral"): HTMLElement {
  const card = element("article", `card status-card ${tone}`);
  card.append(element("h2", undefined, title), element("p", undefined, detail));
  return card;
}

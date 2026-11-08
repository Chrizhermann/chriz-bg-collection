import type { ConfigEnv, IndexHtmlTransformContext, Plugin, UserConfig } from "vite";
import { describe, expect, it } from "vitest";

import indexHtml from "../index.html?raw";
import config, { iwdIndexHtml } from "../vite.config";

function resolved(mode: string): UserConfig {
  const env: ConfigEnv = { mode, command: "build", isSsrBuild: false, isPreview: false };
  return (config as (env: ConfigEnv) => UserConfig)(env);
}

async function transform(plugin: Plugin, html: string): Promise<string> {
  const hook = plugin.transformIndexHtml;
  const handler = typeof hook === "function" ? hook : hook?.handler;
  if (handler === undefined) throw new Error("The plugin has no transformIndexHtml handler.");
  const result = await handler.call({} as never, html, { path: "/index.html", filename: "index.html" } as IndexHtmlTransformContext);
  if (typeof result !== "string") throw new Error("The IWD page transform must return the page text.");
  return result;
}

describe("index.html per build flavor", () => {
  it("rewrites every product word of the page in mode iwd", async () => {
    const plugins = resolved("iwd").plugins as Plugin[];
    expect(plugins.map(plugin => plugin.name)).toEqual(["chriz-easy-iwd-index-html"]);
    const page = await transform(plugins[0], indexHtml);
    expect(page).not.toMatch(/Baldur|Chriz Easy BG|CEBG|\bBG\b/);
    expect(page).toContain("<title>Chriz Easy IWD</title>");
    expect(page).toContain('content="Install and launch Chriz Easy IWD."');
    expect(page).toContain("<h1>Chriz Easy IWD</h1>");
    expect(page).toContain("<h2>Getting Chriz Easy IWD ready…</h2>");
    expect(indexHtml).toContain('<link rel="stylesheet" href="/src/themes/bg.css" />');
    expect(page).toContain('<link rel="stylesheet" href="/src/themes/iwd.css" />');
    expect(page).not.toContain("/src/themes/bg.css");
    // Only the product words and the theme change.
    expect(page.replaceAll("Chriz Easy IWD", "Chriz Easy BG").replace("/src/themes/iwd.css", "/src/themes/bg.css")).toBe(indexHtml);
  });

  it("leaves the BG page untouched in every other mode", () => {
    for (const mode of ["production", "development", "test"]) {
      expect(resolved(mode).plugins).toEqual([]);
    }
  });

  it("refuses a page whose BG words it cannot account for", async () => {
    await expect(transform(iwdIndexHtml(), "<title>Something else</title>")).rejects.toThrow("no longer contains");
    await expect(transform(iwdIndexHtml(), "<title>Chriz Easy BG</title>")).rejects.toThrow('no longer contains "/src/themes/bg.css"');
    const theme = '<link rel="stylesheet" href="/src/themes/bg.css" />';
    await expect(transform(iwdIndexHtml(), `${theme}<title>Chriz Easy BG</title><p>CEBG</p>`)).rejects.toThrow('still contains "CEBG"');
  });
});

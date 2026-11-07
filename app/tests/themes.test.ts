import { describe, expect, it } from "vitest";

import { FLAVORS, themeStylesheet, type FlavorId } from "../src/flavors";
import styles from "../src/styles.css?raw";
import gameInterface from "../src/game-interface.css?raw";
import featureInputs from "../src/components/feature-inputs.css?raw";
import bgTheme from "../src/themes/bg.css?raw";
import iwdTheme from "../src/themes/iwd.css?raw";
import tauriConfig from "../src-tauri/tauri.conf.json?raw";

const themes: Record<FlavorId, string> = { bg: bgTheme, iwd: iwdTheme };
const themeFiles = import.meta.glob("../src/themes/*.css", { query: "?raw", import: "default" });
const assetFiles = import.meta.glob("../src/themes/**/*.{woff2,svg}", { query: "?url", import: "default", eager: true });
const licences = import.meta.glob<string>("../src/themes/fonts/**/OFL.txt", { query: "?raw", import: "default", eager: true });

/** Custom properties a theme defines on `:root`, with their values. */
function rootTokens(css: string): Map<string, string> {
  const tokens = new Map<string, string>();
  for (const block of css.matchAll(/:root\s*\{([^}]*)\}/g)) {
    for (const match of block[1].matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) tokens.set(match[1], match[2].trim());
  }
  return tokens;
}

function usedTokens(...sheets: string[]): Set<string> {
  return new Set(sheets.flatMap(sheet => [...sheet.matchAll(/var\(\s*(--[\w-]+)/g)].map(match => match[1])));
}

function luminance(hex: string): number {
  const channels = [1, 3, 5].map(index => parseInt(hex.slice(index, index + 2), 16) / 255);
  const [r, g, b] = channels.map(c => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

// Text and the surfaces it sits on: cards and panels (--charcoal) and the page and inputs (--iron).
const TEXT_PAIRS = [
  ["--vellum", "--charcoal"], ["--vellum", "--iron"],
  ["--vellum-dim", "--charcoal"], ["--vellum-dim", "--iron"],
  ["--brass", "--charcoal"], ["--brass", "--iron"],
  ["--teal", "--charcoal"], ["--teal", "--iron"],
  ["--ember", "--charcoal"], ["--ember", "--iron"],
  ["--focus", "--charcoal"], ["--focus", "--iron"],
  ["--on-accent", "--brass"], ["--on-accent", "--focus"],
  ["--vellum", "--danger-surface"],
  ["--log-text", "--log-background"],
] as const;
// Focus rings and control borders against their surroundings (WCAG 1.4.11).
const UI_PAIRS = [
  ["--focus", "--charcoal"], ["--focus", "--iron"],
  ["--slate-light", "--charcoal"], ["--slate-light", "--iron"],
] as const;

describe("app themes", () => {
  it("stays within the window's policy for images and fonts", () => {
    const csp: string = JSON.parse(tauriConfig).app.security.csp;
    const directives = new Map(csp.split(";").map(part => part.trim().split(/\s+/)).map(([name, ...values]) => [name, values]));
    for (const directive of ["img-src", "font-src"]) {
      const sources = directives.get(directive) ?? directives.get("default-src") ?? [];
      expect(sources, directive).toContain("'self'");
      expect(sources, directive).not.toContain("data:");
    }
    for (const css of [styles, gameInterface, featureInputs, bgTheme, iwdTheme]) {
      expect(css).not.toMatch(/url\(\s*["']?data:/);
    }
  });

  it("gives every flavor a theme file under the name index.html links", () => {
    for (const id of Object.keys(FLAVORS) as FlavorId[]) {
      expect(Object.keys(themeFiles)).toContain(`..${themeStylesheet(id)}`);
    }
  });

  for (const [id, css] of Object.entries(themes) as [FlavorId, string][]) {
    describe(id, () => {
      const tokens = rootTokens(css);

      it("defines every token the shared stylesheets and the theme use", () => {
        const shared = rootTokens(styles);
        for (const token of usedTokens(styles, gameInterface, featureInputs, css)) {
          expect(tokens.has(token) || shared.has(token), `${id} theme lacks ${token}`).toBe(true);
        }
        expect(css).toMatch(/color-scheme:\s*(light|dark)/);
      });

      it("keeps text readable and focus rings and control borders visible", () => {
        const hex = (token: string): string => {
          const value = tokens.get(token) ?? "";
          expect(value, `${id} ${token} must be a plain #rrggbb color`).toMatch(/^#[0-9a-f]{6}$/i);
          return value;
        };
        for (const [text, ground] of TEXT_PAIRS) {
          expect(contrast(hex(text), hex(ground)), `${id}: ${text} on ${ground}`).toBeGreaterThanOrEqual(4.5);
        }
        for (const [mark, ground] of UI_PAIRS) {
          expect(contrast(hex(mark), hex(ground)), `${id}: ${mark} against ${ground}`).toBeGreaterThanOrEqual(3);
        }
      });

      it("loads its fonts and ornaments from files bundled with the app", () => {
        // Offline, and within the window's content security policy: no network, no data: URLs.
        const urls = [...css.matchAll(/url\(\s*["']?([^"')]+)["']?\s*\)/g)].map(match => match[1]);
        expect(urls.length).toBeGreaterThan(0);
        for (const url of urls) {
          expect(url, `${id} theme url ${url} must be a file beside the theme`).toMatch(/^\.\/(fonts|images)\//);
          expect(Object.keys(assetFiles), `${id} theme file ${url}`).toContain(`../src/themes/${url.slice(2)}`);
        }
        expect(css).not.toMatch(/@import/);
      });

      it("keeps each font's licence beside it", () => {
        const fonts = [...css.matchAll(/url\("?(\.\/fonts\/[^")]+)"?\)/g)].map(match => match[1]);
        expect(fonts.length).toBeGreaterThan(0);
        for (const url of fonts) {
          const licence = licences[`../src/themes/${url.slice(2, url.lastIndexOf("/"))}/OFL.txt`];
          expect(licence, `${id} font ${url} needs OFL.txt beside it`).toMatch(/^Copyright \(c\) \d{4} /);
          expect(licence).toContain("SIL OPEN FONT LICENSE Version 1.1");
        }
        const families = [...css.matchAll(/font-family:\s*"([^"]+)"/g)].map(match => match[1]);
        expect(tokens.get("--font-display")).toMatch(new RegExp(`^"${families[0]}",`));
      });
    });
  }
});

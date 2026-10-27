import type { Plugin } from "vite";
import { defineConfig } from "vitest/config";
import { indexHtmlReplacements } from "./src/flavors.ts";

/** Words of the Baldur's Gate app that must not reach the page of another flavor. */
const BG_INDEX_WORDS = /Baldur|Chriz Easy BG|CEBG/;

/**
 * Rewrites the product words in `index.html` for the Chriz Easy IWD build (`--mode iwd`). The BG
 * build gets no plugin, so its page stays byte for byte as written.
 */
export function iwdIndexHtml(): Plugin {
  return {
    name: "chriz-easy-iwd-index-html",
    transformIndexHtml: {
      order: "pre",
      handler(html) {
        let result = html;
        for (const [from, to] of indexHtmlReplacements("iwd")) {
          if (!result.includes(from)) throw new Error(`index.html no longer contains "${from}"; update the IWD replacements.`);
          result = result.replaceAll(from, to);
        }
        const leftover = BG_INDEX_WORDS.exec(result);
        if (leftover !== null) throw new Error(`index.html still contains "${leftover[0]}" after the IWD replacements.`);
        return result;
      },
    },
  };
}

export default defineConfig(({ mode }) => ({
  clearScreen: false,
  plugins: mode === "iwd" ? [iwdIndexHtml()] : [],
  // The app's content security policy loads images and fonts only from 'self', never from
  // data: URLs, so the themes' SVG ornaments and fonts must stay files.
  build: {
    assetsInlineLimit: 0,
  },
  test: {
    css: true,
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));

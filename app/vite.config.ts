import { defineConfig } from "vitest/config";

export default defineConfig({
  clearScreen: false,
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
});

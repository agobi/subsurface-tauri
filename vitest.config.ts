// AI-generated (Claude)
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: path.resolve(import.meta.dirname, "src/lib"),
      "@tauri-apps/plugin-os": path.resolve(import.meta.dirname, "test/__mocks__/tauri-plugin-os.ts"),
    },
    conditions: ["browser"],
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest-setup.ts"],
    clearMocks: true,
    include: ["test/**/*.test.ts"],
  },
});

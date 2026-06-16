import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'node:path'

// https://vite.dev/config/
// Tauri expects a fixed dev port and quiet output; see https://tauri.app/start/frontend/
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // $lib alias must match vitest.config.ts and tsconfig.app.json so dev/build resolve it too.
  resolve: {
    alias: { $lib: path.resolve(import.meta.dirname, 'src/lib') },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
})

// AI-generated (Claude)
// https://vite.dev/config/
// Tauri expects a fixed dev port and quiet output; see https://tauri.app/start/frontend/
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'node:path'

const visualTestAliases: Record<string, string> = process.env.VITE_VISUAL_TEST
  ? {
      '@tauri-apps/api/core': path.resolve(import.meta.dirname, 'test/visual/tauri-mock.ts'),
      '@tauri-apps/api/event': path.resolve(import.meta.dirname, 'test/visual/tauri-mock.ts'),
      '@tauri-apps/api/window': path.resolve(import.meta.dirname, 'test/visual/tauri-mock.ts'),
      '@tauri-apps/plugin-dialog': path.resolve(import.meta.dirname, 'test/visual/tauri-mock.ts'),
      '@tauri-apps/plugin-os': path.resolve(import.meta.dirname, 'test/visual/tauri-mock.ts'),
      '@tauri-apps/plugin-store': path.resolve(import.meta.dirname, 'test/visual/tauri-mock.ts'),
    }
  : {};

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  resolve: {
    alias: {
      $lib: path.resolve(import.meta.dirname, 'src/lib'),
      ...visualTestAliases,
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(import.meta.dirname, 'index.html'),
        prefs: path.resolve(import.meta.dirname, 'prefs.html'),
        'toolbar-harness': path.resolve(import.meta.dirname, 'toolbar-harness.html'),
        'dc-download-harness': path.resolve(import.meta.dirname, 'dc-download-harness.html'),
      },
    },
  },
})

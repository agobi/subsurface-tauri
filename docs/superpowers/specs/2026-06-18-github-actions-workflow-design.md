# GitHub Actions Workflow Design

**Date:** 2026-06-18
**Project:** subsurface-tauri (Tauri 2.x + Svelte 5 + Rust)

## Goal

Add CI and release automation via two GitHub Actions workflow files:
- `ci.yml` — test on every push to `main` and on pull requests targeting `main`
- `release.yml` — build cross-platform desktop bundles and create a GitHub Release on `v*` tag push

## Workflow 1: CI (`ci.yml`)

**Trigger:** `push` to `main`, `pull_request` targeting `main`

**Runner:** `ubuntu-22.04` (pinned — Tauri 2 recommends 22.04; 24.04 has library name changes that break the webkit dep install)

**Steps:**
1. `actions/checkout@v4`
2. `actions/setup-node@v4` — Node 22, npm cache
3. `npm ci`
4. `npm run check` — Svelte + TypeScript type-check
5. `npm test` — Vitest unit tests
6. `dtolnay/rust-toolchain@stable` — Rust stable
7. Install Linux system libs required by Tauri 2 at compile time:
   `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
8. `swatinem/rust-cache@v2` — Rust dependency cache (keyed on `src-tauri/`)
9. `cargo test` — Rust unit and integration tests (`working-directory: src-tauri`)

CI runs on Linux only. The parser logic is platform-agnostic; per-platform correctness is covered by the release matrix.

## Workflow 2: Release (`release.yml`)

**Trigger:** `push` to tags matching `v*`

**Strategy matrix (`fail-fast: false`):**

| Runner | Artifacts |
|---|---|
| `macos-latest` | `.dmg` (arm64 native) |
| `ubuntu-22.04` | `.AppImage`, `.deb` |
| `windows-latest` | `.msi`, `.exe` |

**Steps (each runner):**
1. `actions/checkout@v4`
2. `actions/setup-node@v4` — Node 22, npm cache
3. `dtolnay/rust-toolchain@stable`
4. Install Linux system libs (Ubuntu only, same set as CI)
5. `swatinem/rust-cache@v2`
6. `npm ci`
7. `tauri-apps/tauri-action@v0` — builds the app, uploads artifacts, creates GitHub Release

**Release settings:**
- `releaseDraft: true` — creates a draft; must be manually published after review
- `GITHUB_TOKEN` (built-in secret) — handles artifact upload and release creation
- Job-level `permissions: contents: write` required for release creation

## Out of Scope

- **macOS notarization** — requires Apple Developer account; add later via `tauri-apps/tauri-action` `with:` params
- **Windows SmartScreen signing** — add later via secrets when needed
- **macOS universal binary** — `macos-latest` builds arm64 natively; x86_64 target can be added later
- **Auto-updater signing** — Tauri updater plugin not in use yet

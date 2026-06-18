# Subsurface — Tauri Prototype

A cross-platform desktop dive-log viewer built with [Tauri 2](https://tauri.app) (Rust backend) and [Svelte 5](https://svelte.dev) (TypeScript frontend).

This is a prototype that reads the same git-based logbook format used by [Subsurface](https://subsurface-divelog.org). It lives alongside the existing Qt/C++ codebase and has no build dependency on it.

## Prerequisites

- [Node.js](https://nodejs.org) 24+
- [Rust](https://rustup.rs) (stable)
- Linux: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

## Development

```bash
# Install JS dependencies
npm install

# Frontend dev server only
npm run dev

# Full Tauri dev app (Rust + Vite)
npm run tauri dev
```

## Testing

```bash
# Vitest unit tests (frontend)
npm test

# TypeScript + Svelte type-check
npm run check

# Rust unit + integration tests
cd src-tauri && cargo test

# Rust lint
cd src-tauri && cargo clippy -- -D warnings
```

## Building

```bash
npm run tauri build
```

Produces platform-native bundles (`.dmg`, `.AppImage`, `.msi`) in `src-tauri/target/release/bundle/`.

## Architecture

```
src/                        Svelte 5 + TypeScript frontend
  lib/
    stores/app.svelte.ts    Single AppStore ($state runes)
    types.ts                Shared type contract — keep in sync with types.rs
    components/             TitleBar, MenuBar, QuadrantGrid, …
src-tauri/src/              Rust backend
  types.rs                  Serde types mirroring types.ts
  ssrf_git/                 Pure-Rust logbook parser
  lib.rs                    Tauri IPC commands
test/                       Vitest tests (mirrors src/)
  fixtures/git-tree/        Golden fixture for Rust integration tests
```

## License

GPL-2.0 — see [LICENSE](LICENSE).

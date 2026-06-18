# GitHub Actions Workflows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CI and release automation via two GitHub Actions workflow files.

**Architecture:** Two independent YAML workflow files under `.github/workflows/`. `ci.yml` runs the full test suite on every push to `main` and on PRs; `release.yml` builds cross-platform desktop bundles and creates a draft GitHub Release on `v*` tag pushes using a 3-runner matrix.

**Tech Stack:** GitHub Actions, Tauri 2.x, Svelte 5, Rust (stable), Node 22.

## Global Constraints

- Runner for CI: `ubuntu-22.04` (pinned — Tauri 2 WebKit dep breaks on 24.04)
- Runner for release matrix: `macos-latest`, `ubuntu-22.04`, `windows-latest`
- Node version: 22, npm cache enabled
- Rust toolchain: stable via `dtolnay/rust-toolchain@stable`
- Linux system deps (compile-time only): `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
- Release must be a draft (`releaseDraft: true`) — never auto-published
- `permissions: contents: write` required at job level for release creation
- All AI-generated files include `# AI-generated (Claude)` comment at top

---

### Task 1: CI Workflow (`ci.yml`)

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: nothing (standalone workflow)
- Produces: GitHub Actions CI that runs on push/PR to `main`

- [ ] **Step 1: Create the workflow directory**

```bash
mkdir -p .github/workflows
```

- [ ] **Step 2: Write the CI workflow**

Create `.github/workflows/ci.yml` with this exact content:

```yaml
# AI-generated (Claude)
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-22.04

    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm

      - name: Install npm dependencies
        run: npm ci

      - name: Type-check (Svelte + TypeScript)
        run: npm run check

      - name: Vitest unit tests
        run: npm test

      - uses: dtolnay/rust-toolchain@stable

      - name: Install Linux system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - uses: swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Cargo tests
        run: cargo test
        working-directory: src-tauri
```

- [ ] **Step 3: Validate with actionlint**

Install actionlint if not present:
```bash
brew install actionlint   # macOS
# or: go install github.com/rhysd/actionlint/cmd/actionlint@latest
```

Run:
```bash
actionlint .github/workflows/ci.yml
```

Expected output: no errors, zero exit code. Fix any reported issues before committing.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions CI workflow"
```

---

### Task 2: Release Workflow (`release.yml`)

**Files:**
- Create: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: nothing (standalone workflow, triggered by `v*` tags)
- Produces: GitHub Actions workflow that creates a draft release with `.dmg`, `.AppImage`, `.deb`, `.msi`, `.exe` artifacts

- [ ] **Step 1: Write the release workflow**

Create `.github/workflows/release.yml` with this exact content:

```yaml
# AI-generated (Claude)
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  release:
    strategy:
      fail-fast: false
      matrix:
        runner: [macos-latest, ubuntu-22.04, windows-latest]

    runs-on: ${{ matrix.runner }}

    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm

      - uses: dtolnay/rust-toolchain@stable

      - name: Install Linux system dependencies
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - uses: swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install npm dependencies
        run: npm ci

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: ${{ github.ref_name }}
          releaseDraft: true
```

- [ ] **Step 2: Validate with actionlint**

```bash
actionlint .github/workflows/release.yml
```

Expected output: no errors, zero exit code. Fix any reported issues before committing.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow"
```

---

## Self-Review

**Spec coverage:**
- CI triggers (push to main, PR to main): ✓ Task 1
- CI runner ubuntu-22.04 pinned: ✓ Task 1
- Node 22 + npm cache: ✓ Both tasks
- `npm ci` → `npm run check` → `npm test`: ✓ Task 1
- Rust stable via dtolnay: ✓ Both tasks
- Linux sys deps: ✓ Both tasks (CI always-on; release conditional on `runner.os == 'Linux'`)
- `swatinem/rust-cache@v2` keyed on `src-tauri/`: ✓ Both tasks
- `cargo test` in `src-tauri/`: ✓ Task 1
- Release trigger `v*` tags: ✓ Task 2
- Matrix `fail-fast: false`: ✓ Task 2
- macOS/Ubuntu/Windows runners: ✓ Task 2
- `tauri-apps/tauri-action@v0`: ✓ Task 2
- `releaseDraft: true`: ✓ Task 2
- `permissions: contents: write`: ✓ Task 2
- `GITHUB_TOKEN` from built-in secret: ✓ Task 2

**Out-of-scope items confirmed absent:** notarization, SmartScreen signing, universal binary, updater signing.

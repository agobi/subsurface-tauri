# Future Projects

## High Priority

- [ ] **Android Phase 1: Pixel 7 device emulation visual regression** — add `devices['Pixel 7']` (412×915) as a second Playwright project in `playwright.config.ts`; generates 8 additional baselines (same test files as desktop, `-android` suffix). Good for catching CSS layout bugs at mobile width. Baselines must be generated on Linux/Chromium. See `docs/superpowers/plans/2026-06-19-visual-regression.md` Task 5 for the exact spec.

- [ ] **Phase 2: Real Android WebView visual regression via Playwright + ADB** — once Tauri Android is set up, enable WebView debugging in the Android build and use Playwright's `playwright.android` API to attach to the running WebView over ADB. Same `mockIPC` fixture and same test files as Phase 1; adds a new project entry in `playwright.config.ts`. Requires: Android emulator or device in CI (e.g. `reactivecircus/android-emulator-runner` GitHub Action), Tauri Android build, `android:debuggable="true"` in the manifest. See Phase 1 visual regression spec for context.

- [ ] **Tauri Android CI build** — add a `android` job to `.github/workflows/ci.yml` that installs the Android SDK/NDK, sets up Java, adds Rust Android targets (`aarch64-linux-android` etc.), and runs `cargo tauri android build --debug`. Add to `ci-success` needs. Coordinate with the release workflow to upload a debug APK artifact on main-branch pushes.

- [ ] **Release workflow: Android APK/AAB** — extend the existing release workflow to include a signed Android build (`cargo tauri android build --release`) with keystore secrets configured in GitHub. Produce an APK/AAB artifact for distribution alongside the desktop binaries.

## Backlog

- [ ] Imperial units support in `parse_divecomputer.rs` sample parsing (currently metric-only: m, bar, °C — ft, psi, °F sample lines are silently ignored)
- [ ] Surface unreadable Dive/Divecomputer parse errors instead of silently dropping dives (`ssrf_git/mod.rs:69/73`)

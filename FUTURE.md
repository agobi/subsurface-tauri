# Future Projects

## High Priority

- [ ] **Release workflow: Android APK/AAB** — extend the existing release workflow to include a signed Android build (`cargo tauri android build --release`) with keystore secrets configured in GitHub. Produce an APK/AAB artifact for distribution alongside the desktop binaries.

## Backlog

- [ ] Imperial units support in `parse_divecomputer.rs` sample parsing (currently metric-only: m, bar, °C — ft, psi, °F sample lines are silently ignored)
- [ ] Surface unreadable Dive/Divecomputer parse errors instead of silently dropping dives (`ssrf_git/mod.rs:69/73`)

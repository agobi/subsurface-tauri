# Subsurface — Tauri Prototype

A dive-log viewer for [Subsurface](https://subsurface-divelog.org)'s git-based logbook format, built with [Tauri 2](https://tauri.app) and [Svelte 5](https://svelte.dev).

> **Early prototype.** Expect missing features and rough edges. The existing [Subsurface Qt app](https://subsurface-divelog.org) remains the full-featured reference implementation.

## Download

Get the latest pre-release from the [Releases](../../releases) page:

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `subsurface-prototype_*_aarch64.dmg` |
| Linux (Debian/Ubuntu) | `subsurface-prototype_*_amd64.deb` |
| Linux (AppImage) | `subsurface-prototype_*_amd64.AppImage` |
| Linux (RPM) | `subsurface-prototype_*_x86_64.rpm` |
| Windows | `subsurface-prototype_*_x64-setup.exe` |
| Android (debug APK) | `app-universal-debug.apk` |

## What it does

- **Opens Subsurface logbooks** — local git-based logbooks or cloud notebooks
- **Dive list** — sortable by any column, dives grouped into trips
- **Customizable columns** — show or hide: date, rating, depth, duration, buddy, guide, country, location, water temp, suit, gas mix, SAC, tags, notes, dive mode, weight
- **Dive profile** — depth-over-time chart from dive computer data
- **Map** — dive site locations
- **Desktop + Android** — native app on macOS, Windows, Linux, and Android

## Opening a logbook

On first launch you can:

- **Open Logbook** — point to an existing Subsurface git logbook directory on disk
- **Open Cloud Notebook** — enter your Subsurface cloud credentials
- **New Logbook** — create a fresh empty logbook

The last-used logbook reopens automatically on the next launch.

## License

GPL-2.0 — see [LICENSE](LICENSE).

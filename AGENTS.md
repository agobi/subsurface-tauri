# Agent Instructions — desktop-tauri / Android

## Logbook Format Compatibility

The `00-Subsurface` settings file and every other git logbook file format (dives,
sites, trips) **must stay compatible with the original Qt Subsurface application**
(`core/save-git.cpp` / `core/load-git.cpp` in the Qt/C++ codebase). Do not add
custom fields or app-specific data to these files, even if convenient — anything
Subsurface (Qt) wouldn't understand does not belong there.

Data that is specific to this Tauri app and not portable across machines/platforms
(e.g. a dive computer's BLE connection address — a CoreBluetooth UUID on macOS is a
different value from an Android MAC address for the same physical device) belongs
in the app's local settings store (`tauri-plugin-store`, `settings.json`), never in
the git-tracked logbook. The logbook only ever holds data that is genuinely part of
the dive log and portable across every Subsurface-compatible reader.

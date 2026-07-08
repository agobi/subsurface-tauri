# Agent Instructions — desktop-tauri / Android

## Logbook Format Compatibility

The **entire git logbook format** — every file this app reads or writes under a
logbook root (`00-Subsurface`, dive files, site files, trip files, everything in
`ssrf_git/`) — **must stay compatible with the original Qt Subsurface application**
(`core/save-git.cpp` / `core/load-git.cpp` in the Qt/C++ codebase). Do not add
custom fields, files, or app-specific data anywhere in this tree, even if
convenient — anything Subsurface (Qt) wouldn't read back correctly does not
belong there. This is not limited to settings; it applies to the whole format.

Data that is specific to this Tauri app and not portable across machines/platforms
(e.g. a dive computer's BLE connection address — a CoreBluetooth UUID on macOS is a
different value from an Android MAC address for the same physical device) belongs
in the app's local settings store (`tauri-plugin-store`, `settings.json`), never in
the git-tracked logbook. The logbook only ever holds data that is genuinely part of
the dive log and portable across every Subsurface-compatible reader.

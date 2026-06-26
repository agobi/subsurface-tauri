# Future Projects

## Dive List Columns — Deferred

These columns were scoped out of the initial sorting/columns implementation (2026-06-18)
because they require O(samples) computation across potentially thousands of dives.
Add memoization or move computation to the Rust backend before implementing.

- [ ] **OTU column** — oxygen toxicity units; requires integrating ppO2 over `samples[]`
- [ ] **Max CNS column** — `Math.max(...samples.map(s => s.cns ?? 0))` per dive

## Dive List — Media Column

Photo/video count column. Requires counting files in the `Pictures/` subdirectory
during the git tree walk in `ssrf_git/mod.rs`, plus adding `mediaCount` to `Dive`
type on both Rust and TypeScript sides.

- [ ] **Media column** — count files in `Pictures/` per dive dir during tree walk

## Dive List — Column Reordering

Allow users to drag column headers to reorder them. Requires persisting a `colOrder: ColId[]`
array in `settings.json` alongside `visibleCols`. The column definitions module already has
the right structure; rendering just needs to respect order. Hold off until drag-and-drop
UX patterns are established in the app.

- [x] Persist `colOrder` in `diveList` settings key
- [ ] Drag-handle on column headers (or drag the header itself)
- [ ] Update grid-template-columns dynamically from order

## Lazy Sample Loading / DiveSummary Split

Currently `Logbook` (including all `Sample[]` arrays) is fully serialized to JS on startup.
Samples are ~80% of payload but only needed for the profile view. For large logbooks
(1000 dives × 3000 samples = 3M objects) this wastes significant JS memory.

Proposed split:
- Keep `Logbook` in Rust `State<Mutex<Logbook>>`
- Expose a `DiveSummary` type (all scalar fields, no `samples` or `events`) for the list view
- Add a `get_dive(number)` command that returns the full `Dive` including samples on demand
- Profile panel fetches on selection; list panel works on summaries only

This is an IPC architecture refactor — keep separate from UI feature work.

## Recents Management

The recents list is currently unbounded and entries can only be added, not removed.

- [ ] **Clear recents** — "Clear Menu" item at the bottom of the "Open Recent" submenu that wipes `settings.json["recents"]` and rebuilds the menu
- [ ] **Remove individual entry** — right-click / context menu on a recent item to remove just that entry
- [ ] **Cap list length** — optional: add a preference for max recents count (e.g. 10, 20, unlimited)

## Backlog

- [ ] **Android dive computer download** — currently desktop-only (libdivecomputer FFI + serial/USB/BLE stacks don't compile on Android). Deferred: would require Android-specific transport bindings (android.hardware.usb, Android BLE APIs), separate FFI layer, and UI entry point in place of the menu bar.
- [ ] **Release workflow: signed Android APK/AAB** — debug APKs are already built and uploaded on release (see `release.yml`). Remaining: switch to `--release`, add keystore secrets, and optionally produce an AAB for Play Store. Only needed if Play Store distribution becomes a goal.
- [ ] Imperial units support in `parse_divecomputer.rs` sample parsing (currently metric-only: m, bar, °C — ft, psi, °F sample lines are silently ignored)
- [ ] Surface unreadable Dive/Divecomputer parse errors instead of silently dropping dives (`ssrf_git/mod.rs:69/73`)

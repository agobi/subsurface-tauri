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

## DC Download — Merge Overlapping Dive Segments

Investigated 2026-06-30: short dives (e.g. CCR pre-dive loop checks — a few minutes,
a few meters) reappear as "new" on every download even though the diver already has
the real dive logged. Confirmed via direct comparison (date, time-of-day, duration,
max depth) against the existing logbook entries — these segments are not duplicates
of full dives, they're short overlapping/adjacent segments that the *original* import
process folded away.

Root cause: Qt Subsurface's `merge_imported_dives()` (`core/divelog.cpp:101-130`)
merges any two consecutively-downloaded dives that overlap in time or where one has
zero duration, via `try_to_merge()`. This collapses pre-dive checks into the adjacent
real dive before anything is written to disk, so the short segment's fingerprint is
never persisted standalone.

This Tauri app's download path (`src-tauri/src/dc/device.rs` `dive_cb`, `writer.rs`)
has no equivalent step — every manifest entry libdc delivers becomes its own
`ParsedDive`, unconditionally. Since the short segment was never written standalone
in the original logbook, it has nothing to dedupe against and gets rebuffered as
"new" on every future download, indefinitely.

- [ ] Port equivalent overlap-merge logic (or a simpler near-real-dive heuristic) into
      the download pipeline — likely as a post-processing pass over `new_dives` in
      `run_download` before they're returned to the review step, mirroring
      `merge_imported_dives`'s overlap/zero-duration check and `try_to_merge` semantics.
- [ ] Until implemented: short overlapping segments must be manually discarded in the
      review dialog on each download.

## Backlog

- [ ] **Android dive computer download** — currently desktop-only. Full design deferred until the
      build spike (see below) reports back, but scope decisions already made during brainstorming
      on 2026-07-01:
      - **Transport: BLE only for v1.** Covers modern dive computers (Shearwater, Suunto, Garmin);
        classic Bluetooth RFCOMM and USB-OTG serial deferred.
      - **BLE bridge: Tauri mobile plugin (Kotlin)**, not direct JNI. Kotlin does BLE scan/connect/
        GATT read-write via `android.bluetooth.le`, exposed to Rust via Tauri's mobile plugin
        channel (invoke + `Channel<T>` for notifications). Rust's `dc_custom_io_t` read/write
        blocks on that channel — same shape as the existing desktop `btleplug` bridge
        (`src-tauri/src/dc/transport/ble.rs`), just swapping the Rust crate for a Kotlin plugin.
      - **Integration point already exists as a stub:** `src-tauri/src/dc/device.rs`'s
        `run_download` has a `#[cfg(target_os = "android")]` branch for opening the iostream, and
        `dc/mod.rs` gates `device.rs`/`parser.rs` with `#[cfg(not(target_os = "android"))]` even
        though `device.rs` internally expects to compile on Android — these gates need removing
        once the Android BLE transport module exists, so `device.rs` picks up its existing branch.
      - **UI entry point:** header icon on the Dives tab in `MobileLayout.svelte`, next to the
        existing "Open logbook" (⊞) and "Cloud logbook" (☁) buttons — reuses the same
        `DcDownloadDialog.svelte`, which already filters transport options to what
        `list_dc_models` reports per model, so Android just needs that list to report BLE-only.
      - **Biggest unknown:** libdivecomputer has never been cross-compiled for Android
        (`build.rs` currently skips `cmake`/`bindgen` entirely for `target_os == "android"`), and
        this project has no Tauri mobile plugin yet. See the build spike spec below before
        attempting the full feature.
      - Spec: `docs/superpowers/specs/2026-07-01-android-libdc-build-spike-design.md` (build
        spike) and `docs/superpowers/specs/2026-07-06-android-ble-dc-download-design.md`
        (full feature design). Implemented via
        `docs/superpowers/plans/2026-07-06-android-ble-dc-download.md`.
- [ ] **Release workflow: signed Android APK/AAB** — debug APKs are already built and uploaded on release (see `release.yml`). Remaining: switch to `--release`, add keystore secrets, and optionally produce an AAB for Play Store. Only needed if Play Store distribution becomes a goal.
- [ ] Imperial units support in `parse_divecomputer.rs` sample parsing (currently metric-only: m, bar, °C — ft, psi, °F sample lines are silently ignored)
- [ ] Surface unreadable Dive/Divecomputer parse errors instead of silently dropping dives (`ssrf_git/mod.rs:69/73`)

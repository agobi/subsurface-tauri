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

- [ ] **Android dive computer download — residual gaps** (PR #66). Core feature (scan,
      connect, download, save, permission-revocation recovery, fingerprint dedup) is
      implemented and verified end-to-end on a real Shearwater Petrel 2 + Pixel 10a.
      Two things not yet confirmed:
      - **Indicate-only BLE characteristic** — the CCCD `ENABLE_INDICATION_VALUE` branch
        (`BleGattClient.kt`'s `cccdValueFor`) is only covered by a Robolectric unit test;
        no indicate-only dive computer was available to verify on real hardware. The
        Petrel 2 tested is notify-based.
      - **Full cancellation mid-download** — attempted once during testing but not
        confirmed end-to-end (partial dives still offered for review afterward).
      - Download throughput is slow (~125–135ms per protocol block) but this was
        investigated and is not considered fixable on our end: it's the dive computer's
        own firmware processing time, not the Android BLE stack — MTU (247) and
        connection-priority (HIGH) are already tuned; connection interval dropped
        49ms→15ms with no meaningful change to per-block latency.
      - Three Android/Tauri integration bugs were found only via on-device testing
        (none catchable by unit tests or static review — worth remembering for any
        future Tauri-mobile-plugin work in this codebase):
        1. A single permission alias listing `ACCESS_FINE_LOCATION` alongside
           `BLUETOOTH_SCAN`/`CONNECT` makes Tauri's plugin framework reject the *entire*
           permission request on API 31+, since the manifest correctly scopes
           `ACCESS_FINE_LOCATION` to `maxSdkVersion="30"` and the framework validates
           every alias string against the manifest regardless of OS version. Fix: split
           into two aliases, request whichever matches `Build.VERSION.SDK_INT`.
        2. `invoke.resolve(JSObject())` (sends `{}`) is not the same as `invoke.resolve()`
           (sends `null`) — Rust's `Result<()>` commands need the latter, or every call
           fails with `failed to deserialize map: invalid type: map, expected unit`.
        3. Android needs an explicit `gatt.requestMtu(247)` after connecting; unlike
           macOS/CoreBluetooth, it defaults to a 23-byte ATT MTU (20 usable bytes),
           silently truncating any BLE notification longer than that.
- [ ] **Release workflow: signed Android APK/AAB** — debug APKs are already built and uploaded on release (see `release.yml`). Remaining: switch to `--release`, add keystore secrets, and optionally produce an AAB for Play Store. Only needed if Play Store distribution becomes a goal.
- [ ] Imperial units support in `parse_divecomputer.rs` sample parsing (currently metric-only: m, bar, °C — ft, psi, °F sample lines are silently ignored)
- [ ] Surface unreadable Dive/Divecomputer parse errors instead of silently dropping dives (`ssrf_git/mod.rs:69/73`)

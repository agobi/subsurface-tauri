# DC Download Logging & Status Feedback

**Date:** 2026-06-29
**Branch:** worktree-feat-dc-download-logging
**Goal:** Add per-dive status labels in the download dialog and structured Rust developer logs (including libdivecomputer internal output) during dive computer download.

---

## Context

The DC download flow (`src-tauri/src/dc/device.rs`) already emits four Tauri events:

| Event | Payload | UI use |
|---|---|---|
| `dc-progress` | `{ current, maximum }` (bytes) | Progress bar |
| `dc-devinfo` | `{ model, firmware, serial }` | Received but **not displayed** |
| `dc-error` | `{ message }` | Error text |
| `dc-complete` | `{ added, skipped }` | Final result |

The download dialog (`src/lib/components/DcDownloadDialog.svelte`) shows a bare progress bar and byte count with no phase labels, no device info, and no per-dive feedback. The Rust DC code has zero `log::info!` calls; `tauri-plugin-log` is configured at `Info` level.

---

## Design

### New Tauri event: `dc-dive`

```ts
{ diveNumber: number, date: string | null, added: boolean }
```

- `diveNumber` — 1-based, incremented in `DownloadCtx` before any processing
- `date` — ISO date string (e.g. `"2026-02-15"`) from `ParsedDive`, `null` for skipped dives (parsing is skipped for known-fingerprint dives)
- `added` — `true` if written to disk, `false` if skipped or failed

### UI status label transitions

```
"Connecting…"                    (initial, before dc-devinfo)
"Connected: Shearwater Perdix AI" (after dc-devinfo)
"Dive 1: 2026-02-15"             (after dc-dive, added=true)
"Skipping dive 2…"               (after dc-dive, added=false, date=null)
```

The existing progress bar and byte count remain unchanged alongside the label.

### Rust changes — `src-tauri/src/dc/`

#### `context.rs` — wire libdivecomputer log callback

Register `dc_context_set_logfunc` in `DcContext::new()` with a C callback that maps libdc log levels to Rust log macros:

| libdc level | Rust macro |
|---|---|
| `DC_LOGLEVEL_ERROR` | `log::error!` |
| `DC_LOGLEVEL_WARNING` | `log::warn!` |
| `DC_LOGLEVEL_INFO` | `log::info!` |
| `DC_LOGLEVEL_DEBUG` | `log::debug!` (filtered at runtime unless level raised) |

The callback receives `(ctx, level, file, line, function, msg, userdata)`. Log as `"[libdc] {file}:{line} {msg}"`. `userdata` is unused (null). The callback is called synchronously on the download thread — no synchronisation needed.

#### `device.rs` — milestones and `dc-dive` event

Add `dive_number: u32` field to `DownloadCtx`.

**`run_download`:**
- After `dc_device_open` succeeds: `log::info!("DC: opened {vendor} {model}")`
- After `dc_device_foreach` returns with error: `log::error!("DC: foreach error status={foreach_rc}")`
- After completion: `log::info!("DC: complete added={} skipped={}", ...)`

**`event_cb` — `DC_EVENT_DEVINFO` branch:**
- `log::info!("DC: devinfo model={} firmware={} serial={}", info.model, info.firmware, info.serial)`

**`dive_cb`:**
- Increment `ctx.dive_number` as the first action.
- For **skipped** dives (known fingerprint): `log::debug!("DC: dive {} skipped (known)", n)` + emit `dc-dive { diveNumber: n, date: null, added: false }`, then return.
- For **added** dives: after `write_dive` succeeds, `log::info!("DC: dive {} added ({})", n, date)` + emit `dc-dive { diveNumber: n, date: Some(date), added: true }`.
- For **parse/write errors**: `log::warn!("DC: dive {} error: {e}", n)` + emit `dc-dive { diveNumber: n, date: null, added: false }` (in addition to existing `dc-error`).

The date string is formatted from `ParsedDive`'s `year`, `month`, `day` fields: `format!("{:04}-{:02}-{:02}", parsed.year, parsed.month, parsed.day)`.

### Frontend changes — `DcDownloadDialog.svelte`

Add state:
```ts
let statusLabel = $state("Connecting…");
```

Listen to `dc-devinfo`:
```ts
listen<{ model: number; firmware: number; serial: number }>("dc-devinfo", (e) => {
  statusLabel = `Connected: ${vendor} ${model}`;  // use UI-level vendor/model strings
});
```

Listen to `dc-dive`:
```ts
listen<{ diveNumber: number; date: string | null; added: boolean }>("dc-dive", (e) => {
  if (e.payload.date) {
    statusLabel = `Dive ${e.payload.diveNumber}: ${e.payload.date}`;
  } else {
    statusLabel = `Skipping dive ${e.payload.diveNumber}…`;
  }
});
```

Reset `statusLabel = "Connecting…"` at the start of `startDownload()`.

Display the label in the `progress` step above the progress bar:
```svelte
<p class="status">{statusLabel}</p>
```

---

## Error Handling

- **Parse fails** — `dive_cb` already emits `dc-error`; also emit `dc-dive { added: false, date: null }` so counter advances and UI doesn't stall.
- **Write fails** — same as parse fails.
- **User cancels** — `dc-complete` fires via existing path with partial counts; no UI change needed.
- **`dc-devinfo` never fires** — frontend stays on `"Connecting…"` until first `dc-dive`; acceptable.
- **libdc log callback thread safety** — callback is synchronous on the download thread; `log::*` macros are thread-safe.

---

## Testing

### Rust
- Smoke test in `context.rs`: `DcContext::new()` succeeds and context drops cleanly (log callback registered, no UB).

### Vitest
- Add test to `DcDownloadDialog`: mock `dc-devinfo` then `dc-dive` events in sequence; assert status label text transitions through `"Connecting…"` → `"Connected: …"` → `"Dive 1: 2026-02-15"`.

### Manual
- Download from a real dive computer; verify label updates per dive and `RUST_LOG=debug` shows libdc output in console.

---

## Files to change

| File | Change |
|---|---|
| `src-tauri/src/dc/context.rs` | Register `dc_context_set_logfunc` |
| `src-tauri/src/dc/device.rs` | Add `dive_number`, `log::*` milestones, emit `dc-dive` |
| `src/lib/components/DcDownloadDialog.svelte` | `statusLabel` state, `dc-devinfo` + `dc-dive` listeners, label in UI |
| `test/DcDownloadDialog.test.ts` (new or existing) | Vitest tests for label transitions |

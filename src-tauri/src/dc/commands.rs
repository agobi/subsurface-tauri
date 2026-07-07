// AI-generated (Claude)
use serde::Serialize;
use super::descriptor::{models_for_vendor, vendors};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DcModelSer {
    pub product: String,
    pub transports: Vec<String>,
}

#[tauri::command]
pub fn list_dc_vendors() -> Vec<String> {
    vendors()
}

#[tauri::command]
pub fn list_dc_models(vendor: String) -> Vec<DcModelSer> {
    models_for_vendor(&vendor)
        .into_iter()
        .map(|m| DcModelSer { product: m.product, transports: m.transports })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownDeviceSer {
    pub vendor: String,
    pub product: String,
    pub serial: String,
    pub nickname: String,
}

/// Resolves every `DeviceRecord` in `settings.devices` back to a
/// (vendor, product) pair via `resolve_vendor_product`, dropping any entry
/// this build's libdivecomputer can't resolve (nothing to reconnect to).
/// `apply_device` always moves a touched record to the end of the vec on
/// commit, so `settings.devices` is already ordered oldest-to-newest by last
/// use — iterating in reverse orders the result most-recently-seen first.
pub fn known_devices_from_settings(settings: &crate::ssrf_git::settings::Settings) -> Vec<KnownDeviceSer> {
    settings.devices.iter().rev()
        .filter_map(|d| {
            let (vendor, product) = crate::dc::descriptor::resolve_vendor_product(&d.model)?;
            Some(KnownDeviceSer { vendor, product, serial: d.serial.clone(), nickname: d.nickname.clone() })
        })
        .collect()
}

#[tauri::command]
pub fn list_known_devices(
    logbook_state: tauri::State<'_, std::sync::Mutex<Option<crate::state::LogbookState>>>,
) -> Result<Vec<KnownDeviceSer>, String> {
    let guard = logbook_state.lock().map_err(|e| e.to_string())?;
    let Some(state) = guard.as_ref() else { return Ok(vec![]); };
    Ok(known_devices_from_settings(&state.settings))
}

pub fn serial_ports_impl() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect()
}

#[tauri::command]
pub fn list_serial_ports() -> Vec<String> {
    serial_ports_impl()
}

/// BLE scan result emitted as the `dc-ble-found` event payload.
#[cfg(not(target_os = "android"))]
#[derive(Clone, serde::Serialize)]
struct BleScanResult {
    name: String,
    address: String,
}

/// Scan for BLE dive computers matching `vendor`/`model`.
///
/// Emits `dc-ble-found` events with `{ name, address }` for each matching
/// peripheral found during a 10-second scan window. Returns immediately;
/// events arrive asynchronously via the Tauri event system.
///
/// The raw `dc_descriptor_t*` is never held across `.await` points — each
/// `dc_descriptor_filter` call is wrapped in `spawn_blocking` to stay on a
/// thread-pool thread, ensuring the async future is `Send`.
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn scan_ble_devices(
    app: tauri::AppHandle,
    vendor: String,
    model: String,
) -> Result<(), String> {
    use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
    use btleplug::platform::Manager;
    use tauri::Emitter;

    // Validate the device exists in libdc before starting the scan.
    {
        let ptr = crate::dc::descriptor::find_descriptor(&vendor, &model)
            .ok_or_else(|| format!("unknown device: {vendor} {model}"))?;
        unsafe { crate::dc::ffi::dc_descriptor_free(ptr) };
    }

    tauri::async_runtime::spawn(async move {
        let manager_result = (async {
            let manager = Manager::new().await.map_err(|e| e.to_string())?;
            let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
            let adapter = adapters
                .into_iter()
                .next()
                .ok_or_else(|| "no BLE adapter found".to_string())?;
            Ok::<_, String>(adapter)
        }).await;

        if let Ok(adapter) = manager_result {
            adapter
                .start_scan(ScanFilter::default())
                .await
                .ok();

            // Poll for peripherals for up to 10 seconds (20 × 500 ms).
            for _ in 0..20 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Ok(peripherals) = adapter.peripherals().await {
                    // Collect (name, address) pairs — async btleplug calls complete here.
                    let mut candidates: Vec<(String, String)> = Vec::new();
                    for p in &peripherals {
                        if let Ok(Some(props)) = p.properties().await {
                            if let Some(name) = props.local_name {
                                // Use p.id() as the identifier: CoreBluetooth UUID on macOS,
                                // MAC address on Linux. props.address is always zero on macOS.
                                candidates.push((name, p.id().to_string()));
                            }
                        }
                    }

                    // Filter candidates via dc_descriptor_filter in spawn_blocking so the
                    // raw pointer never lives across an await point in this future.
                    for (name, address) in candidates {
                        let v = vendor.clone();
                        let m = model.clone();
                        let name_clone = name.clone();
                        let matches = tauri::async_runtime::spawn_blocking(move || {
                            let Some(desc) = crate::dc::descriptor::find_descriptor(&v, &m) else {
                                return false;
                            };
                            let Ok(c_name) = std::ffi::CString::new(name_clone.as_str()) else {
                                unsafe { crate::dc::ffi::dc_descriptor_free(desc) };
                                return false;
                            };
                            // dc_descriptor_filter returns non-zero on match.
                            let result = unsafe {
                                crate::dc::ffi::dc_descriptor_filter(
                                    desc,
                                    crate::dc::ffi::dc_transport_t_DC_TRANSPORT_BLE,
                                    c_name.as_ptr() as *const _,
                                ) != 0
                            };
                            unsafe { crate::dc::ffi::dc_descriptor_free(desc) };
                            result
                        })
                        .await
                        .unwrap_or(false);

                        if matches {
                            app.emit(
                                "dc-ble-found",
                                BleScanResult { name, address },
                            )
                            .ok();
                        }
                    }
                }
            }
            adapter.stop_scan().await.ok();
        } else if let Err(e) = manager_result {
            app.emit("dc-error", serde_json::json!({ "message": e })).ok();
        }
    });

    Ok(())
}

// ── Pending download state ──────────────────────────────────────────────────

/// Dives buffered in memory after a completed download, waiting for user confirmation.
pub struct PendingDownload {
    pub dives: Vec<crate::dc::writer::ParsedDive>,
    pub newest_fp: Option<(u32, Vec<u8>)>,
    pub dc_model: String,
    pub logbook_root: std::path::PathBuf,
}

pub type PendingDownloadState = std::sync::Mutex<Option<PendingDownload>>;

/// Builds the buffered state from a completed [`crate::dc::device::DownloadResult`].
///
/// Uses `result.dc_model` — the model string `run_download` actually used for
/// its fingerprint lookups — rather than re-deriving it from the caller's raw
/// vendor/model input, so the fingerprint saved at commit time always matches
/// the key that was looked up this session (see `resolve_descriptor_for_model`
/// in `descriptor.rs` for why the two can differ).
fn build_pending_download(
    result: crate::dc::device::DownloadResult,
    logbook_root: std::path::PathBuf,
) -> PendingDownload {
    PendingDownload {
        dc_model: result.dc_model,
        dives: result.new_dives,
        newest_fp: result.newest_fp,
        logbook_root,
    }
}

// ── Download commands ───────────────────────────────────────────────────────

/// Summary of a single buffered dive, sent to the frontend for the review step.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiveSummarySer {
    pub date: String,
    pub duration_sec: u32,
    pub max_depth_m: f64,
}

/// Payload carried by both the `dc-complete` event and the `start_dc_download` return value.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadCompleteSer {
    /// New dives ready for review. Empty when cancelled or nothing new.
    pub dives: Vec<DiveSummarySer>,
    pub skipped: u32,
    pub cancelled: bool,
}

/// True only when there is genuinely nothing to save: the download was
/// cancelled before a single dive was fetched. A cancel that already fetched
/// some dives, or a normal completion with zero *new* dives (still needs its
/// fingerprint cutoff committed), must both flow through to buffering.
fn should_discard_without_saving(result: &crate::dc::device::DownloadResult) -> bool {
    result.cancelled && result.new_dives.is_empty()
}

/// Start a dive-computer download.
///
/// Resets the cancel flag, spawns a blocking task that calls
/// [`crate::dc::device::run_download`], and emits `dc-complete` on success.
///
/// Buffers dives in [`PendingDownloadState`] and emits `dc-complete { dives,
/// skipped, cancelled }` for the review step whenever there's anything to
/// save — including a cancelled run that still fetched some dives. Only a
/// cancel with zero fetched dives discards outright (see
/// [`should_discard_without_saving`]).
///
/// The frontend may call [`cancel_dc_download`] at any time to abort.
#[tauri::command]
pub async fn start_dc_download(
    app: tauri::AppHandle,
    vendor: String,
    model: String,
    transport: crate::dc::transport::TransportArg,
    cancel: tauri::State<'_, std::sync::Arc<std::sync::atomic::AtomicBool>>,
    logbook_state: tauri::State<'_, std::sync::Mutex<Option<crate::state::LogbookState>>>,
    pending: tauri::State<'_, PendingDownloadState>,
) -> Result<DownloadCompleteSer, String> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    // Refuse to clobber a prior batch that's still buffered awaiting review.
    if pending.lock().map_err(|e| e.to_string())?.is_some() {
        return Err("A previous dive computer download is still waiting for review — save or discard it first.".to_string());
    }

    // Reset cancel flag from any previous download.
    cancel.store(false, Ordering::Relaxed);
    let cancel_clone = std::sync::Arc::clone(&cancel);

    // Clone (root, settings, dives) from the managed state — lock is released before device I/O.
    let (root, settings, dives) = {
        let guard = logbook_state.lock().map_err(|e| e.to_string())?;
        let state = guard.as_ref().ok_or_else(|| "no logbook open".to_string())?;
        (state.root.clone(), state.settings.clone(), state.dives.clone())
    };

    let app_clone = app.clone();
    let vendor_clone = vendor.clone();
    let model_clone = model.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        crate::dc::device::run_download(
            app_clone,
            settings,
            dives,
            vendor_clone,
            model_clone,
            transport,
            cancel_clone,
        )
    })
    .await
    .map_err(|e| e.to_string())??;

    // `result.cancelled` was captured synchronously inside run_download, right
    // as dc_device_foreach returned — not re-read from the shared flag here,
    // which would race against a cancel click landing in the gap between
    // run_download finishing and this line running.
    let payload = if should_discard_without_saving(&result) {
        // Nothing was fetched before the cancel took effect — nothing to save.
        *pending.lock().map_err(|e| e.to_string())? = None;
        DownloadCompleteSer { dives: vec![], skipped: result.skipped, cancelled: true }
    } else {
        // Either a normal completion, or a cancel that still fetched some
        // dives — buffer whatever arrived so the user can review/save it
        // instead of losing already-fetched progress.
        let skipped = result.skipped;
        let cancelled = result.cancelled;
        let summaries: Vec<DiveSummarySer> = result.new_dives.iter().map(|d| DiveSummarySer {
            date: format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                d.year, d.month, d.day, d.hour, d.minute, d.second),
            duration_sec: d.duration_sec,
            max_depth_m: d.max_depth_m,
        }).collect();

        *pending.lock().map_err(|e| e.to_string())? = Some(build_pending_download(result, root));

        DownloadCompleteSer { dives: summaries, skipped, cancelled }
    };

    app.emit("dc-complete", &payload).ok();
    Ok(payload)
}

/// Keeps only the dives at the given indices, in the order the frontend showed
/// them in the review step. Indices outside the range are ignored.
///
/// The fingerprint cutoff (`newest_fp`) always advances to the device's true
/// newest dive in `commit_dc_download` regardless of selection, so deselected
/// dives won't reappear on the next download even though they're skipped here.
fn select_dives(dives: Vec<crate::dc::writer::ParsedDive>, selected: &[usize]) -> Vec<crate::dc::writer::ParsedDive> {
    let selected: std::collections::HashSet<usize> = selected.iter().copied().collect();
    dives.into_iter()
        .enumerate()
        .filter(|(i, _)| selected.contains(i))
        .map(|(_, d)| d)
        .collect()
}

/// Writes each dive, continuing past individual failures so one bad dive
/// (disk full, permission error, a directory collision) doesn't lose the
/// rest of an already-buffered, no-longer-retryable batch. Returns the
/// count actually written and the error message for each dive that failed.
fn write_all_dives(root: &std::path::Path, dives: Vec<crate::dc::writer::ParsedDive>) -> (u32, Vec<String>) {
    let mut written = 0u32;
    let mut errors = Vec::new();
    for dive in dives {
        match crate::dc::writer::write_dive(root, dive) {
            Ok(()) => written += 1,
            Err(e) => errors.push(e),
        }
    }
    (written, errors)
}

/// Write buffered dives to disk, save fingerprint, and update in-memory logbook state.
///
/// Called by the frontend after the user confirms the review step.
/// `selected_indices` are positions into the dive list as shown in the review
/// step (matching `DownloadCompleteSer::dives` order); only those dives are
/// written. A dive that fails to write does not abort the rest of the batch
/// (see `write_all_dives`) or block the fingerprint update, since the
/// buffered dives are already removed from `PendingDownloadState` and
/// cannot be retried. Returns the count of dives actually written so the
/// frontend can call `startup_logbook` and show the result.
#[tauri::command]
pub async fn commit_dc_download(
    selected_indices: Vec<usize>,
    pending: tauri::State<'_, PendingDownloadState>,
    logbook_state: tauri::State<'_, std::sync::Mutex<Option<crate::state::LogbookState>>>,
) -> Result<u32, String> {
    let pd = pending.lock().map_err(|e| e.to_string())?.take()
        .ok_or_else(|| "no pending download to commit".to_string())?;

    let dives = select_dives(pd.dives, &selected_indices);
    let root = pd.logbook_root;
    let dc_model = pd.dc_model;
    let newest_fp = pd.newest_fp;

    let root_clone = root.clone();
    let dc_model_clone = dc_model.clone();
    let newest_fp_clone = newest_fp.clone();

    let written = tauri::async_runtime::spawn_blocking(move || -> u32 {
        let (written, errors) = write_all_dives(&root_clone, dives);
        if !errors.is_empty() {
            log::warn!("DC: {} of {} dive(s) failed to write: {}",
                errors.len(), written as usize + errors.len(), errors.join("; "));
        }
        // The fingerprint cutoff reflects the device's true newest dive, not
        // which dives were successfully written — always advance it, even if
        // some (or all) writes failed, so a partial-failure retry doesn't
        // re-download dives that already made it to disk.
        if let Some((serial, fp)) = newest_fp_clone {
            crate::dc::fingerprint::upsert_fp(&root_clone, &dc_model_clone, serial, &fp).ok();
            crate::dc::fingerprint::upsert_device(&root_clone, &dc_model_clone, serial).ok();
        }
        written
    })
    .await
    .map_err(|e| e.to_string())?;

    // Apply fingerprint to in-memory settings so subsequent downloads in this session use it.
    if let Some((serial, fp)) = &newest_fp {
        let mut guard = logbook_state.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut state) = *guard {
            crate::dc::fingerprint::apply_fp(&mut state.settings, &dc_model, *serial, fp);
            crate::dc::fingerprint::apply_device(&mut state.settings, &dc_model, *serial);
        }
    }

    Ok(written)
}

/// Discard buffered dives without saving anything.
///
/// Called by the frontend when the user dismisses the review step.
#[tauri::command]
pub fn discard_dc_download(
    pending: tauri::State<'_, PendingDownloadState>,
) -> Result<(), String> {
    *pending.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

/// Set the cancel flag to abort an in-progress download.
///
/// The next call to the libdivecomputer cancel callback will return 1, causing
/// `dc_device_foreach` to stop after the current dive completes.
#[tauri::command]
pub fn cancel_dc_download(
    cancel: tauri::State<'_, std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    cancel.store(true, Ordering::Relaxed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::dc::writer::{ParsedCylinder, ParsedDive};
    use crate::types::Sample;

    #[test]
    fn list_serial_ports_returns_vec() {
        // Just verify it doesn't panic; actual ports depend on hardware.
        let _ports = super::serial_ports_impl();
        // passes if it returns without panicking
    }

    #[test]
    fn build_pending_download_uses_the_session_dc_model_not_a_reconstruction() {
        // Regression test: the fingerprint cutoff must be persisted under the
        // model string run_download actually used for its lookups (which may
        // have been auto-corrected from the device's self-reported model —
        // see resolve_descriptor_for_model), not re-derived from the raw
        // vendor/model strings the UI originally passed in. Using a
        // reconstruction caused a fresh cutoff to be written under the wrong
        // key, so the next download re-scanned past it and re-delivered dives
        // that were already saved.
        let result = crate::dc::device::DownloadResult {
            new_dives: vec![],
            skipped: 0,
            newest_fp: Some((20819261, vec![0x6a, 0x2e, 0x80, 0x20])),
            dc_model: "Shearwater Perdix AI".to_string(),
            cancelled: false,
        };
        let pending = super::build_pending_download(result, std::path::PathBuf::from("/tmp/logbook"));
        assert_eq!(pending.dc_model, "Shearwater Perdix AI");
    }

    fn make_dive(duration_sec: u32) -> ParsedDive {
        ParsedDive {
            year: 2026, month: 6, day: 11,
            hour: 12, minute: 0, second: 0,
            duration_sec,
            max_depth_m: 10.0,
            mean_depth_m: 5.0,
            water_temp_c: None,
            cylinders: Vec::<ParsedCylinder>::new(),
            samples: Vec::<Sample>::new(),
            events: vec![],
            dc_model: "Shearwater Perdix AI".to_string(),
            device_id: "a790cf6c".to_string(),
            dive_id: vec![0, 0, 0, duration_sec as u8],
        }
    }

    #[test]
    fn select_dives_keeps_only_the_given_indices_in_order() {
        let dives = vec![make_dive(100), make_dive(200), make_dive(300)];
        let kept = super::select_dives(dives, &[0, 2]);
        let kept_durations: Vec<u32> = kept.iter().map(|d| d.duration_sec).collect();
        assert_eq!(kept_durations, vec![100, 300]);
    }

    #[test]
    fn select_dives_with_no_indices_returns_empty() {
        let dives = vec![make_dive(100), make_dive(200)];
        let kept = super::select_dives(dives, &[]);
        assert!(kept.is_empty());
    }

    #[test]
    fn select_dives_ignores_out_of_range_indices() {
        let dives = vec![make_dive(100)];
        let kept = super::select_dives(dives, &[0, 5]);
        assert_eq!(kept.len(), 1);
    }

    fn make_dive_on_day(day: u32) -> ParsedDive {
        let mut d = make_dive(100);
        d.day = day;
        d
    }

    fn make_result(new_dives: Vec<ParsedDive>, cancelled: bool) -> crate::dc::device::DownloadResult {
        crate::dc::device::DownloadResult {
            new_dives,
            skipped: 0,
            newest_fp: None,
            dc_model: "Shearwater Perdix AI".to_string(),
            cancelled,
        }
    }

    #[test]
    fn cancelling_with_no_fetched_dives_has_nothing_to_save() {
        let result = make_result(vec![], true);
        assert!(super::should_discard_without_saving(&result));
    }

    #[test]
    fn cancelling_after_fetching_dives_still_preserves_them() {
        let result = make_result(vec![make_dive(100)], true);
        assert!(!super::should_discard_without_saving(&result),
            "a cancel that already fetched dives must not discard them");
    }

    #[test]
    fn a_normal_completion_with_zero_new_dives_is_not_discarded() {
        // Must still flow through to build_pending_download so the fingerprint
        // cutoff can be committed even though there's nothing to review.
        let result = make_result(vec![], false);
        assert!(!super::should_discard_without_saving(&result));
    }

    #[test]
    fn write_all_dives_continues_past_a_failing_dive_and_reports_the_error() {
        let tmp = std::env::temp_dir().join("dc_write_all_dives_partial_failure");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();

        // Dive on day 11 will succeed; dive on day 12 is blocked because a
        // regular file already occupies the directory path it needs to create.
        std::fs::create_dir_all(tmp.join("2026/06")).unwrap();
        std::fs::write(tmp.join("2026/06/12-Fri-12=00=00"), b"blocking file").unwrap();

        let dives = vec![make_dive_on_day(11), make_dive_on_day(12)];
        let (written, errors) = super::write_all_dives(&tmp, dives);

        assert_eq!(written, 1, "the succeeding dive must still be written");
        assert_eq!(errors.len(), 1, "the failing dive's error must be reported, not silently dropped");
        assert!(tmp.join("2026/06/11-Thu-12=00=00/Dive-001").exists());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn known_devices_from_settings_resolves_and_filters_unknown_models() {
        use crate::ssrf_git::settings::{Settings, DeviceRecord};
        let mut settings = Settings::default();
        settings.devices.push(DeviceRecord {
            model: "Shearwater Perdix".to_string(),
            device_id: 1,
            serial: "0001e240".to_string(),
            nickname: "My Perdix".to_string(),
        });
        settings.devices.push(DeviceRecord {
            model: "Nonexistent Vendor Model 9000".to_string(),
            device_id: 2,
            serial: "00000002".to_string(),
            nickname: "".to_string(),
        });
        let known = super::known_devices_from_settings(&settings);
        assert_eq!(known.len(), 1, "the unresolvable model must be filtered out");
        assert_eq!(known[0].vendor, "Shearwater");
        assert_eq!(known[0].product, "Perdix");
        assert_eq!(known[0].serial, "0001e240");
        assert_eq!(known[0].nickname, "My Perdix");
    }

    #[test]
    fn known_devices_from_settings_orders_most_recently_seen_first() {
        use crate::ssrf_git::settings::{Settings, DeviceRecord};
        let mut settings = Settings::default();
        // apply_device always pushes the touched record to the end of the
        // vec, so this insertion order simulates "Perdix" downloaded from
        // first, "Perdix AI" downloaded from most recently.
        settings.devices.push(DeviceRecord {
            model: "Shearwater Perdix".to_string(),
            device_id: 1,
            serial: "00000001".to_string(),
            nickname: "".to_string(),
        });
        settings.devices.push(DeviceRecord {
            model: "Shearwater Perdix AI".to_string(),
            device_id: 2,
            serial: "00000002".to_string(),
            nickname: "".to_string(),
        });
        let known = super::known_devices_from_settings(&settings);
        assert_eq!(known.len(), 2);
        assert_eq!(known[0].serial, "00000002", "most recently touched device must be first");
        assert_eq!(known[1].serial, "00000001");
    }
}

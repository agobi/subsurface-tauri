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

// ── Download commands ───────────────────────────────────────────────────────

/// Summary of a single buffered dive, sent to the frontend for the review step.
#[cfg(not(target_os = "android"))]
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiveSummarySer {
    pub date: String,
    pub duration_sec: u32,
    pub max_depth_m: f64,
}

/// Payload carried by both the `dc-complete` event and the `start_dc_download` return value.
#[cfg(not(target_os = "android"))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadCompleteSer {
    /// New dives ready for review. Empty when cancelled or nothing new.
    pub dives: Vec<DiveSummarySer>,
    pub skipped: u32,
    pub cancelled: bool,
}

/// Start a dive-computer download.
///
/// Resets the cancel flag, spawns a blocking task that calls
/// [`crate::dc::device::run_download`], and emits `dc-complete` on success.
///
/// On success (not cancelled): buffers dives in [`PendingDownloadState`] and
/// emits `dc-complete { dives, skipped, cancelled: false }` for the review step.
/// On cancel: discards buffered dives and emits `dc-complete { cancelled: true }`.
///
/// The frontend may call [`cancel_dc_download`] at any time to abort.
#[cfg(not(target_os = "android"))]
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

    let was_cancelled = cancel.load(Ordering::Relaxed);

    let payload = if was_cancelled {
        // Discard buffered dives; do not advance fingerprint.
        *pending.lock().map_err(|e| e.to_string())? = None;
        DownloadCompleteSer { dives: vec![], skipped: result.skipped, cancelled: true }
    } else {
        let summaries: Vec<DiveSummarySer> = result.new_dives.iter().map(|d| DiveSummarySer {
            date: format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                d.year, d.month, d.day, d.hour, d.minute, d.second),
            duration_sec: d.duration_sec,
            max_depth_m: d.max_depth_m,
        }).collect();

        *pending.lock().map_err(|e| e.to_string())? = Some(PendingDownload {
            dives: result.new_dives,
            newest_fp: result.newest_fp,
            dc_model: format!("{vendor} {model}"),
            logbook_root: root,
        });

        DownloadCompleteSer { dives: summaries, skipped: result.skipped, cancelled: false }
    };

    app.emit("dc-complete", &payload).ok();
    Ok(payload)
}

/// Write buffered dives to disk, save fingerprint, and update in-memory logbook state.
///
/// Called by the frontend after the user confirms the review step.
/// Returns the count of dives written so the frontend can call `startup_logbook`
/// and show the result.
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn commit_dc_download(
    pending: tauri::State<'_, PendingDownloadState>,
    logbook_state: tauri::State<'_, std::sync::Mutex<Option<crate::state::LogbookState>>>,
) -> Result<u32, String> {
    let pd = pending.lock().map_err(|e| e.to_string())?.take()
        .ok_or_else(|| "no pending download to commit".to_string())?;

    let added = pd.dives.len() as u32;
    let root = pd.logbook_root;
    let dc_model = pd.dc_model;
    let newest_fp = pd.newest_fp;

    let root_clone = root.clone();
    let dc_model_clone = dc_model.clone();
    let newest_fp_clone = newest_fp.clone();

    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        for dive in pd.dives {
            crate::dc::writer::write_dive(&root_clone, dive)?;
        }
        if let Some((serial, fp)) = newest_fp_clone {
            crate::dc::fingerprint::upsert_fp(&root_clone, &dc_model_clone, serial, &fp).ok();
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    // Apply fingerprint to in-memory settings so subsequent downloads in this session use it.
    if let Some((serial, fp)) = &newest_fp {
        let mut guard = logbook_state.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut state) = *guard {
            crate::dc::fingerprint::apply_fp(&mut state.settings, &dc_model, *serial, fp);
        }
    }

    Ok(added)
}

/// Discard buffered dives without saving anything.
///
/// Called by the frontend when the user dismisses the review step.
#[cfg(not(target_os = "android"))]
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
#[cfg(not(target_os = "android"))]
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
    #[test]
    fn list_serial_ports_returns_vec() {
        // Just verify it doesn't panic; actual ports depend on hardware.
        let _ports = super::serial_ports_impl();
        // passes if it returns without panicking
    }
}

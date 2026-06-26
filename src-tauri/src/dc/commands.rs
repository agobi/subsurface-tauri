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
                                candidates.push((name, props.address.to_string()));
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

/// Result returned by [`start_dc_download`], serialized to the frontend.
#[cfg(not(target_os = "android"))]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResultSer {
    pub added: u32,
    pub skipped: u32,
}

/// Start a dive-computer download.
///
/// Resets the cancel flag, spawns a blocking task that calls
/// [`crate::dc::device::run_download`], and emits `dc-complete` on success.
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
    dives_state: tauri::State<'_, std::sync::Mutex<Vec<crate::types::Dive>>>,
) -> Result<DownloadResultSer, String> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    // Reset cancel flag from any previous download.
    cancel.store(false, Ordering::Relaxed);
    let cancel_clone = std::sync::Arc::clone(&cancel);

    let dives = dives_state.lock().map_err(|e| e.to_string())?.clone();

    // Read logbook path from the persisted store.
    let root = {
        use tauri_plugin_store::StoreExt;
        let store = app.store("settings.json").map_err(|e| e.to_string())?;
        let path = store
            .get("logbookPath")
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or_else(|| "no logbook open".to_string())?;
        std::path::PathBuf::from(path)
    };

    let app_clone = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        crate::dc::device::run_download(
            app_clone,
            root,
            dives,
            vendor,
            model,
            transport,
            cancel_clone,
        )
    })
    .await
    .map_err(|e| e.to_string())??;

    app.emit(
        "dc-complete",
        serde_json::json!({ "added": result.added, "skipped": result.skipped }),
    )
    .ok();

    Ok(DownloadResultSer { added: result.added, skipped: result.skipped })
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

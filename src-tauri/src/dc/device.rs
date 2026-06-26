// AI-generated (Claude)
//! Orchestrates a full dive-computer download session using libdivecomputer.
//!
//! `run_download` opens the device, registers event/cancel/dive callbacks,
//! calls `dc_device_foreach`, and writes each new dive to the logbook via
//! [`crate::dc::writer::write_dive`].
use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use crate::dc::ffi::*;
use crate::dc::context::DcContext;
use crate::dc::descriptor::find_descriptor;
use crate::dc::fingerprint::{write_fp, known_dive_ids};
use crate::dc::parser::parse_dive;
use crate::dc::transport::{open_iostream, TransportArg};
use crate::dc::writer::write_dive;

/// Result returned by [`run_download`].
pub struct DownloadResult {
    pub added: u32,
    pub skipped: u32,
}

/// Userdata passed through libdivecomputer callbacks.
struct DownloadCtx<R: tauri::Runtime> {
    app: AppHandle<R>,
    logbook_root: std::path::PathBuf,
    descriptor: *mut dc_descriptor_t,
    /// Raw pointer to the context owned by `run_download`'s `DcContext`. Valid
    /// for the entire lifetime of the download (i.e. until after `dc_device_foreach`).
    ctx_ptr: *mut dc_context_t,
    dc_model: String,
    device_id: Option<String>,
    known_fingerprints: HashSet<Vec<u8>>,
    newest_fingerprint: Option<Vec<u8>>,
    added: u32,
    skipped: u32,
    cancel: Arc<AtomicBool>,
}

// Safety: `descriptor` and `ctx_ptr` are raw pointers to libdivecomputer objects.
// They are used only from the single thread that runs `dc_device_foreach`, so
// sending across threads is safe here.
unsafe impl<R: tauri::Runtime> Send for DownloadCtx<R> {}

/// Event callback — handles `DC_EVENT_PROGRESS` and `DC_EVENT_DEVINFO`.
unsafe extern "C" fn event_cb<R: tauri::Runtime>(
    _device: *mut dc_device_t,
    event: dc_event_type_t,
    data: *const c_void,
    userdata: *mut c_void,
) {
    let ctx = &mut *(userdata as *mut DownloadCtx<R>);
    if event == dc_event_type_t_DC_EVENT_PROGRESS {
        let p = &*(data as *const dc_event_progress_t);
        ctx.app
            .emit("dc-progress", serde_json::json!({ "current": p.current, "maximum": p.maximum }))
            .ok();
    } else if event == dc_event_type_t_DC_EVENT_DEVINFO {
        let info = &*(data as *const dc_event_devinfo_t);
        ctx.device_id = Some(format!("{:08x}", info.serial));
        ctx.app
            .emit(
                "dc-devinfo",
                serde_json::json!({
                    "model": info.model,
                    "firmware": info.firmware,
                    "serial": info.serial,
                }),
            )
            .ok();
    }
}

/// Cancel callback — returns 1 when the cancel flag is set.
unsafe extern "C" fn cancel_cb<R: tauri::Runtime>(userdata: *mut c_void) -> ::std::os::raw::c_int {
    let ctx = &*(userdata as *mut DownloadCtx<R>);
    ctx.cancel.load(Ordering::Relaxed) as ::std::os::raw::c_int
}

/// Dive callback — called once per dive by `dc_device_foreach`.
///
/// Creates a parser with `dc_parser_new2`, calls `parse_dive`, then writes
/// the result to the logbook. Skips dives whose fingerprint is already known.
unsafe extern "C" fn dive_cb<R: tauri::Runtime>(
    data: *const ::std::os::raw::c_uchar,
    size: ::std::os::raw::c_uint,
    fingerprint: *const ::std::os::raw::c_uchar,
    fsize: ::std::os::raw::c_uint,
    userdata: *mut c_void,
) -> ::std::os::raw::c_int {
    let ctx = &mut *(userdata as *mut DownloadCtx<R>);
    let fp = std::slice::from_raw_parts(fingerprint, fsize as usize).to_vec();

    // Skip dives already in the logbook (known from dc_dive_id hex strings).
    if ctx.known_fingerprints.contains(&fp) {
        ctx.skipped += 1;
        return 1; // 1 = continue iterating
    }

    let device_id = ctx.device_id.clone().unwrap_or_default();
    let model = ctx.dc_model.clone();

    // Create a parser for this dive's raw bytes.
    // dc_parser_new2 takes (parser_out, context, descriptor, data, size).
    let mut parser: *mut dc_parser_t = ptr::null_mut();
    if dc_parser_new2(
        &mut parser,
        ctx.ctx_ptr,
        ctx.descriptor,
        data,
        size as usize,
    ) != dc_status_t_DC_STATUS_SUCCESS
    {
        return 1; // failed to create parser — skip this dive but continue
    }

    match parse_dive(parser, &model, &device_id, fp.clone()) {
        Ok(parsed) => {
            dc_parser_destroy(parser);
            match write_dive(&ctx.logbook_root, parsed) {
                Ok(_) => {
                    ctx.added += 1;
                    // Track the newest fingerprint for persisting after the download.
                    // libdc delivers dives newest-first, so only keep the first one.
                    if ctx.newest_fingerprint.is_none() {
                        ctx.newest_fingerprint = Some(fp);
                    }
                }
                Err(e) => {
                    ctx.app
                        .emit("dc-error", serde_json::json!({ "message": e }))
                        .ok();
                }
            }
        }
        Err(_) => {
            dc_parser_destroy(parser);
        }
    }
    1 // continue iterating
}

/// Download all new dives from a dive computer and write them to `logbook_root`.
///
/// Emits Tauri events:
/// - `dc-progress { current, maximum }` — download progress
/// - `dc-devinfo { model, firmware, serial }` — device identification
/// - `dc-error { message }` — per-dive write error (non-fatal)
///
/// Returns [`DownloadResult`] with the count of added and skipped dives.
///
/// This function blocks the calling thread; call it via
/// `tauri::async_runtime::spawn_blocking`.
pub fn run_download<R: tauri::Runtime>(
    app: AppHandle<R>,
    logbook_root: std::path::PathBuf,
    dives: Vec<crate::types::Dive>,
    vendor: String,
    model: String,
    transport_arg: TransportArg,
    cancel: Arc<AtomicBool>,
) -> Result<DownloadResult, String> {
    let descriptor = find_descriptor(&vendor, &model)
        .ok_or_else(|| format!("unknown device: {vendor} {model}"))?;

    let ctx_dc = DcContext::new().map_err(|e| {
        unsafe { dc_descriptor_free(descriptor) };
        e
    })?;

    // Open the I/O stream for the selected transport.
    let iostream = {
        #[cfg(not(target_os = "android"))]
        let result = match &transport_arg {
            TransportArg::Ble { name } => {
                let handle = tokio::runtime::Handle::current();
                crate::dc::transport::ble::open_ble_iostream(name, handle)
            }
            _ => open_iostream(&ctx_dc, descriptor, &transport_arg),
        };
        #[cfg(target_os = "android")]
        let result = open_iostream(&ctx_dc, descriptor, &transport_arg);
        result
    };
    let iostream = iostream.map_err(|e| {
        unsafe { dc_descriptor_free(descriptor) };
        e
    })?;

    let known_fingerprints = known_dive_ids(&dives);

    let mut download_ctx = DownloadCtx {
        app: app.clone(),
        logbook_root,
        descriptor,
        ctx_ptr: ctx_dc.as_ptr(),
        dc_model: model.clone(),
        device_id: None,
        known_fingerprints,
        newest_fingerprint: None,
        added: 0,
        skipped: 0,
        cancel,
    };

    unsafe {
        let mut device: *mut dc_device_t = ptr::null_mut();
        let rc = dc_device_open(&mut device, ctx_dc.as_ptr(), descriptor, iostream);
        if rc != dc_status_t_DC_STATUS_SUCCESS {
            dc_iostream_close(iostream);
            dc_descriptor_free(descriptor);
            return Err(format!("dc_device_open failed: {rc}"));
        }

        // Register event callback for progress and device-info events.
        dc_device_set_events(
            device,
            (dc_event_type_t_DC_EVENT_PROGRESS | dc_event_type_t_DC_EVENT_DEVINFO) as ::std::os::raw::c_uint,
            Some(event_cb::<R>),
            &mut download_ctx as *mut DownloadCtx<R> as *mut _,
        );

        // Register cancel callback.
        dc_device_set_cancel(
            device,
            Some(cancel_cb::<R>),
            &mut download_ctx as *mut DownloadCtx<R> as *mut _,
        );

        // Iterate all dives on the device; dive_cb is invoked for each one.
        // Note: ideally we set dc_device_set_fingerprint after DC_EVENT_DEVINFO fires
        // (to know the device serial → look up stored fingerprint). For now,
        // known_fingerprints (from dc_dive_id in the logbook) handles dedup on first
        // download, and subsequent downloads rely on the same mechanism until
        // fingerprint-based dedup is added.
        let foreach_rc = dc_device_foreach(
            device,
            Some(dive_cb::<R>),
            &mut download_ctx as *mut DownloadCtx<R> as *mut _,
        );

        dc_device_close(device);
        dc_iostream_close(iostream);
        dc_descriptor_free(descriptor);

        // If foreach failed (connection lost, timeout, etc.) and user didn't cancel,
        // report the error rather than silently returning partial results.
        if foreach_rc != crate::dc::ffi::dc_status_t_DC_STATUS_SUCCESS
            && !download_ctx.cancel.load(Ordering::Relaxed)
        {
            return Err(format!("device download interrupted: status {foreach_rc}"));
        }
    }

    // Persist the newest fingerprint so the next download can skip already-seen dives.
    if !download_ctx.cancel.load(Ordering::Relaxed) {
        if let (Some(device_id), Some(fp)) =
            (&download_ctx.device_id, &download_ctx.newest_fingerprint)
        {
            write_fp(&app, device_id, fp).ok();
        }
    }

    Ok(DownloadResult {
        added: download_ctx.added,
        skipped: download_ctx.skipped,
    })
}

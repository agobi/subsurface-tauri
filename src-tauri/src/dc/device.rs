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
use crate::dc::fingerprint::{lookup_fp, upsert_fp, known_dive_ids};
use crate::dc::parser::parse_dive;
use crate::dc::transport::{open_iostream, TransportArg};
use crate::dc::writer::write_dive;

/// Result returned by [`run_download`].
pub struct DownloadResult {
    pub added: u32,
    pub skipped: u32,
    /// The serial number and raw fingerprint bytes of the newest dive seen, if any.
    /// The caller uses this to update in-memory state after `upsert_fp` writes to disk.
    pub newest_fp: Option<(u32, Vec<u8>)>,
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
    device_serial: Option<u32>,
    settings: crate::ssrf_git::settings::Settings,
    // SHA1_uint32 of each known fingerprint byte sequence, matching the diveid format
    // written to Divecomputer files and used by original Subsurface's has_dive() check.
    known_fingerprints: HashSet<u32>,
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
    device: *mut dc_device_t,
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

        // Use the stored fingerprint to tell libdivecomputer where to stop, so it
        // doesn't re-download dives we already have.
        if let Some(fp) = lookup_fp(&ctx.settings, &ctx.dc_model, info.serial) {
            dc_device_set_fingerprint(device, fp.as_ptr(), fp.len() as u32);
        }

        ctx.device_serial = Some(info.serial);
        // Original Subsurface stores deviceid as SHA1_uint32(serial_string), matching
        // calculate_string_hash() in libdivecomputer.cpp.
        let device_id_hash = crate::ssrf_git::settings::sha1_u32(
            info.serial.to_string().as_bytes()
        );
        ctx.device_id = Some(format!("{:08x}", device_id_hash));
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

    // libdc delivers dives newest-first, so the first dive delivered is the most recent.
    // Capture its fingerprint unconditionally — even if the dive will be skipped as
    // already-known — so upsert_fp always has a valid frontier to store after the
    // download, regardless of whether any new dives were actually written.
    if ctx.newest_fingerprint.is_none() {
        ctx.newest_fingerprint = Some(fp.clone());
    }

    // Skip dives already in the logbook. Compare SHA1(fp) against the stored set of
    // SHA1 hashes — the Divecomputer file stores SHA1_uint32(fingerprint), not raw bytes.
    let fp_hash = crate::ssrf_git::settings::sha1_u32(&fp);
    if ctx.known_fingerprints.contains(&fp_hash) {
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
/// `settings` is cloned from managed state before the call; it is not re-read from disk.
///
/// Emits Tauri events:
/// - `dc-progress { current, maximum }` — download progress
/// - `dc-devinfo { model, firmware, serial }` — device identification
/// - `dc-error { message }` — per-dive write error (non-fatal)
///
/// Returns [`DownloadResult`] with the count of added and skipped dives, plus the
/// newest fingerprint (serial, bytes) so the caller can update in-memory state.
///
/// This function blocks the calling thread; call it via
/// `tauri::async_runtime::spawn_blocking`.
#[allow(clippy::too_many_arguments)]
pub fn run_download<R: tauri::Runtime>(
    app: AppHandle<R>,
    logbook_root: std::path::PathBuf,
    settings: crate::ssrf_git::settings::Settings,
    dives: Vec<crate::types::Dive>,
    vendor: String,
    model: String,
    transport_arg: TransportArg,
    cancel: Arc<AtomicBool>,
) -> Result<DownloadResult, String> {
    let descriptor = find_descriptor(&vendor, &model)
        .ok_or_else(|| format!("unknown device: {vendor} {model}"))?;

    let ctx_dc = DcContext::new().inspect_err(|_| {
        unsafe { dc_descriptor_free(descriptor) };
    })?;

    // Open the I/O stream for the selected transport.
    let iostream = {
        #[cfg(not(target_os = "android"))]
        let result = match &transport_arg {
            TransportArg::Ble { address } => {
                let handle = tokio::runtime::Handle::current();
                crate::dc::transport::ble::open_ble_iostream(&ctx_dc, address, handle)
            }
            _ => open_iostream(&ctx_dc, descriptor, &transport_arg),
        };
        #[cfg(target_os = "android")]
        let result = open_iostream(&ctx_dc, descriptor, &transport_arg);
        result
    };
    let iostream = iostream.inspect_err(|_| {
        unsafe { dc_descriptor_free(descriptor) };
    })?;

    let known_fingerprints = known_dive_ids(&dives);

    // Qt Subsurface identifies devices as "Vendor Product" (e.g. "Shearwater Perdix AI").
    // Use the same combined string so our fingerprint hashes are interoperable with the
    // Qt logbook's 00-Subsurface fingerprint records.
    let full_model = format!("{vendor} {model}");

    let mut download_ctx = DownloadCtx {
        app: app.clone(),
        logbook_root,
        descriptor,
        ctx_ptr: ctx_dc.as_ptr(),
        dc_model: full_model,
        device_id: None,
        device_serial: None,
        settings,
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
        // DC_EVENT_DEVINFO fires before dives arrive, so dc_device_set_fingerprint is
        // already set (if we have one) by the time enumeration begins.
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

    // Persist the newest fingerprint to disk so the next download can skip already-seen dives.
    let newest_fp = if !download_ctx.cancel.load(Ordering::Relaxed) {
        if let (Some(serial), Some(fp)) =
            (download_ctx.device_serial, download_ctx.newest_fingerprint)
        {
            upsert_fp(&download_ctx.logbook_root, &download_ctx.dc_model, serial, &fp).ok();
            Some((serial, fp))
        } else {
            None
        }
    } else {
        None
    };

    Ok(DownloadResult {
        added: download_ctx.added,
        skipped: download_ctx.skipped,
        newest_fp,
    })
}

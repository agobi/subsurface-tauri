// AI-generated (Claude)
use std::ffi::{c_void, CStr};
use std::ptr;
use crate::dc::ffi::*;

pub struct DcContext(pub *mut dc_context_t);

// Safety: libdivecomputer context is used on a single thread (always inside spawn_blocking).
unsafe impl Send for DcContext {}

unsafe extern "C" fn log_callback(
    _context: *mut dc_context_t,
    loglevel: dc_loglevel_t,
    file: *const ::std::os::raw::c_char,
    line: ::std::os::raw::c_uint,
    _function: *const ::std::os::raw::c_char,
    message: *const ::std::os::raw::c_char,
    _userdata: *mut c_void,
) {
    let file_str = if file.is_null() { "?" } else { CStr::from_ptr(file).to_str().unwrap_or("?") };
    let msg_str = if message.is_null() { "" } else { CStr::from_ptr(message).to_str().unwrap_or("") };
    if loglevel == dc_loglevel_t_DC_LOGLEVEL_ERROR {
        log::error!("[libdc] {}:{} {}", file_str, line, msg_str);
    } else if loglevel == dc_loglevel_t_DC_LOGLEVEL_WARNING {
        log::warn!("[libdc] {}:{} {}", file_str, line, msg_str);
    } else {
        // DC_LOGLEVEL_INFO and DC_LOGLEVEL_DEBUG both map to Rust debug. Audited against
        // libdc source (src-tauri/libdc, commit bae1fc97): iostream.c logs every raw
        // Read/Write/Ioctl transfer plus transport events (Open/Configure/Poll/...) at
        // DC_LOGLEVEL_INFO exclusively (see the INFO()/HEXDUMP() call sites there and in
        // device.c's Fingerprint dump); DC_LOGLEVEL_DEBUG is used only by per-vendor
        // parsers for already-decoded artifacts (Version/Model/Firmware/cmd-rcv frames).
        // So libdc's INFO *is* the raw byte-level firehose — there's no finer-grained
        // level to promote to Rust's info without also promoting the hex dumps.
        log::debug!("[libdc] {}:{} {}", file_str, line, msg_str);
    }
}

impl DcContext {
    pub fn new() -> Result<Self, String> {
        let mut ctx = ptr::null_mut();
        let rc = unsafe { dc_context_new(&mut ctx) };
        if rc == dc_status_t_DC_STATUS_SUCCESS {
            unsafe {
                dc_context_set_loglevel(ctx, dc_loglevel_t_DC_LOGLEVEL_ALL);
                dc_context_set_logfunc(ctx, Some(log_callback), ptr::null_mut());
            }
            Ok(DcContext(ctx))
        } else {
            Err(format!("dc_context_new failed: {rc}"))
        }
    }

    pub fn as_ptr(&self) -> *mut dc_context_t {
        self.0
    }
}

impl Drop for DcContext {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { dc_context_free(self.0) };
        }
    }
}

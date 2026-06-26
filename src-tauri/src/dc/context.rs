// AI-generated (Claude)
use std::ptr;
use crate::dc::ffi::*;

pub struct DcContext(pub *mut dc_context_t);

// Safety: libdivecomputer context is used on a single thread (always inside spawn_blocking).
unsafe impl Send for DcContext {}

impl DcContext {
    pub fn new() -> Result<Self, String> {
        let mut ctx = ptr::null_mut();
        let rc = unsafe { dc_context_new(&mut ctx) };
        if rc == dc_status_t_DC_STATUS_SUCCESS {
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

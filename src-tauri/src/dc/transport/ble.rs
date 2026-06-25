// AI-generated (Claude)
//! BLE transport stub for libdivecomputer.
//!
//! The full iostream implementation requires the characteristic UUIDs for the
//! target device (e.g. `SHEARWATER_WRITE_UUID` / `SHEARWATER_READ_UUID` from
//! `libdc/src/shearwater_common.c`). Extract those UUIDs after the submodule
//! is added and replace the placeholder strings below.
//!
//! Required work to complete this module:
//! 1. Extract read/write characteristic UUIDs from libdc source.
//! 2. Implement `ble_read` using a `std::sync::mpsc::Receiver` fed by btleplug
//!    notification callbacks (subscribe to the notify characteristic).
//! 3. Implement `ble_write` via `peripheral.write()` using
//!    `tokio::runtime::Handle::block_on`.
//! 4. Allocate a `dc_custom_io_t` (once it is available in the generated FFI),
//!    fill its function pointers, and call `dc_custom_open` to produce the
//!    `*mut dc_iostream_t`.

use crate::dc::ffi::dc_iostream_t;
use tokio::runtime::Handle;

// Placeholder UUIDs — replace with values from libdc/src/shearwater_common.c.
#[allow(dead_code)]
const SHEARWATER_WRITE_UUID: &str = "27b7570b-359e-45a3-91bb-cf7e70049bd2";
#[allow(dead_code)]
const SHEARWATER_READ_UUID: &str = "27b7570b-359e-45a3-91bb-cf7e70049bd3";

/// Opens a BLE iostream for the named peripheral.
///
/// **Not yet implemented.** Returns `Err` until the characteristic UUIDs are
/// confirmed from `libdc/src/shearwater_common.c` and `dc_custom_io_t` is
/// exposed in the generated FFI bindings.
pub fn open_ble_iostream(
    _peripheral_name: &str,
    _handle: Handle,
) -> Result<*mut dc_iostream_t, String> {
    Err(
        "BLE iostream: implement after verifying characteristic UUIDs \
         from libdc/src/shearwater_common.c"
            .to_string(),
    )
}

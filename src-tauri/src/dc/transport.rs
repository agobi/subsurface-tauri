// AI-generated (Claude)
use std::ptr;
use crate::dc::ffi::*;
use crate::dc::context::DcContext;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum TransportArg {
    Serial { port: String },
    UsbHid,
    Bluetooth { address: String },
    Ble { name: String },
}

/// Opens a wired iostream for the given transport. BLE is handled separately.
/// Returns `Ok(iostream_ptr)` on success; caller must close with `dc_iostream_close`.
pub fn open_iostream(
    ctx: &DcContext,
    descriptor: *mut dc_descriptor_t,
    transport: &TransportArg,
) -> Result<*mut dc_iostream_t, String> {
    let mut iostream = ptr::null_mut();
    let rc = unsafe {
        match transport {
            TransportArg::Serial { port } => {
                let c_port = std::ffi::CString::new(port.as_str())
                    .map_err(|e| e.to_string())?;
                dc_serial_open(&mut iostream, ctx.as_ptr(), c_port.as_ptr())
            }
            TransportArg::UsbHid => {
                // Iterate USB HID devices to find one matching the descriptor.
                let mut hid_iter = ptr::null_mut();
                let rc = dc_usbhid_iterator_new(&mut hid_iter, ctx.as_ptr(), descriptor);
                if rc != dc_status_t_DC_STATUS_SUCCESS {
                    return Err(format!("dc_usbhid_iterator_new failed: {rc}"));
                }
                let mut hid_dev: *mut dc_usbhid_device_t = ptr::null_mut();
                let rc2 = dc_iterator_next(
                    hid_iter,
                    &mut hid_dev as *mut *mut dc_usbhid_device_t as *mut _,
                );
                dc_iterator_free(hid_iter);
                if rc2 != dc_status_t_DC_STATUS_SUCCESS {
                    return Err("no matching USB HID device found".to_string());
                }
                let rc3 = dc_usbhid_open(&mut iostream, ctx.as_ptr(), hid_dev);
                dc_usbhid_device_free(hid_dev);
                rc3
            }
            TransportArg::Bluetooth { address } => {
                // dc_bluetooth_open takes a numeric address (dc_bluetooth_address_t = u64).
                let addr = dc_bluetooth_str2addr(
                    std::ffi::CString::new(address.as_str())
                        .map_err(|e| e.to_string())?
                        .as_ptr(),
                );
                // Port 1 is the RFCOMM channel for most dive computers.
                dc_bluetooth_open(&mut iostream, ctx.as_ptr(), addr, 1)
            }
            TransportArg::Ble { .. } => {
                return Err("BLE iostream must be opened via transport::ble".to_string());
            }
        }
    };
    if rc == dc_status_t_DC_STATUS_SUCCESS {
        Ok(iostream)
    } else {
        Err(format!("iostream open failed: {rc}"))
    }
}

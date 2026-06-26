// AI-generated (Claude)
//! BLE transport — bridges btleplug's async API to libdivecomputer's synchronous
//! iostream callbacks via `dc_custom_open`.
//!
//! The notification stream is forwarded to a `std::sync::mpsc` channel by a spawned
//! tokio task so the blocking `read` callback can use `recv_timeout`.

use std::collections::BTreeSet;
use std::ffi::c_void;
use std::ptr;
use std::sync::mpsc;
use std::time::Duration;
use futures::StreamExt;
use tokio::runtime::Handle;
use btleplug::api::{
    Central, CharPropFlags, Manager as _, Peripheral as _, ScanFilter, Service, WriteType,
};
use btleplug::platform::Manager;
use crate::dc::context::DcContext;
use crate::dc::ffi::*;

// Known BLE serial service UUIDs (ordered by priority, from Subsurface qt-ble.cpp).
const SERIAL_SERVICES: &[&str] = &[
    "0000fefb-0000-1000-8000-00805f9b34fb", // Heinrichs-Weikamp (Telit/Stollmann)
    "2456e1b9-26e2-8f83-e744-f34f01e9d701", // Heinrichs-Weikamp (U-Blox)
    "544e326b-5b72-c6b0-1c46-41c1bc448118", // Mares BlueLink Pro
    "98ae7120-e62e-11e3-badd-0002a5d5c51b", // Suunto (EON Steel/Core, G5)
    "cb3c4555-d670-4670-bc20-b61dbc851e9a", // Pelagic (i770R, i200C, Pro Plus X, Geo 4.0)
    "ca7b0001-f785-4c38-b599-c7c5fbadb034", // Pelagic (i330R, DSX)
    "fdcdeaaa-295d-470e-bf15-04217b7aa0a0", // ScubaPro (G2, G3)
    "fe25c237-0ece-443c-b0aa-e02033e7029d", // Shearwater (Perdix/Teric/Peregrine/Tern)
    "0000fcef-0000-1000-8000-00805f9b34fb", // Divesoft
    "6e400001-b5a3-f393-e0a9-e50e24dc10b8", // Cressi
    "6e400001-b5a3-f393-e0a9-e50e24dcca9e", // Nordic Semi UART
    "00000001-8c3b-4f2c-a59e-8c08224f3253", // Halcyon Symbios
    "84968ffe-d26d-478a-b953-5010bcf58bca", // Seac
];

// Firmware upgrade services to skip.
const UPGRADE_SERVICES: &[&str] = &[
    "00001530-1212-efde-1523-785feabcd123",
    "9e5d1e47-5c13-43a0-8635-82ad38a1386f",
    "a86abc2d-d44c-442e-99f7-80059a873e36",
];

struct BleState {
    peripheral: btleplug::platform::Peripheral,
    write_char: btleplug::api::Characteristic,
    rx: mpsc::Receiver<Vec<u8>>,
    handle: Handle,
    timeout_ms: i32,
}

// Safety: BleState is accessed only from the single blocking thread that runs libdc.
unsafe impl Send for BleState {}

/// Opens a BLE iostream for the device identified by `address` (the peripheral's
/// platform ID string — a CoreBluetooth UUID on macOS, a MAC address on Linux).
pub fn open_ble_iostream(
    ctx: &DcContext,
    address: &str,
    handle: Handle,
) -> Result<*mut dc_iostream_t, String> {
    let state = handle.block_on(connect_ble(address, handle.clone()))?;
    let userdata = Box::into_raw(Box::new(state)) as *mut c_void;

    let cbs = dc_custom_cbs_t {
        set_timeout: Some(ble_set_timeout),
        set_break: None,
        set_dtr: None,
        set_rts: None,
        get_lines: None,
        get_available: None,
        configure: Some(ble_configure),
        poll: None,
        read: Some(ble_read),
        write: Some(ble_write),
        ioctl: Some(ble_ioctl),
        flush: None,
        purge: Some(ble_purge),
        sleep: None,
        close: Some(ble_close),
    };

    let mut iostream = ptr::null_mut();
    let rc = unsafe {
        dc_custom_open(
            &mut iostream,
            ctx.as_ptr(),
            dc_transport_t_DC_TRANSPORT_BLE,
            &cbs,
            userdata,
        )
    };

    if rc == dc_status_t_DC_STATUS_SUCCESS {
        Ok(iostream)
    } else {
        unsafe { drop(Box::from_raw(userdata as *mut BleState)) };
        Err(format!("dc_custom_open failed: {rc}"))
    }
}

async fn connect_ble(address: &str, handle: Handle) -> Result<BleState, String> {
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
    let adapter = adapters.into_iter().next().ok_or("no BLE adapter found")?;

    // Short scan to populate the adapter's peripheral cache.
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| e.to_string())?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    adapter.stop_scan().await.ok();

    let peripheral = adapter
        .peripherals()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|p| p.id().to_string() == address)
        .ok_or_else(|| format!("peripheral {address} not found after scan"))?;

    peripheral
        .connect()
        .await
        .map_err(|e| format!("BLE connect: {e}"))?;

    peripheral
        .discover_services()
        .await
        .map_err(|e| format!("BLE service discovery: {e}"))?;

    let services = peripheral.services();
    let preferred =
        find_preferred_service(&services).ok_or("no suitable BLE serial service found")?;

    let write_char = preferred
        .characteristics
        .iter()
        .find(|c| {
            c.properties
                .intersects(CharPropFlags::WRITE | CharPropFlags::WRITE_WITHOUT_RESPONSE)
        })
        .cloned()
        .ok_or("no write characteristic")?;

    let notify_char = preferred
        .characteristics
        .iter()
        .find(|c| {
            c.properties
                .intersects(CharPropFlags::NOTIFY | CharPropFlags::INDICATE)
        })
        .cloned()
        .ok_or("no notify characteristic")?;

    // Get the notification stream before subscribing so we don't miss the first packet.
    let mut stream = peripheral
        .notifications()
        .await
        .map_err(|e| e.to_string())?;

    peripheral
        .subscribe(&notify_char)
        .await
        .map_err(|e| format!("BLE subscribe: {e}"))?;

    // Forward async notifications into a sync mpsc channel for the blocking read callback.
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    handle.spawn(async move {
        while let Some(notif) = stream.next().await {
            if tx.send(notif.value).is_err() {
                break;
            }
        }
    });

    Ok(BleState {
        peripheral,
        write_char,
        rx,
        handle,
        timeout_ms: 12_000,
    })
}

/// Select the preferred GATT service: known serial service UUIDs first (in priority
/// order), then heuristic fallback (non-upgrade service with both write and notify).
fn find_preferred_service(services: &BTreeSet<Service>) -> Option<Service> {
    for &known in SERIAL_SERVICES {
        if let Some(svc) = services
            .iter()
            .find(|s| s.uuid.to_string().to_lowercase() == known && has_read_and_write(s))
        {
            return Some(svc.clone());
        }
    }
    services
        .iter()
        .find(|s| {
            let u = s.uuid.to_string().to_lowercase();
            !UPGRADE_SERVICES.iter().any(|&up| up == u) && has_read_and_write(s)
        })
        .cloned()
}

fn has_read_and_write(svc: &Service) -> bool {
    let has_write = svc.characteristics.iter().any(|c| {
        c.properties
            .intersects(CharPropFlags::WRITE | CharPropFlags::WRITE_WITHOUT_RESPONSE)
    });
    let has_notify = svc.characteristics.iter().any(|c| {
        c.properties
            .intersects(CharPropFlags::NOTIFY | CharPropFlags::INDICATE | CharPropFlags::READ)
    });
    has_write && has_notify
}

// --- libdivecomputer custom iostream callbacks ---

unsafe extern "C" fn ble_set_timeout(userdata: *mut c_void, timeout: i32) -> dc_status_t {
    (*(userdata as *mut BleState)).timeout_ms = timeout;
    dc_status_t_DC_STATUS_SUCCESS
}

unsafe extern "C" fn ble_configure(
    _userdata: *mut c_void,
    _baudrate: u32,
    _databits: u32,
    _parity: dc_parity_t,
    _stopbits: dc_stopbits_t,
    _flowcontrol: dc_flowcontrol_t,
) -> dc_status_t {
    dc_status_t_DC_STATUS_SUCCESS // BLE has no serial configuration
}

unsafe extern "C" fn ble_read(
    userdata: *mut c_void,
    data: *mut c_void,
    size: usize,
    actual: *mut usize,
) -> dc_status_t {
    let state = &mut *(userdata as *mut BleState);
    if !actual.is_null() {
        *actual = 0;
    }
    let dur = Duration::from_millis(state.timeout_ms.max(0) as u64);
    let packet = match state.rx.recv_timeout(dur) {
        Ok(p) => p,
        Err(mpsc::RecvTimeoutError::Timeout) => return dc_status_t_DC_STATUS_TIMEOUT,
        Err(mpsc::RecvTimeoutError::Disconnected) => return dc_status_t_DC_STATUS_IO,
    };
    let n = packet.len().min(size);
    ptr::copy_nonoverlapping(packet.as_ptr(), data as *mut u8, n);
    if !actual.is_null() {
        *actual = n;
    }
    if n < packet.len() {
        dc_status_t_DC_STATUS_IO // packet larger than buffer — truncated
    } else {
        dc_status_t_DC_STATUS_SUCCESS
    }
}

unsafe extern "C" fn ble_write(
    userdata: *mut c_void,
    data: *const c_void,
    size: usize,
    actual: *mut usize,
) -> dc_status_t {
    let state = &mut *(userdata as *mut BleState);
    if !actual.is_null() {
        *actual = 0;
    }
    let bytes = std::slice::from_raw_parts(data as *const u8, size).to_vec();
    let write_type = if state
        .write_char
        .properties
        .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
    {
        WriteType::WithoutResponse
    } else {
        WriteType::WithResponse
    };
    match state
        .handle
        .block_on(state.peripheral.write(&state.write_char, &bytes, write_type))
    {
        Ok(()) => {
            if !actual.is_null() {
                *actual = size;
            }
            dc_status_t_DC_STATUS_SUCCESS
        }
        Err(_) => dc_status_t_DC_STATUS_IO,
    }
}

unsafe extern "C" fn ble_ioctl(
    _userdata: *mut c_void,
    _request: u32,
    _data: *mut c_void,
    _size: usize,
) -> dc_status_t {
    // Shearwater does not require BLE ioctl for download.
    dc_status_t_DC_STATUS_UNSUPPORTED
}

unsafe extern "C" fn ble_purge(
    userdata: *mut c_void,
    direction: dc_direction_t,
) -> dc_status_t {
    if direction & dc_direction_t_DC_DIRECTION_INPUT != 0 {
        let state = &mut *(userdata as *mut BleState);
        while state.rx.try_recv().is_ok() {}
    }
    dc_status_t_DC_STATUS_SUCCESS
}

unsafe extern "C" fn ble_close(userdata: *mut c_void) -> dc_status_t {
    let state = Box::from_raw(userdata as *mut BleState);
    state.handle.block_on(state.peripheral.disconnect()).ok();
    dc_status_t_DC_STATUS_SUCCESS
}

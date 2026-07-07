// AI-generated (Claude)
use serde::de::DeserializeOwned;
use tauri::{
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<DcBle<R>> {
    let handle =
        api.register_android_plugin("org.subsurfacedivelog.prototype.dcble", "DcBlePlugin")?;
    Ok(DcBle(handle))
}

/// Native Rust access to the Android BLE plugin. Every method blocks the calling
/// thread until the Kotlin side resolves or rejects the invoke — callers must run
/// on a thread that's allowed to block (e.g. inside `tauri::async_runtime::spawn_blocking`
/// or libdivecomputer's own dedicated download thread), never on the main/UI thread.
pub struct DcBle<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> DcBle<R> {
    pub fn connect(&self, address: &str, channel: Channel<BleEvent>) -> crate::Result<()> {
        self.0
            .run_mobile_plugin(
                "connect",
                ConnectArgs { address: address.to_string(), channel },
            )
            .map_err(Into::into)
    }

    pub fn write(&self, bytes: Vec<u8>) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("write", WriteArgs { bytes })
            .map_err(Into::into)
    }

    pub fn disconnect(&self) -> crate::Result<()> {
        self.0.run_mobile_plugin("disconnect", ()).map_err(Into::into)
    }

    pub fn scan(&self, vendor: String, model: String, channel: Channel<ScanResult>) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("scan", ScanArgs { vendor, model, channel })
            .map_err(Into::into)
    }

    pub fn ensure_permissions(&self) -> crate::Result<EnsurePermissionsResponse> {
        self.0
            .run_mobile_plugin("ensurePermissions", ())
            .map_err(Into::into)
    }

    pub fn open_app_settings(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("openAppSettings", ())
            .map_err(Into::into)
    }
}

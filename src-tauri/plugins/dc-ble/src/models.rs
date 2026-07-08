// AI-generated (Claude)
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectArgs {
    pub address: String,
    pub channel: Channel<BleEvent>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteArgs {
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanArgs {
    pub vendor: String,
    pub model: String,
    pub channel: Channel<ScanResult>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub name: String,
    pub address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsurePermissionsResponse {
    pub granted: bool,
}

/// Payload pushed through a connection's notification `Channel`. Matches the
/// `{"type": "data", "bytes": [...]}` / `{"type": "disconnected"}` shapes
/// `DcBlePlugin.kt`'s `connect` command sends via `Channel.sendObject`.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum BleEvent {
    Data { bytes: Vec<u8> },
    Disconnected,
}

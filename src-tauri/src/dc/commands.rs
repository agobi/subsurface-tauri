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

#[cfg(test)]
mod tests {
    #[test]
    fn list_serial_ports_returns_vec() {
        // Just verify it doesn't panic; actual ports depend on hardware.
        let _ports = super::serial_ports_impl();
        // passes if it returns without panicking
    }
}

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

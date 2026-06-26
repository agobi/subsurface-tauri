// AI-generated (Claude)
use std::collections::HashSet;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_KEY: &str = "dcFingerprints";

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

pub fn write_fp<R: tauri::Runtime>(
    app: &AppHandle<R>,
    device_id: &str,
    fingerprint: &[u8],
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let mut map: serde_json::Map<String, serde_json::Value> = store
        .get(STORE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    map.insert(device_id.to_string(), serde_json::json!(hex_encode(fingerprint)));
    store.set(STORE_KEY, serde_json::Value::Object(map));
    store.save().map_err(|e| e.to_string())
}

/// Collects all dc_dive_id values from the parsed logbook dives as a fingerprint fallback set.
pub fn known_dive_ids(dives: &[crate::types::Dive]) -> HashSet<Vec<u8>> {
    dives
        .iter()
        .filter_map(|d| d.dc_dive_id.as_deref())
        .filter_map(|hex| hex_decode(hex))
        .collect()
}

#[cfg(test)]
mod tests {
    use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
    use tauri::Manager;
    use super::{hex_decode, STORE_KEY};

    fn read_fp<R: tauri::Runtime>(app: &tauri::AppHandle<R>, device_id: &str) -> Option<Vec<u8>> {
        let store = app.store("settings.json").ok()?;
        let map = store.get(STORE_KEY)?;
        let hex = map.get(device_id)?.as_str()?;
        hex_decode(hex)
    }

    fn setup() -> tauri::App<MockRuntime> {
        mock_builder()
            .plugin(tauri_plugin_store::Builder::new().build())
            .build(mock_context(noop_assets()))
            .unwrap()
    }

    #[test]
    fn read_returns_none_when_absent() {
        let app = setup();
        let handle = app.handle();
        // Use a device_id not written by any other test to avoid cross-test contamination
        // (mock_context reuses the same app_data_dir path across test instances).
        assert!(read_fp(handle, "00000000").is_none());
    }

    #[test]
    fn write_then_read_roundtrip() {
        let app = setup();
        let handle = app.handle();
        let fp = vec![0x76u8, 0xb9, 0xbc, 0x25];
        super::write_fp(handle, "a790cf6c", &fp).unwrap();
        let loaded = super::read_fp(handle, "a790cf6c").unwrap();
        assert_eq!(loaded, fp);
    }

    #[test]
    fn known_dive_ids_collects_hex_dive_ids() {
        use crate::types::Dive;
        let dives = vec![
            Dive {
                number: 1,
                date_time: "2024-01-01T10:00:00".to_string(),
                duration_sec: 3600,
                site_id: None,
                tags: vec![],
                rating: None,
                dive_guide: None,
                buddy: None,
                suit: None,
                notes: None,
                cylinders: vec![],
                dc_model: None,
                max_depth_m: None,
                mean_depth_m: None,
                water_temp_c: None,
                deco_model: None,
                divemode: None,
                dc_device_id: None,
                dc_dive_id: Some("76b9bc25".to_string()),
                total_weight_kg: None,
                samples: vec![],
                events: vec![],
            },
            Dive {
                number: 2,
                date_time: "2024-01-02T10:00:00".to_string(),
                duration_sec: 1800,
                site_id: None,
                tags: vec![],
                rating: None,
                dive_guide: None,
                buddy: None,
                suit: None,
                notes: None,
                cylinders: vec![],
                dc_model: None,
                max_depth_m: None,
                mean_depth_m: None,
                water_temp_c: None,
                deco_model: None,
                divemode: None,
                dc_device_id: None,
                dc_dive_id: None,
                total_weight_kg: None,
                samples: vec![],
                events: vec![],
            },
        ];
        let ids = super::known_dive_ids(&dives);
        assert!(ids.contains(&vec![0x76u8, 0xb9, 0xbc, 0x25]));
        assert_eq!(ids.len(), 1);
    }
}

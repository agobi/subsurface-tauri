// AI-generated (Claude)
use std::collections::HashSet;
use std::path::Path;
use crate::ssrf_git::settings::{read_settings, write_settings, sha1_u32, device_id_hash, FingerprintRecord, DeviceRecord, Settings};

/// Returns the raw fingerprint bytes for the given device, or `None` if absent.
pub fn lookup_fp(settings: &Settings, model_name: &str, serial: u32) -> Option<Vec<u8>> {
    let model_hash = sha1_u32(model_name.as_bytes());
    settings.fingerprints.iter()
        .find(|r| r.model == model_hash && r.serial == serial)
        .map(|r| r.data.clone())
}

/// Inserts or replaces the fingerprint for the given device in `settings` (in-memory only).
pub fn apply_fp(settings: &mut Settings, model_name: &str, serial: u32, fp_bytes: &[u8]) {
    let model_hash = sha1_u32(model_name.as_bytes());
    let device_id = device_id_hash(serial);
    let dive_id = sha1_u32(fp_bytes);
    settings.fingerprints.retain(|r| !(r.model == model_hash && r.serial == serial));
    settings.fingerprints.push(FingerprintRecord {
        model: model_hash,
        serial,
        device_id,
        dive_id,
        data: fp_bytes.to_vec(),
    });
}

/// Reads `<logbook_root>/00-Subsurface`, inserts or replaces the fingerprint for
/// the given device, and writes the file back.
pub fn upsert_fp(logbook_root: &Path, model_name: &str, serial: u32, fp_bytes: &[u8]) -> Result<(), String> {
    let mut settings = read_settings(logbook_root);
    apply_fp(&mut settings, model_name, serial, fp_bytes);
    write_settings(logbook_root, &settings)
}

/// Inserts or replaces the device record for (model, serial) in `settings`
/// (in-memory only). Preserves an existing nickname — this code never sets
/// one, but Qt's UI may have.
pub fn apply_device(settings: &mut Settings, model: &str, serial: u32) {
    let device_id = device_id_hash(serial);
    let serial_str = format!("{serial:08x}");
    let nickname = settings.devices.iter()
        .find(|d| d.model == model && d.serial == serial_str)
        .map(|d| d.nickname.clone())
        .unwrap_or_default();
    settings.devices.retain(|d| !(d.model == model && d.serial == serial_str));
    settings.devices.push(DeviceRecord {
        model: model.to_string(),
        device_id,
        serial: serial_str,
        nickname,
    });
}

/// Reads `<logbook_root>/00-Subsurface`, inserts or replaces the device
/// record for the given device, and writes the file back.
pub fn upsert_device(logbook_root: &Path, model: &str, serial: u32) -> Result<(), String> {
    let mut settings = read_settings(logbook_root);
    apply_device(&mut settings, model, serial);
    write_settings(logbook_root, &settings)
}

/// Returns the set of SHA1_uint32(fingerprint) values for all dives in the logbook.
///
/// Each Divecomputer file stores `diveid` as SHA1_uint32(raw_fingerprint_bytes),
/// matching original Subsurface's calculate_diveid() / has_dive() convention.
/// During download, compare sha1_u32(incoming_fp) against this set to skip duplicates.
pub fn known_dive_ids(dives: &[crate::types::Dive]) -> HashSet<u32> {
    dives
        .iter()
        .filter_map(|d| d.dc_dive_id.as_deref())
        .filter_map(|s| u32::from_str_radix(s, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssrf_git::settings::{read_settings, Settings};

    #[test]
    fn apply_fp_inserts_when_absent() {
        let mut s = Settings::default();
        apply_fp(&mut s, "Shearwater Perdix", 123456, &[0x01, 0x02]);
        assert_eq!(s.fingerprints.len(), 1);
        let fp = lookup_fp(&s, "Shearwater Perdix", 123456).unwrap();
        assert_eq!(fp, vec![0x01u8, 0x02]);
    }

    #[test]
    fn apply_fp_replaces_existing() {
        let mut s = Settings::default();
        apply_fp(&mut s, "Shearwater Perdix", 123456, &[0x01, 0x02]);
        apply_fp(&mut s, "Shearwater Perdix", 123456, &[0xAA, 0xBB]);
        assert_eq!(s.fingerprints.len(), 1, "second apply must replace, not append");
        let fp = lookup_fp(&s, "Shearwater Perdix", 123456).unwrap();
        assert_eq!(fp, vec![0xAAu8, 0xBB]);
    }

    #[test]
    fn apply_fp_isolated_per_serial() {
        let mut s = Settings::default();
        apply_fp(&mut s, "Shearwater Perdix", 111111, &[0x01]);
        apply_fp(&mut s, "Shearwater Perdix", 222222, &[0x02]);
        assert_eq!(s.fingerprints.len(), 2);
        assert_eq!(lookup_fp(&s, "Shearwater Perdix", 111111).unwrap(), vec![0x01u8]);
        assert_eq!(lookup_fp(&s, "Shearwater Perdix", 222222).unwrap(), vec![0x02u8]);
    }

    #[test]
    fn lookup_returns_none_when_absent() {
        let tmp = std::env::temp_dir().join("fp_lookup_absent");
        std::fs::create_dir_all(&tmp).unwrap();
        let s = read_settings(&tmp);
        assert!(lookup_fp(&s, "Shearwater Perdix", 123456).is_none());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn upsert_then_lookup_roundtrip() {
        let tmp = std::env::temp_dir().join("fp_upsert_rt");
        std::fs::create_dir_all(&tmp).unwrap();
        upsert_fp(&tmp, "Shearwater Perdix", 123456, &[0x76, 0xb9, 0xbc, 0x25]).unwrap();
        let s = read_settings(&tmp);
        let fp = lookup_fp(&s, "Shearwater Perdix", 123456).unwrap();
        assert_eq!(fp, vec![0x76u8, 0xb9, 0xbc, 0x25]);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn upsert_replaces_existing() {
        let tmp = std::env::temp_dir().join("fp_upsert_replace");
        std::fs::create_dir_all(&tmp).unwrap();
        upsert_fp(&tmp, "Shearwater Perdix", 123456, &[0x76, 0xb9, 0xbc, 0x25]).unwrap();
        upsert_fp(&tmp, "Shearwater Perdix", 123456, &[0xaa, 0xbb]).unwrap();
        let s = read_settings(&tmp);
        assert_eq!(s.fingerprints.len(), 1, "second upsert must replace, not append");
        let fp = lookup_fp(&s, "Shearwater Perdix", 123456).unwrap();
        assert_eq!(fp, vec![0xaa, 0xbb]);
        std::fs::remove_dir_all(&tmp).ok();
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
                media_count: 0,
                otu: None,
                max_cns: None,
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
                media_count: 0,
                otu: None,
                max_cns: None,
                samples: vec![],
                events: vec![],
            },
        ];
        let ids = known_dive_ids(&dives);
        // "76b9bc25" parsed as hex u32 = 0x76b9bc25
        assert!(ids.contains(&0x76b9bc25u32));
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn apply_device_inserts_when_absent() {
        let mut s = Settings::default();
        apply_device(&mut s, "Shearwater Perdix", 123456);
        assert_eq!(s.devices.len(), 1);
        assert_eq!(s.devices[0].model, "Shearwater Perdix");
        assert_eq!(s.devices[0].serial, "0001e240");
        assert_eq!(s.devices[0].device_id, device_id_hash(123456));
        assert_eq!(s.devices[0].nickname, "");
    }

    #[test]
    fn apply_device_replaces_existing_without_duplicating() {
        let mut s = Settings::default();
        apply_device(&mut s, "Shearwater Perdix", 123456);
        apply_device(&mut s, "Shearwater Perdix", 123456);
        assert_eq!(s.devices.len(), 1, "second apply must replace, not append");
    }

    #[test]
    fn apply_device_preserves_an_existing_nickname() {
        use crate::ssrf_git::settings::DeviceRecord;
        let mut s = Settings::default();
        s.devices.push(DeviceRecord {
            model: "Shearwater Perdix".to_string(),
            device_id: device_id_hash(123456),
            serial: "0001e240".to_string(),
            nickname: "My Perdix".to_string(),
        });
        apply_device(&mut s, "Shearwater Perdix", 123456);
        assert_eq!(s.devices.len(), 1);
        assert_eq!(s.devices[0].nickname, "My Perdix", "a nickname set elsewhere (e.g. by Qt) must survive our upsert");
    }

    #[test]
    fn apply_device_isolated_per_serial() {
        let mut s = Settings::default();
        apply_device(&mut s, "Shearwater Perdix", 111111);
        apply_device(&mut s, "Shearwater Perdix", 222222);
        assert_eq!(s.devices.len(), 2);
    }

    #[test]
    fn upsert_device_then_read_back_roundtrip() {
        let tmp = std::env::temp_dir().join("fp_upsert_device_rt");
        std::fs::create_dir_all(&tmp).unwrap();
        upsert_device(&tmp, "Shearwater Perdix", 123456).unwrap();
        let s = read_settings(&tmp);
        assert_eq!(s.devices.len(), 1);
        assert_eq!(s.devices[0].model, "Shearwater Perdix");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn upsert_device_replaces_existing_on_disk() {
        let tmp = std::env::temp_dir().join("fp_upsert_device_replace");
        std::fs::create_dir_all(&tmp).unwrap();
        upsert_device(&tmp, "Shearwater Perdix", 123456).unwrap();
        upsert_device(&tmp, "Shearwater Perdix", 123456).unwrap();
        let s = read_settings(&tmp);
        assert_eq!(s.devices.len(), 1, "second upsert must replace, not append");
        std::fs::remove_dir_all(&tmp).ok();
    }
}

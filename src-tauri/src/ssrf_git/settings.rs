// AI-generated (Claude)
//! Parser and writer for the `00-Subsurface` logbook settings file.
use std::path::Path;
use super::tokenize::{split_keyword, parse_attrs, unquote};

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub version: Option<u32>,
    /// Unit system: "METRIC" (default) or "IMPERIAL".
    pub units: String,
    pub autogroup: bool,
    pub devices: Vec<DeviceRecord>,
    pub fingerprints: Vec<FingerprintRecord>,
    pub prefs: Vec<String>,
    /// Verbatim pass-through for unknown/future lines.
    pub other_lines: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: None,
            units: "METRIC".to_string(),
            autogroup: false,
            devices: vec![],
            fingerprints: vec![],
            prefs: vec![],
            other_lines: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceRecord {
    pub model: String,
    pub device_id: u32,
    pub serial: String,
    pub nickname: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintRecord {
    /// sha1_u32(model_name.as_bytes())
    pub model: u32,
    /// raw DC_EVENT_DEVINFO.serial
    pub serial: u32,
    /// sha1_u32(serial.to_string().as_bytes())
    pub device_id: u32,
    /// sha1_u32(fingerprint_bytes)
    pub dive_id: u32,
    /// raw fingerprint bytes from libdivecomputer
    pub data: Vec<u8>,
}

/// First 4 bytes of SHA-1(data) interpreted as a little-endian u32.
/// Matches Subsurface's `SHA1::hash_uint32()`.
pub fn sha1_u32(data: &[u8]) -> u32 {
    use sha1::{Sha1, Digest};
    let hash = Sha1::digest(data);
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

/// Hashes a libdivecomputer `devinfo.serial` value the way Subsurface does for
/// `deviceid`: as a hex string (`"%08x"`), not a decimal string.
///
/// Subsurface's deviceid normally comes from the dive parser's own embedded
/// "Serial" string field (hex-formatted by libdivecomputer parsers, e.g.
/// `shearwater_predator_parser.c`), falling back to a decimal-formatted
/// `devinfo.serial` only when the parser provides no string. Since the common
/// case (and the one we replicate here) is the hex-formatted parser string,
/// hash that representation.
pub fn device_id_hash(serial: u32) -> u32 {
    sha1_u32(format!("{serial:08x}").as_bytes())
}

fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 { return None; }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_divecomputerid(rest: &str) -> DeviceRecord {
    let rest = rest.trim();
    let (model, attrs_str) = if rest.starts_with('"') {
        let mut i = 1;
        while i < rest.len() {
            if rest.as_bytes()[i] == b'\\' {
                i += 2;
            } else if rest.as_bytes()[i] == b'"' {
                i += 1;
                break;
            } else {
                i += 1;
            }
        }
        (unquote(&rest[..i]), rest[i..].trim_start())
    } else {
        let sp = rest.find(' ').unwrap_or(rest.len());
        (rest[..sp].to_string(), rest[sp..].trim_start())
    };
    let attrs = parse_attrs(attrs_str);
    DeviceRecord {
        model,
        device_id: u32::from_str_radix(attrs.get("deviceid").map_or("0", |s| s), 16).unwrap_or(0),
        serial: attrs.get("serial").cloned().unwrap_or_default(),
        nickname: attrs.get("nickname").cloned().unwrap_or_default(),
    }
}

fn format_divecomputerid(d: &DeviceRecord) -> String {
    // Original uses show_utf8() which always quotes — serial must be quoted to match.
    format!(
        "divecomputerid \"{}\" deviceid={:08x} serial=\"{}\" nickname=\"{}\"",
        d.model, d.device_id, d.serial, d.nickname
    )
}

fn parse_fingerprint_line(rest: &str) -> Option<FingerprintRecord> {
    let attrs = parse_attrs(rest);
    Some(FingerprintRecord {
        model: u32::from_str_radix(attrs.get("model")?, 16).ok()?,
        serial: u32::from_str_radix(attrs.get("serial")?, 16).ok()?,
        device_id: u32::from_str_radix(attrs.get("deviceid")?, 16).ok()?,
        dive_id: u32::from_str_radix(attrs.get("diveid")?, 16).ok()?,
        data: hex_decode(attrs.get("data")?)?,
    })
}

fn format_fingerprint(fp: &FingerprintRecord) -> String {
    format!(
        "fingerprint model={:08x} serial={:08x} deviceid={:08x} diveid={:08x} data=\"{}\"",
        fp.model, fp.serial, fp.device_id, fp.dive_id, hex_encode(&fp.data)
    )
}

fn parse_settings_text(text: &str) -> Settings {
    let mut s = Settings::default();
    for line in text.lines() {
        let (key, rest) = split_keyword(line);
        match key {
            "version" => s.version = rest.trim().parse().ok(),
            "autogroup" => s.autogroup = true,
            "units" => {
                let first = rest.split_whitespace().next().unwrap_or("METRIC");
                s.units = match first {
                    "METRIC" | "IMPERIAL" => first.to_string(),
                    other => {
                        log::warn!("unrecognized units value {other:?}, defaulting to METRIC");
                        "METRIC".to_string()
                    }
                };
            }
            "prefs" => {
                s.prefs = rest.split_whitespace().map(|t| t.to_string()).collect();
            }
            "divecomputerid" => s.devices.push(parse_divecomputerid(rest)),
            "fingerprint" => {
                if let Some(fp) = parse_fingerprint_line(rest) {
                    s.fingerprints.push(fp);
                }
            }
            "" => {}
            _ => s.other_lines.push(line.to_string()),
        }
    }
    s
}

/// Reads and parses `<root>/00-Subsurface`. Returns `Settings::default()` on any I/O or
/// parse error.
pub fn read_settings(root: &Path) -> Settings {
    let text = match std::fs::read_to_string(root.join("00-Subsurface")) {
        Ok(t) => t,
        Err(_) => return Settings::default(),
    };
    parse_settings_text(&text)
}

/// Serialises `settings` to `<root>/00-Subsurface` using Subsurface field order:
/// version → divecomputerid → fingerprint → autogroup → units → prefs → other_lines.
pub fn write_settings(root: &Path, settings: &Settings) -> Result<(), String> {
    let mut out = String::new();
    if let Some(v) = settings.version {
        out.push_str(&format!("version {v}\n"));
    }
    for d in &settings.devices {
        out.push_str(&format_divecomputerid(d));
        out.push('\n');
    }
    for fp in &settings.fingerprints {
        out.push_str(&format_fingerprint(fp));
        out.push('\n');
    }
    if settings.autogroup {
        out.push_str("autogroup\n");
    }
    out.push_str(&format!("units {}\n", settings.units));
    if !settings.prefs.is_empty() {
        out.push_str(&format!("prefs {}\n", settings.prefs.join(" ")));
    }
    for line in &settings.other_lines {
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(root.join("00-Subsurface"), out).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_u32_golden() {
        // SHA1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709 (canonical empty hash)
        // first 4 bytes LE: [0xda, 0x39, 0xa3, 0xee] = 0xeea339da
        assert_eq!(sha1_u32(b""), 0xeea3_39da);
        // Different inputs must produce different values.
        assert_ne!(sha1_u32(b"Shearwater Perdix"), sha1_u32(b"Shearwater Perdix AI"));
    }

    #[test]
    fn device_id_hash_uses_hex_serial_format() {
        // Shearwater's own dive parser embeds the serial as a hex string
        // ("%08x", e.g. shearwater_predator_parser.c) and Subsurface hashes
        // that hex string for deviceid — not a decimal-formatted string.
        // Verified against a real device: raw serial 545362529 (0x20819261)
        // must hash to the deviceid already present in this user's existing
        // Divecomputer files (a790cf6c), not sha1_u32("545362529") (a2f7ccf7).
        assert_eq!(device_id_hash(545362529), 0xa790cf6c);
    }

    #[test]
    fn parse_and_write_roundtrip() {
        let tmp = std::env::temp_dir().join("ssrf_settings_rt");
        std::fs::create_dir_all(&tmp).unwrap();

        let original = Settings {
            version: Some(3),
            units: "METRIC".to_string(),
            autogroup: true,
            devices: vec![DeviceRecord {
                model: "Shearwater Perdix".to_string(),
                device_id: 0xa790cf6c,
                serial: "123456".to_string(),
                nickname: "My Perdix".to_string(),
            }],
            fingerprints: vec![FingerprintRecord {
                model: 0x355d7e96,
                serial: 0x0001e240,
                device_id: 0xdeadbeef,
                dive_id: 0xcafebabe,
                data: vec![0x76, 0xb9, 0xbc, 0x25],
            }],
            prefs: vec!["TANKBAR".to_string(), "DCCEILING".to_string()],
            other_lines: vec!["subsurface \"5.0.10\"".to_string()],
        };

        write_settings(&tmp, &original).unwrap();
        let loaded = read_settings(&tmp);
        assert_eq!(loaded, original);

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn read_settings_missing_file_returns_default() {
        let tmp = std::env::temp_dir().join("ssrf_settings_missing");
        std::fs::create_dir_all(&tmp).unwrap();
        // No 00-Subsurface file — must return default without error.
        let s = read_settings(&tmp);
        assert_eq!(s.units, "METRIC");
        assert!(s.fingerprints.is_empty());
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn detects_imperial() {
        let s = parse_settings_text("units IMPERIAL\n");
        assert_eq!(s.units, "IMPERIAL");
    }

    #[test]
    fn defaults_to_metric_when_absent() {
        let s = parse_settings_text("version 3\n");
        assert_eq!(s.units, "METRIC");
    }

    #[test]
    fn falls_back_to_metric_on_unrecognized_units() {
        let s = parse_settings_text("units BOGUS\n");
        assert_eq!(s.units, "METRIC");
    }

}

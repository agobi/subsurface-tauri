// AI-generated (Claude)
mod tokenize;
mod parse_header;
mod parse_site;
mod parse_dive;
mod parse_divecomputer;

use std::path::Path;
use parse_header::parse_header;
use parse_site::parse_site;
use parse_dive::parse_dive;
use parse_divecomputer::parse_divecomputer;
use tokenize::unquote;
use crate::types::{Dive, Logbook, Site, Trip};

fn is_year(s: &str) -> bool { s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) }
fn is_month(s: &str) -> bool { s.len() == 2 && s.chars().all(|c| c.is_ascii_digit()) }

fn is_dive_dir(s: &str) -> bool {
    // Handles [[yyyy-]mm-]nn-ddd-hh=mm=ss[~hex] (= or : as time separator).
    // The distinguishing marker is '=' or ':' at position len-3 of the base name.
    let base = s.split('~').next().unwrap_or(s);
    let b = base.as_bytes();
    b.len() >= 8 && (b[b.len() - 3] == b'=' || b[b.len() - 3] == b':')
}

// Trip dirs start with two digits + '-' but are not dive dirs (e.g. "15-Egypt", "01-trip").
fn is_trip_dir(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 3 && b[0].is_ascii_digit() && b[1].is_ascii_digit() && b[2] == b'-' && !is_dive_dir(s)
}

fn read_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))
}

fn parse_sites(root: &Path) -> Vec<Site> {
    let dir = root.join("01-Divesites");
    let Ok(entries) = std::fs::read_dir(&dir) else { return vec![]; };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("Site-"))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let text = read_file(&e.path()).ok()?;
            Some(parse_site(&name, &text))
        })
        .collect()
}

fn parse_dive_dir(dir: &Path, year: &str, month: &str, dir_name: &str) -> Option<Dive> {
    // Handles [[yyyy-]mm-]nn-ddd-hh=mm=ss[~hex] (= or : as time separator).
    // Strip optional ~hex uniqueness suffix first.
    let base = dir_name.split('~').next().unwrap_or(dir_name);
    if base.len() < 8 { return None; }

    // Last 8 chars are the time: HH=MM=SS or HH:MM:SS.
    let time_str = &base[base.len() - 8..];
    let sep = time_str.as_bytes()[2];
    let sep_char = if sep == b'=' { '=' } else { ':' };
    let tparts: Vec<&str> = time_str.split(sep_char).collect();
    if tparts.len() != 3 || !tparts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    let (hh, mm, ss) = (tparts[0], tparts[1], tparts[2]);

    // Strip the trailing '-HH=MM=SS' (9 chars) then strip the day-name after the last '-'.
    // What remains is the date prefix: "nn", "mm-nn", or "yyyy-mm-nn".
    let before_time = &base[..base.len() - 9];
    let day_name_sep = before_time.rfind('-')?;
    let date_prefix = &before_time[..day_name_sep];
    let parts: Vec<&str> = date_prefix.split('-').collect();
    let (dd, month_out, year_out) = match parts.len() {
        1 => (parts[0], month, year),
        2 => (parts[1], parts[0], year),
        3 => (parts[2], parts[1], parts[0]),
        _ => return None,
    };
    if dd.len() != 2 || !dd.chars().all(|c| c.is_ascii_digit()) { return None; }

    // Find "Dive-NNN" file
    let entries = std::fs::read_dir(dir).ok()?;
    let dive_entry = entries
        .filter_map(|e| e.ok())
        .find(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("Dive-") && n[5..].chars().all(|c| c.is_ascii_digit())
        })?;
    let dive_name = dive_entry.file_name().to_string_lossy().to_string();
    let number: i32 = dive_name[5..].parse().ok()?;
    let overview = parse_dive(&read_file(&dive_entry.path()).ok()?);

    let dc_path = dir.join("Divecomputer");
    let dc = if dc_path.exists() {
        parse_divecomputer(&read_file(&dc_path).ok()?)
    } else {
        parse_divecomputer("")
    };

    Some(Dive {
        number,
        date_time: format!("{year_out}-{month_out}-{dd}T{hh}:{mm}:{ss}"),
        duration_sec: overview.duration_sec,
        site_id: overview.site_id,
        tags: overview.tags,
        rating: overview.rating,
        dive_guide: overview.dive_guide,
        buddy: overview.buddy,
        suit: overview.suit,
        notes: overview.notes,
        cylinders: overview.cylinders,
        dc_model: dc.model,
        max_depth_m: dc.max_depth_m,
        mean_depth_m: dc.mean_depth_m,
        water_temp_c: dc.water_temp_c,
        deco_model: dc.deco_model,
        divemode: dc.divemode,
        total_weight_kg: overview.total_weight_kg,
        samples: dc.samples,
        events: dc.events,
    })
}

// Derives a display label from the trip directory name, e.g. "15-Egypt" → "Egypt".
fn dir_name_label(dir_name: &str) -> String {
    dir_name.split_once('-').map(|x| x.1).unwrap_or(dir_name).replace('-', " ")
}

// Parses the 00-Trip metadata file: "location …" → label, "notes …" → notes.
fn parse_trip_file(text: &str, dir_name: &str) -> (String, Option<String>) {
    let mut label = None;
    let mut notes = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("location ") {
            label = Some(unquote(rest.trim()));
        } else if let Some(rest) = line.strip_prefix("notes ") {
            notes = Some(unquote(rest.trim()));
        }
    }
    (label.unwrap_or_else(|| dir_name_label(dir_name)), notes)
}

// Parses a trip directory: reads 00-Trip for metadata and collects all dive subdirs.
fn parse_trip_dir(dir: &Path, year: &str, month: &str, dir_name: &str) -> Option<(Trip, Vec<Dive>)> {
    let trip_file = dir.join("00-Trip");
    let (label, notes) = if trip_file.exists() {
        let text = read_file(&trip_file).ok()?;
        parse_trip_file(&text, dir_name)
    } else {
        (dir_name_label(dir_name), None)
    };

    let Ok(entries) = std::fs::read_dir(dir) else { return None; };
    let mut trip_dives: Vec<Dive> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && is_dive_dir(&e.file_name().to_string_lossy()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            parse_dive_dir(&e.path(), year, month, &name)
        })
        .collect();

    if trip_dives.is_empty() { return None; }
    trip_dives.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    let dive_numbers: Vec<i32> = trip_dives.iter().map(|d| d.number).collect();
    Some((Trip { label, area: None, notes, dive_numbers }, trip_dives))
}

pub fn parse_logbook(root: &Path) -> Result<Logbook, String> {
    let header_path = root.join("00-Subsurface");
    let units = if header_path.exists() {
        parse_header(&read_file(&header_path)?)
    } else {
        "METRIC".to_owned()
    };

    let sites = parse_sites(root);
    let mut dives: Vec<Dive> = vec![];
    let mut trips: Vec<Trip> = vec![];

    let year_entries = std::fs::read_dir(root)
        .map_err(|e| format!("cannot read {}: {}", root.display(), e))?;

    for year_entry in year_entries.filter_map(|e| e.ok()) {
        let year = year_entry.file_name().to_string_lossy().to_string();
        if !is_year(&year) { continue; }
        let year_dir = year_entry.path();

        let Ok(month_entries) = std::fs::read_dir(&year_dir) else { continue; };
        for month_entry in month_entries.filter_map(|e| e.ok()) {
            let month = month_entry.file_name().to_string_lossy().to_string();
            if !is_month(&month) { continue; }
            let month_dir = month_entry.path();

            let Ok(slot_entries) = std::fs::read_dir(&month_dir) else { continue; };
            for slot_entry in slot_entries.filter_map(|e| e.ok()) {
                let name = slot_entry.file_name().to_string_lossy().to_string();
                let path = slot_entry.path();
                if !path.is_dir() { continue; }

                if is_dive_dir(&name) {
                    if let Some(dive) = parse_dive_dir(&path, &year, &month, &name) {
                        dives.push(dive);
                    }
                } else if is_trip_dir(&name) {
                    if let Some((trip, trip_dives)) = parse_trip_dir(&path, &year, &month, &name) {
                        dives.extend(trip_dives);
                        trips.push(trip);
                    }
                }
            }
        }
    }

    dives.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(Logbook { dives, trips, sites, units })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> PathBuf {
        // CARGO_MANIFEST_DIR = desktop-tauri/src-tauri
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()        // desktop-tauri
            .join("test/fixtures/git-tree")
    }

    fn fixture_trips() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("test/fixtures/git-tree-trips")
    }

    #[test]
    fn golden_units_and_sites() {
        let lb = parse_logbook(&fixture()).unwrap();
        assert_eq!(lb.units, "METRIC");
        assert_eq!(lb.sites.len(), 1);
        let s = &lb.sites[0];
        assert_eq!(s.id, "04782ed8");
        assert_eq!(s.name, "Test Spring");
        let gps = s.gps.as_ref().unwrap();
        assert!((gps.lat - 47.668408).abs() < 1e-6);
        assert!((gps.lon - 18.307076).abs() < 1e-6);
        assert_eq!(s.country.as_deref(), Some("Testland"));
    }

    #[test]
    fn golden_dive_assembly() {
        let lb = parse_logbook(&fixture()).unwrap();
        assert_eq!(lb.dives.len(), 1);
        let d = &lb.dives[0];
        assert_eq!(d.number, 1);
        assert_eq!(d.date_time, "2024-03-15T12:28:43");
        assert_eq!(d.duration_sec, 300);
        assert_eq!(d.rating, Some(4));
        assert_eq!(d.tags, vec!["cave"]);
        assert_eq!(d.site_id.as_deref(), Some("04782ed8"));
        assert_eq!(d.dc_model.as_deref(), Some("Shearwater Perdix AI"));
        assert!((d.max_depth_m.unwrap() - 34.7).abs() < 1e-6);
        assert_eq!(d.deco_model.as_deref(), Some("GF 55/85"));
        assert_eq!(d.cylinders[0].description, "D12 232 bar");
        assert_eq!(d.divemode.as_deref(), Some("OC"));
        assert!((d.total_weight_kg.unwrap() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn golden_samples_and_carry() {
        let lb = parse_logbook(&fixture()).unwrap();
        let d = &lb.dives[0];
        // fixture Divecomputer has 5 sample lines
        assert_eq!(d.samples.len(), 5);
        // sample at index 2 = "  2:30 20.0m 24.0°C ndl=55:00"
        let s2 = &d.samples[2];
        assert_eq!(s2.time_sec, 150);
        assert_eq!(s2.depth_m, 20.0);
        assert_eq!(s2.temp_c, Some(24.0));
        assert_eq!(s2.ndl_sec, Some(55 * 60));
    }

    #[test]
    fn dives_sorted_by_datetime() {
        let lb = parse_logbook(&fixture()).unwrap();
        let times: Vec<&str> = lb.dives.iter().map(|d| d.date_time.as_str()).collect();
        let mut sorted = times.clone();
        sorted.sort();
        assert_eq!(times, sorted);
    }

    #[test]
    fn empty_dir_returns_empty_logbook() {
        let tmp = std::env::temp_dir().join("ssrf_test_empty");
        std::fs::create_dir_all(&tmp).unwrap();
        let lb = parse_logbook(&tmp).unwrap();
        assert_eq!(lb.dives.len(), 0);
        assert_eq!(lb.units, "METRIC");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn trip_dives_grouped_and_standalone_separate() {
        let lb = parse_logbook(&fixture_trips()).unwrap();
        // Fixture has 2 dives in a trip + 1 standalone
        assert_eq!(lb.dives.len(), 3);
        assert_eq!(lb.trips.len(), 1);
        let t = &lb.trips[0];
        assert_eq!(t.label, "Egypt");
        assert_eq!(t.notes.as_deref(), Some("Great trip"));
        assert_eq!(t.dive_numbers.len(), 2);
        // Dive numbers in trip come before standalone
        assert!(t.dive_numbers.contains(&2));
        assert!(t.dive_numbers.contains(&3));
        // Standalone dive 1 is not in any trip
        let ungrouped: Vec<i32> = lb.dives.iter()
            .map(|d| d.number)
            .filter(|n| !t.dive_numbers.contains(n))
            .collect();
        assert_eq!(ungrouped, vec![1]);
    }

    #[test]
    fn trip_dir_label_from_dir_name_when_no_trip_file() {
        assert_eq!(dir_name_label("15-Egypt"), "Egypt");
        assert_eq!(dir_name_label("01-trip"), "trip");
        assert_eq!(dir_name_label("08-Red-Sea"), "Red Sea");
    }

    #[test]
    fn is_dive_dir_handles_extended_formats() {
        // Simple format
        assert!(is_dive_dir("15-Fri-12=28=43"));
        // MM-DD format (trip spanning month boundary)
        assert!(is_dive_dir("08-30-Thu-09=00=30"));
        assert!(is_dive_dir("09-01-Sat-09=00=32"));
        // Old colon separator
        assert!(is_dive_dir("15-Fri-12:28:43"));
        // With ~hex uniqueness suffix
        assert!(is_dive_dir("15-Fri-12=28=43~a1b2c3"));
        // Trip dirs must not match
        assert!(!is_dive_dir("30-KRK"));
        assert!(!is_dive_dir("15-Egypt"));
        assert!(!is_dive_dir("01-trip"));
    }

    #[test]
    fn is_trip_dir_distinguishes_from_dive_dir() {
        assert!(is_trip_dir("15-Egypt"));
        assert!(is_trip_dir("01-trip"));
        assert!(is_trip_dir("30-KRK"));
        assert!(!is_trip_dir("15-Fri-12=28=43"));
        assert!(!is_trip_dir("08-30-Thu-09=00=30"));
        assert!(!is_trip_dir("2024"));
        assert!(!is_trip_dir("03"));
    }
}

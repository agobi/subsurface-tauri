// AI-generated (Claude)
mod tokenize;
pub mod settings;
mod parse_site;
mod parse_dive;
mod parse_divecomputer;

use std::path::Path;
use settings::read_settings;
use parse_site::parse_site;
use parse_dive::parse_dive;
use parse_divecomputer::parse_divecomputer;
use tokenize::unquote;
use crate::types::{Dive, ParsedLogbook, Site, Trip};

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

// Each file under a dive dir's "Pictures" subdir is one photo/video attachment
// (see save-git.cpp's save_one_picture); count them without parsing contents.
fn count_media(dir: &Path) -> i32 {
    let Ok(entries) = std::fs::read_dir(dir.join("Pictures")) else { return 0; };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count() as i32
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

fn parse_dive_dir(dir: &Path, year: &str, month: &str, dir_name: &str, warnings: &mut Vec<String>) -> Option<Dive> {
    // Handles [[yyyy-]mm-]nn-ddd-hh=mm=ss[~hex] (= or : as time separator).
    // Strip optional ~hex uniqueness suffix first.
    let base = dir_name.split('~').next().unwrap_or(dir_name);
    // Need at least 9 chars: 8 for the time portion plus 1 separator before it,
    // so base.len()-9 in the date-prefix slice below does not underflow.
    if base.len() < 9 { return None; }

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
    // Validate month and year when they come from the directory name, not from the caller.
    if parts.len() >= 2 && !is_month(month_out) { return None; }
    if parts.len() == 3 && !is_year(year_out) { return None; }

    // Find "Dive-NNN" file; sort entries for a deterministic result if multiple exist.
    let entries = std::fs::read_dir(dir).ok()?;
    let mut dive_entries: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("Dive-") && !n[5..].is_empty() && n[5..].chars().all(|c| c.is_ascii_digit())
        })
        .collect();
    dive_entries.sort_by_key(|e| e.file_name());
    let dive_entry = dive_entries.into_iter().next()?;
    let dive_name = dive_entry.file_name().to_string_lossy().to_string();
    let number: i32 = dive_name[5..].parse().ok()?;
    let overview_text = match read_file(&dive_entry.path()) {
        Ok(t) => t,
        Err(e) => {
            log::warn!("skipping dive, cannot read {}: {}", dive_name, e);
            warnings.push(e);
            return None;
        }
    };
    let overview = parse_dive(&overview_text);

    let dc_path = dir.join("Divecomputer");
    let dc = if dc_path.exists() {
        match read_file(&dc_path) {
            Ok(t) => parse_divecomputer(&t),
            Err(e) => {
                log::warn!("skipping dive, cannot read Divecomputer: {}", e);
                warnings.push(e);
                return None;
            }
        }
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
        dc_device_id: dc.device_id,
        dc_dive_id: dc.dive_id,
        total_weight_kg: overview.total_weight_kg,
        media_count: count_media(dir),
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
fn parse_trip_dir(dir: &Path, year: &str, month: &str, dir_name: &str, warnings: &mut Vec<String>) -> Option<(Trip, Vec<Dive>)> {
    let trip_file = dir.join("00-Trip");
    let (label, notes) = if trip_file.exists() {
        // An unreadable 00-Trip must not discard all dives — fall back to dir-name label.
        let text = read_file(&trip_file).unwrap_or_default();
        parse_trip_file(&text, dir_name)
    } else {
        (dir_name_label(dir_name), None)
    };

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Some((Trip { label, area: None, notes, dive_numbers: vec![] }, vec![])),
    };
    let mut trip_dives: Vec<Dive> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && is_dive_dir(&e.file_name().to_string_lossy()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            parse_dive_dir(&e.path(), year, month, &name, warnings)
        })
        .collect();

    trip_dives.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    let dive_numbers: Vec<i32> = trip_dives.iter().map(|d| d.number).collect();
    Some((Trip { label, area: None, notes, dive_numbers }, trip_dives))
}

pub fn parse_logbook(root: &Path) -> Result<ParsedLogbook, String> {
    let settings = read_settings(root);

    let sites = parse_sites(root);
    let mut dives: Vec<Dive> = vec![];
    let mut trips: Vec<Trip> = vec![];
    let mut warnings: Vec<String> = vec![];

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
                    if let Some(dive) = parse_dive_dir(&path, &year, &month, &name, &mut warnings) {
                        dives.push(dive);
                    }
                } else if is_trip_dir(&name) {
                    if let Some((trip, trip_dives)) = parse_trip_dir(&path, &year, &month, &name, &mut warnings) {
                        dives.extend(trip_dives);
                        trips.push(trip);
                    }
                }
            }
        }
    }

    dives.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(ParsedLogbook { dives, trips, sites, settings, warnings })
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
        assert_eq!(lb.settings.units, "METRIC");
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
        assert_eq!(d.media_count, 2, "fixture Pictures dir has 2 files");
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
        assert_eq!(lb.settings.units, "METRIC");
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
    fn parse_dive_dir_non_numeric_month_in_filename_returns_none() {
        // Two-part date prefix "Aug-15" has a non-numeric month; must be rejected to
        // prevent a garbage date_time like "2024-Aug-15T12:28:43".
        let tmp = std::env::temp_dir().join("ssrf_test_non_numeric_month");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("Dive-1"), "").unwrap();
        let result = parse_dive_dir(&tmp, "2024", "08", "Aug-15-Fri-12=28=43", &mut Vec::new());
        assert!(result.is_none(), "non-numeric month in filename must be rejected");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn parse_dive_dir_ignores_bare_dive_dash_and_picks_numbered() {
        // A file named "Dive-" (empty suffix after dash) must be skipped; "Dive-1" must
        // be selected. Also verifies deterministic selection when multiple files exist.
        let tmp = std::env::temp_dir().join("ssrf_test_dive_dash_pick");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("Dive-"), "").unwrap();
        std::fs::write(tmp.join("Dive-1"), "").unwrap();
        let result = parse_dive_dir(&tmp, "2024", "03", "15-Fri-12=28=43", &mut Vec::new());
        assert!(result.is_some(), "Dive-1 must be selected, ignoring bare Dive-");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn media_count_zero_when_no_pictures_dir() {
        let tmp = std::env::temp_dir().join("ssrf_test_media_no_pictures");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("Dive-1"), "").unwrap();
        let dive = parse_dive_dir(&tmp, "2024", "03", "15-Fri-12=28=43").unwrap();
        assert_eq!(dive.media_count, 0);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn media_count_matches_files_in_pictures_dir() {
        let tmp = std::env::temp_dir().join("ssrf_test_media_with_pictures");
        let pics = tmp.join("Pictures");
        std::fs::create_dir_all(&pics).unwrap();
        std::fs::write(tmp.join("Dive-1"), "").unwrap();
        std::fs::write(pics.join("+00=00=05"), "filename \"a.jpg\"\n").unwrap();
        std::fs::write(pics.join("+00=01=30"), "filename \"b.jpg\"\n").unwrap();
        std::fs::write(pics.join("+00=02=00"), "filename \"c.jpg\"\n").unwrap();
        let dive = parse_dive_dir(&tmp, "2024", "03", "15-Fri-12=28=43").unwrap();
        assert_eq!(dive.media_count, 3);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn parse_trip_dir_returns_trip_when_all_dives_fail() {
        // A trip whose child dive directories all fail to parse must still appear in
        // the logbook rather than being silently discarded.
        let tmp = std::env::temp_dir().join("ssrf_test_trip_no_dives");
        let trip_dir = tmp.join("2024").join("03").join("15-Egypt");
        std::fs::create_dir_all(&trip_dir).unwrap();
        std::fs::write(trip_dir.join("00-Trip"), "location \"Egypt\"\n").unwrap();
        // Corrupt dive sub-dir: "Dive-NNN" file is missing, so parse_dive_dir returns None.
        let dive_dir = trip_dir.join("15-Fri-12=28=43");
        std::fs::create_dir_all(&dive_dir).unwrap();
        let lb = parse_logbook(&tmp).unwrap();
        assert_eq!(lb.trips.len(), 1, "trip must survive even when all dives fail to parse");
        assert_eq!(lb.trips[0].label, "Egypt");
        assert_eq!(lb.trips[0].dive_numbers.len(), 0);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    #[cfg(unix)]
    fn parse_trip_dir_read_dir_failure_returns_empty_trip_not_none() {
        // When read_dir on the trip directory itself fails (e.g. permission denied),
        // parse_trip_dir must return Some(empty trip) rather than None so the trip
        // still appears in the logbook.
        use std::os::unix::fs::PermissionsExt;
        let tmp = std::env::temp_dir().join("ssrf_test_trip_read_dir_fail");
        let trip_dir = tmp.join("2024").join("03").join("08-RedSea");
        std::fs::create_dir_all(&trip_dir).unwrap();
        std::fs::write(trip_dir.join("00-Trip"), "location \"Red Sea\"\n").unwrap();
        // Remove read+execute permission so read_dir fails.
        std::fs::set_permissions(&trip_dir, std::fs::Permissions::from_mode(0o000)).unwrap();

        let lb = parse_logbook(&tmp);
        // Restore permissions before asserting so cleanup always succeeds.
        std::fs::set_permissions(&trip_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::remove_dir_all(&tmp).ok();

        let lb = lb.unwrap();
        assert_eq!(lb.trips.len(), 1, "trip must survive read_dir failure");
        assert_eq!(lb.trips[0].dive_numbers.len(), 0);
    }

    #[test]
    #[cfg(unix)]
    fn parse_dive_dir_unreadable_dive_file_returns_none_not_panic() {
        // An unreadable Dive-N file (e.g. permission denied) must be skipped gracefully
        // rather than panicking; a warning is logged instead of failing silently, and
        // the same failure string is pushed to `warnings` so the caller can surface it.
        use std::os::unix::fs::PermissionsExt;
        let tmp = std::env::temp_dir().join("ssrf_test_unreadable_dive_file");
        std::fs::create_dir_all(&tmp).unwrap();
        let dive_file = tmp.join("Dive-1");
        std::fs::write(&dive_file, "").unwrap();
        std::fs::set_permissions(&dive_file, std::fs::Permissions::from_mode(0o000)).unwrap();

        let mut warnings = Vec::new();
        let result = parse_dive_dir(&tmp, "2024", "03", "15-Fri-12=28=43", &mut warnings);

        std::fs::set_permissions(&dive_file, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::remove_dir_all(&tmp).ok();

        assert!(result.is_none(), "unreadable Dive-N file must be skipped, not panic");
        assert_eq!(warnings.len(), 1, "unreadable Dive-N file must push exactly one warning");
        assert!(
            warnings[0].contains(&dive_file.display().to_string()),
            "warning must contain the file's path: {}",
            warnings[0]
        );
    }

    #[test]
    #[cfg(unix)]
    fn parse_dive_dir_unreadable_divecomputer_file_returns_none_not_panic() {
        // An unreadable Divecomputer file must be skipped gracefully rather than panicking;
        // a warning is logged instead of failing silently, and the same failure string is
        // pushed to `warnings` so the caller can surface it.
        use std::os::unix::fs::PermissionsExt;
        let tmp = std::env::temp_dir().join("ssrf_test_unreadable_dc_file");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("Dive-1"), "").unwrap();
        let dc_file = tmp.join("Divecomputer");
        std::fs::write(&dc_file, "").unwrap();
        std::fs::set_permissions(&dc_file, std::fs::Permissions::from_mode(0o000)).unwrap();

        let mut warnings = Vec::new();
        let result = parse_dive_dir(&tmp, "2024", "03", "15-Fri-12=28=43", &mut warnings);

        std::fs::set_permissions(&dc_file, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::remove_dir_all(&tmp).ok();

        assert!(result.is_none(), "unreadable Divecomputer file must be skipped, not panic");
        assert_eq!(warnings.len(), 1, "unreadable Divecomputer file must push exactly one warning");
        assert!(
            warnings[0].contains(&dc_file.display().to_string()),
            "warning must contain the file's path: {}",
            warnings[0]
        );
    }

    #[test]
    #[cfg(unix)]
    fn parse_logbook_reports_warning_for_unreadable_dive_keeps_readable_one() {
        // A logbook with one readable dive and one unreadable dive (permission-denied
        // Dive-N file) must surface exactly one warning on ParsedLogbook.warnings, while
        // the readable dive still appears in `dives` — a parse failure on one dive must
        // not affect the rest of the logbook.
        use std::os::unix::fs::PermissionsExt;
        let tmp = std::env::temp_dir().join("ssrf_test_logbook_warnings");
        let month_dir = tmp.join("2024").join("03");

        let good_dir = month_dir.join("15-Fri-12=28=43");
        std::fs::create_dir_all(&good_dir).unwrap();
        std::fs::write(good_dir.join("Dive-1"), "").unwrap();

        let bad_dir = month_dir.join("16-Sat-09=00=00");
        std::fs::create_dir_all(&bad_dir).unwrap();
        let bad_dive_file = bad_dir.join("Dive-2");
        std::fs::write(&bad_dive_file, "").unwrap();
        std::fs::set_permissions(&bad_dive_file, std::fs::Permissions::from_mode(0o000)).unwrap();

        let lb = parse_logbook(&tmp);

        std::fs::set_permissions(&bad_dive_file, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::remove_dir_all(&tmp).ok();

        let lb = lb.unwrap();
        assert_eq!(lb.dives.len(), 1, "readable dive must still be parsed");
        assert_eq!(lb.dives[0].number, 1);
        assert_eq!(lb.warnings.len(), 1, "exactly one warning for the unreadable dive");
        assert!(
            lb.warnings[0].contains(&bad_dive_file.display().to_string()),
            "warning must reference the broken dive's path: {}",
            lb.warnings[0]
        );
    }

    #[test]
    fn parse_dive_dir_eight_char_name_returns_none_not_panic() {
        // "01=01=01" is 8 chars and satisfies is_dive_dir (b[5]=='='), but the
        // date-prefix slice `&base[..base.len()-9]` would underflow if the guard
        // were `< 8`. With the corrected guard of `< 9` this must return None.
        let result = parse_dive_dir(std::path::Path::new("/nonexistent"), "2024", "03", "01=01=01", &mut Vec::new());
        assert!(result.is_none());
    }

    #[test]
    fn trip_file_read_error_falls_back_to_dir_name_label() {
        // parse_trip_file("", ...) simulates an unreadable 00-Trip — must still return a usable label.
        let (label, notes) = parse_trip_file("", "10-Egypt");
        assert_eq!(label, "Egypt");
        assert!(notes.is_none());
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

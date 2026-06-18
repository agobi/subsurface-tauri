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
use crate::types::{Dive, Logbook, Site};

fn is_year(s: &str) -> bool { s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) }
fn is_month(s: &str) -> bool { s.len() == 2 && s.chars().all(|c| c.is_ascii_digit()) }

fn is_dive_dir(s: &str) -> bool {
    // Matches DD-Day-HH=MM=SS (e.g. "15-Fri-12=28=43")
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() < 2 { return false; }
    if parts[0].len() != 2 || !parts[0].chars().all(|c| c.is_ascii_digit()) { return false; }
    let rest = parts[1]; // "Fri-12=28=43"
    let time_start = rest.find('-').map(|i| i + 1).unwrap_or(rest.len());
    let time_part = &rest[time_start..]; // "12=28=43"
    let tparts: Vec<&str> = time_part.split('=').collect();
    tparts.len() == 3 && tparts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_digit()))
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
    // Extract DD, HH, MM, SS from "DD-Day-HH=MM=SS"
    let dash1 = dir_name.find('-')?;
    let day = &dir_name[..dash1];
    let rest = &dir_name[dash1 + 1..];
    let dash2 = rest.find('-')?;
    let time_part = &rest[dash2 + 1..]; // "HH=MM=SS"
    let tparts: Vec<&str> = time_part.split('=').collect();
    if tparts.len() != 3 { return None; }
    let (hh, mm, ss) = (tparts[0], tparts[1], tparts[2]);

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
        date_time: format!("{year}-{month}-{day}T{hh}:{mm}:{ss}"),
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
        samples: dc.samples,
        events: dc.events,
    })
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

            let Ok(dive_entries) = std::fs::read_dir(&month_dir) else { continue; };
            for dive_entry in dive_entries.filter_map(|e| e.ok()) {
                let name = dive_entry.file_name().to_string_lossy().to_string();
                let path = dive_entry.path();
                if !path.is_dir() { continue; }
                if name.ends_with("trip") || name.starts_with('-') { continue; }
                if !is_dive_dir(&name) { continue; }
                if let Some(dive) = parse_dive_dir(&path, &year, &month, &name) {
                    dives.push(dive);
                }
            }
        }
    }

    dives.sort_by(|a, b| a.date_time.cmp(&b.date_time));

    Ok(Logbook { dives, trips: vec![], sites, units })
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
}

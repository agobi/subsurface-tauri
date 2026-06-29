// AI-generated (Claude)
use std::path::Path;
use crate::types::{DiveEvent, Sample};
use crate::ssrf_git::settings::sha1_u32;

pub struct ParsedDive {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub duration_sec: u32,
    pub max_depth_m: f64,
    pub mean_depth_m: f64,
    pub water_temp_c: Option<f64>,
    pub cylinders: Vec<ParsedCylinder>,
    pub samples: Vec<Sample>,
    pub events: Vec<DiveEvent>,
    pub dc_model: String,
    pub device_id: String,
    pub dive_id: Vec<u8>,
}

pub struct ParsedCylinder {
    pub description: Option<String>,
    pub o2_percent: Option<f64>,
    pub he_percent: Option<f64>,
    pub volume_l: Option<f64>,
    pub work_pressure_bar: Option<f64>,
    pub start_bar: Option<f64>,
    pub end_bar: Option<f64>,
}

fn weekday_abbrev(year: i32, month: u32, day: u32) -> &'static str {
    use chrono::{Datelike, NaiveDate, Weekday};
    match NaiveDate::from_ymd_opt(year, month, day)
        .map(|d| d.weekday())
        .unwrap_or(Weekday::Mon)
    {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

fn duration_mmss(sec: u32) -> String {
    format!("{:02}:{:02}", sec / 60, sec % 60)
}

fn sample_line(s: &Sample) -> String {
    let mut line = format!("  {} {:.1}m", duration_mmss(s.time_sec as u32), s.depth_m);
    if let Some(t) = s.temp_c {
        line.push_str(&format!(" {t:.1}\u{00b0}C"));
    }
    if let Some(ndl) = s.ndl_sec {
        line.push_str(&format!(" ndl={}", duration_mmss(ndl as u32)));
    }
    if let Some(tts) = s.tts_sec {
        line.push_str(&format!(" tts={}", duration_mmss(tts as u32)));
    }
    if let Some(cns) = s.cns {
        line.push_str(&format!(" cns={:.0}%", cns));
    }
    if let Some(p) = s.pressure_bar {
        line.push_str(&format!(" {p:.1}bar"));
    }
    line
}

fn event_line(e: &DiveEvent) -> String {
    let mut line = format!("event {}", duration_mmss(e.time_sec as u32));
    if let Some(t) = e.event_type { line.push_str(&format!(" type={t}")); }
    if let Some(f) = e.flags { line.push_str(&format!(" flags={f}")); }
    if let Some(v) = e.value { line.push_str(&format!(" value={v}")); }
    if !e.name.is_empty() { line.push_str(&format!(" name=\"{}\"", e.name)); }
    if let Some(c) = e.cylinder { line.push_str(&format!(" cylinder={c}")); }
    if let Some(o2) = e.o2_percent { line.push_str(&format!(" o2={o2:.1}%")); }
    line
}

/// Scans the logbook tree recursively for all Dive-NNN files and returns max(N) + 1,
/// or 1 if none exist yet.
pub fn next_dive_number(root: &Path) -> i32 {
    let mut max = 0i32;
    fn scan(dir: &Path, max: &mut i32) {
        let Ok(entries) = std::fs::read_dir(dir) else { return; };
        for e in entries.filter_map(|e| e.ok()) {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(suffix) = name.strip_prefix("Dive-") {
                let n: i32 = suffix.parse().unwrap_or(0);
                if n > *max { *max = n; }
            } else if e.path().is_dir() {
                scan(&e.path(), max);
            }
        }
    }
    scan(root, &mut max);
    max + 1
}

/// Writes a dive into the logbook at `logbook_root` using the ssrf_git on-disk format.
///
/// Creates `{YYYY}/{MM}/{DD}-{DayAbbrev}-{HH}={MM}={SS}/Dive-NNN` and
/// `{YYYY}/{MM}/{DD}-{DayAbbrev}-{HH}={MM}={SS}/Divecomputer`.
/// The dive number is derived by scanning for the current maximum and incrementing.
pub fn write_dive(logbook_root: &Path, dive: ParsedDive) -> Result<(), String> {
    let number = next_dive_number(logbook_root);
    let day_abbrev = weekday_abbrev(dive.year, dive.month, dive.day);
    let dir_rel = format!(
        "{:04}/{:02}/{:02}-{}-{:02}={:02}={:02}",
        dive.year, dive.month, dive.day, day_abbrev,
        dive.hour, dive.minute, dive.second
    );
    let dir = logbook_root.join(&dir_rel);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // ── Dive-NNN ────────────────────────────────────────────────────────────
    let mut dive_content = String::new();
    dive_content.push_str(&format!("duration {} min\n", duration_mmss(dive.duration_sec)));
    for cyl in &dive.cylinders {
        let desc = cyl.description.as_deref().unwrap_or("unknown");
        let mut cyl_line = format!("cylinder description=\"{desc}\"");
        if let Some(v) = cyl.volume_l { cyl_line.push_str(&format!(" vol={v:.1}l")); }
        if let Some(wp) = cyl.work_pressure_bar { cyl_line.push_str(&format!(" workpressure={wp:.1}bar")); }
        if let Some(o2) = cyl.o2_percent { cyl_line.push_str(&format!(" o2={o2:.1}%")); }
        if let Some(he) = cyl.he_percent { cyl_line.push_str(&format!(" he={he:.1}%")); }
        if let Some(s) = cyl.start_bar { cyl_line.push_str(&format!(" start={s:.1}bar")); }
        if let Some(e) = cyl.end_bar { cyl_line.push_str(&format!(" end={e:.1}bar")); }
        dive_content.push_str(&cyl_line);
        dive_content.push('\n');
    }
    std::fs::write(dir.join(format!("Dive-{number:03}")), &dive_content)
        .map_err(|e| e.to_string())?;

    // ── Divecomputer ────────────────────────────────────────────────────────
    let mut dc_content = String::new();
    dc_content.push_str(&format!("model \"{}\"\n", dive.dc_model));
    dc_content.push_str(&format!("deviceid {}\n", dive.device_id));
    // diveid = SHA1_uint32(fingerprint_bytes), matching original Subsurface's calculate_diveid().
    dc_content.push_str(&format!("diveid {:08x}\n", sha1_u32(&dive.dive_id)));
    dc_content.push_str(&format!("maxdepth {:.1}m\n", dive.max_depth_m));
    dc_content.push_str(&format!("meandepth {:.3}m\n", dive.mean_depth_m));
    if let Some(t) = dive.water_temp_c {
        dc_content.push_str(&format!("watertemp {t:.1}\u{00b0}C\n"));
    }
    for e in &dive.events {
        dc_content.push_str(&event_line(e));
        dc_content.push('\n');
    }
    for s in &dive.samples {
        dc_content.push_str(&sample_line(s));
        dc_content.push('\n');
    }
    std::fs::write(dir.join("Divecomputer"), &dc_content)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Sample;

    fn make_parsed_dive() -> ParsedDive {
        ParsedDive {
            year: 2024, month: 6, day: 15,
            hour: 12, minute: 30, second: 0,
            duration_sec: 2700,
            max_depth_m: 30.5,
            mean_depth_m: 15.2,
            water_temp_c: Some(12.0),
            cylinders: vec![ParsedCylinder {
                description: Some("D12 232bar".to_string()),
                o2_percent: Some(32.0),
                volume_l: Some(12.0),
                work_pressure_bar: Some(232.0),
                he_percent: None,
                start_bar: Some(200.0),
                end_bar: Some(50.0),
            }],
            samples: vec![
                Sample { time_sec: 0, depth_m: 0.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None },
                Sample { time_sec: 60, depth_m: 10.0, temp_c: Some(12.0), ndl_sec: Some(99*60), tts_sec: None, cns: None, pressure_bar: Some(180.0) },
            ],
            events: vec![],
            dc_model: "Shearwater Perdix".to_string(),
            device_id: "a790cf6c".to_string(),
            dive_id: vec![0x76, 0xb9, 0xbc, 0x25],
        }
    }

    #[test]
    fn write_and_read_back_via_parser() {
        let tmp = std::env::temp_dir().join("dc_writer_test");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();

        let dive = make_parsed_dive();
        write_dive(&tmp, dive).unwrap();

        // Read back with existing parser
        let lb = crate::ssrf_git::parse_logbook(&tmp).unwrap();
        assert_eq!(lb.dives.len(), 1, "exactly one dive written");
        let d = &lb.dives[0];
        assert_eq!(d.number, 1);
        assert_eq!(d.date_time, "2024-06-15T12:30:00");
        assert_eq!(d.duration_sec, 2700);
        assert!((d.max_depth_m.unwrap() - 30.5).abs() < 0.01);
        assert!((d.water_temp_c.unwrap() - 12.0).abs() < 0.01);
        assert_eq!(d.dc_model.as_deref(), Some("Shearwater Perdix"));
        assert_eq!(d.dc_device_id.as_deref(), Some("a790cf6c"));
        // diveid is SHA1_uint32(fingerprint_bytes), not raw bytes
        let expected_diveid = format!("{:08x}", sha1_u32(&[0x76u8, 0xb9, 0xbc, 0x25]));
        assert_eq!(d.dc_dive_id.as_deref(), Some(expected_diveid.as_str()));
        assert_eq!(d.cylinders.len(), 1);
        assert_eq!(d.samples.len(), 2);

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn next_dive_number_on_empty_logbook_is_1() {
        let tmp = std::env::temp_dir().join("dc_writer_number_test");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();
        assert_eq!(next_dive_number(&tmp), 1);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn next_dive_number_increments_past_existing() {
        let tmp = std::env::temp_dir().join("dc_writer_number_incr");
        std::fs::remove_dir_all(&tmp).ok();
        std::fs::create_dir_all(&tmp).unwrap();
        let dive = make_parsed_dive();
        write_dive(&tmp, dive).unwrap();
        assert_eq!(next_dive_number(&tmp), 2);
        std::fs::remove_dir_all(&tmp).ok();
    }
}

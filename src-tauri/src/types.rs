// AI-generated (Claude)
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub time_sec: i64,
    pub depth_m: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndl_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tts_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cns: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure_bar: Option<f64>,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cylinder {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_l: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_pressure_bar: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub o2_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub he_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_bar: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_bar: Option<f64>,
    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub cylinder_use: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth_m: Option<f64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiveEvent {
    pub time_sec: i64,
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub event_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cylinder: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub o2_percent: Option<f64>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dive {
    pub number: i32,
    pub date_time: String,
    pub duration_sec: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dive_guide: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buddy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub cylinders: Vec<Cylinder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_depth_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_temp_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deco_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divemode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_dive_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_weight_kg: Option<f64>,
    pub samples: Vec<Sample>,
    pub events: Vec<DiveEvent>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trip {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub dive_numbers: Vec<i32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GpsCoord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps: Option<GpsCoord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

/// All scalar fields from `Dive`; no samples or events. Serialized in `Logbook` and
/// returned by every open command so the list view never crosses the IPC boundary with
/// bulk sample arrays.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiveSummary {
    pub number: i32,
    pub date_time: String,
    pub duration_sec: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_id: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dive_guide: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buddy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub cylinders: Vec<Cylinder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dc_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_depth_m: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_temp_c: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deco_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divemode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_weight_kg: Option<f64>,
}

impl From<&Dive> for DiveSummary {
    fn from(d: &Dive) -> Self {
        DiveSummary {
            number: d.number,
            date_time: d.date_time.clone(),
            duration_sec: d.duration_sec,
            site_id: d.site_id.clone(),
            tags: d.tags.clone(),
            rating: d.rating,
            dive_guide: d.dive_guide.clone(),
            buddy: d.buddy.clone(),
            suit: d.suit.clone(),
            notes: d.notes.clone(),
            cylinders: d.cylinders.clone(),
            dc_model: d.dc_model.clone(),
            max_depth_m: d.max_depth_m,
            mean_depth_m: d.mean_depth_m,
            water_temp_c: d.water_temp_c,
            deco_model: d.deco_model.clone(),
            divemode: d.divemode.clone(),
            total_weight_kg: d.total_weight_kg,
        }
    }
}

/// Internal parse result — holds full `Dive` objects with samples and events.
/// Not serialized to IPC; converted to `Logbook` (with `DiveSummary`) before returning.
#[derive(Clone, Debug)]
pub struct ParsedLogbook {
    pub dives: Vec<Dive>,
    pub trips: Vec<Trip>,
    pub sites: Vec<Site>,
    pub units: String,
}

impl ParsedLogbook {
    /// Splits into the full dive list (stored in managed state) and a summary-only `Logbook`
    /// suitable for the IPC payload.
    pub fn into_summary(self) -> (Vec<Dive>, Logbook) {
        let summaries: Vec<DiveSummary> = self.dives.iter().map(DiveSummary::from).collect();
        let logbook = Logbook { dives: summaries, trips: self.trips, sites: self.sites, units: self.units };
        (self.dives, logbook)
    }
}

/// Logbook as returned over IPC: dives are summaries only; samples/events are fetched on demand.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Logbook {
    pub dives: Vec<DiveSummary>,
    pub trips: Vec<Trip>,
    pub sites: Vec<Site>,
    pub units: String,
}

/// A logbook entry in the recents list. Path variants use index-based menu IDs
/// so paths never appear in menu item identifiers.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum RecentEntry {
    Local { path: String },
    Cloud { email: String, url: String },
}

/// Returned by every open command so the frontend can update the window title
/// and recents list without a second IPC round-trip.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResult {
    pub logbook: Logbook,
    pub display_name: String,
    pub recents: Vec<RecentEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_serializes_camel_case() {
        let s = Sample {
            time_sec: 30,
            depth_m: 5.0,
            temp_c: Some(24.0),
            ndl_sec: Some(3600),
            tts_sec: None,
            cns: None,
            pressure_bar: Some(200.0),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"timeSec\":30"), "timeSec missing: {json}");
        assert!(json.contains("\"depthM\":5.0"), "depthM missing: {json}");
        assert!(json.contains("\"tempC\":24.0"), "tempC missing: {json}");
        assert!(json.contains("\"ndlSec\":3600"), "ndlSec missing: {json}");
        assert!(!json.contains("\"ttsSec\""), "ttsSec should be absent: {json}");
        assert!(json.contains("\"pressureBar\":200.0"), "pressureBar missing: {json}");
    }

    #[test]
    fn cylinder_reserved_field_serializes_as_use() {
        let c = Cylinder {
            description: "D12".to_owned(),
            cylinder_use: Some("primary".to_owned()),
            ..Default::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"use\":\"primary\""), "use field missing: {json}");
        assert!(!json.contains("cylinderUse"), "cylinderUse must not appear: {json}");
    }

    #[test]
    fn event_type_serializes_as_type() {
        let e = DiveEvent {
            time_sec: 5,
            name: "gaschange".to_owned(),
            event_type: Some(25),
            flags: Some(1),
            value: None,
            cylinder: Some(0),
            o2_percent: Some(32.0),
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":25"), "type field missing: {json}");
        assert!(!json.contains("eventType"), "eventType must not appear: {json}");
    }

    #[test]
    fn logbook_empty_serializes() {
        let lb = Logbook {
            dives: vec![],
            trips: vec![],
            sites: vec![],
            units: "METRIC".to_owned(),
        };
        let json = serde_json::to_string(&lb).unwrap();
        assert!(json.contains("\"units\":\"METRIC\""));
    }

    fn make_dive(number: i32) -> Dive {
        Dive {
            number,
            date_time: "2024-06-15T09:00:00".to_owned(),
            duration_sec: 2700,
            site_id: Some("site-1".to_owned()),
            tags: vec!["cold".to_owned()],
            rating: Some(4),
            dive_guide: None,
            buddy: Some("Alice".to_owned()),
            suit: None,
            notes: Some("Great dive".to_owned()),
            cylinders: vec![],
            dc_model: Some("Shearwater Perdix".to_owned()),
            max_depth_m: Some(30.5),
            mean_depth_m: Some(15.0),
            water_temp_c: Some(12.0),
            deco_model: None,
            divemode: None,
            dc_device_id: None,
            dc_dive_id: None,
            total_weight_kg: Some(8.0),
            samples: vec![Sample { time_sec: 0, depth_m: 0.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None }],
            events: vec![],
        }
    }

    #[test]
    fn dive_summary_has_no_samples_or_events() {
        let dive = make_dive(42);
        let summary = DiveSummary::from(&dive);
        let json = serde_json::to_string(&summary).unwrap();
        assert!(!json.contains("samples"), "DiveSummary must not contain samples: {json}");
        assert!(!json.contains("events"), "DiveSummary must not contain events: {json}");
        assert!(json.contains("\"number\":42"), "number missing: {json}");
        assert!(json.contains("\"dateTime\":"), "dateTime missing: {json}");
        assert!(json.contains("\"maxDepthM\":30.5"), "maxDepthM missing: {json}");
        assert!(json.contains("\"rating\":4"), "rating missing: {json}");
    }

    #[test]
    fn dive_summary_from_copies_all_scalar_fields() {
        let dive = make_dive(7);
        let summary = DiveSummary::from(&dive);
        assert_eq!(summary.number, dive.number);
        assert_eq!(summary.date_time, dive.date_time);
        assert_eq!(summary.duration_sec, dive.duration_sec);
        assert_eq!(summary.site_id, dive.site_id);
        assert_eq!(summary.tags, dive.tags);
        assert_eq!(summary.rating, dive.rating);
        assert_eq!(summary.buddy, dive.buddy);
        assert_eq!(summary.notes, dive.notes);
        assert_eq!(summary.dc_model, dive.dc_model);
        assert_eq!(summary.max_depth_m, dive.max_depth_m);
        assert_eq!(summary.water_temp_c, dive.water_temp_c);
        assert_eq!(summary.total_weight_kg, dive.total_weight_kg);
    }

    #[test]
    fn parsed_logbook_into_summary_splits_correctly() {
        let d1 = make_dive(1);
        let d2 = make_dive(2);
        let parsed = ParsedLogbook {
            dives: vec![d1.clone(), d2.clone()],
            trips: vec![],
            sites: vec![],
            units: "METRIC".to_owned(),
        };
        let (full_dives, logbook) = parsed.into_summary();
        assert_eq!(full_dives.len(), 2, "full dives count");
        assert_eq!(logbook.dives.len(), 2, "summary dives count");
        assert_eq!(full_dives[0].samples.len(), 1, "full dive retains samples");
        assert_eq!(logbook.dives[0].number, 1);
        assert_eq!(logbook.dives[1].number, 2);
    }
}

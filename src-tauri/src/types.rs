// AI-generated (Claude)
use serde::Serialize;

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

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Logbook {
    pub dives: Vec<Dive>,
    pub trips: Vec<Trip>,
    pub sites: Vec<Site>,
    pub units: String,
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
}

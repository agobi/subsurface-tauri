// AI-generated (Claude)
use std::path::PathBuf;
use crate::ssrf_git::settings::Settings;
use crate::types::{Dive, DiveSummary, Logbook, Site, Trip};

pub struct LogbookState {
    pub root: PathBuf,
    pub dives: Vec<Dive>,
    pub trips: Vec<Trip>,
    pub sites: Vec<Site>,
    pub settings: Settings,
}

impl LogbookState {
    pub fn to_logbook(&self) -> Logbook {
        Logbook {
            dives: self.dives.iter().map(DiveSummary::from).collect(),
            trips: self.trips.clone(),
            sites: self.sites.clone(),
            units: self.settings.units.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(units: &str) -> LogbookState {
        let mut settings = Settings::default();
        settings.units = units.to_string();
        LogbookState {
            root: PathBuf::from("/tmp/test-logbook"),
            dives: vec![],
            trips: vec![],
            sites: vec![],
            settings,
        }
    }

    #[test]
    fn to_logbook_derives_units_from_settings() {
        let state = make_state("IMPERIAL");
        let logbook = state.to_logbook();
        assert_eq!(logbook.units, "IMPERIAL");
    }

    #[test]
    fn to_logbook_metric_default() {
        let state = make_state("METRIC");
        let logbook = state.to_logbook();
        assert_eq!(logbook.units, "METRIC");
    }

    #[test]
    fn to_logbook_dives_become_summaries_without_samples() {
        use crate::types::Sample;
        let settings = Settings::default();
        let state = LogbookState {
            root: PathBuf::from("/tmp/test"),
            dives: vec![Dive {
                number: 1,
                date_time: "2024-06-01T09:00:00".to_string(),
                duration_sec: 3600,
                site_id: None,
                tags: vec![],
                rating: Some(5),
                dive_guide: None,
                buddy: None,
                suit: None,
                notes: None,
                cylinders: vec![],
                dc_model: Some("Perdix".to_string()),
                max_depth_m: Some(30.0),
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
                samples: vec![
                    Sample { time_sec: 0, depth_m: 0.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None },
                    Sample { time_sec: 60, depth_m: 10.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None },
                ],
                events: vec![],
            }],
            trips: vec![],
            sites: vec![],
            settings,
        };
        let logbook = state.to_logbook();
        assert_eq!(logbook.dives.len(), 1);
        assert_eq!(logbook.dives[0].number, 1);
        assert_eq!(logbook.dives[0].rating, Some(5));
        // DiveSummary has no samples or events fields — verify via JSON
        let json = serde_json::to_string(&logbook.dives[0]).unwrap();
        assert!(!json.contains("samples"), "summary must not contain samples");
        assert!(!json.contains("events"), "summary must not contain events");
    }

    #[test]
    fn to_logbook_preserves_trips_and_sites() {
        use crate::types::Trip;
        let state = LogbookState {
            root: PathBuf::from("/tmp/test"),
            dives: vec![],
            trips: vec![Trip { label: "Egypt".to_string(), area: None, notes: None, dive_numbers: vec![1, 2] }],
            sites: vec![crate::types::Site { id: "abc".to_string(), name: "Blue Hole".to_string(), description: None, notes: None, gps: None, country: Some("Egypt".to_string()) }],
            settings: Settings::default(),
        };
        let logbook = state.to_logbook();
        assert_eq!(logbook.trips.len(), 1);
        assert_eq!(logbook.trips[0].label, "Egypt");
        assert_eq!(logbook.sites.len(), 1);
        assert_eq!(logbook.sites[0].name, "Blue Hole");
    }
}

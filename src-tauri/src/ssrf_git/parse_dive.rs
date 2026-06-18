// AI-generated (Claude)
use super::tokenize::{mmss, parse_attrs, split_keyword, strip_unit, unquote};
use crate::types::Cylinder;

pub struct DiveOverview {
    pub duration_sec: i64,
    pub rating: Option<i32>,
    pub tags: Vec<String>,
    pub site_id: Option<String>,
    pub dive_guide: Option<String>,
    pub buddy: Option<String>,
    pub suit: Option<String>,
    pub notes: Option<String>,
    pub cylinders: Vec<Cylinder>,
}

fn parse_cylinder(rest: &str) -> Cylinder {
    let a = parse_attrs(rest);
    Cylinder {
        description: a.get("description").cloned().unwrap_or_default(),
        volume_l: a.get("vol").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
        work_pressure_bar: a.get("workpressure").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
        o2_percent: a.get("o2").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
        he_percent: a.get("he").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
        start_bar: a.get("start").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
        end_bar: a.get("end").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
        cylinder_use: a.get("use").cloned(),
        depth_m: a.get("depth").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
    }
}

pub fn parse_dive(text: &str) -> DiveOverview {
    let mut d = DiveOverview {
        duration_sec: 0,
        rating: None,
        tags: vec![],
        site_id: None,
        dive_guide: None,
        buddy: None,
        suit: None,
        notes: None,
        cylinders: vec![],
    };
    for line in text.lines() {
        let (key, rest) = split_keyword(line);
        match key {
            "duration" => d.duration_sec = mmss(rest),
            "rating" => d.rating = rest.trim().parse().ok(),
            "tags" => {
                d.tags = rest
                    .split(',')
                    .map(|t| unquote(t.trim()))
                    .filter(|t| !t.is_empty())
                    .collect();
            }
            "divesiteid" => d.site_id = Some(rest.trim().to_owned()),
            "divemaster" => d.dive_guide = Some(unquote(rest)),
            "buddy" => d.buddy = Some(unquote(rest)),
            "suit" => d.suit = Some(unquote(rest)),
            "notes" => d.notes = Some(unquote(rest)),
            "cylinder" => d.cylinders.push(parse_cylinder(rest)),
            _ => {}
        }
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIVE_TEXT: &str = r#"duration 05:00 min
rating 4
notrip
tags "cave"
divesiteid 04782ed8
divemaster "Test Diver"
cylinder vol=24.0l workpressure=232.0bar description="D12 232 bar" o2=32.0% depth=39.845m
"#;

    #[test]
    fn parses_dive_overview() {
        let d = parse_dive(DIVE_TEXT);
        assert_eq!(d.duration_sec, 300);
        assert_eq!(d.rating, Some(4));
        assert_eq!(d.tags, vec!["cave"]);
        assert_eq!(d.site_id.as_deref(), Some("04782ed8"));
        assert_eq!(d.dive_guide.as_deref(), Some("Test Diver"));
    }

    #[test]
    fn parses_cylinder() {
        let d = parse_dive(DIVE_TEXT);
        assert_eq!(d.cylinders.len(), 1);
        let c = &d.cylinders[0];
        assert_eq!(c.description, "D12 232 bar");
        assert_eq!(c.volume_l, Some(24.0));
        assert_eq!(c.work_pressure_bar, Some(232.0));
        assert_eq!(c.o2_percent, Some(32.0));
        assert!((c.depth_m.unwrap() - 39.845).abs() < 1e-3);
    }

    #[test]
    fn multi_tags() {
        let d = parse_dive("tags \"cave\",\"night\",\"deep\"\n");
        assert_eq!(d.tags, vec!["cave", "night", "deep"]);
    }
}

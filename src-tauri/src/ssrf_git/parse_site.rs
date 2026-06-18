// AI-generated (Claude)
use super::tokenize::{split_keyword, unquote};
use crate::types::{GpsCoord, Site};

pub fn parse_site(file_name: &str, text: &str) -> Site {
    let id = file_name.trim_start_matches("Site-").to_owned();
    let mut site = Site { id, name: String::new(), description: None, notes: None, gps: None };
    for line in text.lines() {
        let (key, rest) = split_keyword(line);
        match key {
            "name" => site.name = unquote(rest),
            "description" => site.description = Some(unquote(rest)),
            "notes" => site.notes = Some(unquote(rest)),
            "gps" => {
                let mut parts = rest.trim().split_whitespace();
                if let (Some(lat_s), Some(lon_s)) = (parts.next(), parts.next()) {
                    if let (Ok(lat), Ok(lon)) = (lat_s.parse::<f64>(), lon_s.parse::<f64>()) {
                        site.gps = Some(GpsCoord { lat, lon });
                    }
                }
            }
            _ => {}
        }
    }
    site
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_site_file() {
        let text = "name \"Test Spring\"\ndescription \"\"\nnotes \"\"\ngps 47.668408 18.307076\ngeo cat 2 origin 2 \"Testland\"\n";
        let site = parse_site("Site-04782ed8", text);
        assert_eq!(site.id, "04782ed8");
        assert_eq!(site.name, "Test Spring");
        assert_eq!(site.description, Some(String::new()));
        assert_eq!(site.notes, Some(String::new()));
        let gps = site.gps.unwrap();
        assert!((gps.lat - 47.668408).abs() < 1e-6);
        assert!((gps.lon - 18.307076).abs() < 1e-6);
    }

    #[test]
    fn site_without_gps() {
        let site = parse_site("Site-aabbccdd", "name \"Deep Blue\"\n");
        assert_eq!(site.id, "aabbccdd");
        assert_eq!(site.name, "Deep Blue");
        assert!(site.gps.is_none());
    }
}

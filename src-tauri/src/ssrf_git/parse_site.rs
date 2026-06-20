// AI-generated (Claude)
use super::tokenize::{split_keyword, unquote};
use crate::types::{GpsCoord, Site};

pub fn parse_site(file_name: &str, text: &str) -> Site {
    let id = file_name.trim_start_matches("Site-").to_owned();
    let mut site = Site { id, name: String::new(), description: None, notes: None, gps: None, country: None };
    for line in text.lines() {
        let (key, rest) = split_keyword(line);
        match key {
            "name" => site.name = unquote(rest),
            "description" => site.description = Some(unquote(rest)),
            "notes" => site.notes = Some(unquote(rest)),
            "gps" => {
                let mut parts = rest.split_whitespace();
                if let (Some(lat_s), Some(lon_s)) = (parts.next(), parts.next()) {
                    if let (Ok(lat), Ok(lon)) = (lat_s.parse::<f64>(), lon_s.parse::<f64>()) {
                        site.gps = Some(GpsCoord { lat, lon });
                    }
                }
            }
            "geo" => {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&s| s == "cat") {
                    if parts.get(pos + 1).map_or(false, |&v| v == "2") {
                        if let Some(last) = parts.last() {
                            let c = unquote(last);
                            if !c.is_empty() {
                                site.country = Some(c);
                            }
                        }
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

    #[test]
    fn parses_country_from_geo_line() {
        let text = "name \"Test Spring\"\ngps 47.668408 18.307076\ngeo cat 2 origin 2 \"Hungary\"\n";
        let site = parse_site("Site-04782ed8", text);
        assert_eq!(site.country.as_deref(), Some("Hungary"));
    }

    #[test]
    fn no_geo_line_yields_no_country() {
        let site = parse_site("Site-aabbccdd", "name \"Deep Blue\"\n");
        assert!(site.country.is_none());
    }

    #[test]
    fn cat2_wins_when_multiple_geo_lines() {
        let text = "name \"Test Spring\"\ngps 47.668408 18.307076\ngeo cat 1 origin 1 \"Europe\"\ngeo cat 2 origin 2 \"Hungary\"\ngeo cat 3 origin 3 \"Budapest\"\n";
        let site = parse_site("Site-04782ed8", text);
        assert_eq!(site.country.as_deref(), Some("Hungary"));
    }

    #[test]
    fn non_cat2_geo_line_yields_no_country() {
        let text = "name \"Deep Blue\"\ngeo cat 3 origin 3 \"Budapest\"\n";
        let site = parse_site("Site-aabbccdd", text);
        assert!(site.country.is_none());
    }
}

// AI-generated (Claude)
use super::tokenize::split_keyword;

pub fn parse_header(text: &str) -> String {
    for line in text.lines() {
        let (key, rest) = split_keyword(line);
        if key == "units" && rest.trim().starts_with("IMPERIAL") {
            return "IMPERIAL".to_owned();
        }
    }
    "METRIC".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_metric() {
        let text = "version 3\nautogroup\nunits METRIC\nprefs DCCEILING\n";
        assert_eq!(parse_header(text), "METRIC");
    }

    #[test]
    fn detects_imperial() {
        assert_eq!(parse_header("units IMPERIAL\n"), "IMPERIAL");
    }

    #[test]
    fn defaults_to_metric_when_absent() {
        assert_eq!(parse_header("version 3\n"), "METRIC");
        assert_eq!(parse_header(""), "METRIC");
    }
}

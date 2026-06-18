// AI-generated (Claude)
use std::collections::HashMap;

pub fn split_keyword(line: &str) -> (&str, &str) {
    let trimmed = line.trim_start();
    match trimmed.find(' ') {
        None => (trimmed, ""),
        Some(i) => (&trimmed[..i], &trimmed[i + 1..]),
    }
}

pub fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        let inner = &t[1..t.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some(other) => { out.push('\\'); out.push(other); }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        out
    } else {
        t.to_owned()
    }
}

// "34.7m" -> 34.7, "232.0bar" -> 232.0, "32.0%" -> 32.0. Returns NAN if no number.
pub fn strip_unit(s: &str) -> f64 {
    let t = s.trim();
    let end = t
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || (i == 0 && c == '-')))
        .map(|(i, _)| i)
        .unwrap_or(t.len());
    t[..end].parse().unwrap_or(f64::NAN)
}

// Parse 'a=1 b="x y" c=2.0unit' into { "a" => "1", "b" => "x y", "c" => "2.0unit" }.
pub fn parse_attrs(rest: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut s = rest;
    while !s.is_empty() {
        s = s.trim_start();
        let eq = match s.find('=') {
            Some(i) => i,
            None => break,
        };
        let key = s[..eq].trim().to_owned();
        s = &s[eq + 1..];
        let (val_raw, remaining) = if s.starts_with('"') {
            // scan to closing unescaped quote
            let mut i = 1;
            while i < s.len() {
                if s.as_bytes()[i] == b'\\' { i += 2; }
                else if s.as_bytes()[i] == b'"' { i += 1; break; }
                else { i += 1; }
            }
            (&s[..i], &s[i..])
        } else {
            let end = s.find(' ').unwrap_or(s.len());
            (&s[..end], &s[end..])
        };
        out.insert(key, unquote(val_raw));
        s = remaining;
    }
    out
}

// "0:15" -> 15, "12:28" -> 748, "99:00" -> 5940
pub fn mmss(s: &str) -> i64 {
    let colon = match s.find(':') {
        Some(i) => i,
        None => return 0,
    };
    let mins: i64 = s[..colon].trim().parse().unwrap_or(0);
    let secs: i64 = s[colon + 1..].trim_end_matches(|c: char| !c.is_ascii_digit())
        .parse().unwrap_or(0);
    mins * 60 + secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_keyword_basic() {
        assert_eq!(split_keyword("duration 05:00 min"), ("duration", "05:00 min"));
        assert_eq!(split_keyword("  maxdepth 34.7m"), ("maxdepth", "34.7m"));
        assert_eq!(split_keyword("notrip"), ("notrip", ""));
    }

    #[test]
    fn unquote_strips_quotes() {
        assert_eq!(unquote(r#""Shearwater Perdix AI""#), "Shearwater Perdix AI");
        assert_eq!(unquote(r#""escaped\"quote""#), r#"escaped"quote"#);
        assert_eq!(unquote("bare"), "bare");
        assert_eq!(unquote(r#""""#), "");
    }

    #[test]
    fn strip_unit_values() {
        assert_eq!(strip_unit("34.7m"), 34.7);
        assert_eq!(strip_unit("232.0bar"), 232.0);
        assert_eq!(strip_unit("32.0%"), 32.0);
        assert_eq!(strip_unit("-5.0m"), -5.0);
        assert!(strip_unit("abc").is_nan());
    }

    #[test]
    fn parse_attrs_basic() {
        let a = parse_attrs(r#"vol=24.0l workpressure=232.0bar description="D12 232 bar" o2=32.0%"#);
        assert_eq!(a["vol"], "24.0l");
        assert_eq!(a["workpressure"], "232.0bar");
        assert_eq!(a["description"], "D12 232 bar");
        assert_eq!(a["o2"], "32.0%");
    }

    #[test]
    fn mmss_converts() {
        assert_eq!(mmss("0:05"), 5);
        assert_eq!(mmss("12:28"), 748);
        assert_eq!(mmss("99:00"), 5940);
        assert_eq!(mmss("5:00"), 300);
    }
}

// AI-generated (Claude)
use super::tokenize::{mmss, parse_attrs, split_keyword, strip_unit, unquote};
use crate::types::{DiveEvent, Sample};

pub struct DivecomputerData {
    pub model: Option<String>,
    pub max_depth_m: Option<f64>,
    pub mean_depth_m: Option<f64>,
    pub water_temp_c: Option<f64>,
    pub deco_model: Option<String>,
    pub events: Vec<DiveEvent>,
    pub samples: Vec<Sample>,
}

// A sample line starts with whitespace then "M:SS". Example:
//   "  0:15 2.5m 24.0°C ndl=99:00 cns=5%"
// Fields absent on a line carry forward from the previous sample.
fn parse_sample_line(line: &str, carry: &mut Sample) -> Sample {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return carry.clone();
    }
    carry.time_sec = mmss(tokens[0]);
    carry.depth_m = 0.0;
    for &tok in tokens.iter().skip(1) {
        if tok.ends_with('m') && !tok.contains('=') {
            carry.depth_m = strip_unit(tok);
        } else if tok.ends_with("°C") || tok.ends_with("\u{00b0}C") {
            carry.temp_c = Some(strip_unit(tok));
        } else if let Some(v) = tok.strip_prefix("ndl=") {
            carry.ndl_sec = Some(mmss(v));
        } else if let Some(v) = tok.strip_prefix("tts=") {
            carry.tts_sec = Some(mmss(v));
        } else if let Some(v) = tok.strip_prefix("cns=") {
            carry.cns = Some(strip_unit(v));
        } else if tok.ends_with("bar") && !tok.contains('=') {
            carry.pressure_bar = Some(strip_unit(tok.split(':').next().unwrap_or(tok)));
        }
    }
    carry.clone()
}

fn parse_event(rest: &str) -> DiveEvent {
    let time_tok = rest.split_whitespace().next().unwrap_or("0:00");
    // Skip the time token before parsing attributes
    let attrs_str = rest
        .split_whitespace()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ");
    let a = parse_attrs(&attrs_str);
    DiveEvent {
        time_sec: mmss(time_tok),
        name: a.get("name").cloned().unwrap_or_default(),
        event_type: a.get("type").and_then(|s| s.parse().ok()),
        flags: a.get("flags").and_then(|s| s.parse().ok()),
        value: a.get("value").and_then(|s| s.parse().ok()),
        cylinder: a.get("cylinder").and_then(|s| s.parse().ok()),
        o2_percent: a.get("o2").map(|s| strip_unit(s)).filter(|v| !v.is_nan()),
    }
}

pub fn parse_divecomputer(text: &str) -> DivecomputerData {
    let mut dc = DivecomputerData {
        model: None,
        max_depth_m: None,
        mean_depth_m: None,
        water_temp_c: None,
        deco_model: None,
        events: vec![],
        samples: vec![],
    };
    // carry holds the accumulated optional fields across sample lines
    let mut carry = Sample { time_sec: 0, depth_m: 0.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None };

    for line in text.lines() {
        // Sample lines start with whitespace + M:SS digit pattern
        if line.starts_with(' ') || line.starts_with('\t') {
            let trimmed = line.trim_start();
            if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                dc.samples.push(parse_sample_line(line, &mut carry));
                continue;
            }
        }
        let (key, rest) = split_keyword(line);
        match key {
            "model" => dc.model = Some(unquote(rest)),
            "maxdepth" => dc.max_depth_m = Some(strip_unit(rest)).filter(|v| !v.is_nan()),
            "meandepth" => dc.mean_depth_m = Some(strip_unit(rest)).filter(|v| !v.is_nan()),
            "watertemp" => dc.water_temp_c = Some(strip_unit(rest)).filter(|v| !v.is_nan()),
            "event" => dc.events.push(parse_event(rest)),
            "keyvalue" => {
                // keyvalue "Deco model" "GF 55/85"
                let trimmed = rest.trim();
                if let Some(rest2) = trimmed.strip_prefix('"') {
                    if let Some(end) = rest2.find('"') {
                        let kv_key = &rest2[..end];
                        let after = rest2[end + 1..].trim();
                        if kv_key == "Deco model" {
                            dc.deco_model = Some(unquote(after));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    dc
}

#[cfg(test)]
mod tests {
    use super::*;

    const DC_TEXT: &str = "model \"Shearwater Perdix AI\"\ndeviceid a790cf6c\ndiveid 76b9bc25\nmaxdepth 34.7m\nmeandepth 14.793m\nwatertemp 19.0\u{00b0}C\nsalinity 1020g/l\nkeyvalue \"Deco model\" \"GF 55/85\"\nevent 0:05 type=25 flags=1 name=\"gaschange\" cylinder=0 o2=32.0%\n  0:05 1.6m 25.0\u{00b0}C cns=5%\n  0:10 1.9m\n  0:15 2.5m ndl=99:00\n  4:55 15.0m 24.0\u{00b0}C\n";

    #[test]
    fn parses_dc_meta() {
        let dc = parse_divecomputer(DC_TEXT);
        assert_eq!(dc.model.as_deref(), Some("Shearwater Perdix AI"));
        assert_eq!(dc.max_depth_m, Some(34.7));
        assert_eq!(dc.mean_depth_m, Some(14.793));
        assert_eq!(dc.water_temp_c, Some(19.0));
        assert_eq!(dc.deco_model.as_deref(), Some("GF 55/85"));
    }

    #[test]
    fn parses_event() {
        let dc = parse_divecomputer(DC_TEXT);
        assert_eq!(dc.events.len(), 1);
        let ev = &dc.events[0];
        assert_eq!(ev.time_sec, 5);
        assert_eq!(ev.name, "gaschange");
        assert_eq!(ev.event_type, Some(25));
        assert_eq!(ev.flags, Some(1));
        assert_eq!(ev.cylinder, Some(0));
        assert_eq!(ev.o2_percent, Some(32.0));
    }

    #[test]
    fn parses_samples_with_carry_forward() {
        let dc = parse_divecomputer(DC_TEXT);
        assert_eq!(dc.samples.len(), 4);
        // sample 0: temp 25 and cns 5 set
        let s0 = &dc.samples[0];
        assert_eq!(s0.time_sec, 5);
        assert_eq!(s0.depth_m, 1.6);
        assert_eq!(s0.temp_c, Some(25.0));
        assert_eq!(s0.cns, Some(5.0));
        // sample 1: no new temp/ndl → carries 25°C
        let s1 = &dc.samples[1];
        assert_eq!(s1.time_sec, 10);
        assert_eq!(s1.depth_m, 1.9);
        assert_eq!(s1.temp_c, Some(25.0));
        assert_eq!(s1.ndl_sec, None);
        // sample 2: ndl=99:00 set, temp still 25
        let s2 = &dc.samples[2];
        assert_eq!(s2.time_sec, 15);
        assert_eq!(s2.temp_c, Some(25.0));
        assert_eq!(s2.ndl_sec, Some(99 * 60));
        // sample 3: temp updated to 24, ndl still 99:00
        let s3 = &dc.samples[3];
        assert_eq!(s3.time_sec, 295);
        assert_eq!(s3.depth_m, 15.0);
        assert_eq!(s3.temp_c, Some(24.0));
        assert_eq!(s3.ndl_sec, Some(99 * 60));
    }
}

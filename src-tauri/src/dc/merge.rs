// AI-generated (Claude)
//! Groups overlapping or closely-adjacent dive segments from a single
//! download batch (e.g. a CCR pre-dive loop check immediately followed by
//! the real dive) so the review step can offer a folded, single-dive view
//! while keeping every raw segment available to fall back to.
//!
//! Deliberately does not port Qt Subsurface's `dive::likely_same()`
//! (`core/dive.cpp:2125`) predicate: its depth/duration-similarity gates
//! don't fire for the short-segment-into-real-dive case this exists for (a
//! 3 min/3 m check against a 45 min/22 m dive fails both). Instead this
//! uses time-adjacency (configurable) plus depth-continuity across the
//! segment boundary — see `should_group`.

use chrono::Datelike;
use crate::dc::writer::ParsedDive;
use crate::types::{DiveEvent, Sample};

/// A batch of raw dive segments considered to be (potentially) one dive.
pub struct DiveGroup {
    /// Ascending chronological order, len >= 1.
    pub segments: Vec<ParsedDive>,
}

impl DiveGroup {
    /// Folds all segments into one dive. Segments are cloned, not consumed,
    /// so the raw per-segment data survives for the review step's "unmerge"
    /// action.
    pub fn merged(&self) -> ParsedDive {
        let mut iter = self.segments.iter();
        let first = iter.next().expect("DiveGroup always has at least one segment").clone();
        iter.fold(first, |acc, seg| merge_pair(&acc, seg))
    }
}

/// Seconds since an arbitrary epoch, usable only for relative comparisons
/// within a single download batch.
fn start_seconds(d: &ParsedDive) -> i64 {
    let days = chrono::NaiveDate::from_ymd_opt(d.year, d.month, d.day)
        .map(|nd| nd.num_days_from_ce() as i64)
        .unwrap_or(0);
    days * 86_400 + d.hour as i64 * 3600 + d.minute as i64 * 60 + d.second as i64
}

/// Mirrors Qt's `similar()` (`core/dive.cpp:2049`): true when the absolute
/// difference is below `floor`, or below 10% of the larger magnitude.
fn similar_f64(a: f64, b: f64, floor: f64) -> bool {
    let diff = (a - b).abs();
    diff < floor || diff * 10.0 < a.abs().max(b.abs())
}

/// Depth at the start (or end) of a segment: its first (or last) sample's
/// depth, or 0.0 (surface) if it has no samples.
fn boundary_depth(samples: &[Sample], at_start: bool) -> f64 {
    let sample = if at_start { samples.first() } else { samples.last() };
    sample.map(|s| s.depth_m).unwrap_or(0.0)
}

/// Two chronologically-adjacent segments (`prev.start <= dive.start`) belong
/// in the same group when they're within `gap_seconds` of each other (zero
/// or negative meaning overlap) *and* the depth is continuous across the
/// boundary — see the module doc for why depth/duration whole-dive
/// similarity (Qt's approach) isn't used here.
fn should_group(prev: &ParsedDive, dive: &ParsedDive, gap_seconds: i64) -> bool {
    let prev_end = start_seconds(prev) + prev.duration_sec as i64;
    let dive_start = start_seconds(dive);
    let time_adjacent = dive_start <= prev_end + gap_seconds;
    let depth_continuous = similar_f64(
        boundary_depth(&prev.samples, false),
        boundary_depth(&dive.samples, true),
        1.0,
    );
    time_adjacent && depth_continuous
}

/// Groups a download batch's dives by time-adjacency and depth-continuity.
/// Input order doesn't matter; each returned group's segments are sorted
/// chronologically (oldest first). Grouping is transitive: if A groups with
/// B and B groups with C, all three land in one group.
pub fn group_overlapping_dives(mut dives: Vec<ParsedDive>, gap_seconds: i64) -> Vec<DiveGroup> {
    dives.sort_by_key(start_seconds);

    let mut groups: Vec<DiveGroup> = Vec::with_capacity(dives.len());
    for dive in dives {
        let attach = groups.last().is_some_and(|g: &DiveGroup| {
            should_group(g.segments.last().expect("group is never empty"), &dive, gap_seconds)
        });
        if attach {
            groups.last_mut().unwrap().segments.push(dive);
        } else {
            groups.push(DiveGroup { segments: vec![dive] });
        }
    }
    groups
}

/// Folds two dives into one. The longer segment is treated as the "real"
/// dive; the shorter one's samples/events are time-shifted onto the merged
/// timeline and combined in. `prev.start` must be `<= dive.start`.
fn merge_pair(prev: &ParsedDive, dive: &ParsedDive) -> ParsedDive {
    let prev_start = start_seconds(prev);
    let dive_start = start_seconds(dive);
    let prev_end = prev_start + prev.duration_sec as i64;
    let dive_end = dive_start + dive.duration_sec as i64;
    let merged_duration = (prev_end.max(dive_end) - prev_start).max(0) as u32;
    let (year, month, day, hour, minute, second) =
        (prev.year, prev.month, prev.day, prev.hour, prev.minute, prev.second);

    let (primary, secondary, primary_start, secondary_start) = if dive.duration_sec >= prev.duration_sec {
        (dive, prev, dive_start, prev_start)
    } else {
        (prev, dive, prev_start, dive_start)
    };
    let primary_offset = primary_start - prev_start;
    let secondary_offset = secondary_start - prev_start;

    let mut samples = Vec::with_capacity(primary.samples.len() + secondary.samples.len());
    samples.extend(primary.samples.iter().map(|s| Sample { time_sec: s.time_sec + primary_offset, ..s.clone() }));
    samples.extend(secondary.samples.iter().map(|s| Sample { time_sec: s.time_sec + secondary_offset, ..s.clone() }));
    samples.sort_by_key(|s| s.time_sec);

    let mut events = Vec::with_capacity(primary.events.len() + secondary.events.len());
    events.extend(primary.events.iter().cloned().map(|e| DiveEvent { time_sec: e.time_sec + primary_offset, ..e }));
    events.extend(secondary.events.iter().cloned().map(|e| DiveEvent { time_sec: e.time_sec + secondary_offset, ..e }));
    events.sort_by_key(|e| e.time_sec);

    let total_duration = (primary.duration_sec as f64 + secondary.duration_sec as f64).max(1.0);
    let mean_depth_m = (primary.mean_depth_m * primary.duration_sec as f64
        + secondary.mean_depth_m * secondary.duration_sec as f64) / total_duration;

    ParsedDive {
        year, month, day, hour, minute, second,
        duration_sec: merged_duration,
        max_depth_m: primary.max_depth_m.max(secondary.max_depth_m),
        mean_depth_m,
        water_temp_c: primary.water_temp_c.or(secondary.water_temp_c),
        cylinders: if primary.cylinders.is_empty() { secondary.cylinders.clone() } else { primary.cylinders.clone() },
        samples,
        events,
        dc_model: primary.dc_model.clone(),
        device_id: primary.device_id.clone(),
        dive_id: primary.dive_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::dc::writer::{ParsedCylinder, ParsedDive};
    use crate::types::Sample;

    const FIFTEEN_MIN: i64 = 900;

    fn make_dive(hour: u32, minute: u32, duration_sec: u32) -> ParsedDive {
        ParsedDive {
            year: 2026, month: 6, day: 11,
            hour, minute, second: 0,
            duration_sec,
            max_depth_m: 10.0,
            mean_depth_m: 5.0,
            water_temp_c: None,
            cylinders: Vec::<ParsedCylinder>::new(),
            samples: Vec::<Sample>::new(),
            events: vec![],
            dc_model: "Shearwater Perdix AI".to_string(),
            device_id: "a790cf6c".to_string(),
            dive_id: vec![hour as u8, minute as u8],
        }
    }

    #[test]
    fn non_overlapping_dives_beyond_the_gap_stay_separate() {
        // 09:00-09:30, then 11:00-11:45 — 90 min gap, way past 15 min.
        let dives = vec![make_dive(9, 0, 1800), make_dive(11, 0, 2700)];
        let groups = super::group_overlapping_dives(dives, FIFTEEN_MIN);
        assert_eq!(groups.len(), 2, "dives with a gap beyond the threshold must stay separate");
    }

    #[test]
    fn overlapping_dives_are_grouped_together() {
        // 09:00-09:40 overlaps 09:30-10:00.
        let dives = vec![make_dive(9, 0, 2400), make_dive(9, 30, 1800)];
        let groups = super::group_overlapping_dives(dives, 0);
        assert_eq!(groups.len(), 1, "overlapping dives must be grouped even with zero gap tolerance");
        assert_eq!(groups[0].segments.len(), 2);
    }

    #[test]
    fn short_segment_within_the_gap_threshold_is_grouped_with_the_real_dive() {
        // A 2-minute pre-dive check ending at 09:02, then the real dive starting at 09:10 (8 min later).
        let dives = vec![make_dive(9, 0, 120), make_dive(9, 10, 2700)];
        let groups = super::group_overlapping_dives(dives, FIFTEEN_MIN);
        assert_eq!(groups.len(), 1, "a short segment within the gap threshold must group with the adjacent real dive");
    }

    #[test]
    fn a_gap_larger_than_the_threshold_is_not_grouped() {
        // Same segments, but the threshold is now tighter than the 8-minute gap.
        let dives = vec![make_dive(9, 0, 120), make_dive(9, 10, 2700)];
        let groups = super::group_overlapping_dives(dives, 5 * 60);
        assert_eq!(groups.len(), 2, "a gap larger than the configured threshold must not be grouped");
    }

    #[test]
    fn depth_discontinuity_at_the_boundary_prevents_grouping() {
        // Time-adjacent (well within the gap), but the first segment ends deep
        // and the second starts at the surface — not a continuous profile.
        let mut a = make_dive(9, 0, 600);
        a.samples = vec![Sample { time_sec: 599, depth_m: 20.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None }];
        let mut b = make_dive(9, 2, 600);
        b.samples = vec![Sample { time_sec: 0, depth_m: 0.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None }];
        let groups = super::group_overlapping_dives(vec![a, b], FIFTEEN_MIN);
        assert_eq!(groups.len(), 2, "a depth jump at the boundary must not be grouped despite time-adjacency");
    }

    #[test]
    fn depth_continuity_near_the_surface_allows_grouping() {
        // Both segments have no samples (empty), so boundary depth defaults
        // to the surface (0.0) on both sides — continuous.
        let dives = vec![make_dive(9, 0, 120), make_dive(9, 10, 2700)];
        let groups = super::group_overlapping_dives(dives, FIFTEEN_MIN);
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn transitive_chain_of_overlaps_forms_one_group() {
        // 09:00-09:20, 09:15-09:35, 09:30-10:15 — each overlaps the next.
        let dives = vec![make_dive(9, 0, 1200), make_dive(9, 15, 1200), make_dive(9, 30, 2700)];
        let groups = super::group_overlapping_dives(dives, 0);
        assert_eq!(groups.len(), 1, "a chain of pairwise overlaps must collapse to one group");
        assert_eq!(groups[0].segments.len(), 3);
    }

    #[test]
    fn unrelated_dives_a_day_apart_are_untouched() {
        let a = make_dive(9, 0, 1800);
        let mut b = make_dive(9, 0, 1800);
        b.day = 12;
        let groups = super::group_overlapping_dives(vec![a, b], FIFTEEN_MIN);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn empty_input_returns_no_groups() {
        assert!(super::group_overlapping_dives(vec![], FIFTEEN_MIN).is_empty());
    }

    #[test]
    fn single_dive_forms_its_own_group() {
        let groups = super::group_overlapping_dives(vec![make_dive(9, 0, 1800)], FIFTEEN_MIN);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].segments.len(), 1);
    }

    #[test]
    fn input_order_does_not_matter() {
        let dives = vec![make_dive(11, 0, 2700), make_dive(9, 0, 1800)]; // reverse chronological
        let groups = super::group_overlapping_dives(dives, FIFTEEN_MIN);
        assert_eq!(groups.len(), 2);
        assert_eq!((groups[0].segments[0].hour, groups[0].segments[0].minute), (9, 0), "groups must come out sorted chronologically");
    }

    // ── DiveGroup::merged() ──────────────────────────────────────────────

    #[test]
    fn merged_of_a_single_segment_group_equals_that_segment() {
        let group = super::DiveGroup { segments: vec![make_dive(9, 0, 1800)] };
        let merged = group.merged();
        assert_eq!(merged.duration_sec, 1800);
        assert_eq!((merged.hour, merged.minute), (9, 0));
    }

    #[test]
    fn merged_spans_from_earliest_start_to_latest_end() {
        let group = super::DiveGroup { segments: vec![make_dive(9, 0, 2400), make_dive(9, 30, 1800)] }; // 09:00-09:40, 09:30-10:00
        let merged = group.merged();
        assert_eq!((merged.hour, merged.minute), (9, 0), "merged dive must start at the earliest segment's start");
        assert_eq!(merged.duration_sec, 3600, "merged duration must span to the latest segment's end (09:00-10:00)");
    }

    #[test]
    fn merged_takes_max_depth_across_segments() {
        let mut a = make_dive(9, 0, 2400);
        a.max_depth_m = 5.0;
        let mut b = make_dive(9, 30, 1800);
        b.max_depth_m = 22.0;
        let group = super::DiveGroup { segments: vec![a, b] };
        let merged = group.merged();
        assert!((merged.max_depth_m - 22.0).abs() < 1e-9);
    }

    #[test]
    fn merged_combines_and_time_shifts_samples_from_both_segments() {
        use crate::types::Sample;
        let mut a = make_dive(9, 0, 120);
        a.samples = vec![Sample { time_sec: 0, depth_m: 0.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None }];
        let mut b = make_dive(9, 1, 120); // starts 60s after a
        b.samples = vec![Sample { time_sec: 0, depth_m: 8.0, temp_c: None, ndl_sec: None, tts_sec: None, cns: None, pressure_bar: None }];
        let group = super::DiveGroup { segments: vec![a, b] };
        let merged = group.merged();
        assert_eq!(merged.samples.len(), 2, "samples from both segments must be present");
        let times: Vec<i64> = merged.samples.iter().map(|s| s.time_sec).collect();
        assert_eq!(times, vec![0, 60], "b's sample must be shifted by its 60s offset from the merged start");
    }

    #[test]
    fn merged_combines_events_from_both_segments() {
        use crate::types::DiveEvent;
        let mut a = make_dive(9, 0, 120);
        a.events = vec![DiveEvent { time_sec: 0, name: "gaschange".into(), event_type: None, flags: None, value: None, cylinder: None, o2_percent: None }];
        let mut b = make_dive(9, 1, 120);
        b.events = vec![DiveEvent { time_sec: 0, name: "surface".into(), event_type: None, flags: None, value: None, cylinder: None, o2_percent: None }];
        let group = super::DiveGroup { segments: vec![a, b] };
        assert_eq!(group.merged().events.len(), 2);
    }

    #[test]
    fn merged_of_a_three_segment_chain_spans_all_of_them() {
        let group = super::DiveGroup { segments: vec![make_dive(9, 0, 1200), make_dive(9, 15, 1200), make_dive(9, 30, 2700)] }; // 09:00-09:20, 09:15-09:35, 09:30-10:15
        let merged = group.merged();
        assert_eq!((merged.hour, merged.minute), (9, 0));
        assert_eq!(merged.duration_sec, 4500, "must span 09:00 to 10:15");
    }

    #[test]
    fn original_segments_survive_merging_unchanged() {
        // The whole point of DiveGroup over a destructive merge: raw segments
        // must still be there for the review step's "unmerge" action.
        let group = super::DiveGroup { segments: vec![make_dive(9, 0, 2400), make_dive(9, 30, 1800)] };
        let _ = group.merged();
        assert_eq!(group.segments.len(), 2, "merged() must not consume/mutate the original segments");
        assert_eq!(group.segments[0].duration_sec, 2400);
        assert_eq!(group.segments[1].duration_sec, 1800);
    }
}

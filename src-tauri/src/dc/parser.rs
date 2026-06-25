// AI-generated (Claude)
//! Parse a single raw dive from libdivecomputer into a [`ParsedDive`].
//!
//! The caller is responsible for creating the `dc_parser_t*` (via
//! `dc_parser_new2`) and destroying it after this function returns.
use std::ffi::c_void;
use crate::dc::ffi::*;
use crate::dc::writer::{ParsedCylinder, ParsedDive};
use crate::types::Sample;

struct SampleState {
    samples: Vec<Sample>,
    current: Option<Sample>,
}

unsafe extern "C" fn sample_cb(
    sample_type: dc_sample_type_t,
    value: *const dc_sample_value_t,
    userdata: *mut c_void,
) {
    let state = &mut *(userdata as *mut SampleState);
    match sample_type {
        dc_sample_type_t_DC_SAMPLE_TIME => {
            if let Some(prev) = state.current.take() {
                state.samples.push(prev);
            }
            let time_sec = (*value).time as i64;
            state.current = Some(Sample {
                time_sec,
                depth_m: 0.0,
                temp_c: None,
                ndl_sec: None,
                tts_sec: None,
                cns: None,
                pressure_bar: None,
            });
        }
        dc_sample_type_t_DC_SAMPLE_DEPTH => {
            if let Some(ref mut s) = state.current {
                s.depth_m = (*value).depth;
            }
        }
        dc_sample_type_t_DC_SAMPLE_TEMPERATURE => {
            if let Some(ref mut s) = state.current {
                s.temp_c = Some((*value).temperature);
            }
        }
        dc_sample_type_t_DC_SAMPLE_DECO => {
            // NDL and TTS come via the DECO sample type.
            // deco.type_ == DC_DECO_NDL means NDL; deco.time is the NDL value in seconds.
            // deco.tts is the time-to-surface in seconds (always present in the struct).
            let deco = &(*value).deco;
            if let Some(ref mut s) = state.current {
                if deco.type_ == dc_deco_type_t_DC_DECO_NDL {
                    s.ndl_sec = Some(deco.time as i64);
                }
                if deco.tts > 0 {
                    s.tts_sec = Some(deco.tts as i64);
                }
            }
        }
        dc_sample_type_t_DC_SAMPLE_CNS => {
            // libdivecomputer CNS is a fraction 0.0–1.0; our Sample stores percentage.
            if let Some(ref mut s) = state.current {
                s.cns = Some((*value).cns * 100.0);
            }
        }
        dc_sample_type_t_DC_SAMPLE_PRESSURE => {
            // pressure.value is bar; pressure.tank is the tank index (ignored here;
            // we record tank 0 pressure as the main cylinder pressure).
            if let Some(ref mut s) = state.current {
                if (*value).pressure.tank == 0 {
                    s.pressure_bar = Some((*value).pressure.value);
                }
            }
        }
        _ => {}
    }
}

/// Parse raw dive bytes from libdivecomputer into a [`ParsedDive`].
///
/// # Safety
/// `parser` must be a valid `dc_parser_t*` whose data has already been set.
/// The caller must destroy `parser` after this call returns.
pub fn parse_dive(
    parser: *mut dc_parser_t,
    dc_model: &str,
    device_id: &str,
    fingerprint: Vec<u8>,
) -> Result<ParsedDive, String> {
    unsafe {
        // Extract datetime via dc_parser_get_datetime (not dc_parser_get_field).
        let mut dt: dc_datetime_t = std::mem::zeroed();
        dc_parser_get_datetime(parser, &mut dt);

        // Extract dive time in seconds.
        let mut duration_sec: u32 = 0;
        dc_parser_get_field(
            parser,
            dc_field_type_t_DC_FIELD_DIVETIME,
            0,
            &mut duration_sec as *mut u32 as *mut _,
        );

        // Extract max depth.
        let mut max_depth: f64 = 0.0;
        dc_parser_get_field(
            parser,
            dc_field_type_t_DC_FIELD_MAXDEPTH,
            0,
            &mut max_depth as *mut f64 as *mut _,
        );

        // Extract average depth.
        let mut mean_depth: f64 = 0.0;
        dc_parser_get_field(
            parser,
            dc_field_type_t_DC_FIELD_AVGDEPTH,
            0,
            &mut mean_depth as *mut f64 as *mut _,
        );

        // Use minimum temperature as proxy for water temperature.
        // (DC_FIELD_TEMPERATURE_WATER does not exist in this libdivecomputer version.)
        let mut water_temp: f64 = f64::NAN;
        dc_parser_get_field(
            parser,
            dc_field_type_t_DC_FIELD_TEMPERATURE_MINIMUM,
            0,
            &mut water_temp as *mut f64 as *mut _,
        );
        let water_temp_c = if water_temp.is_nan() || water_temp == 0.0 {
            None
        } else {
            Some(water_temp)
        };

        // Extract tank count.
        let mut tank_count: u32 = 0;
        dc_parser_get_field(
            parser,
            dc_field_type_t_DC_FIELD_TANK_COUNT,
            0,
            &mut tank_count as *mut u32 as *mut _,
        );

        // Extract cylinders.
        let mut cylinders = Vec::new();
        for i in 0..tank_count {
            let mut tank: dc_tank_t = std::mem::zeroed();
            dc_parser_get_field(
                parser,
                dc_field_type_t_DC_FIELD_TANK,
                i,
                &mut tank as *mut dc_tank_t as *mut _,
            );
            // Get gas mix for this tank's gasmix index.
            let mut gasmix: dc_gasmix_t = std::mem::zeroed();
            dc_parser_get_field(
                parser,
                dc_field_type_t_DC_FIELD_GASMIX,
                tank.gasmix,
                &mut gasmix as *mut dc_gasmix_t as *mut _,
            );
            cylinders.push(ParsedCylinder {
                description: None,
                o2_percent: if gasmix.oxygen > 0.0 { Some(gasmix.oxygen * 100.0) } else { None },
                he_percent: if gasmix.helium > 0.0 { Some(gasmix.helium * 100.0) } else { None },
                volume_l: if tank.volume > 0.0 { Some(tank.volume) } else { None },
                work_pressure_bar: if tank.workpressure > 0.0 { Some(tank.workpressure) } else { None },
                start_bar: if tank.beginpressure > 0.0 { Some(tank.beginpressure) } else { None },
                end_bar: if tank.endpressure > 0.0 { Some(tank.endpressure) } else { None },
            });
        }

        // Collect samples via callback.
        let mut state = SampleState { samples: vec![], current: None };
        dc_parser_samples_foreach(
            parser,
            Some(sample_cb),
            &mut state as *mut SampleState as *mut _,
        );
        // Flush the last in-progress sample.
        if let Some(last) = state.current.take() {
            state.samples.push(last);
        }

        Ok(ParsedDive {
            year: dt.year,
            month: dt.month as u32,
            day: dt.day as u32,
            hour: dt.hour as u32,
            minute: dt.minute as u32,
            second: dt.second as u32,
            duration_sec,
            max_depth_m: max_depth,
            mean_depth_m: mean_depth,
            water_temp_c,
            cylinders,
            samples: state.samples,
            events: vec![],
            dc_model: dc_model.to_string(),
            device_id: device_id.to_string(),
            dive_id: fingerprint,
        })
    }
}

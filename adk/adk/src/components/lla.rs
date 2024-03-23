use adk_derive::{c_result_fn, CConstructor, CDebug};

use crate::utils;

/// LLA structure
#[derive(Debug, Default, Clone, CDebug, CConstructor)]
pub struct Lla {
    lat: f64,
    lon: f64,
    alt: f64,
}

/// Update the latitude of the LLA point
/// 
/// This will overwrite the old value
/// 
/// unit: degrees
#[c_result_fn]
fn lla_with_lat(lla: *mut Lla, lat: f64) -> utils::CResult {
    utils::check_null(lla)?.lat = lat;

    Ok(())
}

/// Update the longitude of the LLA point
/// 
/// This will overwrite the old value
/// 
/// unit: degrees
#[c_result_fn]
fn lla_with_lon(lla: *mut Lla, lon: f64) -> utils::CResult {
    utils::check_null(lla)?.lon = lon;

    Ok(())
}

/// Update the altitude of the LLA point
/// 
/// This will overwrite the old value
/// 
/// unit: meters
#[c_result_fn]
fn lla_with_alt(lla: *mut Lla, alt: f64) -> utils::CResult {
    utils::check_null(lla)?.alt = alt;

    Ok(())
}

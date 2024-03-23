use adk_derive::{c_result_fn, CDebug};

use crate::utils;

#[derive(Debug, Default, Clone, CDebug)]
pub struct Lla {
    lat: f64,
    lon: f64,
    alt: f64,
}

#[no_mangle]
pub extern "C" fn lla_new() -> *mut Lla {
    Box::into_raw(Box::default())
}

#[c_result_fn]
fn lla_with_lat(lla: *mut Lla, lat: f64) -> utils::CResult {
    let lla = utils::check_null(lla)?;

    lla.lat = lat;

    Ok(())
}

#[c_result_fn]
fn lla_with_lon(lla: *mut Lla, lon: f64) -> utils::CResult {
    let lla = utils::check_null(lla)?;

    lla.lon = lon;

    Ok(())
}

#[c_result_fn]
fn lla_with_alt(lla: *mut Lla, alt: f64) -> utils::CResult {
    let lla = utils::check_null(lla)?;

    lla.alt = alt;

    Ok(())
}

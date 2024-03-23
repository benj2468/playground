use adk_derive::{c_result_fn, CConstructor, CDebug};

use crate::utils;

/// Sensor structure
#[derive(Debug, Default, Clone, CDebug, CConstructor)]
pub struct Sensor {
    name: String,
}

/// Update the name of the Sensor
/// 
/// This will overwrite the old value
#[c_result_fn]
fn sensor_with_name(sensor: *mut Sensor, name: *const libc::c_char) -> utils::CResult {
    utils::check_null(sensor)?.name = utils::as_string(name)?;

    Ok(())
}

use adk_derive::{c_result_fn, CDebug};

use crate::utils;

use super::{dis::DisConfig, lla::Lla, sensor::Sensor};



#[derive(Debug, Default, Clone, CDebug)]
pub struct Entity {
    name: String,
    position: Lla,
    speed: f64,
    heading: f64,
    sensors: Vec<Sensor>,
    dis: Option<DisConfig>,
}

#[no_mangle]
pub extern "C" fn entity_new() -> *mut Entity {
    Box::into_raw(Box::new(Entity::default()))
}

#[c_result_fn]
fn entity_with_name(entity: *mut Entity, name: *const libc::c_char) -> utils::CResult {
    let entity = utils::check_null(entity)?;

    entity.name = utils::as_string(name)?;

    Ok(())
}

#[c_result_fn]
fn entity_with_position(entity: *mut Entity, position: &Lla) -> utils::CResult {
    let entity = utils::check_null(entity)?;

    entity.position = position.clone();

    Ok(())
}

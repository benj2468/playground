use adk_derive::{c_result_fn, CConstructor, CDebug};

use crate::utils;

use super::{dis::DisConfig, lla::Lla, sensor::Sensor};

/// Entity structure
#[derive(Debug, Default, Clone, CDebug, CConstructor)]
pub struct Entity {
    name: String,
    position: Lla,
    speed: f64,
    heading: f64,
    sensors: Vec<Sensor>,
    dis: Option<DisConfig>,
}

/// Update the name of the Entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_with_name(entity: *mut Entity, name: *const libc::c_char) -> utils::CResult {
    utils::check_null(entity)?.name = utils::as_string(name)?;

    Ok(())
}

/// Update the position of the Entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_with_position(entity: *mut Entity, position: &Lla) -> utils::CResult {
    utils::check_null(entity)?.position = position.clone();

    Ok(())
}

/// Update the speed of the Entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_with_speed(entity: *mut Entity, speed: f64) -> utils::CResult {
    utils::check_null(entity)?.speed = speed;

    Ok(())
}

/// Update the heading of the Entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_with_heading(entity: *mut Entity, heading: f64) -> utils::CResult {
    utils::check_null(entity)?.heading = heading;

    Ok(())
}

/// Add a sensor to the Entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_with_sensor(entity: *mut Entity, sensor: &Sensor) -> utils::CResult {
    utils::check_null(entity)?.sensors.push(sensor.clone());

    Ok(())
}

/// Pop the last sensor added tot he entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_pop_sensor(entity: *mut Entity, sensor: *mut Sensor) -> utils::CResult {
    let entity = utils::check_null(entity)?;
    let sensor = utils::check_null(sensor)?;

    if let Some(removed) = entity.sensors.pop() {
        *sensor = removed;
        Ok(())
    } else {
        Err("No sensor to remove".into())
    }
}

/// Update the DIS config of the Entity
/// 
/// This will overwrite the old value
#[c_result_fn]
fn entity_with_dis_config(entity: *mut Entity, dis_config: &DisConfig) -> utils::CResult {
    utils::check_null(entity)?.dis.replace(dis_config.clone());

    Ok(())
}

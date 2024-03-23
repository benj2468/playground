use adk_derive::{c_result_fn, CDebug};

use crate::utils;

use super::entity::Entity;

#[derive(Debug, Default, CDebug)]
pub struct Scenario {
    entities: Vec<Entity>,
}

#[no_mangle]
pub extern "C" fn scenario_new() -> *mut Scenario {
    Box::into_raw(Box::new(Scenario::default()))
}

#[c_result_fn]
fn scenario_with_entity(sim: *mut Scenario, entity: &Entity) -> utils::CResult {
    let sim = utils::check_null(sim)?;

    sim.entities.push(entity.clone());

    Ok(())
}

use adk_derive::{c_result_fn, CConstructor, CDebug};

use crate::utils;

use super::entity::Entity;

/// Scenario structure
#[derive(Debug, Default, Clone, CDebug, CConstructor)]
pub struct Scenario {
    entities: Vec<Entity>,
}

/// Add an Entity to the Scenario
/// 
/// This will overwrite the old value
#[c_result_fn]
fn scenario_with_entity(sim: *mut Scenario, entity: &Entity) -> utils::CResult {
    utils::check_null(sim)?.entities.push(entity.clone());

    Ok(())
}

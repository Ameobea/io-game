//! Responsible for generating the initial version of the world.

use nalgebra::Isometry2;
use nphysics2d::algebra::Velocity2;

use physics::entities::EntityType;

pub struct EntitySpawn {
    pub isometry: Isometry2<f32>,
    pub entity: EntityType,
    pub velocity: Velocity2<f32>,
}

pub fn get_initial_entities() -> Vec<EntitySpawn> {
    Vec::new()
}

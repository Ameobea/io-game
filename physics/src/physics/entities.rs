//! Contains definitions for the various types of entities that are spawnable into the world.

use nalgebra::Vector2;
use ncollide2d::shape::{Cuboid, ShapeHandle};

use super::COLLIDER_MARGIN;

pub fn create_player_shape_handle(size: f32) -> ShapeHandle<f32> {
    let shape = Cuboid::new(Vector2::new(
        size / 2. - COLLIDER_MARGIN,
        size / 2. - COLLIDER_MARGIN,
    ));
    ShapeHandle::new(shape)
}

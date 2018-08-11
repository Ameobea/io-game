//! Defines the actual physics engine which holds the state of all entities and handles performing
//! the steps of the physics simulation.

use nalgebra::Vector2;

pub mod entities;
#[cfg(feature = "elixir-interop")]
pub mod server;
pub mod world;

pub use self::world::PhysicsWorldInner;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Movement {
    Stop,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Default for Movement {
    fn default() -> Self {
        Movement::Stop
    }
}

impl Into<Vector2<f32>> for Movement {
    fn into(self) -> Vector2<f32> {
        let (dir_x, dir_y): (f32, f32) = match self {
            Movement::Up => (0., -1.),
            Movement::UpRight => (1., -1.),
            Movement::Right => (1., 0.),
            Movement::DownRight => (1., 1.),
            Movement::Down => (0., 1.),
            Movement::DownLeft => (-1., 1.),
            Movement::Left => (-1., 0.),
            Movement::UpLeft => (-1., -1.),
            Movement::Stop => {
                return Vector2::new(0., 0.);
            }
        };
        Vector2::new(dir_x, dir_y).normalize()
    }
}

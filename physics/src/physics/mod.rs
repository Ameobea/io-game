//! Defines the actual physics engine which holds the state of all entities and handles performing
//! the steps of the physics simulation.

use std::collections::BTreeMap;
use std::convert::TryInto;
use std::sync::Mutex;

use nalgebra::{Isometry2, Vector2};
use nphysics2d::algebra::Force2;
use nphysics2d::object::{BodyHandle, BodyMut, ColliderHandle, Material, RigidBody};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;
use rustler::error::Error as NifError;
use rustler::types::atom::Atom;
use rustler::{Encoder, Env, NifResult, Term};

use super::atoms;

pub mod entities;

use self::entities::create_player_shape_handle;

pub const COLLIDER_MARGIN: f32 = 0.01;
pub const DEFAULT_PLAYER_SIZE: f32 = 20.0;
pub const FRICTION_PER_TICK: f32 = 0.98;

pub struct PhysicsWorldInner {
    uuid_map: BTreeMap<String, (ColliderHandle, BodyHandle)>,
    world: World<f32>,
    user_handles: Vec<BodyHandle>,
}

impl PhysicsWorldInner {
    pub fn new() -> Self {
        PhysicsWorldInner {
            uuid_map: BTreeMap::new(),
            world: World::new(),
            user_handles: Vec::new(),
        }
    }
}

pub struct PhysicsWorld(Mutex<PhysicsWorldInner>);

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld(Mutex::new(PhysicsWorldInner::new()))
    }

    pub fn apply<F: FnOnce(&mut PhysicsWorldInner) -> ()>(&self, f: F) {
        let mut inner = self.0.lock().unwrap();
        f(&mut inner)
    }
}

lazy_static! {
    /// The main world in which the entire simulation exists
    pub static ref WORLD: PhysicsWorld = PhysicsWorld::new();
}

/// Generates the world with the set of initial objects and returns handles to them which can be
/// stored in Elixir.
pub fn init() {}

#[derive(NifStruct)]
#[module = "Elixir.NativePhysics.UserDiff"]
pub struct UserDiff<'a> {
    id: String,
    action_type: Atom,
    payload: Term<'a>,
}

impl<'a> TryInto<InternalUserDiff> for UserDiff<'a> {
    type Error = NifError;

    fn try_into(self) -> NifResult<InternalUserDiff> {
        let internal_action = match self.action_type {
            t if atoms::Movement() == t => {
                let movement = Movement::from_term(self.payload)?;
                InternalUserDiffAction::Movement(movement)
            }
            _ => Err(NifError::Atom("invalid_action_type"))?,
        };

        Ok(InternalUserDiff {
            id: self.id,
            action: internal_action,
        })
    }
}

pub struct InternalUserDiff {
    id: String,
    action: InternalUserDiffAction,
}

/// Holds a change between the status of a user between ticks.  This status is different than the
/// physics state held by the physics engine which consists of position, velocity, rotation, etc.
/// and instead
pub enum InternalUserDiffAction {
    Movement(Movement),
}

pub enum Movement {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    Stop,
}

impl Movement {
    pub fn from_term<'a>(term: Term<'a>) -> NifResult<Self> {
        match term {
            t if atoms::UP() == t => Ok(Movement::Up),
            t if atoms::UP_RIGHT() == t => Ok(Movement::UpRight),
            t if atoms::RIGHT() == t => Ok(Movement::Right),
            t if atoms::DOWN_RIGHT() == t => Ok(Movement::DownRight),
            t if atoms::DOWN() == t => Ok(Movement::Down),
            t if atoms::DOWN_LEFT() == t => Ok(Movement::DownLeft),
            t if atoms::LEFT() == t => Ok(Movement::Left),
            t if atoms::UP_LEFT() == t => Ok(Movement::UpLeft),
            _ => Err(NifError::Atom("invalid_movement_atom")),
        }
    }
}

impl Into<Force2<f32>> for Movement {
    fn into(self) -> Force2<f32> {
        let (dir_x, dir_y): (f32, f32) = match self {
            Movement::Up => (0., -1.),
            Movement::UpRight => (1., -1.),
            Movement::Right => (1., 0.),
            Movement::DownRight => (1., 1.),
            Movement::Down => (0., 1.),
            Movement::DownLeft => (-1., 1.),
            Movement::Left => (-1., 0.),
            Movement::UpLeft => (-1., -1.),
            Movement::Stop => (0., 0.),
        };
        let direction_vector: Vector2<f32> = Vector2::new(dir_x, dir_y).normalize();

        Force2::linear(direction_vector)
    }
}

/// An update that is returned from the physics world as the result of a tick.  These should
/// generally map to updates of the outer state held by Elixir or updates sent directly to users.
#[derive(NifStruct)]
#[module = "Elixir.NativePhysics.Update"]
pub struct Update<'a> {
    id: String,
    update_type: Atom,
    payload: Term<'a>,
}

impl<'a> Update<'a> {
    pub fn new_isometry_update(env: Env<'a>, id: String, isometry: Isometry2<f32>) -> Self {
        let isometry: Isometry = isometry.into();

        Update {
            id,
            update_type: atoms::Isometry(),
            payload: isometry.encode(env),
        }
    }
}

#[derive(NifStruct)]
#[module = "Elixir.NativePhysics.Position"]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(NifStruct)]
#[module = "Elixir.NativePhysics.Isometry"]
pub struct Isometry {
    pub position: Position,
    pub rotation: f32,
}

impl Into<Isometry> for Isometry2<f32> {
    fn into(self) -> Isometry {
        Isometry {
            position: Position {
                x: self.translation.vector.x,
                y: self.translation.vector.y,
            },
            rotation: self.rotation.angle(),
        }
    }
}

/// This is called by the Elixir code every tick of the game.  It will be provided an array of
/// updates to the game state which will be applied to the internal state that the physics
/// engine manages and return a set of messages that need to be sent to the user.
pub fn tick<'a>(env: Env<'a>, diffs: Vec<InternalUserDiff>) {
    WORLD.apply(move |world| {
        let &mut PhysicsWorldInner {
            ref mut uuid_map,
            ref mut world,
            ref mut user_handles,
        } = world;

        // Apply friction for all user entities
        for user_body_handle in user_handles {
            let mut user_entity_body = world.body_mut(*user_body_handle);
            let user_rigid_body: &mut RigidBody<f32> = match user_entity_body {
                BodyMut::RigidBody(rigid_body) => rigid_body,
                _ => {
                    println!("ERROR: Player wasn't a rigid body!");
                    continue;
                }
            };

            let linear_velocity = user_rigid_body.velocity().linear;
            user_rigid_body.set_linear_velocity(linear_velocity * (1.0 - FRICTION_PER_TICK));
        }

        // Process all incoming diffs from Elixir
        for InternalUserDiff { id, action } in diffs {
            let (_collider_handle, body_handle) = match uuid_map.get(&id) {
                Some(handle) => handle,
                None => {
                    println!(
                        "ERROR: Received update for user with id {} but no such user exists!",
                        id
                    );
                    continue;
                }
            };

            match action {
                InternalUserDiffAction::Movement(movement) => {
                    let mut user_entity_body = world.body_mut(*body_handle);
                    let rigid_body: &mut RigidBody<f32> = match user_entity_body {
                        BodyMut::RigidBody(rigid_body) => rigid_body,
                        _ => {
                            println!("ERROR: Player wasn't a rigid body!");
                            continue;
                        }
                    };

                    let force: Force2<f32> = movement.into();
                    rigid_body.apply_force(&force);
                }
            }
        }
    })
}

/// Adds a new user into the world with a given UUID, returning the location at which it was
/// spawned in.
pub fn spawn_user(uuid: String) -> Position {
    let player_shape_handle = create_player_shape_handle(DEFAULT_PLAYER_SIZE);
    let pos = Isometry2::new(Vector2::new(0.0, 0.0), 0.0);

    // `ShapeHandle` implements `AsRef<Shape>`, and `Shape` implements `Volumetric` which has the
    // `inertia()` and `center_of_mass()` functions.  Yeah.
    let inertia = player_shape_handle.inertia(1.0);
    let center_of_mass = player_shape_handle.center_of_mass();

    WORLD.apply(move |world| {
        let &mut PhysicsWorldInner {
            ref mut uuid_map,
            ref mut world,
            ref mut user_handles,
        } = world;

        // Add a rigid body and collision object into the world for the player
        let body_handle = world.add_rigid_body(pos, inertia, center_of_mass);
        let collider_handle = world.add_collider(
            COLLIDER_MARGIN,
            player_shape_handle,
            body_handle,
            Isometry2::identity(),
            Material::default(),
        );

        // Insert an entry into the UUID map for the created player's internal handles
        uuid_map.insert(uuid, (collider_handle, body_handle));
        // Add the handle to the `user_handles` cache
        user_handles.push(body_handle);
    });

    Position { x: 0.0, y: 0.0 }
}

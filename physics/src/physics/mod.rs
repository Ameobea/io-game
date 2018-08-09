//! Defines the actual physics engine which holds the state of all entities and handles performing
//! the steps of the physics simulation.

use std::collections::BTreeMap;
use std::sync::Mutex;

use nalgebra::{Isometry2, Vector2};
use ncollide2d::events::ContactEvent;
use ncollide2d::query::Proximity;
use ncollide2d::world::CollisionObject;
use nphysics2d::algebra::Force2;
use nphysics2d::object::{
    Body, BodyHandle, ColliderData, ColliderHandle, Material, RigidBody, SensorHandle,
};
use nphysics2d::solver::SignoriniModel;
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;
use rustler::error::Error as NifError;
use rustler::types::atom::Atom;
use rustler::{Encoder, Env, NifResult, Term};
use uuid::Uuid;

use super::{atoms, UserDiff};
use conf::CONF;
use worldgen::{get_initial_entities, EntitySpawn};

pub mod entities;

use self::entities::{create_player_shape_handle, EntityType, BEAM_SHAPE_HANDLE};

pub const COLLIDER_MARGIN: f32 = CONF.physics.collider_margin;
pub const DEFAULT_PLAYER_SIZE: f32 = CONF.game.default_player_size;

pub struct EntityHandles {
    collider_handle: ColliderHandle,
    body_handle: BodyHandle,
    beam_handle: Option<SensorHandle>,
}

pub struct PhysicsWorldInner {
    /// Maps UUIDs to internal physics entity handles
    uuid_map: BTreeMap<String, EntityHandles>,
    /// Maps `ColliderHandle`s to UUIDs
    handle_map: BTreeMap<ColliderHandle, (String, EntityType)>,
    world: World<f32>,
    user_handles: Vec<(BodyHandle, ColliderHandle)>,
    /// Maps the collider handles of beam sensors to the User entities that own them
    beam_sensors: BTreeMap<ColliderHandle, String>,
}

impl PhysicsWorldInner {
    pub fn new() -> Self {
        let mut world = World::new();
        world.set_contact_model(SignoriniModel::new());
        world.set_timestep(CONF.physics.engine_time_step);

        let mut uuid_map = BTreeMap::new();
        let mut handle_map = BTreeMap::new();

        // Populate the world with initial entities
        for EntitySpawn {
            isometry,
            velocity,
            entity,
        } in get_initial_entities()
        {
            let shape_handle = entity.get_shape_handle();
            let inertia = shape_handle.inertia(entity.get_density());
            let center_of_mass = shape_handle.center_of_mass();
            let body_handle = world.add_rigid_body(isometry, inertia, center_of_mass);
            {
                world
                    .rigid_body_mut(body_handle)
                    .unwrap()
                    .set_velocity(velocity);
            }

            let collider_handle = world.add_collider(
                COLLIDER_MARGIN,
                shape_handle,
                body_handle,
                Isometry2::identity(),
                Material::default(),
            );

            let uuid = Uuid::new_v4();
            let handles = EntityHandles {
                collider_handle,
                body_handle,
                beam_handle: None,
            };
            uuid_map.insert(uuid.to_string(), handles);
            handle_map.insert(collider_handle, (uuid.to_string(), entity));
        }

        PhysicsWorldInner {
            uuid_map,
            handle_map,
            world,
            user_handles: Vec::new(),
            beam_sensors: BTreeMap::new(),
        }
    }

    pub fn apply_diff<'a>(
        &mut self,
        env: Env<'a>,
        diff: InternalUserDiff,
        updates: &mut Vec<Update<'a>>,
    ) {
        let EntityHandles {
            body_handle,
            collider_handle,
            ref mut beam_handle,
        } = match self.uuid_map.get_mut(&diff.id) {
            Some(handle) => handle,
            None => {
                println!(
                    "ERROR: Received update for user with id {} but no such user exists!",
                    diff.id
                );
                return;
            }
        };

        match diff.action {
            InternalUserDiffAction::Movement(movement) => {
                let rigid_body: &mut RigidBody<f32> = self
                    .world
                    .rigid_body_mut(*body_handle)
                    .expect("ERROR: Player wasn't a rigid body!");

                let force: Force2<f32> = movement.into();
                rigid_body.apply_force(&force);
            }
            InternalUserDiffAction::BeamAim { x, y } => {
                let (_, entity) = self
                    .handle_map
                    .get_mut(collider_handle)
                    .expect("ERROR: No matching entry in `handle_map` for entry in `uuid_map`!");
                match *entity {
                    EntityType::Player {
                        ref mut beam_aim, ..
                    } => {
                        // Calculate the angle in radians produced by looking at (x, y) from the
                        // player's position
                        let new_beam_aim = (y / x).atan();

                        *beam_aim = new_beam_aim;
                        match beam_handle {
                            Some(beam_handle) => {
                                // Move the beam sensor
                                let mut sensor = self
                                    .world
                                    .collision_world_mut()
                                    .collision_object_mut(*beam_handle)
                                    .expect(
                                        "No beam sensor in the world matching the stored handle!",
                                    );
                                let new_pos = {
                                    let mut old_pos = sensor.position();
                                    Isometry2::new(old_pos.translation.vector, new_beam_aim)
                                };
                                sensor.set_position(new_pos);
                            }
                            None => (),
                        }
                    }
                    _ => println!("ERROR: Received `beam_aim` update for non-player entity!"),
                }
            }
            InternalUserDiffAction::BeamToggle(new_beam_on) => {
                let (uuid, entity) = self
                    .handle_map
                    .get_mut(collider_handle)
                    .expect("ERROR: No matching entry in `handle_map` for entry in `uuid_map`!");

                // Remove the existing beam sensor
                match *entity {
                    EntityType::Player {
                        beam_aim,
                        ref mut beam_on,
                        ..
                    } => {
                        *beam_on = new_beam_on;
                        if new_beam_on {
                            // Add a new sensor for the player's beam
                            if beam_handle.is_some() {
                                println!("WARN: Received message to turn beam on but we already have a sensor handle for it!");
                                return;
                            }

                            let new_sensor_handle = self.world.add_sensor(
                                BEAM_SHAPE_HANDLE.clone(),
                                *body_handle,
                                Isometry2::new(Vector2::zeros(), beam_aim),
                            );
                            *beam_handle = Some(new_sensor_handle);
                            self.beam_sensors.insert(new_sensor_handle, uuid.clone());
                        } else {
                            {
                                let beam_handle_inner = match beam_handle.as_mut() {
                                    Some(handle) => handle,
                                    None => {
                                        println!("WARN: Received message to turn beam off but it was already off");
                                        return;
                                    }
                                };
                                self.world.remove_colliders(&[*beam_handle_inner]);
                            }
                            *beam_handle = None;
                        }
                    }
                    _ => println!("ERROR: Received `beam_toggle` update for non-player entity!"),
                }
            }
            InternalUserDiffAction::Username(username) => {
                updates.push(Update::new_username(env, diff.id, username));
            }
        }
    }
}

pub struct PhysicsWorld(Mutex<PhysicsWorldInner>);

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld(Mutex::new(PhysicsWorldInner::new()))
    }

    pub fn apply<T, F: FnOnce(&mut PhysicsWorldInner) -> T>(&self, f: F) -> T {
        let mut inner = self.0.lock().unwrap();
        f(&mut inner)
    }
}

lazy_static! {
    /// The main world in which the entire simulation exists
    pub static ref WORLD: PhysicsWorld = PhysicsWorld::new();
}

impl<'a> UserDiff<'a> {
    pub fn parse(self, env: Env<'a>) -> NifResult<InternalUserDiff> {
        let internal_action = match self.action_type {
            t if atoms::direction() == t => {
                let movement = Movement::from_term(self.payload)?;
                InternalUserDiffAction::Movement(movement)
            }
            t if atoms::beam_rotation() == t => {
                let x: u32 = self.payload.map_get(atoms::x().encode(env))?.decode()?;
                let y: u32 = self.payload.map_get(atoms::y().encode(env))?.decode()?;
                InternalUserDiffAction::BeamAim {
                    x: x as f32,
                    y: y as f32,
                }
            }
            t if atoms::beam_toggle() == t => {
                let beam_on: bool = self.payload.decode()?;
                InternalUserDiffAction::BeamToggle(beam_on)
            }
            t if atoms::username() == t => {
                let username: String = self.payload.decode()?;
                InternalUserDiffAction::Username(username)
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
    BeamAim { x: f32, y: f32 },
    BeamToggle(bool),
    Username(String),
}

#[derive(Clone, Copy)]
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
            t if atoms::STOP() == t => Ok(Movement::Stop),
            _ => Err(NifError::Atom("invalid_movement_atom")),
        }
    }
}

impl Into<Atom> for Movement {
    fn into(self) -> Atom {
        match self {
            Movement::Up => atoms::UP(),
            Movement::UpRight => atoms::UP_RIGHT(),
            Movement::Right => atoms::RIGHT(),
            Movement::DownRight => atoms::DOWN_RIGHT(),
            Movement::Down => atoms::DOWN(),
            Movement::DownLeft => atoms::DOWN_LEFT(),
            Movement::Left => atoms::LEFT(),
            Movement::UpLeft => atoms::UP_LEFT(),
            Movement::Stop => atoms::STOP(),
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

        Force2::linear(direction_vector * CONF.physics.acceleration_per_tick)
    }
}

/// An update that is returned from the physics world as the result of a tick.  These should
/// generally map to updates of the outer state held by Elixir or updates sent directly to users.
#[derive(NifStruct)]
#[module = "NativePhysics.Update"]
pub struct Update<'a> {
    id: String,
    update_type: Atom,
    payload: Term<'a>,
}

impl<'a> Update<'a> {
    pub fn new_movement_update(env: Env<'a>, id: String, movement_update: MovementUpdate) -> Self {
        Update {
            id,
            update_type: atoms::isometry(),
            payload: movement_update.encode(env),
        }
    }

    pub fn new_beam_event(
        env: Env<'a>,
        user_id: String,
        target_entity_id: String,
        prev_status: Proximity,
        cur_status: Proximity,
    ) -> Self {
        Update {
            id: user_id,
            update_type: atoms::beam_event(),
            payload: BeamEvent::new(target_entity_id, prev_status, cur_status).encode(env),
        }
    }

    pub fn new_username(env: Env<'a>, id: String, username: String) -> Self {
        Update {
            id,
            update_type: atoms::username(),
            payload: username.encode(env),
        }
    }
}

#[derive(NifStruct)]
#[module = "NativePhysics.BeamEvent"]
pub struct BeamEvent {
    target_id: String,
    prev_status: Atom,
    cur_status: Atom,
}

fn proximity_to_atom(prox: Proximity) -> Atom {
    match prox {
        Proximity::Intersecting => atoms::intersecting(),
        Proximity::WithinMargin | Proximity::Disjoint => atoms::disjoint(),
    }
}

impl BeamEvent {
    pub fn new(target_entity_id: String, prev_status: Proximity, cur_status: Proximity) -> Self {
        BeamEvent {
            target_id: target_entity_id,
            prev_status: proximity_to_atom(prev_status),
            cur_status: proximity_to_atom(cur_status),
        }
    }
}

/// This is called by the Elixir code every tick of the game.  It will be provided an array of
/// updates to the game state which will be applied to the internal state that the physics
/// engine manages and return a set of messages that need to be sent to the user.
pub fn tick<'a>(env: Env<'a>, update_all: bool, diffs: Vec<InternalUserDiff>) -> Vec<Update> {
    WORLD.apply(move |world| {
        let mut updates = Vec::new();

        // Process all incoming diffs from Elixir
        for diff in diffs {
            world.apply_diff(env, diff, &mut updates)
        }

        let &mut PhysicsWorldInner {
            ref mut handle_map,
            ref mut world,
            ref mut user_handles,
            ref mut beam_sensors,
            ..
        } = world;

        // Apply friction and movement updates for all user entities
        for (user_body_handle, user_collider_handle) in user_handles {
            let user_rigid_body: &mut RigidBody<f32> = world
                .rigid_body_mut(*user_body_handle)
                .expect("ERROR: Player wasn't a rigid body!");

            // Apply thrust force from movement input
            let (_uuid, user_data) = handle_map
                .get(user_collider_handle)
                .expect("User collider handle isn't in the `handle_map`!");
            let movement: Movement = match user_data {
                EntityType::Player { movement, .. } => (*movement),
                _ => panic!("Expected a player entity but the entity data wasn't one!"),
            };
            let force: Force2<f32> = movement.into();
            user_rigid_body.apply_force(&force);

            // Apply friction
            let linear_velocity = user_rigid_body.velocity().linear;
            user_rigid_body
                .set_linear_velocity(linear_velocity * (1.0 - CONF.physics.friction_per_tick));
        }

        // Step the physics simulation
        world.step();

        let create_pos_update_inner = |collider: &CollisionObject<f32, ColliderData<f32>>,
                                       uuid: String,
                                       body_handle: BodyHandle|
         -> Update {
            let isometry = collider.position();
            let velocity = world
                .rigid_body(body_handle)
                .expect("Non-rigid body in `create_pos_update`")
                .velocity();
            let movement_update = MovementUpdate {
                pos_x: isometry.translation.vector.x,
                pos_y: isometry.translation.vector.y,
                rotation: isometry.rotation.angle(),
                velocity_x: velocity.linear.x,
                velocity_y: velocity.linear.y,
                angular_velocity: velocity.angular,
            };
            Update::new_movement_update(env, uuid, movement_update)
        };

        // Looks up the collider with the given handle, creates an `Update` with its position,
        // and pushes it into the update list.
        let create_pos_update =
            |collider_handle: ColliderHandle, body_handle: BodyHandle| -> Update {
                let collider = world.collider(collider_handle).unwrap();
                let (uuid, _entity_type) = handle_map
                    .get(&collider_handle)
                    .expect("`ColliderHandle` wasn't in the `handle_map`!");

                create_pos_update_inner(collider, uuid.clone(), body_handle)
            };

        for prox_evt in world.proximity_events() {
            // We don't care if the beam just got close to something
            if (prox_evt.prev_status != Proximity::WithinMargin
                && prox_evt.new_status != Proximity::WithinMargin)
                || prox_evt.prev_status != prox_evt.new_status
            {
                continue;
            }

            if let Some((user_id, Some((target_entity_id, _target_entity)))) = match (
                beam_sensors.get(&prox_evt.collider1),
                beam_sensors.get(&prox_evt.collider2),
            ) {
                (Some(_), Some(_)) => None, // Two beams colliding; ignore
                (None, None) => {
                    println!("WARN: proximity event between two non-sensors");
                    None
                }
                (Some(user_id), None) => Some((user_id, handle_map.get(&prox_evt.collider2))),
                (None, Some(user_id)) => Some((user_id, handle_map.get(&prox_evt.collider1))),
            } {
                // Create an update for the beam collision event and push it into the event list
                let update = Update::new_beam_event(
                    env,
                    user_id.clone(),
                    target_entity_id.clone(),
                    prox_evt.prev_status,
                    prox_evt.new_status,
                );
                updates.push(update);
            }
        }

        if update_all {
            // Create position updates for all managed entities
            for (collider_handle, (uuid, _entity_type)) in handle_map.iter() {
                let collider = world.collider(*collider_handle).unwrap();
                updates.push(create_pos_update_inner(
                    collider,
                    uuid.clone(),
                    world.collider_body_handle(*collider_handle).unwrap(),
                ));
            }
        } else {
            // Create position events for all entities that have just been involved in a collision
            for contact_evt in world.contact_events() {
                match contact_evt {
                    ContactEvent::Started(handle_1, handle_2)
                    | ContactEvent::Stopped(handle_1, handle_2) => {
                        updates.push(create_pos_update(
                            *handle_1,
                            world.collider_body_handle(*handle_1).unwrap(),
                        ));
                        updates.push(create_pos_update(
                            *handle_2,
                            world.collider_body_handle(*handle_2).unwrap(),
                        ));
                    }
                }
            }
        }

        updates
    })
}

/// Adds a new user into the world with a given UUID, returning the location at which it was
/// spawned in.  Returns `(center_of_mass_x, center_of_mass_y, MovementUpdate)`
pub fn spawn_user(uuid: String) -> (f32, f32, MovementUpdate) {
    let player_shape_handle = create_player_shape_handle(DEFAULT_PLAYER_SIZE);
    // TODO: decide where to spawn the user some better way
    let pos = Isometry2::new(Vector2::new(200.0, 200.0), 0.0);

    // `ShapeHandle` implements `AsRef<Shape>`, and `Shape` implements `Volumetric` which has the
    // `inertia()` and `center_of_mass()` functions.  Yeah.
    let inertia = player_shape_handle.inertia(1.0);
    let center_of_mass = player_shape_handle.center_of_mass();

    let com = WORLD.apply(move |world| {
        let &mut PhysicsWorldInner {
            ref mut uuid_map,
            ref mut handle_map,
            ref mut world,
            ref mut user_handles,
            ref mut beam_sensors,
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

        // Create a `Sensor` for the player's beam
        let beam_handle = world.add_sensor(
            BEAM_SHAPE_HANDLE.clone(),
            body_handle,
            Isometry2::identity(),
        );

        // Insert an entry into the UUID map for the created player's internal handles
        let handles = EntityHandles {
            collider_handle,
            body_handle,
            beam_handle: Some(beam_handle),
        };
        uuid_map.insert(uuid.clone(), handles);
        // Also insert an entry into the reverse lookup map
        handle_map.insert(
            collider_handle,
            (
                uuid.clone(),
                EntityType::Player {
                    size: DEFAULT_PLAYER_SIZE as u32,
                    movement: Movement::Stop,
                    beam_aim: 0.0,
                    beam_on: false,
                },
            ),
        );
        beam_sensors.insert(beam_handle, uuid);
        // Add the handle to the `user_handles` cache
        user_handles.push((body_handle, collider_handle));

        world.rigid_body(body_handle).unwrap().center_of_mass()
    });

    let mvmt_update = MovementUpdate {
        pos_x: 200.0,
        pos_y: 200.0,
        rotation: 0.0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        angular_velocity: 0.0,
    };

    (com.x, com.y, mvmt_update)
}

#[derive(NifStruct)]
#[module = "NativePhysics.MovementUpdate"]
pub struct MovementUpdate {
    pub pos_x: f32,
    pub pos_y: f32,
    pub rotation: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub angular_velocity: f32,
}

#[derive(NifStruct)]
#[module = "NativePhysics.EntityData"]
pub struct EntityData<'a> {
    pub id: String,
    pub center_of_mass_x: f32,
    pub center_of_mass_y: f32,
    pub movement: MovementUpdate,
    pub entity_type: Atom,
    pub entity_meta: Term<'a>,
}

pub fn get_snapshot<'a>(env: Env<'a>, _args: &[Term<'a>]) -> NifResult<Term<'a>> {
    WORLD.apply(|world| -> NifResult<Term<'a>> {
        let mut acc = Term::map_new(env);

        for (handle, (uuid, entity_type)) in &world.handle_map {
            let collider = world
                .world
                .collider(*handle)
                .expect("No collider with a handle stored in `handle_map` found!");
            let isometry: &Isometry2<f32> = collider.position();
            let (entity_name, data) = entity_type.to_data(env)?;

            let body_handle: BodyHandle = collider.data().body();
            let body = world.world.body(body_handle);
            let (velocity, center_of_mass) = match body {
                Body::RigidBody(rigid_body) => (rigid_body.velocity(), rigid_body.center_of_mass()),
                Body::Multibody(_) => unimplemented!(),
                Body::Ground(_) => unimplemented!(),
            };

            let data = EntityData {
                id: uuid.clone(),
                center_of_mass_x: center_of_mass.x,
                center_of_mass_y: center_of_mass.y,
                movement: MovementUpdate {
                    pos_x: isometry.translation.vector.x,
                    pos_y: isometry.translation.vector.y,
                    rotation: isometry.rotation.angle(),
                    velocity_x: (*velocity).linear.x,
                    velocity_y: (*velocity).linear.y,
                    angular_velocity: velocity.angular,
                },
                entity_type: entity_name,
                entity_meta: data,
            };

            acc = acc.map_put(uuid.encode(env), data.encode(env))?;
        }

        Ok(acc)
    })
}

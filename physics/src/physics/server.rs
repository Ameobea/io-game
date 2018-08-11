use std::sync::Mutex;

use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::events::ContactEvent;
use ncollide2d::query::Proximity;
use ncollide2d::world::CollisionObject;
use nphysics2d::object::{Body, BodyHandle, ColliderData, ColliderHandle, Material};
use nphysics2d::volumetric::Volumetric;
use rustler::error::Error as NifError;
use rustler::{types::atom::Atom, Encoder, Env, NifResult, Term};

use super::super::atoms;
use super::entities::{
    create_player_shape_handle, Entity, EntityHandles, PlayerEntity, BEAM_SHAPE_HANDLE,
    DEFAULT_PLAYER_SIZE,
};
use super::world::{PhysicsWorldInner, COLLIDER_MARGIN};
use super::Movement;

pub struct PhysicsWorld(Mutex<PhysicsWorldInner>);

impl PhysicsWorld {
    pub fn new() -> Self {
        let mut world = PhysicsWorldInner::new();
        // Spawn initial entities into the world
        world.initialize();

        PhysicsWorld(Mutex::new(world))
    }

    pub fn apply<T, F: FnOnce(&mut PhysicsWorldInner) -> T>(&self, f: F) -> T {
        let mut inner = self.0.lock().unwrap();
        f(&mut inner)
    }

    pub fn apply_diff<'a>(
        &self,
        env: Env<'a>,
        diff: InternalUserDiff,
        updates: &mut Vec<Update<'a>>,
    ) {
        let PhysicsWorldInner {
            ref mut uuid_map,
            ref mut world,
            ref mut beam_sensors,
            ..
        } = &mut *self.0.lock().unwrap();

        let uuid = diff.id;

        let EntityHandles {
            body_handle,
            ref mut beam_handle,
            entity,
            ..
        } = match uuid_map.get_mut(&uuid) {
            Some(handle) => handle,
            None => {
                println!(
                    "ERROR: Received update for user with id {} but no such user exists!",
                    uuid
                );
                return;
            }
        };

        let expected_player =
            || println!("ERROR: Received `beam_aim` update for non-player entity!");

        match diff.action {
            InternalUserDiffAction::Movement(new_movement) => match *entity {
                Entity::Player(PlayerEntity {
                    ref mut movement, ..
                }) => *movement = new_movement,
                _ => expected_player(),
            },
            InternalUserDiffAction::BeamAim { x, y } => {
                match *entity {
                    Entity::Player(PlayerEntity {
                        ref mut beam_aim, ..
                    }) => {
                        // Calculate the angle in radians produced by looking at (x, y) from the
                        // player's position
                        let new_beam_aim = Point2::new(x, y);
                        let rotation = (y / x).atan();

                        *beam_aim = new_beam_aim;
                        match beam_handle {
                            Some(beam_handle) => {
                                // Move the beam sensor
                                let sensor = world
                                    .collision_world_mut()
                                    .collision_object_mut(*beam_handle)
                                    .expect(
                                        "No beam sensor in the world matching the stored handle!",
                                    );
                                let new_pos = {
                                    let old_pos = sensor.position();
                                    Isometry2::new(old_pos.translation.vector, rotation)
                                };
                                sensor.set_position(new_pos);
                            }
                            None => (),
                        }
                    }
                    _ => expected_player(),
                }
            }
            InternalUserDiffAction::BeamToggle(new_beam_on) => {
                // Remove the existing beam sensor
                match *entity {
                    Entity::Player(PlayerEntity {
                        beam_aim,
                        ref mut beam_on,
                        ..
                    }) => {
                        *beam_on = new_beam_on;
                        if new_beam_on {
                            // Add a new sensor for the player's beam
                            if beam_handle.is_some() {
                                println!("WARN: Received message to turn beam on but we already have a sensor handle for it!");
                                return;
                            }

                            let new_sensor_handle = world.add_sensor(
                                BEAM_SHAPE_HANDLE.clone(),
                                *body_handle,
                                Isometry2::new(Vector2::zeros(), (beam_aim.y / beam_aim.x).atan()),
                            );
                            *beam_handle = Some(new_sensor_handle);
                            beam_sensors.insert(new_sensor_handle, uuid.clone());
                        } else {
                            {
                                let beam_handle_inner = match beam_handle.as_mut() {
                                    Some(handle) => handle,
                                    None => {
                                        println!("WARN: Received message to turn beam off but it was already off");
                                        return;
                                    }
                                };
                                world.remove_colliders(&[*beam_handle_inner]);
                            }
                            *beam_handle = None;
                        }
                    }
                    _ => println!("ERROR: Received `beam_toggle` update for non-player entity!"),
                }
            }
            InternalUserDiffAction::Username(username) => {
                updates.push(Update::new_username(env, uuid, username));
            }
        }
    }
}

pub struct InternalUserDiff {
    pub id: String,
    pub action: InternalUserDiffAction,
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

/// An update that is returned from the physics world as the result of a tick.  These should
/// generally map to updates of the outer state held by Elixir or updates sent directly to users.
#[derive(Debug, NifStruct)]
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

lazy_static! {
    /// The main world in which the entire simulation exists
    pub static ref WORLD: PhysicsWorld = PhysicsWorld::new();
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
    let mut updates = Vec::new();

    // Process all incoming diffs from Elixir
    for diff in diffs {
        WORLD.apply_diff(env, diff, &mut updates)
    }

    WORLD.apply(move |world| {
        // Apply friction and movement updates for all user entities
        world.step();

        let &mut PhysicsWorldInner {
            ref mut handle_map,
            ref mut world,
            ref mut beam_sensors,
            ..
        } = world;

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
                let uuid = handle_map
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

            if let Some((user_id, Some(target_entity_id))) = match (
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
            for (collider_handle, uuid) in handle_map.iter() {
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
            entity: Entity::Player(PlayerEntity::default()),
            data: (),
        };
        uuid_map.insert(uuid.clone(), handles);
        // Also insert an entry into the reverse lookup map
        handle_map.insert(collider_handle, uuid.clone());
        beam_sensors.insert(beam_handle, uuid.clone());
        // Add the handle to the `user_handles` cache
        user_handles.push((body_handle, uuid));

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
#[module = "NativePhysics.EntityData"]
pub struct EntityData<'a> {
    pub id: String,
    pub center_of_mass_x: f32,
    pub center_of_mass_y: f32,
    pub movement: MovementUpdate,
    pub entity_type: Atom,
    pub entity_meta: Term<'a>,
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

pub fn get_snapshot<'a>(env: Env<'a>, _args: &[Term<'a>]) -> NifResult<Term<'a>> {
    WORLD.apply(|world| -> NifResult<Term<'a>> {
        let mut acc = Term::map_new(env);

        for (
            uuid,
            EntityHandles {
                collider_handle,
                entity,
                ..
            },
        ) in &world.uuid_map
        {
            let collider = world
                .world
                .collider(*collider_handle)
                .expect("No collider with a handle stored in `handle_map` found!");
            let isometry: &Isometry2<f32> = collider.position();
            let (entity_name, data) = entity.to_data(env)?;

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

use std::sync::Mutex;

use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::events::ContactEvent;
use ncollide2d::query::Proximity;
use nphysics2d::algebra::Velocity2;
use nphysics2d::object::{Body, BodyHandle, BodyStatus, ColliderHandle};
use rustler::error::Error as NifError;
use rustler::{types::atom::Atom, Encoder, Env, NifResult, Term};

use super::super::atoms;
use super::entities::{Entity, EntityHandles, EntitySpawn, PlayerEntity, BEAM_SHAPE_HANDLE};
use super::world::PhysicsWorldInner;
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

        let expected_player = || println!("ERROR: Received invalid update for non-player entity!");

        match diff.action {
            InternalUserDiffAction::Movement(new_movement) => match *entity {
                Entity::Player(PlayerEntity {
                    ref mut movement, ..
                }) => {
                    *movement = new_movement;
                    updates.push(Update::new_player_movement(env, uuid, new_movement))
                }
                _ => expected_player(),
            },
            InternalUserDiffAction::BeamAim { x, y } => {
                let PlayerEntity { beam_aim, .. } = match *entity {
                    Entity::Player(ref mut player) => player,
                    _ => {
                        expected_player();
                        return;
                    }
                };

                // Calculate the angle in radians produced by looking at (x, y) from the
                // player's position
                let new_beam_aim = Point2::new(x, y);
                let rotation = (y / x).atan();

                *beam_aim = new_beam_aim;
                match beam_handle {
                    Some(beam_handle) => {
                        // Move the beam sensor
                        let beam_collider = world
                            .collider(*beam_handle)
                            .expect("No beam sensor in the world matching the stored handle!");
                        let old_pos = beam_collider.position();
                        let pos_wrt_body = *beam_collider.data().position_wrt_body();
                        let new_pos =
                            pos_wrt_body * Isometry2::new(old_pos.translation.vector, rotation);

                        world
                            .collision_world_mut()
                            .set_position(*beam_handle, new_pos);
                    }
                    None => (),
                }

                updates.push(Update::new_beam_aim(env, uuid, Point2::new(x, y)));
            }

            InternalUserDiffAction::BeamToggle(new_beam_on) => {
                // Remove the existing beam sensor
                let PlayerEntity {
                    beam_aim,
                    ref mut beam_on,
                    ..
                } = match *entity {
                    Entity::Player(ref mut player) => player,
                    _ => {
                        expected_player();
                        return;
                    }
                };

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

                updates.push(Update::new_beam_toggle(env, uuid, new_beam_on));
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

    pub fn new_player_movement(env: Env<'a>, player_id: String, movement: Movement) -> Self {
        let movement_atom: Atom = movement.into();

        Update {
            id: player_id,
            update_type: atoms::player_movement(),
            payload: movement_atom.encode(env),
        }
    }

    pub fn new_beam_toggle(env: Env<'a>, player_id: String, beam_on: bool) -> Self {
        Update {
            id: player_id,
            update_type: atoms::beam_toggle(),
            payload: beam_on.encode(env),
        }
    }

    pub fn new_beam_aim(env: Env<'a>, player_id: String, beam_aim: Point2<f32>) -> Self {
        let map = Term::map_new(env);
        const ERR_MSG: &'static str = "Error while building map in `new_beam_aim`!";
        let map = map
            .map_put(atoms::x().encode(env), beam_aim.x.encode(env))
            .map_err(|_| ERR_MSG)
            .unwrap();
        let map = map
            .map_put(atoms::y().encode(env), beam_aim.y.encode(env))
            .map_err(|_| ERR_MSG)
            .unwrap();

        Update {
            id: player_id,
            update_type: atoms::beam_aim(),
            payload: map,
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

        let create_pos_update_inner =
            |pos: &Isometry2<f32>, uuid: String, body_handle: BodyHandle| -> Option<Update> {
                let velocity = world.rigid_body(body_handle)?.velocity();
                let movement_update = MovementUpdate {
                    pos_x: pos.translation.vector.x,
                    pos_y: pos.translation.vector.y,
                    rotation: pos.rotation.angle(),
                    velocity_x: velocity.linear.x,
                    velocity_y: velocity.linear.y,
                    angular_velocity: velocity.angular,
                };

                Some(Update::new_movement_update(env, uuid, movement_update))
            };

        // Looks up the collider with the given handle, creates an `Update` with its position,
        // and pushes it into the update list.
        let create_pos_update =
            |collider_handle: ColliderHandle, body_handle: BodyHandle| -> Option<Update> {
                let collider = world.collider(collider_handle).unwrap();
                let uuid = handle_map
                    .get(&collider_handle)
                    .expect("`ColliderHandle` wasn't in the `handle_map`!");

                create_pos_update_inner(collider.position(), uuid.clone(), body_handle)
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
                let update_opt = create_pos_update_inner(
                    collider.position(),
                    uuid.clone(),
                    world.collider_body_handle(*collider_handle).unwrap(),
                );
                if let Some(update) = update_opt {
                    updates.push(update);
                }
            }
        } else {
            // Create position events for all entities that have just been involved in a collision
            for contact_evt in world.contact_events() {
                match contact_evt {
                    ContactEvent::Started(handle_1, handle_2)
                    | ContactEvent::Stopped(handle_1, handle_2) => {
                        for handle in &[handle_1, handle_2] {
                            let update_opt = create_pos_update(
                                **handle,
                                world.collider_body_handle(**handle).unwrap(),
                            );
                            if let Some(update) = update_opt {
                                updates.push(update);
                            }
                        }
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
    // TODO: decide where to spawn the user some better way
    let pos = Isometry2::new(Vector2::new(200.0, 200.0), 0.0);

    let entity_spawn = EntitySpawn {
        entity: Entity::Player(PlayerEntity::default()),
        isometry: pos,
        velocity: Velocity2::zero(),
        data: (),
        body_status: BodyStatus::Dynamic,
    };

    let com = WORLD.apply(move |world| {
        world.spawn_entity(
            uuid.parse().expect("Invalid player UUID provided!"),
            entity_spawn,
        );
        let body_handle = world.uuid_map.get(&uuid).unwrap().body_handle;
        world
            .world
            .rigid_body(body_handle)
            .unwrap()
            .center_of_mass()
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

pub fn despawn_user(uuid: String) {
    WORLD.apply(|world: &mut PhysicsWorldInner| world.remove_entity(&uuid))
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
                Body::RigidBody(rigid_body) => {
                    (rigid_body.velocity().clone(), rigid_body.center_of_mass())
                }
                Body::Multibody(_) => unimplemented!(),
                Body::Ground(_) => (Velocity2::new(Vector2::zeros(), 0.0), Point2::origin()),
            };

            let data = EntityData {
                id: uuid.clone(),
                center_of_mass_x: center_of_mass.x,
                center_of_mass_y: center_of_mass.y,
                movement: MovementUpdate {
                    pos_x: isometry.translation.vector.x,
                    pos_y: isometry.translation.vector.y,
                    rotation: isometry.rotation.angle(),
                    velocity_x: (velocity).linear.x,
                    velocity_y: (velocity).linear.y,
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

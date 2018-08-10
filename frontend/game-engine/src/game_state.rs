//! Manages the state for the game and exposes methods for interacting and observing it.

use std::collections::BTreeMap;
use std::mem;
use std::ptr;

use nalgebra::{Isometry2, Point2, Vector2};
use native_physics::physics::world::{EntityHandles, PhysicsWorldInner as PhysicsWorld};
use nphysics2d::algebra::Velocity2;
use uuid::Uuid;

use entity::Entity;
use game::entities::asteroid::Asteroid;
use game::PlayerEntity;
use proto_utils::{parse_server_msg_payload, InnerServerMessage, ServerMessageContent};
use protos::server_messages::{
    CreationEvent, CreationEvent_oneof_entity as EntityType, MovementUpdate, ServerMessage,
    Snapshot, StatusUpdate, StatusUpdate_SimpleEvent as SimpleEvent,
    StatusUpdate_oneof_payload as StatusPayload,
};
use render_effects::RenderEffectManager;
use user_input::CurHeldKeys;
use util::{error, log, warn};

pub static mut STATE: *mut GameState = ptr::null_mut();
pub static mut EFFECTS_MANAGER: *mut RenderEffectManager = ptr::null_mut();
pub static mut CUR_HELD_KEYS: *mut CurHeldKeys = ptr::null_mut();

#[inline(always)]
pub fn get_state() -> &'static mut GameState {
    unsafe { mem::transmute(STATE) }
}

#[inline(always)]
pub fn get_effects_manager() -> &'static mut RenderEffectManager {
    unsafe { mem::transmute(EFFECTS_MANAGER) }
}

#[inline(always)]
pub fn get_cur_held_keys() -> &'static mut CurHeldKeys {
    unsafe { mem::transmute(CUR_HELD_KEYS) }
}

static mut PLAYER_ENTITY_FASTPATH: *mut PlayerEntity = ptr::null_mut();

pub fn player_entity_fastpath() -> &'static mut PlayerEntity {
    unsafe { &mut *PLAYER_ENTITY_FASTPATH as &mut PlayerEntity }
}

// /// BVT Visitor that returns the UUIDs of all entities that a given BV may be colliding with.
// struct CollisionVisitor<'a> {
//     bv: &'a AABB<f32>,
//     acc: &'a mut Vec<Uuid>,
// }

// impl<'a> BVTVisitor<Uuid, AABB<f32>> for CollisionVisitor<'a> {
//     fn visit_internal(&mut self, bv: &AABB<f32>) -> bool {
//         bv.intersects(self.bv)
//     }

//     fn visit_leaf(&mut self, entity_id: &Uuid, bv: &AABB<f32>) {
//         if bv.intersects(self.bv) {
//             self.acc.push(*entity_id);
//         }
//     }
// }

pub struct GameState {
    pub cur_tick: usize,
    pub player_uuid: Uuid,
    // TODO: Merge this into the inner `PhysicsWorld` via custom data
    pub entity_map: BTreeMap<Uuid, Box<dyn Entity>>,
    pub world: PhysicsWorld,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            cur_tick: 0,
            player_uuid: Uuid::nil(), // Placeholder until we are assigned an ID by the server
            entity_map: BTreeMap::new(),
            world: PhysicsWorld::new(),
        }
    }

    pub fn apply_msg(&mut self, msg: ServerMessage) {
        let tick = msg.get_tick();
        let timestamp = msg.get_timestamp();

        for InnerServerMessage { id, content } in parse_server_msg_payload(msg) {
            self.apply_inner_msg(id, content, tick, timestamp)
        }
    }

    fn apply_inner_msg(
        &mut self,
        entity_id: Uuid,
        update: ServerMessageContent,
        tick: u32,
        timestamp: u64,
    ) {
        let PhysicsWorld {
            ref mut uuid_map,
            ref mut handle_map,
            ..
        } = self.world;

        // TODO: handle tick and timestamp; check for skipped messages and request re-sync etc.
        match update {
            ServerMessageContent::status_update(StatusUpdate { payload, .. }) => match payload {
                Some(StatusPayload::creation_event(creation_evt)) => {
                    self.create_entity(entity_id, &creation_evt)
                }
                Some(StatusPayload::other(SimpleEvent::DELETION)) => {
                    // Remove the entity from the UUID map as well as the underlying `PhysicsWorld`
                    match self.entity_map.remove(&entity_id) {
                        Some(_) => (),
                        None => {
                            error(format!("Attempted to remove entity with id {} from world, but it doesn't exist in `entity_map`", entity_id));
                            return;
                        }
                    }
                    self.world.remove_entity(&entity_id);
                }
                None => warn("Received `StatusUpdate` with no payload"),
            },
            ServerMessageContent::snapshot(snapshot) => self.apply_snapshot(snapshot),
            ServerMessageContent::connect_successful(player_id) => {
                self.init_player_fastpath(player_id.into())
            }
            _ => {
                let entity = match self.entity_map.get_mut(&entity_id) {
                    Some(key) => key,
                    None => {
                        error(format!(
                            "Received update for entity {} which doesn't exist",
                            entity_id
                        ));
                        return;
                    }
                };

                if let ServerMessageContent::movement_update(ref movement_update) = update {
                    let (pos, velocity) = movement_update.into();
                    // Update the entity's position and velocity on the underlying `PhysicsWorld`
                    self.world.update_movement(&entity_id, &pos, &velocity);
                    return;
                }

                entity.apply_update(&update);
            }
        }
    }

    /// Removes all items from the UUID map and the DBVT, then reconstruct the game state from the
    /// contents of the snapshot
    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        log("Clearing game state and applying snapshot...");
        // Too bad we have to do it like this.  TODO: Add a `clear()` method to `DBVT` via PR?
        for (uuid, _) in self.entity_map.iter() {
            if *uuid == self.player_uuid {
                continue;
            }

            self.world.remove_entity(uuid);
        }
        // Clear the UUID map, but keep the player in it
        let player_entry = self.entity_map.remove(&self.player_uuid).unwrap();
        self.entity_map.clear();
        self.entity_map.insert(self.player_uuid, player_entry);

        for mut snapshot_item in snapshot.items.into_iter() {
            log("Applying snapshot item...");
            let uuid: Uuid = snapshot_item.take_id().into();
            let creation_evt = snapshot_item.get_item();
            self.create_entity(uuid, creation_evt);
        }
    }

    /// Renders all entities in random order.  Some entities take a default action every game tick
    /// without taking input from the server.  This method iterates over all entities and
    /// optionally performs this mutation before rendering.  Returns the current tick.
    pub fn tick(&mut self) -> usize {
        for (entity_id, entity) in &mut self.entity_map {
            entity.tick(self.cur_tick);
            // TODO: figure out how to do this so that the entity can get position info
            entity.render(self.cur_tick);
        }

        self.cur_tick += 1;
        self.cur_tick
    }

    pub fn create_entity(&mut self, entity_id: Uuid, creation_evt: &CreationEvent) {
        let entity = match creation_evt.entity.as_ref() {
            Some(entity) => entity,
            None => {
                error("Received `CreationEvent` without an `enity` field");
                return;
            }
        };
        let movement = creation_evt.get_movement();
        let center_of_mass = Point2::new(
            creation_evt.get_center_of_mass_x(),
            creation_evt.get_center_of_mass_y(),
        );

        let boxed_entity: Box<dyn Entity> = match entity {
            EntityType::player(proto_player) => {
                let (pos, velocity) = movement.into();

                if entity_id == self.player_uuid {
                    // If this is the spawn event for our own entity, just update it in-place so that
                    // we don't have to mess with the static fastpath
                    let player = player_entity_fastpath();

                    self.world.update_movement(&entity_id, &pos, &velocity);
                    player.center_of_mass = center_of_mass;
                    player.size = proto_player.size as u16;
                    return;
                } else {
                    box PlayerEntity::new(pos, center_of_mass, proto_player.get_size() as u16)
                }
            }
            EntityType::asteroid(asteroid) => {
                box Asteroid::from_proto(asteroid, center_of_mass, movement)
            }
        };

        // TODO
        self.world.spawn_entity(entity_id, unimplemented!());
    }

    pub fn init_player_fastpath(&mut self, player_id: Uuid) {
        self.player_uuid = player_id;
        let player_entity =
            box PlayerEntity::new(Isometry2::new(Vector2::zeros(), 0.0), Point2::origin(), 20);
        let player_entity_ptr = Box::into_raw(player_entity);
        unsafe { PLAYER_ENTITY_FASTPATH = player_entity_ptr };
        let player_entity = unsafe { Box::from_raw(player_entity_ptr) };
        let player_entity = player_entity as Box<dyn Entity>;
        self.entity_map.insert(player_id, player_entity);
    }
}

//! Manages the state for the game and exposes methods for interacting and observing it.

use std::collections::BTreeMap;
use std::mem;
use std::ptr;

use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, BoundingVolume};
use ncollide2d::partitioning::{BVTVisitor, DBVTLeaf, DBVTLeafId, DBVT};
use uuid::Uuid;

use entity::Entity;
use game::entities::asteroid::Asteroid;
use game::PlayerEntity;
use proto_utils::{parse_server_msg_payload, InnerServerMessage, ServerMessageContent};
use protos::server_messages::{
    CreationEvent, CreationEvent_oneof_entity as EntityType, ServerMessage, Snapshot, StatusUpdate,
    StatusUpdate_SimpleEvent as SimpleEvent, StatusUpdate_oneof_payload as StatusPayload,
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

/// BVT Visitor that returns the UUIDs of all entities that a given BV may be colliding with.
struct CollisionVisitor<'a> {
    bv: &'a AABB<f32>,
    acc: &'a mut Vec<Uuid>,
}

impl<'a> BVTVisitor<Uuid, AABB<f32>> for CollisionVisitor<'a> {
    fn visit_internal(&mut self, bv: &AABB<f32>) -> bool {
        bv.intersects(self.bv)
    }

    fn visit_leaf(&mut self, entity_id: &Uuid, bv: &AABB<f32>) {
        if bv.intersects(self.bv) {
            self.acc.push(*entity_id);
        }
    }
}

pub struct GameState {
    cur_tick: usize,
    pub entity_map: DBVT<f32, Uuid, AABB<f32>>,
    pub uuid_map: BTreeMap<Uuid, (DBVTLeafId, Box<dyn Entity>)>,
    player_uuid: Uuid,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            cur_tick: 0,
            entity_map: DBVT::new(),
            uuid_map: BTreeMap::new(),
            player_uuid: Uuid::nil(),
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
        // TODO: handle tick and timestamp; check for skipepd messages and request re-sync etc.
        match update {
            ServerMessageContent::status_update(StatusUpdate { payload, .. }) => match payload {
                Some(StatusPayload::creation_event(creation_evt)) => {
                    self.create_entity(entity_id, &creation_evt)
                }
                Some(StatusPayload::other(SimpleEvent::DELETION)) => {
                    // Remove the entity from the UUID map as well as the DBVT
                    let (leaf_id, _) = match self.uuid_map.remove(&entity_id) {
                        Some(entry) => entry,
                        None => {
                            error(format!(
                                "Attempted to delete entity with id {} but it doesn't exist",
                                entity_id
                            ));
                            return;
                        }
                    };

                    self.entity_map.remove(leaf_id);
                }
                None => warn("Received `StatusUpdate` with no payload"),
            },
            ServerMessageContent::snapshot(snapshot) => self.apply_snapshot(snapshot),
            ServerMessageContent::connect_successful(player_id) => {
                self.init_player_fastpath(player_id.into())
            }
            _ => {
                let (leaf_id, entity) = match self.uuid_map.get_mut(&entity_id) {
                    Some(key) => key,
                    None => {
                        error(format!(
                            "Received update for entity {} which doesn't exist",
                            entity_id
                        ));
                        return;
                    }
                };

                if let ServerMessageContent::movement_update(movement) = update {
                    entity.set_movement(&movement);
                    return;
                }

                if entity.apply_update(&update) {
                    self.entity_map.remove(*leaf_id);
                    let new_bv = entity.get_bounding_volume();
                    let leaf = DBVTLeaf::new(new_bv, entity_id);
                    let new_leaf_id = self.entity_map.insert(leaf);
                    *leaf_id = new_leaf_id;
                }
            }
        }
    }

    /// Removes all items from the UUID map and the DBVT, then reconstruct the game state from the
    /// contents of the snapshot
    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        log("Clearing game state and applying snapshot...");
        // Too bad we have to do it like this.  TODO: Add a `clear()` method to `DBVT` via PR?
        for (uuid, (leaf_id, _entity)) in self.uuid_map.iter() {
            if *uuid == self.player_uuid {
                continue;
            }

            self.entity_map.remove(*leaf_id);
        }
        // Clear the UUID map, but keep the player in it
        let player_entry = self.uuid_map.remove(&self.player_uuid).unwrap();
        self.uuid_map.clear();
        self.uuid_map.insert(self.player_uuid, player_entry);

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
        for (entity_id, (leaf_id, entity)) in &mut self.uuid_map {
            if entity.tick(self.cur_tick) {
                // Remove it from the collision tree and re-insert it with a new `BoundingVolume`.
                self.entity_map.remove(*leaf_id);
                let new_bv = entity.get_bounding_volume();
                let leaf = DBVTLeaf::new(new_bv, *entity_id);
                let new_leaf_id = self.entity_map.insert(leaf);
                *leaf_id = new_leaf_id;
            }
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
            EntityType::player(proto_player) => if entity_id == self.player_uuid {
                // If this is the spawn event for our own entity, just update it in-place so that
                // we don't have to mess with the static fastpath
                let player = player_entity_fastpath();
                player.set_movement(movement);
                player.center_of_mass = center_of_mass;
                player.size = proto_player.size as u16;
                return;
            } else {
                let pos = Isometry2::new(
                    Vector2::new(movement.get_pos_x(), movement.get_pos_y()),
                    movement.get_angular_velocity(),
                );

                box PlayerEntity::new(pos, center_of_mass, proto_player.get_size() as u16)
            },
            EntityType::asteroid(asteroid) => {
                box Asteroid::from_proto(asteroid, center_of_mass, movement)
            }
        };

        let leaf = DBVTLeaf::new(boxed_entity.get_bounding_volume(), entity_id);
        let leaf_id = self.entity_map.insert(leaf);
        log(format!("Spawning entity {}", entity_id));
        self.uuid_map.insert(entity_id, (leaf_id, boxed_entity));
    }

    /// Returns a list of entity IDs that may be colliding with the given BV.
    pub fn broad_phase(&self, test_bv: &AABB<f32>) -> Vec<Uuid> {
        let mut found_uuids = Vec::new();
        let mut visitor = CollisionVisitor {
            bv: test_bv,
            acc: &mut found_uuids,
        };
        self.entity_map.visit(&mut visitor);

        found_uuids
    }

    pub fn init_player_fastpath(&mut self, player_id: Uuid) {
        self.player_uuid = player_id;
        let player_entity =
            box PlayerEntity::new(Isometry2::new(Vector2::zeros(), 0.0), Point2::origin(), 20);
        let player_entity_ptr = Box::into_raw(player_entity);
        unsafe { PLAYER_ENTITY_FASTPATH = player_entity_ptr };
        let player_entity = unsafe { Box::from_raw(player_entity_ptr) };
        let player_entity = player_entity as Box<dyn Entity>;
        let leaf = DBVTLeaf::new(player_entity.get_bounding_volume(), player_id);
        let leaf_id = self.entity_map.insert(leaf);
        self.uuid_map.insert(player_id, (leaf_id, player_entity));
    }
}

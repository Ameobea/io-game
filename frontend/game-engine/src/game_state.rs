//! Manages the state for the game and exposes methods for interacting and observing it.

use std::collections::BTreeMap;
use std::mem;
use std::ptr;

use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, BoundingVolume};
use ncollide2d::partitioning::{BVTVisitor, DBVTLeaf, DBVTLeafId, DBVT};
use uuid::Uuid;

use entity::Entity;
use game::entities::asteroid::Asteroid;
use game::PlayerEntity;
use proto_utils::ServerMessageContent;
use protos::server_messages::{
    CreationEvent, CreationEvent_oneof_entity as EntityType, StatusUpdate,
    StatusUpdate_SimpleEvent as SimpleEvent, StatusUpdate_oneof_payload as StatusPayload,
};
use render_effects::RenderEffectManager;
use user_input::CurHeldKeys;
use util::{error, warn};

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
}

impl GameState {
    pub fn new(user_id: Uuid) -> Self {
        let mut entity_map = DBVT::new();
        // set up the player entity fast path
        let player_entity = box PlayerEntity::new(Point2::origin(), 20);
        let player_entity_ptr = Box::into_raw(player_entity);
        unsafe { PLAYER_ENTITY_FASTPATH = player_entity_ptr };
        let player_entity = unsafe { Box::from_raw(player_entity_ptr) };
        let player_entity = player_entity as Box<dyn Entity>;
        let leaf = DBVTLeaf::new(player_entity.get_bounding_volume(), user_id);
        let leaf_id = entity_map.insert(leaf);
        let mut uuid_map = BTreeMap::new();
        uuid_map.insert(user_id, (leaf_id, player_entity));

        GameState {
            cur_tick: 0,
            entity_map,
            uuid_map,
        }
    }

    pub fn apply_msg(&mut self, entity_id: Uuid, update: ServerMessageContent) {
        match update {
            ServerMessageContent::status_update(StatusUpdate { payload, .. }) => match payload {
                Some(StatusPayload::creation_event(CreationEvent {
                    pos_x,
                    pos_y,
                    entity,
                    ..
                })) => if let Some(entity) = entity {
                    self.create_entity(&entity, entity_id, Vector2::new(pos_x, pos_y))
                } else {
                    warn("Received entity creation update with no inner entity payload")
                },
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

    pub fn create_entity(
        &mut self,
        entity: &EntityType,
        entity_id: Uuid,
        translation: Vector2<f32>,
    ) {
        let boxed_entity: Box<dyn Entity> = match entity {
            EntityType::player(player) => box PlayerEntity::new(
                Point2::new(translation.x, translation.y),
                player.get_size() as u16,
            ),
            EntityType::asteroid(asteroid) => box Asteroid::from_proto(asteroid, translation),
        };

        let leaf = DBVTLeaf::new(boxed_entity.get_bounding_volume(), entity_id);
        let leaf_id = self.entity_map.insert(leaf);
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
}

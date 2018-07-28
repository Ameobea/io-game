//! Manages the state for the game and exposes methods for interacting and observing it.

use std::collections::BTreeMap;
use std::mem;
use std::ptr;

use ncollide2d::bounding_volume::aabb::AABB;
use ncollide2d::partitioning::{BVTVisitor, DBVTLeaf, DBVTLeafId, DBVT};
use uuid::Uuid;

use entity::Entity;
use game::PlayerEntity;
use proto_utils::ServerMessageContent;
use protos::server_messages::{
    CreationEvent, CreationEvent_oneof_entity as EntityType, StatusUpdate,
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

struct FullVisitor {
    pub tick: usize,
}

impl BVTVisitor<Box<dyn Entity>, AABB<f32>> for FullVisitor {
    fn visit_internal(&mut self, _bv: &AABB<f32>) -> bool {
        true
    }

    #[allow(mutable_transmutes)]
    fn visit_leaf(&mut self, entity: &Box<dyn Entity>, _bv: &AABB<f32>) {
        // No reason this shouldn't be mutable... I'm just changing the entity behind the
        // pointer after all.
        let entity: &mut Box<dyn Entity> = unsafe { mem::transmute(entity) };
        entity.tick(self.tick);
        entity.render();
    }
}

pub struct GameState {
    cur_tick: usize,
    entity_map: DBVT<f32, Box<dyn Entity>, AABB<f32>>,
    uuid_map: BTreeMap<Uuid, DBVTLeafId>,
}

impl GameState {
    pub fn new(user_id: Uuid) -> Self {
        let mut entity_map = DBVT::new();
        // set up the player entity fast path
        let player_entity = box PlayerEntity::new(0.0, 0.0, 20);
        let player_entity_ptr = Box::into_raw(player_entity);
        unsafe { PLAYER_ENTITY_FASTPATH = player_entity_ptr };
        let player_entity = unsafe { Box::from_raw(player_entity_ptr) };
        let player_entity = player_entity as Box<dyn Entity>;
        let leaf = DBVTLeaf::new(player_entity.get_bounding_volume(), player_entity);
        let leaf_id = entity_map.insert(leaf);
        let mut uuid_map = BTreeMap::new();
        uuid_map.insert(user_id, leaf_id);

        GameState {
            cur_tick: 0,
            entity_map,
            uuid_map,
        }
    }

    #[allow(mutable_transmutes)]
    pub fn apply_msg(&mut self, entity_id: Uuid, update: ServerMessageContent) {
        match update {
            ServerMessageContent::status_update(StatusUpdate { payload, .. }) => match payload {
                Some(StatusPayload::creation_event(CreationEvent {
                    pos_x,
                    pos_y,
                    entity,
                    ..
                })) => if let Some(entity) = entity {
                    self.create_entity(&entity, entity_id, pos_x, pos_y)
                } else {
                    warn("Received entity creation update with no inner entity payload")
                },
                Some(StatusPayload::other(SimpleEvent::DELETION)) => {
                    // TODO
                }
                None => warn("Received `StatusUpdate` with no payload"),
            },
            _ => {
                let entity_key: &DBVTLeafId = match self.uuid_map.get(&entity_id) {
                    Some(key) => key,
                    None => {
                        error(format!(
                            "Received update for entity {} which doesn't exist",
                            entity_id
                        ));
                        return;
                    }
                };

                let DBVTLeaf { data, .. } = &self.entity_map[*entity_key];
                let entity: &mut Box<dyn Entity> = unsafe { mem::transmute(data) };
                entity.apply_update(&update);
            }
        }
    }

    /// Renders all entities in random order.  Some entities take a default action every game tick
    /// without taking input from the server.  This method iterates over all entities and
    /// optionally performs this mutation before rendering.  Returns the current tick.
    pub fn tick(&mut self) -> usize {
        let mut visitor = FullVisitor {
            tick: self.cur_tick,
        };
        self.entity_map.visit(&mut visitor);

        self.cur_tick += 1;
        self.cur_tick
    }

    fn create_entity(&mut self, entity: &EntityType, entity_id: Uuid, pos_x: f32, pos_y: f32) {
        let leaf: DBVTLeaf<_, _, _> = match entity {
            EntityType::player(player) => {
                log("Creating entity...");
                let entity = PlayerEntity::new(pos_x, pos_y, player.get_size() as u16);
                let entity: Box<dyn Entity> = box entity;
                DBVTLeaf::new(entity.get_bounding_volume(), entity)
            }
        };

        let leaf_id = self.entity_map.insert(leaf);
        self.uuid_map.insert(entity_id, leaf_id);
    }
}

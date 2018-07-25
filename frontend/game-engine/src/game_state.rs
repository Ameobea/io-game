//! Manages the state for the game and exposes methods for interacting and observing it.

use std::collections::BTreeMap;
use std::mem;
use std::sync::Mutex;

use uuid::Uuid;

use entity::Entity;
use proto_utils::ServerMessageContent;
use protos::server_messages::{
    CreationEvent_oneof_entity as EntityType, StatusUpdate,
    StatusUpdate_SimpleEvent as SimpleEvent, StatusUpdate_oneof_payload as Status,
};
use render_effects::RenderEffectManager;
use util::{error, log, warn};

pub struct State(pub Mutex<GameState>);

lazy_static! {
    static ref STATE: State = State(Mutex::new(GameState::new()));
    static ref RENDER_EFFECTS: Mutex<RenderEffectManager> = Mutex::new(RenderEffectManager::new());
}

/// Helper function to get the global game state.  We can do this disgusting unsafe lifetime hack
/// because this is running in WebAssembly.  The Mutex isn't even real, everything is running in a
/// single thread, and the global state will never get dropped anyway.
pub fn get_state() -> &'static mut GameState {
    let inner: &mut GameState = &mut *STATE.0.lock().unwrap();
    unsafe { mem::transmute(inner) }
}

/// Helper function to get the global render effect manager.  Uses the same unsafe hack as `state`.
pub fn get_effect_manager() -> &'static mut RenderEffectManager {
    let inner: &mut RenderEffectManager = &mut *RENDER_EFFECTS.lock().unwrap();
    unsafe { mem::transmute(inner) }
}

pub struct GameState {
    cur_tick: usize,
    pub entity_map: BTreeMap<Uuid, Box<Entity + Send>>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            cur_tick: 0,
            entity_map: BTreeMap::new(),
        }
    }

    pub fn apply_msg(&mut self, entity_id: Uuid, update: &ServerMessageContent) {
        log(format!("Applying message with id {}", entity_id));
        match update {
            ServerMessageContent::status_update(StatusUpdate { payload, .. }) => match payload {
                Some(Status::creation_event(evt)) => {
                    self.create_entity(evt.entity.as_ref().unwrap(), entity_id, evt.get_pos_x(), evt.get_pos_y())
                },
                Some(Status::other(simple_event)) => match simple_event {
                    SimpleEvent::DELETION => match self.entity_map.remove(&entity_id) {
                        Some(_) => (),
                        None => warn(format!(
                            "Unable to delete entity {} because it doesn't exist in the entity map!",
                            entity_id
                        )),
                    },
                },
                None => warn("Received a message with an empty status payload!"),
            },
            _ => {
                let entity: &mut Box<Entity + Send> = match self.entity_map.get_mut(&entity_id) {
                    Some(entity) => entity,
                    None => {
                        error(format!(
                            "Unable to find entity id {} to apply update!",
                            entity_id
                        ));
                        return;
                    }
                };

                entity.apply_update(update)
            }
        }
    }

    /// Renders all entities in random order.  Some entities take a default action every game tick
    /// without taking input from the server.  This method iterates over all entities and
    /// optionally performs this mutation before rendering.  Returns the current tick.
    pub fn tick(&mut self) -> usize {
        for (_id, entity) in &mut self.entity_map {
            entity.tick(self.cur_tick);
            entity.render();
        }
        self.cur_tick += 1;
        self.cur_tick
    }

    fn create_entity(&mut self, entity: &EntityType, entity_id: Uuid, pos_x: f64, pos_y: f64) {
        match entity {
            EntityType::player(player) => {
                log("Creating entity...");
                let entity = ::game::BaseEntity::new(
                    pos_x,
                    pos_y,
                    player.get_direction(),
                    player.get_size() as u16,
                );
                match self.entity_map.insert(entity_id, box entity) {
                    Some(_) => error(format!(
                        "While creating an entity, an old entity existed with the id {}!",
                        entity_id
                    )),
                    None => (),
                }
            }
        }
    }
}

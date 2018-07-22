//! Manages the state for the game and exposes methods for interacting and observing it.

use std::collections::BTreeMap;
use std::mem;
use std::sync::Mutex;

use uuid::Uuid;

use entity::Entity;
use proto_utils::ServerMessageContent;
use protos::server_messages::{StatusUpdate, StatusUpdate_Status as Status};
use util::{error, warn};

pub struct State(pub Mutex<GameState>);

lazy_static! {
    static ref STATE: State = State(Mutex::new(GameState::new()));
}

/// Helper function to get the global game state.  We can do this disgusting unsafe lifetime hack
/// because this is running in WebAssembly.  The Mutex isn't even real, everything is running in a
/// single thread, and the global state will never get dropped anyway.
pub fn state() -> &'static mut GameState {
    let state_inner: &mut GameState = &mut *STATE.0.lock().unwrap();
    unsafe { mem::transmute(state_inner) }
}

pub struct GameState {
    pub entity_map: BTreeMap<Uuid, Box<Entity + Send>>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            entity_map: BTreeMap::new(),
        }
    }

    pub fn apply_msg(&mut self, entity_id: Uuid, update: &ServerMessageContent) {
        match update {
            ServerMessageContent::status_update(StatusUpdate { status, .. }) => match status {
                Status::CREATED => {
                    unimplemented!();
                }
                Status::DELETED => match self.entity_map.remove(&entity_id) {
                    Some(_) => (),
                    None => warn(format!(
                        "Unable to delete entity {} because it doesn't exist in the entity map!",
                        entity_id
                    )),
                },
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
    /// optionally performs this mutation before rendering.
    pub fn tick(&mut self) {
        for (_id, entity) in &mut self.entity_map {
            entity.tick();
            entity.render();
        }
    }
}

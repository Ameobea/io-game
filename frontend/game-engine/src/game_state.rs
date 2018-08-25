//! Manages the state for the game and exposes methods for interacting and observing it.

use std::hint::unreachable_unchecked;
use std::mem;
use std::ptr;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT};

use native_physics::physics::entities::EntityHandles;
use native_physics::physics::world::PhysicsWorldInner as PhysicsWorld;
use uuid::Uuid;

use super::{init_input_handlers, start_game_loop};
use conf::CONF;
use entity::{apply_update, parse_proto_entity, render, tick, ClientState, Entity, PlayerEntity};
use proto_utils::{parse_server_msg_payload, InnerServerMessage, ServerMessageContent};
use protos::server_messages::{
    CreationEvent, ServerMessage, Snapshot, StatusUpdate, StatusUpdate_SimpleEvent as SimpleEvent,
    StatusUpdate_oneof_payload as StatusPayload,
};
use render_effects::RenderEffectManager;
use user_input::CurHeldKeys;
use util::{error, warn, CircularBuffer};

pub static mut STATE: *mut GameState = ptr::null_mut();
pub static mut EFFECTS_MANAGER: *mut RenderEffectManager = ptr::null_mut();
pub static mut CUR_HELD_KEYS: *mut CurHeldKeys = ptr::null_mut();
pub static GAME_LOOP_STARTED: AtomicBool = ATOMIC_BOOL_INIT; // false

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

pub struct GameState {
    pub cur_tick: usize,
    pub player_uuid: Uuid,
    pub world: PhysicsWorld<ClientState>,
    pub msg_buffer: CircularBuffer<ServerMessage>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            cur_tick: 0,
            player_uuid: Uuid::nil(), // Placeholder until we are assigned an ID by the server
            world: PhysicsWorld::new(),
            msg_buffer: CircularBuffer::new(CONF.network.message_buffer_size),
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
        // TODO: handle tick and timestamp; check for skipped messages and request re-sync etc.
        match update {
            ServerMessageContent::status_update(StatusUpdate { payload, .. }) => match payload {
                Some(StatusPayload::creation_event(creation_evt)) => {
                    self.create_entity(entity_id, &creation_evt)
                }
                Some(StatusPayload::other(SimpleEvent::DELETION)) => {
                    self.world.remove_entity(&entity_id);
                }
                None => warn("Received `StatusUpdate` with no payload"),
            },
            ServerMessageContent::snapshot(snapshot) => {
                self.apply_snapshot(snapshot);

                // It's possible this is the first snapshot, so start listening for mouse/keyoard
                // events now that we have a user entity in the snapshot.
                init_input_handlers();
            }
            ServerMessageContent::connect_successful(player_id) => {
                let player_id: Uuid = player_id.into();
                self.player_uuid = player_id;
            }
            ServerMessageContent::movement_update(ref movement_update) => {
                let (pos, velocity) = movement_update.into();
                // Update the entity's position and velocity on the underlying `PhysicsWorld`
                self.world.update_movement(&entity_id, &pos, &velocity);
            }
            _ => {
                let EntityHandles {
                    ref mut entity,
                    data: ref mut client_state,
                    ..
                } = match self.world.uuid_map.get_mut(&entity_id) {
                    Some(key) => key,
                    None => {
                        error(format!(
                            "Received update for entity {} which doesn't exist",
                            entity_id
                        ));
                        return;
                    }
                };

                apply_update(entity_id, entity, client_state, &update);
            }
        }
    }

    /// Removes all items from the UUID map and the DBVT, then reconstruct the game state from the
    /// contents of the snapshot
    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.world.clear();

        for mut snapshot_item in snapshot.items.into_iter() {
            let uuid: Uuid = snapshot_item.take_id().into();
            let creation_evt = snapshot_item.get_item();
            self.create_entity(uuid, creation_evt);
        }
    }

    /// Renders all entities in random order.  Some entities take a default action every game tick
    /// without taking input from the server.  This method iterates over all entities and
    /// optionally performs this mutation before rendering.  Returns the current tick.
    pub fn tick(&mut self) -> usize {
        self.world.step();

        for (
            _,
            EntityHandles {
                entity,
                body_handle,
                data: client_state,
                collider_handle,
                ..
            },
        ) in &mut self.world.uuid_map
        {
            tick(entity, client_state, self.cur_tick);
            let pos = match self.world.world.rigid_body(*body_handle) {
                Some(body) => body.position(),
                None => *self
                    .world
                    .world
                    .collider(*collider_handle)
                    .expect("Neither rigid body nor collider in world")
                    .position(),
            };

            render(entity, client_state, &pos, self.cur_tick);
        }

        self.cur_tick += 1;
        self.cur_tick
    }

    /// Creates an `Entity` from a `CreationEvent` and spawns it into the world
    pub fn create_entity(&mut self, entity_id: Uuid, creation_evt: &CreationEvent) {
        let entity_data = match parse_proto_entity(creation_evt) {
            Some(entity) => entity,
            None => {
                error("Error while parsing `CreationEvent` into an entity");
                return;
            }
        };

        self.world.spawn_entity(entity_id, entity_data);

        if entity_id == self.player_uuid {
            // Start the game loop if it's not yet been started
            let game_loop_started = GAME_LOOP_STARTED.load(Ordering::Relaxed);
            if !game_loop_started {
                GAME_LOOP_STARTED.store(false, Ordering::Relaxed);
                start_game_loop();
            }
        }
    }

    pub fn get_player_entity(&self) -> (&PlayerEntity, &ClientState) {
        let EntityHandles {
            entity,
            data: client_state,
            ..
        } = self.get_player_entity_handles();

        let player = match entity {
            Entity::Player(player) => player,
            _ => unsafe { unreachable_unchecked() },
        };

        (player, client_state)
    }

    pub fn get_player_entity_mut(&mut self) -> (&mut PlayerEntity, &mut ClientState) {
        let EntityHandles {
            entity,
            data: client_state,
            ..
        } = self
            .world
            .uuid_map
            .get_mut(&self.player_uuid)
            .expect(&format!(
                "Player entity ID {} not in `uuid_map` (`get_player_entity_mut`)",
                self.player_uuid,
            ));

        let player = match entity {
            Entity::Player(player) => player,
            _ => unsafe { unreachable_unchecked() },
        };

        (player, client_state)
    }

    pub fn get_player_entity_handles(&self) -> &EntityHandles<ClientState> {
        self.world.uuid_map.get(&self.player_uuid).expect(&format!(
            "Player entity ID {} not in `uuid_map` (`get_player_entity_handles`)",
            self.player_uuid,
        ))
    }
}

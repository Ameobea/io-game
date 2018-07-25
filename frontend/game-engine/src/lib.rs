#![feature(
    box_syntax,
    use_extern_macros,
    wasm_custom_section,
    wasm_import_module,
    u128_type
)]

extern crate protobuf;
extern crate uuid;
extern crate wasm_bindgen;

use std::panic;

use uuid::Uuid;
use wasm_bindgen::prelude::*;

pub mod entity;
pub mod game;
pub mod game_state;
pub mod proto_utils;
pub mod protos;
pub mod render_effects;
pub mod user_input;
pub mod util;

use self::game_state::{get_effects_manager, get_state, GameState, EFFECTS_MANAGER, STATE};
use self::proto_utils::{
    msg_to_bytes, parse_server_message, parse_socket_message, InnerServerMessage,
};
use self::protos::message_common::MovementDirection;
use self::protos::server_messages::{
    CreationEvent, CreationEvent_oneof_entity as EntityType, PlayerEntity, ServerMessage,
    StatusUpdate, StatusUpdate_oneof_payload as Status,
};
use render_effects::RenderEffectManager;
use util::error;

#[wasm_bindgen(module = "./renderMethods")]
extern "C" {
    pub fn render_quad(r: u8, g: u8, b: u8, x: u16, y: u16, width: u16, height: u16);
    pub fn render_arc(
        r: u8,
        g: u8,
        b: u8,
        x: u16,
        y: u16,
        width: u16,
        radius: u16,
        startAngle: f64,
        endAngle: f64,
        counterClockwise: bool,
    );
}

#[wasm_bindgen(module = "./inputWrapper")]
extern "C" {
    pub fn send_message(msg: Vec<u8>);
}

#[wasm_bindgen]
pub fn init() {
    panic::set_hook(Box::new(|info: &panic::PanicInfo| error(info.to_string())));

    let game_state = box GameState::new();
    unsafe { STATE = Box::into_raw(game_state) };

    let effects_manager = box RenderEffectManager::new();
    unsafe { EFFECTS_MANAGER = Box::into_raw(effects_manager) };
}

#[wasm_bindgen]
pub fn handle_message(bytes: &[u8]) {
    if let Some(InnerServerMessage { id, content }) = parse_server_message(bytes) {
        get_state().apply_msg(id, &content)
    } else {
        error("Error while parsing server message!");
    }
}

fn create_status_update(status: Status) -> StatusUpdate {
    let mut status_update = StatusUpdate::new();
    status_update.payload = Some(status);

    status_update
}

fn create_server_msg(
    id: Uuid,
    status_update: Option<StatusUpdate>,
    direction: Option<MovementDirection>,
) -> ServerMessage {
    let mut msg = ServerMessage::new();
    msg.set_id(id.into());
    if let Some(status_update) = status_update {
        msg.set_status_update(status_update);
    } else if let Some(direction) = direction {
        msg.set_movement_direction(direction);
    } else {
        panic!("ERROR: You must provide either a `status_update` or `movement_update`!");
    }

    msg
}

#[wasm_bindgen]
pub fn temp_gen_server_message_1() -> Vec<u8> {
    let mut creation_event = CreationEvent::new();
    creation_event.set_pos_x(50.);
    creation_event.set_pos_y(50.);
    let mut player_entity = PlayerEntity::new();
    player_entity.set_direction(MovementDirection::STOP);
    player_entity.set_size(60);
    creation_event.entity = Some(EntityType::player(player_entity));
    let status_update = create_status_update(Status::creation_event(creation_event));
    let msg = create_server_msg(Uuid::nil(), Some(status_update), None);

    msg_to_bytes(msg)
}

#[wasm_bindgen]
pub fn temp_gen_server_message_2() -> Vec<u8> {
    let movement_update = MovementDirection::RIGHT;
    let msg = create_server_msg(Uuid::nil(), None, Some(movement_update));

    msg_to_bytes(msg)
}

#[wasm_bindgen]
pub fn tick() {
    let cur_tick = get_state().tick();
    get_effects_manager().render_all(cur_tick);
}

#[wasm_bindgen]
pub fn decode_socket_message(bytes: &[u8]) -> Option<String> {
    parse_socket_message(bytes)
}

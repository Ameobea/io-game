#![feature(
    box_syntax,
    use_extern_macros,
    wasm_custom_section,
    wasm_import_module,
    u128_type
)]

#[macro_use]
extern crate lazy_static;
extern crate palette;
extern crate protobuf;
extern crate rand;
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
pub mod util;

use self::game_state::state;
use self::proto_utils::{msg_to_bytes, parse_server_message, InnerServerMessage};
use self::protos::message_common::MovementDirection;
use self::protos::server_messages::{
    CreationEvent, CreationEvent_oneof_entity as EntityType, PlayerEntity, ServerMessage,
    StatusUpdate, StatusUpdate_oneof_payload as Status,
};
use util::{error, log};

#[wasm_bindgen(module = "./renderMethods")]
extern "C" {
    pub fn render_quad(r: u8, g: u8, b: u8, x: u16, y: u16, width: u16, height: u16);
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    panic::set_hook(Box::new(|info: &panic::PanicInfo| error(info.to_string())));
}

#[wasm_bindgen]
pub fn handle_message(bytes: &[u8]) {
    if let Some(InnerServerMessage { id, content }) = parse_server_message(bytes) {
        state().apply_msg(id, &content)
    };
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
        error("ERROR: You must provide either a `status_update` or `movement_update`!");
        panic!();
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
    state().tick()
}

#[wasm_bindgen]
pub fn handle_mouse_down(x: u16, y: u16) {
    log(format!("MOUSE DOWN {}, {}", x, y));
}

#[wasm_bindgen]
pub fn handle_mouse_move(x: u16, y: u16) {
    log(format!("MOUSE MOVE {}, {}", x, y));
}

#[wasm_bindgen]
pub fn handle_mouse_up(x: u16, y: u16) {
    log(format!("MOUSE UP {}, {}", x, y))
}

#[wasm_bindgen]
pub fn handle_key_down(code: usize) {
    log(format!("KEY DOWN {}", code))
}

#[wasm_bindgen]
pub fn handle_key_up(code: usize) {
    log(format!("KE YUP {}", code));
}

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

use protobuf::Message;
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
use self::protos::message_common::MovementUpdate;
use self::protos::server_messages::{ServerMessage, StatusUpdate, StatusUpdate_Status as Status};
use util::{error, log};

#[wasm_bindgen(module = "./renderMethods")]
extern "C" {
    pub fn render_quad(color: &str, x: u16, y: u16, width: u16, height: u16);
}

#[wasm_bindgen]
pub fn handle_message(bytes: &[u8]) {
    let InnerServerMessage { id, content } = match parse_server_message(bytes) {
        Some(msg) => msg,
        None => {
            return;
        }
    };

    state().apply_msg(id, &content)
}

fn create_status_update(status: Status, pos_x: Option<f64>, pos_y: Option<f64>) -> StatusUpdate {
    let mut status_update = StatusUpdate::new();
    status_update.set_status(status);
    status_update.set_pos_x(pos_x.unwrap_or(0.));
    status_update.set_pos_y(pos_y.unwrap_or(0.));

    status_update
}

fn create_server_msg(
    id: Uuid,
    status_update: Option<StatusUpdate>,
    movement_update: Option<MovementUpdate>,
) -> ServerMessage {
    let mut msg = ServerMessage::new();
    msg.set_id(id.into());
    if let Some(status_update) = status_update {
        msg.set_status_update(status_update);
    } else if let Some(movement_update) = movement_update {
        msg.set_movement_update(movement_update);
    } else {
        error("ERROR: You must provide either a `status_update` or `movement_update`!");
        panic!();
    }

    msg
}

/// Simulates a random UUID, but uses the rand crate with WebAssembly support.
fn v4_uuid() -> Uuid {
    // Because I really don't care, honestly.
    let high_quality_entropy: (f64, f64) = (self::util::math_random(), self::util::math_random());
    unsafe { ::std::mem::transmute(high_quality_entropy) }
}

#[wasm_bindgen]
pub fn temp_gen_server_message_1() -> Vec<u8> {
    let status_update = create_status_update(Status::CREATED, Some(50.), Some(50.));
    let msg = create_server_msg(Uuid::nil(), Some(status_update), None);

    msg_to_bytes(msg)
}

#[wasm_bindgen]
pub fn temp_gen_server_message_2() -> Vec<u8> {
    let movement_update = MovementUpdate::RIGHT;
    let msg = create_server_msg(Uuid::nil(), None, Some(movement_update));

    msg_to_bytes(msg)
}

#[wasm_bindgen]
pub fn tick() {
    state().tick()
}

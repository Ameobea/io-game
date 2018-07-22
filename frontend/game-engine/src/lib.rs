#![feature(
    use_extern_macros,
    wasm_custom_section,
    wasm_import_module,
    u128_type
)]

#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate uuid;
extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

pub mod entity;
pub mod game_state;
pub mod proto_utils;
pub mod protos;
pub mod util;

use self::game_state::state;
use self::proto_utils::{parse_server_message, InnerServerMessage};

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

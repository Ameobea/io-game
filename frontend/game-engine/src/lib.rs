#![feature(
    box_syntax,
    use_extern_macros,
    wasm_custom_section,
    wasm_import_module,
    u128_type,
    trivial_bounds,
    nll,
)]

extern crate nalgebra;
extern crate ncollide2d;
extern crate protobuf;
extern crate uuid;
extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;

use std::panic;

use nalgebra::Vector2;
use wasm_bindgen::prelude::*;

pub mod conf;
pub mod entity;
pub mod game;
pub mod game_state;
pub mod phoenix_proto;
pub mod proto_utils;
pub mod protos;
pub mod render_effects;
pub mod user_input;
pub mod util;

use game_state::{get_effects_manager, get_state, GameState, EFFECTS_MANAGER, STATE};
use phoenix_proto::{join_game_channel, send_connect_message};
use proto_utils::{parse_server_message, InnerServerMessage};
use protos::server_messages::{AsteroidEntity, CreationEvent_oneof_entity as EntityType};
use render_effects::RenderEffectManager;
use util::{error, v4_uuid};

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
    pub fn render_line(r: u8, g: u8, b: u8, width: u16, x1: u16, y1: u16, x2: u16, y2: u16);
    pub fn fill_poly(r: u8, g: u8, b: u8, vertex_coords: &[f32]);
    pub fn render_point(r: u8, g: u8, b: u8, x: u16, y: u16);
}

#[wasm_bindgen(module = "./inputWrapper")]
extern "C" {
    pub fn send_message(msg: Vec<u8>);
}

#[wasm_bindgen]
pub fn init() {
    panic::set_hook(Box::new(|info: &panic::PanicInfo| error(info.to_string())));

    let player_id = join_game_channel();

    let game_state = box GameState::new(player_id);
    unsafe { STATE = Box::into_raw(game_state) };

    let effects_manager = box RenderEffectManager::new();
    unsafe { EFFECTS_MANAGER = Box::into_raw(effects_manager) };

    send_connect_message();
}

#[wasm_bindgen]
pub fn handle_message(bytes: &[u8]) {
    if let Some(InnerServerMessage { id, content }) = parse_server_message(bytes) {
        get_state().apply_msg(id, content)
    } else {
        error("Error while parsing server message!");
    }
}

#[wasm_bindgen]
pub fn tick() {
    let cur_tick = get_state().tick();
    get_effects_manager().render_all(cur_tick);
}

#[wasm_bindgen]
pub fn handle_channel_message(bytes: &[u8]) {
    phoenix_proto::handle_server_msg(bytes)
}

#[wasm_bindgen]
pub fn spawn_asteroid(
    point_coords: Vec<f32>,
    offset_x: f32,
    offset_y: f32,
    rotation_rads: f32,
    velocity_x: f32,
    velocity_y: f32,
    delta_rotation_rads: f32,
) {
    let mut entity = AsteroidEntity::new();
    entity.set_vert_coords(point_coords);
    entity.set_rotation(rotation_rads);
    entity.set_velocity_x(velocity_x);
    entity.set_velocty_y(velocity_y);
    entity.set_delta_rotation(delta_rotation_rads);

    get_state().create_entity(
        &EntityType::asteroid(entity),
        v4_uuid(),
        Vector2::new(offset_x, offset_y),
    );
}

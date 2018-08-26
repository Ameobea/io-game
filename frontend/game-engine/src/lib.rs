#![feature(box_syntax, trivial_bounds, nll, vec_resize_default)]

extern crate libcomposition;
extern crate nalgebra;
extern crate native_physics;
extern crate ncollide2d;
extern crate noise;
extern crate nphysics2d;
extern crate protobuf;
extern crate uuid;
extern crate wasm_bindgen;

use std::panic;

use nalgebra::Vector2;
use wasm_bindgen::prelude::*;

pub mod conf;
pub mod entity;
pub mod game;
pub mod game_state;
pub mod phoenix_proto;
pub mod physics_math;
pub mod proto_utils;
pub mod protos;
pub mod render_effects;
pub mod render_methods;
pub mod user_input;
pub mod util;

use game_state::{get_effects_manager, get_state, GameState, EFFECTS_MANAGER, STATE};
use phoenix_proto::{join_game_channel, send_connect_message};
use render_effects::RenderEffectManager;
use util::error;

#[wasm_bindgen(module = "./index")]
extern "C" {
    pub fn start_game_loop();
}

#[wasm_bindgen(module = "./inputWrapper")]
extern "C" {
    pub fn send_message(msg: Vec<u8>);
    pub fn init_input_handlers();
}

#[wasm_bindgen(module = "./webgl")]
extern "C" {
    pub fn create_background_texture(height: usize, width: usize, texture_data: &[u8]);
    pub fn draw_background(
        player_pos_x: f32,
        player_pos_y: f32,
        texture_width: usize,
        texture_height: usize,
    );
}

#[wasm_bindgen(module = "./renderMethods")]
extern "C" {
    pub fn clear_canvas();
    pub fn render_quad(r: u8, g: u8, b: u8, x: u16, y: u16, width: u16, height: u16);
    pub fn render_arc(
        r: u8,
        g: u8,
        b: u8,
        x: u16,
        y: u16,
        width: u16,
        radius: u16,
        startAngle: f32,
        endAngle: f32,
        counterClockwise: bool,
    );
    pub fn render_line(r: u8, g: u8, b: u8, width: u16, x1: u16, y1: u16, x2: u16, y2: u16);
    pub fn fill_poly(r: u8, g: u8, b: u8, vertex_coords: &[f32]);
    pub fn render_point(r: u8, g: u8, b: u8, x: u16, y: u16);
}

static mut CANVAS_WIDTH: f32 = 0.0;
static mut CANVAS_HEIGHT: f32 = 0.0;

/// Called by TypeScript after the WebSocket has been initialized.  Sends the message to join the
/// game channel.
#[wasm_bindgen]
pub fn init(canvas_width: f32, canvas_height: f32) {
    panic::set_hook(box |info: &panic::PanicInfo| error(info.to_string()));

    unsafe {
        CANVAS_WIDTH = canvas_width;
        CANVAS_HEIGHT = canvas_height;
    }

    join_game_channel();

    let game_state = box GameState::new();
    unsafe { STATE = Box::into_raw(game_state) };

    let effects_manager = box RenderEffectManager::new();
    unsafe { EFFECTS_MANAGER = Box::into_raw(effects_manager) };

    let background_texture = game::noise::generate_background_texture(1500, 1500);
    create_background_texture(1500, 1500, &background_texture);

    send_connect_message();
}

#[wasm_bindgen]
pub fn tick() {
    let player_body_handle = get_state().get_player_entity_handles().body_handle;
    let player_pos = get_state()
        .world
        .world
        .rigid_body(player_body_handle)
        .as_ref()
        .map(|rigid_body| rigid_body.position().translation.vector)
        .unwrap_or_else(Vector2::zeros);
    draw_background(player_pos.x, player_pos.y, 1500, 1500);

    let cur_tick = get_state().tick();
    get_effects_manager().render_all(cur_tick);
}

#[wasm_bindgen]
pub fn handle_channel_message(bytes: &[u8]) {
    phoenix_proto::handle_server_msg(bytes)
}

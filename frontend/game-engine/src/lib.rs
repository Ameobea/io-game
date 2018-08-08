#![feature(
    box_syntax,
    use_extern_macros,
    wasm_custom_section,
    wasm_import_module,
    u128_type,
    trivial_bounds,
    nll,
)]

extern crate libcomposition;
extern crate nalgebra;
extern crate ncollide2d;
extern crate noise;
extern crate protobuf;
extern crate uuid;
extern crate wasm_bindgen;

use std::panic;

use wasm_bindgen::prelude::*;

pub mod conf;
pub mod entity;
pub mod game;
pub mod game_state;
pub mod phoenix_proto;
pub mod physics;
pub mod proto_utils;
pub mod protos;
pub mod render_effects;
pub mod render_methods;
pub mod user_input;
pub mod util;

use game_state::{
    get_effects_manager, get_state, player_entity_fastpath, GameState, EFFECTS_MANAGER, STATE,
};
use phoenix_proto::{join_game_channel, send_connect_message};
use proto_utils::parse_server_message;
use protos::server_messages::{
    AsteroidEntity, CreationEvent, CreationEvent_oneof_entity as EntityType, MovementUpdate,
};
use render_effects::RenderEffectManager;
use util::{error, v4_uuid};

#[wasm_bindgen(module = "./inputWrapper")]
extern "C" {
    pub fn send_message(msg: Vec<u8>);
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

#[wasm_bindgen]
pub fn init(canvas_width: f32, canvas_height: f32) {
    panic::set_hook(Box::new(|info: &panic::PanicInfo| error(info.to_string())));

    unsafe {
        CANVAS_WIDTH = canvas_width;
        CANVAS_HEIGHT = canvas_height;
    }

    let player_id = join_game_channel();

    let game_state = box GameState::new(player_id);
    unsafe { STATE = Box::into_raw(game_state) };

    let effects_manager = box RenderEffectManager::new();
    unsafe { EFFECTS_MANAGER = Box::into_raw(effects_manager) };

    let background_texture = game::noise::generate_background_texture(1500, 1500);
    create_background_texture(1500, 1500, &background_texture);

    send_connect_message();
}

#[wasm_bindgen]
pub fn handle_message(bytes: &[u8]) {
    if let Some(msg) = parse_server_message(bytes) {
        get_state().apply_msg(msg)
    }
}

#[wasm_bindgen]
pub fn tick() {
    let player_pos = player_entity_fastpath().pos();
    draw_background(player_pos.x, player_pos.y, 1500, 1500);

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
    angular_momentum: f32,
) {
    let mut entity = AsteroidEntity::new();
    entity.set_vert_coords(point_coords);
    let mut movement = MovementUpdate::new();
    movement.set_pos_x(offset_x);
    movement.set_pos_y(offset_y);
    movement.set_rotation(rotation_rads);
    movement.set_velocity_x(velocity_x);
    movement.set_velocity_y(velocity_y);
    movement.set_angular_velocity(angular_momentum);
    let mut creation_evt = CreationEvent::new();
    creation_evt.entity = Some(EntityType::asteroid(entity));

    get_state().create_entity(v4_uuid(), &creation_evt);
}

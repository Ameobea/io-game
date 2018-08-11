use std::hint::unreachable_unchecked;

use nalgebra::Point2;
use wasm_bindgen::prelude::*;

use game::effects::DemoCircle;
use game_state::{get_cur_held_keys, get_effects_manager, get_state};
use proto_utils::send_user_message;
use protos::client_messages::{BeamAim, ClientMessage_oneof_payload as ClientMessageContent};
use protos::message_common::MovementDirection as Direction;
use util::Color;

#[wasm_bindgen]
pub fn handle_mouse_down(x: u16, y: u16) {
    let effect = DemoCircle {
        color: Color::random(),
        width: 2,
        pos: Point2::new(x as f32, y as f32),
        cur_size: 0.,
        max_size: 10.,
        increment: 3.4,
    };
    get_effects_manager().add_effect(box effect);

    get_state().get_player_entity_mut().0.beam_on = true;

    // Send a "beam on" message to the server
    let payload = ClientMessageContent::beam_toggle(true);
    send_user_message(payload);
}

#[wasm_bindgen]
pub fn handle_mouse_move(x: f32, y: f32) {
    // Update the beam direction locally
    get_state().get_player_entity_mut().0.beam_aim = Point2::new(x, y);

    // Send a beam direction update message to the server
    let mut aim = BeamAim::new();
    aim.set_x(x as u32);
    aim.set_y(y as u32);
    let payload = ClientMessageContent::beam_rotation(aim);
    send_user_message(payload);
}

#[wasm_bindgen]
pub fn handle_mouse_up(_x: u16, _y: u16) {
    get_state().get_player_entity_mut().0.beam_on = false;

    // Send a "beam off" message to the server
    let payload = ClientMessageContent::beam_toggle(false);
    send_user_message(payload);
}

pub struct CurHeldKeys {
    w: bool,
    s: bool,
    a: bool,
    d: bool,
}

impl CurHeldKeys {
    pub fn new() -> Self {
        CurHeldKeys {
            w: false,
            s: false,
            a: false,
            d: false,
        }
    }

    pub fn set(&mut self, code: usize, down: bool) {
        match code {
            87 => self.w = down,
            83 => self.s = down,
            68 => self.d = down,
            65 => self.a = down,
            _ => (),
        }
    }

    pub fn no_keys_held(&self) -> bool {
        !self.w && !self.s && !self.a && !self.d
    }

    pub fn get_cur_direction(&self) -> Direction {
        fn movement_vector(a: bool, b: bool) -> i8 {
            match (a, b) {
                (true, true) | (false, false) => 0,
                (false, true) => 1,
                (true, false) => -1,
            }
        }

        let horiz = movement_vector(self.a, self.d);
        let vert = movement_vector(self.w, self.s);

        match (horiz, vert) {
            (0, 1) => Direction::DOWN,
            (-1, 1) => Direction::DOWN_LEFT,
            (1, 1) => Direction::DOWN_RIGHT,
            (-1, 0) => Direction::LEFT,
            (1, 0) => Direction::RIGHT,
            (0, 0) => Direction::STOP,
            (0, -1) => Direction::UP,
            (-1, -1) => Direction::UP_LEFT,
            (1, -1) => Direction::UP_RIGHT,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

fn send_movement_msg(direction: Direction) {
    let payload = ClientMessageContent::player_move(direction);
    send_user_message(payload);
}

fn process_movement_update(code: usize, down: bool) {
    let cur_held_keys = get_cur_held_keys();
    let old_direction = cur_held_keys.get_cur_direction();
    get_cur_held_keys().set(code, down);
    let new_direction = cur_held_keys.get_cur_direction();

    if old_direction != new_direction {
        send_movement_msg(new_direction);

        // Update direction input directly on the local player entity
        get_state().get_player_entity_mut().0.movement = new_direction.into();
    }
}

#[wasm_bindgen]
pub fn handle_key_down(code: usize) {
    process_movement_update(code, true);
}

#[wasm_bindgen]
pub fn handle_key_up(code: usize) {
    process_movement_update(code, false);
}

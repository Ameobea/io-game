use game::effects::DemoCircleEffect;
use util::Color;
use wasm_bindgen::prelude::*;

use game_state::{get_cur_held_keys, get_effects_manager};
use proto_utils::send_user_message;
use protos::client_messages::ClientMessage_oneof_payload as ClientMessageContent;
use protos::message_common::MovementDirection;

#[wasm_bindgen]
pub fn handle_mouse_down(x: u16, y: u16) {
    let effect = DemoCircleEffect {
        color: Color::random(),
        width: 6,
        x: x as f32,
        y: y as f32,
        cur_size: 0.,
        max_size: 30.,
        increment: 1.,
    };
    get_effects_manager().add_effect(box effect);
}

#[wasm_bindgen]
pub fn handle_mouse_move(x: u16, y: u16) {
    let effect = DemoCircleEffect {
        color: Color::random(),
        width: 6,
        x: x as f32,
        y: y as f32,
        cur_size: 0.,
        max_size: 6.,
        increment: 0.75,
    };
    get_effects_manager().add_effect(box effect);
}

#[wasm_bindgen]
pub fn handle_mouse_up(_x: u16, _y: u16) {}

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

    pub fn set_down(&mut self, code: usize) {
        match code {
            87 => self.w = true,
            83 => self.s = true,
            68 => self.d = true,
            65 => self.a = true,
            _ => (),
        }
    }

    pub fn set_up(&mut self, code: usize) {
        match code {
            87 => self.w = false,
            83 => self.s = false,
            68 => self.d = false,
            65 => self.a = false,
            _ => (),
        }
    }

    pub fn is_pressed(&self, code: usize) -> bool {
        match code {
            87 => self.w == true,
            83 => self.s == true,
            68 => self.d == true,
            65 => self.a == true,
            _ => false,
        }
    }

    pub fn no_keys_held(&self) -> bool {
        !self.w && !self.s && !self.a && !self.d
    }
}

fn send_movement_msg(direction: MovementDirection) {
    let payload = ClientMessageContent::player_move(direction);
    send_user_message(payload);
}

#[wasm_bindgen]
pub fn handle_key_down(code: usize) {
    let movement_direction = match code {
        87 => MovementDirection::UP,
        83 => MovementDirection::DOWN,
        68 => MovementDirection::RIGHT,
        65 => MovementDirection::LEFT,
        _ => {
            return;
        }
    };

    if !get_cur_held_keys().is_pressed(code) {
        send_movement_msg(movement_direction);
    }
    get_cur_held_keys().set_down(code);
}

#[wasm_bindgen]
pub fn handle_key_up(code: usize) {
    get_cur_held_keys().set_up(code);
    if get_cur_held_keys().no_keys_held() {
        send_movement_msg(MovementDirection::STOP);
    }
}

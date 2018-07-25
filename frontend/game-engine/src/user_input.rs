use std::sync::Mutex;

use game::effects::DemoCircleEffect;
use util::Color;
use wasm_bindgen::prelude::*;

use game_state::get_effect_manager;
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
    get_effect_manager().add_effect(box effect);
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
    get_effect_manager().add_effect(box effect);
}

#[wasm_bindgen]
pub fn handle_mouse_up(_x: u16, _y: u16) {}

struct CurHeldKeysInner {
    w: bool,
    s: bool,
    a: bool,
    d: bool,
}

struct CurHeldKeys(Mutex<CurHeldKeysInner>);

lazy_static! {
    static ref CUR_HELD_KEYS: CurHeldKeys = CurHeldKeys::new();
}

impl CurHeldKeys {
    pub fn new() -> Self {
        CurHeldKeys(Mutex::new(CurHeldKeysInner {
            w: false,
            s: false,
            a: false,
            d: false,
        }))
    }

    pub fn set_down(&self, code: usize) {
        let mut inner = self.0.lock().unwrap();
        match code {
            87 => inner.w = true,
            83 => inner.s = true,
            68 => inner.d = true,
            65 => inner.a = true,
            _ => (),
        }
    }

    pub fn set_up(&self, code: usize) {
        let mut inner = self.0.lock().unwrap();
        match code {
            87 => inner.w = false,
            83 => inner.s = false,
            68 => inner.d = false,
            65 => inner.a = false,
            _ => (),
        }
    }

    pub fn is_pressed(&self, code: usize) -> bool {
        let inner = self.0.lock().unwrap();
        match code {
            87 => inner.w == true,
            83 => inner.s == true,
            68 => inner.d == true,
            65 => inner.a == true,
            _ => false,
        }
    }

    pub fn no_keys_held(&self) -> bool {
        let inner = self.0.lock().unwrap();
        !inner.w && !inner.s && !inner.a && !inner.d
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

    if !CUR_HELD_KEYS.is_pressed(code) {
        send_movement_msg(movement_direction);
    }
    CUR_HELD_KEYS.set_down(code);
}

#[wasm_bindgen]
pub fn handle_key_up(code: usize) {
    CUR_HELD_KEYS.set_up(code);
    if CUR_HELD_KEYS.no_keys_held() {
        send_movement_msg(MovementDirection::STOP);
    }
}

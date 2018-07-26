//! Contains implementation-specific code that is not generic for the engine.

use super::render_quad;
use conf::CONF;
use entity::Entity;
use game_state::get_effects_manager;
use protos::message_common::MovementDirection as Direction;
use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::{math_random, Color};

pub mod effects;

use self::effects::DemoCircleEffect;

struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Rgb { red, green, blue }
    }
}

/// The basic entity that is used right now.  They're all rendered as a square, but they all have
/// a unique color.
pub struct PlayerEntity {
    color: Rgb,
    pub x: f32,
    pub y: f32,
    // Which way the user is telling this entity to go
    pub direction_input: Direction,
    // Direction in radians
    pub direction: f32,
    // pixels/tick
    pub speed: f32,
    pub size: u16,
}

impl PlayerEntity {
    pub fn new(x: f32, y: f32, size: u16) -> Self {
        PlayerEntity {
            color: Rgb::new(
                (math_random() * 255.) as u8,
                (math_random() * 255.) as u8,
                (math_random() * 255.) as u8,
            ),
            x,
            y,
            direction_input: Direction::STOP,
            direction: 0.,
            speed: 0.,
            size,
        }
    }
}

impl Entity for PlayerEntity {
    fn render(&self) {
        render_quad(
            self.color.red,
            self.color.green,
            self.color.blue,
            self.x as u16,
            self.y as u16,
            self.size,
            self.size,
        );
    }

    fn tick(&mut self, tick: usize) {
        if tick % 120 == 0 {
            let effect = DemoCircleEffect {
                color: Color::random(),
                width: 3,
                x: self.x as f32,
                y: (self.y + 50.) as f32,
                cur_size: 0.,
                max_size: 50.,
                increment: 0.5,
            };

            get_effects_manager().add_effect(box effect);
        }
    }

    fn apply_update(&mut self, update: &ServerMessageContent) {
        unimplemented!()
        // match update {
        //     ServerMessageContent::movement_direction(direction) => self.direction = *direction,
        //     _ => error("Unexpected server message type received in entity update handler!"),
        // }
    }
}

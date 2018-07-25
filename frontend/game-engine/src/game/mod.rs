//! Contains implementation-specific code that is not generic for the engine.

use super::render_quad;
use entity::Entity;
use game_state::get_effects_manager;
use protos::message_common::MovementDirection as Direction;
use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::{error, math_random, Color};

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
pub struct BaseEntity {
    color: Rgb,
    pub x: f64,
    pub y: f64,
    pub direction: Direction,
    pub size: u16,
}

impl BaseEntity {
    pub fn new(x: f64, y: f64, direction: Direction, size: u16) -> Self {
        BaseEntity {
            color: Rgb::new(
                (math_random() * 255.) as u8,
                (math_random() * 255.) as u8,
                (math_random() * 255.) as u8,
            ),
            x,
            y,
            direction,
            size,
        }
    }
}

impl Entity for BaseEntity {
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
        match self.direction {
            Direction::DOWN => self.y += 0.1,
            Direction::LEFT => self.x -= 0.1,
            Direction::RIGHT => self.x += 0.1,
            Direction::UP => self.y -= 0.1,
            Direction::STOP => (),
        };

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
        match update {
            ServerMessageContent::movement_direction(direction) => self.direction = *direction,
            _ => error("Unexpected server message type received in entity update handler!"),
        }
    }
}

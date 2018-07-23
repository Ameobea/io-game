//! Contains implementation-specific code that is not generic for the engine.

use palette::rgb::Rgb;

use super::render_quad;
use entity::Entity;
use protos::message_common::MovementUpdate as Direction;
use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::{error, math_random};

/// The basic entity that is used right now.  They're all rendered as a square, but they all have
/// a unique color.
pub struct BaseEntity {
    pub color: Rgb,
    pub x: f64,
    pub y: f64,
    pub direction: Direction,
    pub size: u16,
}

impl BaseEntity {
    pub fn new(x: f64, y: f64) -> Self {
        BaseEntity {
            color: Rgb::new(
                math_random() as f32,
                math_random() as f32,
                math_random() as f32,
            ),
            x,
            y,
            direction: Direction::STOP,
            size: 10,
        }
    }
}

impl Entity for BaseEntity {
    fn render(&self) {
        render_quad(
            &format!(
                "rgb({},{},{})",
                self.color.red, self.color.green, self.color.blue,
            ),
            self.x as u16,
            self.y as u16,
            self.size,
            self.size,
        );
    }

    fn tick(&mut self) {
        match self.direction {
            Direction::DOWN => self.y += 0.1,
            Direction::LEFT => self.x -= 0.1,
            Direction::RIGHT => self.x += 0.1,
            Direction::UP => self.y -= 0.1,
            Direction::STOP => (),
        }
    }

    fn apply_update(&mut self, update: &ServerMessageContent) {
        match update {
            ServerMessageContent::movement_update(direction) => self.direction = *direction,
            _ => error("Unexpected server message type received in entity update handler!"),
        }
    }
}

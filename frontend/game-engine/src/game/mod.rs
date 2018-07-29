//! Contains implementation-specific code that is not generic for the engine.

use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::aabb::AABB;

use super::{render_line, render_quad};
use conf::CONF;
use entity::Entity;
use game_state::get_effects_manager;
use protos::message_common::MovementDirection as Direction;
use protos::server_messages::{
    MovementUpdate, ServerMessage_oneof_payload as ServerMessageContent,
};
use util::{error, log, magnitude, Color};

pub mod effects;

use self::effects::DemoCircleEffect;

/// The basic entity that is used right now.  They're all rendered as a square, but they all have
/// a unique color.
pub struct PlayerEntity {
    color: Color,
    pub pos: Point2<f32>,
    /// Which way the user is telling this entity to go
    pub direction_input: Direction,
    /// Velocity vector along the x/y axises in pixels/tick
    pub velocity: Vector2<f32>,
    pub size: u16,
    /// A normalized vector that represents the direction that the beam is pointing
    pub beam_rotation: Vector2<f32>,
    /// Current x location of the mouse pointer from the last mouse move event
    cached_mouse_pos: Point2<f32>,
}

impl PlayerEntity {
    pub fn new(pos: Point2<f32>, size: u16) -> Self {
        PlayerEntity {
            color: Color::random(),
            pos,
            direction_input: Direction::STOP,
            velocity: Vector2::zeros(),
            size,
            beam_rotation: Vector2::new(1., 0.),
            cached_mouse_pos: unsafe { Point2::new_uninitialized() },
        }
    }

    fn set_movement(
        &mut self,
        &MovementUpdate {
            pos_x,
            pos_y,
            velocity_x,
            velocity_y,
            ..
        }: &MovementUpdate,
    ) {
        log(format!(
            "{}, {}, {}, {}",
            pos_x, pos_y, velocity_x, velocity_y
        ));
        self.pos = Point2::new(pos_x, pos_y);
        self.velocity = Vector2::new(velocity_x, velocity_y);
    }

    fn tick_movement(&mut self) {
        let (x_diff, y_diff) = match self.direction_input {
            Direction::DOWN => (0., 1.),
            Direction::DOWN_LEFT => (-1., 1.),
            Direction::DOWN_RIGHT => (1., 1.),
            Direction::LEFT => (-1., 0.),
            Direction::RIGHT => (1., 0.),
            Direction::STOP => (0., 0.),
            Direction::UP => (0., -1.),
            Direction::UP_LEFT => (-1., -1.),
            Direction::UP_RIGHT => (1., -1.),
        };

        let acceleration = CONF.physics.acceleration_per_tick;
        let max_speed = CONF.physics.max_player_speed;
        let movement_diff = Vector2::new(x_diff, y_diff) * acceleration;
        if movement_diff.x + movement_diff.y < max_speed {
            self.velocity += movement_diff;
        }

        self.pos += self.velocity;
        self.pos *= 1. - CONF.physics.friction_per_tick;
    }

    pub fn update_beam(&mut self, mouse_pos: Point2<f32>) {
        self.cached_mouse_pos = mouse_pos;
        let mouse_pos_diff = mouse_pos - self.pos;
        let mouse_vector_magnitude = magnitude(mouse_pos_diff);

        let normalized_mouse_vector = mouse_pos_diff / mouse_vector_magnitude;
        self.beam_rotation = normalized_mouse_vector;
    }
}

impl Entity for PlayerEntity {
    fn render(&self) {
        // Draw entity body
        render_quad(
            self.color.red,
            self.color.green,
            self.color.blue,
            self.pos.x as u16,
            self.pos.y as u16,
            self.size,
            self.size,
        );

        let beam_len: f32 = 25.;
        let beam_vec = self.beam_rotation * beam_len;
        let beam_endpoint = self.pos + beam_vec;
        // Draw beam
        render_line(
            8,
            self.pos.x as u16,
            self.pos.y as u16,
            beam_endpoint.x as u16,
            beam_endpoint.y as u16,
        );
    }

    fn tick(&mut self, tick: usize) -> bool {
        self.tick_movement();
        self.update_beam(self.cached_mouse_pos);

        if tick % 120 == 0 {
            let effect = DemoCircleEffect {
                color: Color::random(),
                width: 3,
                x: self.pos.x,
                y: self.pos.y + 50.,
                cur_size: 0.,
                max_size: 50.,
                increment: 0.5,
            };

            get_effects_manager().add_effect(box effect);
        }

        self.velocity.x + self.velocity.y > 0.
    }

    fn apply_update(&mut self, update: &ServerMessageContent) -> bool {
        match update {
            ServerMessageContent::movement_update(movement_update) => {
                self.set_movement(movement_update);
                true
            }
            _ => {
                error("Unexpected server message type received in entity update handler!");
                false
            }
        }
    }

    fn get_bounding_volume(&self) -> AABB<f32> {
        AABB::new(
            self.pos,
            self.pos + Vector2::new(self.size as f32, self.size as f32),
        )
    }
}

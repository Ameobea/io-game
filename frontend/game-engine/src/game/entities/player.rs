use nalgebra::{Isometry2, Point2, Translation2, UnitComplex, Vector2};
use ncollide2d::bounding_volume::aabb::AABB;
use ncollide2d::shape::Shape;

use conf::CONF;
use entity::Entity;
use game::effects::DrillingParticles;
use game_state::{get_effects_manager, get_state};
use physics::ray_collision;
use proto_utils::ServerMessageContent;
use protos::message_common::MovementDirection as Direction;
use protos::server_messages::MovementUpdate;
use render_methods::{render_line, render_quad};
use util::{error, Color, Rotation, Velocity2};

use super::super::effects::DemoCircle;

fn player_vertices(size: u16) -> Vec<Point2<f32>> {
    let half_perim = 0.5 * (size as f32);
    vec![
        Point2::new(half_perim, half_perim),
        Point2::new(-half_perim, half_perim),
        Point2::new(-half_perim, -half_perim),
        Point2::new(half_perim, -half_perim),
    ]
}

// The basic entity that is used right now.  They're all rendered as a square, but they all have
/// a unique color.
pub struct PlayerEntity {
    pub color: Color,
    pub isometry: Isometry2<f32>,
    /// Center of mass of the entity in the entity's coordinate space
    pub local_center_of_mass: Point2<f32>,
    /// Center of mass of the player's entity in world coordinates
    pub center_of_mass: Point2<f32>,
    /// Which way the user is telling this entity to go
    pub direction_input: Direction,
    /// Velocity vector along the x/y axises in pixels/tick
    pub velocity: Velocity2,
    pub size: u16,
    vertices: Vec<Point2<f32>>,
    /// A normalized vector that represents the direction that the beam is pointing
    pub beam_rotation: Vector2<f32>,
    /// If the beam is currently being fired
    pub beam_active: bool,
    /// Current x location of the mouse pointer from the last mouse move event
    pub cached_mouse_pos: Point2<f32>,
}

impl Into<Vector2<f32>> for Direction {
    fn into(self) -> Vector2<f32> {
        let (dir_x, dir_y): (f32, f32) = match self {
            Direction::UP => (0., -1.),
            Direction::UP_RIGHT => (1., -1.),
            Direction::RIGHT => (1., 0.),
            Direction::DOWN_RIGHT => (1., 1.),
            Direction::DOWN => (0., 1.),
            Direction::DOWN_LEFT => (-1., 1.),
            Direction::LEFT => (-1., 0.),
            Direction::UP_LEFT => (-1., -1.),
            Direction::STOP => {
                return Vector2::new(0., 0.);
            }
        };
        Vector2::new(dir_x, dir_y).normalize()
    }
}

impl PlayerEntity {
    pub fn new(isometry: Isometry2<f32>, center_of_mass: Point2<f32>, size: u16) -> Self {
        PlayerEntity {
            color: Color::random(),
            isometry,
            local_center_of_mass: isometry.inverse() * center_of_mass,
            center_of_mass,
            direction_input: Direction::STOP,
            velocity: Velocity2::new(Vector2::zeros(), 0.),
            size,
            vertices: player_vertices(size),
            beam_rotation: Vector2::new(1., 0.),
            beam_active: false,
            cached_mouse_pos: unsafe { Point2::new_uninitialized() },
        }
    }

    fn tick_movement(&mut self) {
        // Apply the force from movement to the entity and simulate its effect on the entity's
        // acceleration and velocity
        let mut movement_acceleration: Vector2<f32> = self.direction_input.into();
        movement_acceleration *= CONF.physics.acceleration_per_tick;
        self.velocity.linear += movement_acceleration * CONF.physics.engine_time_step;

        // TODO: Generalize
        // The linear + angular components of the velocity
        let rotation = Rotation::new(self.velocity.angular);
        let translation = Translation2::from_vector(self.velocity.linear);
        // Vector from the origin to this entity's center of mass in world coordinates
        let shift = Translation2::from_vector(self.center_of_mass.coords);
        // The actual displacement of position that will occur
        let disp = translation * shift * rotation * shift.inverse();
        // Adjust the position of this entity by the displacement
        self.isometry = disp * self.isometry;
        self.center_of_mass = self.isometry * self.local_center_of_mass;

        self.velocity.linear *= 1.0 - CONF.physics.friction_per_tick;
    }

    pub fn update_beam(&mut self, mouse_pos: Point2<f32>) {
        self.cached_mouse_pos = mouse_pos;
        self.beam_rotation = (mouse_pos - self.pos()).normalize();
    }

    pub fn set_beam_active(&mut self, active: bool) {
        self.beam_active = active;
    }

    /// Finds the closest collision point between the mouse coordinates and the player's entity.
    fn get_beam_start_point(&self) -> Option<Point2<f32>> {
        let inverse_beam_rotation = -1. * self.beam_rotation;
        ray_collision(
            self.cached_mouse_pos,
            inverse_beam_rotation,
            &self.vertices,
            self.get_isometry(),
        ).map(|(pt, _)| pt + 2.5 * inverse_beam_rotation)
    }

    #[inline(always)]
    pub fn pos(&self) -> Point2<f32> {
        Point2::new(
            self.isometry.translation.vector.x,
            self.isometry.translation.vector.y,
        )
    }
}

impl Shape<f32> for PlayerEntity {
    fn aabb(&self, m: &Isometry2<f32>) -> AABB<f32> {
        AABB::new(
            m * self.pos(),
            m * (self.pos() + Vector2::new(self.size as f32, self.size as f32)),
        )
    }
}

impl Entity for PlayerEntity {
    // TODO: Account for isometry
    fn render(&self, cur_tick: usize) {
        // Draw entity body
        render_quad(
            &self.color,
            self.pos() - Vector2::new(0.5 * self.size as f32, 0.5 * self.size as f32),
            self.size,
            self.size,
        );

        let beam_gun_len: f32 = 25.;
        let beam_vec = self.beam_rotation * beam_gun_len;
        let beam_gun_start_point = match self.get_beam_start_point() {
            Some(pos) => pos,
            None => {
                return;
            }
        };
        let beam_gun_endpoint = beam_gun_start_point + beam_vec;

        // Draw beam gun
        render_line(&self.color, 8, beam_gun_start_point, beam_gun_endpoint);

        // Draw beam if beam is active
        if !self.beam_active {
            return;
        }

        let beam_start = beam_gun_endpoint;
        let beam_endpoint = beam_start + (beam_vec * 10.);

        let (mins, maxs) = (
            Point2::new(
                beam_start.x.min(beam_endpoint.x) - 0.5,
                beam_start.y.min(beam_endpoint.y) - 0.5,
            ),
            Point2::new(
                beam_start.x.max(beam_endpoint.x) + 0.5,
                beam_start.y.max(beam_endpoint.y) + 0.5,
            ),
        );

        // Check if anything collides with the beam
        let beam_bv = AABB::new(mins, maxs);
        let possible_collisions = get_state().broad_phase(&beam_bv);
        let broad_phase_miss = possible_collisions.is_empty();

        let collision_check_opt = possible_collisions
            .into_iter()
            .map(|entity_id| -> Option<(Point2<f32>, f32)> {
                let (_leaf_id, entity) = get_state()
                    .uuid_map
                    .get(&entity_id)
                    .expect("Entity was in the collision tree but not the UUID map!");

                let verts = entity.get_vertices();
                ray_collision(beam_start, self.beam_rotation, verts, entity.get_isometry())
            }).fold(None, |acc, distance_opt| -> Option<(Point2<f32>, f32)> {
                match (acc, distance_opt) {
                    (None, Some(res)) => Some(res),
                    (Some((nearest_collision, smallest_distance)), Some((collision, dist))) => {
                        if smallest_distance < dist {
                            Some((nearest_collision, smallest_distance))
                        } else {
                            Some((collision, dist))
                        }
                    }
                    (Some(acc), None) => Some(acc),
                    (None, None) => None,
                }
            });

        let (line_color, beam_endpoint) = if let Some((nearest_collision, _)) = collision_check_opt
        {
            let drilling_effect = DrillingParticles::new(
                nearest_collision,
                cur_tick,
                5,
                4,
                1.45,
                Color {
                    red: 240,
                    green: 30,
                    blue: 41,
                },
            );
            get_effects_manager().add_effect(box drilling_effect);
            let color = &Color {
                red: 255,
                green: 0,
                blue: 0,
            };
            (color, nearest_collision)
        } else {
            let color = if broad_phase_miss {
                &Color {
                    red: 0,
                    green: 0,
                    blue: 255,
                }
            } else {
                &self.color
            };
            (color, beam_endpoint)
        };

        render_line(&line_color, 1, beam_start, beam_endpoint);
    }

    fn set_movement(
        &mut self,
        &MovementUpdate {
            pos_x,
            pos_y,
            rotation,
            velocity_x,
            velocity_y,
            angular_velocity,
            ..
        }: &MovementUpdate,
    ) {
        self.isometry = Isometry2::from_parts(
            Translation2::from_vector(Vector2::new(pos_x, pos_y)),
            UnitComplex::from_angle(rotation),
        );
        self.velocity = Velocity2::new(Vector2::new(velocity_x, velocity_y), angular_velocity);
    }

    // TODO: Account for angular momentum
    fn tick(&mut self, tick: usize) -> bool {
        self.tick_movement();
        self.update_beam(self.cached_mouse_pos);

        if tick % 120 == 0 {
            let effect = DemoCircle {
                color: Color::random(),
                width: 3,
                pos: self.pos() + Vector2::new(0.0, 50.0),
                cur_size: 0.,
                max_size: 50.,
                increment: 0.5,
            };

            get_effects_manager().add_effect(box effect);
        }

        self.velocity.linear.x + self.velocity.linear.y + self.velocity.angular > 0.
    }

    fn apply_update(&mut self, update: &ServerMessageContent) -> bool {
        match update {
            _ => {
                error("Unexpected server message type received in entity update handler!");
                false
            }
        }
    }

    fn get_bounding_volume(&self) -> AABB<f32> {
        AABB::new(
            self.pos(),
            self.pos() + Vector2::new(self.size as f32, self.size as f32),
        )
    }

    fn get_isometry(&self) -> &Isometry2<f32> {
        &self.isometry
    }

    fn get_vertices(&self) -> &[Point2<f32>] {
        &self.vertices
    }
}

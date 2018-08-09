use nalgebra::{Isometry2, Point2, Translation2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, HasBoundingVolume};
use ncollide2d::query::RayCast;
use ncollide2d::shape::{Polyline, Shape};

use entity::Entity;
use proto_utils::ServerMessageContent;
use protos::server_messages::{AsteroidEntity as ProtoAsteroidEntity, MovementUpdate};
use render_methods::fill_poly;
use util::{error, Color, Rotation, Velocity2};

pub struct Asteroid {
    /// Position of this entity in the world's coordinate space
    pub isometry: Isometry2<f32>,
    /// Center of mass of the entity in the entity's coordinate space
    pub local_center_of_mass: Point2<f32>,
    /// Center of mass of the entity in the world's coordinate space
    pub center_of_mass: Point2<f32>,
    /// The vertices of this entity in the entity's local space
    pub verts: Vec<Point2<f32>>,
    pub color: Color,
    /// Linear + angular speed of the asteroid in world units/time step
    pub velocity: Velocity2,
    /// The 2D line that forms the perimeter of this entity
    poly_line: Polyline<f32>,
    /// The vertices of this entity stored as points, transformed into the world space
    transformed_coords_buffer: Vec<f32>,
}

fn process_movement(movement: &MovementUpdate) -> (Isometry2<f32>, Velocity2) {
    let pos = Isometry2::new(
        Vector2::new(movement.get_pos_x(), movement.get_pos_y()),
        movement.get_rotation(),
    );
    let velocity = Velocity2::new(
        Vector2::new(movement.get_velocity_x(), movement.get_velocity_y()),
        movement.get_angular_velocity(),
    );

    (pos, velocity)
}

impl Asteroid {
    pub fn new(
        verts: Vec<Point2<f32>>,
        center_of_mass: Point2<f32>,
        isometry: Isometry2<f32>,
        velocity: Velocity2,
    ) -> Self {
        let poly_line = Polyline::new(verts.clone());
        let vert_count = verts.len();

        Asteroid {
            verts,
            local_center_of_mass: isometry.inverse() * center_of_mass,
            center_of_mass,
            isometry,
            velocity,
            color: Color::random(),
            poly_line,
            transformed_coords_buffer: Vec::with_capacity(vert_count * 2),
        }
    }

    pub fn from_proto(
        asteroid: &ProtoAsteroidEntity,
        center_of_mass: Point2<f32>,
        movement: &MovementUpdate,
    ) -> Self {
        let (pos, velocity) = process_movement(movement);
        let verts = asteroid
            .get_vert_coords()
            .chunks(2)
            .map(|pt| Point2::new(pt[0], pt[1]))
            .collect();

        Asteroid::new(verts, center_of_mass, pos, velocity)
    }
}

impl Shape<f32> for Asteroid {
    fn aabb(&self, m: &Isometry2<f32>) -> AABB<f32> {
        self.poly_line.aabb(m)
    }

    fn as_ray_cast(&self) -> Option<&RayCast<f32>> {
        Some(&self.poly_line)
    }
}

impl Entity for Asteroid {
    fn set_movement(&mut self, movement: &MovementUpdate) {
        let (pos, velocity) = process_movement(movement);
        self.isometry = pos;
        self.velocity = velocity;
    }

    fn render(&self, _cur_tick: usize) {
        fill_poly(&self.color, &self.transformed_coords_buffer);
    }

    fn tick(&mut self, _tick: usize) -> bool {
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

        self.transformed_coords_buffer.clear();
        for vert in &self.verts {
            let transformed_point = self.isometry * vert;
            self.transformed_coords_buffer.push(transformed_point.x);
            self.transformed_coords_buffer.push(transformed_point.y);
        }

        // self.velocity != Velocity2::zero()
        true
    }

    fn apply_update(&mut self, update: &ServerMessageContent) -> bool {
        match &update {
            _ => {
                error("Unexpected server message type received in entity update handler!");
                false
            }
        }
    }

    fn get_bounding_volume(&self) -> AABB<f32> {
        self.poly_line.bounding_volume(&self.isometry)
    }

    fn get_isometry(&self) -> &Isometry2<f32> {
        &self.isometry
    }

    fn get_vertices(&self) -> &[Point2<f32>] {
        &self.verts
    }
}

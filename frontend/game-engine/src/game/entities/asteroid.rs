use nalgebra::geometry::Isometry2;
use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, HasBoundingVolume};
use ncollide2d::query::RayCast;
use ncollide2d::shape::{Polyline, Shape};

use entity::Entity;
use proto_utils::ServerMessageContent;
use protos::server_messages::{AsteroidEntity as ProtoAsteroidEntity, MovementUpdate};
use render_methods::fill_poly;
use util::{log, Color};

pub struct Asteroid {
    pub isometry: Isometry2<f32>,
    pub verts: Vec<Point2<f32>>,
    pub color: Color,
    pub delta_isometry: Isometry2<f32>,
    poly_line: Polyline<f32>,
    transformed_coords_buffer: Vec<f32>,
}

fn process_movement(movement: &MovementUpdate) -> (Isometry2<f32>, Isometry2<f32>) {
    let pos = Isometry2::new(
        Vector2::new(movement.get_pos_x(), movement.get_pos_y()),
        movement.get_rotation(),
    );
    let velocity = Isometry2::new(
        Vector2::new(movement.get_velocity_x(), movement.get_velocity_y()),
        movement.get_angular_velocity(),
    );

    (pos, velocity)
}

impl Asteroid {
    pub fn new(
        verts: Vec<Point2<f32>>,
        isometry: Isometry2<f32>,
        delta_isometry: Isometry2<f32>,
    ) -> Self {
        let poly_line = Polyline::new(verts.clone());
        let vert_count = verts.len();

        Asteroid {
            verts,
            isometry,
            delta_isometry,
            color: Color::random(),
            poly_line,
            transformed_coords_buffer: Vec::with_capacity(vert_count * 2),
        }
    }

    fn set_movement(&mut self, movement: &MovementUpdate) {
        let (pos, velocity) = process_movement(movement);
        self.isometry = pos;
        self.delta_isometry = velocity;
    }

    pub fn from_proto(asteroid: &ProtoAsteroidEntity, movement: &MovementUpdate) -> Self {
        let (pos, velocity) = process_movement(movement);
        log(format!(
            "Creating asteroid with verts: {:?}, movement: {:?}, {:?}",
            asteroid.get_vert_coords(),
            pos,
            velocity
        ));
        Asteroid::new(
            asteroid
                .get_vert_coords()
                .chunks(2)
                .map(|pt| Point2::new(pt[0], pt[1]))
                .collect(),
            pos,
            velocity,
        )
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
    fn render(&self, _cur_tick: usize) {
        fill_poly(&self.color, &self.transformed_coords_buffer);
    }

    fn tick(&mut self, _tick: usize) -> bool {
        self.isometry *= self.delta_isometry;

        self.transformed_coords_buffer.clear();
        for vert in &self.verts {
            let transformed_point = self.isometry * vert;
            self.transformed_coords_buffer.push(transformed_point.x);
            self.transformed_coords_buffer.push(transformed_point.y);
        }

        self.delta_isometry != Isometry2::new(Vector2::new(0., 0.), 0.)
    }

    fn apply_update(&mut self, update: &ServerMessageContent) -> bool {
        match &update {
            &ServerMessageContent::movement_update(movement) => {
                self.set_movement(&movement);
                true
            }
            _ => false,
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

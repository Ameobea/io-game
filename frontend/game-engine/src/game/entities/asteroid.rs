use nalgebra::geometry::Isometry2;
use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, HasBoundingVolume};
use ncollide2d::query::RayCast;
use ncollide2d::shape::{Polyline, Shape};

use entity::Entity;
use protos::server_messages::{
    AsteroidEntity as ProtoAsteroidEntity, ServerMessage_oneof_payload as ServerMessageContent,
};
use render_methods::fill_poly;
use util::Color;

pub struct Asteroid {
    pub isometry: Isometry2<f32>,
    pub verts: Vec<Point2<f32>>,
    pub color: Color,
    pub delta_isometry: Isometry2<f32>,
    poly_line: Polyline<f32>,
    transformed_coords_buffer: Vec<f32>,
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

    pub fn from_proto(asteroid: &ProtoAsteroidEntity, translation: Vector2<f32>) -> Self {
        Asteroid::new(
            asteroid
                .get_vert_coords()
                .chunks(2)
                .map(|pt| Point2::new(pt[0], pt[1]))
                .collect(),
            Isometry2::new(translation, asteroid.get_rotation()),
            Isometry2::new(
                Vector2::new(asteroid.get_velocity_x(), asteroid.get_velocity_y()),
                asteroid.get_delta_rotation(),
            ),
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

    fn apply_update(&mut self, _update: &ServerMessageContent) -> bool {
        false
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

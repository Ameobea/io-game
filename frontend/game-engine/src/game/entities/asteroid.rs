use nalgebra::geometry::Isometry2;
use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::{aabb::AABB, HasBoundingVolume};
use ncollide2d::query::RayCast;
use ncollide2d::shape::{Polyline, Shape};

use super::super::super::fill_poly;
use entity::Entity;
use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::Color;

pub struct Asteroid {
    pub isometry: Isometry2<f32>,
    pub verts: Vec<Point2<f32>>,
    pub color: Color,
    pub delta_isometry: Isometry2<f32>,
    poly_line: Polyline<f32>,
}

impl Asteroid {
    pub fn new(
        verts: Vec<Point2<f32>>,
        isometry: Isometry2<f32>,
        delta_isometry: Isometry2<f32>,
    ) -> Self {
        let poly_line = Polyline::new(verts.clone());

        Asteroid {
            verts,
            isometry,
            delta_isometry,
            color: Color::random(),
            poly_line,
        }
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
    fn render(&self) {
        // TODO: Cache the calculated transformed polyline?
        let coords: Vec<f32> = self
            .verts
            .iter()
            .map(|pt| -> Point2<f32> { self.isometry * pt })
            .flat_map(|pt| vec![pt.x, pt.y])
            .collect();

        fill_poly(self.color.red, self.color.green, self.color.blue, &coords);
    }

    fn tick(&mut self, _tick: usize) -> bool {
        self.isometry *= self.delta_isometry;
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

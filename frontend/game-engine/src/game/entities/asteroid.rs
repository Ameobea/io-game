use nalgebra::geometry::Isometry2;
use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::aabb::{aabb, AABB};
use ncollide2d::shape::{ConvexPolygon, Shape};

use super::super::super::fill_poly;
use entity::Entity;
use protos::server_messages::{
    MovementUpdate, ServerMessage_oneof_payload as ServerMessageContent,
};
use util::Color;

pub struct Asteroid {
    pub isometry: Isometry2<f32>,
    pub verts: Vec<Point2<f32>>,
    pub color: Color,
    pub delta_isometry: Isometry2<f32>,
}

impl Asteroid {
    pub fn new(
        verts: Vec<Point2<f32>>,
        isometry: Isometry2<f32>,
        delta_isometry: Isometry2<f32>,
    ) -> Self {
        Asteroid {
            verts,
            isometry,
            delta_isometry,
            color: Color::random(),
        }
    }
}

impl Entity for Asteroid {
    fn render(&self) {
        // TODO: Cache the calculated transformed isometry
        let coords: Vec<f32> = self
            .verts
            .iter()
            .map(|pt| -> Point2<f32> { self.isometry * pt })
            .flat_map(|pt| vec![pt.x, pt.y])
            .collect();

        fill_poly(self.color.red, self.color.green, self.color.blue, &coords);
    }

    fn tick(&mut self, tick: usize) -> bool {
        self.isometry *= self.delta_isometry;
        // TODO: set up bounding or something to prevent us re-calculating BV every tick
        self.isometry != Isometry2::new(Vector2::new(0., 0.), 0.)
    }

    fn apply_update(&mut self, update: &ServerMessageContent) -> bool {
        false
    }

    fn get_bounding_volume(&self) -> AABB<f32> {
        let convex_poly = ConvexPolygon::try_from_points(&self.verts)
            .expect("Unable to compute convex polygon for asteroid!");
        convex_poly.aabb(&self.isometry)
    }
}

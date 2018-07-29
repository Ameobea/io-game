use nalgebra::geometry::Isometry2;
use nalgebra::{Point2, Vector2};
use ncollide2d::bounding_volume::{
    aabb::{aabb, AABB},
    HasBoundingVolume,
};
use ncollide2d::query::RayCast;
use ncollide2d::shape::{ConvexPolygon, Shape};

use super::super::super::fill_poly;
use entity::Entity;
use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::Color;

pub struct Asteroid {
    pub isometry: Isometry2<f32>,
    pub verts: Vec<Point2<f32>>,
    pub color: Color,
    pub delta_isometry: Isometry2<f32>,
    convex_poly: ConvexPolygon<f32>,
}

impl Asteroid {
    pub fn new(
        verts: Vec<Point2<f32>>,
        isometry: Isometry2<f32>,
        delta_isometry: Isometry2<f32>,
    ) -> Self {
        let convex_poly = ConvexPolygon::try_from_points(&verts)
            .expect("Unable to compute convex polygon for asteroid!");

        Asteroid {
            verts,
            isometry,
            delta_isometry,
            color: Color::random(),
            convex_poly,
        }
    }
}

impl Shape<f32> for Asteroid {
    fn aabb(&self, m: &Isometry2<f32>) -> AABB<f32> {
        let convex_poly = ConvexPolygon::try_from_points(&self.verts)
            .expect("Unable to compute convex polygon for asteroid!");
        convex_poly.aabb(m)
    }

    fn as_ray_cast(&self) -> Option<&RayCast<f32>> {
        Some(&self.convex_poly)
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

    fn tick(&mut self, _tick: usize) -> bool {
        self.isometry *= self.delta_isometry;
        // TODO: set up bounding or something to prevent us re-calculating BV every tick
        let needs_update = self.delta_isometry != Isometry2::new(Vector2::new(0., 0.), 0.);
        if needs_update {
            self.convex_poly = ConvexPolygon::try_from_points(&self.verts)
                .expect("Unable to compute convex polygon for asteroid!");
        }
        needs_update
    }

    fn apply_update(&mut self, update: &ServerMessageContent) -> bool {
        false
    }

    fn get_bounding_volume(&self) -> AABB<f32> {
        self.convex_poly.bounding_volume(&self.isometry)
    }

    fn get_isometry(&self) -> &Isometry2<f32> {
        &self.isometry
    }
}

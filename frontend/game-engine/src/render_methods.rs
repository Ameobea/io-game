use nalgebra::Point2;

use util::Color;

pub use super::clear_canvas;

#[inline(always)]
pub fn render_quad(color: &Color, pos: Point2<f32>, width: u16, height: u16) {
    super::render_quad(
        color.red,
        color.green,
        color.blue,
        pos.x as u16,
        pos.y as u16,
        width,
        height,
    )
}

#[inline(always)]
pub fn render_arc(
    color: &Color,
    pos: Point2<f32>,
    width: u16,
    radius: u16,
    start_angle: f32,
    end_angle: f32,
    counter_clockwise: bool,
) {
    super::render_arc(
        color.red,
        color.green,
        color.blue,
        pos.x as u16,
        pos.y as u16,
        width,
        radius,
        start_angle,
        end_angle,
        counter_clockwise,
    )
}

#[inline(always)]
pub fn render_line(color: &Color, width: u16, p1: Point2<f32>, p2: Point2<f32>) {
    super::render_line(
        color.red,
        color.green,
        color.blue,
        width,
        p1.x as u16,
        p1.y as u16,
        p2.x as u16,
        p2.y as u16,
    )
}

#[inline(always)]
pub fn fill_poly(color: &Color, vertex_coords: &[f32]) {
    super::fill_poly(color.red, color.green, color.blue, vertex_coords)
}

#[inline(always)]
pub fn render_point(color: &Color, pos: Point2<f32>) {
    super::render_point(
        color.red,
        color.green,
        color.blue,
        pos.x as u16,
        pos.y as u16,
    )
}

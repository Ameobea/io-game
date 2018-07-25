use super::super::render_arc;
use render_effects::RenderEffect;
use std::f64;
use util::Color;

pub struct DemoCircleEffect {
    pub color: Color,
    pub width: u16,
    pub x: f32,
    pub y: f32,
    pub cur_size: f32,
    pub max_size: f32,
    pub increment: f32,
}

impl DemoCircleEffect {
    fn render(&self) {
        render_arc(
            self.color.red,
            self.color.green,
            self.color.blue,
            self.x as u16,
            self.y as u16,
            self.width,
            self.cur_size as u16,
            0.,
            2. * f64::consts::PI,
            true,
        );
    }
}

impl RenderEffect for DemoCircleEffect {
    fn tick_and_render(&mut self, _tick: usize) -> bool {
        self.cur_size += self.increment;
        self.render();

        self.cur_size >= self.max_size
    }
}

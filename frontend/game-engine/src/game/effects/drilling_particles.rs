use super::super::super::render_quad;
use nalgebra::{Point2, Vector2};
use render_effects::RenderEffect;
use util::{math_random, Color};

pub struct DrillingParticles {
    pos: Point2<f32>,
    dur_ticks: usize,
    start_tick: usize,
    /// Particle velocity in px/tick
    particle_velocity: f32,
    particle_vectors: Vec<Vector2<f32>>,
    color: Color,
}

impl DrillingParticles {
    pub fn new(
        pos: Point2<f32>,
        cur_tick: usize,
        dur_ticks: usize,
        particle_count: usize,
        particle_velocity: f32,
        color: Color,
    ) -> Self {
        let particle_vectors: Vec<Vector2<f32>> = (0..particle_count)
            .map(|_| -> Vector2<f32> {
                Vector2::new(
                    math_random() as f32 * 2. - 1.,
                    math_random() as f32 * 2. - 1.,
                ).normalize()
            }).collect();

        DrillingParticles {
            pos,
            dur_ticks,
            start_tick: cur_tick,
            particle_velocity,
            particle_vectors,
            color,
        }
    }
}

fn render_drill_particle(pos: Point2<f32>, color: &Color) {
    render_quad(
        color.red,
        color.green,
        color.blue,
        pos.x as u16,
        pos.y as u16,
        2,
        2,
    );
}

impl RenderEffect for DrillingParticles {
    fn tick_and_render(&mut self, cur_tick: usize) -> bool {
        for dir in &self.particle_vectors {
            render_drill_particle(
                self.pos + (self.particle_velocity * (cur_tick - self.start_tick) as f32 * dir),
                &self.color,
            );
        }

        self.start_tick + self.dur_ticks == cur_tick
    }
}

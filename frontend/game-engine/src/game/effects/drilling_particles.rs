use nalgebra::{Point2, Vector2};
use render_effects::RenderEffect;
use render_methods::render_point;
use util::{math_random, Color};

pub struct DrillingParticles {
    pos: Point2<f32>,
    dur_ticks: u32,
    start_tick: u32,
    /// Particle velocity in px/tick
    particle_velocity: f32,
    particle_vectors: Vec<Vector2<f32>>,
    color: Color,
}

impl DrillingParticles {
    pub fn new(
        pos: Point2<f32>,
        cur_tick: u32,
        dur_ticks: u32,
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

impl RenderEffect for DrillingParticles {
    fn tick_and_render(&mut self, cur_tick: u32) -> bool {
        for dir in &self.particle_vectors {
            let adjusted_pos =
                self.pos + (self.particle_velocity * (cur_tick - self.start_tick) as f32 * dir);
            render_point(&self.color, adjusted_pos);
        }

        self.start_tick + self.dur_ticks >= cur_tick
    }
}

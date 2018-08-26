//! Provides support for stateful render effects like particles.

pub trait RenderEffect {
    /// Given the current tick iteration, triggers the effect to render.  If `true` is returned,
    /// the effect will be deleted and never rendered again.
    fn tick_and_render(&mut self, tick: u32) -> bool;
}

pub struct RenderEffectManager {
    effects: Vec<Box<RenderEffect + Send + Sync>>,
}

impl RenderEffectManager {
    pub fn new() -> Self {
        RenderEffectManager {
            effects: Vec::new(),
        }
    }

    pub fn render_all(&mut self, tick: u32) {
        let mut offset = 0;
        for i in 0..self.effects.len() {
            let should_remove = {
                let effect = &mut self.effects[i - offset];
                effect.tick_and_render(tick)
            };

            if should_remove {
                self.effects.swap_remove(i - offset);
                offset += 1;
            }
        }
    }

    pub fn add_effect(&mut self, effect: Box<RenderEffect + Send + Sync>) {
        self.effects.push(effect)
    }
}

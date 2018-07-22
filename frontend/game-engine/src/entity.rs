//! Every "thing" in the game is an entity.  Every entity is renderable, and the game loop runs
//! by looping over all entities and rendering them.

use proto_utils::ServerMessageContent;

pub trait Entity {
    fn render(&self);

    fn tick(&mut self);

    fn apply_update(&mut self, update: &ServerMessageContent);
}

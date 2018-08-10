//! Every "thing" in the game is an entity.  Every entity is renderable, and the game loop runs
//! by looping over all entities and rendering them.

use proto_utils::ServerMessageContent;

pub trait Entity {
    fn render(&self, tick: usize);

    /// Updates this entity's state for one tick.
    fn tick(&mut self, tick: usize);

    /// Updates the entity's state with the data from a message from the server.
    fn apply_update(&mut self, update: &ServerMessageContent);
}

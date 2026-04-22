// engine/src/game/state.rs
//
// GameState — runtime representation of an active game.
// Stored in-memory in AppState; persisted to disk via store/ (TODO).

use serde::{Deserialize, Serialize};

use crate::game::objects::{player::Player, universe::Universe};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// UUID v4 string — the engine's canonical identifier for this game.
    pub id: String,
    pub name: String,
    pub universe: Universe,
    pub players: Vec<Player>,
    pub year: u32,
}

// engine/src/game/state.rs
//
// GameState — runtime representation of an active game.
// Stored in-memory in AppState; persisted to disk via store/.
//
// `GameState` is the host's ground-truth view of the game and must never be
// serialised directly to a non-host client. Per-player turn files are derived
// from `GameState` through a single choke point (see game/view.rs once
// PlayerView lands).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::objects::{planet::PlanetState, player::Player, universe::Universe};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// UUID v4 string — the engine's canonical identifier for this game.
    pub id: String,
    pub name: String,
    /// Static, shared map data (.xy-equivalent).
    pub universe: Universe,
    /// Mutable per-planet state, keyed by planet id. Host-only ground truth;
    /// reach this through PlayerView when serving a per-player turn file.
    pub planet_states: HashMap<u32, PlanetState>,
    pub players: Vec<Player>,
    pub year: u32,
}

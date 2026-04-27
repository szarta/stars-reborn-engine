// engine/src/game/objects/universe.rs
//
// Universe — the static, shared map data (.xy-equivalent).
//
// Holds only data that is safe to expose to every player at /universe:
// boundary dimensions, density bucket, and the static identity/position of
// each planet. Mutable per-game planet state lives in `GameState::planet_states`,
// not here.

use std::collections::HashMap;

use super::planet::PlanetStatic;

/// The static, shared map for a game.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Universe {
    /// Universe size constant (0=Tiny … 4=Huge)
    pub size: u32,
    /// Side length of the square boundary in light years
    pub width: u32,
    pub height: u32,
    /// All planets, keyed by planet id (1-indexed). Identity + position only;
    /// see `GameState::planet_states` for hab/ownership/population/etc.
    pub planets: HashMap<u32, PlanetStatic>,
}

impl Universe {
    /// Return planet count.
    pub fn planet_count(&self) -> usize {
        self.planets.len()
    }

    /// Return a list of all PlanetStatic objects (order not guaranteed).
    pub fn planet_list(&self) -> Vec<&PlanetStatic> {
        self.planets.values().collect()
    }
}

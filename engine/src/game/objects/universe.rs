// engine/src/game/objects/universe.rs
//
// The Universe data model: the full set of space objects for a game.

use std::collections::HashMap;

use super::planet::Planet;

/// A generated universe: boundary dimensions and the full planet set.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Universe {
    /// Universe size constant (0=Tiny … 4=Huge)
    pub size: u32,
    /// Side length of the square boundary in light years
    pub width: u32,
    pub height: u32,
    /// All planets, keyed by planet id (1-indexed)
    pub planets: HashMap<u32, Planet>,
}

impl Universe {
    /// Return planet count.
    pub fn planet_count(&self) -> usize {
        self.planets.len()
    }

    /// Return a list of all Planet objects (order not guaranteed).
    pub fn planet_list(&self) -> Vec<&Planet> {
        self.planets.values().collect()
    }
}

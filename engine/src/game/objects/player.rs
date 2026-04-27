// engine/src/game/objects/player.rs
//
// Player — links a Race to in-game state (tech levels, ship designs,
// score, research allocation, orders).
//
// NPC (computer-controlled) players share the same data model.
//
// See stars-reborn-design/docs/architecture.rst (Game Object Model)

use serde::{Deserialize, Serialize};

use super::race::Race;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechLevels {
    pub energy: u32,
    pub weapons: u32,
    pub propulsion: u32,
    pub construction: u32,
    pub electronics: u32,
    pub biotechnology: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub race_name: String,
    pub homeworld_id: Option<u32>,
    pub tech: TechLevels,
    /// Full race definition. Optional for backwards compatibility — pre-existing
    /// games may have been persisted without this field. New games always set
    /// it; per-player view derivation (planet value, scanner ranges, etc.)
    /// requires it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub race: Option<Race>,
}

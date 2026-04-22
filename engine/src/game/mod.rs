// engine/src/game/mod.rs
//
// All game logic: universe generation, turn processing, fog of war,
// AI order generation, scoring, and victory detection.
//
// The http layer calls into this module and has no game logic of its own.
// The store layer persists and retrieves game state used by this module.

pub mod ai;
pub mod combat;
pub mod objects;
pub mod state;
pub mod turn;
pub mod universe;

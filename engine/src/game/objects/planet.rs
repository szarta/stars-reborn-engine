// engine/src/game/objects/planet.rs
//
// Planet data is split into two types matching the engine's clean separation
// between the static, shared map (.xy-equivalent) and the mutable per-game
// state (.hst-equivalent). See stars-reborn-design/docs/architecture.rst.
//
//   PlanetStatic — identity and position. Lives in `Universe`. Never changes
//                  for the lifetime of a game. Safe to expose at /universe.
//   PlanetState  — everything mutable: hab values (terraforming), mineral
//                  concentrations (degrade as mines extract), surface
//                  minerals, ownership, population, factories, mines, the
//                  homeworld flag. Lives in `GameState`, host-only ground
//                  truth. Per-player views are derived from this; raw state
//                  is never sent to a non-host client.

/// Static planet data — the .xy-equivalent shape.
///
/// Identity and position. Constant for the lifetime of the game.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanetStatic {
    pub id: u32,
    pub name: String,
    /// X coordinate within the universe boundary (light years)
    pub x: i32,
    /// Y coordinate within the universe boundary (light years)
    pub y: i32,
}

impl PlanetStatic {
    /// ``(x, y)`` coordinate tuple.
    pub fn location(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

/// Mutable per-game planet state — host-side ground truth.
///
/// Hab values are stored in the same units as the original game:
///   - gravity:     one of the 101 discrete floating-point values (0.12 – 8.00)
///   - temperature: integer, -200 to +200, step 4
///   - radiation:   integer, 0 – 100
///
/// Mineral concentrations start 1–100 and diminish as mines extract ore.
/// Surface minerals start equal to concentration (Normal-start rule).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanetState {
    pub gravity: f32,
    pub temperature: i32,
    pub radiation: u32,

    pub ironium_concentration: u32,
    pub boranium_concentration: u32,
    pub germanium_concentration: u32,

    pub surface_ironium: u32,
    pub surface_boranium: u32,
    pub surface_germanium: u32,

    pub homeworld: bool,
    pub owner: Option<u32>,
    pub population: u32,
    pub factories: u32,
    pub mines: u32,
}

impl PlanetState {
    /// Override hab values to the centre of the player's race tolerance per
    /// axis. Immune axes use the canonical defaults (1.00 g, 0 °C, 50 mR/yr).
    ///
    /// Per stars-reborn-design/docs/new_game/initial_state.rst: a homeworld's
    /// hab values are set to the midpoint of the race's preferred range on
    /// each axis. The midpoint is taken in **index** space (0–100), then
    /// converted back to physical units so it lands on a discrete game step.
    pub fn set_homeworld_hab(&mut self, race: &super::race::Race) {
        use super::advantage_points::{axis_to_idx, idx_to_grav, idx_to_rad, idx_to_temp};
        self.gravity = if race.hab.gravity.immune {
            1.00
        } else {
            let (lo, hi) = axis_to_idx(&race.hab.gravity, 0);
            idx_to_grav((lo + hi) / 2) as f32
        };
        self.temperature = if race.hab.temperature.immune {
            0
        } else {
            let (lo, hi) = axis_to_idx(&race.hab.temperature, 1);
            idx_to_temp((lo + hi) / 2)
        };
        self.radiation = if race.hab.radiation.immune {
            50
        } else {
            let (lo, hi) = axis_to_idx(&race.hab.radiation, 2);
            idx_to_rad((lo + hi) / 2) as u32
        };
    }
}

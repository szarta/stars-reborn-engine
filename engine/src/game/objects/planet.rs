// engine/src/game/objects/planet.rs
//
// The Planet struct.

/// A single planet in the universe.
///
/// Hab values are stored in the same units as the original game:
///   - gravity:     one of the 101 discrete floating-point values (0.12 – 8.00)
///   - temperature: integer, -200 to +200, step 4
///   - radiation:   integer, 0 – 100
///
/// Mineral concentrations start 1–100 and diminish as mines extract ore.
/// Surface minerals start equal to concentration (Normal-start rule).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Planet {
    pub id: u32,
    pub name: String,
    /// X coordinate within the universe boundary (light years)
    pub x: i32,
    /// Y coordinate within the universe boundary (light years)
    pub y: i32,

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

impl Planet {
    /// ``(x, y)`` coordinate tuple.
    pub fn location(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

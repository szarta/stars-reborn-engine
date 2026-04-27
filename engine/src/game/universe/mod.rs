// engine/src/game/universe/mod.rs
//
// Universe generation — the performance-critical randomisation and placement
// algorithm that produces a playable Stars! universe.
//
// Planet count formula (reverse-engineered from game data):
//     count = floor((dimension / 10)² × density_value / 100)
//
// Validated against actual game data:
//   Tiny  (400 ly) + Normal (2.0) → 32 planets
//   Medium (1200 ly) + Normal (2.0) → 288 planets
//
// See stars-reborn-design/docs/mechanics/universe_generation.rst

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::game::objects::planet::{PlanetState, PlanetStatic};
use crate::game::objects::universe::Universe;

/// The two halves produced by `generate_universe`: the static, shared map and
/// the initial mutable per-planet state. The caller (typically `create_game`)
/// stores the universe inside `GameState::universe` and the state map inside
/// `GameState::planet_states`.
pub struct GeneratedUniverse {
    pub universe: Universe,
    pub planet_states: HashMap<u32, PlanetState>,
}

// ---------------------------------------------------------------------------
// Game constants
// ---------------------------------------------------------------------------

/// Universe side lengths indexed by UniverseSize (0=Tiny … 4=Huge)
const DIMENSIONS: [u32; 5] = [400, 800, 1200, 1600, 2000];

/// Planets-per-(dim/10)²/100 indexed by DensityLevel (0=Sparse … 3=Packed)
const DENSITY: [f32; 4] = [1.5, 2.0, 2.5, 3.75];

/// Minimum inter-planet distance (light years)
const MIN_SPACING: i32 = 20;

/// Maximum placement attempts per planet before falling back to unconstrained
const MAX_ATTEMPTS: u32 = 200;

/// Gravity values matching the original game's Gravity_Map (101 discrete entries, 0.12–8.00).
const GRAVITY_VALUES: [f32; 96] = [
    0.12, 0.13, 0.14, 0.15, 0.16, 0.17, 0.18, 0.19, 0.20, 0.21, 0.22, 0.24, 0.25, 0.27, 0.29, 0.31,
    0.33, 0.36, 0.40, 0.44, 0.50, 0.51, 0.52, 0.53, 0.54, 0.55, 0.56, 0.58, 0.59, 0.60, 0.62, 0.64,
    0.65, 0.67, 0.69, 0.71, 0.73, 0.75, 0.78, 0.80, 0.83, 0.86, 0.89, 0.92, 0.96, 1.00, 1.04, 1.08,
    1.12, 1.16, 1.20, 1.24, 1.28, 1.32, 1.36, 1.40, 1.44, 1.48, 1.52, 1.56, 1.60, 1.64, 1.68, 1.72,
    1.76, 1.80, 1.84, 1.88, 1.92, 1.96, 2.00, 2.24, 2.48, 2.72, 2.96, 3.20, 3.44, 3.68, 3.92, 4.16,
    4.40, 4.64, 4.88, 5.12, 5.36, 5.60, 5.84, 6.08, 6.32, 6.56, 6.80, 7.04, 7.28, 7.52, 7.76, 8.00,
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return the side length in light years for a given UniverseSize index.
pub fn universe_dimension(size: u32) -> u32 {
    DIMENSIONS[size as usize]
}

/// Return the expected planet count for size/density.
pub fn planet_count(size: u32, density: u32) -> u32 {
    let dim = DIMENSIONS[size as usize] as f32;
    let d = DENSITY[density as usize];
    ((dim / 10.0).powi(2) * d / 100.0) as u32
}

/// Generate a full universe.
///
/// Parameters:
///   size        — UniverseSize index: 0=Tiny, 1=Small, 2=Medium, 3=Large, 4=Huge
///   density     — DensityLevel index: 0=Sparse, 1=Normal, 2=Dense, 3=Packed
///   num_players — minimum planet count (guarantees a homeworld slot for each player)
///   seed        — optional RNG seed for reproducible generation
pub fn generate_universe(
    size: u32,
    density: u32,
    num_players: u32,
    seed: Option<u64>,
) -> GeneratedUniverse {
    let mut rng: StdRng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let dim = DIMENSIONS[size as usize];
    let count = planet_count(size, density).max(num_players);

    // Build name pool from embedded list (shuffled)
    let mut names: Vec<String> = PLANET_NAMES.iter().map(|s| s.to_string()).collect();
    shuffle(&mut names, &mut rng);

    // Place planets
    let positions = place_planets(count, dim, dim, &mut rng);

    // Build planets
    let mut planets = HashMap::with_capacity(count as usize);
    let mut planet_states = HashMap::with_capacity(count as usize);
    let mut name_idx = 0usize;
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (i, (x, y)) in positions.into_iter().enumerate() {
        let id = (i + 1) as u32;
        let name = next_name(&names, &mut name_idx, &used);
        used.insert(name.clone());
        let (static_, state) = generate_planet(id, x, y, name, &mut rng);
        planets.insert(id, static_);
        planet_states.insert(id, state);
    }

    GeneratedUniverse {
        universe: Universe {
            size,
            width: dim,
            height: dim,
            planets,
        },
        planet_states,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn generate_planet(
    id: u32,
    x: i32,
    y: i32,
    name: String,
    rng: &mut StdRng,
) -> (PlanetStatic, PlanetState) {
    let grav_idx = rng.gen_range(0..GRAVITY_VALUES.len());
    let gravity = GRAVITY_VALUES[grav_idx];

    // Temperature: -200 to +200 in steps of 4  (101 values)
    let temp_step = rng.gen_range(0i32..=100i32);
    let temperature = -200 + temp_step * 4;

    let radiation = rng.gen_range(0u32..=100);

    let iron_conc = rng.gen_range(1u32..=100);
    let bor_conc = rng.gen_range(1u32..=100);
    let ger_conc = rng.gen_range(1u32..=100);

    let static_ = PlanetStatic { id, name, x, y };
    let state = PlanetState {
        gravity,
        temperature,
        radiation,
        ironium_concentration: iron_conc,
        boranium_concentration: bor_conc,
        germanium_concentration: ger_conc,
        surface_ironium: iron_conc,
        surface_boranium: bor_conc,
        surface_germanium: ger_conc,
        homeworld: false,
        owner: None,
        population: 0,
        factories: 0,
        mines: 0,
    };
    (static_, state)
}

fn place_planets(count: u32, width: u32, height: u32, rng: &mut StdRng) -> Vec<(i32, i32)> {
    let margin = MIN_SPACING;
    let w = width as i32;
    let h = height as i32;
    let mut positions: Vec<(i32, i32)> = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let mut placed = false;
        for _ in 0..MAX_ATTEMPTS {
            let x = rng.gen_range(margin..w - margin);
            let y = rng.gen_range(margin..h - margin);
            if positions.iter().all(|(px, py)| {
                let dx = (x - px) as f32;
                let dy = (y - py) as f32;
                (dx * dx + dy * dy).sqrt() >= MIN_SPACING as f32
            }) {
                positions.push((x, y));
                placed = true;
                break;
            }
        }
        if !placed {
            // Fallback: unconstrained placement
            let x = rng.gen_range(0..w);
            let y = rng.gen_range(0..h);
            positions.push((x, y));
        }
    }
    positions
}

fn next_name(pool: &[String], idx: &mut usize, used: &std::collections::HashSet<String>) -> String {
    while *idx < pool.len() {
        let name = &pool[*idx];
        *idx += 1;
        if !used.contains(name) {
            return name.clone();
        }
    }
    // Pool exhausted — numbered fallback
    let mut n = used.len() as u32 + 1;
    loop {
        let candidate = format!("Planet {n}");
        if !used.contains(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Fisher-Yates shuffle using the provided RNG.
fn shuffle<T>(v: &mut [T], rng: &mut StdRng) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = rng.gen_range(0..=i);
        v.swap(i, j);
    }
}

// ---------------------------------------------------------------------------
// Planet name pool (embedded at compile time)
// ---------------------------------------------------------------------------

/// The planet name pool — the original 999 names from the original game.
/// Loaded from the text file at compile time.
static PLANET_NAMES: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
    include_str!("../../../data/planet_names/original.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
});

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planet_count_tiny_normal() {
        assert_eq!(planet_count(0, 1), 32);
    }

    #[test]
    fn test_planet_count_medium_normal() {
        assert_eq!(planet_count(2, 1), 288);
    }

    #[test]
    fn test_universe_has_correct_planet_count() {
        let g = generate_universe(0, 1, 1, Some(42));
        assert_eq!(g.universe.planets.len(), 32);
        assert_eq!(g.planet_states.len(), 32);
    }

    #[test]
    fn test_universe_reproducible() {
        let g1 = generate_universe(0, 1, 1, Some(99));
        let g2 = generate_universe(0, 1, 1, Some(99));
        let mut p1: Vec<_> = g1.universe.planets.values().map(|p| (p.x, p.y)).collect();
        let mut p2: Vec<_> = g2.universe.planets.values().map(|p| (p.x, p.y)).collect();
        p1.sort();
        p2.sort();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_planet_names_unique() {
        let g = generate_universe(0, 1, 1, Some(7));
        let mut names: Vec<_> = g
            .universe
            .planets
            .values()
            .map(|p| p.name.clone())
            .collect();
        let total = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), total);
    }

    #[test]
    fn test_planet_hab_ranges() {
        let g = generate_universe(0, 2, 1, Some(1));
        for p in g.universe.planets.values() {
            let state = g.planet_states.get(&p.id).expect("state for every planet");
            assert!(GRAVITY_VALUES.contains(&state.gravity));
            assert!((-200..=200).contains(&state.temperature));
            assert!(state.temperature % 4 == 0);
            assert!(state.radiation <= 100);
        }
    }

    #[test]
    fn test_at_least_num_players_planets() {
        let g = generate_universe(0, 0, 50, Some(5)); // Tiny+Sparse normally 24
        assert!(g.universe.planets.len() >= 50);
    }
}

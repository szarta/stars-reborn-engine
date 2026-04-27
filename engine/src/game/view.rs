// engine/src/game/view.rs
//
// PlayerView — the single choke point that projects a player's fog-of-war
// view from the host-side GameState. Every per-player turn file MUST be
// produced by `player_view()`; serialising GameState directly to a non-host
// client is a leak.
//
// Per stars-reborn-design/docs/architecture.rst (Per-Player Visibility),
// each planet appears in one of three states for a given player:
//
//   1. Never observed     — only identity (id, name, x, y) is known.
//   2. Observed, stale    — last-snapshot values + years_since_last_scan > 0.
//   3. Currently scanned  — current ground-truth values; years_since_last_scan = 0.
//
// State (2) is not yet reachable: scanner mechanics and per-player intel
// persistence land in a later commit. For now the visibility rule is
// "fields visible iff the player owns the planet" — the player's homeworld
// (and any later-colonised planets) appear as Observed with years_since_last_scan = 0;
// every other planet is Unobserved.
//
// As scanner mechanics land, the only place that needs updating is the
// `is_observable` predicate inside `player_view()`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game::objects::planet::{PlanetState, PlanetStatic};
use crate::game::state::GameState;

/// A player's view of one planet.
///
/// `Unobserved` carries only identity (always known from .xy data).
/// `Observed` carries the full snapshot at last scan; `years_since_last_scan = 0`
/// indicates current ground-truth values, anything higher is stale.
///
/// Untagged on the wire: clients distinguish by whether contents fields are
/// present.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlanetIntel {
    Observed(PlanetObserved),
    Unobserved(PlanetUnobserved),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetUnobserved {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetObserved {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,

    /// 0 = currently in penetrating-scan range (or owned).
    pub years_since_last_scan: u32,

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
    /// `None` = observed but no current owner (e.g. abandoned colony).
    /// `Some(id)` = observed and owned by player `id`.
    pub owner: Option<u32>,
    pub population: u32,
    pub factories: u32,
    pub mines: u32,
}

/// A player's complete turn-file view.
///
/// Everything in this struct is safe to serialise to player `player_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerView {
    pub year: u32,
    pub game_id: String,
    pub game_name: String,
    pub player_id: u32,
    pub homeworld_id: Option<u32>,
    /// Planets sorted by id for client determinism.
    pub planets: Vec<PlanetIntel>,
}

/// Project a `PlayerView` for `player_id` from the host-side `GameState`.
///
/// This is the single choke point that derives per-player turn files. Any
/// host-only field that escapes through this function is a leak; the
/// integration test in `tests/player_view_leak.rs` is the safety net.
pub fn player_view(state: &GameState, player_id: u32) -> PlayerView {
    let player = state.players.iter().find(|p| p.id == player_id);

    let mut planet_ids: Vec<u32> = state.universe.planets.keys().copied().collect();
    planet_ids.sort();

    let planets: Vec<PlanetIntel> = planet_ids
        .into_iter()
        .filter_map(|id| {
            let static_ = state.universe.planets.get(&id)?;
            let pstate = state.planet_states.get(&id);
            Some(intel_for(static_, pstate, player_id, &state.planet_states))
        })
        .collect();

    PlayerView {
        year: state.year,
        game_id: state.id.clone(),
        game_name: state.name.clone(),
        player_id,
        homeworld_id: player.and_then(|p| p.homeworld_id),
        planets,
    }
}

/// Decide whether `player_id` can currently observe this planet, and emit
/// the matching `PlanetIntel` variant.
///
/// Today's rule: observable iff the player owns the planet. When scanner
/// mechanics land, this is the function that grows: union of fleet/planet
/// scanner coverage, with stale snapshots persisted across turns from a
/// future `PlayerIntel` store on `GameState`.
fn intel_for(
    static_: &PlanetStatic,
    pstate: Option<&PlanetState>,
    player_id: u32,
    _all_states: &HashMap<u32, PlanetState>,
) -> PlanetIntel {
    let observable = matches!(pstate, Some(s) if s.owner == Some(player_id));

    match (observable, pstate) {
        (true, Some(s)) => PlanetIntel::Observed(PlanetObserved {
            id: static_.id,
            name: static_.name.clone(),
            x: static_.x,
            y: static_.y,
            years_since_last_scan: 0,
            gravity: s.gravity,
            temperature: s.temperature,
            radiation: s.radiation,
            ironium_concentration: s.ironium_concentration,
            boranium_concentration: s.boranium_concentration,
            germanium_concentration: s.germanium_concentration,
            surface_ironium: s.surface_ironium,
            surface_boranium: s.surface_boranium,
            surface_germanium: s.surface_germanium,
            homeworld: s.homeworld,
            owner: s.owner,
            population: s.population,
            factories: s.factories,
            mines: s.mines,
        }),
        _ => PlanetIntel::Unobserved(PlanetUnobserved {
            id: static_.id,
            name: static_.name.clone(),
            x: static_.x,
            y: static_.y,
        }),
    }
}

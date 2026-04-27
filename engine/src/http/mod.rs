// engine/src/http/mod.rs
//
// Axum router and request handlers.
//
// This layer is intentionally thin: parse the request, call into game/,
// serialize the response. No game logic lives here.
//
// API surfaces (see stars-reborn-design/docs/architecture.rst):
//
//   GET /model/version                              — engine version + API schema version
//   GET /model/technologies                         — full technology tree
//   GET /model/technologies/{id}                    — single technology item
//   GET /model/race/traits                          — PRT/LRT definitions and point costs
//   GET /model/ships/hulls                          — all hull definitions
//   GET /model/schemas                              — JSON schemas for all request/response types
//
//   POST /games                                     — create new game
//   GET  /games/{game_id}                           — game metadata
//   GET  /games/{game_id}/turns/{year}/status       — per-player submission status
//   GET  /games/{game_id}/turns/{year}/players/{pid}— turn file for player
//   PUT  /games/{game_id}/turns/{year}/orders/{pid} — submit player orders (idempotent)
//   POST /games/{game_id}/turns/{year}/skip/{pid}   — generate AI orders, mark skipped
//   GET  /games/{game_id}/turns/{year}/ai-orders/{pid} — AI-suggested orders (not submitted)

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::game::objects::{
    advantage_points::advantage_points,
    player::{Player, TechLevels},
    race::Race,
};
use crate::game::state::GameState;
use crate::game::universe::generate_universe;
use crate::game::view::player_view;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    /// All active games, keyed by UUID string.
    games: Arc<RwLock<HashMap<String, GameState>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let loaded = crate::store::load_all_games();
        let map: HashMap<String, GameState> =
            loaded.into_iter().map(|g| (g.id.clone(), g)).collect();
        log::info!("{} game(s) loaded from disk", map.len());
        Self {
            games: Arc::new(RwLock::new(map)),
        }
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Build and return the axum Router with all routes registered.
pub fn router(state: AppState) -> Router {
    Router::new()
        // Data model endpoints (read-only, static)
        .route("/model/version", get(model_version))
        .route("/model/technologies", get(model_technologies))
        .route("/model/technologies/:id", get(model_technology_by_id))
        .route("/model/race/traits", get(model_race_traits))
        .route("/model/ships/hulls", get(model_ship_hulls))
        .route("/model/schemas", get(model_schemas))
        // Race design utilities
        .route("/race/validate", post(race_validate))
        // Game endpoints (stateful)
        .route("/games", post(create_game))
        .route("/games/:game_id", get(get_game))
        .route("/games/:game_id/turns/:year/status", get(turn_status))
        .route(
            "/games/:game_id/turns/:year/players/:pid",
            get(get_turn_file),
        )
        .route(
            "/games/:game_id/turns/:year/orders/:pid",
            put(submit_orders),
        )
        .route("/games/:game_id/turns/:year/skip/:pid", post(skip_player))
        .route(
            "/games/:game_id/turns/:year/ai-orders/:pid",
            get(get_ai_orders),
        )
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Data model handlers
// ---------------------------------------------------------------------------

async fn model_version() -> Json<Value> {
    Json(json!({
        "engine_version": crate::VERSION,
        "api_version": 1,
    }))
}

async fn model_technologies() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn model_technology_by_id(Path(id): Path<String>) -> impl IntoResponse {
    let _ = id;
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn model_race_traits() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn model_ship_hulls() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn model_schemas() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

// ---------------------------------------------------------------------------
// Race design handlers
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct RaceValidateResponse {
    advantage_points: i32,
    valid: bool,
}

/// POST /race/validate
async fn race_validate(Json(race): Json<Race>) -> impl IntoResponse {
    let points = advantage_points(&race);
    Json(RaceValidateResponse {
        advantage_points: points,
        valid: points >= 0,
    })
}

// ---------------------------------------------------------------------------
// Request / response types for game creation
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HumanPlayerRequest {
    id: u32,
    #[allow(dead_code)]
    name: Option<String>,
    race: Value, // string (predefined name) or full race object
}

#[derive(Deserialize)]
struct AiPlayerRequest {
    #[allow(dead_code)]
    difficulty: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct UniverseParams {
    size: String,
    density: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct GameParams {
    name: String,
    human_players: Vec<HumanPlayerRequest>,
    ai_players: Option<Vec<AiPlayerRequest>>,
    universe: UniverseParams,
}

#[derive(Deserialize)]
struct CreateGameRequest {
    game: GameParams,
}

#[derive(Serialize)]
struct CreatedGame {
    id: String,
}

#[derive(Serialize)]
struct CreateGameResponse {
    #[serde(rename = "request-is-valid")]
    request_is_valid: bool,
    #[serde(rename = "created-game", skip_serializing_if = "Option::is_none")]
    created_game: Option<CreatedGame>,
    #[serde(rename = "failure-reason", skip_serializing_if = "Option::is_none")]
    failure_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Universe file response types  (.xy equivalent — public, same for all players)
// ---------------------------------------------------------------------------

/// A planet entry in the universe file: position and name only.
/// No owner, population, or player-specific state — those belong in the turn file.
#[derive(Serialize)]
struct UniversePlanet {
    id: u32,
    name: String,
    x: i32,
    y: i32,
}

#[derive(Serialize)]
struct UniverseResponse {
    game_id: String,
    game_name: String,
    year: u32,
    universe_width: u32,
    universe_height: u32,
    planet_count: usize,
    planets: Vec<UniversePlanet>,
}

// ---------------------------------------------------------------------------
// Turn file response types  (.m equivalent — private, per-player)
// ---------------------------------------------------------------------------

// The per-player turn-file response shape lives in `game::view::PlayerView`,
// produced by the single `player_view()` choke point. The handler below
// serialises it directly — there is no http-layer intermediate type.

// ---------------------------------------------------------------------------
// Game handlers
// ---------------------------------------------------------------------------

fn parse_universe_size(s: &str) -> Option<u32> {
    match s {
        "tiny" => Some(0),
        "small" => Some(1),
        "medium" => Some(2),
        "large" => Some(3),
        "huge" => Some(4),
        _ => None,
    }
}

fn parse_density(s: &str) -> Option<u32> {
    match s {
        "sparse" => Some(0),
        "normal" => Some(1),
        "dense" => Some(2),
        "packed" => Some(3),
        _ => None,
    }
}

fn race_name_from_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        obj => obj
            .get("singular-name")
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown")
            .to_string(),
    }
}

/// POST /games — create a new game
async fn create_game(
    State(state): State<AppState>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    let gp = req.game;

    let size_idx = match parse_universe_size(&gp.universe.size) {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid universe size"})),
            )
                .into_response()
        }
    };

    let density_idx = match parse_density(&gp.universe.density) {
        Some(v) => v,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid density"})),
            )
                .into_response()
        }
    };

    let num_human = gp.human_players.len() as u32;
    let num_ai = gp.ai_players.as_ref().map(|v| v.len()).unwrap_or(0) as u32;
    let num_players = num_human + num_ai;

    let generated = generate_universe(size_idx, density_idx, num_players, None);
    let universe = generated.universe;
    let mut planet_states = generated.planet_states;

    // Sort planet ids so homeworld assignment is deterministic
    let mut planet_ids: Vec<u32> = universe.planets.keys().copied().collect();
    planet_ids.sort();

    let mut players: Vec<Player> = Vec::with_capacity(num_players as usize);

    // Assign homeworlds to human players
    for (slot, hp) in gp.human_players.iter().enumerate() {
        let homeworld_id = planet_ids.get(slot).copied();
        if let Some(hw_id) = homeworld_id {
            if let Some(state) = planet_states.get_mut(&hw_id) {
                state.homeworld = true;
                state.owner = Some(hp.id);
                state.population = 25_000;
                state.factories = 10;
                state.mines = 10;
            }
        }
        players.push(Player {
            id: hp.id,
            race_name: race_name_from_value(&hp.race),
            homeworld_id,
            tech: TechLevels::default(),
        });
    }

    // Assign homeworlds to AI players
    let human_count = gp.human_players.len();
    for (ai_slot, _ai) in gp.ai_players.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let slot = human_count + ai_slot;
        let ai_id = slot as u32;
        let homeworld_id = planet_ids.get(slot).copied();
        if let Some(hw_id) = homeworld_id {
            if let Some(state) = planet_states.get_mut(&hw_id) {
                state.homeworld = true;
                state.owner = Some(ai_id);
                state.population = 25_000;
                state.factories = 10;
                state.mines = 10;
            }
        }
        players.push(Player {
            id: ai_id,
            race_name: "AI".to_string(),
            homeworld_id,
            tech: TechLevels::default(),
        });
    }

    let game_id = uuid::Uuid::new_v4().to_string();
    let game = GameState {
        id: game_id.clone(),
        name: gp.name,
        universe,
        planet_states,
        players,
        year: 2400,
    };

    if let Err(e) = crate::store::save_game(&game) {
        log::error!("Failed to persist game {}: {}", game_id, e);
    }

    state.games.write().unwrap().insert(game_id.clone(), game);

    let response = CreateGameResponse {
        request_is_valid: true,
        created_game: Some(CreatedGame { id: game_id }),
        failure_reason: None,
    };
    (StatusCode::CREATED, Json(response)).into_response()
}

/// GET /games/{game_id} — universe file (.xy equivalent)
///
/// Returns the public universe state: all planet positions and names.
/// This resource is the same for every player in the game.
async fn get_game(State(state): State<AppState>, Path(game_id): Path<String>) -> impl IntoResponse {
    let games = state.games.read().unwrap();
    match games.get(&game_id) {
        Some(g) => {
            let mut planets: Vec<UniversePlanet> = g
                .universe
                .planets
                .values()
                .map(|p| UniversePlanet {
                    id: p.id,
                    name: p.name.clone(),
                    x: p.x,
                    y: p.y,
                })
                .collect();
            planets.sort_by_key(|p| p.id);
            let response = UniverseResponse {
                game_id: g.id.clone(),
                game_name: g.name.clone(),
                year: g.year,
                universe_width: g.universe.width,
                universe_height: g.universe.height,
                planet_count: g.universe.planet_count(),
                planets,
            };
            Json(response).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "game not found"})),
        )
            .into_response(),
    }
}

/// GET /games/{game_id}/turns/{year}/status
async fn turn_status(
    State(state): State<AppState>,
    Path((game_id, year)): Path<(String, u32)>,
) -> impl IntoResponse {
    let games = state.games.read().unwrap();
    match games.get(&game_id) {
        Some(g) => {
            let slots: Vec<Value> = g
                .players
                .iter()
                .map(|p| {
                    json!({
                        "slot_index": p.id,
                        "player_type": if p.race_name == "AI" { "ai" } else { "human" },
                        "status": if p.race_name == "AI" { "ready" } else { "pending" },
                    })
                })
                .collect();
            Json(json!({
                "game_id": game_id,
                "year": year,
                "slots": slots,
                "turn_processed": false,
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "game not found"})),
        )
            .into_response(),
    }
}

/// GET /games/{game_id}/turns/{year}/players/{pid} — turn file for player
async fn get_turn_file(
    State(state): State<AppState>,
    Path((game_id, _year, pid)): Path<(String, u32, u32)>,
) -> impl IntoResponse {
    let games = state.games.read().unwrap();
    let game = match games.get(&game_id) {
        Some(g) => g,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "game not found"})),
            )
                .into_response()
        }
    };

    if !game.players.iter().any(|p| p.id == pid) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "player not found"})),
        )
            .into_response();
    }

    let view = player_view(game, pid);
    Json(view).into_response()
}

/// PUT /games/{game_id}/turns/{year}/orders/{pid} — submit player orders
async fn submit_orders(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

/// POST /games/{game_id}/turns/{year}/skip/{pid}
async fn skip_player(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

/// GET /games/{game_id}/turns/{year}/ai-orders/{pid}
async fn get_ai_orders(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

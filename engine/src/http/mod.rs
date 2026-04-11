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

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde_json::{json, Value};

/// Build and return the axum Router with all routes registered.
pub fn router() -> Router {
    Router::new()
        // Data model endpoints (read-only, static)
        .route("/model/version", get(model_version))
        .route("/model/technologies", get(model_technologies))
        .route("/model/technologies/:id", get(model_technology_by_id))
        .route("/model/race/traits", get(model_race_traits))
        .route("/model/ships/hulls", get(model_ship_hulls))
        .route("/model/schemas", get(model_schemas))
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
        .route(
            "/games/:game_id/turns/:year/skip/:pid",
            post(skip_player),
        )
        .route(
            "/games/:game_id/turns/:year/ai-orders/:pid",
            get(get_ai_orders),
        )
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
// Game handlers
// ---------------------------------------------------------------------------

async fn create_game() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn get_game(Path(game_id): Path<String>) -> impl IntoResponse {
    let _ = game_id;
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn turn_status(Path((game_id, year)): Path<(String, u32)>) -> impl IntoResponse {
    let (_, _) = (game_id, year);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn get_turn_file(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn submit_orders(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn skip_player(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

async fn get_ai_orders(Path((game_id, year, pid)): Path<(String, u32, u32)>) -> impl IntoResponse {
    let (_, _, _) = (game_id, year, pid);
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({"error": "not yet implemented"})),
    )
}

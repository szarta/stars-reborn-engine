// engine/src/store/mod.rs
//
// Game state persistence — one sub-directory per game under ENGINE_DATA_DIR.
//
// Layout:
//   {data_dir}/{game_uuid}/game.json   — full serialized GameState
//
// ENGINE_DATA_DIR env var overrides the default location.
// Default: $HOME/.local/share/stars-reborn/games  (Linux XDG convention)
//          ./engine_data                           (fallback if HOME unset)

use std::fs;
use std::path::PathBuf;

use log::{error, info, warn};

use crate::game::state::GameState;

// ---------------------------------------------------------------------------
// Data directory
// ---------------------------------------------------------------------------

pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("ENGINE_DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("stars-reborn")
            .join("games");
    }
    PathBuf::from("./engine_data")
}

// ---------------------------------------------------------------------------
// Save
// ---------------------------------------------------------------------------

pub fn save_game(game: &GameState) -> Result<(), Box<dyn std::error::Error>> {
    let game_dir = data_dir().join(&game.id);
    fs::create_dir_all(&game_dir)?;
    let path = game_dir.join("game.json");
    let json = serde_json::to_string_pretty(game)?;
    fs::write(&path, json)?;
    info!("Saved game {} to {}", game.id, path.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Load
// ---------------------------------------------------------------------------

pub fn load_all_games() -> Vec<GameState> {
    let dir = data_dir();
    if !dir.exists() {
        return vec![];
    }
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("Could not read data dir {}: {}", dir.display(), e);
            return vec![];
        }
    };

    let mut games = Vec::new();
    for entry in entries.flatten() {
        let game_path = entry.path().join("game.json");
        if !game_path.exists() {
            continue;
        }
        match fs::read_to_string(&game_path) {
            Ok(content) => match serde_json::from_str::<GameState>(&content) {
                Ok(game) => {
                    info!("Loaded game {} ({})", game.id, game.name);
                    games.push(game);
                }
                Err(e) => error!("Failed to parse {}: {}", game_path.display(), e),
            },
            Err(e) => error!("Failed to read {}: {}", game_path.display(), e),
        }
    }
    games
}
